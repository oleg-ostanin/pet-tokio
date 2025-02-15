use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;

    use lib_dto::book::BookList;
    use lib_dto::user::AuthCode;
    use lib_utils::rpc::request;

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;
    use crate::utils::body_utils::message_from_response;
    use crate::utils::file_utils::from_file;

    #[tokio::test]
    async fn without_login() {
        let mut ctx = TestContext::new(ServiceType::Web).await;

        let book_list: BookList = from_file("books_refactored.json");
        let request = request("add_books", Some(book_list));
        let rpc_response = ctx.post("/api/rpc", request).await;

        assert_eq!(rpc_response.status(), StatusCode::FORBIDDEN);
        let message = message_from_response(rpc_response).await;
        assert_eq!(message, "LOGIN_FAIL");
    }

    #[tokio::test]
    async fn login_forbidden() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;
        let auth_code_invalid = AuthCode::new("2128506".to_string(), "invalid_code");
        ctx.mock_forbidden(json!(auth_code_invalid)).await;

        let book_list: BookList = from_file("books_refactored.json");
        let request = request("add_books", Some(book_list));
        let rpc_response = ctx.post("/api/rpc", request).await;

        assert_eq!(rpc_response.status(), StatusCode::FORBIDDEN);
        let message = message_from_response(rpc_response).await;
        assert_eq!(message, "LOGIN_FAIL");
    }
}