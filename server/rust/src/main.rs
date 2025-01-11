#[macro_use] extern crate rocket;

mod config;
mod models;
mod routes;
mod services;
mod utils;

use std::path::PathBuf;
use crate::config::AppConfig;
use crate::services::comic_service::ComicService;
use crate::utils::cors::CORS;

#[launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
    let comics_dir = std::env::var("COMICS_DIR")
        .unwrap_or_else(|_| String::from("/comics"));
    let server_password = std::env::var("SERVER_PASSWORD").ok();

    let config = AppConfig {
        comics_dir: comics_dir.clone(),
        server_password,
    };

    // Initialize comic service
    let comic_service = ComicService::new(PathBuf::from(comics_dir))
        .await
        .expect("Failed to initialize comic service");

    rocket::build()
        .attach(CORS)
        .manage(config)
        .manage(comic_service)
        .mount("/", routes![
            routes::auth::check_auth,
            routes::comics::list_comics,
            routes::comics::get_cover,
            routes::comics::get_comic,
        ])
}