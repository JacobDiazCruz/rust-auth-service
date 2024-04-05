use actix_web::{ web, Result, HttpResponse, http::header::HeaderValue };
use mongodb::results::InsertOneResult;
use crate::{
    database::mongo::Mongo,
    models::user_model::{ User, Email },
    helpers::errors::{ ServiceError, ErrorMessages },
    helpers::form_data::{ LoginForm },
    helpers::obj_id_converter::Converter,
    helpers::jwt::{ sign_jwt, get_token },
};
use std::env;
use serde_json::{ json, Value };
use google_oauth::{ AsyncClient, GooglePayload };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    access_token: String,
    user: User,
}

pub async fn create_user_service(
    db: web::Data<Mongo>,
    user: User
) -> Result<InsertOneResult, ServiceError> {
    match db.create_user(user) {
        Ok(insert_result) => Ok(insert_result),
        Err(_) => Err(ServiceError::BadRequest(ErrorMessages::CreateUserError.error_msg())),
    }
}

pub async fn get_user_by_id_service(
    db: web::Data<Mongo>,
    user_id: String
) -> Result<Option<User>, ServiceError> {
    let obj_id = Converter::string_to_bson(user_id)?;
    match db.get_user_by_id(obj_id) {
        Ok(insert_result) => Ok(insert_result),
        Err(_) => Err(ServiceError::BadRequest(ErrorMessages::CreateUserError.error_msg())),
    }
}

pub async fn check_google_payload(id_token: String) -> Result<GooglePayload, ServiceError> {
    let client_id: String = env
        ::var("GOOGLE_CLIENT_ID")
        .expect("GOOGLE_CLIENT_ID environment variable not set");
    let client = AsyncClient::new(client_id);
    let payload_result = client.validate_id_token(id_token).await;
    let payload = match payload_result {
        Ok(payload) => payload,
        Err(_) => {
            return Err(ServiceError::BadRequest(ErrorMessages::InvalidToken.error_msg()));
        }
    };
    return Ok(payload);
}

pub async fn login_google_user_service(
    db: web::Data<Mongo>,
    form: web::Json<LoginForm>
) -> Result<LoginResponse, ServiceError> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();

    let id_token = form.id_token.clone();
    let payload = check_google_payload(id_token).await?;

    if payload.at_hash.is_none() || payload.azp.is_none() || payload.email.is_none() {
        return Err(ServiceError::BadRequest(ErrorMessages::InvalidToken.error_msg()));
    }

    let user = db.get_user_by_email(email_str);
    let access_token = sign_jwt()?;

    match user {
        Ok(Some(data)) => {
            let response = LoginResponse {
                access_token,
                user: data,
            };
            Ok(response)
        }
        Ok(None) => {
            let payload: User = User::new(name, email);
            let new_user = create_user_service(db.clone(), payload).await;
            match new_user {
                Ok(insert_result) => {
                    let user_details = get_user_by_id_service(
                        db,
                        insert_result.inserted_id.to_string()
                    ).await;
                    match user_details {
                        Ok(Some(data)) => {
                            let response = LoginResponse {
                                access_token,
                                user: data,
                            };
                            Ok(response)
                        }
                        Ok(None) =>
                            Err(ServiceError::BadRequest(ErrorMessages::UserNotExist.error_msg())),
                        Err(_) =>
                            Err(
                                ServiceError::BadRequest(ErrorMessages::UserFetchError.error_msg())
                            ),
                    }
                }
                Err(_) => Err(ServiceError::BadRequest(ErrorMessages::UserFetchError.error_msg())),
            }
        }
        Err(_) => Err(ServiceError::BadRequest(ErrorMessages::UserFetchError.error_msg())),
    }
}

pub async fn logout_user_service(
    db: web::Data<Mongo>,
    auth_header: Option<&HeaderValue>
) -> Result<Value, String> {
    let token = get_token(auth_header);

    match token {
        Ok(val) => {
            println!("Access Token: {}", val);
            let res = db.store_invalidated_token(val);
            let response =
                json!({
                "message": "User logged out successfully!"
            });
            match res {
                Ok(_) => Ok(response),
                Err(_) => Err(ErrorMessages::InvalidateTokenError.error_msg()),
            }
        }
        Err(_) => Err(ErrorMessages::InvalidateTokenError.error_msg()),
    }
}
