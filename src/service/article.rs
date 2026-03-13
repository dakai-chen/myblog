use std::borrow::Cow;
use std::net::IpAddr;
use std::sync::LazyLock;

use regex::Regex;

use crate::error::{AppError, AppErrorMeta};
use crate::model::bo::article::{
    AdminArticleDetailsBo, ArticleAttachmentBo, ArticleBo, ArticleDetailsBo, ArticleListBo,
    ArticleListItemBo, CreateArticleBo, DownloadArticleAttachmentBo, GetArticleBo,
    RemoveArticleAttachmentBo, RemoveArticleBo, SearchArticleBo, UnlockArticleBo, UpdateArticleBo,
    UploadArticleAttachmentBo, VisitorArticleDetailsBo,
};
use crate::model::bo::auth::AdminBo;
use crate::model::bo::resource::{RemoveResourceBo, UploadResourceOptionsBo};
use crate::model::bo::visitor::VisitorBo;
use crate::model::co::article::{
    ArticleUnlockBanCo, ArticleUnlockBanCoIdGen, VisitorArticleAccessRecordCo,
    VisitorArticleAccessRecordCoIdGen,
};
use crate::model::common::article::{ArticleStatus, SearchArticleSort};
use crate::model::po::article::{ArticlePo, SearchArticle};
use crate::model::po::article_attachment::ArticleAttachmentPo;
use crate::model::po::article_stats::ArticleStatsPo;
use crate::model::po::article_unlock_attempts::ArticleUnlockAttemptsPo;
use crate::model::po::resource::ResourcePo;
use crate::storage::cache::storage::CacheSetMode;
use crate::storage::cache::{Cache, CacheIdGenerator};
use crate::storage::db::DbConn;
use crate::util::join::HashJoin;
use crate::util::pagination::PageData;
use crate::util::time::UnixTimestampSecs;

async fn unlock_ban(ip: IpAddr, article_id: &str) -> Result<(), AppError> {
    let id = ArticleUnlockBanCoIdGen { ip, article_id };
    let co = Cache::builder(ArticleUnlockBanCo)
        .id(&id)
        .ttl(crate::config::get().article.unlock_ban_ttl)
        .build()?;
    co.set(CacheSetMode::Overwrite).await?;
    Ok(())
}

async fn unlock_ban_exists(ip: IpAddr, article_id: &str) -> Result<bool, AppError> {
    let id = ArticleUnlockBanCoIdGen { ip, article_id };
    Cache::<ArticleUnlockBanCo>::exists(id.generate_id().as_ref())
        .await
        .map_err(From::from)
}

/// 解锁文章（获取访问令牌）
pub async fn unlock_article(
    visitor: &VisitorBo,
    bo: &UnlockArticleBo<'_>,
    db: &mut DbConn,
) -> Result<(), AppError> {
    let Some(article) = crate::storage::db::article::find(&bo.article_id, db).await? else {
        return Err(AppErrorMeta::NotFound.with_message("文章不存在"));
    };
    let Some(password) = article.password else {
        return Err(AppErrorMeta::BadRequest.with_message("该文章无需密码进行访问"));
    };

    if unlock_ban_exists(visitor.ip(), &bo.article_id).await? {
        return Err(AppErrorMeta::BadRequest.with_message("文章解锁尝试次数过多，请稍后再试"));
    }

    if bo.password != password {
        let now = UnixTimestampSecs::now();
        let po = ArticleUnlockAttemptsPo {
            ip: visitor.ip().to_string(),
            article_id: article.id,
            count: 1,
            created_at: now.as_i64(),
            expires_at: now
                .saturating_add(crate::config::get().article.unlock_try_window)
                .as_i64(),
        };
        crate::storage::db::article_unlock_attempts::remove_single_expired(
            &po.ip,
            &po.article_id,
            db,
        )
        .await?;
        let count = crate::storage::db::article_unlock_attempts::incr_count(&po, db).await?;

        // 计算剩余次数
        let remaining_times = crate::config::get()
            .article
            .unlock_try_max_times
            .saturating_sub(count);

        if remaining_times == 0 {
            unlock_ban(visitor.ip(), &bo.article_id).await?;
            return Err(AppErrorMeta::BadRequest.with_message("文章解锁尝试次数过多，请稍后再试"));
        } else {
            return Err(AppErrorMeta::BadRequest.with_message(format!(
                "文章访问密码错误！剩余尝试次数 {remaining_times} 次"
            )));
        }
    }
    visitor.add_article(&article.id).await?;
    Ok(())
}

