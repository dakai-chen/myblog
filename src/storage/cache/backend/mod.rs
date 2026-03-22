mod db;
pub use db::{get, init};

use crate::storage::cache::{Cache, CacheData, CacheSetMode};

pub trait CacheBackend {
    fn get<T>(&self, id: &str) -> impl Future<Output = anyhow::Result<Option<Cache<T>>>> + Send
    where
        T: CacheData;

    fn get_expires_at<T>(
        &self,
        id: &str,
    ) -> impl Future<Output = anyhow::Result<Option<i64>>> + Send
    where
        T: CacheData;

    fn set<T>(
        &self,
        cache: &Cache<T>,
        mode: CacheSetMode,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send
    where
        T: CacheData;

    fn set_expires_at<T>(
        &self,
        id: &str,
        expires_at: i64,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send
    where
        T: CacheData;

    fn exists<T>(&self, id: &str) -> impl Future<Output = anyhow::Result<bool>> + Send
    where
        T: CacheData;

    fn remove<T>(&self, id: &str) -> impl Future<Output = anyhow::Result<()>> + Send
    where
        T: CacheData;

    fn batch_remove<T>(&self, id_prefix: &str) -> impl Future<Output = anyhow::Result<()>> + Send
    where
        T: CacheData;
}
