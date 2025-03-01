use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use sqlx::Acquire;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;
use tracing::{info, instrument};

use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus, OrderStored};
use lib_dto::order::OrderStatus::ReadyToDeliver;
use crate::bmc::book_info::BookBmc;
use crate::bmc::general::update_storage_and_order;
use crate::bmc::order::OrderBmc;
use crate::bmc::storage::{StorageBmc, UpdateType};
use crate::bmc::storage::UpdateType::Add;
use crate::context::app_context::ModelManager;

pub async fn handle_storage(
    app_context: Arc<ModelManager>,
    mut order_item_rx: Receiver<OrderStored>,
    delivery_tx: Sender<OrderStored>,
) {
    while let Some(order) = order_item_rx.recv().await {
        info!("updating storage for order is: {:?}", &order);
        while let Err(e) = update_storage_and_order(app_context.clone(), &order, Add, ReadyToDeliver).await {
            info!("retrying update storage for order is: {:?}", &order);
            //sleep(Duration::from_millis(1000)).await;
        }
        delivery_tx.send(order).await.expect("ok")
    }
}

