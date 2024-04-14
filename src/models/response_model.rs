use serde::{ Serialize, Deserialize };
use serde_json::{ Value, json };
use axum::{ http::StatusCode };
use axum::extract::Json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T = Option<Value>> {
    #[serde(skip_serializing, skip_deserializing)]
    pub response_type: StatusCode,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBuilder<T = Option<Value>> {
    #[serde(skip_serializing, skip_deserializing)]
    pub response_type: StatusCode,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    message: String,
    code: u16,
}

impl<T: Serialize> ResponseBuilder<T> {
    pub fn new(status_code: StatusCode, message: &str) -> Self {
        let status = Status {
            message: message.to_string(),
            code: status_code.as_u16(),
        };
        Self {
            response_type: status_code,
            status,
            data: None,
            error: None,
            meta: None,
        }
    }

    pub fn data(mut self, data: T) -> Self {
        self.data = Some(data);
        self.error = None;
        self
    }

    pub fn error(mut self, message: &str) -> Self {
        self.error = Some(json!({
            "message": message
        }));
        self.data = None;
        self.meta = None;
        self
    }

    pub fn meta(mut self, meta: Value) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn build(self) -> (StatusCode, Json<Value>) {
        let obj = json!(self);
        return (self.response_type, Json(obj));
    }
}
