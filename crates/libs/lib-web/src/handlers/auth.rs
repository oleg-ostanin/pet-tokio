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
use lib_dto::user::{AuthCode, UserForCreate, UserForSignIn};

use crate::error::{Error, Result};

pub async fn sign_up(
    State(app_context): State<Arc<ModelManager>>,
    Json(user): Json<UserForCreate>,
) -> Result<Json<Value>> {
    let phone = user.phone.clone();
    info!("Creating user {:<12}", &phone);
    UserBmc::create(app_context.deref(), user).await?;
    let auth_code = auth_code(app_context.deref(), phone).await?;
    Ok(auth_code)
}

pub async fn check_if_exists(
    State(app_context): State<Arc<ModelManager>>,
    Json(user): Json<UserForCreate>,
) -> Result<Json<Value>> {
    let phone = user.phone;
    info!("Checking user {:<12}", &phone);
    let user_exists = UserBmc::check_if_exists(app_context.deref(), phone).await?;
    Ok(Json(json!(user_exists)))
}

pub async fn sign_in(
    State(app_context): State<Arc<ModelManager>>,
    Json(user): Json<UserForSignIn>,
) -> Result<Json<Value>> {
    let phone = user.phone.clone();
    info!("Validating user {:<12}", &phone);
    UserBmc::validate(app_context.deref(), &user).await?;
    let auth_code = auth_code(app_context.deref(), phone).await?;
    Ok(auth_code)
}

pub async fn check_code(
    State(app_context): State<Arc<ModelManager>>,
    Json(user): Json<AuthCode>,
) -> Result<()> {
    let phone = user.phone;
    info!("Checking user {:<12}", &phone);
    let mut cache = app_context.cache().read()?;
    if let Some(code) = cache.get(&phone) {
        if code.eq(&user.auth_code) {
            return Ok(());
        }
    }
    Err(Error::WebError)
}

async fn auth_code(app_context: &ModelManager, phone: String) -> Result<Json<Value>>{
    let code = Uuid::new_v4();
    let mut cache = app_context.cache().write()?;
    cache.insert(phone.clone(), code.to_string());

    let auth_code = Json(json!({
        "phone": phone,
		"auth_code": code.to_string()
	}));

    Ok(auth_code)
}