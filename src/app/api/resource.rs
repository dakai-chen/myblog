use boluo::BoxError;
use boluo::data::Json;
use boluo::extract::Path;
use boluo::request::Request;
use boluo::response::IntoResponse;
use boluo::static_file::ServeFile;

use crate::context::auth::Admin;
use crate::context::db::DbPoolConnection;
use crate::error::AppErrorMeta;
use crate::model::common::resource::ResourceKind;
use crate::model::dto::api::resource::{
    DownloadResourceDto, RemoveResourceDto, ResourceDto, UploadResourceDto,
};
use crate::validator::Validation;

#[boluo::route("/resource/upload", method = "POST")]
pub async fn upload(
    _: Admin,
    params: UploadResourceDto,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let resource = crate::service::resource::save_resource(params.into(), &mut db).await?;
    Ok(crate::response::ok(ResourceDto::from(resource)))
}

#[boluo::route("/resource/remove", method = "POST")]
pub async fn remove(
    _: Admin,
    Json(params): Json<RemoveResourceDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    crate::service::resource::remove_resource(&params.into(), &mut db).await?;
    Ok(crate::response::ok(()))
}

#[boluo::route("/resources/{resource_id}", method = "GET")]
pub async fn download(
    Path(params): Path<DownloadResourceDto>,
    DbPoolConnection(mut db): DbPoolConnection,
    request: Request,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let Some(resource) = crate::service::resource::find_resource(&params.into(), &mut db).await?
    else {
        return Err(AppErrorMeta::NotFound.into_error().into());
    };
    if resource.kind != ResourceKind::Public {
        return Err(AppErrorMeta::NotFound.into_error().into());
    }
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
