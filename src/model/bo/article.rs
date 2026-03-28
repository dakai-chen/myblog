use std::borrow::Cow;

use crate::error::AppError;
use crate::model::bo::resource::{ResourceBo, UploadResourceBo};
use crate::model::common::article::{ArticleStatus, SearchArticleSort};
use crate::model::po::article::ArticlePo;
use crate::model::po::article_attachment::ArticleAttachmentPo;
use crate::model::po::article_stats::ArticleStatsPo;
use crate::util::pagination::{OptionalPage, Page, PageData};

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
    /// Markdown 格式的正文
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
    /// Markdown 格式的正文
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
        let config = &crate::config::get().article.pagination;
        OptionalPage::new(self.page, self.size)
            .with_defaults(1, config.default_page_size)
            .validate(1..=config.max_page_number, &*config.allowed_page_sizes)
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
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 渲染后的 HTML 结果
    pub render_content: String,
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
            render_content: article.render_content,
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
}

/// 访客可见的文章详情
#[derive(Debug, Clone)]
pub struct VisitorArticleDetailBo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 渲染后的 HTML 结果
    pub render_content: String,
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

impl VisitorArticleDetailBo {
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
            render_content: article.render_content,
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
pub struct AdminArticleDetailBo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 渲染后的 HTML 结果
    pub render_content: String,
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

impl AdminArticleDetailBo {
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
            render_content: article.render_content,
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
pub enum ArticleDetailBo {
    Visitor(VisitorArticleDetailBo),
    Admin(AdminArticleDetailBo),
}

impl From<VisitorArticleDetailBo> for ArticleDetailBo {
    fn from(value: VisitorArticleDetailBo) -> Self {
        Self::Visitor(value)
    }
}

impl From<AdminArticleDetailBo> for ArticleDetailBo {
    fn from(value: AdminArticleDetailBo) -> Self {
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
    /// 附件ID
    pub attachment_id: Cow<'a, str>,
}

/// 下载文章附件
#[derive(Debug, Clone)]
pub struct DownloadArticleAttachmentBo<'a> {
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
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 清理标签、格式后的纯文本
    pub plain_content: String,
    /// 渲染后的 HTML 结果
    pub render_content: String,
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
            render_content: value.render_content,
            password: value.password,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            published_at: value.published_at,
        }
    }
}
