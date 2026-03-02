use std::env;
use std::path::Path;

use boluo::BoxError;
use boluo::body::Bytes;
use futures_util::{Stream, StreamExt};
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppErrorMeta};
use crate::model::bo::resource::{
    FindResourceBo, RemoveResourceBo, ResourceBo, UploadResourceBo, UploadResourceMetaBo,
    UploadResourceOptionsBo,
};
use crate::model::po::resource::ResourcePo;
use crate::storage::db::DbConn;
use crate::util::time::UnixTimestampSecs;

/// 删除资源文件
pub async fn remove_resource(bo: &RemoveResourceBo<'_>, db: &mut DbConn) -> Result<(), AppError> {
    let Some(resource) = crate::storage::db::resource::find(&bo.resource_id, db).await? else {
        return Ok(());
    };
    crate::storage::db::resource::remove(&resource.id, db).await?;

    // 不再即时删除物理文件，改为定期统一清理
    //
    // let ref_count = crate::storage::db::resource::count_by_path(&resource.path, db).await?;
    // if ref_count == 0
    //     && let Err(e) = std::fs::remove_file(&resource.path)
    // {
    //     tracing::warn!("文件删除失败，路径：{}，错误：{e}", resource.path);
    // }

    Ok(())
}

/// 获取资源文件
pub async fn find_resource(
    bo: &FindResourceBo<'_>,
    db: &mut DbConn,
) -> Result<Option<ResourcePo>, AppError> {
    let Some(resource) = crate::storage::db::resource::find(&bo.resource_id, db).await? else {
        return Ok(None);
    };
    if !Path::new(&resource.path).is_file() {
        tracing::warn!(
            "资源文件不存在 (id: {}, path: {})",
            resource.id,
            resource.path
        );
        return Ok(None);
    }
    Ok(Some(resource))
}

/// 基于自定义选项上传资源文件
pub async fn upload_resource_with_options(
    upload: UploadResourceBo,
    options: UploadResourceOptionsBo,
    db: &mut DbConn,
) -> Result<ResourceBo, AppError> {
    if upload.meta.size > crate::config::get().resource.upload_file_max_size {
        return Err(AppErrorMeta::DataTooLarge {
            limit: crate::config::get().resource.upload_file_max_size,
        }
        .into());
    }

    let temp_file = save_file_verify_sha256(&upload.meta, upload.data).await?;

    let duplicate_path = if let Some(duplicate) =
        crate::storage::db::resource::find_duplicate(&upload.meta.sha256, upload.meta.size, db)
            .await?
        && Path::new(&duplicate.path).is_file()
    {
        Some(duplicate.path)
    } else {
        None
    };

    let (created_file, path) = match duplicate_path {
        Some(duplicate_path) => (None, duplicate_path),
        None => {
            let path = generate_storage_path();
            let file = temp_file.move_to(&path).await?;
            (Some(file), path)
        }
    };

    let extension = crate::util::path::extension(&upload.meta.name).to_owned();
    let resource = ResourcePo {
        id: options.resource_id,
        name: upload.meta.name,
        extension,
        path,
        size: upload.meta.size,
        mime_type: upload.meta.mime_type,
        is_public: options.is_public,
        sha256: upload.meta.sha256,
        created_at: UnixTimestampSecs::now().as_i64(),
    };
    crate::storage::db::resource::create(&resource, db).await?;

    if let Some(file) = created_file {
        TempFileGuard::keep(file)
    }

    Ok(ResourceBo::from(resource))
}

/// 上传资源文件
pub async fn upload_resource(
    bo: UploadResourceBo,
    db: &mut DbConn,
) -> Result<ResourceBo, AppError> {
    let options = UploadResourceOptionsBo {
        resource_id: crate::util::uuid::v4(),
        is_public: true,
    };
    upload_resource_with_options(bo, options, db).await
}

/// 将数据存储到临时文件并计算其 SHA256 值，并校验与用户提供的哈希是否一致
async fn save_file_verify_sha256<T>(
    meta: &UploadResourceMetaBo,
    mut data: T,
) -> Result<TempFileGuard, AppError>
where
    T: Stream<Item = Result<Bytes, BoxError>> + Unpin,
{
    let temp_path = generate_tempfile_path()?;
    let mut remaining = meta.size;
    let mut hasher = Sha256::new();
    let mut temp_file = File::create_new(&temp_path).await?;

    let guard = TempFileGuard::new(temp_path);

    while let Some(chunk) = data.next().await.transpose()? {
        let Ok(chunk_size) = u64::try_from(chunk.len()) else {
            return Err(AppErrorMeta::BadRequest
                .with_message("文件大小超出系统支持范围（超过 u64 最大值）"));
        };
        if chunk_size > remaining {
            return Err(AppErrorMeta::BadRequest
                .with_message("上传文件的大小与实际不符，请检查文件是否完整或重新上传"));
        }
        remaining -= chunk_size;
        temp_file.write_all(&chunk).await?;
        hasher.update(&chunk);
    }
    temp_file.sync_all().await?;

    if remaining != 0 {
        return Err(
            AppErrorMeta::BadRequest.with_message("上传文件数据不完整，可能是网络中断或文件损坏")
        );
    }

    let actual_sha256 = format!("{:x}", hasher.finalize());
    if meta.sha256 != actual_sha256 {
        return Err(AppErrorMeta::BadRequest
            .with_message("上传文件的 SHA256 哈希校验失败，请检查文件完整性或哈希值是否正确"));
    }

    Ok(guard)
}

/// 生成文件存储路径
fn generate_storage_path() -> String {
    let file_name = crate::util::uuid::v4();
    crate::util::path::root(&crate::config::get().resource.upload_dir)
        .join(file_name.get(0..2).unwrap())
        .join(file_name.get(2..4).unwrap())
        .join(file_name)
        .into_string()
}

/// 生成一个临时文件路径
fn generate_tempfile_path() -> anyhow::Result<String> {
    env::temp_dir()
        .join(crate::util::uuid::v4())
        .into_os_string()
        .into_string()
        .map_err(|os_str| {
            anyhow::anyhow!(
                "临时目录路径无法转换为 UTF-8 字符串（路径：{os_str:?}），可能包含非 UTF-8 字符"
            )
        })
}

struct TempFileGuard {
    path: String,
}

impl TempFileGuard {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn keep(this: Self) {
        std::mem::forget(this);
    }

    pub async fn move_to(self, to: &str) -> std::io::Result<TempFileGuard> {
        if let Some(dir) = Path::new(to).parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        // 原子创建文件，防止覆盖原有的文件
        let f = File::create_new(to).await.map(|_| TempFileGuard::new(to))?;
        // 尝试移动文件
        if tokio::fs::rename(&self.path, to).await.is_ok() {
            // 如果移动成功，原文件就不存在了，就不需要删除了
            TempFileGuard::keep(self);
        } else {
            tokio::fs::copy(&self.path, to).await?;
        }
        Ok(f)
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::warn!("文件删除失败，路径：{}，错误：{e}", self.path);
        }
    }
}
