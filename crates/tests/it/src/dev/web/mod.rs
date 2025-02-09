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