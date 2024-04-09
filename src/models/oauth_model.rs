use serde::{ Serialize, Deserialize };
use serde_json::Value;
use crate::{ helpers::errors::ServiceError };
use std::error::Error;
use jsonwebtoken::{ decode, Algorithm, Validation, DecodingKey };
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
    // Add other relevant fields as needed
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

    fn fetch_google_decoding_key() -> Result<DecodingKey, Box<dyn Error>> {
        // Make an HTTPS GET request to fetch Google's public keys
        let public_keys_response = reqwest::blocking::get(
            "https://www.googleapis.com/oauth2/v3/certs"
        )?;
        let public_keys_json: Value = public_keys_response.json()?;
        let public_key = public_keys_json["keys"][0]["x5c"][0]
            .as_str()
            .ok_or("Public key not found")?;

        // Create a DecodingKey from the parsed public key
        let decoding_key = DecodingKey::from_rsa_pem(public_key.as_bytes())?;
        Ok(decoding_key)
    }

    // Function to verify and decode Google access tokens
    fn verify_google_access_token(access_token: &str) -> Result<GoogleTokenClaims, Box<dyn Error>> {
        // Fetch Google's public keys for signature verification
        let public_keys_response = reqwest::blocking::get(
            "https://www.googleapis.com/oauth2/v3/certs"
        )?;
        let public_keys_json: serde_json::Value = public_keys_response.json()?;
        let public_keys = public_keys_json["keys"].as_array().ok_or("Public keys not found")?;
        let decoding_key = Oauth::fetch_google_decoding_key()?;

        // Find the appropriate public key based on the token's key ID (kid)
        let token_header = decode::<serde_json::Value>(
            &access_token,
            &decoding_key,
            &Validation::new(Algorithm::RS256)
        )?.header;
        let key_id = token_header
            .get("kid")
            .and_then(|kid| kid.as_str())
            .ok_or("Key ID not found in token header")?;

        let public_key_json = public_keys
            .iter()
            .find(|key| key["kid"].as_str() == Some(key_id))
            .ok_or("Public key not found for token")?;

        let public_key = base64::decode_config(
            public_key_json["x5c"][0].as_str().unwrap(),
            base64::STANDARD
        )?;

        // Verify the token's signature using the fetched public key
        let token_data = decode::<GoogleTokenClaims>(
            &access_token,
            &public_key,
            &Validation::new(Algorithm::RS256)
        )?;

        // Perform additional validation checks as needed
        // For example, check token expiry, issuer, audience, scopes, etc.

        Ok(token_data.claims)
    }
}
