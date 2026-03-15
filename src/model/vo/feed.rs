use serde::Serialize;

use crate::model::bo::article::{ArticleListBo, ArticleListItemBo};
use crate::model::common::article::ArticleStatus;
use crate::template::render::TemplateRenderData;

/// 文章列表项
#[derive(Debug, Clone, Serialize)]
pub struct FeedListItemVo {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// Markdown 格式的正文
    pub markdown_content: Option<String>,
    /// 渲染后的 HTML 结果
    pub render_content: Option<String>,
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

impl From<ArticleListItemBo> for FeedListItemVo {
    fn from(value: ArticleListItemBo) -> Self {
        Self {
            article_id: value.article_id,
            title: value.title,
            markdown_content: (!value.need_password).then_some(value.markdown_content),
            render_content: (!value.need_password).then_some(value.render_content),
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            published_at: value.published_at,
            need_password: value.need_password,
        }
    }
}

/// RSS 页面
#[derive(Debug, Clone, Serialize)]
pub struct RssVo {
    /// 文章列表数据
    pub items: Vec<FeedListItemVo>,
}

impl From<ArticleListBo> for RssVo {
    fn from(value: ArticleListBo) -> Self {
        Self {
            items: value
                .data
                .items
                .into_iter()
                .map(FeedListItemVo::from)
                .collect(),
        }
    }
}

impl TemplateRenderData for RssVo {
    fn template_name() -> &'static str {
        "feed/rss.xml"
    }
}

/// Atom 页面
#[derive(Debug, Clone, Serialize)]
pub struct AtomVo {
    /// 修改时间
    pub updated_at: i64,
    /// 文章列表数据
    pub items: Vec<FeedListItemVo>,
}

impl From<ArticleListBo> for AtomVo {
    fn from(value: ArticleListBo) -> Self {
        Self {
            updated_at: value
                .data
                .items
                .iter()
                .map(|item| item.updated_at)
                .max()
                .unwrap_or(0),
            items: value
                .data
                .items
                .into_iter()
                .map(FeedListItemVo::from)
                .collect(),
        }
    }
}

impl TemplateRenderData for AtomVo {
    fn template_name() -> &'static str {
        "feed/atom.xml"
    }
}
