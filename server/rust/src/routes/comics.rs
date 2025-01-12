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

#[get("/comics/<id>")]
pub async fn get_comic(
    comic_service: &State<ComicService>,
    id: String
) -> Result<BinaryResponse, Status> {
    println!("Requested comic ID: {}", id);

    let data = comic_service.get_comic_data(&id)
        .await
        .map_err(|e| {
            println!("Error getting comic data: {:?}", e);
            match e {
                ComicError::ComicNotFound => Status::NotFound,
                _ => Status::InternalServerError,
            }
        })?;

    let comic = comic_service.get_comic(&id)
        .await
        .ok_or(Status::NotFound)?;

    Ok(BinaryResponse {
        data,
        content_type: ContentType::ZIP,
        filename: Some(comic.file_name),
    })
}

#[get("/covers/<id>")]
pub async fn get_cover(
    comic_service: &State<ComicService>,
    id: String
) -> Result<BinaryResponse, Status> {
    println!("Requested cover ID: {}", id);

    let cover = comic_service.get_cover(&id)
        .await
        .ok_or(Status::NotFound)?;

    Ok(BinaryResponse {
        data: cover.data,
        content_type: ContentType::JPEG,
        filename: None,
    })
}

#[options("/comics")]
pub fn comics_options() -> Status {
    Status::NoContent
}

#[options("/covers/<_id>")]
pub fn cover_options(_id: String) -> Status {
    Status::NoContent
}

#[options("/comics/<_id>")]
pub fn comic_options(_id: String) -> Status {
    Status::NoContent
}