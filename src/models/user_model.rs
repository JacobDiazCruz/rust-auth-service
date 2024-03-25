use actix_web::error::ResponseError;
use mongodb::bson::oid::ObjectId;
use serde::{ Serialize, Deserialize };
use std::fmt;
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: Email,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email(String);
#[derive(Debug)]
pub struct AuthError(String);

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for AuthError {}

impl Email {
    pub fn parse(email: &str) -> Result<Email, AuthError> {
        if email.is_empty() {
            Err(AuthError("Email is required.".to_string()))
        } else {
            Ok(Email(email.to_string()))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl User {
    pub fn new(name: String, email: Email) -> Self {
        Self {
            id: None,
            name,
            email,
        }
    }
}
