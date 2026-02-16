use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::storage::cache::{CacheData, CacheIdGenerator};

#[derive(Debug, Clone)]
pub struct VisitorArticleAccessPermitCoIdGen<'a> {
    /// 访客ID
    pub visitor_id: &'a str,
    /// 文章ID
    pub article_id: &'a str,
}

impl CacheIdGenerator for VisitorArticleAccessPermitCoIdGen<'_> {
    fn generate_id(&self) -> Cow<'_, str> {
        format!("{}:{}", self.visitor_id, self.article_id).into()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisitorArticleAccessPermitCo;

impl CacheData for VisitorArticleAccessPermitCo {
    fn kind() -> &'static str {
        "visitor_article_access_permit"
    }
}

#[derive(Debug, Clone)]
pub struct VisitorArticleAccessRecordCoIdGen<'a> {
    /// 访客ID
    pub visitor_id: &'a str,
    /// 文章ID
    pub article_id: &'a str,
}

impl CacheIdGenerator for VisitorArticleAccessRecordCoIdGen<'_> {
    fn generate_id(&self) -> Cow<'_, str> {
        format!("{}:{}", self.visitor_id, self.article_id).into()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisitorArticleAccessRecordCo;

impl CacheData for VisitorArticleAccessRecordCo {
    fn kind() -> &'static str {
        "visitor_article_access_record"
    }
}
