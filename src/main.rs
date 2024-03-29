use actix_web::{ HttpServer, App, web };
use api::user::get_user_by_id;
use crate::api::check_version::check_version;
use crate::api::user::{ create_user, logout_user };
use crate::{ database::mongo::Mongo };
use dotenv::dotenv;

pub mod api;
pub mod database;
pub mod models;
pub mod helpers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .app_data(
                web::Data::new({
                    let db = Mongo::init();
                    db
                })
            )
            .route("/user", web::post().to(create_user))
            .route("/user/{id}", web::get().to(get_user_by_id))
            .route("/logout", web::post().to(logout_user))
            .route("/check-version", web::get().to(check_version))
    })
        .bind("127.0.0.1:8080")?
        .run().await
}
