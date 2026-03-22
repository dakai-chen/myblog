use std::borrow::Cow;
use std::net::IpAddr;

use boluo::data::Extension;
use boluo::extract::FromRequest;
use boluo::request::Request;
use serde::{Deserialize, Serialize};

use crate::context::ip::ClientIP;
use crate::context::visitor::VisitorId;
use crate::error::{AppError, AppErrorMeta};
use crate::model::co::article::{VisitorArticleAccessPermitCo, VisitorArticleAccessPermitCoIdGen};
use crate::model::co::visitor::VisitorCo;
use crate::storage::cache::{Cache, CacheIdGenerator, CacheSetMode};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisitorBo {
    visitor: Cache<VisitorCo>,
    ip: IpAddr,
}

impl VisitorBo {
    pub async fn from_cache(visitor_id: &str, ip: IpAddr) -> anyhow::Result<Option<Self>> {
        Ok(Cache::get(visitor_id)
            .await?
            .map(|visitor| Self { visitor, ip }))
    }

    pub fn ip(&self) -> IpAddr {
        self.ip
    }

    pub fn visitor_id(&self) -> &str {
        &self.visitor.data.visitor_id
    }

    pub fn created_at(&self) -> i64 {
        self.visitor.created_at
    }

    pub fn expires_at(&self) -> i64 {
        self.visitor.expires_at
    }

    pub async fn add_article(&self, article_id: &str) -> anyhow::Result<()> {
        VisitorArticleAccessPermitBo::new(self.visitor_id())
            .add_article(article_id)
            .await
    }

    pub async fn has_article(&self, article_id: &str) -> anyhow::Result<bool> {
        VisitorArticleAccessPermitBo::new(self.visitor_id())
            .has_article(article_id)
            .await
    }

    pub async fn clear_article(&self) -> anyhow::Result<()> {
        VisitorArticleAccessPermitBo::new(self.visitor_id())
            .clear_article()
            .await
    }
}

impl FromRequest for VisitorBo {
    type Error = AppError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        let ClientIP(ip) = ClientIP::from_request(request).await?;
        let Some(visitor) = Option::<Extension<VisitorId>>::from_request(request).await? else {
            return Err(AppErrorMeta::Internal.with_context("请求扩展中 VisitorId 不存在"));
        };
        let Some(visitor) = VisitorBo::from_cache(visitor.visitor_id(), ip).await? else {
            return Err(AppErrorMeta::Internal.with_context(format!(
                "缓存中没有找到访客信息，访客ID: {}",
                visitor.visitor_id()
            )));
        };
        Ok(visitor)
    }
}

#[derive(Debug, Clone)]
pub struct VisitorArticleAccessPermitBo<'a> {
    visitor_id: Cow<'a, str>,
}

impl<'a> VisitorArticleAccessPermitBo<'a> {
    pub fn new(visitor_id: &'a str) -> Self {
        Self {
            visitor_id: visitor_id.into(),
        }
    }

    pub async fn add_article(&self, article_id: &str) -> anyhow::Result<()> {
        let cache_id = VisitorArticleAccessPermitCoIdGen {
            visitor_id: self.visitor_id.as_ref().into(),
            article_id: article_id.into(),
        };
        let permit = Cache::builder(VisitorArticleAccessPermitCo)
            .id(&cache_id)
            .ttl(crate::config::get().article.access_ttl)
            .build()?;
        permit.set(CacheSetMode::Overwrite).await?;
        Ok(())
    }

    pub async fn has_article(&self, article_id: &str) -> anyhow::Result<bool> {
        let cache_id = VisitorArticleAccessPermitCoIdGen {
            visitor_id: self.visitor_id.as_ref(),
            article_id: article_id,
        };
        Cache::<VisitorArticleAccessPermitCo>::exists(cache_id.generate_id().as_ref()).await
    }

    pub async fn clear_article(&self) -> anyhow::Result<()> {
        Cache::<VisitorArticleAccessPermitCo>::batch_remove(&self.visitor_id)
            .await
            .map_err(From::from)
    }
}
