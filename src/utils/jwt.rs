use axum::http::HeaderValue;
use std::time::SystemTime;
use jsonwebtoken::{ encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey };
use crate::services::user::error_response;
use chrono::{ Utc, Duration };

use serde::{ Serialize, Deserialize };
use axum::{ extract::Json, http::StatusCode };

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
    issued_at: u64,
    exp: u64,
}

pub fn sign_jwt(
    user_id: &str,
    exp_time_mins: i64
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let header = Header::new(Algorithm::HS512);

    let current_time = Utc::now().timestamp() as u64;

    // Token expires in 1 hour
    let expiration_time = (Utc::now() + Duration::minutes(exp_time_mins)).timestamp() as u64;

    let my_claims = Claims {
        user_id: String::from(user_id),
        issued_at: current_time,
        exp: expiration_time,
    };

    let token = encode(&header, &my_claims, &EncodingKey::from_secret("secret".as_ref()));
    Ok(token.unwrap())
}

pub fn get_token(
    auth_header: &HeaderValue
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    if auth_header.is_empty() {
        return Err(error_response("No auth header.", StatusCode::BAD_REQUEST));
    }

    let auth_str = auth_header
        .to_str()
        .map_err(|_| error_response("Invalid auth header format.", StatusCode::BAD_REQUEST))
        .unwrap();

    if !auth_str.starts_with("Bearer ") {
        return Err(error_response("Invalid auth header format.", StatusCode::BAD_REQUEST));
    }

    let parts: Vec<&str> = auth_str.split_whitespace().collect();

    if let Some(token) = parts.get(1) {
        Ok(String::from(token.to_owned()))
    } else {
        return Err(
            error_response(
                "Error in getting parts of the token.",
                StatusCode::INTERNAL_SERVER_ERROR
            )
        );
    }
}

pub fn validate_jwt(access_token: &str) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let decoding_key = DecodingKey::from_secret("secret".as_ref());
    let mut validation = Validation::new(Algorithm::HS512);

    let token_data = match decode::<Claims>(&access_token, &decoding_key, &validation) {
        Ok(token_data) => token_data,
        Err(err) => {
            return Err(error_response("Invalid access token.", StatusCode::UNAUTHORIZED));
        }
    };

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    if token_data.claims.exp < current_time {
        return Err(error_response("Expired access token.", StatusCode::UNAUTHORIZED));
    }

    Ok(String::from(token_data.claims.user_id))
}
