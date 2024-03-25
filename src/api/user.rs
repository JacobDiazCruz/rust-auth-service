use actix_web::{ web, Result, Error, HttpResponse };
use mongodb::results::InsertOneResult;
use crate::{
    database::mongo::Mongo,
    models::user_model::{ User, Email },
    helpers::errors::ServiceError,
    helpers::form_data::{ FormData, LoginForm },
};
use mongodb::bson::oid::ObjectId;
use std::env;
use google_oauth::AsyncClient;

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
    // Convert the user_id string to a MongoDB ObjectId
    let obj_id = match ObjectId::parse_str(&user_id.to_string()) {
        Ok(obj_id) => obj_id,
        Err(_) => {
            return Err(ServiceError::BadRequest("Invalid ID format".to_string()));
        }
    };

    let user = db.get_user_by_id(obj_id);
    match user {
        Ok(Some(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(None) => Err(ServiceError::BadRequest("User does not exist.".to_string())),
        Err(_) => Err(ServiceError::BadRequest("Error fetching user.".to_string())),
    }
}

// pub async fn login_google_user(
//     db: web::Data<Mongo>,
//     form: web::Json<LoginForm>
// ) -> Result<HttpResponse, Error> {
//     let client_id: String = env
//         ::var("GOOGLE_CLIENT_ID")
//         .expect("GOOGLE_CLIENT_ID environment variable not set");
//     let name = form.name.clone();
//     let email = Email::parse(&form.email.clone())?;
//     let id_token = form.id_token.clone();
//     let client = AsyncClient::new(client_id);
//     let payload_result = client.validate_id_token(id_token).await;

//     let payload = match payload_result {
//         Ok(payload) => payload,
//         Err(_) => {
//             return Err(ServiceError::BadRequest(String::from("Invalid ID token")).into());
//         }
//     };

//     if payload.at_hash.is_none() || payload.azp.is_none() || payload.email.is_none() {
//         return Err(ServiceError::BadRequest(String::from("Invalid ID token")).into());
//     }

//     let user = db.get_user_by_email(form.email.clone());

//     match user {
//         Ok(Some(data)) => Ok(HttpResponse::Ok().json(data)),
//         Ok(None) => {
//             let user = User::new(name, email);
//             match db.create_user(user) {
//                 Ok(insert_result) => {
//                     let user = db.get_user_by_id(insert_result.inserted_id);
//                     Ok(HttpResponse::Ok().json(user)
//                 },
//                 Err(_) => Err(ServiceError::BadRequest(String::from("oh no")).into()),
//             }
//         }
//         Err(_) => Err(ServiceError::BadRequest("Error fetching user.".to_string())),
//     }
// }
