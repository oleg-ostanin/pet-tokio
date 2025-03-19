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
use lib_load::requests::user_context::UserContext;
use lib_load::scenario::load::start_user;
use lib_load::scenario::stage_01::load;
use lib_utils::json::{body, value};
use lib_utils::rpc::request;
// use crate::requests::user_context::UserContext;
// use crate::scenario::load::start_user;
// use crate::scenario::stage_01::load;
// use crate::utils::file::from_file;

const USERS_NUM: usize = 1;

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
