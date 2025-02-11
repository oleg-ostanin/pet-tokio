mod requests;
mod utils;

use std::error::Error;
use std::sync::Arc;

use axum::body::Body;
use axum::http::HeaderValue;
use axum::Json;
use axum::response::Response;
use http_body_util::BodyExt;
use hyper::{http, Request};
use hyper::body::Buf;
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::{json, Value};
use tracing::info;

use lib_core::context::app_context::ModelManager;
use lib_dto::book::BookList;
use lib_dto::order::{OrderContent, OrderItem};
use lib_dto::user::{AuthCode, UserExists, UserForCreate, UserForSignIn};
use lib_utils::json::{body, value};
use lib_utils::rpc::request;
use crate::requests::user_context::UserContext;
use crate::utils::file::from_file;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();
    info!("info");
    println!("starts");

    let mut user_ctx = UserContext::new("2128506".to_string()).await;
    let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");
    let check_response = user_ctx.post("/check-if-exists", json!(&user_to_create)).await;

    println!("check_response: {:?}", &check_response);
    let check_user_value = value(check_response).await.expect("must be ok");
    println!("check_user_value: {:?}", &check_user_value);
    let user_exists: UserExists = body(check_user_value).expect("must be ok");

    if !user_exists.exists {
        user_ctx.post("/sign-up", json!(user_to_create)).await;
    }
    let user_to_sign_in = UserForSignIn::new("2128506", "pwd");
    let sign_in_response = user_ctx.post("/sign-in", json!(user_to_sign_in)).await;
    let auth_code = message_from_response(sign_in_response).await;

    println!("auth_code: {:?}", &auth_code);

    let auth_code = AuthCode::new("2128506".to_string(), auth_code);
    user_ctx.post("/login", json!(auth_code)).await;

    assert!(user_ctx.auth_token().is_some());

    let book_list: BookList = from_file("books_refactored.json");
    let add_books = request("add_books", Some(book_list));
    let rpc_response = user_ctx.post("/api/rpc", add_books).await;

    let order_item = OrderItem::new(1, 2);
    let order_content = OrderContent::new(vec!(order_item));
    let create_order = request("create_order", Some(order_content));
    let create_order_response = user_ctx.post("/api/rpc", create_order).await;
    let create_order_value = value(create_order_response).await.expect("must be ok");
    println!("create order: {:?}", &create_order_value);

    let check_order = request("check_order", Some(create_order_value.get("result")));
    let check_order_response = user_ctx.post("/api/rpc", check_order).await;
    let check_order_value = value(check_order_response).await.expect("must be ok");
    println!("check order: {:?}", &check_order_value);

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