use std::sync::OnceLock;

use crate::model::po::cache::CachePo;
use crate::storage::cache::backend::CacheBackend;
use crate::storage::cache::{Cache, CacheData, CacheSetMode};
use crate::storage::db::DbPool;

static BACKEND: OnceLock<DbCacheBackend> = OnceLock::new();

pub fn init(db: DbPool) -> anyhow::Result<()> {
    BACKEND
        .set(DbCacheBackend { db })
        .map_err(|_| anyhow::anyhow!("重复初始化数据库缓存存储器"))
}

pub fn get() -> &'static DbCacheBackend {
    BACKEND.get().expect("数据库缓存存储器未初始化")
}

pub struct DbCacheBackend {
    db: DbPool,
}

impl CacheBackend for DbCacheBackend {
    async fn get<T>(&self, id: &str) -> anyhow::Result<Option<Cache<T>>>
    where
        T: CacheData,
    {
        let mut db = self.db.acquire().await?;
        let Some(po) = crate::storage::db::cache::find_active(T::kind(), id, &mut db).await? else {
            return Ok(None);
        };
        Ok(Some(Cache::try_from(po)?))
    }

    async fn get_expires_at<T>(&self, id: &str) -> anyhow::Result<Option<i64>>
    where
        T: CacheData,
    {
        let mut db = self.db.acquire().await?;
        let Some(expires_at) =
            crate::storage::db::cache::get_active_expires_at(T::kind(), id, &mut db).await?
        else {
            return Ok(None);
        };
        Ok(Some(expires_at))
    }

    async fn set<T>(&self, cache: &Cache<T>, mode: CacheSetMode) -> anyhow::Result<bool>
    where
        T: CacheData,
    {
        let mut db = self.db.acquire().await?;
        let po = CachePo {
            id: cache.id.clone(),
            kind: cache.kind.clone(),
            data: serde_json::to_string(&cache.data)?,
            created_at: cache.created_at,
            expires_at: cache.expires_at,
        };
        match mode {
            CacheSetMode::Overwrite => crate::storage::db::cache::create_or_update(&po, &mut db)
                .await
                .map(|rows| rows == 1),
            CacheSetMode::OnlyIfNotExists => {
                crate::storage::db::cache::remove_single_expired(T::kind(), &po.id, &mut db)
                    .await?;
                match crate::storage::db::cache::create(&po, &mut db).await {
                    Ok(_) => Ok(true),
                    Err(err) if crate::util::sqlx::error_is_unique_violation(&err) => Ok(false),
                    Err(err) => Err(err),
                }
            }
            CacheSetMode::OnlyIfExists => crate::storage::db::cache::update_active(&po, &mut db)
                .await
                .map(|rows| rows == 1),
        }
    }

    async fn set_expires_at<T>(&self, id: &str, expires_at: i64) -> anyhow::Result<bool>
    where
        T: CacheData,
    {
        let mut db = self.db.acquire().await?;
        crate::storage::db::cache::update_active_expires_at(T::kind(), id, expires_at, &mut db)
            .await
            .map(|rows| rows == 1)
    }

    async fn exists<T>(&self, id: &str) -> anyhow::Result<bool>
    where
        T: CacheData,
    {
        let mut db = self.db.acquire().await?;
        crate::storage::db::cache::exists_active(T::kind(), id, &mut db).await
    }

    async fn remove<T>(&self, id: &str) -> anyhow::Result<()>
    where
        T: CacheData,
    {
        let mut db = self.db.acquire().await?;
        crate::storage::db::cache::remove(T::kind(), id, &mut db)
            .await
            .map(|_| ())
    }

    async fn batch_remove<T>(&self, id_prefix: &str) -> anyhow::Result<()>
    where
        T: CacheData,
    {
        let mut db = self.db.acquire().await?;
        crate::storage::db::cache::remove_by_id_prefix(T::kind(), id_prefix, &mut db)
            .await
            .map(|_| ())
    }
}

impl<T> TryFrom<&Cache<T>> for CachePo
where
    T: CacheData,
{
    type Error = anyhow::Error;

    fn try_from(value: &Cache<T>) -> Result<Self, Self::Error> {
        Ok(CachePo {
            id: value.id.clone(),
            kind: value.kind.clone(),
            data: serde_json::to_string(&value.data)?,
            created_at: value.created_at,
            expires_at: value.expires_at,
        })
    }
}

impl<T> TryFrom<CachePo> for Cache<T>
where
    T: CacheData,
{
    type Error = anyhow::Error;

    fn try_from(value: CachePo) -> Result<Self, Self::Error> {
        Ok(Cache {
            id: value.id,
            created_at: value.created_at,
            expires_at: value.expires_at,
            kind: value.kind,
            data: serde_json::from_str(&value.data)?,
        })
    }
}
