use std::borrow::Cow;
use std::net::IpAddr;

use crate::error::AppError;
use crate::model::bo::resource::{ResourceBo, UploadResourceBo};
use crate::model::co::article::{ArticleUnlockBanCo, ArticleUnlockBanCoIdGen};
use crate::model::common::article::{ArticleStatus, SearchArticleSort};
use crate::model::po::article::ArticlePo;
use crate::model::po::article_attachment::ArticleAttachmentPo;
use crate::model::po::article_stats::ArticleStatsPo;
use crate::model::po::article_unlock_attempts::ArticleUnlockAttemptsPo;
use crate::storage::cache::storage::CacheSetMode;
use crate::storage::cache::{Cache, CacheIdGenerator};
use crate::storage::db::DbConn;
use crate::util::pagination::{OptionalPage, Page, PageData};
use crate::util::time::UnixTimestampSecs;

/// 解锁文章
#[derive(Debug, Clone)]
pub struct UnlockArticleBo<'a> {
    /// 文章ID
    pub article_id: Cow<'a, str>,
    /// 文章访问密码
    pub password: Cow<'a, str>,
}

/// 创建文章
#[derive(Debug, Clone)]
pub struct CreateArticleBo {
    /// 标题
    pub title: String,
    /// 存储 Markdown 格式的正文
    pub markdown_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
}

/// 修改文章
#[derive(Debug, Clone)]
pub struct UpdateArticleBo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 存储 Markdown 格式的正文
    pub markdown_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
}

/// 删除文章
#[derive(Debug, Clone)]
pub struct RemoveArticleBo<'a> {
    /// 文章ID
    pub article_id: Cow<'a, str>,
}

/// 搜索文章
#[derive(Debug, Clone)]
pub struct SearchArticleBo<'a> {
    /// 全文搜索
    pub full_text: Option<Cow<'a, str>>,
    /// 状态
    pub status: Option<ArticleStatus>,
    /// 发布时间（大于等于）
    pub published_at_ge: Option<i64>,
    /// 发布时间（小于）
    pub published_at_lt: Option<i64>,
    /// 分页页码
    pub page: Option<u64>,
    /// 分页大小
    pub size: Option<u64>,
    /// 排序方式
    pub sort: Option<SearchArticleSort>,
}

impl SearchArticleBo<'_> {
    pub fn page(&self) -> Result<Page, AppError> {
        OptionalPage::new(self.page, self.size)
            .with_defaults(1, 20)
            .validate(1..1000, [10, 20, 30, 40, 50])
            .map_err(From::from)
    }

    pub fn trim_full_text(&self) -> Option<&str> {
        self.full_text
            .as_deref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
    }
}

/// 文章列表项
#[derive(Debug, Clone)]
pub struct ArticleListItemBo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// 存储 Markdown 格式的正文
    pub markdown_content: String,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 是否需要密码访问
    pub need_password: bool,
}

impl From<ArticlePo> for ArticleListItemBo {
    fn from(article: ArticlePo) -> Self {
        Self {
            article_id: article.id,
            title: article.title,
            excerpt: article.excerpt,
            markdown_content: article.markdown_content,
            status: article.status,
            created_at: article.created_at,
            updated_at: article.updated_at,
            published_at: article.published_at,
            need_password: article.password.is_some(),
        }
    }
}

/// 文章列表项
#[derive(Debug, Clone)]
pub struct ArticleListBo {
    /// 文章列表数据
    pub data: PageData<Vec<ArticleListItemBo>>,
    /// 分页
    pub page: Page,
}

/// 获取文章详情
pub struct GetArticleBo<'a> {
    /// 文章ID
    pub article_id: Cow<'a, str>,
    /// 忽视文章状态条件获取文章
    pub ignore_status: bool,
}

/// 访客可见的文章详情
#[derive(Debug, Clone)]
pub struct VisitorArticleDetailsBo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// 存储 Markdown 格式的正文
    pub markdown_content: String,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 是否需要密码访问
    pub need_password: bool,
    /// 附件列表
    pub attachments: Vec<ArticleAttachmentBo>,
    /// 累计页面访问量
    pub pv: u64,
    /// 累计独立访客数
    pub uv: u64,
}

impl VisitorArticleDetailsBo {
    pub fn from_entities(
        article: ArticlePo,
        attachments: Vec<ArticleAttachmentBo>,
        stats: ArticleStatsPo,
    ) -> Self {
        Self {
            article_id: article.id,
            title: article.title,
            excerpt: article.excerpt,
            markdown_content: article.markdown_content,
            status: article.status,
            created_at: article.created_at,
            updated_at: article.updated_at,
            published_at: article.published_at,
            need_password: article.password.is_some(),
            attachments,
            pv: stats.pv,
            uv: stats.uv,
        }
    }
}

/// 管理员可见的文章详情
#[derive(Debug, Clone)]
pub struct AdminArticleDetailsBo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// 存储 Markdown 格式的正文
    pub markdown_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 是否需要密码访问
    pub need_password: bool,
    /// 附件列表
    pub attachments: Vec<ArticleAttachmentBo>,
    /// 累计页面访问量
    pub pv: u64,
    /// 累计独立访客数
    pub uv: u64,
}

