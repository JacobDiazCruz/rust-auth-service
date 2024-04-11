use actix_web::{ web, Result, http::header::HeaderValue };
use crate::{
    database::mongo::Mongo,
    models::user_model::{ User, Email, Password, LoginTypes },
    helpers::errors::{ ServiceError::{ BadRequest, InternalServerError }, ErrorMessages },
    helpers::form_data::LoginForm,
    helpers::obj_id_converter::Converter,
    helpers::{ jwt::{ sign_jwt, get_token }, form_data::ManualLoginForm },
};
use crate::helpers::errors::ServiceError;
use serde_json::{ json, Value };
use serde::{ Serialize, Deserialize };
use bcrypt;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{ Message, SmtpTransport, Transport };

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
    user: User,
}

pub async fn register_user_service(
    db: web::Data<Mongo>,
    form: web::Json<ManualLoginForm>
) -> Result<String, ServiceError> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();

    // check if email exists
    let email_exist = db.get_user_by_email(email_str);

    if let Some(_) = email_exist.unwrap() {
        return Err(BadRequest(ErrorMessages::EmailAlreadyExist.error_msg()));
    } else {
        let password = Password::parse(String::from(&form.password))?;
        let hashed_password = Password::hash(&password);
        let new_user = User {
            id: None,
            name,
            email,
            login_type: LoginTypes::Manual,
            password: Some(hashed_password.unwrap()),
            is_verified: Some(false),
        };
        match db.create_user(new_user) {
            Ok(_) => Ok("User created successfully!".to_string()),
            Err(_) => Err(BadRequest(ErrorMessages::CreateUserError.error_msg())),
        }
    }
}

pub async fn send_code_service(receiver: Email) {
    let email = Message::builder()
        .from("NoBody <your@domain.tld>".parse().unwrap())
        .reply_to("Yuin <my@email.tld>".parse().unwrap())
        .to(receiver.get_email().parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    let creds = Credentials::new("smtp_username".to_owned(), "smtp_password".to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com").unwrap().credentials(creds).build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}

pub async fn get_user_by_id_service(
    db: web::Data<Mongo>,
    user_id: String
) -> Result<Option<User>, ServiceError> {
    let obj_id = Converter::string_to_bson(user_id)?;
    match db.get_user_by_id(obj_id) {
        Ok(insert_result) => Ok(insert_result),
        Err(_) => Err(BadRequest(ErrorMessages::CreateUserError.error_msg())),
    }
}

fn login_response(data: User) -> Result<LoginResponse, ServiceError> {
    let user_id_str = match data.id {
        Some(object_id) => object_id.to_hex(),
        None => {
            return Err(ServiceError::InternalServerError("User ID not found.".to_string()));
        }
    };
    let access_token = sign_jwt(&user_id_str)?;
    let refresh_token = sign_jwt(&user_id_str)?;
    let response = LoginResponse {
        access_token,
        refresh_token,
        user: data,
    };
    return Ok(response);
}

pub async fn manual_login_user_service(
    db: web::Data<Mongo>,
    form: web::Json<ManualLoginForm>
) -> Result<User, ServiceError> {
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();
    let password = Password::parse(String::from(&form.password))?;

    let user = db.get_user_by_email(email_str);
    match user {
        Ok(data) => {
            let user_data = data.unwrap();
            let user_password = user_data.password
                .as_ref()
                .ok_or_else(|| InternalServerError("User password not found.".to_string()))?;

            let is_pw_verified = bcrypt::verify(
                password.get_password(),
                &user_password.get_password()
            );
            if !is_pw_verified.unwrap() {
                return Err(BadRequest("Wrong password. Please try again.".to_string()));
            }
            if !user_data.is_verified.unwrap() {
                return Err(BadRequest("Please verify your account first.".to_string()));
            }
            Ok(user_data)
        }
        Err(_) => Err(InternalServerError("User ID not found.".to_string())),
    }
}

pub async fn login_google_user_service(
    db: web::Data<Mongo>,
    form: web::Json<LoginForm>
) -> Result<LoginResponse, ServiceError> {
    let name = form.name.clone();
    let email = Email::parse(String::from(&form.email))?;
    let email_str = email.get_email().clone();

    // Note: Add verify id_token here in the future

    let user = db.get_user_by_email(email_str);

    if let Some(data) = user.unwrap() {
        let response = login_response(data);
        return Ok(response.unwrap());
    }

    let new_user_payload: User = User::new(name, email, Some(true), LoginTypes::Google);
    let new_user = db.create_user(new_user_payload);
    let new_user_details = get_user_by_id_service(
        db,
        new_user.unwrap().inserted_id.to_string()
    ).await?;

    if let Some(data) = new_user_details {
        let response = login_response(data);
        Ok(response.unwrap())
    } else {
        Err(BadRequest(ErrorMessages::UserNotExist.error_msg()))
    }
}

pub async fn logout_user_service(
    db: web::Data<Mongo>,
    auth_header: Option<&HeaderValue>
) -> Result<Value, ServiceError> {
    let token = get_token(auth_header);

    match token {
        Ok(val) => {
            println!("Access Token: {}", val);
            let res = db.store_invalidated_token(val.to_string());
            let response =
                json!({
                "message": "User logged out successfully!"
            });
            match res {
                Ok(_) => Ok(response),
                Err(_) => Err(BadRequest(ErrorMessages::InvalidateTokenError.error_msg())),
            }
        }
        Err(_) => Err(BadRequest(ErrorMessages::InvalidateTokenError.error_msg())),
    }
}
