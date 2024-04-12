use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
    pub id_token: String,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualLoginForm {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterForm {
    pub name: String,
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationCodeForm {
    pub email: String,
    pub code: String,
}
