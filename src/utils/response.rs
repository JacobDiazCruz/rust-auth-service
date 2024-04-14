use serde::{ Serialize, Deserialize };
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub message: String,
    pub data: Value,
}
