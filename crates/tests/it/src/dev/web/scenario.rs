use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::http::StatusCode;
    use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
    use rdkafka::Message;
    use serde_json::{json, Value};
    use serial_test::serial;
    use tokio::select;
    use tokio::time::sleep;
    use tokio_util::sync::CancellationToken;
    use tracing::info;
    use tracing::log::error;
    use lib_dto::book::BookList;
    use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderStatus, OrderStored};
    use lib_load::requests::user_context::UserContext;
    use lib_load::scenario::books::BOOK_LIST;
    use lib_utils::json::body;
    use lib_utils::rpc::request;
    use lib_core::task::kafka::consumer_task::create;

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

        let iterations = 16;

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

        let orders = check_orders(&user, order_ids).await;
        check_kafka(&ctx, orders).await;

        //sleep(Duration::from_secs(3)).await;

        ctx.cancel().await;

        //let token = ctx.app_context().cancellation_token();
        // let cloned_token = token.clone();
        //
        // let join_handle = tokio::spawn(async move {
        //     // Wait for either cancellation or a very long time
        //     select! {
        //     _ = cloned_token.cancelled() => {
        //         info!("Cancelled by cancellation token.");
        //
        //         // The token was cancelled
        //         5
        //     }
        //     _ = tokio::time::sleep(std::time::Duration::from_secs(9999)) => {
        //         99
        //     }
        // }
        // });

        // tokio::spawn(async move {
        //     //tokio::time::sleep(Duration::from_millis(10)).await;
        //     token.cancel();
        // });
    }

    async fn check_orders(user: &UserContext, order_ids: Vec<i64>) -> Vec<OrderStored> {
        let mut orders = Vec::with_capacity(order_ids.len());
        for order_id in order_ids {
            let check_order_id = OrderId::new(order_id);
            let check_stored: OrderStored = user.post_rpc("check_order", json!(check_order_id)).await;
            assert_eq!(&OrderStatus::Delivered, check_stored.status());
            orders.push(check_stored);
        }
        orders
    }

    async fn check_kafka(ctx: &TestContext, orders: Vec<OrderStored>) {
        let app_config = ctx.app_context().app_config();
        let consumer = create(app_config.clone(), "test_group").await;

        info!("Handling kafka consumer task");
        consumer.subscribe(
            &["order-topic"]
        ).expect("Can't Subscribe");

        let mut orders_from_kafka = Vec::with_capacity(orders.len());

        while orders_from_kafka.len() < orders.len() {
            match consumer.recv().await {
                Err(e) => error!("{:?}",e),
                Ok(message) => {
                    match message.payload_view::<str>() {
                        None => info!("None message"),
                        Some(Ok(msg)) => {
                            info!("Message Consumed in test: {}", &msg);
                            let order: OrderStored = serde_json::from_str(msg).expect("must be ok");
                            orders_from_kafka.push(order);
                        },
                        Some(Err(e)) => error!("Error Parsing : {}",e)
                    }
                    consumer.commit_message(&message, CommitMode::Async).unwrap();
                }
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn cancel() {
        let mut ctx = TestContext::new(ServiceType::Web).await;

        let token = ctx.app_context().cancellation_token();
        let cloned_token = token.clone();

        let join_handle = tokio::spawn(async move {
            // Wait for either cancellation or a very long time
            select! {
            _ = cloned_token.cancelled() => {
                info!("Cancelled by cancellation token.");

                // The token was cancelled
                5
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(9999)) => {
                99
            }
        }
        });

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            token.cancel();
        });

        assert_eq!(5, join_handle.await.unwrap());
    }
}