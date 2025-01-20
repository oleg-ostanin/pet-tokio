use std::error::Error;
use std::sync::Arc;
use axum::body::Body;
use hyper::{http, Request};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use tracing::info;
use lib_core::context::app_context::ModelManager;
use lib_web::app::app::{web_app, create_app_context};

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

    let addr = "localhost:3000";

    let get_response = client
        .request(Request::builder()
            .method(http::Method::GET)
            .uri(format!("http://{addr}/get-books"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .header("cookie", "auth-token=token".to_string())
            .header("cookie", "new-auth-token=new-token".to_string())

            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    println!("{:?}", &get_response);

    Ok(())
}