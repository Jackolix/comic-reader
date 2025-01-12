use rocket::http::Status;
use rocket::State;
use rocket::serde::json::Json;
use crate::config::AppConfig;

#[derive(serde::Serialize)]
pub struct AuthCheck {
    requires_password: bool,
}

#[get("/auth/check")]
pub fn check_auth(config: &State<AppConfig>) -> Json<AuthCheck> {
    Json(AuthCheck {
        requires_password: config.server_password.is_some()
    })
}

#[options("/auth/check")]
pub fn auth_check_options() -> Status {
    Status::NoContent
}