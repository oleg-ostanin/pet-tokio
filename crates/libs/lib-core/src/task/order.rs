use std::ops::Deref;
use std::sync::Arc;
use sqlx::types::Json;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::info;

use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderItemExt, OrderStored};

use crate::bmc::book_info::BookBmc;
use crate::bmc::storage::StorageBmc;
use crate::context::app_context::ModelManager;
use crate::task::main::MainTaskRequest;
use crate::task::storage::handle_requests;

#[derive(Debug)]
pub(crate) enum OrderRequest {
    Health(oneshot::Sender<OrderResponse>),
    ProcessOrder(OrderStored, oneshot::Sender<OrderResponse>),
}

#[derive(Debug)]
pub(crate) enum OrderResponse {
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
        tokio::spawn(handle_order_new(main_tx, rx));
        tx.clone()
    }
}

pub async fn handle_order_new(
    main_tx: Sender<MainTaskRequest>,
    mut order_rx: Receiver<OrderRequest>,
) {
    while let Some(order) = order_rx.recv().await {
        info!("received order is {:?}", &order);

        //storage_tx.send(order).await.unwrap();
    }
}

pub async fn handle_order(
    app_context: Arc<ModelManager>,
    mut order_rx: Receiver<OrderStored>,
    storage_tx: Sender<OrderStored>,
    delivery_tx: Sender<OrderStored>,
) {
    while let Some(order) = order_rx.recv().await {
        info!("received order is {:?}", &order);

        storage_tx.send(order).await.unwrap();
    }
}