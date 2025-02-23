use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;
use tracing::{error, info};
use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus, OrderStored};
use lib_dto::order::OrderStatus::{Delivered, ReadyToDeliver};
use crate::bmc::book_info::BookBmc;
use crate::bmc::general::update_storage_and_order;
use crate::bmc::order::OrderBmc;
use crate::bmc::storage::StorageBmc;
use crate::bmc::storage::UpdateType::{Add, Remove};
use crate::context::app_context::ModelManager;

pub async fn handle_delivery(
    app_context: Arc<ModelManager>,
    mut delivery_rx: Receiver<OrderStored>,
) {
    while let Some(order) = delivery_rx.recv().await {
        let order_id = order.order_id();
        info!("delivering order: {:?}", &order_id);

        while let Err(e) = update_storage_and_order(app_context.clone(), &order, Remove, Delivered).await {
            info!("delivery retrying update storage for order is: {:?}", &order);
            //sleep(Duration::from_millis(700)).await;
        }
    }
}