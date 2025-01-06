use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::http::Status;
use rocket::State;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::config::AppConfig;

pub struct AuthGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuard {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = request.guard::<&State<AppConfig>>().await.succeeded().unwrap();

        if config.server_password.is_none() {
            return Outcome::Success(AuthGuard);
        }

        match request.headers().get_one("Authorization") {
            None => Outcome::Error((Status::Unauthorized, ())),
            Some(auth_header) if !auth_header.starts_with("Basic ") => {
                Outcome::Error((Status::Unauthorized, ()))
            }
            Some(auth_header) => {
                let credentials = &auth_header[6..];
                match BASE64.decode(credentials) {
                    Ok(decoded) => {
                        match String::from_utf8(decoded) {
                            Ok(credentials_str) => {
                                let parts: Vec<&str> = credentials_str.split(':').collect();
                                if parts.len() == 2 && parts[1] == config.server_password.as_ref().unwrap() {
                                    Outcome::Success(AuthGuard)
                                } else {
                                    Outcome::Error((Status::Unauthorized, ()))
                                }
                            }
                            Err(_) => Outcome::Error((Status::Unauthorized, ())),
                        }
                    }
                    Err(_) => Outcome::Error((Status::Unauthorized, ())),
                }
            }
        }
    }
}