use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::{json, Value};

    use lib_dto::book::BookList;
    use lib_dto::user::AuthCode;
    use lib_utils::json::body;
    use lib_utils::json::value;

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;
    use crate::dev::web::request;
    use crate::utils::body_utils::message_from_response;
    use crate::utils::file_utils::from_file;

    #[tokio::test]
    async fn add_books() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;

        let book_list: BookList = from_file("books_refactored.json");
        let request = request("add_books", Some(book_list));
        let rpc_response = ctx.post("/api/rpc", request).await;
        assert_eq!(rpc_response.status(), StatusCode::OK);
        println!("{:?}", &rpc_response);
        let v = value(rpc_response).await;
        println!("{:?}", &v);

        let request = crate::dev::web::request("all_books", Some(Value::Null));
        let rpc_response = ctx.post("/api/rpc", request).await;
        assert_eq!(rpc_response.status(), StatusCode::OK);
        println!("all books response: {:?}", &rpc_response);
        let v = value(rpc_response).await;
        println!("all books value: {:?}", &v);
        let result = v.get("result").expect("should be valid");
        let body: BookList = body(result.clone());
        println!("all books: {:?}", &body);
        assert_eq!(5, body.book_list.len());
    }

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