impl AdminArticleDetailsBo {
    pub fn from_entities(
        article: ArticlePo,
        attachments: Vec<ArticleAttachmentBo>,
        stats: ArticleStatsPo,
    ) -> Self {
        Self {
            article_id: article.id,
            title: article.title,
            excerpt: article.excerpt,
            markdown_content: article.markdown_content,
            status: article.status,
            created_at: article.created_at,
            updated_at: article.updated_at,
            published_at: article.published_at,
            need_password: article.password.is_some(),
            password: article.password,
            attachments,
            pv: stats.pv,
            uv: stats.uv,
        }
    }
}

/// 文章详情
#[derive(Debug, Clone)]
pub enum ArticleDetailsBo {
    Visitor(VisitorArticleDetailsBo),
    Admin(AdminArticleDetailsBo),
}

impl From<VisitorArticleDetailsBo> for ArticleDetailsBo {
    fn from(value: VisitorArticleDetailsBo) -> Self {
        Self::Visitor(value)
    }
}

impl From<AdminArticleDetailsBo> for ArticleDetailsBo {
    fn from(value: AdminArticleDetailsBo) -> Self {
        Self::Admin(value)
    }
}

/// 文章附件
#[derive(Debug, Clone)]
pub struct ArticleAttachmentBo {
    /// 附件ID
    pub attachment_id: String,
    /// 文章ID
    pub article_id: String,
    /// 附件名
    pub name: String,
    /// 附件扩展名
    pub extension: String,
    /// 附件大小
    pub size: u64,
    /// 附件类型
    pub mime_type: String,
    /// 附件哈希
    pub sha256: String,
    /// 创建时间
    pub created_at: i64,
}

impl ArticleAttachmentBo {
    pub fn from_entities(attachment: ArticleAttachmentPo, resource: ResourceBo) -> Self {
        Self {
            attachment_id: attachment.id,
            article_id: attachment.article_id,
            name: resource.name,
            extension: resource.extension,
            size: resource.size,
            mime_type: resource.mime_type,
            sha256: resource.sha256,
            created_at: attachment.created_at,
        }
    }
}

/// 上传文章附件
pub struct UploadArticleAttachmentBo<'a> {
    /// 文章ID
    pub article_id: Cow<'a, str>,
    /// 文章附件
    pub attachment: UploadResourceBo,
}

/// 删除文章附件
#[derive(Debug, Clone)]
pub struct RemoveArticleAttachmentBo<'a> {
    /// 文章ID
    pub article_id: Cow<'a, str>,
    /// 附件ID
    pub attachment_id: Cow<'a, str>,
}

/// 下载文章附件
#[derive(Debug, Clone)]
pub struct DownloadArticleAttachmentBo<'a> {
    /// 文章ID
    pub article_id: Cow<'a, str>,
    /// 附件ID
    pub attachment_id: Cow<'a, str>,
}

/// 文章
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArticleBo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// 存储 Markdown 格式的正文
    pub markdown_content: String,
    /// 清理标签、格式后的纯文本
    pub plain_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
}

impl From<ArticlePo> for ArticleBo {
    fn from(value: ArticlePo) -> Self {
        Self {
            article_id: value.id,
            title: value.title,
            excerpt: value.excerpt,
            markdown_content: value.markdown_content,
            plain_content: value.plain_content,
            password: value.password,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            published_at: value.published_at,
        }
    }
}

#[derive(Debug)]
pub struct ArticleUnlockBanBo;

impl ArticleUnlockBanBo {
    pub async fn ban(ip: IpAddr, article_id: &str) -> Result<(), AppError> {
        let id = ArticleUnlockBanCoIdGen { ip, article_id };
        let co = Cache::builder(ArticleUnlockBanCo)
            .id(&id)
            .ttl(crate::config::get().article.unlock_ban_ttl)
            .build()?;
        co.set(CacheSetMode::Overwrite).await?;
        Ok(())
    }

    pub async fn is_banned(ip: IpAddr, article_id: &str) -> Result<bool, AppError> {
        let id = ArticleUnlockBanCoIdGen { ip, article_id };
        Cache::<ArticleUnlockBanCo>::exists(id.generate_id().as_ref())
            .await
            .map_err(From::from)
    }

    pub async fn record_failed(
        ip: IpAddr,
        article_id: impl Into<String>,
        db: &mut DbConn,
    ) -> Result<u32, AppError> {
        let now = UnixTimestampSecs::now();
        let po = ArticleUnlockAttemptsPo {
            ip: ip.to_string(),
            article_id: article_id.into(),
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

        crate::storage::db::article_unlock_attempts::incr_count(&po, db)
            .await
            .map_err(From::from)
    }

    pub async fn record_failed_with_ban(
        ip: IpAddr,
        article_id: impl Into<String>,
        db: &mut DbConn,
    ) -> Result<(u32, bool), AppError> {
        let article_id = article_id.into();
        let count = Self::record_failed(ip, &article_id, db).await?;

        // 计算剩余次数
        let remaining_times = crate::config::get()
            .article
            .unlock_try_max_times
            .saturating_sub(count);

        if remaining_times == 0 {
            ArticleUnlockBanBo::ban(ip, &article_id).await?;
            Ok((remaining_times, true))
        } else {
            Ok((remaining_times, false))
        }
    }
}
