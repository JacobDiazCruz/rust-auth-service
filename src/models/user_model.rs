use mongodb::bson::oid::ObjectId;
use serde::{ Serialize, Deserialize };
use crate::{ helpers::errors::ServiceError };
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: Email,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Email, ServiceError> {
        if email.is_empty() {
            Err(ServiceError::BadRequest("Email is required.".to_string()))
        } else {
            Ok(Email(email))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_email(&self) -> &String {
        &self.0
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
