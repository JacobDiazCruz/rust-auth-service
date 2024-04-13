use mongodb::{
    bson::{ extjson::de::Error, doc, oid::ObjectId },
    results::{ InsertOneResult, UpdateResult },
    sync::{ Client, Collection },
};
use std::env;
use crate::{ models::user_model::{ User, UserVerificationCode } };
use serde::{ Serialize, Deserialize };

pub struct Mongo {
    user_col: Collection<User>,
    verification_codes_col: Collection<UserVerificationCode>,
    invalidated_tokens_col: Collection<InvalidateTokenPayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvalidateTokenPayload {
    access_token: String,
}

impl Mongo {
    pub fn init() -> Self {
        let uri: String = env::var("MONGO_URI").expect("MONGO_URI environment variable not set");

        let client: Client = Client::with_uri_str(uri).unwrap();
        let db = client.database("rustDB");
        let user_col: Collection<User> = db.collection("users");
        let verification_codes_col: Collection<UserVerificationCode> =
            db.collection("verification_codes");
        let invalidated_tokens_col: Collection<InvalidateTokenPayload> =
            db.collection("invalidated_tokens");
        Mongo { user_col, invalidated_tokens_col, verification_codes_col }
    }

    pub fn create_user(&self, new_user: &User) -> Result<InsertOneResult, Error> {
        let data = User {
            id: None,
            name: new_user.name.clone(),
            email: new_user.email.clone(),
            password: if let Some(password) = new_user.password.clone() {
                Some(password)
            } else {
                None
            },
            is_verified: new_user.is_verified.clone(),
            login_type: new_user.login_type.clone(),
        };
        let user = self.user_col.insert_one(data, None).ok().expect("Error Creating User");
        Ok(user)
    }

    pub fn get_user_by_id(&self, user_id: ObjectId) -> Result<Option<User>, Error> {
        let filter = doc! { "_id": user_id };
        let user = self.user_col.find_one(filter, None).ok().expect("Error Getting User");
        Ok(user)
    }

    pub fn get_user_by_email(&self, email: String) -> Result<Option<User>, Error> {
        let filter = doc! { "email": email };
        let user = self.user_col.find_one(filter, None).ok().expect("Error Getting User");
        Ok(user)
    }

    pub fn store_verification_code(
        &self,
        data: UserVerificationCode
    ) -> Result<InsertOneResult, Error> {
        let user = self.verification_codes_col
            .insert_one(data, None)
            .ok()
            .expect("Error in Storing Verification Code Data.");
        Ok(user)
    }

    pub fn delete_verification_codes(&self, email: &str) -> Result<String, Error> {
        let filter = doc! {
            "email": email
        };
        let _ = self.verification_codes_col
            .delete_many(filter, None)
            .ok()
            .expect("Error in Storing Verification Code Data.");
        Ok("Verification code deleted successfully!".to_string())
    }

    pub fn get_verification_code(
        &self,
        data: UserVerificationCode
    ) -> Result<Option<UserVerificationCode>, String> {
        let filter = doc! { "email": data.email.get_email(), "code": data.code };
        match self.verification_codes_col.find_one(filter, None) {
            Ok(verif_code_data) => Ok(verif_code_data),
            Err(_) => {
                println!("Error. code did not match!");
                Err("".to_string())
            }
        }
    }

    pub fn update_user_verification(&self, email: &str) -> Result<UpdateResult, Error> {
        let filter = doc! { "email": email };
        let update = doc! { "$set": { "is_verified": true } };

        let res = self.user_col
            .update_one(filter, update, None)
            .ok()
            .expect("Error Updating User.");

        Ok(res)
    }

    pub fn store_invalidated_token(&self, access_token: String) -> Result<InsertOneResult, Error> {
        let data = InvalidateTokenPayload {
            access_token,
        };
        let result = self.invalidated_tokens_col
            .insert_one(data, None)
            .ok()
            .expect("Error Invalidating Token");
        Ok(result)
    }
}
