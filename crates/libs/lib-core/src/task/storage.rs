use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{info, instrument};

use anyhow::Result;

use lib_dto::order::OrderStored;
use lib_dto::order::OrderStatus::ReadyToDeliver;
use crate::bmc::general::update_storage_and_order;
use crate::bmc::storage::UpdateType::Add;
use crate::context::app_context::ModelManager;
use crate::task::main_task::{MainTaskRequest, TaskManager};
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
        tokio::spawn(handle_storage_requests(app_context, rx));
        tx.clone()
    }
}

#[instrument(skip_all)]
pub async fn handle_storage_requests(
    app_context: Arc<ModelManager>,
    mut storage_rx: Receiver<StorageRequest>,
) -> Result<()> {
    while let Some(request) = storage_rx.recv().await {
        info!("Got storage request: {:#?}", &request);

        match request {
            StorageRequest::Health(tx) => {
                tx.send(HealthOk).unwrap()
            }
            StorageRequest::UpdateStorage(order, tx) => {
                tokio::spawn(handle_storage(app_context.clone(), order, tx)).await.unwrap()
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

