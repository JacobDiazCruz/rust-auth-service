use mongodb::bson::oid::ObjectId;
use serde::{ Serialize, Deserialize };
use bcrypt::{ hash_with_result, BcryptError };
use regex::Regex;
use axum::{ http::StatusCode };
use crate::services::user::json_response;
use axum::{ extract::Json };

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

impl Email {
    pub fn parse(email: String) -> Result<Email, (StatusCode, Json<serde_json::Value>)> {
        if email.is_empty() {
            return Err((StatusCode::BAD_REQUEST, Json(json_response("Email is required."))));
        }
        let email_regex = Regex::new(r"^[\w-]+(\.[\w-]+)*@([\w-]+\.)+[a-zA-Z]{2,7}$").unwrap();
        if !email_regex.is_match(&email) {
            return Err((StatusCode::BAD_REQUEST, Json(json_response("Invalid email format."))));
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
    pub fn parse(password: String) -> Result<Password, (StatusCode, Json<serde_json::Value>)> {
        if password.is_empty() {
            return Err((StatusCode::BAD_REQUEST, Json(json_response("Password is required."))));
        }
        if password.len() < 6 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json_response("Password must be at least 6 characters long.")),
            ));
        }
        if !password.chars().any(|c| c.is_digit(10)) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json_response("Password must contain at least one number.")),
            ));
        }
        if !password.chars().any(|c| c.is_ascii_punctuation()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json_response("Password must contain at least one special character.")),
            ));
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

    pub fn get_password(&self) -> &String {
        &self.0
    }
}

impl User {
    pub fn new(
        name: String,
        email: Email,
        is_verified: Option<bool>,
        login_type: LoginTypes
    ) -> Self {
        Self {
            id: None,
            name,
            email,
            password: None,
            is_verified,
            login_type,
        }
    }
}
