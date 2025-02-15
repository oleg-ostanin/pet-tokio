use serde::Serialize;
use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;
use lib_core::bmc::user::UserBmc;
use lib_dto::user::{AuthCode, UserForCreate};

use crate::context::context::{ServiceType, TestContext};

mod scenario;
mod login;
mod bad_request;

/// performs login for further RPC requests
async fn login(ctx: &mut TestContext) {
    let auth_code = AuthCode::new("2128506".to_string(), "valid_code");
    let auth_code_invalid = AuthCode::new("2128506".to_string(), "invalid_code");
    ctx.mock_ok(json!(auth_code)).await;
    ctx.mock_forbidden(json!(auth_code_invalid)).await;
    let login_response = ctx.post("/login", json!(auth_code)).await;
    info!("{:?}", &login_response);

    let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");
    let _ = UserBmc::create(ctx.app_context(), user_to_create).await;
}