use std::borrow::Cow;
use std::pin::Pin;

use boluo::BoxError;
use boluo::body::Bytes;
use futures_util::Stream;

use crate::model::common::resource::ResourcePath;
use crate::model::po::resource::ResourcePo;

/// 上传资源文件自定义选项
#[derive(Debug, Clone)]
pub struct UploadResourceOptionsBo {
    /// 资源ID
    pub resource_id: String,
    /// 是否公开访问
    pub is_public: bool,
}

/// 上传资源文件元数据
#[derive(Debug, Clone)]
pub struct UploadResourceMetaBo {
    /// 文件名
    pub name: String,
    /// 文件大小
    pub size: u64,
    /// 文件类型
    pub mime_type: String,
    /// 文件哈希
    pub sha256: String,
}

/// 上传资源文件
pub struct UploadResourceBo {
    /// 文件元数据
    pub meta: UploadResourceMetaBo,
    /// 文件数据
    pub data: Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send>>,
}

/// 资源文件
#[derive(Debug, Clone)]
pub struct ResourceBo {
    /// 资源ID
    pub resource_id: String,
    /// 文件名
    pub name: String,
    /// 文件扩展名
    pub extension: String,
    /// 文件存储路径
    pub path: ResourcePath,
    /// 文件大小
    pub size: u64,
    /// 文件类型
    pub mime_type: String,
    /// 是否公开访问
    pub is_public: bool,
    /// 文件哈希
    pub sha256: String,
    /// 创建时间
    pub created_at: i64,
}

impl From<ResourcePo> for ResourceBo {
    fn from(value: ResourcePo) -> Self {
        Self {
            resource_id: value.id,
            name: value.name,
            extension: value.extension,
            path: value.path,
            size: value.size,
            mime_type: value.mime_type,
            is_public: value.is_public,
            sha256: value.sha256,
            created_at: value.created_at,
        }
    }
}

/// 获取资源文件
#[derive(Debug, Clone)]
pub struct FindResourceBo<'a> {
    /// 资源文件ID
    pub resource_id: Cow<'a, str>,
}

/// 删除资源文件
#[derive(Debug, Clone)]
pub struct RemoveResourceBo<'a> {
    /// 资源文件ID
    pub resource_id: Cow<'a, str>,
}
