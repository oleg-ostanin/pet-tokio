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
    cookies: Cookies,
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

    let token_key = std::env::var("SERVICE_TOKEN_KEY").expect("MAILCOACH_API_TOKEN must be set.");
    if check_response.status() == StatusCode::OK {
        let token = token(&user.identity, token_key.as_str());
        let mut cookie = Cookie::new(AUTH_TOKEN, token);
        cookie.set_http_only(true);
        cookie.set_path("/");
        cookies.add(cookie);

        info!("{:<12} - login code", &user.auth_code);
        return Ok(());
    }

    Err(StatusCode::FORBIDDEN)
}