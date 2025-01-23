use std::error::Error;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use http_body_util::BodyExt;
use hyper::{http, Request};
use hyper::body::Buf;
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::{json, Value};
use tower_cookies::Cookies;
use tracing::info;

use lib_core::context::app_context::ModelManager;
use lib_dto::user::{AuthCode, UserForCreate};

pub async fn login(
    State(app_context): State<Arc<ModelManager>>,
    Json(user): Json<AuthCode>,
) -> Result<(), StatusCode> {

    info!("{:<12} - login identity", &user.identity);
    info!("{:<12} - login code", &user.auth_code);

    let client: Client<HttpConnector, Body> =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build_http();

    let addr = "localhost:3001";

    let check_response = client
        .request(Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{addr}/check-code"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .header("cookie", "auth-token=token".to_string())
            .header("cookie", "new-auth-token=new-token".to_string())

            .body(Body::from(serde_json::to_string(&json!(user)).unwrap()))
            .unwrap())
        .await
        .unwrap();

    if check_response.status() == StatusCode::OK {
        info!("{:<12} - login code", &user.auth_code);
        return Ok(());
    }

    Err(StatusCode::FORBIDDEN)
}