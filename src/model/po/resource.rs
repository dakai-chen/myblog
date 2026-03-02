use crate::model::common::resource::ResourcePath;

/// 资源文件
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResourcePo {
    /// 资源ID
    pub id: String,
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
