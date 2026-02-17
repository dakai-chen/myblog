mod article;
mod cache;

use std::sync::Arc;

use crate::cron::CronTaskCollector;
use crate::cron::core::task_function_name;
use crate::state::AppState;

pub fn build(state: Arc<AppState>) -> anyhow::Result<CronTaskCollector<Arc<AppState>>> {
    CronTaskCollector::new(state)
        .config_add(cache::prune_db_table_cache)?
        .config_add(article::prune_db_table_article_unlock_try_count)
}

impl CronTaskCollector<Arc<AppState>> {
    fn config_add<F, Fut>(self, func: F) -> anyhow::Result<Self>
    where
        F: FnMut(Arc<AppState>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let name = task_function_name(std::any::type_name::<F>());
        let Some(config) = &crate::config::get().cron.tasks.get(name) else {
            return Err(anyhow::anyhow!("定时任务配置缺失，任务名: {name}"));
        };
        Ok(self.add_with_name_if(config.enabled, name, &config.schedule, func))
    }
}
