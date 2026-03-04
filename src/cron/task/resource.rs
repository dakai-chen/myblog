use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tracing::Span;
use walkdir::{DirEntry, WalkDir};

use crate::model::common::resource::ResourcePath;
use crate::state::AppState;
use crate::util::path::PathJoin;
use crate::util::time::UnixTimestampSecs;

const FS_BATCH_SIZE: usize = 1000;
const DB_BATCH_SIZE: usize = 200;

pub async fn purge_orphaned_resources(state: Arc<AppState>) -> anyhow::Result<()> {
    let mut scanner = {
        let dir = &crate::config::get().resource.upload_dir;
        let threshold = crate::config::get().resource.trash_threshold;
        FileScanner::new(dir, threshold)
    };

    let mut count: u64 = 0;

    loop {
        let scanned_files = {
            let f = move || {
                let scanned_files = scanner.next_batch(FS_BATCH_SIZE);
                (scanner, scanned_files)
            };
            let span = Span::current();
            let task = tokio::task::spawn_blocking(move || span.in_scope(f));
            let (r_scanner, scanned_files) = task.await?;
            scanner = r_scanner;
            scanned_files
        };

        for batch_files in scanned_files.chunks(DB_BATCH_SIZE).map(<[_]>::to_vec) {
            let resources = {
                let mut db = state.db.acquire().await?;
                let paths = batch_files.iter().map(|f| f.relative()).collect::<Vec<_>>();
                crate::storage::db::resource::list_by_paths(&paths, &mut db)
                    .await?
                    .into_iter()
                    .map(|r| r.path.into_relative())
                    .collect::<HashSet<_>>()
            };

            count += {
                let f = move || purge(resources, batch_files);
                let span = Span::current();
                let task = tokio::task::spawn_blocking(move || span.in_scope(f));
                task.await?
            };
        }

        if scanned_files.len() < FS_BATCH_SIZE {
            break;
        }
    }

    tracing::info!("清理孤立资源文件 {count} 个");

    Ok(())
}

fn purge(resources: HashSet<String>, files: Vec<ResourcePath>) -> u64 {
    let mut count = 0;
    for path in files {
        if !resources.contains(path.relative()) {
            if let Err(e) = move_to_trash(&path) {
                tracing::warn!("清理文件失败，路径：{}，错误：{e}", path.absolute());
            } else {
                count += 1;
            }
        }
    }
    count
}

fn move_to_trash(path: &ResourcePath) -> Result<(), std::io::Error> {
    let to = PathJoin::root(&crate::config::get().resource.trash_dir)
        .join(path.relative())
        .into_string();
    if let Some(dir) = Path::new(&to).parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::rename(path.absolute(), &to)
}

struct FileScanner {
    iter: Box<dyn Iterator<Item = DirEntry> + Send + 'static>,
    time_threshold: UnixTimestampSecs,
}

impl FileScanner {
    pub fn new(dir: &str, threshold: Duration) -> Self {
        Self {
            iter: Box::new(
                WalkDir::new(dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file()),
            ),
            time_threshold: UnixTimestampSecs::now().sub(threshold),
        }
    }

    pub fn next_batch_into(&mut self, n: usize, buf: &mut Vec<ResourcePath>) -> usize {
        let buf_init_len = buf.len();
        for entry in self.iter.as_mut().take(n) {
            let Some(created) = entry
                .metadata()
                .ok()
                .and_then(|m| m.created().ok())
                .and_then(UnixTimestampSecs::from_system_time)
            else {
                continue;
            };
            if created.as_i64() > self.time_threshold.as_i64() {
                continue;
            }
            let Some(path) = entry.path().to_str() else {
                continue;
            };
            let path = match ResourcePath::from_absolute(&path) {
                Ok(path) => path,
                Err(err) => {
                    tracing::warn!("{err}");
                    continue;
                }
            };
            buf.push(path);
        }
        buf.len() - buf_init_len
    }

    pub fn next_batch(&mut self, n: usize) -> Vec<ResourcePath> {
        let mut buf = Vec::with_capacity(n);
        self.next_batch_into(n, &mut buf);
        buf
    }
}
