use serde::{ Serialize, Deserialize };
use mongodb::bson::oid::ObjectId;

use super::user_model::Email;

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub email: Email,
    pub refresh_token: String,
}
