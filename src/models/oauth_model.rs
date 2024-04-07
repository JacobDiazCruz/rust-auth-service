use serde::{ Serialize, Deserialize };
use crate::{ helpers::errors::{ ServiceError, ErrorMessages } };
use std::env;
use google_oauth::{ AsyncClient, GooglePayload };

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

    pub async fn validate_google_token(id_token: String) -> Result<GooglePayload, ServiceError> {
        let client_id: String = env
            ::var("GOOGLE_CLIENT_ID")
            .expect("GOOGLE_CLIENT_ID environment variable not set");
        let client = AsyncClient::new(client_id);
        let payload_result = client.validate_id_token(id_token).await;
        let payload = match payload_result {
            Ok(payload) => payload,
            Err(_) => {
                return Err(ServiceError::BadRequest(ErrorMessages::InvalidToken.error_msg()));
            }
        };
        return Ok(payload);
    }
}
