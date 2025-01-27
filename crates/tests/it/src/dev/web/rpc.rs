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
    use axum::Json;

    use lib_dto::user::AuthCode;
    use lib_utils::json::value;

    use crate::context::context::{ServiceType, TestContext};

    use super::*;

    #[tokio::test]
    async fn rpc() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        ctx.setup_mock().await;
        let auth_code = AuthCode::new("2128506".to_string(), "any_string");
        let web_addr = &ctx.socket_addr;

        let login_response = ctx.client
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

        let rpc_response = ctx.client
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
    }
}

pub(crate) fn extract_token(response: Response<Incoming>) -> String {
    let headers = response.headers();
    let value: Option<&HeaderValue> = headers.get("set-cookie");
    let s = value.unwrap().to_str().unwrap();

    println!("auth token: {:?}", &s);
    s.to_string()
}