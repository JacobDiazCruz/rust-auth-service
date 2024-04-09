use actix_web::{ web, Result, HttpResponse, HttpRequest };
use crate::{
    database::mongo::Mongo,
    services::user::{ login_google_user_service, logout_user_service, register_user_service },
    helpers::errors::ServiceError,
    helpers::{
        form_data::{ LoginForm, ManualLoginForm },
        jwt::{ sign_jwt, get_token, validate_jwt },
    },
};
use serde_json::json;

pub async fn register_user_api(
    db: web::Data<Mongo>,
    form: web::Json<ManualLoginForm>
) -> Result<HttpResponse, ServiceError> {
    let response = register_user_service(db, form).await;
    match response {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(err),
    }
}

pub async fn login_google_user_api(
    db: web::Data<Mongo>,
    form: web::Json<LoginForm>
) -> Result<HttpResponse, ServiceError> {
    let response = login_google_user_service(db, form).await;
    match response {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(err),
    }
}

pub async fn logout_user_api(
    db: web::Data<Mongo>,
    req: HttpRequest
) -> Result<HttpResponse, ServiceError> {
    let auth_header = req.headers().get("Authorization");
    let response = logout_user_service(db, auth_header).await;
    match response {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(err),
    }
}

pub async fn refresh_token_api(req: HttpRequest) -> Result<HttpResponse, ServiceError> {
    let auth_header = req.headers().get("Authorization");
    let refresh_token = get_token(auth_header);
    let user_id = validate_jwt(&refresh_token.unwrap());
    match user_id {
        Ok(data) => {
            let new_refresh_token = sign_jwt(&data)?;
            let new_access_token = sign_jwt(&data)?;
            let data =
                json!({
                    "new_refresh_token": new_refresh_token,
                    "new_access_token": new_access_token
                });
            Ok(HttpResponse::Ok().json(data))
        }
        Err(err) => Err(err),
    }
}
