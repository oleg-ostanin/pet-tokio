use redis::ExpireOption::NONE;
use serde_json::{json, Value};
use tracing::error;
use lib_core::bmc::order::OrderBmc;
use lib_core::bmc::storage::StorageBmc;
use lib_core::bmc::user::UserBmc;
use lib_core::context::app_context::ModelManager;
use lib_dto::order::{OrderContent, OrderForCreate, OrderId, OrderStatus};

use crate::ctx::Ctx;

pub(super) async fn create_order(mm: &ModelManager, params: Value, ctx: Ctx) -> crate::error::Result<Value> {
    let order_content: OrderContent = serde_json::from_value(params)?;
    let user_stored = UserBmc::get_by_phone(mm, ctx.phone()).await?;
    let order_for_create = OrderForCreate::new(user_stored.id(), order_content);
    let order_id = OrderBmc::create(mm, order_for_create).await.unwrap();

    Ok(json!(order_id))
}

pub(super) async fn check_order(mm: &ModelManager, params: Value, ctx: Ctx) -> crate::error::Result<Value> {
    //todo return 404 when order id does not exist
    let order_id: OrderId = serde_json::from_value(params)?;
    let order_stored = OrderBmc::get_by_id(mm, order_id.order_id()).await?;
    let user_stored = UserBmc::get_by_id(mm, order_stored.user_id()).await?;
    if !ctx.phone().eq(user_stored.phone()) {
        return Err(crate::error::Error::UnauthorizedAccess)
    }

    Ok(json!(order_stored))
}

pub(super) async fn pick_up_order(mm: &ModelManager, params: Value, ctx: Ctx) -> crate::error::Result<Value> {
    //todo return 404 when order id does not exist
    let order_id: OrderId = serde_json::from_value(params)?;
    let order_stored = OrderBmc::get_by_id(mm, order_id.order_id()).await?;
    let user_stored = UserBmc::get_by_id(mm, order_stored.user_id()).await?;
    if !ctx.phone().eq(user_stored.phone()) {
        return Err(crate::error::Error::UnauthorizedAccess)
    }
    match OrderBmc::update_status(mm, order_id.order_id(), OrderStatus::Delivered).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to update status for order {}, error: {:?}", order_id.order_id(), e)
        }
    }
    Ok(json!(order_stored))
}

pub(super) async fn clean_up(mm: &ModelManager) -> crate::error::Result<Value> {
    OrderBmc::cleanup_orders(mm).await?;
    StorageBmc::cleanup_storage(mm).await?;
    Ok(json!("Ignored"))
}