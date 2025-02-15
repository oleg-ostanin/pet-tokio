use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::{json, Value};
    use serial_test::serial;
    use lib_dto::book::BookList;
    use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderStored};
    use lib_utils::rpc::request;

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;
    use crate::utils::file_utils::from_file;

    #[tokio::test]
    #[serial]
    async fn scenario() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;

        let handle = tokio::spawn(lib_core::notify::order::notify(ctx.pool().clone()));

        let book_list: BookList = from_file("books_refactored.json");
        let add_books_request = request("add_books", Some(book_list));
        let add_books_response = ctx.post("/api/rpc", add_books_request).await;
        assert_eq!(add_books_response.status(), StatusCode::OK);

        let all_books_request = request("all_books", Some(Value::Null));
        let book_list: BookList = ctx.post_ok("/api/rpc", all_books_request).await;
        assert_eq!(5, book_list.book_list().len());

        let order_item = OrderItem::new(1, 2);
        let order_content = OrderContent::new(vec!(order_item));
        let order_id: OrderId = ctx.post_rpc("create_order", json!(order_content)).await;
        assert_eq!(1, order_id.order_id());

        let check_stored: OrderStored = ctx.post_rpc("check_order", json!(order_id)).await;
        assert_eq!(1, check_stored.order_id());

        handle.await.expect("ok");
    }
}