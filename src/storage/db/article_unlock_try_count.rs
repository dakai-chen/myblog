use crate::model::po::article_unlock_try_count::ArticleUnlockTryCountPo;
use crate::storage::db::DbConn;
use crate::util::time::UnixTimestampSecs;

pub async fn incr_count(po: &ArticleUnlockTryCountPo, db: &mut DbConn) -> anyhow::Result<u32> {
    sqlx::query_scalar(
        "
        INSERT INTO article_unlock_try_count (
            `ip`,
            `article_id`,
            `count`,
            `created_at`,
            `expires_at`
        ) VALUES (?, ?, ?, ?, ?)
        ON CONFLICT (`ip`, `article_id`) DO UPDATE SET
            `count` = `count` + 1
        RETURNING `count`
        ",
    )
    .bind(&po.ip)
    .bind(&po.article_id)
    .bind(&po.count)
    .bind(&po.created_at)
    .bind(&po.expires_at)
    .fetch_one(db)
    .await
    .map_err(From::from)
}

pub async fn remove_single_expired(
    ip: &str,
    article_id: &str,
    db: &mut DbConn,
) -> anyhow::Result<u64> {
    sqlx::query(
        "DELETE FROM article_unlock_try_count WHERE ip = ? AND article_id = ? AND expires_at < ?",
    )
    .bind(ip)
    .bind(article_id)
    .bind(UnixTimestampSecs::now().as_i64())
    .execute(db)
    .await
    .map(|res| res.rows_affected())
    .map_err(From::from)
}

pub async fn remove_all_expired(limit: u64, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query(
        "
        DELETE FROM article_unlock_try_count WHERE rowid IN (
            SELECT rowid FROM article_unlock_try_count WHERE expires_at < ? ORDER BY expires_at LIMIT ?
        )
        ",
    )
    .bind(UnixTimestampSecs::now().as_i64())
    .bind(i64::try_from(limit)?)
    .execute(db)
    .await
    .map(|res| res.rows_affected())
    .map_err(From::from)
}
