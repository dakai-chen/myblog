use boluo::BoxError;
use boluo::data::Json;
use boluo::extract::Path;
use boluo::request::Request;
use boluo::response::IntoResponse;
use boluo::static_file::ServeFile;

use crate::context::auth::Admin;
use crate::context::db::DbPoolConnection;
use crate::error::AppErrorMeta;
use crate::model::bo::visitor::VisitorBo;
use crate::model::dto::api::article::{
    ArticleAttachmentDto, ArticleDetailsDto, ArticleListDto, CreateArticleDto,
    DownloadArticleAttachmentDto, GetArticleDto, RemoveArticleAttachmentDto, RemoveArticleDto,
    SearchArticleDto, UnlockArticleDto, UpdateArticleDto, UploadArticleAttachmentDto,
};
use crate::validator::Validation;

#[boluo::route("/article/unlock", method = "POST")]
pub async fn unlock_article(
    visitor: VisitorBo,
    Json(params): Json<UnlockArticleDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    crate::service::article::unlock_article(&visitor, &params.into(), &mut db).await?;
    Ok(crate::response::ok(()))
}

#[boluo::route("/article/create", method = "POST")]
pub async fn create_article(
    _: Admin,
    Json(params): Json<CreateArticleDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let article = crate::service::article::create_article(params.into(), &mut db).await?;
    Ok(crate::response::ok(serde_json::json!({
        "id": article.article_id
    })))
}

#[boluo::route("/article/update", method = "POST")]
pub async fn update_article(
    _: Admin,
    Json(params): Json<UpdateArticleDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    crate::service::article::update_article(params.into(), &mut db).await?;
    Ok(crate::response::ok(()))
}

#[boluo::route("/article/remove", method = "POST")]
pub async fn remove_article(
    _: Admin,
    Json(params): Json<RemoveArticleDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    crate::service::article::remove_article(&params.into(), &mut db).await?;
    Ok(crate::response::ok(()))
}

#[boluo::route("/article/search", method = "POST")]
pub async fn search_article(
    admin: Option<Admin>,
    Json(params): Json<SearchArticleDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let list =
        crate::service::article::search_article(admin.as_deref(), &params.into(), &mut db).await?;
    Ok(crate::response::ok(ArticleListDto::from(list)))
}

#[boluo::route("/article/get", method = "POST")]
pub async fn get_article(
    admin: Option<Admin>,
    visitor: VisitorBo,
    Json(params): Json<GetArticleDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let Some(details) =
        crate::service::article::get_article(admin.as_deref(), &visitor, &params.into(), &mut db)
            .await?
    else {
        return Err(AppErrorMeta::NotFound.with_message("文章不存在").into());
    };
    Ok(crate::response::ok(ArticleDetailsDto::from(details)))
}

#[boluo::route("/article/upload_attachment", method = "POST")]
pub async fn upload_attachment(
    _: Admin,
    params: UploadArticleAttachmentDto,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let attachment = crate::service::article::upload_attachment(params.into(), &mut db).await?;
    Ok(crate::response::ok(ArticleAttachmentDto::from(attachment)))
}

#[boluo::route("/article/remove_attachment", method = "POST")]
pub async fn remove_attachment(
    _: Admin,
    Json(params): Json<RemoveArticleAttachmentDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    crate::service::article::remove_attachment(&params.into(), &mut db).await?;
    Ok(crate::response::ok(()))
}

#[boluo::route("/articles/{article_id}/attachments/{attachment_id}", method = "GET")]
pub async fn download_attachment(
    admin: Option<Admin>,
    visitor: VisitorBo,
    Path(params): Path<DownloadArticleAttachmentDto>,
    DbPoolConnection(mut db): DbPoolConnection,
    request: Request,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let Some(resource) = crate::service::article::download_attachment(
        admin.as_deref(),
        &visitor,
        &params.into(),
        &mut db,
    )
    .await?
    else {
        return Err(AppErrorMeta::HttpNotFound.into_error().into());
    };
    let response = ServeFile::new(resource.path.absolute())
        .call(request)
        .await?;
    let encoded_filename = urlencoding::encode(&resource.name);
    let response_headers = [
        ("Content-Type", resource.mime_type),
        (
            "Content-Disposition",
            format!("filename={encoded_filename}"),
        ),
    ];
    (response_headers, response).into_response()
}
