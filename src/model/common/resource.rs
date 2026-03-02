use sqlx::Database;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use crate::storage::db::Db;

/// 资源路径工具类
#[derive(Debug, Clone)]
pub struct ResourcePath {
    // 存储数据库中的相对路径
    relative_path: String,
    // 预计算的绝对路径
    absolute_path: String,
}

impl ResourcePath {
    pub fn from_relative(relative_path: impl Into<String>) -> anyhow::Result<Self> {
        let relative_path = relative_path.into();
        let absolute_path = crate::util::path::root(&crate::config::get().resource.upload_dir)
            .join(&relative_path)
            .into_string();

        Ok(Self {
            relative_path,
            absolute_path,
        })
    }

    pub fn from_absolute(absolute_path: impl Into<String>) -> anyhow::Result<Self> {
        let absolute_path = absolute_path.into();
        let relative_path = absolute_path
            .strip_prefix(&crate::config::get().resource.upload_dir)
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
        ResourcePath::from_relative(<String as sqlx::Decode<Db>>::decode(value)?)
            .map_err(From::from)
    }
}
