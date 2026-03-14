mod cache;
mod failed_attempts;
mod resource;

use std::sync::Arc;

use crate::cron::CronTaskCollector;
use crate::state::AppState;

pub fn build(state: Arc<AppState>) -> anyhow::Result<CronTaskCollector<Arc<AppState>>> {
    CronTaskCollectorBuilder::new(CronTaskCollector::new(state))
        .config_add("prune_cache", cache::prune)
        .config_add("purge_orphaned_resources", resource::purge_orphaned)
        .config_add("prune_failed_attempts", failed_attempts::prune)
        .build()
}

struct CronTaskCollectorBuilder {
    inner: anyhow::Result<CronTaskCollector<Arc<AppState>>>,
}

impl CronTaskCollectorBuilder {
    fn new(collector: CronTaskCollector<Arc<AppState>>) -> Self {
        Self {
            inner: Ok(collector),
        }
    }

    fn config_add<F, Fut>(mut self, name: &str, func: F) -> Self
    where
        F: FnMut(Arc<AppState>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.inner = self.inner.and_then(|collector| {
            let Some(config) = &crate::config::get().cron.tasks.get(name) else {
                return Err(anyhow::anyhow!("定时任务配置缺失，任务名: {name}"));
            };
            Ok(collector.add_if(config.enabled, name, &config.schedule, func))
        });
        self
    }

    fn build(self) -> anyhow::Result<CronTaskCollector<Arc<AppState>>> {
        self.inner
    }
}
