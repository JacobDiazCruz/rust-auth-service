use serde_json::{ json, Value };

use actix_web::{ error::ResponseError, HttpResponse };
use derive_more::Display;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")] InternalServerError(String),
    #[display(fmt = "Unauthorized")] Unauthorized(String),
    #[display(fmt = "BadRequest: {}", _0)] BadRequest(String),
    #[display(fmt = "Forbidden")] Forbidden(String),
}

#[derive(Debug, Display)]
pub enum ErrorMessages {
    InvalidToken,
    UserNotExist,
    EmailAlreadyExist,
    UserFetchError,
    CreateUserError,
    InvalidateTokenError,
}

impl ErrorMessages {
    pub fn error_msg(&self) -> String {
        match self {
            ErrorMessages::InvalidToken => "Invalid token.".to_string(),
            ErrorMessages::UserNotExist => "User does not exist.".to_string(),
            ErrorMessages::EmailAlreadyExist => "Email already exist.".to_string(),
            ErrorMessages::UserFetchError => "Error fetching user.".to_string(),
            ErrorMessages::CreateUserError => "Failed creating user.".to_string(),
            ErrorMessages::InvalidateTokenError => "Failed invalidating token.".to_string(),
        }
    }
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
            ServiceError::InternalServerError(ref message) => {
                let error_json = init_error("Internal Server Error. Please try again.", 500);
                HttpResponse::InternalServerError().json(error_json)
            }
            ServiceError::Unauthorized(ref message) => {
                let error_json = init_error("Unauthorized", 401);
                HttpResponse::InternalServerError().json(error_json)
            }
            ServiceError::BadRequest(ref message) => {
                let error_json = init_error(message, 400);
                HttpResponse::BadRequest().json(error_json)
            }
            ServiceError::Forbidden(ref message) => {
                let error_json = init_error(message, 403);
                HttpResponse::BadRequest().json(error_json)
            }
        }
    }
}
