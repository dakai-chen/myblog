use std::sync::Arc;

use boluo::BoxError;
use boluo::data::{Extension, Form};
use boluo::extract::Path;
use boluo::response::{Html, IntoResponse, Redirect};

use crate::context::auth::AdminFromCookie;
use crate::context::db::DbPoolConnection;
use crate::error::AppErrorMeta;
use crate::model::bo::visitor::VisitorBo;
use crate::model::dto::web::article::{
    CreateArticleSubmitDto, GetArticleDto, SearchArticleDto, UnlockArticleDto, UpdateArticleDto,
    UpdateArticleSubmitDto,
};
use crate::model::vo::article::{
    ArticleDetailsVo, ArticleListVo, CreateArticleVo, UpdateArticleVo,
};
use crate::state::AppState;
use crate::template::render::PageContext;
use crate::validator::Validation;

#[boluo::route("/articles", method = ["GET"])]
pub async fn list(
    admin: Option<AdminFromCookie>,
    Form(params): Form<SearchArticleDto>,
    Extension(state): Extension<Arc<AppState>>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let list =
        crate::service::article::search_article(admin.as_deref(), &(&params).into(), &mut db)
            .await?;
    let vo = ArticleListVo::from(list, params)?;
    let context = PageContext::new(vo).admin(admin.map(Into::into));
    Ok(Html(state.template.typed_render(&context)))
}

#[boluo::route("/articles/{article_id}", method = ["GET"])]
pub async fn detail(
    admin: Option<AdminFromCookie>,
    visitor: VisitorBo,
    Path(params): Path<GetArticleDto>,
    Extension(state): Extension<Arc<AppState>>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let Some(article) =
        crate::service::article::get_article(admin.as_deref(), &visitor, &params.into(), &mut db)
            .await?
    else {
        return Err(AppErrorMeta::NotFound.with_message("文章不存在").into());
    };
    let vo = ArticleDetailsVo::from(article);
    let context = PageContext::new(vo).admin(admin.map(Into::into));
    Ok(Html(state.template.typed_render(&context)))
}

#[boluo::route("/articles/{article_id}/_unlock", method = ["POST"])]
pub async fn unlock(
    visitor: VisitorBo,
    params: UnlockArticleDto,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    let url = format!("/articles/{}", params.article_id);
    params.validate(&())?;
    crate::service::article::unlock_article(&visitor, &params.into(), &mut db).await?;
    Ok(Redirect::to(&url))
}

#[boluo::route("/articles/_create", method = ["GET"])]
pub async fn create(
    AdminFromCookie(admin): AdminFromCookie,
    Extension(state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, BoxError> {
    let context = PageContext::new(CreateArticleVo).admin(admin);
    Ok(Html(state.template.typed_render(&context)))
}

#[boluo::route("/articles/_create", method = ["POST"])]
pub async fn create_submit(
    AdminFromCookie(_): AdminFromCookie,
    Form(params): Form<CreateArticleSubmitDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let bo = crate::service::article::create_article(params.into(), &mut db).await?;
    Ok(Redirect::to(&format!("/articles/{}", bo.article_id)))
}

#[boluo::route("/articles/{article_id}/_update", method = ["GET"])]
pub async fn update(
    AdminFromCookie(admin): AdminFromCookie,
    visitor: VisitorBo,
    Path(params): Path<UpdateArticleDto>,
    Extension(state): Extension<Arc<AppState>>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let Some(article) =
        crate::service::article::get_article(Some(&admin), &visitor, &params.into(), &mut db)
            .await?
    else {
        return Err(AppErrorMeta::NotFound.with_message("文章不存在").into());
    };
    let vo = UpdateArticleVo::try_from(article)?;
    let context = PageContext::new(vo).admin(admin);
    Ok(Html(state.template.typed_render(&context)))
}

#[boluo::route("/articles/{article_id}/_update", method = ["POST"])]
pub async fn update_submit(
    AdminFromCookie(_): AdminFromCookie,
    params: UpdateArticleSubmitDto,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    let url = format!("/articles/{}", params.article_id);
    params.validate(&())?;
    crate::service::article::update_article(params.into(), &mut db).await?;
    Ok(Redirect::to(&url))
}
