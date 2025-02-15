use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use serial_test::serial;

    use lib_dto::order::{OrderContent, OrderItem};

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;

    #[tokio::test]
    #[serial]
    async fn unknown_method() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;

        let (message, detail) = ctx.post_bad("wrong_method", json!(Some(Value::Null))).await;
        assert_eq!("RPC_REQUEST_INVALID", message);
        assert_eq!("Unknown method: wrong_method", detail);
    }

    #[tokio::test]
    #[serial]
    async fn no_params() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;

        let (message, detail) = ctx.post_bad("create_order", json!(Some(Value::Null))).await;
        assert_eq!("RPC_REQUEST_INVALID", message);
        assert_eq!("No params", detail);
    }

    #[tokio::test]
    #[serial]
    async fn wrong_params() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;

        let order_item = OrderItem::new(1, 2);
        let order_content = OrderContent::new(vec!(order_item));

        let (message, detail) = ctx.post_bad("check_order", json!(order_content)).await;
        assert_eq!("RPC_REQUEST_INVALID", message);
        assert_eq!("Wrong params", detail);
    }
}