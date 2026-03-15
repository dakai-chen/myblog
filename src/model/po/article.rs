use std::borrow::Cow;

use crate::model::common::article::{ArticleStatus, SearchArticleSort};

/// 文章
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArticlePo {
    /// 文章ID
    pub id: String,
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
    /// 渲染版本
    pub render_version: u32,
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

/// 搜索文章
#[derive(Debug, Clone)]
pub struct SearchArticle<'a> {
    /// 全文搜索
    pub full_text: Option<Cow<'a, str>>,
    /// 状态
    pub status: Option<ArticleStatus>,
    /// 发布时间（大于等于）
    pub published_at_ge: Option<i64>,
    /// 发布时间（小于）
    pub published_at_lt: Option<i64>,
    /// 是否需要密码访问
    pub need_password: Option<bool>,
    /// 排序方式
    pub sort: SearchArticleSort,
}
