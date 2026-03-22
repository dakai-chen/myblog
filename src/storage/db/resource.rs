use std::fmt::Write;

use sqlx::sqlite::SqliteArguments;
use sqlx::{Arguments, AssertSqlSafe};

use crate::model::po::resource::ResourcePo;
use crate::storage::db::DbConn;
use crate::util::result::ResultExt;

pub async fn create(resource: &ResourcePo, db: &mut DbConn) -> anyhow::Result<()> {
    sqlx::query(
        "
        INSERT INTO resource (
            `id`,
            `name`,
            `extension`,
            `path`,
            `size`,
            `mime_type`,
            `kind`,
            `sha256`,
            `created_at`
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
    )
    .bind(&resource.id)
    .bind(&resource.name)
    .bind(&resource.extension)
    .bind(&resource.path)
    .bind(&i64::try_from(resource.size)?)
    .bind(&resource.mime_type)
    .bind(&resource.kind)
    .bind(&resource.sha256)
    .bind(&resource.created_at)
    .execute(db)
    .await
    .map(|_| ())
    .map_err(From::from)
}

pub async fn update(resource: &ResourcePo, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query(
        "
        UPDATE resource SET
            `name` = ?,
            `extension` = ?,
            `path` = ?,
            `size` = ?,
            `mime_type` = ?,
            `kind` = ?,
            `sha256` = ?,
            `created_at` = ?
        WHERE
            `id` = ?
        ",
    )
    .bind(&resource.name)
    .bind(&resource.extension)
    .bind(&resource.path)
    .bind(&i64::try_from(resource.size)?)
    .bind(&resource.mime_type)
    .bind(&resource.kind)
    .bind(&resource.sha256)
    .bind(&resource.created_at)
    .bind(&resource.id)
    .execute(db)
    .await
    .map(|res| res.rows_affected())
    .map_err(From::from)
}

pub async fn remove(id: &str, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query("DELETE FROM resource WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map(|res| res.rows_affected())
        .map_err(From::from)
}

pub async fn remove_by_path(path: &str, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query("DELETE FROM resource WHERE path = ?")
        .bind(path)
        .execute(db)
        .await
        .map(|res| res.rows_affected())
        .map_err(From::from)
}

pub async fn find(id: &str, db: &mut DbConn) -> anyhow::Result<Option<ResourcePo>> {
    sqlx::query_as("SELECT * FROM resource WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(From::from)
}

pub async fn find_duplicate(
    sha256: &str,
    size: u64,
    db: &mut DbConn,
) -> anyhow::Result<Option<ResourcePo>> {
    sqlx::query_as(
        "SELECT * FROM resource WHERE sha256 = ? AND size = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(sha256)
    .bind(i64::try_from(size)?)
    .fetch_optional(db)
    .await
    .map_err(From::from)
}

pub async fn count_by_path(path: &str, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM resource WHERE path = ?")
        .bind(path)
        .fetch_one(db)
        .await
        .map_err(From::from)
}

pub async fn list_by_ids<Id>(ids: &[Id], db: &mut DbConn) -> anyhow::Result<Vec<ResourcePo>>
where
    Id: AsRef<str>,
{
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut sql = String::new();
    let mut sql_params = SqliteArguments::default();

    let binders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(&mut sql, "SELECT * FROM resource WHERE id IN ({binders})",)?;
    for id in ids {
        sql_params.add(id.as_ref()).anyhow()?;
    }

    sqlx::query_as_with(AssertSqlSafe(sql), sql_params)
        .fetch_all(db)
        .await
        .map_err(From::from)
}

pub async fn list_by_paths<Path>(paths: &[Path], db: &mut DbConn) -> anyhow::Result<Vec<ResourcePo>>
where
    Path: AsRef<str>,
{
    if paths.is_empty() {
        return Ok(vec![]);
    }

    let mut sql = String::new();
    let mut sql_params = SqliteArguments::default();

    let binders = std::iter::repeat_n("?", paths.len())
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(&mut sql, "SELECT * FROM resource WHERE path IN ({binders})",)?;
    for path in paths {
        sql_params.add(path.as_ref()).anyhow()?;
    }

    sqlx::query_as_with(AssertSqlSafe(sql), sql_params)
        .fetch_all(db)
        .await
        .map_err(From::from)
}

pub async fn all(db: &mut DbConn) -> anyhow::Result<Vec<ResourcePo>> {
    sqlx::query_as("SELECT * FROM resource")
        .fetch_all(db)
        .await
        .map_err(From::from)
}
