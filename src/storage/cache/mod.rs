pub mod backend;

use std::borrow::Cow;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::storage::cache::backend::CacheBackend;
use crate::util::time::UnixTimestampSecs;

#[derive(Debug)]
pub struct CacheBuilder<T> {
    /// 缓存ID
    pub id: Option<String>,
    /// 创建时间
    pub created_at: i64,
    /// 过期时间
    pub expires_at: Option<i64>,
    /// 缓存类型
    pub kind: String,
    /// 缓存数据
    pub data: T,
}

impl<T: CacheData> CacheBuilder<T> {
    pub fn new(data: T) -> Self {
        Self {
            id: None,
            created_at: UnixTimestampSecs::now().as_i64(),
            expires_at: None,
            kind: T::kind().to_owned(),
            data,
        }
    }

    pub fn self_id(mut self) -> Self
    where
        T: CacheIdGenerator,
    {
        self.id = Some(self.data.generate_id().into_owned());
        self
    }

    pub fn id<G>(mut self, id: &G) -> Self
    where
        G: CacheIdGenerator,
    {
        self.id = Some(id.generate_id().into_owned());
        self
    }

    pub fn created_at(mut self, created_at: i64) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn expires_at(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.expires_at = Some(self.created_at.saturating_add_unsigned(ttl.as_secs()));
        self
    }

    pub fn build(self) -> anyhow::Result<Cache<T>> {
        Ok(Cache::new(
            self.id
                .ok_or_else(|| anyhow::anyhow!("Failed to build Cache: Missing `id`"))?,
            self.data,
            self.created_at,
            self.expires_at
                .ok_or_else(|| anyhow::anyhow!("Failed to build Cache: Missing `expires_at`"))?,
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cache<T> {
    /// 缓存ID
    pub id: String,
    /// 创建时间
    pub created_at: i64,
    /// 过期时间
    pub expires_at: i64,
    /// 缓存类型
    pub kind: String,
    /// 缓存数据
    pub data: T,
}

pub trait CacheIdGenerator {
    fn generate_id(&self) -> Cow<'_, str>;
}

impl<G: ?Sized + CacheIdGenerator> CacheIdGenerator for &G {
    fn generate_id(&self) -> Cow<'_, str> {
        G::generate_id(self)
    }
}

impl CacheIdGenerator for str {
    fn generate_id(&self) -> Cow<'_, str> {
        self.into()
    }
}

impl CacheIdGenerator for String {
    fn generate_id(&self) -> Cow<'_, str> {
        self.into()
    }
}

impl CacheIdGenerator for Cow<'_, str> {
    fn generate_id(&self) -> Cow<'_, str> {
        self.as_ref().into()
    }
}

pub trait CacheData: DeserializeOwned + Serialize + Send + Sync {
    fn kind() -> &'static str;
}

impl<T: CacheData> Cache<T> {
    pub fn builder(data: T) -> CacheBuilder<T> {
        CacheBuilder::new(data)
    }

    pub fn new<G>(id: G, data: T, created_at: i64, expires_at: i64) -> Self
    where
        G: CacheIdGenerator,
    {
        Self {
            id: id.generate_id().into_owned(),
            created_at,
            expires_at,
            kind: T::kind().to_owned(),
            data,
        }
    }

    pub async fn get_in<S>(id: &str, backend: &S) -> anyhow::Result<Option<Self>>
    where
        S: CacheBackend,
    {
        backend.get(id).await
    }

    pub async fn get_ttl_in<S>(id: &str, backend: &S) -> anyhow::Result<Option<Duration>>
    where
        S: CacheBackend,
    {
        let Some(expires_at) = Self::get_expires_at_in(id, backend).await? else {
            return Ok(None);
        };
        Ok(expires_at
            .checked_sub(UnixTimestampSecs::now().as_i64())
            .filter(|time| *time > 0)
            .map(|time| Duration::from_secs(time as u64)))
    }

    pub async fn get_expires_at_in<S>(id: &str, backend: &S) -> anyhow::Result<Option<i64>>
    where
        S: CacheBackend,
    {
        backend.get_expires_at::<T>(id).await
    }

    pub async fn set_in<S>(&self, mode: CacheSetMode, backend: &S) -> anyhow::Result<bool>
    where
        S: CacheBackend,
    {
        backend.set(self, mode).await
    }

    pub async fn set_ttl_in<S>(id: &str, ttl: Duration, backend: &S) -> anyhow::Result<bool>
    where
        S: CacheBackend,
    {
        Self::set_expires_at_in(
            id,
            UnixTimestampSecs::now().saturating_add(ttl).as_i64(),
            backend,
        )
        .await
    }

    pub async fn set_expires_at_in<S>(
        id: &str,
        expires_at: i64,
        backend: &S,
    ) -> anyhow::Result<bool>
    where
        S: CacheBackend,
    {
        backend.set_expires_at::<T>(id, expires_at).await
    }

    pub async fn exists_in<S>(id: &str, backend: &S) -> anyhow::Result<bool>
    where
        S: CacheBackend,
    {
        backend.exists::<T>(id).await
    }

    pub async fn remove_in<S>(id: &str, backend: &S) -> anyhow::Result<()>
    where
        S: CacheBackend,
    {
        backend.remove::<T>(id).await
    }

    pub async fn batch_remove_in<S>(id_prefix: &str, backend: &S) -> anyhow::Result<()>
    where
        S: CacheBackend,
    {
        backend.batch_remove::<T>(id_prefix).await
    }

    pub async fn get(id: &str) -> anyhow::Result<Option<Self>> {
        Self::get_in(id, backend::get()).await
    }

    pub async fn get_ttl(id: &str) -> anyhow::Result<Option<Duration>> {
        Self::get_ttl_in(id, backend::get()).await
    }

    pub async fn get_expires_at(id: &str) -> anyhow::Result<Option<i64>> {
        Self::get_expires_at_in(id, backend::get()).await
    }

    pub async fn set(&self, mode: CacheSetMode) -> anyhow::Result<bool> {
        Self::set_in(self, mode, backend::get()).await
    }

    pub async fn set_ttl(id: &str, ttl: Duration) -> anyhow::Result<bool> {
        Self::set_ttl_in(id, ttl, backend::get()).await
    }

    pub async fn set_expires_at(id: &str, expires_at: i64) -> anyhow::Result<bool> {
        Self::set_expires_at_in(id, expires_at, backend::get()).await
    }

    pub async fn exists(id: &str) -> anyhow::Result<bool> {
        Self::exists_in(id, backend::get()).await
    }

    pub async fn remove(id: &str) -> anyhow::Result<()> {
        Self::remove_in(id, backend::get()).await
    }

    pub async fn batch_remove(id_prefix: &str) -> anyhow::Result<()> {
        Self::batch_remove_in(id_prefix, backend::get()).await
    }

    pub fn is_expired(&self) -> bool {
        UnixTimestampSecs::now().as_i64() > self.expires_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheSetMode {
    /// 无条件覆盖写入
    /// 无论缓存中是否已存在该缓存类型的 ID ，直接写入/更新缓存，并设置过期时间
    Overwrite,
    /// 仅当缓存中不存在该缓存类型的 ID 时才写入（不存在则新增，存在则忽略）
    /// 用于避免并发场景下的重复写入
    OnlyIfNotExists,
    /// 仅当缓存中已存在该缓存类型的 ID 时才更新（存在则覆盖，不存在则忽略）
    /// 用于仅更新已有的缓存数据，避免新增无效缓存
    OnlyIfExists,
}
