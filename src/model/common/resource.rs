use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use sqlx::Database;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use crate::storage::db::Db;
use crate::util::path::PathJoin;

static NORMALIZE_SEP_UPLOAD_DIR: LazyLock<String> = LazyLock::new(|| {
    let dir = &crate::config::get().resource.upload_dir;
    let dir = crate::util::path::normalize_sep(dir);
    dir.trim_end_matches('/').to_owned()
});

/// 资源路径工具类
#[derive(Debug, Clone)]
pub struct ResourcePath {
    // 存储数据库中的相对路径
    relative_path: String,
    // 预计算的绝对路径
    absolute_path: String,
}

impl ResourcePath {
    pub fn from_relative(relative_path: &str) -> Self {
        let relative_path = crate::util::path::normalize_sep(relative_path).into_owned();
        let absolute_path = PathJoin::root(NORMALIZE_SEP_UPLOAD_DIR.as_str())
            .join(&relative_path)
            .into_string();

        Self {
            relative_path,
            absolute_path,
        }
    }

    pub fn from_absolute(absolute_path: &str) -> anyhow::Result<Self> {
        let absolute_path = crate::util::path::normalize_sep(absolute_path).into_owned();
        let relative_path = absolute_path
            .strip_prefix(NORMALIZE_SEP_UPLOAD_DIR.as_str())
            .ok_or_else(|| anyhow::anyhow!("资源路径不在上传目录内：{absolute_path}"))?;

        let relative_path = relative_path
            .strip_prefix('/')
            .ok_or_else(|| anyhow::anyhow!("资源路径不在上传目录内：{absolute_path}"))?;

        Ok(Self {
            relative_path: relative_path.to_owned(),
            absolute_path,
        })
    }

    /// 获取相对路径
    pub fn relative(&self) -> &str {
        &self.relative_path
    }

    /// 获取绝对路径
    pub fn absolute(&self) -> &str {
        &self.absolute_path
    }

    /// 获取相对路径
    pub fn into_relative(self) -> String {
        self.relative_path
    }

    /// 获取绝对路径
    pub fn into_absolute(self) -> String {
        self.absolute_path
    }
}

impl sqlx::Type<Db> for ResourcePath {
    fn type_info() -> <Db as Database>::TypeInfo {
        <String as sqlx::Type<Db>>::type_info()
    }
}

impl sqlx::Encode<'_, Db> for ResourcePath {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        <&str as sqlx::Encode<Db>>::encode(self.relative(), buf)
    }
}

impl sqlx::Decode<'_, Db> for ResourcePath {
    fn decode(value: <Db as Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <String as sqlx::Decode<Db>>::decode(value).map(|path| ResourcePath::from_relative(&path))
    }
}

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum ResourceKind {
    /// 公开资源
    Public,
    /// 附件资源
    Attachment,
}
