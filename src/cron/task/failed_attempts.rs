use std::sync::Arc;

use crate::state::AppState;

/// 每次清理数据的条数上限
const PRUNE_LIMIT: u64 = 100;

pub async fn prune(state: Arc<AppState>) -> anyhow::Result<()> {
    let mut db = state.db.acquire().await?;
    loop {
        let rows =
            crate::storage::db::failed_attempts::remove_all_expired(PRUNE_LIMIT, &mut db).await?;
        tracing::info!("清理 {rows} 条数据");
        if rows < PRUNE_LIMIT {
            break;
        }
    }
    Ok(())
}
