use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::error::{AppError, AppErrorMeta};
use crate::model::bo::article::{
    ArticleAttachmentBo, ArticleDetailBo, ArticleListBo, ArticleListItemBo,
};
use crate::model::common::article::ArticleStatus;
use crate::model::dto::web::article::SearchArticleDto;
use crate::template::render::TemplateRenderData;
use crate::util::pagination::PageNavigation;

/// 文章列表项
#[derive(Debug, Clone, Serialize)]
pub struct ArticleListItemVo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
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

impl From<ArticleListItemBo> for ArticleListItemVo {
    fn from(value: ArticleListItemBo) -> Self {
        Self {
            article_id: value.article_id,
            title: value.title,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            published_at: value.published_at,
            need_password: value.need_password,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageNumberLinkVo {
    /// 页码
    pub num: u64,
    /// 页码跳转链接
    pub link: String,
    /// 是否为当前选中的页码
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageLinksVo {
    /// 开始页
    pub head: String,
    /// 上一页
    pub prev: Option<String>,
    /// 下一页
    pub next: Option<String>,
    /// 结束页
    pub tail: Option<String>,
    /// 页码表
    pub list: Vec<PageNumberLinkVo>,
}

impl PageLinksVo {
    pub fn from(page_navigation: &PageNavigation, search: &SearchArticleDto) -> Self {
        let a = |page| Self::build_link("/articles", page, search);
        let b = |page| PageNumberLinkVo {
            num: page,
            link: a(page),
            is_current: page == page_navigation.current_page,
        };
        Self {
            head: a(1),
            prev: page_navigation.prev.map(a),
            next: page_navigation.next.map(a),
            tail: page_navigation.tail.map(a),
            list: page_navigation.list.iter().copied().map(b).collect(),
        }
    }

    fn build_link(base_url: &str, page: u64, search: &SearchArticleDto) -> String {
        let mut params = Vec::with_capacity(3);

        if let Some(q) = &search.q {
            if !q.is_empty() {
                params.push(format!("q={}", urlencoding::encode(q)));
            }
        }
        if let Some(size) = search.size {
            params.push(format!("size={size}"));
        }
        if page != 1 {
            params.push(format!("page={page}"));
        }

        if params.is_empty() {
            format!("{base_url}")
        } else {
            format!("{base_url}?{}", params.join("&"))
        }
    }
}

/// 文章列表页面
#[derive(Debug, Clone, Serialize)]
pub struct ArticleListVo {
    /// 文章列表数据
    pub items: Vec<ArticleListItemVo>,
    /// 分页链接
    pub page_links: PageLinksVo,
    /// 分页导航栏
    pub page_navigation: PageNavigation,
    /// 文章搜索条件
    pub search: SearchArticleDto,
}

impl ArticleListVo {
    pub fn from(bo: ArticleListBo, search: SearchArticleDto) -> Result<Self, AppError> {
        let config = &crate::config::get().article.pagination;
        let page_navigation = PageNavigation::new(
            &bo.data,
            bo.page,
            config.page_nav_max_visible,
            config.max_page_number,
        )?;
        Ok(ArticleListVo {
            items: bo
                .data
                .items
                .into_iter()
                .map(ArticleListItemVo::from)
                .collect(),
            page_links: PageLinksVo::from(&page_navigation, &search),
            page_navigation,
            search,
        })
    }
}

impl TemplateRenderData for ArticleListVo {
    fn template_name() -> &'static str {
        "article/list.html"
    }
}

/// 文章附件页面
#[derive(Debug, Clone)]
pub struct ArticleAttachmentVo {
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

impl From<ArticleAttachmentBo> for ArticleAttachmentVo {
    fn from(value: ArticleAttachmentBo) -> Self {
        Self {
            attachment_id: value.attachment_id,
            article_id: value.article_id,
            name: value.name,
            extension: value.extension,
            size: value.size,
            mime_type: value.mime_type,
            sha256: value.sha256,
            created_at: value.created_at,
        }
    }
}

impl Serialize for ArticleAttachmentVo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArticleAttachmentVo", 9)?;

        state.serialize_field("attachment_id", &self.attachment_id)?;
        state.serialize_field("article_id", &self.article_id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("extension", &self.extension)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("mime_type", &self.mime_type)?;
        state.serialize_field("sha256", &self.sha256)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field(
            "url",
            &format!(
                "/articles/{}/attachments/{}",
                self.article_id, self.attachment_id
            ),
        )?;

        state.end()
    }
}

/// 文章详情页面
#[derive(Debug, Clone, Serialize)]
pub struct ArticleDetailVo {
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
    pub attachments: Vec<ArticleAttachmentVo>,
    /// 累计页面访问量
    pub pv: u64,
    /// 累计独立访客数
    pub uv: u64,
}

impl From<ArticleDetailBo> for ArticleDetailVo {
    fn from(value: ArticleDetailBo) -> Self {
        match value {
            ArticleDetailBo::Visitor(bo) => Self {
                article_id: bo.article_id,
                title: bo.title,
                excerpt: bo.excerpt,
                markdown_content: bo.markdown_content,
                render_content: bo.render_content,
                password: None,
                status: bo.status,
                created_at: bo.created_at,
                updated_at: bo.updated_at,
                published_at: bo.published_at,
                need_password: bo.need_password,
                attachments: bo
                    .attachments
                    .into_iter()
                    .map(ArticleAttachmentVo::from)
                    .collect(),
                pv: bo.pv,
                uv: bo.uv,
            },
            ArticleDetailBo::Admin(bo) => Self {
                article_id: bo.article_id,
                title: bo.title,
                excerpt: bo.excerpt,
                markdown_content: bo.markdown_content,
                render_content: bo.render_content,
                password: bo.password,
                status: bo.status,
                created_at: bo.created_at,
                updated_at: bo.updated_at,
                published_at: bo.published_at,
                need_password: bo.need_password,
                attachments: bo
                    .attachments
                    .into_iter()
                    .map(ArticleAttachmentVo::from)
                    .collect(),
                pv: bo.pv,
                uv: bo.uv,
            },
        }
    }
}

impl TemplateRenderData for ArticleDetailVo {
    fn template_name() -> &'static str {
        "article/detail.html"
    }
}

/// 解锁文章页面
#[derive(Debug, Clone, Serialize)]
pub struct UnlockArticleVo<'a> {
    /// 文章ID
    pub article_id: &'a str,
    /// 标题
    pub title: &'a str,
}

impl TemplateRenderData for UnlockArticleVo<'_> {
    fn template_name() -> &'static str {
        "article/unlock.html"
    }
}

