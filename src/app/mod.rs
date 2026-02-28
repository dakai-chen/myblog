mod api;
mod web;

use std::convert::Infallible;
use std::sync::Arc;

use boluo::BoxError;
use boluo::data::Extension;
use boluo::http::StatusCode;
use boluo::request::Request;
use boluo::response::{Html, IntoResponse, Response};
use boluo::route::Router;
use boluo::service::{Service, ServiceExt};
use boluo::static_file::ServeDir;

use crate::context::auth::AdminCleanCookie;
use crate::error::AppErrorMeta;
use crate::middleware::error::OrElseWith;
use crate::middleware::limit::BodyLimit;
use crate::middleware::logger::{Logger, LoggerSpan};
use crate::middleware::request_id::RequestIdMiddleware;
use crate::middleware::visitor::VisitorMiddleware;
use crate::model::vo::article::UnlockArticleVo;
use crate::model::vo::error::{Err404Vo, Err405Vo, ErrOtherVo};
use crate::state::AppState;
use crate::template::render::PageContext;

pub async fn build(
    state: Arc<AppState>,
) -> anyhow::Result<impl Service<Request, Response = impl IntoResponse, Error = BoxError>> {
    Ok(Router::new()
        .merge_with(build_static_file(), OrElseWith::new(error_to_status_code))
        .merge_with(build_download(), OrElseWith::new(error_to_status_code))
        .merge_with(build_web(), OrElseWith::new(error_to_status_code))
        .scope_merge("/api/", build_api())
        .with(VisitorMiddleware::new())
        .with(BodyLimit::new(&crate::config::get().body_limit)?)
        .with(Extension(state))
        .or_else(handle_error)
        .with(Logger)
        .with(RequestIdMiddleware::with_header_name("x-request-id"))
        .with(LoggerSpan))
}

fn build_static_file() -> Router {
    let theme_assets = ServeDir::new(&crate::config::get().theme.current().assets_dir);
    let public = ServeDir::new(&crate::config::get().resource.public_dir);

    Router::new()
        .scope("/theme/assets/{*}", theme_assets)
        .scope("/{*}", public)
}

fn build_download() -> Router {
    Router::new()
        .mount(api::resource::download_resource)
        .mount(api::article::download_attachment)
}

fn build_web() -> Router {
    Router::new()
        .mount(web::feed::rss)
        .mount(web::feed::atom)
        .mount(web::home::home)
        .mount(web::about::about)
        .mount(web::article::list)
        .mount(web::article::detail)
        .mount(web::article::unlock)
        .mount(web::article::create)
        .mount(web::article::create_submit)
        .mount(web::article::update)
        .mount(web::article::update_submit)
        .mount(web::auth::login)
        .mount(web::auth::login_submit)
        .mount(web::auth::logout)
        .mount(web::auth::logout_submit)
}

fn build_api() -> Router {
    Router::new()
        // 系统模块路由
        .mount(api::system::info)
        .mount(api::system::get_log_level)
        .mount(api::system::set_log_level)
        .mount(api::system::get_shutdown_timeout)
        .mount(api::system::set_shutdown_timeout)
        // 认证模块路由
        .mount(api::auth::login)
        // 资源模块路由
        .mount(api::resource::upload_resource)
        .mount(api::resource::remove_resource)
        // 文章模块路由
        .mount(api::article::create_article)
        .mount(api::article::update_article)
        .mount(api::article::remove_article)
        .mount(api::article::unlock_article)
        .mount(api::article::search_article)
        .mount(api::article::get_article)
        .mount(api::article::upload_attachment)
        .mount(api::article::remove_attachment)
}

async fn handle_error(error: BoxError) -> Result<Response, Infallible> {
    Ok(crate::response::error(error))
}

async fn error_to_status_code(
    error: BoxError,
    state: Extension<Arc<AppState>>,
) -> Result<Response, BoxError> {
    let app_error = crate::error::into_app_error(error);
    tracing::error!("{app_error}");

    if let AppErrorMeta::ArticleLocked { article_id, title } = app_error.meta() {
        let vo = UnlockArticleVo { article_id, title };
        let context = PageContext::new(vo);
        let html = state.template.typed_render(&context);
        return (app_error.meta().status_code(), Html(html)).into_response();
    }

    if let AppErrorMeta::AdminAccessTokenExpired
    | AppErrorMeta::AdminAccessTokenNotEffective
    | AppErrorMeta::AdminAccessTokenInvalid = app_error.meta()
    {
        let context = PageContext::new(ErrOtherVo::from(&app_error));
        let html = state.template.typed_render(&context);
        return (app_error.meta().status_code(), AdminCleanCookie, Html(html)).into_response();
    }

    let html = match app_error.meta().status_code() {
        StatusCode::NOT_FOUND => state
            .template
            .typed_render(&PageContext::new(Err404Vo::from(&app_error))),
        StatusCode::METHOD_NOT_ALLOWED => state
            .template
            .typed_render(&PageContext::new(Err405Vo::from(&app_error))),
        _ => state
            .template
            .typed_render(&PageContext::new(ErrOtherVo::from(&app_error))),
    };
    (app_error.meta().status_code(), Html(html)).into_response()
}
