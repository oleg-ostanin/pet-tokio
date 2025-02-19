use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::info;
use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus};
use crate::bmc::book_info::BookBmc;
use crate::bmc::order::OrderBmc;
use crate::context::app_context::ModelManager;
use crate::notify::order::OrderPayload;

pub async fn handle_storage_item(
    app_context: Arc<ModelManager>,
    mut order_item_rx: Receiver<OrderItemExt>,
) {
    loop {
        while let Some(item) = order_item_rx.recv().await {
            info!("received item is {:?}", &item);
            let order_item = OrderItem::new(item.book_id(), item.quantity());
            BookBmc::update_storage(app_context.deref(), &order_item).await.unwrap();
            OrderBmc::update_status(app_context.deref(), item.order_id(), OrderStatus::ReadyToDeliver).await.unwrap();
            let order_stored = OrderBmc::get_by_id(app_context.deref(), item.order_id())
                .await.unwrap();
            info!("updated order is {:?}", &order_stored);

        }
    }
}