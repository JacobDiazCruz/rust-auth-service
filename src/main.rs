use actix_web::{ HttpServer, App, web };
use api::user::{ login_google_user_api, logout_user_api, refresh_token_api };
use crate::database::mongo::Mongo;
use dotenv::dotenv;
use actix_cors::Cors;

pub mod api;
pub mod services;
pub mod database;
pub mod models;
pub mod helpers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin().allowed_methods(vec!["GET", "POST"]);

        App::new()
            .app_data(
                web::Data::new({
                    let db = Mongo::init();
                    db
                })
            )
            .wrap(cors)
            .route("/login/google", web::post().to(login_google_user_api))
            .route("/logout", web::post().to(logout_user_api))
            .route("/refresh-token", web::get().to(refresh_token_api))
    })
        .bind("127.0.0.1:8080")?
        .run().await
}
