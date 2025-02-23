use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;
    use serial_test::serial;
    use lib_dto::book::BookList;
    use lib_dto::user::AuthCode;
    use lib_utils::rpc::request;

    use crate::context::context::{ServiceType, TestContext};
    use lib_load::scenario::books::BOOK_LIST;
    use lib_load::utils::body_utils::message_from_response;

    #[tokio::test]
    #[serial]
    async fn without_login() {
        let mut ctx = TestContext::new(ServiceType::Web).await;

        let book_list: BookList = serde_json::from_str(BOOK_LIST).expect("must be ok");
        let request = request("add_books", Some(book_list));
        let rpc_response = ctx.post("/api/rpc", request).await;

        assert_eq!(rpc_response.status(), StatusCode::FORBIDDEN);
        let message = message_from_response(rpc_response).await;
        assert_eq!(message, "LOGIN_FAIL");
    }

    #[tokio::test]
    #[serial]
    async fn login_forbidden() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        //login(&mut ctx).await;
        let auth_code_invalid = AuthCode::new("2128506".to_string(), "invalid_code");
        ctx.mock_forbidden(json!(auth_code_invalid)).await;

        let book_list: BookList = serde_json::from_str(BOOK_LIST).expect("must be ok");
        let request = request("add_books", Some(book_list));
        let rpc_response = ctx.post("/api/rpc", request).await;

        assert_eq!(rpc_response.status(), StatusCode::FORBIDDEN);
        let message = message_from_response(rpc_response).await;
        assert_eq!(message, "LOGIN_FAIL");
    }
}