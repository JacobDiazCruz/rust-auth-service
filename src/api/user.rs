use actix_web::{ web, Result, Error, HttpResponse, HttpRequest };
use mongodb::results::InsertOneResult;
use crate::{
    database::mongo::Mongo,
    models::user_model::{ User, Email },
    helpers::errors::ServiceError,
    helpers::form_data::{ FormData, LoginForm },
    helpers::obj_id_converter::Converter,
    helpers::jwt::sign_jwt,
};
use std::env;
use serde_json::json;
use google_oauth::{ AsyncClient, GooglePayload };

pub async fn create_user(
    db: web::Data<Mongo>,
    form: web::Json<FormData>
) -> Result<web::Json<InsertOneResult>, Error> {
    let email = Email::parse(String::from(&form.email))?;
    let name = form.name.clone();
    let user = User::new(name, email);

    match db.create_user(user) {
        Ok(insert_result) => Ok(web::Json(insert_result)),
        Err(_) => Err(ServiceError::BadRequest(String::from("oh no")).into()),
    }
}

pub async fn get_user_by_id(
    db: web::Data<Mongo>,
    user_id: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
    let obj_id = Converter::string_to_bson(user_id.to_string())?;
    let user = db.get_user_by_id(obj_id);
    match user {
        Ok(Some(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(None) => Err(ServiceError::BadRequest("User does not exist.".to_string())),
        Err(_) => Err(ServiceError::BadRequest("Error fetching user.".to_string())),
    }
}

pub async fn login_google_user(
    db: web::Data<Mongo>,
    form: web::Json<LoginForm>
) -> Result<HttpResponse, ServiceError> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let id_token = form.id_token.clone();
    let payload = check_payload(id_token).await?;

    if payload.at_hash.is_none() || payload.azp.is_none() || payload.email.is_none() {
        return Err(ServiceError::BadRequest(String::from("Invalid ID token")).into());
    }

    let user = db.get_user_by_email(form.email.clone());

    match user {
        Ok(Some(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(None) => {
            let user = User::new(name, email);
            match db.create_user(user) {
                Ok(insert_result) => {
                    let obj_id = Converter::string_to_bson(insert_result.inserted_id.to_string())?;
                    let user_details = db.get_user_by_id(obj_id);
                    let access_token = sign_jwt()?;
                    let response =
                        json!({
                            "access_token": access_token
                        });
                    match user_details {
                        Ok(Some(_)) => Ok(HttpResponse::Ok().json(response)),
                        Ok(None) =>
                            Err(ServiceError::BadRequest("User does not exist.".to_string())),
                        Err(_) => Err(ServiceError::BadRequest("Error fetching user.".to_string())),
                    }
                }
                Err(_) => Err(ServiceError::BadRequest(String::from("oh no")).into()),
            }
        }
        Err(_) => Err(ServiceError::BadRequest("Error fetching user.".to_string())),
    }
}

async fn check_payload(id_token: String) -> Result<GooglePayload, ServiceError> {
    let client_id: String = env
        ::var("GOOGLE_CLIENT_ID")
        .expect("GOOGLE_CLIENT_ID environment variable not set");
    let client = AsyncClient::new(client_id);
    let payload_result = client.validate_id_token(id_token).await;
    let payload = match payload_result {
        Ok(payload) => payload,
        Err(_) => {
            return Err(ServiceError::BadRequest(String::from("Invalid ID token")).into());
        }
    };
    return Ok(payload);
}

pub async fn logout_user(
    db: web::Data<Mongo>,
    req: HttpRequest
) -> Result<HttpResponse, ServiceError> {
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_none() {
        return Err(ServiceError::BadRequest(String::from("No auth header.")));
    }
    let auth_str = auth_header
        .unwrap()
        .to_str()
        .map_err(|_| ServiceError::BadRequest(String::from("Invalid auth header.")))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(ServiceError::BadRequest("Invalid auth header format.".to_string()));
    }

    let parts: Vec<&str> = auth_str.split_whitespace().collect();
    if let Some(token) = parts.get(1) {
        println!("Access Token: {}", token);
        let res = db.store_invalidated_token(token);
        let response = json!({
                "message": "User logout successfully!"
            });
        match res {
            Ok(_) => Ok(HttpResponse::Ok().json(response)),
            Err(_) => Err(ServiceError::BadRequest("Error storing invalidated token.".to_string())),
        }
    } else {
        Err(ServiceError::BadRequest("Invalid auth header format.".to_string()))
    }
}
