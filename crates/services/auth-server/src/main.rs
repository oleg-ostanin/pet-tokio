use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::routing::{get, post};
use tracing::info;
use lib_core::context::app_context::ModelManager;
use lib_web::app::auth_app::auth_app;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use dotenv::dotenv;

use lib_dto::user::{AuthCode, UserForCreate};
use uuid::Uuid;
use lib_web::app::context::create_app_context;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();
    info!("starting auth server");
    let app_context: Arc<ModelManager> = create_app_context().await;
    let app = auth_app(app_context).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    Ok(axum::serve(listener, app).await?)
}



