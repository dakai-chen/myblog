pub mod article;
pub mod article_attachment;
pub mod article_stats;
pub mod article_unlock_try_count;
pub mod cache;
pub mod resource;

use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Acquire, AssertSqlSafe, Transaction};

use crate::config::DatabaseConfig;

pub type Db = sqlx::Sqlite;
pub type DbPool = sqlx::Pool<Db>;
pub type DbConn = <Db as sqlx::Database>::Connection;
pub type DbPoolConn = sqlx::pool::PoolConnection<Db>;

pub async fn build_pool(config: &DatabaseConfig) -> anyhow::Result<DbPool> {
    let mut conn_opts = config.url.parse::<SqliteConnectOptions>()?;

    conn_opts = conn_opts.journal_mode(SqliteJournalMode::Wal);

    // 添加扩展
    for extension in sqlite_extensions(config) {
        conn_opts = unsafe { conn_opts.extension(extension.clone()) };
    }

    let mut pool_opts = SqlitePoolOptions::new();

    pool_opts = pool_opts.min_connections(config.pool.min_connections);
    pool_opts = pool_opts.max_connections(config.pool.max_connections);
    pool_opts = pool_opts.acquire_timeout(config.pool.acquire_timeout);
    pool_opts = pool_opts.idle_timeout(config.pool.idle_timeout);
    pool_opts = pool_opts.max_lifetime(config.pool.max_lifetime);

    if let Some(level) = &config.log.acquire_slow_level {
        pool_opts = pool_opts.acquire_slow_level(level.parse()?);
    }
    if let Some(threshold) = config.log.acquire_slow_threshold {
        pool_opts = pool_opts.acquire_slow_threshold(threshold);
    }

    Ok(pool_opts.connect_with(conn_opts).await?)
}

/// 初始化数据库
pub async fn init(db: &mut DbConn) -> anyhow::Result<()> {
    transaction(db, async |tx| {
        let sql = build_init_sql()?;
        sqlx::raw_sql(sql).execute(tx as &mut DbConn).await?;
        Ok(())
    })
    .await
}

fn build_init_sql() -> anyhow::Result<AssertSqlSafe<String>> {
    let mut sql = String::new();
    for entry in std::fs::read_dir(&crate::config::get().database.migrations.script_dir)? {
        let path = entry?.path();
        if path.is_file()
            && file_name_ends_with(
                &path,
                &crate::config::get().database.migrations.script_extension,
            )
        {
            sql.push('\n');
            sql.push_str(&std::fs::read_to_string(path)?);
        }
    }
    Ok(AssertSqlSafe(sql))
}

fn file_name_ends_with(path: &Path, extension: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .ends_with(extension)
}

#[cfg(unix)]
fn sqlite_extensions(config: &DatabaseConfig) -> Vec<String> {
    vec![
        crate::util::path::root(&config.sqlite.extensions_dir)
            .join("libsimple/linux/libsimple")
            .into_string(),
    ]
}

#[cfg(not(unix))]
fn sqlite_extensions(config: &DatabaseConfig) -> Vec<String> {
    vec![
        crate::util::path::root(&config.sqlite.extensions_dir)
            .join("libsimple/windows/simple")
            .into_string(),
    ]
}

pub async fn transaction<F, Res, Err>(db: &mut DbConn, f: F) -> Result<Res, Err>
where
    F: AsyncFnOnce(&mut Transaction<'_, Db>) -> Result<Res, Err>,
    Err: From<sqlx::error::Error>,
{
    let mut tx = db.begin().await?;
    let result = f(&mut tx).await;
    match &result {
        Ok(_) => tx.commit().await?,
        Err(_) => tx.rollback().await?,
    }
    result
}
