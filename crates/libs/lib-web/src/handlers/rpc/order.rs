use serde_json::{json, Value};
use lib_core::bmc::order::OrderBmc;
use lib_core::bmc::user::UserBmc;
use lib_core::context::app_context::ModelManager;
use lib_dto::order::{OrderContent, OrderForCreate, OrderId};

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