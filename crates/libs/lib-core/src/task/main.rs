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
use crate::task::delivery::handle_delivery;
use crate::task::order::handle_order;
use crate::task::storage::handle_storage;

pub async fn main(app_context: Arc<ModelManager>) -> Result<()> {
    info!("Starting main task");

    let (order_tx, order_rx) = tokio::sync::mpsc::channel(64);
    let (storage_tx, storage_rx) = tokio::sync::mpsc::channel(64);
    let (delivery_tx, delivery_rx) = tokio::sync::mpsc::channel(64);

    select! {
        _ = tokio::spawn(handle_order(app_context.clone(), order_rx, storage_tx, delivery_tx.clone())) => {}
        _ = tokio::spawn(handle_storage(app_context.clone(), storage_rx, delivery_tx)) => {}
        _ = tokio::spawn(handle_delivery(app_context.clone(), delivery_rx)) => {}
        _ = tokio::spawn(notify_order(order_tx, app_context)) => {}
    }
    Ok(())
}

