/// 文章解锁尝试次数统计
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArticleUnlockTryCountPo {
    /// IP
    pub ip: String,
    /// 文章ID
    pub article_id: String,
    /// 错误次数统计
    pub count: u32,
    /// 创建时间
    pub created_at: i64,
    /// 过期时间
    pub expires_at: i64,
}
