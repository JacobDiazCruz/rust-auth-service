use actix_web::{ web, Result, Error, HttpResponse };
use mongodb::results::InsertOneResult;
use crate::{
    database::mongo::Mongo,
    models::user_model::{ User, Email },
    helpers::errors::ServiceError,
    helpers::form_data::{ FormData, LoginForm },
    helpers::obj_id_converter::{ Converter },
};
use std::env;
use google_oauth::{ AsyncClient, GooglePayload };

pub async fn create_user(
    db: web::Data<Mongo>,
    form: web::Json<FormData>
) -> Result<web::Json<InsertOneResult>, Error> {
    let email = Email::parse(&form.email.clone())?;
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
    let email = Email::parse(&form.email.clone())?;
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
                    match user_details {
                        Ok(Some(user_details_data)) => Ok(HttpResponse::Ok().json(user_details)),
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
