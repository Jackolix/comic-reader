use rocket::State;
use rocket::http::{Status, ContentType};
use rocket::serde::json::Json;

use crate::models::comic::Comic;
use crate::models::error::ComicError;
use crate::services::comic_service::ComicService;
use crate::services::auth::AuthGuard;
use crate::utils::response::BinaryResponse;

#[get("/comics")]
pub async fn list_comics(
    _auth: AuthGuard,
    comic_service: &State<ComicService>
) -> Json<Vec<Comic>> {
    Json(comic_service.get_all_comics().await)
}

#[get("/covers/<id>")]
pub async fn get_cover(
    _auth: AuthGuard,
    comic_service: &State<ComicService>,
    id: String
) -> Result<BinaryResponse, Status> {
    let cover = comic_service.get_cover(&id).await
        .ok_or(Status::NotFound)?;

    Ok(BinaryResponse {
        data: cover.data,
        content_type: ContentType::JPEG,
        filename: None,
    })
}

#[get("/comics/<id>")]
pub async fn get_comic(
    _auth: AuthGuard,
    comic_service: &State<ComicService>,
    id: String
) -> Result<BinaryResponse, Status> {
    let comic = comic_service.get_comic(&id).await
        .ok_or(Status::NotFound)?;

    let data = comic_service.get_comic_data(&id).await
        .map_err(|e| match e {
            ComicError::ComicNotFound => Status::NotFound,
            _ => Status::InternalServerError,
        })?;

    Ok(BinaryResponse {
        data,
        content_type: ContentType::ZIP,
        filename: Some(comic.file_name),
    })
}