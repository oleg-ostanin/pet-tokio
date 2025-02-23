mod requests;
mod scenario;
mod utils;

use std::error::Error;
use std::time::Duration;
use axum::http::HeaderValue;
use axum::response::Response;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::body::Incoming;
use serde_json::{json, Value};
use tokio::time::sleep;
use tracing::info;

use lib_dto::book::BookList;
use lib_dto::order::{OrderContent, OrderItem};
use lib_dto::user::{AuthCode, UserExists, UserForCreate, UserForSignIn};
use lib_utils::json::{body, value};
use lib_utils::rpc::request;
use crate::requests::user_context::UserContext;
use crate::scenario::load::start_user;
use crate::scenario::stage_01::load;
use crate::utils::file::from_file;

const USERS_NUM: usize = 2;
pub(crate) const ITERATIONS: i64 = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();
    info!("info");
    info!("starts");

    let mut users: Vec<UserContext> = Vec::with_capacity(USERS_NUM);
    for i in 1..=USERS_NUM {
        users.push(start_user(i).await);
    }
    users.get_mut(0).expect("must be some").clean_up().await;
    sleep(Duration::from_secs(1)).await;
    load(users).await;

    Ok(())
}

//#[tokio::main]
async fn main_old() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();
    info!("info");
    info!("starts");

    let mut user_ctx = UserContext::new(0, "2128506".to_string()).await;
    let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");
    let check_response = user_ctx.post("/check-if-exists", json!(&user_to_create)).await;

    info!("check_response: {:?}", &check_response);
    let check_user_value = value(check_response).await.expect("must be ok");
    info!("check_user_value: {:?}", &check_user_value);
    let user_exists: UserExists = body(check_user_value).expect("must be ok");

    if !user_exists.exists {
        user_ctx.post("/sign-up", json!(user_to_create)).await;
    }
    let user_to_sign_in = UserForSignIn::new("2128506", "pwd");
    let sign_in_response = user_ctx.post("/sign-in", json!(user_to_sign_in)).await;
    let auth_code = message_from_response(sign_in_response).await;

    info!("auth_code: {:?}", &auth_code);

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
    info!("create order: {:?}", &create_order_value);

    let check_order = request("check_order", Some(create_order_value.get("result")));
    let check_order_response = user_ctx.post("/api/rpc", check_order).await;
    let check_order_value = value(check_order_response).await.expect("must be ok");
    info!("check order: {:?}", &check_order_value);

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
