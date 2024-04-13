use axum::{ response::Result, Json };
use mongodb::bson::oid::ObjectId;
use reqwest::StatusCode;

use crate::services::user::json_response;

pub struct Converter;

impl Converter {
    pub fn string_to_bson(id: String) -> Result<ObjectId, (StatusCode, Json<serde_json::Value>)> {
        let obj_id = match ObjectId::parse_str(&id) {
            Ok(obj_id) => obj_id,
            Err(_) => {
                return Err((StatusCode::BAD_REQUEST, Json(json_response("Invalid ID format."))));
            }
        };
        Ok(obj_id)
    }
}
