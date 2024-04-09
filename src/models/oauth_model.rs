use serde::{ Serialize, Deserialize };
use serde_json::Value;
use crate::{ helpers::errors::ServiceError };
use std::error::Error;
use jsonwebtoken::{ DecodingKey };
use reqwest;

use crate::config::config::Config;

#[derive(Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub id_token: String,
}

pub struct AppState {
    pub env: Config,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTokenClaims {
    iss: String, // Issuer
    aud: String, // Audience
    exp: usize, // Expiry time
    iat: usize, // Issued at time
    sub: String, // Subject (user ID)
    email: String, // User's email
}

#[derive(Deserialize)]
pub struct GoogleUserResult {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OauthType {
    Google,
    Facebook,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth(OauthType);

impl Oauth {
    pub fn validate(oauth_type: OauthType) -> Result<String, ServiceError> {
        let oauth = Oauth(oauth_type);
        match oauth.0 {
            OauthType::Google => Ok(String::from("Google")),
            OauthType::Facebook => Ok(String::from("Facebook")),
            _ => Err(ServiceError::BadRequest(String::from("Invalid OAuth type"))),
        }
    }
}
