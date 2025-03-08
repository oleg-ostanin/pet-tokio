use std::ops::Deref;
use std::sync::Arc;
use std::thread::sleep;
use crate::context::app_context::ModelManager;

use anyhow::Result;
use chrono::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::info;
use lib_dto::order::OrderStored;
use crate::bmc::book_info::BookBmc;
use crate::notify::order::{notify_order, NotifyTask};
use crate::task::delivery::{DeliveryRequest, DeliveryTask, handle_delivery};
use crate::task::order::{handle_order, OrderRequest, OrderTask};
use crate::task::storage::{handle_storage, StorageRequest, StorageTask};

pub(crate) enum MainTaskRequest {
    Health(oneshot::Sender<MainTaskResponse>),
    AppContext(oneshot::Sender<Arc<ModelManager>>),
    OrderSender(oneshot::Sender<Sender<OrderRequest>>),
    StorageSender(oneshot::Sender<Sender<StorageRequest>>),
    DeliverySender(oneshot::Sender<Sender<DeliveryRequest>>),
}

#[derive(Debug)]
pub(crate) enum MainTaskResponse {
    HealthOk,
}

#[derive(Clone)]
pub struct TaskManager {
    app_context: Arc<ModelManager>,
    tx: Sender<MainTaskRequest>,
    order_tx: Sender<OrderRequest>,
    storage_tx: Sender<StorageRequest>,
    delivery_tx: Sender<DeliveryRequest>,
}

impl TaskManager {
    pub async fn start(app_context: Arc<ModelManager>) -> Result<()> {
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        NotifyTask::start(tx.clone());
        let order_tx = OrderTask::start(tx.clone());
        let storage_tx = StorageTask::start(tx.clone());
        let delivery_tx = DeliveryTask::start(tx.clone());

        let main_task = TaskManager {
            app_context,
            tx: tx.clone(),
            order_tx,
            storage_tx,
            delivery_tx,
        };

        let jh = tokio::spawn(main_task.handle_requests(rx));
        jh.await.expect("TODO: panic message").expect("TODO: panic message");

        Ok(())
    }

    async fn handle_requests(
        self,
        mut rx: Receiver<MainTaskRequest>,
    ) -> Result<()> {
        while let Some(request) = rx.recv().await {
            self.match_requests(request).await;
        }
        Ok(())
    }

    async fn match_requests(&self, request: MainTaskRequest) {
        match request {
            MainTaskRequest::Health(_) => {}
            MainTaskRequest::AppContext(tx) => {
                tx.send(self.app_context.clone()); // todo handle result and below
            }
            MainTaskRequest::OrderSender(tx) => {
                tx.send(self.order_tx.clone()).expect("TODO: panic message");
            }
            MainTaskRequest::StorageSender(tx) => {
                tx.send(self.storage_tx.clone()).expect("TODO: panic message");
            }
            MainTaskRequest::DeliverySender(tx) => {
                tx.send(self.delivery_tx.clone()).expect("TODO: panic message");
            }
        }
    }

    pub(crate) async fn app_context(main_tx: Sender<MainTaskRequest>) -> Result<Arc<ModelManager>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::AppContext(tx)).await?;
        Ok(rx.await?)
    }

    pub(crate) async fn order_sender(main_tx: Sender<MainTaskRequest>) -> Result<Sender<OrderRequest>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::OrderSender(tx)).await?;
        Ok(rx.await?)
    }
}

pub async fn main(app_context: Arc<ModelManager>) -> Result<()> {
    info!("Starting main task");

    let (order_tx, order_rx) = tokio::sync::mpsc::channel(64);
    let (storage_tx, storage_rx) = tokio::sync::mpsc::channel(64);
    let (delivery_tx, delivery_rx) = tokio::sync::mpsc::channel(64);

    select! {
        _ = tokio::spawn(handle_order(app_context.clone(), order_rx, storage_tx, delivery_tx.clone())) => {}
        _ = tokio::spawn(handle_storage(app_context.clone(), storage_rx, delivery_tx)) => {}
        _ = tokio::spawn(handle_delivery(app_context.clone(), delivery_rx)) => {}
        _ = tokio::spawn(notify_order(app_context, order_tx)) => {}
    }
    Ok(())
}

