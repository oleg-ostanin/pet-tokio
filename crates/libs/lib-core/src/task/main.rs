use std::ops::Deref;
use std::sync::Arc;
use std::thread::sleep;
use crate::context::app_context::ModelManager;

use anyhow::Result;
use chrono::Duration;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tracing::info;
use crate::bmc::book_info::BookBmc;
use crate::notify::order::{notify_order, OrderPayload};

pub async fn main(app_context: Arc<ModelManager>) -> Result<()> {
    info!("Starting main task");

    let (order_sender, order_receiver) = tokio::sync::mpsc::channel(64);

    select! {
        _ = tokio::spawn(handle_order(order_receiver, app_context.clone())) => {}
        _ = tokio::spawn(notify_order(order_sender, app_context)) => {}
    }
    Ok(())
}

pub async fn handle_order(mut order_payload_rx: Receiver<OrderPayload>, app_context: Arc<ModelManager>) {
    loop {
        while let Some(payload) = order_payload_rx.recv().await {
            info!("received payload is {:?}", &payload);

            let order_content = payload.content().content();
            for order_item in order_content {
                let book_id = order_item.book_id();
                let book_storage_info = BookBmc::get_quantity(app_context.deref(), book_id).await;
                if let Ok(book_storage_info) = book_storage_info {
                    info!("book_storage_info is {:?}", book_storage_info);
                }
            }

        }
    }
}