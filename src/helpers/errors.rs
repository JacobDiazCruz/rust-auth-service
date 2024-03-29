use serde_json::{ json, Value };

use actix_web::{ error::ResponseError, HttpResponse };
use derive_more::Display;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "BadRequest: {}", _0)] BadRequest(String),

    #[display(fmt = "JWKSFetchError")]
    JWKSFetchError,
}

fn init_error(message: &str, status_code: i32) -> Value {
    let error_obj = json!({
        "message": message,
        "status_code": status_code,
    });
    error_obj
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError => {
                let error_json = init_error("Internal Server Error. Please try again.", 500);
                HttpResponse::InternalServerError().json(error_json)
            }
            ServiceError::BadRequest(ref message) => {
                let error_json = init_error(message, 400);
                HttpResponse::BadRequest().json(error_json)
            }
            ServiceError::JWKSFetchError => {
                let error_json = init_error("Could not fetch JWKS", 500);
                HttpResponse::InternalServerError().json(error_json)
            }
        }
    }
}