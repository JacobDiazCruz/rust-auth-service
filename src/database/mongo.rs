use mongodb::{
    bson::{ extjson::de::Error, doc, oid::ObjectId },
    results::InsertOneResult,
    sync::{ Client, Collection },
};
use std::env;
use crate::models::user_model::User;
use serde::{ Serialize, Deserialize };

pub struct Mongo {
    col: Collection<User>,
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
        let col: Collection<User> = db.collection("User");
        let invalidated_tokens_col: Collection<InvalidateTokenPayload> =
            db.collection("InvalidateTokenPayload");
        Mongo { col, invalidated_tokens_col }
    }

    pub fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let data = User {
            id: None,
            name: new_user.name,
            email: new_user.email,
        };
        let user = self.col.insert_one(data, None).ok().expect("Error Creating User");
        Ok(user)
    }

    pub fn get_user_by_id(&self, user_id: ObjectId) -> Result<Option<User>, Error> {
        let filter = doc! { "_id": user_id };
        let user = self.col.find_one(filter, None).ok().expect("Error Getting User");
        Ok(user)
    }

    pub fn get_user_by_email(&self, email: String) -> Result<Option<User>, Error> {
        let filter = doc! { "email": email };
        let user = self.col.find_one(filter, None).ok().expect("Error Getting User");
        Ok(user)
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
