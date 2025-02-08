#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use axum::{Router, routing::post};

use lib_core::context::app_context::ModelManager;

use crate::handlers::auth::{check_code, sign_in, sign_up};

pub async fn auth_app(app_context: Arc<ModelManager>) -> Router {
    Router::new()
        .route("/sign-up", post(sign_up))
        .route("/sign-in", post(sign_in))
        .route("/check-code", post(check_code))
        .with_state(app_context)
}

