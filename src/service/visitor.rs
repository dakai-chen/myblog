use std::time::Duration;

use crate::error::{AppError, AppErrorMeta};
use crate::model::bo::visitor::VisitorArticleAccessPermitBo;
use crate::model::co::visitor::VisitorCo;
use crate::storage::cache::storage::CacheSetMode;
use crate::storage::cache::{Cache, CacheData};

pub const VISITOR_TTL: Duration = Duration::from_secs(3600 * 24 * 7);
pub const VISITOR_KEEP_THRESHOLD: Duration = Duration::from_secs(3600 * 24);

pub async fn create() -> Result<String, AppError> {
    let data = VisitorCo {
        visitor_id: crate::util::uuid::v4(),
    };
    let visitor = data.gen_id_with_ttl(VISITOR_TTL);
    if !visitor.set(CacheSetMode::OnlyIfNotExists).await? {
        return Err(AppErrorMeta::Internal.with_context(format!(
            "创建访客失败，访客ID已存在。访客ID: {}",
            visitor.data.visitor_id,
        )));
    }

    VisitorArticleAccessPermitBo::new(&visitor.data.visitor_id)
        .clear_article()
        .await?;

    Ok(visitor.data.visitor_id)
}

pub async fn keep_or_create(visitor_id: &str) -> Result<String, AppError> {
    if keep(visitor_id).await? {
        return Ok(visitor_id.to_owned());
    }
    create().await
}

pub async fn keep(visitor_id: &str) -> anyhow::Result<bool> {
    let Some(ttl) = Cache::<VisitorCo>::get_ttl(visitor_id).await? else {
        return Ok(false);
    };
    if ttl <= VISITOR_KEEP_THRESHOLD {
        Cache::<VisitorCo>::set_ttl(visitor_id, VISITOR_TTL).await
    } else {
        Ok(true)
    }
}
