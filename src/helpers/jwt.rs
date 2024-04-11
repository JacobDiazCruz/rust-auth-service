use actix_web::http::header::HeaderValue;
use std::time::{ SystemTime };
use jsonwebtoken::{ encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey };
use super::errors::ServiceError;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
    issued_at: u64,
    exp: u64,
}

pub fn sign_jwt(user_id: &str) -> Result<String, ServiceError> {
    let header = Header::new(Algorithm::HS512);

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    // Token expires in 1 hour
    let expiration_time = current_time + 3600;

    let my_claims = Claims {
        user_id: String::from(user_id),
        issued_at: current_time,
        exp: expiration_time,
    };

    let token = encode(&header, &my_claims, &EncodingKey::from_secret("secret".as_ref()));
    Ok(token.unwrap())
}

pub fn get_token(auth_header: Option<&HeaderValue>) -> Result<String, ServiceError> {
    if auth_header.is_none() {
        return Err(ServiceError::BadRequest(String::from("No auth header.")));
    }
    let auth_str = auth_header
        .unwrap()
        .to_str()
        .map_err(|_| ServiceError::BadRequest(String::from("Invalid auth header.")))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(ServiceError::BadRequest(String::from("Invalid auth header format.")));
    }

    let parts: Vec<&str> = auth_str.split_whitespace().collect();

    if let Some(token) = parts.get(1) {
        return Ok(String::from(token.to_owned()));
    } else {
        Err(ServiceError::BadRequest(String::from("Error in getting parts of the token.")))
    }
}

pub fn validate_jwt(access_token: &str) -> Result<String, ServiceError> {
    let decoding_key = DecodingKey::from_secret("secret".as_ref());
    let mut validation = Validation::new(Algorithm::HS512);

    let token_data = match decode::<Claims>(&access_token, &decoding_key, &validation) {
        Ok(token_data) => token_data,
        Err(_) => {
            return Err(ServiceError::Unauthorized("Invalid access token.".to_string()));
        }
    };

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    if token_data.claims.exp < current_time {
        return Err(ServiceError::Unauthorized("Expired access token.".to_string()));
    }

    Ok(String::from(token_data.claims.user_id))
}
