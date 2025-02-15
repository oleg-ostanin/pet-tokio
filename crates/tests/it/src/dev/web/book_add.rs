use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::{json, Value};
    use tracing::info;
    use lib_core::bmc::user::UserBmc;
    use lib_dto::book::BookList;
    use lib_dto::order::{OrderContent, OrderId, OrderItem};
    use lib_dto::user::{AuthCode, UserForCreate};
    use lib_utils::json::body;
    use lib_utils::json::value;

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;
    use lib_utils::rpc::request;
    use crate::utils::body_utils::message_from_response;
    use crate::utils::file_utils::from_file;

    #[tokio::test]
    async fn scenario() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;

        let handle = tokio::spawn(lib_core::notify::order::notify(ctx.pool().clone()));

        let book_list: BookList = from_file("books_refactored.json");
        let add_books_request = request("add_books", Some(book_list));
        let add_books_response = ctx.post("/api/rpc", add_books_request).await;
        assert_eq!(add_books_response.status(), StatusCode::OK);
        info!("add_books_response: {:?}", &add_books_response);
        let add_books_value = value(add_books_response).await;
        info!("add_books_value: {:?}", &add_books_value);

        let all_books_request = request("all_books", Some(Value::Null));
        let rpc_response = ctx.post("/api/rpc", all_books_request).await;
        assert_eq!(rpc_response.status(), StatusCode::OK);
        info!("all books response: {:?}", &rpc_response);
        let v = value(rpc_response).await.expect("should be valid");
        info!("all books value: {:?}", &v);
        let result = v.get("result").expect("should be valid");
        let body: BookList = body(result.clone()).expect("should ve valid");
        info!("all books: {:?}", &body);
        assert_eq!(5, body.book_list().len());

        let order_item = OrderItem::new(1, 2);
        let order_content = OrderContent::new(vec!(order_item));
        let create_order = request("create_order", Some(order_content));
        let create_order_response = ctx.post("/api/rpc", create_order).await;
        let create_order_value = value(create_order_response).await.expect("must be ok");
        info!("create order: {:?}", &create_order_value);

        let order_id = OrderId::new(1);
        let check_order = request("check_order", Some(order_id));
        let check_order_response = ctx.post("/api/rpc", check_order).await;
        let check_order_value = value(check_order_response).await.expect("must be ok");
        info!("check order: {:?}", &check_order_value);

        handle.await.expect("ok");
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