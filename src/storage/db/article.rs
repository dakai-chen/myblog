use std::fmt::Write;

use sqlx::sqlite::SqliteArguments;
use sqlx::{Arguments, AssertSqlSafe};

use crate::model::common::article::SearchArticleSort;
use crate::model::po::article::{ArticlePo, SearchArticle};
use crate::storage::db::DbConn;
use crate::util::pagination::Offset;
use crate::util::result::ResultExt;

pub async fn create(article: &ArticlePo, db: &mut DbConn) -> anyhow::Result<()> {
    sqlx::query(
        "
        INSERT INTO article (
            `id`,
            `title`,
            `excerpt`,
            `markdown_content`,
            `plain_content`,
            `render_content`,
            `render_version`,
            `password`,
            `status`,
            `created_at`,
            `updated_at`,
            `published_at`
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
    )
    .bind(&article.id)
    .bind(&article.title)
    .bind(&article.excerpt)
    .bind(&article.markdown_content)
    .bind(&article.plain_content)
    .bind(&article.render_content)
    .bind(&article.render_version)
    .bind(&article.password)
    .bind(&article.status)
    .bind(&article.created_at)
    .bind(&article.updated_at)
    .bind(&article.published_at)
    .execute(db)
    .await
    .map(|_| ())
    .map_err(From::from)
}

pub async fn update(article: &ArticlePo, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query(
        "
        UPDATE article SET
            `title` = ?,
            `excerpt` = ?,
            `markdown_content` = ?,
            `plain_content` = ?,
            `render_content` = ?,
            `render_version` = ?,
            `password` = ?,
            `status` = ?,
            `created_at` = ?,
            `updated_at` = ?,
            `published_at` = ?
        WHERE
            `id` = ?
        ",
    )
    .bind(&article.title)
    .bind(&article.excerpt)
    .bind(&article.markdown_content)
    .bind(&article.plain_content)
    .bind(&article.render_content)
    .bind(&article.render_version)
    .bind(&article.password)
    .bind(&article.status)
    .bind(&article.created_at)
    .bind(&article.updated_at)
    .bind(&article.published_at)
    .bind(&article.id)
    .execute(db)
    .await
    .map(|res| res.rows_affected())
    .map_err(From::from)
}

pub async fn update_render_content(
    id: &str,
    render_content: &str,
    render_version: &str,
    db: &mut DbConn,
) -> anyhow::Result<u64> {
    sqlx::query(
        "
        UPDATE article SET
            `render_content` = ?,
            `render_version` = ?
        WHERE
            `id` = ? AND `render_version` != ?
        ",
    )
    .bind(render_content)
    .bind(render_version)
    .bind(id)
    .bind(render_version)
    .execute(db)
    .await
    .map(|res| res.rows_affected())
    .map_err(From::from)
}

pub async fn remove(id: &str, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query("DELETE FROM article WHERE id = ?")
        .bind(id)
        .execute(db)
        .await
        .map(|res| res.rows_affected())
        .map_err(From::from)
}

pub async fn update_password(
    id: &str,
    password: impl Into<Option<&str>>,
    db: &mut DbConn,
) -> anyhow::Result<u64> {
    if let Some(password) = password.into() {
        sqlx::query("UPDATE article SET password = ? WHERE id = ?")
            .bind(password)
            .bind(id)
            .execute(db)
            .await
            .map(|res| res.rows_affected())
            .map_err(From::from)
    } else {
        sqlx::query("UPDATE article SET password = NULL WHERE id = ?")
            .bind(id)
            .execute(db)
            .await
            .map(|res| res.rows_affected())
            .map_err(From::from)
    }
}

pub async fn find(id: &str, db: &mut DbConn) -> anyhow::Result<Option<ArticlePo>> {
    sqlx::query_as("SELECT * FROM article WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(From::from)
}

pub async fn search(
    params: &SearchArticle<'_>,
    offset: Offset,
    db: &mut DbConn,
) -> anyhow::Result<Vec<ArticlePo>> {
    let mut sql = String::new();
    let mut sql_params = SqliteArguments::default();

    writeln!(&mut sql, "SELECT * FROM article")?;

    let where_conditions = search_where_conditions(params, &mut sql_params)?;
    if !where_conditions.is_empty() {
        writeln!(&mut sql, "WHERE {}", where_conditions.join(" AND "))?;
    }

    let order = match params.sort {
        SearchArticleSort::ByPublishedAtDesc => "published_at DESC NULLS FIRST, updated_at DESC",
        SearchArticleSort::ByUpdatedAtDesc => "updated_at DESC NULLS FIRST",
    };

    sqlx::query_as_with(
        AssertSqlSafe(format!("{sql} ORDER BY {order} LIMIT ? OFFSET ?")),
        sql_params,
    )
    .bind(i64::try_from(offset.size)?)
    .bind(i64::try_from(offset.offset)?)
    .fetch_all(db)
    .await
    .map_err(From::from)
}

pub async fn search_count(params: &SearchArticle<'_>, db: &mut DbConn) -> anyhow::Result<u64> {
    let mut sql = String::new();
    let mut sql_params = SqliteArguments::default();

    writeln!(&mut sql, "SELECT COUNT(*) FROM article")?;

    let where_conditions = search_where_conditions(params, &mut sql_params)?;
    if !where_conditions.is_empty() {
        writeln!(&mut sql, "WHERE {}", where_conditions.join(" AND "))?;
    }

    sqlx::query_scalar_with(AssertSqlSafe(sql), sql_params)
        .fetch_one(db)
        .await
        .map_err(From::from)
}

fn search_where_conditions(
    params: &SearchArticle<'_>,
    sql_params: &mut SqliteArguments,
) -> anyhow::Result<Vec<String>> {
    let mut conditions = vec![];

    if let Some(full_text) = params
        .full_text
        .as_deref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        sql_params.add(full_text).anyhow()?;
        conditions.push(format!(
            "id IN (SELECT id FROM article_fts WHERE article_fts MATCH simple_query(?) LIMIT {})",
            crate::config::get().article.full_text_search_limit
        ));
    }
    if let Some(status) = &params.status {
        sql_params.add(status).anyhow()?;
        conditions.push(format!("status = ?"));
    }
    if let Some(published_at_ge) = params.published_at_ge {
        sql_params.add(published_at_ge).anyhow()?;
        conditions.push(format!("published_at >= ?"));
    }
    if let Some(published_at_lt) = params.published_at_lt {
        sql_params.add(published_at_lt).anyhow()?;
        conditions.push(format!("published_at < ?"));
    }
    if let Some(need_password) = params.need_password {
        if need_password {
            conditions.push(format!("password IS NOT NULL"));
        } else {
            conditions.push(format!("password IS NULL"));
        }
    }

    Ok(conditions)
}
