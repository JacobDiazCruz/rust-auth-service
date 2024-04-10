use mongodb::bson::oid::ObjectId;
use serde::{ Serialize, Deserialize };
use crate::helpers::errors::ServiceError;
use bcrypt::{ hash_with_result, BcryptError };
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: Email,
    pub password: Option<Password>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password(String);

impl Email {
    pub fn parse(email: String) -> Result<Email, ServiceError> {
        if email.is_empty() {
            return Err(ServiceError::BadRequest("Email is required.".to_string()));
        }
        let email_regex = Regex::new(r"^[\w-]+(\.[\w-]+)*@([\w-]+\.)+[a-zA-Z]{2,7}$").unwrap();
        if !email_regex.is_match(&email) {
            return Err(ServiceError::BadRequest("Invalid email format.".to_string()));
        }
        Ok(Email(email))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_email(&self) -> &String {
        &self.0
    }
}

impl Password {
    pub fn parse(password: String) -> Result<Password, ServiceError> {
        if password.is_empty() {
            Err(ServiceError::BadRequest("Password is required.".to_string()))
        } else {
            Ok(Password(password))
        }
    }

    pub fn hash(&self) -> Result<Password, BcryptError> {
        let hashed_password = hash_with_result(&self.0, 12)?;
        Ok(Password(hashed_password.to_string()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_password(&self) -> &String {
        &self.0
    }
}

impl User {
    pub fn new(name: String, email: Email) -> Self {
        Self {
            id: None,
            name,
            email,
            password: None,
            is_verified: Some(false),
        }
    }
}
