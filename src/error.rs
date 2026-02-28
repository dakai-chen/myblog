use std::convert::Infallible;
use std::num::TryFromIntError;
use std::time::SystemTimeError;

use boluo::extract::{FormError, JsonError, PathError, QueryError, TypedHeaderError};
use boluo::response::RedirectUriError;
use boluo::route::{RouteError, RouteErrorKind};
use boluo::static_file::ServeFileError;
use boluo::{BoxError, http::StatusCode};
use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

use crate::util::pagination::{NumericalOverflow, PageDataCountOverflow, PageValidateError};
use crate::validator::ValidationError;

#[derive(Debug, thiserror::Error)]
pub struct AppError {
    /// 错误元数据
    meta: AppErrorMeta,
    /// 用户可见的友好消息
    message: String,
    /// 上下文信息
    context: Option<String>,
    /// 原始错误
    #[source]
    source: Option<BoxError>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_log_string())
    }
}

impl AppError {
    pub fn new(meta: AppErrorMeta) -> Self {
        Self {
            message: meta.default_message(),
            context: None,
            source: None,
            meta,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_source<T>(mut self, source: T) -> Self
    where
        T: Into<BoxError>,
    {
        self.source = Some(source.into());
        self
    }

    pub fn meta(&self) -> &AppErrorMeta {
        &self.meta
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    pub fn source(&self) -> Option<&BoxError> {
        self.source.as_ref()
    }

    pub fn to_log_string(&self) -> String {
        let mut log = format!("{}: {}", self.meta.code(), self.message);

        if let Some(source) = &self.source {
            log.push_str(&format!(" | SOURCE: {source}"));
        }
        if let Some(context) = &self.context {
            log.push_str(&format!(" | CONTEXT: {context}"));
        }

        log
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AppErrorMeta {
    /// HTTP 路径不存在
    HttpNotFound,
    /// HTTP 方法不支持
    HttpMethodNotAllowed,
    /// 服务器内部错误
    Internal,
    /// 服务暂时不可用
    ServiceUnavailable,
    /// 客户端请求错误
    BadRequest,
    /// 资源不存在
    NotFound,
    /// 管理员访问令牌缺失
    AdminAccessTokenMissing,
    /// 管理员访问令牌过期
    AdminAccessTokenExpired,
    /// 管理员访问令牌未生效
    AdminAccessTokenNotEffective,
    /// 管理员访问令牌无效
    AdminAccessTokenInvalid,
    /// 权限不足
    PermissionDenied,
    /// 数据大小超出限制
    DataTooLarge { limit: u64 },
    /// 文章已锁定
    ArticleLocked { article_id: String, title: String },
}

impl AppErrorMeta {
    pub fn into_error(self) -> AppError {
        self.into()
    }

    pub fn with_message(self, message: impl Into<String>) -> AppError {
        self.into_error().with_message(message)
    }

    pub fn with_context(self, context: impl Into<String>) -> AppError {
        self.into_error().with_context(context)
    }

    pub fn with_source<T>(self, source: T) -> AppError
    where
        T: Into<BoxError>,
    {
        self.into_error().with_source(source)
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppErrorMeta::HttpNotFound => "HttpNotFound",
            AppErrorMeta::HttpMethodNotAllowed => "HttpMethodNotAllowed",
            AppErrorMeta::Internal => "Internal",
            AppErrorMeta::ServiceUnavailable => "ServiceUnavailable",
            AppErrorMeta::BadRequest => "BadRequest",
            AppErrorMeta::NotFound => "NotFound",
            AppErrorMeta::AdminAccessTokenMissing => "AdminAccessTokenMissing",
            AppErrorMeta::AdminAccessTokenExpired => "AdminAccessTokenExpired",
            AppErrorMeta::AdminAccessTokenNotEffective => "AdminAccessTokenNotEffective",
            AppErrorMeta::AdminAccessTokenInvalid => "AdminAccessTokenInvalid",
            AppErrorMeta::PermissionDenied => "PermissionDenied",
            AppErrorMeta::DataTooLarge { .. } => "DataTooLarge",
            AppErrorMeta::ArticleLocked { .. } => "ArticleAccessTokenMissing",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppErrorMeta::HttpNotFound => StatusCode::NOT_FOUND,
            AppErrorMeta::HttpMethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            AppErrorMeta::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorMeta::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppErrorMeta::BadRequest => StatusCode::BAD_REQUEST,
            AppErrorMeta::NotFound => StatusCode::NOT_FOUND,
            AppErrorMeta::AdminAccessTokenMissing => StatusCode::UNAUTHORIZED,
            AppErrorMeta::AdminAccessTokenExpired => StatusCode::UNAUTHORIZED,
            AppErrorMeta::AdminAccessTokenNotEffective => StatusCode::UNAUTHORIZED,
            AppErrorMeta::AdminAccessTokenInvalid => StatusCode::UNAUTHORIZED,
            AppErrorMeta::PermissionDenied => StatusCode::FORBIDDEN,
            AppErrorMeta::DataTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            AppErrorMeta::ArticleLocked { .. } => StatusCode::FORBIDDEN,
        }
    }

    pub fn default_message(&self) -> String {
        match self {
            AppErrorMeta::HttpNotFound => "HTTP 路径不存在".into(),
            AppErrorMeta::HttpMethodNotAllowed => "HTTP 方法不支持".into(),
            AppErrorMeta::Internal => "服务器内部错误".into(),
            AppErrorMeta::ServiceUnavailable => "服务暂时不可用".into(),
            AppErrorMeta::BadRequest => "客户端请求错误".into(),
            AppErrorMeta::NotFound => "资源不存在".into(),
            AppErrorMeta::AdminAccessTokenMissing => "管理员访问令牌缺失".into(),
            AppErrorMeta::AdminAccessTokenExpired => "管理员访问令牌过期".into(),
            AppErrorMeta::AdminAccessTokenNotEffective => "管理员访问令牌未生效".into(),
            AppErrorMeta::AdminAccessTokenInvalid => "管理员访问令牌无效".into(),
            AppErrorMeta::PermissionDenied => "权限不足".into(),
            AppErrorMeta::DataTooLarge { limit } => {
                format!("数据大小超出限制，最大允许 {}", ByteSize::b(*limit))
            }
            AppErrorMeta::ArticleLocked { .. } => "文章已锁定".into(),
        }
    }
}

impl From<AppErrorMeta> for AppError {
    fn from(meta: AppErrorMeta) -> Self {
        Self::new(meta)
    }
}

impl From<Infallible> for AppError {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppErrorMeta::Internal.with_source(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppErrorMeta::Internal.with_source(e.to_string())
    }
}

impl From<TryFromIntError> for AppError {
    fn from(e: TryFromIntError) -> Self {
        AppErrorMeta::Internal.with_source(e)
    }
}

impl From<SystemTimeError> for AppError {
    fn from(e: SystemTimeError) -> Self {
        AppErrorMeta::Internal.with_source(e)
    }
}

impl From<RedirectUriError> for AppError {
    fn from(e: RedirectUriError) -> Self {
        AppErrorMeta::Internal.with_source(e)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppErrorMeta::Internal.with_source(e)
    }
}

impl From<sqlx::error::BoxDynError> for AppError {
    fn from(e: sqlx::error::BoxDynError) -> Self {
        AppErrorMeta::Internal.with_source(e)
    }
}

impl From<PageValidateError> for AppError {
    fn from(e: PageValidateError) -> Self {
        AppErrorMeta::BadRequest.with_message(e.to_string())
    }
}

impl From<NumericalOverflow> for AppError {
    fn from(e: NumericalOverflow) -> Self {
        AppErrorMeta::BadRequest.with_message(e.to_string())
    }
}

impl<D> From<PageDataCountOverflow<D>> for AppError {
    fn from(e: PageDataCountOverflow<D>) -> Self {
        AppErrorMeta::BadRequest.with_message(e.to_string())
    }
}

pub fn into_app_error(error: BoxError) -> AppError {
    let error = match error.downcast::<AppError>() {
        Ok(e) => return *e,
        Err(error) => error,
    };

    if let Some(e) = error.downcast_ref::<RouteError>() {
        return match e.kind() {
            RouteErrorKind::NotFound => AppErrorMeta::HttpNotFound,
            RouteErrorKind::MethodNotAllowed => AppErrorMeta::HttpMethodNotAllowed,
        }
        .into();
    }

    if let Some(e) = error.downcast_ref::<FormError>() {
        return AppErrorMeta::BadRequest.with_message(e.to_string());
    }
    if let Some(e) = error.downcast_ref::<JsonError>() {
        return AppErrorMeta::BadRequest.with_message(e.to_string());
    }
    if let Some(e) = error.downcast_ref::<PathError>() {
        return AppErrorMeta::BadRequest.with_message(e.to_string());
    }
    if let Some(e) = error.downcast_ref::<QueryError>() {
        return AppErrorMeta::BadRequest.with_message(e.to_string());
    }
    if let Some(e) = error.downcast_ref::<TypedHeaderError>() {
        return AppErrorMeta::BadRequest.with_message(e.to_string());
    }
    if let Some(e) = error.downcast_ref::<ServeFileError>() {
        return match e {
            ServeFileError::NotFound => AppErrorMeta::HttpNotFound.into(),
            ServeFileError::IO(_) => AppErrorMeta::Internal.with_source(error),
        };
    }

    if let Some(ValidationError::Validation(message)) = error.downcast_ref::<ValidationError>() {
        return AppErrorMeta::BadRequest.with_message(message);
    }

    AppErrorMeta::Internal.with_source(error)
}
