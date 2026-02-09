use serde::{Deserialize, Serialize};

/// 文章状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum ArticleStatus {
    /// 草稿
    Draft,
    /// 私密
    Private,
    /// 发布
    Published,
}
