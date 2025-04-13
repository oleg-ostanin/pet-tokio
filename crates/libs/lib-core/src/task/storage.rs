use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{info, instrument};

use lib_dto::order::OrderStatus::ReadyToDeliver;
use lib_dto::order::OrderStored;

use crate::bmc::general::update_storage_and_order;
use crate::bmc::storage::UpdateType::Add;
use crate::context::app_context::ModelManager;
use crate::select_cancel;
use crate::task::storage::StorageResponse::HealthOk;

#[derive(Debug)]
pub enum StorageRequest {
    Health(oneshot::Sender<StorageResponse>),
    UpdateStorage(OrderStored, oneshot::Sender<StorageResponse>),
}

#[derive(Debug)]
pub enum StorageResponse {
    HealthOk,
    Updated,
    FailedToUpdate(OrderStored)
}

pub(crate) struct StorageTask {}

impl StorageTask {
    pub(crate) fn start(
        app_context: Arc<ModelManager>,
    ) -> Sender<StorageRequest> {
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        let cancellation_token = app_context.cancellation_token();
        tokio::spawn(async move {
            select! {
                _ = handle_storage_requests(app_context, rx) => {}
                _ = cancellation_token.cancelled() => {
                    info!("Cancelled by cancellation token.")
                }
            }
        });

        tx.clone()
    }
}

#[instrument(skip_all)]
pub async fn handle_storage_requests(
    app_context: Arc<ModelManager>,
    mut storage_rx: Receiver<StorageRequest>,
) -> Result<()> {
    let cancellation_token = app_context.cancellation_token();

    while let Some(request) = storage_rx.recv().await {
        info!("Got storage request: {:#?}", &request);

        let cancellation_token_cloned = cancellation_token.clone();
        let app_context_cloned = app_context.clone();
        match request {
            StorageRequest::Health(tx) => {
                tx.send(HealthOk).unwrap()
            }
            StorageRequest::UpdateStorage(order, tx) => {
                select_cancel!(handle_storage(app_context_cloned, order, tx), cancellation_token_cloned);
            }
        }
    }

    Ok(())
}

#[instrument(skip_all)]
pub async fn handle_storage(
    app_context: Arc<ModelManager>,
    order: OrderStored,
    response_tx: oneshot::Sender<StorageResponse>
) {
    info!("updating storage for order: {:#?}", &order);
    select! {
        _ = update_with_retry(app_context, order.clone()) => {
            response_tx.send(StorageResponse::Updated).unwrap();
        }
        _ = tokio::time::sleep(Duration::from_secs(3)) => {
            response_tx.send(StorageResponse::FailedToUpdate(order)).unwrap();
        }
    }
}

#[instrument(skip_all)]
async fn update_with_retry(
    app_context: Arc<ModelManager>,
    order: OrderStored,
)  {
    while let Err(e) = update_storage_and_order(app_context.clone(), &order, Add, ReadyToDeliver).await {
        info!("delivery retrying update storage for order is: {:#?} because of {:#?}", &order, e);
        //sleep(Duration::from_millis(700)).await;
    }
}

