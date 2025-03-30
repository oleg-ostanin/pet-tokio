use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{info, instrument};
use anyhow::Result;
use lib_dto::order::OrderStored;

use crate::task::delivery::DeliveryRequest;
use crate::task::main_task::{MainTaskRequest, TaskManager};
use crate::task::storage::StorageRequest;

#[derive(Debug)]
pub enum OrderRequest {
    Health(oneshot::Sender<OrderResponse>),
    ProcessOrder(OrderStored, oneshot::Sender<OrderResponse>),
}

#[derive(Debug)]
pub enum OrderResponse {
    HealthOk,
    Processed,
    FailedToProcess(OrderStored)
}

pub(crate) struct OrderTask {}

impl OrderTask {
    pub(crate) fn start(
        main_tx: Sender<MainTaskRequest>,
    ) -> Sender<OrderRequest> {
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::spawn(handle_order(main_tx, rx));
        tx.clone()
    }
}

#[instrument(skip_all)]
pub async fn handle_order(
    main_tx: Sender<MainTaskRequest>,
    mut order_rx: Receiver<OrderRequest>,
) -> Result<()> {
    let storage_tx = TaskManager::storage_sender(main_tx.clone()).await?;
    let delivery_tx = TaskManager::delivery_sender(main_tx.clone()).await?;

    while let Some(order_request) = order_rx.recv().await {
        info!("received order is {:#?}", &order_request);

        match order_request {
            OrderRequest::Health(tx) => {
                tx.send(OrderResponse::HealthOk).expect("TODO: panic message");
            }
            OrderRequest::ProcessOrder(order, tx) => {
                let (storage_resp_tx, storage_resp_rx) = oneshot::channel();
                //todo deal with clone()
                storage_tx.send(StorageRequest::UpdateStorage(order.clone(), storage_resp_tx)).await.unwrap();
                let _storage_resp = storage_resp_rx.await.unwrap();

                let (delivery_resp_tx, delivery_resp_rx) = oneshot::channel();
                delivery_tx.send(DeliveryRequest::Deliver(order, delivery_resp_tx)).await.unwrap();
                let _delivery_resp = delivery_resp_rx.await.unwrap();

                tx.send(OrderResponse::Processed).unwrap();
            }
        }

        //storage_tx.send(order).await.unwrap();
    }

    Ok(())
}