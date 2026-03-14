use std::pin::Pin;
use std::time::Instant;

use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::Instrument;

trait CronTaskRunnable<S>: Send + Sync {
    fn run(&mut self) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;
}

impl<S, F, Fut> CronTaskRunnable<S> for (S, F)
where
    S: Clone + Send + Sync,
    F: FnMut(S) -> Fut + Send + Sync,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    fn run(&mut self) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin((self.1)(self.0.clone()))
    }
}

pub struct CronTask<S> {
    schedule: String,
    func: Box<dyn CronTaskRunnable<S>>,
    name: String,
}

impl<S> CronTask<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new<F, Fut>(
        name: impl Into<String>,
        schedule: impl Into<String>,
        func: F,
        state: S,
    ) -> Self
    where
        F: FnMut(S) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        Self {
            schedule: schedule.into(),
            func: Box::new((state, func)),
            name: name.into(),
        }
    }
}

pub struct CronTaskCollector<S> {
    state: S,
    collector: Vec<CronTask<S>>,
}

impl<S> CronTaskCollector<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(state: S) -> Self {
        Self {
            state,
            collector: vec![],
        }
    }

    pub fn add_task(mut self, task: impl Into<CronTask<S>>) -> Self {
        self.collector.push(task.into());
        self
    }

    pub fn add_task_if(self, b: bool, task: impl Into<CronTask<S>>) -> Self {
        if b { self.add_task(task) } else { self }
    }

    pub fn add<F, Fut>(mut self, name: impl Into<String>, schedule: &str, func: F) -> Self
    where
        F: FnMut(S) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let task = CronTask::new(name, schedule, func, self.state.clone());
        self.collector.push(task);
        self
    }

    pub fn add_if<F, Fut>(self, b: bool, name: impl Into<String>, schedule: &str, func: F) -> Self
    where
        F: FnMut(S) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        if b {
            self.add(name, schedule, func)
        } else {
            self
        }
    }

    pub async fn register_to(self, scheduler: &JobScheduler) -> anyhow::Result<()> {
        for mut task in self.collector {
            let job = Job::new_async_tz(task.schedule, chrono::Local, move |_uuid, _lock| {
                let span = tracing::error_span!(
                    "CRON",
                    task_name = task.name,
                    exec_id = crate::util::uuid::v4(),
                );
                let start = Instant::now(); // 计时开始
                let fut = span.in_scope(|| {
                    tracing::info!("定时任务开始执行");
                    task.func.run()
                });
                Box::pin(
                    async move {
                        let result = fut.await;
                        let elapsed = start.elapsed(); // 计时结束
                        match result {
                            Ok(_) => tracing::info!("定时任务执行成功，执行耗时：{elapsed:.2?}"),
                            Err(e) => tracing::error!(
                                "定时任务执行失败，执行耗时：{elapsed:.2?}，错误详情：{e}"
                            ),
                        }
                    }
                    .instrument(span),
                )
            })?;
            scheduler.add(job).await?;
        }
        Ok(())
    }
}
