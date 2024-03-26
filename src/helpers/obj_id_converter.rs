use mongodb::bson::oid::ObjectId;
use actix_web::{ Result };
use crate::{ helpers::errors::ServiceError };

pub struct Converter;

impl Converter {
    pub fn string_to_bson(id: String) -> Result<ObjectId, ServiceError> {
        let obj_id = match ObjectId::parse_str(&id) {
            Ok(obj_id) => obj_id,
            Err(_) => {
                return Err(ServiceError::BadRequest("Invalid ID format".to_string()));
            }
        };
        Ok(obj_id)
    }
}
