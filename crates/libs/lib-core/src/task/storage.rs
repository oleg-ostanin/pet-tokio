use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus, OrderStored};

use crate::bmc::book_info::BookBmc;
use crate::bmc::order::OrderBmc;
use crate::bmc::storage::StorageBmc;
use crate::context::app_context::ModelManager;

pub async fn handle_storage(
    app_context: Arc<ModelManager>,
    mut order_item_rx: Receiver<OrderStored>,
    delivery_tx: Sender<OrderStored>,
) {
    while let Some(order) = order_item_rx.recv().await {
        info!("received order is: {:?}", &order);
        for item in order.content() {
            let order_item = OrderItem::new(item.book_id(), item.quantity());
            StorageBmc::update_storage(app_context.deref(), &order_item).await.unwrap();
        }
        delivery_tx.send(order).await.expect("ok")
    }
}