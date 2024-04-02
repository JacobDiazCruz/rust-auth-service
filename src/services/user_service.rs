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
