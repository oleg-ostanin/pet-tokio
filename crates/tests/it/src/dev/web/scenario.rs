use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use axum::http::StatusCode;
    use serde_json::{json, Value};
    use serial_test::serial;
    use tokio::time::sleep;
    use tracing::info;
    use lib_dto::book::{BookDescription, BookList};
    use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderStored};
    use lib_utils::rpc::request;

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;
    use crate::utils::file_utils::from_file;

    #[tokio::test]
    #[serial]
    async fn scenario() {
        let mut user = TestContext::new(ServiceType::Web).await;
        login(&mut user).await;

        let book_list: BookList = from_file("books_refactored.json");
        let add_books_request = request("add_books", Some(book_list));
        let add_books_response = user.post("/api/rpc", add_books_request).await;
        assert_eq!(add_books_response.status(), StatusCode::OK);

        let all_books_request = request("all_books", Some(Value::Null));
        let book_list: BookList = user.post_ok("/api/rpc", all_books_request).await;
        assert_eq!(5, book_list.book_list().len());

        let order_item_1 = OrderItem::new(1, 2);
        let order_item_2 = OrderItem::new(2, 4);
        let order_content = OrderContent::new(vec!(order_item_1, order_item_2));
        let order_id: OrderId = user.post_rpc("create_order", json!(order_content)).await;
        assert_eq!(1, order_id.order_id());

        let check_stored: OrderStored = user.post_rpc("check_order", json!(order_id)).await;
        assert_eq!(1, check_stored.order_id());

        let description = BookDescription::new("the");
        let book_list: BookList = user.post_rpc("books_by_description", json!(description)).await;
        info!("books by description: {:?}", book_list);

        //sleep(Duration::from_secs(10)).await;
        user.cancel().await;
    }
}