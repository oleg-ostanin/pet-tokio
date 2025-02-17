use std::sync::Arc;
use std::thread::sleep;
use crate::context::app_context::ModelManager;

use anyhow::Result;
use chrono::Duration;
use tokio::select;
use tracing::info;
use crate::notify::order::notify;

pub async fn main(app_context: Arc<ModelManager>) -> Result<()> {
    info!("Starting main task");
    select! {
        _ = tokio::spawn(notify(app_context.pg_pool().clone())) => {}
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            info!("Cancelling by cancellation token.");
            app_context.cancellation_token().cancel()
        }
    }
    Ok(())
}