use mongodb::bson::oid::ObjectId;
use serde::{ Serialize, Deserialize };
use bcrypt::{ hash_with_result, BcryptError };
use regex::Regex;
use axum::{ http::StatusCode };
use crate::services::user::error_response;
use axum::{ extract::Json };

#[allow(non_snake_case)]

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: Email,
    pub password: Option<Password>,
    pub is_verified: Option<bool>,
    pub login_type: LoginTypes,
}

#[derive(Debug)]
pub struct UserBuilder {
    id: Option<ObjectId>,
    name: String,
    email: Email,
    password: Option<Password>,
    is_verified: Option<bool>,
    login_type: LoginTypes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserVerificationCode {
    pub email: Email,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginTypes {
    GOOGLE,
    FACEBOOK,
    MANUAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password(String);

impl UserBuilder {
    pub fn new(name: String, email: Email, login_type: LoginTypes) -> Self {
        Self {
            id: None,
            name,
            email,
            password: None,
            is_verified: None,
            login_type,
        }
    }

    pub fn id(mut self, id: ObjectId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn password(mut self, password: Password) -> Self {
        self.password = Some(password);
        self
    }

    pub fn is_verified(mut self, is_verified: bool) -> Self {
        self.is_verified = Some(is_verified);
        self
    }

    pub fn build(self) -> User {
        User {
            id: self.id,
            name: self.name,
            email: self.email,
            password: self.password,
            is_verified: self.is_verified,
            login_type: self.login_type,
        }
    }
}

impl Email {
    pub fn parse(email: String) -> Result<Email, (StatusCode, Json<serde_json::Value>)> {
        if email.is_empty() {
            return Err(error_response("Email is required.", StatusCode::BAD_REQUEST));
        }
        let email_regex = Regex::new(r"^[\w-]+(\.[\w-]+)*@([\w-]+\.)+[a-zA-Z]{2,7}$").unwrap();
        if !email_regex.is_match(&email) {
            return Err(error_response("Invalid email format.", StatusCode::BAD_REQUEST));
        }
        Ok(Email(email))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_str(&self) -> &String {
        &self.0
    }
}

impl Password {
    pub fn parse(password: String) -> Result<Password, (StatusCode, Json<serde_json::Value>)> {
        if password.is_empty() {
            return Err(error_response("Password is required.", StatusCode::BAD_REQUEST));
        }
        if password.len() < 6 {
            return Err(
                error_response(
                    "Password must be at least 6 characters long.",
                    StatusCode::BAD_REQUEST
                )
            );
        }
        if !password.chars().any(|c| c.is_digit(10)) {
            return Err(
                error_response(
                    "Password must contain at least one number.",
                    StatusCode::BAD_REQUEST
                )
            );
        }
        if !password.chars().any(|c| c.is_ascii_punctuation()) {
            return Err(
                error_response(
                    "Password must contain at least one special character.",
                    StatusCode::BAD_REQUEST
                )
            );
        }
        Ok(Password(password))
    }

    pub fn hash(&self) -> Result<Password, BcryptError> {
        let hashed_password = hash_with_result(&self.0, 12)?;
        Ok(Password(hashed_password.to_string()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_str(&self) -> &String {
        &self.0
    }
}
