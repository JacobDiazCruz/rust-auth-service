use actix_web::{ web, Result, http::header::HeaderValue };
use mongodb::results::InsertOneResult;
use crate::{
    database::mongo::Mongo,
    models::user_model::{ User, Email },
    models::oauth_model::Oauth,
    helpers::errors::{ ServiceError, ErrorMessages },
    helpers::form_data::LoginForm,
    helpers::obj_id_converter::Converter,
    helpers::jwt::{ sign_jwt, get_token },
};
use serde_json::{ json, Value };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
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

pub async fn login_google_user_service(
    db: web::Data<Mongo>,
    form: web::Json<LoginForm>
) -> Result<LoginResponse, ServiceError> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();

    let id_token = form.id_token.clone();
    let payload = Oauth::validate_google_token(id_token).await?;

    if payload.at_hash.is_none() || payload.azp.is_none() || payload.email.is_none() {
        return Err(ServiceError::BadRequest(ErrorMessages::InvalidToken.error_msg()));
    }

    let user = db.get_user_by_email(email_str);
    let access_token = sign_jwt()?;
    let refresh_token = sign_jwt()?;

    if let Some(data) = user.unwrap() {
        let response = LoginResponse {
            access_token,
            refresh_token,
            user: data,
        };
        return Ok(response);
    }

    let new_user_payload: User = User::new(name, email);
    let new_user = create_user_service(db.clone(), new_user_payload).await?;
    let new_user_details = get_user_by_id_service(db, new_user.inserted_id.to_string()).await?;

    if let Some(data) = new_user_details {
        let response = LoginResponse {
            access_token,
            refresh_token,
            user: data,
        };
        Ok(response)
    } else {
        Err(ServiceError::BadRequest(ErrorMessages::UserNotExist.error_msg()))
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
            let res = db.store_invalidated_token(val.to_string());
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
