use std::error::Error;
use std::sync::Arc;
use axum::body::Body;
use axum::http::HeaderValue;
use axum::Json;
use axum::response::Response;
use hyper::{http, Request};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::{json, Value};
use tracing::info;
use lib_core::context::app_context::ModelManager;
use lib_dto::user::{AuthCode, UserForCreate, UserForSignIn};

use http_body_util::BodyExt;
use hyper::body::Buf;
use lib_utils::json::value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();
    info!("info");
    println!("starts");

    let client: Client<HttpConnector, Body> =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build_http();

    let auth_addr = "localhost:3001";

    let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");

    // let create_response = client
    //     .request(Request::builder()
    //         .method(http::Method::POST)
    //         .uri(format!("http://{auth_addr}/sign-up"))
    //         .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    //         .body(Body::from(serde_json::to_string(&json!(user_to_create)).unwrap()))
    //         .unwrap())
    //     .await
    //     .unwrap();

    let user_to_sign_in = UserForSignIn::new("2128506", "pwd");

    let sign_in_response = client
        .request(Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{auth_addr}/sign-in"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_string(&json!(user_to_sign_in)).unwrap()))
            .unwrap())
        .await
        .unwrap();

    let auth_code = message_from_response(sign_in_response).await;

    println!("{:?}", &auth_code);

    let auth_code = AuthCode::new("2128506".to_string(), auth_code);

    let web_addr = "localhost:3000";

    let login_response = client
        .request(Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{web_addr}/login"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_string(&json!(auth_code)).unwrap()))
            .unwrap())
        .await
        .unwrap();

    println!("{:?}", &login_response);

    let token = extract_token(login_response);

    //{"jsonrpc": "2.0", "method": "subtract", "params": {"minuend": 42, "subtrahend": 23}, "id": 4}
    let request = Json(json!({
        "jsonrpc": "2.0",
        "method": "get",
        "params": {"minuend": 42, "subtrahend": 23},
        "id": 4})
    );
    let request_str = request.to_string();
    println!("request_str: {:?}", &request_str);

    let rpc_response = client
        .request(Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{web_addr}/api/rpc"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .header("cookie", token)
            .body(Body::from(request_str))
            .unwrap())
        .await
        .unwrap();

    println!("{:?}", &rpc_response);
    let value = value(rpc_response).await;
    println!("{:?}", &value);

    Ok(())
}


pub(crate) async fn message_from_response(response: Response<Incoming>) -> String {
    let body = response.collect().await.unwrap().aggregate();
    let json_value: Value = serde_json::from_reader(body.reader()).unwrap();
    get_auth_code(json_value)
}

pub(crate) fn get_auth_code(json: Value) -> String {
    let auth_code: AuthCode = serde_json::from_value(json).unwrap();
    auth_code.auth_code
}

pub(crate) fn extract_token(response: Response<Incoming>) -> String {
    let headers = response.headers();
    let value: Option<&HeaderValue> = headers.get("set-cookie");
    let s = value.unwrap().to_str().unwrap();

    println!("auth token: {:?}", &s);
    s.to_string()
}