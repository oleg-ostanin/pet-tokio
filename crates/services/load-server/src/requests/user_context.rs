use core::net::SocketAddr;
use std::fmt::Debug;
use std::sync::Arc;

use axum::{body::Body, Error, http::{self, Request, StatusCode}};
use axum::http::HeaderValue;
use axum::response::Response;
use axum::routing::get;
use dotenv::dotenv;
use http_body_util::BodyExt;
use hyper::body::{Buf, Incoming};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::{json, Value};
use sqlx::Pool;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower::builder;
use tower_cookies::{Cookie, Cookies};
use tracing::info;
use uuid::Uuid;

use lib_core::context::app_context::{AppConfig, ModelManager};
use lib_dto::user::{AuthCode, UserForCreate, UserForSignIn};
use lib_utils::constants::{AUTH_SOCKET_ADDR, WEB_SOCKET_ADDR};
use lib_web::app::auth_app::auth_app;
use lib_web::app::web_app::web_app;

#[derive(Debug, Clone)]
struct HeaderWrapper {
    key: String,
    value: String,
}

pub(crate) struct UserContext<> {
    phone: String,
    pub(crate) client: Client<HttpConnector, Body>,
    auth_token: Option<String>,
    headers: Vec<HeaderWrapper>,
}

impl UserContext {
    pub(crate) async fn new(phone: String) -> Self {
        dotenv().ok();
        tracing_subscriber::fmt()
            .without_time() // For early local development.
            .with_target(false)
            .init();
        info!("delete me");

        let client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        Self {
            phone,
            client,
            auth_token: None,
            headers: Vec::new(),
        }
    }

    pub(crate) fn invalidate_token(&mut self) -> Option<String> {
        self.auth_token.take()
    }

    pub(crate) async fn create_user(&mut self, user_body: &UserForCreate) -> Response<Incoming> {
        self.post("/sign-up", json!(user_body)).await
    }

    pub(crate) async fn sign_in_user(&mut self, user_body: UserForSignIn) -> Response<Incoming> {
        self.post("/sign-in", json!(user_body)).await
    }

    pub(crate) async fn check_code(&mut self, user_body: AuthCode) -> Response<Incoming> {
        self.post("/check-code", json!(user_body)).await
    }

    pub(crate) async fn post(&mut self, path: impl Into<String>, body: Value) -> Response<Incoming> {
        let path: String = path.into();
        let addr = &self.socket_addr(&path);

        let mut builder = Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{addr}{path}"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref());

        if let Some(auth_token) = self.auth_token.clone() {
            builder = builder.header("cookie", auth_token)
        }

        let builder = builder.body(Body::from(serde_json::to_string(&json!(body)).unwrap())).unwrap();
        let response = self.client
            .request(builder)
            .await
            .unwrap();

        let token = extract_token(&response);
        if let Some(token) = token {
            self.auth_token = Some(token);
        }

        response
    }

    fn socket_addr(path: &str) -> String {
        if path.starts_with("/login") || path.starts_with("/api") {
            info!("returning web socket address");
            return std::env::var(WEB_SOCKET_ADDR).expect("Must be set.");
        }
        std::env::var(AUTH_SOCKET_ADDR).expect("Must be set.")
    }
}

pub(crate) fn extract_token(response: &Response<Incoming>) -> Option<String> {
    let headers = response.headers();
    let value: Option<&HeaderValue> = headers.get("set-cookie");
    if let Some(value) = value {
        return Some(value.to_str().unwrap().to_string())
    }
    None
}