/// 创建文章
pub async fn create_article(bo: CreateArticleBo, db: &mut DbConn) -> Result<ArticleBo, AppError> {
    crate::storage::db::transaction(db, async |tx| {
        let plain_content = clean_markdown_content(&bo.markdown_content);
        let now = UnixTimestampSecs::now().as_i64();
        let article = ArticlePo {
            id: crate::util::uuid::v4(),
            title: bo.title,
            excerpt: truncate_excerpt(&plain_content),
            markdown_content: bo.markdown_content,
            plain_content,
            password: bo.password,
            status: bo.status,
            created_at: now,
            updated_at: now,
            published_at: match bo.status {
                ArticleStatus::Draft => None,
                ArticleStatus::Private => Some(now),
                ArticleStatus::Published => Some(now),
            },
        };
        let stats = ArticleStatsPo {
            id: crate::util::uuid::v4(),
            article_id: article.id.clone(),
            pv: 0,
            uv: 0,
        };
        crate::storage::db::article::create(&article, tx).await?;
        crate::storage::db::article_stats::create(&stats, tx).await?;
        Ok(ArticleBo::from(article))
    })
    .await?
}

/// 修改文章
pub async fn update_article(bo: UpdateArticleBo, db: &mut DbConn) -> Result<(), AppError> {
    let Some(mut article) = crate::storage::db::article::find(&bo.article_id, db).await? else {
        return Err(AppErrorMeta::NotFound.with_message("文章不存在，无法编辑文章"));
    };

    let now = UnixTimestampSecs::now().as_i64();

    let content_changed = article.markdown_content != bo.markdown_content;
    let title_changed = article.title != bo.title;

    if content_changed {
        article.plain_content = clean_markdown_content(&bo.markdown_content);
        article.excerpt = truncate_excerpt(&article.plain_content);
    }

    if content_changed || title_changed {
        article.updated_at = now;
    }

    article.title = bo.title;
    article.markdown_content = bo.markdown_content;
    article.password = bo.password;
    article.status = bo.status;
    article.published_at = match bo.status {
        ArticleStatus::Draft => None,
        ArticleStatus::Private | ArticleStatus::Published => {
            Some(article.published_at.unwrap_or(now))
        }
    };

    // 若发布时间存在，且更新时间早于发布时间，则将更新时间同步为发布时间
    if let Some(published_at) = article.published_at
        && article.updated_at < published_at
    {
        article.updated_at = published_at;
    }

    crate::storage::db::article::update(&article, db).await?;
    Ok(())
}

/// 删除文章
pub async fn remove_article(bo: &RemoveArticleBo<'_>, db: &mut DbConn) -> Result<(), AppError> {
    crate::storage::db::transaction(db, async |tx| {
        let Some(article) = crate::storage::db::article::find(&bo.article_id, tx).await? else {
            return Ok(());
        };
        let attachments =
            crate::storage::db::article_attachment::list_by_article_id(&article.id, tx).await?;
        crate::storage::db::article::remove(&article.id, tx).await?;
        crate::storage::db::article_attachment::remove_by_article_id(&article.id, tx).await?;

        // 清理文章附件对应的资源文件。文章附件对应的资源文件不会在其他地方复用，所以删除不会引起
        // 其他内容的错误。
        for attachment in attachments {
            let remove_resource = RemoveResourceBo {
                resource_id: attachment.resource_id.as_str().into(),
            };
            crate::service::resource::remove_resource(&remove_resource, tx).await?;
        }
        Ok(())
    })
    .await?
}

/// 搜索文章
pub async fn search_article(
    admin: Option<&AdminBo>,
    bo: &SearchArticleBo<'_>,
    db: &mut DbConn,
) -> Result<ArticleListBo, AppError> {
    let page = bo.page()?;
    let offset = page.to_offset()?;

    let full_text = bo.trim_full_text();
    let params = SearchArticle {
        full_text: full_text.map(Cow::from),
        status: if admin.is_some() {
            bo.status
        } else {
            Some(ArticleStatus::Published)
        },
        published_at_ge: bo.published_at_ge,
        published_at_lt: bo.published_at_lt,
        need_password: if full_text.is_some() && admin.is_none() {
            Some(false)
        } else {
            None
        },
        sort: bo.sort.unwrap_or(SearchArticleSort::ByPublishedAtDesc),
    };

    let items = crate::storage::db::article::search(&params, offset, db).await?;
    let total = crate::storage::db::article::search_count(&params, db).await?;

    let items = items.into_iter().map(ArticleListItemBo::from).collect();

    Ok(ArticleListBo {
        data: PageData::from_vec(items)?.with_total(total),
        page,
    })
}

