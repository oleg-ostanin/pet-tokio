use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;
use lib_dto::order::{OrderItem, OrderItemExt};
use crate::bmc::book_info::BookBmc;
use crate::context::app_context::ModelManager;
use crate::notify::order::OrderPayload;

pub async fn handle_order(
    app_context: Arc<ModelManager>,
    mut order_rx: Receiver<OrderPayload>,
    storage_tx: Sender<OrderItemExt>,
) {
    loop {
        while let Some(payload) = order_rx.recv().await {
            info!("received payload is {:?}", &payload);

            let order_content = payload.content().content();
            for order_item in order_content {
                let book_id = order_item.book_id();
                let book_storage_info = BookBmc::get_quantity(app_context.deref(), book_id).await;
                if let Ok(book_storage_info) = book_storage_info {
                    info!("book_storage_info is {:?}", book_storage_info);
                    let quantity = book_storage_info.quantity().unwrap_or(0);
                    if quantity < order_item.quantity() {
                        let diff = order_item.quantity() - quantity;
                        let order_item_ext = OrderItemExt::new(payload.order_id(), book_id, diff);
                        storage_tx.send(order_item_ext).await.unwrap()
                    }
                }
            }

        }
    }
}