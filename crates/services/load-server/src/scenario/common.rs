use serde_json::json;
use lib_dto::order::{OrderContent, OrderId, OrderStored};
use crate::requests::user_context::UserContext;

pub(crate) const BOOKS_SIZE: usize = 5;

pub(crate) async fn create_order(user_ctx: &mut UserContext, order_content: OrderContent) -> OrderId {
    user_ctx.post_rpc("create_order", json!(order_content)).await
}

pub(crate) async fn check_order(user_ctx: &mut UserContext, order_id: OrderId) -> OrderStored {
    user_ctx.post_rpc("check_order", json!(order_id)).await
}