/// 获取文章详情
pub async fn get_article(
    admin: Option<&AdminBo>,
    visitor: &VisitorBo,
    bo: &GetArticleBo<'_>,
    db: &mut DbConn,
) -> Result<Option<ArticleDetailsBo>, AppError> {
    let Some(article) = crate::storage::db::article::find(&bo.article_id, db).await? else {
        return Ok(None);
    };
    if admin.is_none() && !bo.ignore_status && article.status != ArticleStatus::Published {
        return Ok(None);
    };
    if admin.is_none() && article.password.is_some() && !visitor.has_article(&article.id).await? {
        return Err(AppErrorMeta::ArticleLocked {
            article_id: article.id,
            title: article.title,
        }
        .into_error());
    }

    let attachments = list_attachment(&article.id, db).await?;

    if admin.is_none() {
        if let Err(e) = update_article_visit_stats(&article, visitor, db).await {
            tracing::error!("更新文章访问记录信息失败：{e}");
        }
    }
    let Some(stats) =
        crate::storage::db::article_stats::find_by_article_id(&article.id, db).await?
    else {
        return Err(AppErrorMeta::Internal.with_context(format!(
            "获取文章统计信息失败，文章不存在对应的统计记录，文章ID: {}",
            article.id
        )));
    };

    if admin.is_some() {
        let details = AdminArticleDetailsBo::from_entities(article, attachments, stats);
        Ok(Some(ArticleDetailsBo::from(details)))
    } else {
        let details = VisitorArticleDetailsBo::from_entities(article, attachments, stats);
        Ok(Some(ArticleDetailsBo::from(details)))
    }
}

/// 上传文章附件
pub async fn upload_attachment(
    bo: UploadArticleAttachmentBo<'_>,
    db: &mut DbConn,
) -> Result<ArticleAttachmentBo, AppError> {
    crate::storage::db::transaction(db, async |tx| {
        let Some(article) = crate::storage::db::article::find(&bo.article_id, tx).await? else {
            return Err(AppErrorMeta::NotFound.with_message("文章不存在，无法上传附件"));
        };

        let attachment = ArticleAttachmentPo {
            id: crate::util::uuid::v4(),
            article_id: article.id,
            resource_id: crate::util::uuid::v4(),
            created_at: UnixTimestampSecs::now().as_i64(),
        };
        crate::storage::db::article_attachment::create(&attachment, tx).await?;

        let options = UploadResourceOptionsBo {
            resource_id: attachment.resource_id.clone(),
            is_public: false,
        };
        let resource =
            crate::service::resource::upload_resource_with_options(bo.attachment, options, tx)
                .await?;

        Ok(ArticleAttachmentBo::from_entities(attachment, resource))
    })
    .await?
}

/// 删除文章附件
pub async fn remove_attachment(
    bo: &RemoveArticleAttachmentBo<'_>,
    db: &mut DbConn,
) -> Result<(), AppError> {
    crate::storage::db::transaction(db, async |tx| {
        let Some(attachment) =
            crate::storage::db::article_attachment::find(&bo.attachment_id, tx).await?
        else {
            return Ok(());
        };
        if attachment.article_id != bo.article_id {
            return Err(AppErrorMeta::BadRequest.with_message("附件不属于指定文章"));
        }
        crate::storage::db::article_attachment::remove(&attachment.id, tx).await?;
        let remove_resource = RemoveResourceBo {
            resource_id: attachment.resource_id.as_str().into(),
        };
        crate::service::resource::remove_resource(&remove_resource, tx).await?;
        Ok(())
    })
    .await?
}

