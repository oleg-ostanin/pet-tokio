use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;
use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus};
use crate::bmc::book_info::BookBmc;
use crate::bmc::order::OrderBmc;
use crate::context::app_context::ModelManager;
use crate::notify::order::OrderPayload;

pub async fn handle_delivery(
    app_context: Arc<ModelManager>,
    mut delivery_rx: Receiver<OrderId>,
) {
    while let Some(order_id) = delivery_rx.recv().await {
        let order_id = order_id.order_id();
        info!("delivery received order_id is {:?}", &order_id);
        OrderBmc::update_status(app_context.deref(), order_id, OrderStatus::ReadyToDeliver).await.unwrap();
        let order_stored = OrderBmc::get_by_id(app_context.deref(), order_id)
            .await.unwrap();
        info!("updated order is {:?}", &order_stored);
    }
}