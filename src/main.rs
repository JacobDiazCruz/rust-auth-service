use crate::database::mongo::Mongo;
use dotenv::dotenv;
use axum::http::{ header::{ ACCEPT, AUTHORIZATION, CONTENT_TYPE }, HeaderValue, Method };
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use route::create_router;

pub mod handlers;
pub mod services;
pub mod database;
pub mod models;
pub mod config;
pub mod helpers;
mod route;

pub struct AppState {
    db: Mongo,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    dotenv().ok();

    let db = Mongo::init();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState { db })).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
