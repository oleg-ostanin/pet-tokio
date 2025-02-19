use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;
use lib_dto::order::{OrderId, OrderItem, OrderItemExt};
use crate::bmc::book_info::BookBmc;
use crate::context::app_context::ModelManager;
use crate::notify::order::OrderPayload;

pub async fn handle_order(
    app_context: Arc<ModelManager>,
    mut order_rx: Receiver<OrderPayload>,
    storage_tx: Sender<(OrderId, Vec<OrderItem>)>,
    delivery_tx: Sender<OrderId>,
) {
    while let Some(payload) = order_rx.recv().await {
        info!("received payload is {:?}", &payload);

        let mut not_enough: Vec<OrderItem> = vec![];

        let order_content = payload.content().content();
        for order_item in order_content {
            let book_id = order_item.book_id();
            let book_storage_info = BookBmc::get_quantity(app_context.deref(), book_id).await;
            if let Ok(book_storage_info) = book_storage_info {
                info!("book_storage_info is {:?}", book_storage_info);
                let quantity = book_storage_info.quantity().unwrap_or(0);
                if quantity < order_item.quantity() {
                    let diff = order_item.quantity() - quantity;
                    let order_item_ext = OrderItem::new(book_id, diff);
                    not_enough.push(order_item_ext);
                }
            }
        }
        let order_id = OrderId::new(payload.order_id());

        if !not_enough.is_empty() {
            storage_tx.send((order_id, not_enough)).await.unwrap()
        } else {
            delivery_tx.send(order_id).await.unwrap()
        }
    }
}