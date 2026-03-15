use boluo::body::Body;
use boluo::extract::FromRequest;
use boluo::http::HeaderMap;
use boluo::request::Request;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use crate::error::{AppError, AppErrorMeta};
use crate::model::bo::article::{
    AdminArticleDetailsBo, ArticleAttachmentBo, ArticleDetailsBo, ArticleListBo, ArticleListItemBo,
    CreateArticleBo, DownloadArticleAttachmentBo, GetArticleBo, RemoveArticleAttachmentBo,
    RemoveArticleBo, SearchArticleBo, UnlockArticleBo, UpdateArticleBo, UploadArticleAttachmentBo,
    VisitorArticleDetailsBo,
};
use crate::model::common::article::ArticleStatus;
use crate::model::dto::api::resource::UploadResourceDto;
use crate::util::pagination::PageData;

/// 解锁文章
#[derive(Debug, Clone, Deserialize)]
pub struct UnlockArticleDto {
    /// 文章ID
    pub article_id: String,
    /// 文章访问密码
    pub password: String,
}

impl<'a> Into<UnlockArticleBo<'a>> for UnlockArticleDto {
    fn into(self) -> UnlockArticleBo<'a> {
        UnlockArticleBo {
            article_id: self.article_id.into(),
            password: self.password.into(),
        }
    }
}

/// 创建文章
#[derive(Debug, Clone, Deserialize)]
pub struct CreateArticleDto {
    /// 标题
    pub title: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
}

impl Into<CreateArticleBo> for CreateArticleDto {
    fn into(self) -> CreateArticleBo {
        CreateArticleBo {
            title: self.title,
            markdown_content: self.markdown_content,
            password: self.password,
            status: self.status,
        }
    }
}

/// 修改文章
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateArticleDto {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
}

impl Into<UpdateArticleBo> for UpdateArticleDto {
    fn into(self) -> UpdateArticleBo {
        UpdateArticleBo {
            article_id: self.article_id,
            title: self.title,
            markdown_content: self.markdown_content,
            password: self.password,
            status: self.status,
        }
    }
}

/// 删除文章
#[derive(Debug, Clone, Deserialize)]
pub struct RemoveArticleDto {
    /// 文章ID
    pub article_id: String,
}

impl<'a> Into<RemoveArticleBo<'a>> for RemoveArticleDto {
    fn into(self) -> RemoveArticleBo<'a> {
        RemoveArticleBo {
            article_id: self.article_id.into(),
        }
    }
}

/// 搜索文章
#[derive(Debug, Clone, Deserialize)]
pub struct SearchArticleDto {
    /// 全文搜索
    pub full_text: Option<String>,
    /// 状态
    pub status: Option<ArticleStatus>,
    /// 发布时间（大于等于）
    pub published_at_ge: Option<i64>,
    /// 发布时间（小于）
    pub published_at_lt: Option<i64>,
    /// 分页页码
    pub page: Option<u64>,
    /// 分页大小
    pub size: Option<u64>,
}

impl<'a> Into<SearchArticleBo<'a>> for SearchArticleDto {
    fn into(self) -> SearchArticleBo<'a> {
        SearchArticleBo {
            full_text: self.full_text.map(Into::into),
            status: self.status,
            published_at_ge: self.published_at_ge,
            published_at_lt: self.published_at_lt,
            page: self.page,
            size: self.size,
            sort: None,
        }
    }
}

/// 文章列表项
#[derive(Debug, Clone, Serialize)]
pub struct ArticleListItemDto {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 是否需要密码访问
    pub need_password: bool,
}

impl From<ArticleListItemBo> for ArticleListItemDto {
    fn from(value: ArticleListItemBo) -> Self {
        Self {
            article_id: value.article_id,
            title: value.title,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            published_at: value.published_at,
            need_password: value.need_password,
        }
    }
}

/// 文章列表
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct ArticleListDto(PageData<Vec<ArticleListItemDto>>);

impl From<ArticleListBo> for ArticleListDto {
    fn from(value: ArticleListBo) -> Self {
        Self(PageData {
            items: value
                .data
                .items
                .into_iter()
                .map(ArticleListItemDto::from)
                .collect(),
            count: value.data.count,
            total: value.data.total,
        })
    }
}

/// 获取文章详情
#[derive(Debug, Clone, Deserialize)]
pub struct GetArticleDto {
    /// 文章ID
    pub article_id: String,
}

impl<'a> Into<GetArticleBo<'a>> for GetArticleDto {
    fn into(self) -> GetArticleBo<'a> {
        GetArticleBo {
            article_id: self.article_id.into(),
            ignore_status: false,
        }
    }
}

/// 访客可见的文章详情
#[derive(Debug, Clone, Serialize)]
pub struct VisitorArticleDetailsDto {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 渲染后的 HTML 结果
    pub render_content: String,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 是否需要密码访问
    pub need_password: bool,
    /// 附件列表
    pub attachments: Vec<ArticleAttachmentDto>,
    /// 累计页面访问量
    pub pv: u64,
    /// 累计独立访客数
    pub uv: u64,
}

impl From<VisitorArticleDetailsBo> for VisitorArticleDetailsDto {
    fn from(value: VisitorArticleDetailsBo) -> Self {
        Self {
            article_id: value.article_id,
            title: value.title,
            excerpt: value.excerpt,
            markdown_content: value.markdown_content,
            render_content: value.render_content,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            published_at: value.published_at,
            need_password: value.need_password,
            attachments: value
                .attachments
                .into_iter()
                .map(ArticleAttachmentDto::from)
                .collect(),
            pv: value.pv,
            uv: value.uv,
        }
    }
}

