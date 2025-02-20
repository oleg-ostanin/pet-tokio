use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus};

use crate::bmc::book_info::BookBmc;
use crate::bmc::order::OrderBmc;
use crate::bmc::storage::StorageBmc;
use crate::context::app_context::ModelManager;
use crate::notify::order::OrderPayload;

pub async fn handle_storage(
    app_context: Arc<ModelManager>,
    mut order_item_rx: Receiver<(OrderId, Vec<OrderItem>)>,
    delivery_tx: Sender<OrderId>,
) {
    while let Some((order_id, items)) = order_item_rx.recv().await {
        info!("received order_id is: {:?}, items are {:?}", &order_id, &items);
        for item in items {
            let order_item = OrderItem::new(item.book_id(), item.quantity());
            StorageBmc::update_storage(app_context.deref(), &order_item).await.unwrap();
        }
        delivery_tx.send(order_id).await.expect("ok")
    }
}