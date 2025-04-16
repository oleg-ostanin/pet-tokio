use std::sync::Mutex;

use axum::{body::Body, http::{self, Request}};
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use dotenv::dotenv;
use hyper::body::{Incoming};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info};

use lib_dto::user::{AuthCode, UserForCreate, UserForSignIn};
use lib_utils::constants::{AUTH_SOCKET_ADDR, WEB_SOCKET_ADDR};
use lib_utils::json::result;
use lib_utils::rpc::request;

use crate::utils::body_utils::message_and_detail;

pub struct UserContext {
    idx: usize,
    // todo make Arc
    phone: String,
    pub client: Client<HttpConnector, Body>,
    test_socket_addr: Option<String>,
    auth_token: Mutex<Option<String>>,
}

impl UserContext {
    pub fn new(idx: usize) -> Self {
        dotenv().ok();

        let phone = format!("{}", 2128500 + idx);

        let client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        Self {
            idx,
            phone,
            client,
            test_socket_addr: None,
            auth_token: Mutex::new(None),
        }
    }

    pub fn with_socket_address(idx: usize, test_socket_addr: Option<String>) -> Self {
        dotenv().ok();

        let phone = format!("{}", 2128500 + idx);

        let client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        Self {
            idx,
            phone,
            client,
            test_socket_addr,
            auth_token: Mutex::new(None),
        }
    }

    pub async fn clean_up(&mut self) {
        info!("Cleaning up tables");
        let request = request("clean_up", Some("ignored"));
        self.post("/api/rpc", request).await;
    }

    pub async fn create_user(&mut self, user_body: &UserForCreate) -> Response<Incoming> {
        self.post("/sign-up", json!(user_body)).await
    }

    pub async fn sign_in_user(&mut self, user_body: UserForSignIn) -> Response<Incoming> {
        self.post("/sign-in", json!(user_body)).await
    }

    pub async fn check_code(&mut self, user_body: AuthCode) -> Response<Incoming> {
        self.post("/check-code", json!(user_body)).await
    }

    pub async fn post(&self, path: impl Into<String>, body: Value) -> Response<Incoming> {
        let path: String = path.into();
        let addr = &self.socket_addr(&path);

        debug!("socket_addr_in_post: {:#?}", &addr);
        let mut builder = Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{addr}{path}"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref());

        if let Ok(guard) = self.auth_token.lock() {
            if let Some(auth_token) = guard.as_ref() {
                builder = builder.header("cookie", auth_token)
            }
        }

        let builder = builder.body(Body::from(serde_json::to_string(&json!(body)).unwrap())).unwrap();
        let response = self.client
            .request(builder)
            .await
            .unwrap();

        let token = extract_token(&response);
        if let Some(token) = token {
            if let Ok(mut guard) = self.auth_token.lock() {
                let _ = guard.insert(token);
            }
        }

        response
    }

    pub async fn post_rpc<T: for<'a> Deserialize<'a>>(&self, rpc_path: impl Into<String>, body: Value) -> T {
        let body = request(rpc_path, Some(body));
        let response = self.post("/api/rpc", body).await;
        assert_eq!(response.status(), StatusCode::OK);
        result(response).await.expect("must be ok")
    }

    pub async fn post_ok<T: for<'a> Deserialize<'a>>(&self, path: impl Into<String>, body: Value) -> T {
        let response = self.post(path, body).await;
        assert_eq!(response.status(), StatusCode::OK);
        result(response).await.expect("must be ok")
    }

    pub async fn post_bad(&self, path: impl Into<String>, body: Value) -> (String, String) {
        let body = request(path, Some(body));
        let response = self.post("/api/rpc", body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        message_and_detail(response).await
    }


    fn socket_addr(&self, path: &str) -> String {
        if let Some(addr) = &self.test_socket_addr {
            return addr.to_string();
        }

        if path.starts_with("/login") || path.starts_with("/api") {
            info!("returning web socket address");
            return std::env::var(WEB_SOCKET_ADDR).expect("Must be set.");
        }
        std::env::var(AUTH_SOCKET_ADDR).expect("Must be set.")
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }

    pub fn client(&self) -> &Client<HttpConnector, Body> {
        &self.client
    }

    pub fn auth_token(&self) -> &Mutex<Option<String>> {
        &self.auth_token
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

pub fn extract_token(response: &Response<Incoming>) -> Option<String> {
    let headers = response.headers();
    let value: Option<&HeaderValue> = headers.get("set-cookie");
    if let Some(value) = value {
        return Some(value.to_str().unwrap().to_string())
    }
    None
}


