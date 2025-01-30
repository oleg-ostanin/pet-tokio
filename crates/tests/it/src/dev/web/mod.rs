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
    ctx.setup_mock().await;
    let auth_code = AuthCode::new("2128506".to_string(), "any_string");
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