use http_body_util::BodyExt;
use hyper::body::Buf;
// for `collect`
use serde_json::json;
use tower::{Service, ServiceExt};

use lib_dto::user::AuthCode;

// for `call`, `oneshot`, and `ready`

#[cfg(test)]
mod tests {
    use lib_dto::user::AuthCode;
    use lib_utils::json::value;

    use crate::context::context::{ServiceType, TestContext};

    use super::*;

    #[tokio::test]
    async fn rpc() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        let auth_code = AuthCode::new("2128506".to_string(), "any_string");
        ctx.mock_ok(json!(auth_code)).await;
        let web_addr = &ctx.socket_addr;

        let login_response = ctx.post("/login", json!(auth_code)).await;

        println!("{:?}", &login_response);

        //{"jsonrpc": "2.0", "method": "subtract", "params": {"minuend": 42, "subtrahend": 23}, "id": 4}
        let request = json!({
        "jsonrpc": "2.0",
        "method": "get",
        "params": {"minuend": 42, "subtrahend": 23},
        "id": 4});
        let request_str = &request.to_string();
        println!("request_str: {:?}", &request_str);

        let rpc_response = ctx.post("/api/rpc", request).await;

        println!("{:?}", &rpc_response);
        let value = value(rpc_response).await;
        println!("{:?}", &value);
    }
}