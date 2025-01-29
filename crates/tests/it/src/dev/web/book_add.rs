use axum::{body::Body, http::{self, Request}};
use axum::http::HeaderValue;
use axum::response::Response;
use http_body_util::BodyExt;
use hyper::body::{Buf, Incoming};
// for `collect`
use serde_json::json;
use tower::{Service, ServiceExt};

use lib_dto::user::AuthCode;

// for `call`, `oneshot`, and `ready`

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use axum::Json;
    use uuid::Uuid;
    use lib_dto::book::BookList;
    use lib_dto::user::AuthCode;
    use lib_utils::json::value;

    use crate::context::context::{ServiceType, TestContext};

    use super::*;

    #[tokio::test]
    async fn add_books() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        ctx.setup_mock().await;
        let auth_code = AuthCode::new("2128506".to_string(), "any_string");
        let web_addr = &ctx.socket_addr;
        let login_response = ctx.post("/login", json!(auth_code)).await;
        println!("{:?}", &login_response);

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/books_refactored.json");

        println!("{}", d.display());

        let file = File::open(d).expect("Should be there");
        let reader = BufReader::new(file);
        let book_list: BookList = serde_json::from_reader(reader).expect("Should be valid");

        println!("book list: {:?}", book_list);

        let req_uuid = Uuid::new_v4().to_string();

        //{"jsonrpc": "2.0", "method": "subtract", "params": {"minuend": 42, "subtrahend": 23}, "id": 4}
        let request = json!({
        "jsonrpc": "2.0",
        "method": "add_books",
        "params": book_list,
        "id": req_uuid});
        let request_str = &request.to_string();
        println!("request_str: {:?}", &request_str);

        let rpc_response = ctx.post("/api/rpc", request).await;

        println!("{:?}", &rpc_response);
        let value = value(rpc_response).await;
        println!("{:?}", &value);
    }
}