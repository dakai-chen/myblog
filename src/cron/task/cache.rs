use std::sync::Arc;

use crate::state::AppState;

pub async fn prune_db_table_cache(state: Arc<AppState>) -> anyhow::Result<()> {
    /// 每次清理数据的条数上限
    const PRUNE_LIMIT: u64 = 100;

    let mut db = state.db.acquire().await?;
    loop {
        let rows = crate::storage::db::cache::remove_all_expired(PRUNE_LIMIT, &mut db).await?;
        tracing::info!("数据库缓存表清理成功，清理 {rows} 条数据");
        if rows < PRUNE_LIMIT {
            break;
        }
    }

    Ok(())
}
