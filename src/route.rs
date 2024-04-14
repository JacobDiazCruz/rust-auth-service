use std::sync::Arc;

use axum::{ routing::post, Router };

use crate::handlers::user::{
    login_google_user_handler,
    register_user_handler,
    logout_user_handler,
    manual_login_user_handler,
    refresh_token_handler,
    account_verification_handler,
};
use crate::AppState;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/register", post(register_user_handler))
        .route("/login", post(manual_login_user_handler))
        .route("/account/verify", post(account_verification_handler))
        .route("/login/google", post(login_google_user_handler))
        .route("/logout", post(logout_user_handler))
        .route("/refresh-token", post(refresh_token_handler))
        .with_state(app_state)
}
