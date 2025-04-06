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
    use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderStatus, OrderStored};
    use lib_load::scenario::books::BOOK_LIST;
    use lib_utils::rpc::request;

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;

    #[tokio::test]
    #[serial]
    async fn scenario() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        let mut user = ctx.user(6);
        login(&mut ctx, &mut user).await;

        let book_list: BookList = serde_json::from_str(BOOK_LIST).expect("must be ok");
        let add_books_request = request("add_books", Some(book_list));
        let add_books_response = user.post("/api/rpc", add_books_request).await;
        assert_eq!(add_books_response.status(), StatusCode::OK);

        let all_books_request = request("all_books", Some(Value::Null));
        let book_list: BookList = user.post_ok("/api/rpc", all_books_request).await;
        assert_eq!(5, book_list.book_list().len());

        //let description = BookDescription::new("the");
        //let book_list: BookList = user.post_rpc("books_by_description", json!(description)).await;
        //info!("books by description: {:#?}", book_list);

        let iterations = 2;

        let mut order_ids: Vec<i64> = Vec::with_capacity(iterations);

        for i in (1..iterations) {
            let order_item_1 = OrderItem::new(1, 2);
            let order_item_2 = OrderItem::new(2, 4);
            let order_content = OrderContent::new(vec!(order_item_1, order_item_2));
            let order_id: OrderId = user.post_rpc("create_order", json!(order_content)).await;
            assert_eq!(i as i64, order_id.order_id());
            order_ids.push(order_id.order_id());
        }

        sleep(Duration::from_secs(3)).await;

        for i in (1..iterations) {
            let check_order_id = OrderId::new(i as i64);
            let check_stored: OrderStored = user.post_rpc("check_order", json!(check_order_id)).await;
            assert_eq!(i as i64, check_stored.order_id());
            assert_eq!(&OrderStatus::Delivered, check_stored.status());
        }

        sleep(Duration::from_secs(3)).await;

        ctx.cancel().await;
    }


}