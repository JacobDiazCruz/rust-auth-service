use actix_web::http::header::HeaderValue;
use hmac::{ Hmac, Mac };
use jwt::{ AlgorithmType, Header, SignWithKey, Token };
use sha2::Sha384;
use std::collections::BTreeMap;

use super::errors::ServiceError;

pub fn sign_jwt() -> Result<String, ServiceError> {
    let key: Hmac<Sha384> = Hmac::new_from_slice(b"some-secret").map_err(|e|
        ServiceError::BadRequest(format!("Key initialization error: {:?}", e))
    )?;

    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };

    // Create a BTreeMap for JWT claims
    let mut claims = BTreeMap::new();
    claims.insert("sub", "someone");

    // Sign the JWT token
    let token = Token::new(header, claims);
    let token_str = token
        .sign_with_key(&key)
        .map_err(|e| ServiceError::BadRequest(format!("Token signing error: {:?}", e)))?;

    Ok(token_str.into())
}

pub fn get_token(auth_header: Option<&HeaderValue>) -> Result<&str, ServiceError> {
    if auth_header.is_none() {
        return Err(ServiceError::BadRequest(String::from("No auth header.")));
    }
    let auth_str = auth_header
        .unwrap()
        .to_str()
        .map_err(|_| ServiceError::BadRequest(String::from("Invalid auth header.")))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(ServiceError::BadRequest("Invalid auth header format.".to_string()));
    }

    let parts: Vec<&str> = auth_str.split_whitespace().collect();

    if let Some(token) = parts.get(1) {
        return Ok(token);
    } else {
        Err(ServiceError::BadRequest("Invalid auth header format.".to_string()))
    }
}
