use std::error::Error;
use std::sync::Arc;
use axum::body::Body;
use axum::response::Response;
use hyper::{http, Request};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::{json, Value};
use tracing::info;
use lib_core::context::app_context::ModelManager;
use lib_dto::user::{AuthCode, UserForCreate};

use http_body_util::BodyExt;
use hyper::body::Buf;

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

    let addr = "localhost:3001";

    let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");


    let get_response = client
        .request(Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{addr}/create-code"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .header("cookie", "auth-token=token".to_string())
            .header("cookie", "new-auth-token=new-token".to_string())

            .body(Body::from(serde_json::to_string(&json!(user_to_create)).unwrap()))
            .unwrap())
        .await
        .unwrap();

    let auth_code = message_from_response(get_response).await;

    println!("{:?}", &auth_code);

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