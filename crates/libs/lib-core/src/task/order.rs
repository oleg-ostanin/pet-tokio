use std::ops::Deref;
use std::sync::Arc;
use sqlx::types::Json;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderItemExt};

use crate::bmc::book_info::BookBmc;
use crate::bmc::storage::StorageBmc;
use crate::context::app_context::ModelManager;
use crate::notify::order::OrderPayload;

enum CheckResult {
    Enough,
    NotEnough(Vec<OrderItem>),
}

pub async fn handle_order(
    app_context: Arc<ModelManager>,
    mut order_rx: Receiver<(OrderId, OrderContent)>,
    storage_tx: Sender<(OrderId, Vec<OrderItem>)>,
    delivery_tx: Sender<OrderId>,
) {
    while let Some((order_id, order_content)) = order_rx.recv().await {
        info!("received payload is {:?}", &order_content);

        match check_if_enough(app_context.clone(), order_content).await {
            CheckResult::NotEnough(items) => {
                storage_tx.send((order_id, items)).await.unwrap();
            }
            CheckResult::Enough => {
                delivery_tx.send(order_id).await.unwrap();
            }
        }
    }
}

async fn check_if_enough(
    app_context: Arc<ModelManager>,
    order_content: OrderContent,
) -> CheckResult {
    let mut not_enough: Vec<OrderItem> = vec![];

    let order_content = order_content.content();
    for order_item in order_content {
        let book_id = order_item.book_id();
        let book_storage_info = StorageBmc::get_quantity(app_context.deref(), book_id).await;
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

    if not_enough.is_empty() {
        CheckResult::Enough
    } else {
        CheckResult::NotEnough(not_enough)
    }
}