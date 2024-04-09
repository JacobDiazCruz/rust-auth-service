use actix_web::{ HttpServer, http, App, web };
use api::user::{
    login_google_user_api,
    logout_user_api,
    refresh_token_api,
    register_user_api,
    manual_login_user_api,
};
use crate::database::mongo::Mongo;
use dotenv::dotenv;
use actix_cors::Cors;

pub mod api;
pub mod services;
pub mod database;
pub mod models;
pub mod config;
pub mod helpers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b"localhost:3000")
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(
                web::Data::new({
                    let db = Mongo::init();
                    db
                })
            )
            .wrap(cors)
            .route("/user/login/google", web::post().to(login_google_user_api))
            .route("/user/login", web::post().to(manual_login_user_api))
            .route("/user/register", web::post().to(register_user_api))
            .route("/logout", web::post().to(logout_user_api))
            .route("/refresh-token", web::get().to(refresh_token_api))
    })
        .bind("127.0.0.1:8080")?
        .run().await
}
