use std::ops::Deref;
use std::sync::Arc;
use sqlx::types::Json;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderItemExt, OrderStored};

use crate::bmc::book_info::BookBmc;
use crate::bmc::storage::StorageBmc;
use crate::context::app_context::ModelManager;

enum CheckResult {
    Enough,
    NotEnough(Vec<OrderItem>),
}

pub async fn handle_order(
    app_context: Arc<ModelManager>,
    mut order_rx: Receiver<OrderStored>,
    storage_tx: Sender<OrderStored>,
    delivery_tx: Sender<OrderStored>,
) {
    while let Some(order) = order_rx.recv().await {
        info!("received order is {:?}", &order);

        storage_tx.send(order).await.unwrap();
    }
}