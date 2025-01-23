use std::ops::Deref;
use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;
use lib_core::bmc::user::UserBmc;
use lib_core::context::app_context::ModelManager;
use lib_dto::user::UserForCreate;

pub async fn sign_up(
    State(app_context): State<Arc<ModelManager>>,
    Json(user): Json<UserForCreate>,
) -> Result<Json<Value>, StatusCode> {
    let identity = user.identity.clone();
    let code = Uuid::new_v4();
    //info!("{:<12} - identity", &identity);
    info!("{:<12} - code", &code);

    UserBmc::create(app_context.deref(), user).await;

    let auth_code = Json(json!({
        "identity": identity,
		"auth_code": code.to_string()
	}));

    Ok(auth_code)
}