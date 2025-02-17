use std::sync::Arc;
use std::thread::sleep;
use crate::context::app_context::ModelManager;

use anyhow::Result;
use chrono::Duration;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tracing::info;
use crate::notify::order::{notify_order, OrderPayload};

pub async fn main(app_context: Arc<ModelManager>) -> Result<()> {
    info!("Starting main task");

    let (order_sender, order_receiver) = tokio::sync::mpsc::channel(64);

    select! {
        _ = tokio::spawn(handle_order(order_receiver, app_context.pg_pool().clone())) => {}
        _ = tokio::spawn(notify_order(order_sender, app_context.pg_pool().clone())) => {}
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            info!("Cancelling by cancellation token.");
            app_context.cancellation_token().cancel()
        }
    }
    Ok(())
}

pub async fn handle_order(mut order_payload_rx: Receiver<OrderPayload>, pool: sqlx::PgPool) {
    loop {
        while let Some(payload) = order_payload_rx.recv().await {
            info!("received payload is {:?}", payload);
        }
    }
}