use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;

use sqlx::{Postgres, Transaction};
use tracing::{debug, instrument};

use lib_dto::order::{OrderItem, OrderStatus, OrderStored};

use crate::bmc::order::OrderBmc;
use crate::bmc::storage::{StorageBmc, UpdateType};
use crate::context::app_context::ModelManager;
use crate::error::Result;

const SET_TX_ISOLATION_LEVEL: &str = r#"
SET TRANSACTION ISOLATION LEVEL
"#;


#[instrument(skip_all)]
pub(crate) async fn update_storage_and_order(
    app_context: Arc<ModelManager>,
    order: &OrderStored,
    update_type: UpdateType,
    new_status: OrderStatus,
) -> Result<()> {
    let mut book_ids = Vec::with_capacity(order.content().len());
    for order_item in order.content() {
        let book_id = order_item.book_id();
        book_ids.push(book_id);
    }

    let mut tx = app_context.pg_pool()
        .begin()
        .await?;

    tx_isolation_level(&mut tx, "REPEATABLE READ").await?;

    let book_storage_infos = StorageBmc::get_quantity_tx(&mut tx, book_ids).await?;
    debug!("book_storage_info: {:#?}", book_storage_infos);

    let map: HashMap<i64, i64> = book_storage_infos
        .into_iter()
        .map(|info| (info.id(), info.quantity().unwrap_or(0)))
        .collect();

    for order_item in order.content() {
        let old_quantity = map.get(&order_item.book_id()).unwrap_or(&0i64);
        let new_quantity = match update_type {
            UpdateType::Add => {old_quantity + order_item.quantity()}
            UpdateType::Remove => {old_quantity - order_item.quantity()}
        } ;
        let new_order_item = OrderItem::new(order_item.book_id(), new_quantity);
        StorageBmc::update_storage_tx(&mut tx, &new_order_item).await?;
    }

    OrderBmc::update_status_tx(&mut tx, order.order_id(), new_status).await?;

    tx.commit().await?;

    Ok(())
}

pub async fn tx_isolation_level(
    tx: &mut Transaction<'_, Postgres>,
    isolation_level: &str,
) -> Result<()> {
    let query = SET_TX_ISOLATION_LEVEL.to_string().add(isolation_level);
    sqlx::query(&query)
        .fetch_optional(& mut **tx)
        .await?;

    Ok(())
}