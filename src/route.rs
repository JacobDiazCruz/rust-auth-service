use std::sync::Arc;

use axum::{ routing::{ get, post }, Router };

use crate::api::user::{
    // login_google_user_api,
    // logout_user_api,
    // refresh_token_api,
    register_user_api,
    // manual_login_user_api,
    // account_verification_api,
};
use crate::AppState;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/register", post(register_user_api))
        // .route("/api/notes/", post(create_note_handler))
        // .route("/api/notes", get(note_list_handler))
        // .route(
        //     "/api/notes/:id",
        //     get(get_note_handler).patch(edit_note_handler).delete(delete_note_handler)
        // )
        .with_state(app_state)
}
