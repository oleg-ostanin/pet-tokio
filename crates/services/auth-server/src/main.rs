use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::routing::{get, post};
use tracing::info;
use lib_core::context::app_context::ModelManager;
use lib_web::app::web_app::{web_app, create_app_context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use lib_dto::user::{AuthCode, UserForCreate};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();
    info!("info");
    println!("starts");

    let codes = Arc::new(Mutex::new(HashMap::new()));
    let app_context: Arc<Codes> = Arc::new(
        Codes {
            codes
        }
    );

    let app = auth_app(app_context).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    Ok(axum::serve(listener, app).await?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Codes {
    codes: Arc<Mutex<HashMap<String, String>>>
}

async fn auth_app(app_context: Arc<Codes>) -> Router {
    Router::new()
        .route("/create-code", post(create_code))
        //.route("/check-code", get(chek_code))
        .with_state(app_context)
}


async fn create_code(
    State(app_context): State<Arc<Codes>>,
    Json(user): Json<UserForCreate>,
) -> Result<Json<Value>, StatusCode> {
    let identity = user.identity;
    let code = Uuid::new_v4();
    info!("{:<12} - identity", &identity);
    info!("{:<12} - code", &code);

    if let Ok(mut map) = app_context.codes.lock() {
        map.insert(identity, code.clone().to_string());
    };

    let auth_code = Json(json!({
		"auth_code": code.to_string()
	}));

    Ok(auth_code)
}