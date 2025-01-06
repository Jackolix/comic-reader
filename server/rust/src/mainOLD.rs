mod models;

#[macro_use] extern crate rocket;

use rocket::{State, Request};
use rocket::http::{Status, ContentType, Header};
use rocket::response::{self, Response, Responder};
use rocket::serde::json::Json;
use rocket::response::status::Custom;
use rocket::request::{self, FromRequest, Outcome};
use serde::Serialize;
use std::path::Path;
use std::env;
use std::fs;
use zip::ZipArchive;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Serialize)]
struct Comic {
    id: String,
    name: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct AuthCheck {
    requires_password: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

struct AppConfig {
    comics_dir: String,
    server_password: Option<String>,
}

// Custom responder for binary data
struct BinaryResponse {
    data: Vec<u8>,
    content_type: ContentType,
    filename: Option<String>,
}

impl<'r> Responder<'r, 'static> for BinaryResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let mut response = Response::build();
        response.header(self.content_type);

        if let Some(filename) = self.filename {
            response.header(Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename)
            ));
        }

        response.sized_body(self.data.len(), Cursor::new(self.data));
        Ok(response.finalize())
    }
}

// Authentication guard
struct AuthGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuard {
    type Error = Custom<Json<ErrorResponse>>;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = request.guard::<&State<AppConfig>>().await.succeeded().unwrap();

        if config.server_password.is_none() {
            return Outcome::Success(AuthGuard);
        }

        let auth_header = match request.headers().get_one("Authorization") {
            Some(h) => h,
            None => {
                return Outcome::Error((
                    Status::Unauthorized,
                    Custom(Status::Unauthorized, Json(ErrorResponse {
                        error: "Authentication required".to_string()
                    }))
                ));
            }
        };

        if !auth_header.starts_with("Basic ") {
            return Outcome::Error((
                Status::Unauthorized,
                Custom(Status::Unauthorized, Json(ErrorResponse {
                    error: "Invalid authentication format".to_string()
                }))
            ));
        }

        let credentials = auth_header[6..].trim();
        let decoded = match BASE64.decode(credentials) {
            Ok(d) => d,
            Err(_) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    Custom(Status::Unauthorized, Json(ErrorResponse {
                        error: "Invalid authentication format".to_string()
                    }))
                ));
            }
        };

        let credentials_str = match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    Custom(Status::Unauthorized, Json(ErrorResponse {
                        error: "Invalid authentication format".to_string()
                    }))
                ));
            }
        };

        let parts: Vec<&str> = credentials_str.split(':').collect();
        if parts.len() != 2 || parts[1] != config.server_password.as_ref().unwrap() {
            return Outcome::Error((
                Status::Unauthorized,
                Custom(Status::Unauthorized, Json(ErrorResponse {
                    error: "Invalid password".to_string()
                }))
            ));
        }

        Outcome::Success(AuthGuard)
    }
}

#[get("/auth/check")]
fn check_auth(config: &State<AppConfig>) -> Json<AuthCheck> {
    Json(AuthCheck {
        requires_password: config.server_password.is_some()
    })
}

#[get("/comics")]
async fn list_comics(_auth: AuthGuard, config: &State<AppConfig>) -> Result<Json<Vec<Comic>>, Custom<Json<ErrorResponse>>> {
    let comics_dir = Path::new(&config.comics_dir);
    let entries = fs::read_dir(comics_dir).map_err(|_| {
        Custom(Status::InternalServerError, Json(ErrorResponse {
            error: "Failed to read comics directory".to_string()
        }))
    })?;

    let mut comics = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| {
            Custom(Status::InternalServerError, Json(ErrorResponse {
                error: "Failed to read directory entry".to_string()
            }))
        })?;

        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension.to_string_lossy().to_lowercase() == "cbz" {
                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                let name = path.file_stem().unwrap().to_string_lossy().into_owned();
                let id = urlencoding::encode(&file_name);
                comics.push(Comic {
                    id: id.to_string(),
                    name,
                    path: format!("/comics/{}", id),
                });
            }
        }
    }

    Ok(Json(comics))
}

#[get("/covers/<filename>")]
async fn get_cover(
    _auth: AuthGuard,
    config: &State<AppConfig>,
    filename: String
) -> Result<BinaryResponse, Custom<Json<ErrorResponse>>> {
    let decoded_filename = urlencoding::decode(&filename).map_err(|_| {
        Custom(Status::BadRequest, Json(ErrorResponse {
            error: "Invalid filename".to_string()
        }))
    })?.into_owned();

    let file_path = Path::new(&config.comics_dir).join(&decoded_filename);
    let mut file = File::open(&file_path).await.map_err(|_| {
        Custom(Status::NotFound, Json(ErrorResponse {
            error: "Comic file not found".to_string()
        }))
    })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.map_err(|_| {
        Custom(Status::InternalServerError, Json(ErrorResponse {
            error: "Failed to read comic file".to_string()
        }))
    })?;

    let mut archive = ZipArchive::new(Cursor::new(buffer)).map_err(|_| {
        Custom(Status::InternalServerError, Json(ErrorResponse {
            error: "Failed to process comic file".to_string()
        }))
    })?;

    // Find first image file in archive
    let mut image_data = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|_| {
            Custom(Status::InternalServerError, Json(ErrorResponse {
                error: "Failed to read archive entry".to_string()
            }))
        })?;

        let name = file.name().to_lowercase();
        if name.ends_with(".jpg") || name.ends_with(".jpeg") ||
            name.ends_with(".png") || name.ends_with(".gif") {
            let mut buf = Vec::new();
            std::io::copy(&mut file, &mut buf).map_err(|_| {
                Custom(Status::InternalServerError, Json(ErrorResponse {
                    error: "Failed to read image data".to_string()
                }))
            })?;
            image_data = Some(buf);
            break;
        }
    }

    let image_data = image_data.ok_or_else(|| {
        Custom(Status::NotFound, Json(ErrorResponse {
            error: "No cover image found in comic".to_string()
        }))
    })?;

    Ok(BinaryResponse {
        data: image_data,
        content_type: ContentType::JPEG,
        filename: None,
    })
}

#[get("/comics/<filename>")]
async fn get_comic(
    _auth: AuthGuard,
    config: &State<AppConfig>,
    filename: String
) -> Result<BinaryResponse, Custom<Json<ErrorResponse>>> {
    let decoded_filename = urlencoding::decode(&filename).map_err(|_| {
        Custom(Status::BadRequest, Json(ErrorResponse {
            error: "Invalid filename".to_string()
        }))
    })?.into_owned();

    let file_path = Path::new(&config.comics_dir).join(&decoded_filename);
    let mut file = File::open(&file_path).await.map_err(|_| {
        Custom(Status::NotFound, Json(ErrorResponse {
            error: "Comic file not found".to_string()
        }))
    })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.map_err(|_| {
        Custom(Status::InternalServerError, Json(ErrorResponse {
            error: "Failed to read comic file".to_string()
        }))
    })?;

    Ok(BinaryResponse {
        data: buffer,
        content_type: ContentType::ZIP,
        filename: Some(decoded_filename),
    })
}

#[launch]
fn rocket() -> _ {
    let comics_dir = env::var("COMICS_DIR").unwrap_or_else(|_| String::from("D:\\Nextcloud\\Comics\\Its-Jeff"));
    let server_password = env::var("SERVER_PASSWORD").ok();

    let config = AppConfig {
        comics_dir,
        server_password,
    };

    rocket::build()
        .manage(config)
        .mount("/", routes![
            check_auth,
            list_comics,
            get_cover,
            get_comic
        ])
}