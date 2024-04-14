use std::sync::Arc;
use axum::{ extract::Json, http::{ StatusCode, HeaderMap }, response::IntoResponse };
use axum::extract::State;
use serde_json::json;

use crate::{
    services::user::{
        login_google_user_service,
        register_user_service,
        logout_user_service,
        manual_login_user_service,
        account_verification_service,
    },
    utils::{
        form_data::{ LoginForm, ManualLoginForm, VerificationCodeForm, RegisterForm, LogoutForm },
        jwt::{ sign_jwt, get_token, validate_jwt },
    },
};
use crate::AppState;

pub async fn register_user_handler(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<RegisterForm>
) -> impl IntoResponse {
    let response = register_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn account_verification_handler(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<VerificationCodeForm>
) -> impl IntoResponse {
    let response = account_verification_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn manual_login_user_handler(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<ManualLoginForm>
) -> impl IntoResponse {
    let response = manual_login_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn login_google_user_handler(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<LoginForm>
) -> impl IntoResponse {
    let response = login_google_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

// flow:
// this will delete the refresh token data in db
// upon logout, this will not revoke the access token, the access token will wait for its expiry.
pub async fn logout_user_handler(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<LogoutForm>
) -> impl IntoResponse {
    let response = logout_user_service(State(app_state), Json(form)).await;
    match response {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

pub async fn refresh_token_handler(headers: HeaderMap) -> impl IntoResponse {
    let auth_header = headers.get("Authorization").unwrap();
    let refresh_token = get_token(auth_header);
    let user_id = validate_jwt(&refresh_token.unwrap());
    match user_id {
        Ok(data) => {
            let new_refresh_token = sign_jwt(&data, 1440)?;
            let new_access_token = sign_jwt(&data, 5)?;
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
