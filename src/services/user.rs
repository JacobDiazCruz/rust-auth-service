use actix_web::{ web, Result, http::header::HeaderValue };
use mongodb::results::InsertOneResult;
use crate::{
    database::mongo::Mongo,
    models::user_model::{ User, Email, Password },
    helpers::errors::{ ServiceError, ErrorMessages },
    helpers::form_data::LoginForm,
    helpers::obj_id_converter::Converter,
    helpers::{ jwt::{ sign_jwt, get_token }, form_data::ManualLoginForm },
};
use serde_json::{ json, Value };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
    user: User,
}

pub async fn register_user_service(
    db: web::Data<Mongo>,
    form: web::Json<ManualLoginForm>
) -> Result<InsertOneResult, ServiceError> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let password = Password::parse(String::from(&form.password))?;
    let hashed_password = Password::hash(&password);

    let user = User {
        id: None,
        name,
        email,
        password: Some(hashed_password.unwrap()),
    };

    match db.create_user(user) {
        Ok(insert_result) => Ok(insert_result),
        Err(_) => Err(ServiceError::BadRequest(ErrorMessages::CreateUserError.error_msg())),
    }
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

fn login_response(data: User) -> Result<LoginResponse, ServiceError> {
    let user_id_str = match data.id {
        Some(object_id) => object_id.to_hex(),
        None => {
            return Err(ServiceError::InternalServerError("User ID not found.".to_string()));
        }
    };
    let access_token = sign_jwt(&user_id_str)?;
    let refresh_token = sign_jwt(&user_id_str)?;
    let response = LoginResponse {
        access_token,
        refresh_token,
        user: data,
    };
    return Ok(response);
}

pub async fn login_google_user_service(
    db: web::Data<Mongo>,
    form: web::Json<LoginForm>
) -> Result<LoginResponse, ServiceError> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();

    // Note: Add verify id_token here in the future

    let user = db.get_user_by_email(email_str);

    if let Some(data) = user.unwrap() {
        let response = login_response(data);
        return Ok(response.unwrap());
    }

    let new_user_payload: User = User::new(name, email);
    let new_user = create_user_service(db.clone(), new_user_payload).await?;
    let new_user_details = get_user_by_id_service(db, new_user.inserted_id.to_string()).await?;

    if let Some(data) = new_user_details {
        let response = login_response(data);
        Ok(response.unwrap())
    } else {
        Err(ServiceError::BadRequest(ErrorMessages::UserNotExist.error_msg()))
    }
}

pub async fn logout_user_service(
    db: web::Data<Mongo>,
    auth_header: Option<&HeaderValue>
) -> Result<Value, ServiceError> {
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
                Err(_) =>
                    Err(ServiceError::BadRequest(ErrorMessages::InvalidateTokenError.error_msg())),
            }
        }
        Err(_) => Err(ServiceError::BadRequest(ErrorMessages::InvalidateTokenError.error_msg())),
    }
}
