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
use tower_cookies::{Cookie, Cookies};
use tracing::info;
use lib_utils::constants::AUTH_TOKEN;
use lib_utils::constants::SERVICE_TOKEN_KEY;
use lib_core::context::app_context::ModelManager;
use lib_dto::user::{AuthCode, UserForCreate};
use lib_utils::jwt::token;

pub async fn login(
    State(mm): State<Arc<ModelManager>>,
    cookies: Cookies,
    Json(user): Json<AuthCode>,
) -> Result<(), StatusCode> {

    info!("{:<12} - login phone", &user.phone);
    info!("{:<12} - login code", &user.auth_code);

    let client: Client<HttpConnector, Body> =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build_http();

    let addr = mm.app_config().auth_url.as_str();
    info!("{:<12} - auth url", &addr);

    let check_response = client
        .request(Request::builder()
            .method(http::Method::POST)
            .uri(format!("{addr}/check-code"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_string(&json!(user)).unwrap()))
            .unwrap())
        .await
        .unwrap();

    let token_key = std::env::var("SERVICE_TOKEN_KEY").expect("TOKEN must be set.");
    if check_response.status() == StatusCode::OK {
        let token = token(&user.phone, token_key.as_str());
        let mut cookie = Cookie::new(AUTH_TOKEN, token);
        cookie.set_http_only(true);
        cookie.set_path("/");
        cookies.add(cookie);

        info!("{:<12} - login code", &user.auth_code);
        return Ok(());
    }

    Err(StatusCode::FORBIDDEN)
}