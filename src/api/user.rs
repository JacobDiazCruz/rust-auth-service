use std::sync::Arc;
use axum::{ extract::Json, http::{ StatusCode, HeaderMap }, response::IntoResponse };
use serde_json::{ json, Value };

use crate::{
    services::user::{
        login_google_user_service,
        register_user_service,
        logout_user_service,
        manual_login_user_service,
        account_verification_service,
    },
    helpers::{
        form_data::{ LoginForm, ManualLoginForm, VerificationCodeForm, RegisterForm, LogoutForm },
        jwt::{ sign_jwt, get_token, validate_jwt },
        response::LoginResponse,
    },
};
use axum::{ extract::State };
use crate::AppState;

pub async fn register_user_api(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<RegisterForm>
) -> impl IntoResponse {
    let response = register_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn account_verification_api(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<VerificationCodeForm>
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let response = account_verification_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn manual_login_user_api(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<ManualLoginForm>
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let response = manual_login_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn login_google_user_api(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<LoginForm>
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let response = login_google_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

// flow:
// this will delete the refresh token data in db
// upon logout, this will not revoke the access token, the access token will wait for its expiry.
pub async fn logout_user_api(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<LogoutForm>
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let response = logout_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn refresh_token_api(
    headers: HeaderMap
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let auth_header = headers.get("Authorization").unwrap();
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
            Ok((StatusCode::OK, Json(data)))
        }
        Err(err) => Err(err),
    }
}
