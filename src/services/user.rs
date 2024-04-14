use std::sync::Arc;

use axum::response::Result;
use axum::extract::Json;
use axum::{ http::StatusCode, response::IntoResponse, extract::State };

use crate::AppState;
use crate::helpers::form_data::LogoutForm;
use crate::models::refresh_token_model::RefreshToken;
use crate::{
    models::user_model::{ User, Email, Password, LoginTypes, UserVerificationCode },
    helpers::form_data::LoginForm,
    helpers::{ obj_id_converter::Converter, form_data::{ VerificationCodeForm, RegisterForm } },
    helpers::{ jwt::sign_jwt, form_data::ManualLoginForm },
};

use serde_json::{ json, Value };
use bcrypt;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{ Message, SmtpTransport, Transport };
use rand::distributions::Alphanumeric;
use rand::{ thread_rng, Rng };
use crate::config::config::Config;

pub fn json_response(message: &str) -> Value {
    let error_obj = json!({
        "message": message,
        "data": {}
    });
    error_obj
}

pub async fn register_user_service(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<RegisterForm>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();

    // check if email exists
    let email_exist = app_state.db.get_user_by_email(email_str);

    if let Some(_) = email_exist.unwrap() {
        Err((StatusCode::BAD_REQUEST, Json(json_response("Email already exist."))))
    } else {
        let password = Password::parse(String::from(&form.password))?;
        let hashed_password = Password::hash(&password);
        let cloned_email = email.clone();
        let new_user = User {
            id: None,
            name,
            email,
            login_type: LoginTypes::MANUAL,
            password: Some(hashed_password.unwrap()),
            is_verified: Some(false),
        };
        let _ = smtp_service(State(app_state.clone()), cloned_email);
        match app_state.db.create_user(&new_user) {
            Ok(_) => Ok((StatusCode::CREATED, Json(new_user))),
            Err(_) => {
                Err((StatusCode::BAD_REQUEST, Json(json_response("Failed creating user"))))
            }
        }
    }
}

pub fn smtp_service(
    State(app_state): State<Arc<AppState>>,
    receiver: Email
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let conf = Config::init();
    let code: String = thread_rng().sample_iter(&Alphanumeric).take(4).map(char::from).collect();

    // store the code to the "codes" collection with the "email" of its owner.
    let verif_code_payload = UserVerificationCode {
        code: code.clone(),
        email: receiver.clone(),
    };
    let verif_code_res = app_state.db.store_verification_code(verif_code_payload);

    match verif_code_res {
        Ok(_) => {
            let email = Message::builder()
                .from("NoBody <your@domain.tld>".parse().unwrap())
                .reply_to("Yuin <my@email.tld>".parse().unwrap())
                .to(receiver.get_email().parse().unwrap())
                .subject("Your code")
                .header(ContentType::TEXT_PLAIN)
                .body(format!("Your verification code is: {}", code))
                .unwrap();

            let creds = Credentials::new(
                conf.google_smtp_username.into(),
                conf.google_smtp_password.into()
            );

            // Open a remote connection to gmail
            let mailer = SmtpTransport::relay("smtp.gmail.com").unwrap().credentials(creds).build();

            // Send the email
            match mailer.send(&email) {
                Ok(_) => Ok("Email sent successfully!".to_string()),
                Err(_) =>
                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(json_response("Failed sending email. Please try again later.")),
                    )),
            }
        }
        Err(_) =>
            Err((
                StatusCode::BAD_REQUEST,
                Json(json_response("Failed storing verification code.")),
            )),
    }
}

// User is logged in but still need to submit the code to verify their account.
pub async fn account_verification_service(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<VerificationCodeForm>
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let email = Email::parse(form.email.clone()).unwrap();
    let payload = UserVerificationCode {
        email,
        code: form.code.clone(),
    };

    // Update is_verified data of the user if it matches
    let res: std::result::Result<
        Option<UserVerificationCode>,
        String
    > = app_state.db.get_verification_code(payload);

    match res {
        Ok(Some(res)) => {
            let email = res.email.get_email().to_string();
            let update_user_res = app_state.db.update_user_verification(&email);
            match update_user_res {
                Ok(_) => {
                    // remove the verification codes in the verif codes collection after
                    let _ = app_state.db.delete_verification_codes(&email);
                    Ok((StatusCode::OK, Json(json_response("Account verified!"))))
                }
                Err(_) =>
                    Err((StatusCode::BAD_REQUEST, Json(json_response("Error updating user")))),
            }
        }
        Ok(None) =>
            Err((StatusCode::BAD_REQUEST, Json(json_response("Wrong code. Please try again.")))),
        Err(_) =>
            Err((StatusCode::BAD_REQUEST, Json(json_response("Error geting verification code.")))),
    }
}

