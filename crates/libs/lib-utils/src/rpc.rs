use serde::Serialize;
use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;

pub fn request(method: impl Into<String>, params: Option<impl Serialize>) -> Value {
    let req_uuid = Uuid::new_v4().to_string();
    let method: String = method.into();
    let request: Value = match params {
        Some(params) => {
            json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": req_uuid})
        }
        None => {
            json!({
                "jsonrpc": "2.0",
                "method": method,
                "id": req_uuid})
        }
    };
    let request_str = &request.to_string();
    info!("request_str: {:?}", &request_str);
    request
}