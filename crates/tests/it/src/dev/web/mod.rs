use serde::Serialize;
use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;
use lib_dto::user::AuthCode;
use crate::context::context::{ServiceType, TestContext};

mod rpc;
mod book_add;

/// performs login for further RPC requests
async fn login(ctx: &mut TestContext) {
    let auth_code = AuthCode::new("2128506".to_string(), "valid_code");
    let auth_code_invalid = AuthCode::new("2128506".to_string(), "invalid_code");
    ctx.mock_ok(json!(auth_code)).await;
    ctx.mock_forbidden(json!(auth_code_invalid)).await;
    let login_response = ctx.post("/login", json!(auth_code)).await;
    info!("{:?}", &login_response);
}

fn request(method: impl Into<String>, params: Option<impl Serialize>) -> Value {
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
    println!("request_str: {:?}", &request_str);
    request
}