pub async fn get_user_by_id_service(
    State(app_state): State<Arc<AppState>>,
    user_id: String
) -> Result<Option<User>, (StatusCode, Json<serde_json::Value>)> {
    let obj_id = Converter::string_to_bson(user_id)?;
    match app_state.db.get_user_by_id(obj_id) {
        Ok(insert_result) => Ok(insert_result),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(json_response("Failed creating user.")))),
    }
}

fn login_response(
    State(app_state): State<Arc<AppState>>,
    data: User
) -> Result<Value, (StatusCode, Json<serde_json::Value>)> {
    let user_id_str = match data.id {
        Some(object_id) => object_id.to_hex(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json_response("User ID not found.")),
            ));
        }
    };

    let access_token = sign_jwt(&user_id_str, 5).unwrap();
    let refresh_token = sign_jwt(&user_id_str, 1440).unwrap();
    let refresh_token_data = RefreshToken {
        id: None,
        user_id: data.id.into(),
        email: data.email.clone(),
        refresh_token: refresh_token.clone(),
    };

    let _ = app_state.db.store_refresh_token(refresh_token_data);

    let response =
        json!({
            "message": "User logged in successfully!",
            "data": {
                "access_token": access_token,
                "refresh_token": refresh_token,
                "user": {
                    "_id": user_id_str,
                    "email": data.email.get_email()
                }
            }
    });
    return Ok(response);
}

pub async fn manual_login_user_service(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<ManualLoginForm>
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();
    let password = Password::parse(String::from(&form.password))?;

    let user = app_state.db.get_user_by_email(email_str);
    match user {
        Ok(data) => {
            let user_data = data.unwrap();
            let user_password = user_data.password
                .as_ref()
                .ok_or_else(|| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Can't get user password".to_string()),
                ));

            let is_pw_verified = bcrypt::verify(
                password.get_password(),
                &user_password.unwrap().get_password()
            );
            if !is_pw_verified.unwrap() {
                return Err((StatusCode::BAD_REQUEST, Json(json_response("Wrong password!"))));
            }

            // If user is not verified yet, send a code to their email.
            if !user_data.is_verified.unwrap_or_default() {
                let _ = smtp_service(State(app_state), email);
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(
                        json_response("Verify your account first. We've sent a code to your email.")
                    ),
                ));
            }
            let response = login_response(State(app_state), user_data).unwrap();
            Ok((StatusCode::OK, Json(response)))
        }
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(json_response("Handle this mongo error.")))),
    }
}

pub async fn login_google_user_service(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<LoginForm>
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email)).unwrap();
    let email_str = email.get_email().clone();

    // Note: Add verify id_token here in the future

    let user = app_state.db.get_user_by_email(email_str);

    if let Some(data) = user.unwrap() {
        let response = login_response(State(app_state.clone()), data).unwrap();
        return Ok((StatusCode::OK, Json(response)));
    }

    let new_user_payload = User::new(name, email, Some(true), LoginTypes::GOOGLE);
    let new_user = app_state.db.create_user(&new_user_payload);
    let new_user_details = get_user_by_id_service(
        State(app_state.clone()),
        new_user.unwrap().inserted_id.to_string()
    ).await.unwrap();

    if let Some(data) = new_user_details {
        let response = login_response(State(app_state.clone()), data).unwrap();
        Ok((StatusCode::OK, Json(response)))
    } else {
        Err((StatusCode::BAD_REQUEST, Json(json_response("User does not exist."))))
    }
}

pub async fn logout_user_service(
    State(app_state): State<Arc<AppState>>,
    Json(form): Json<LogoutForm>
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let res = app_state.db.delete_refresh_token(form.refresh_token);
    match res {
        Ok(_) => Ok((StatusCode::OK, Json(json_response("User logged out successfully!")))),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(json_response("Error Invalidating Token")))),
    }
}