/// 创建文章页面
#[derive(Debug, Clone, Serialize)]
pub struct CreateArticleVo;

impl TemplateRenderData for CreateArticleVo {
    fn template_name() -> &'static str {
        "article/create.html"
    }
}

/// 创建文章页面
#[derive(Debug, Clone, Serialize)]
pub struct UpdateArticleVo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 访问密码
    pub password: Option<String>,
    /// 附件列表
    pub attachments: Vec<ArticleAttachmentVo>,
    /// 累计页面访问量
    pub pv: u64,
    /// 累计独立访客数
    pub uv: u64,
}

impl TryFrom<ArticleDetailBo> for UpdateArticleVo {
    type Error = AppError;

    fn try_from(value: ArticleDetailBo) -> Result<Self, Self::Error> {
        match value {
            ArticleDetailBo::Visitor(_) => Err(AppErrorMeta::Internal
                .with_context("无法将 ArticleDetailBo::Visitor 转换为 UpdateArticleVo")),
            ArticleDetailBo::Admin(bo) => Ok(Self {
                article_id: bo.article_id,
                title: bo.title,
                excerpt: bo.excerpt,
                markdown_content: bo.markdown_content,
                status: bo.status,
                created_at: bo.created_at,
                updated_at: bo.updated_at,
                published_at: bo.published_at,
                password: bo.password,
                attachments: bo
                    .attachments
                    .into_iter()
                    .map(ArticleAttachmentVo::from)
                    .collect(),
                pv: bo.pv,
                uv: bo.uv,
            }),
        }
    }
}

impl TemplateRenderData for UpdateArticleVo {
    fn template_name() -> &'static str {
        "article/update.html"
    }
}
