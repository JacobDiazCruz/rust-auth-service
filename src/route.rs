use std::sync::Arc;

use axum::{ routing::{ get, post }, Router };

use crate::api::user::{
    login_google_user_api,
    register_user_api,
    logout_user_api,
    manual_login_user_api,
    refresh_token_api,
    account_verification_api,
};
use crate::AppState;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/register", post(register_user_api))
        .route("/login", post(manual_login_user_api))
        .route("/account/verify", post(account_verification_api))
        .route("/login/google", post(login_google_user_api))
        .route("/logout", post(logout_user_api))
        .route("/refresh-token", post(refresh_token_api))
        .with_state(app_state)
}