/// 下载文章附件
pub async fn download_attachment(
    admin: Option<&AdminBo>,
    visitor: &VisitorBo,
    bo: &DownloadArticleAttachmentBo<'_>,
    db: &mut DbConn,
) -> Result<Option<ResourcePo>, AppError> {
    let Some(attachment) =
        crate::storage::db::article_attachment::find(&bo.attachment_id, db).await?
    else {
        return Ok(None);
    };
    if attachment.article_id != bo.article_id {
        return Ok(None);
    }
    let Some(article) = crate::storage::db::article::find(&bo.article_id, db).await? else {
        return Err(AppErrorMeta::Internal
            .with_message("附件关联的文章不存在")
            .with_context(format!(
                "附件ID: {}, 缺失的文章ID: {}",
                attachment.id, attachment.article_id
            )));
    };
    if admin.is_none() && article.password.is_some() && !visitor.has_article(&article.id).await? {
        return Err(AppErrorMeta::ArticleLocked {
            article_id: article.id,
            title: article.title,
        }
        .into_error());
    }
    let Some(resource) = crate::storage::db::resource::find(&attachment.resource_id, db).await?
    else {
        return Err(AppErrorMeta::Internal
            .with_message("附件关联的资源不存在")
            .with_context(format!(
                "文章ID: {}, 附件ID: {}, 缺失的资源ID: {}",
                attachment.article_id, attachment.id, attachment.resource_id
            )));
    };
    Ok(Some(resource))
}

async fn list_attachment(
    article_id: &str,
    db: &mut DbConn,
) -> Result<Vec<ArticleAttachmentBo>, AppError> {
    let mut attachments =
        crate::storage::db::article_attachment::list_by_article_id(article_id, db).await?;

    attachments.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let resources = crate::storage::db::resource::list_by_ids(
        &attachments
            .iter()
            .map(|a| a.resource_id.as_str())
            .collect::<Vec<_>>(),
        db,
    )
    .await?;

    let mut attachments = HashJoin::new()
        .l_source(attachments)
        .r_source(resources)
        .l_key_extractor(|attachment| Ok(attachment.resource_id.as_str().into()))
        .r_key_extractor(|resource| Ok(resource.id.as_str().into()))
        .unique_join(|attachment, resource| {
            // 附件和资源之间互相唯一对应，所以这里可以直接取出 ResourcePo 对象
            let Some(resource) = resource.take() else {
                return Err(AppErrorMeta::Internal
                    .with_message("附件关联的资源不存在")
                    .with_context(format!(
                        "文章ID: {}, 附件ID: {}, 缺失的资源ID: {}",
                        article_id, attachment.id, attachment.resource_id
                    )));
            };
            Ok(Some(ArticleAttachmentBo::from_entities(
                attachment,
                resource.into(),
            )))
        })?;

    attachments.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.attachment_id.cmp(&b.attachment_id))
    });
    Ok(attachments)
}

async fn update_article_visit_stats(
    article: &ArticlePo,
    visitor: &VisitorBo,
    db: &mut DbConn,
) -> Result<(), AppError> {
    let cache_id = VisitorArticleAccessRecordCoIdGen {
        visitor_id: visitor.visitor_id(),
        article_id: article.id.as_str(),
    };
    let cache = Cache::builder(VisitorArticleAccessRecordCo)
        .id(&cache_id)
        .expires_at(UnixTimestampSecs::now().tomorrow_start_local()?.as_i64())
        .build()?;
    let uv_add = match cache.set(CacheSetMode::OnlyIfNotExists).await? {
        true => 1,
        false => 0,
    };
    crate::storage::db::article_stats::increment_by_article_id(&article.id, 1, uv_add, db).await?;
    Ok(())
}

fn clean_markdown_content(markdown: &str) -> String {
    // HTML 标签
    static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<[^>]+>"#).unwrap());
    // 多语言文字、数字、中文标点、空格以外的特殊符号
    static SPECIAL_SYMBOLS_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"[^\p{L}\p{N}，。！？：；\s]"#).unwrap());
    // 空白符
    static WHITESPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

    let plain = HTML_TAG_RE.replace_all(markdown, " ");
    let plain = SPECIAL_SYMBOLS_RE.replace_all(plain.as_ref(), " ");
    let plain = WHITESPACE_RE.replace_all(plain.as_ref(), " ");

    plain.trim().to_owned()
}

fn truncate_excerpt(plain_content: &str) -> String {
    plain_content
        .chars()
        .take(crate::config::get().article.excerpt_max_length)
        .collect()
}