/// 管理员可见的文章详情
#[derive(Debug, Clone, Serialize)]
pub struct AdminArticleDetailsDto {
    /// 文章ID
    pub article_id: String,
    /// 标题
    pub title: String,
    /// 摘要
    pub excerpt: String,
    /// Markdown 格式的正文
    pub markdown_content: String,
    /// 渲染后的 HTML 结果
    pub render_content: String,
    /// 访问密码
    pub password: Option<String>,
    /// 状态
    pub status: ArticleStatus,
    /// 创建时间
    pub created_at: i64,
    /// 修改时间
    pub updated_at: i64,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 是否需要密码访问
    pub need_password: bool,
    /// 附件列表
    pub attachments: Vec<ArticleAttachmentDto>,
    /// 累计页面访问量
    pub pv: u64,
    /// 累计独立访客数
    pub uv: u64,
}

impl From<AdminArticleDetailsBo> for AdminArticleDetailsDto {
    fn from(value: AdminArticleDetailsBo) -> Self {
        Self {
            article_id: value.article_id,
            title: value.title,
            excerpt: value.excerpt,
            markdown_content: value.markdown_content,
            render_content: value.render_content,
            password: value.password,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            published_at: value.published_at,
            need_password: value.need_password,
            attachments: value
                .attachments
                .into_iter()
                .map(ArticleAttachmentDto::from)
                .collect(),
            pv: value.pv,
            uv: value.uv,
        }
    }
}

/// 文章详情
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ArticleDetailsDto {
    Visitor(VisitorArticleDetailsDto),
    Admin(AdminArticleDetailsDto),
}

impl From<ArticleDetailsBo> for ArticleDetailsDto {
    fn from(value: ArticleDetailsBo) -> Self {
        match value {
            ArticleDetailsBo::Visitor(bo) => ArticleDetailsDto::Visitor(bo.into()),
            ArticleDetailsBo::Admin(bo) => ArticleDetailsDto::Admin(bo.into()),
        }
    }
}

/// 文章附件
#[derive(Debug, Clone)]
pub struct ArticleAttachmentDto {
    /// 附件ID
    pub attachment_id: String,
    /// 文章ID
    pub article_id: String,
    /// 附件名
    pub name: String,
    /// 附件扩展名
    pub extension: String,
    /// 附件大小
    pub size: u64,
    /// 附件类型
    pub mime_type: String,
    /// 附件哈希
    pub sha256: String,
    /// 创建时间
    pub created_at: i64,
}

impl From<ArticleAttachmentBo> for ArticleAttachmentDto {
    fn from(value: ArticleAttachmentBo) -> Self {
        Self {
            attachment_id: value.attachment_id,
            article_id: value.article_id,
            name: value.name,
            extension: value.extension,
            size: value.size,
            mime_type: value.mime_type,
            sha256: value.sha256,
            created_at: value.created_at,
        }
    }
}

impl Serialize for ArticleAttachmentDto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArticleAttachmentDto", 9)?;

        state.serialize_field("attachment_id", &self.attachment_id)?;
        state.serialize_field("article_id", &self.article_id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("extension", &self.extension)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("mime_type", &self.mime_type)?;
        state.serialize_field("sha256", &self.sha256)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field(
            "url",
            &format!(
                "/articles/{}/attachments/{}",
                self.article_id, self.attachment_id
            ),
        )?;

        state.end()
    }
}

/// 上传文章附件
pub struct UploadArticleAttachmentDto {
    /// 文章ID
    pub article_id: String,
    /// 文章附件
    pub attachment: UploadResourceDto,
}

impl UploadArticleAttachmentDto {
    pub fn from_http(headers: &HeaderMap, data: Body) -> Result<Self, AppError> {
        let article_id = crate::util::http::typed_header::<String>(headers, "x-article-id")
            .map_err(|e| AppErrorMeta::BadRequest.with_message(e))?;
        Ok(Self {
            article_id,
            attachment: UploadResourceDto::from_http(headers, data)?,
        })
    }
}

impl FromRequest for UploadArticleAttachmentDto {
    type Error = AppError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        let body = std::mem::take(request.body_mut());
        Self::from_http(request.headers(), body)
    }
}

impl<'a> Into<UploadArticleAttachmentBo<'a>> for UploadArticleAttachmentDto {
    fn into(self) -> UploadArticleAttachmentBo<'a> {
        UploadArticleAttachmentBo {
            article_id: self.article_id.into(),
            attachment: self.attachment.into(),
        }
    }
}

/// 删除文章附件
#[derive(Debug, Clone, Deserialize)]
pub struct RemoveArticleAttachmentDto {
    /// 文章ID
    pub article_id: String,
    /// 附件ID
    pub attachment_id: String,
}

impl<'a> Into<RemoveArticleAttachmentBo<'a>> for RemoveArticleAttachmentDto {
    fn into(self) -> RemoveArticleAttachmentBo<'a> {
        RemoveArticleAttachmentBo {
            article_id: self.article_id.into(),
            attachment_id: self.attachment_id.into(),
        }
    }
}

/// 下载文章附件
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadArticleAttachmentDto {
    /// 文章ID
    pub article_id: String,
    /// 附件ID
    pub attachment_id: String,
}

impl<'a> Into<DownloadArticleAttachmentBo<'a>> for DownloadArticleAttachmentDto {
    fn into(self) -> DownloadArticleAttachmentBo<'a> {
        DownloadArticleAttachmentBo {
            article_id: self.article_id.into(),
            attachment_id: self.attachment_id.into(),
        }
    }
}
