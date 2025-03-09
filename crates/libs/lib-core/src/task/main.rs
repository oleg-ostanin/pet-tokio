use std::ops::Deref;
use std::sync::Arc;
use std::thread::sleep;
use crate::context::app_context::ModelManager;

use anyhow::Result;
use chrono::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{error, info};
use lib_dto::order::OrderStored;
use crate::bmc::book_info::BookBmc;
use crate::notify::order::{notify_order, NotifyTask};
use crate::task::delivery::{DeliveryRequest, DeliveryTask, handle_delivery};
use crate::task::order::{handle_order, OrderRequest, OrderResponse, OrderTask};
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

        NotifyTask::start(tx.clone()).await;
        let order_tx = OrderTask::start(tx.clone());
        let storage_tx = StorageTask::start(tx.clone());
        let delivery_tx = DeliveryTask::start(tx.clone());

        let mut main_task = TaskManager {
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
        mut self,
        mut rx: Receiver<MainTaskRequest>,
    ) -> Result<()> {
        loop {
            if let Some(request) = rx.recv().await {
                info!("got MainTaskRequest");
                self.match_requests(request).await;
            } else {
                info!("got None");

            }
        }
        Ok(())
    }

    async fn match_requests(&mut self, request: MainTaskRequest) {
        info!("matching MainTaskRequest");
        match request {
            MainTaskRequest::Health(_) => {}
            MainTaskRequest::AppContext(tx) => {
                info!("got context request");
                tx.send(self.app_context.clone()); // todo handle result and below
                info!("answered context request");
            }
            MainTaskRequest::OrderSender(tx) => {
                // if let Err(e) = self.check_order_task().await {
                //     error!("Failed to check task: {:?}", e);
                //     self.order_tx = OrderTask::start(self.tx.clone());
                // }
                tx.send(self.order_tx.clone()).expect("should be ok");
            }
            MainTaskRequest::StorageSender(tx) => {
                // if let Err(e) = self.check_storage_task().await {
                //     error!("Failed to check task: {:?}", e);
                //     self.storage_tx = StorageTask::start(self.tx.clone());
                // }
                tx.send(self.storage_tx.clone()).expect("TODO: panic message");
            }
            MainTaskRequest::DeliverySender(tx) => {
                // if let Err(e) = self.check_delivery_task().await {
                //     error!("Failed to check task: {:?}", e);
                //     self.delivery_tx = DeliveryTask::start(self.tx.clone());
                // }
                tx.send(self.delivery_tx.clone()).expect("TODO: panic message");
            }
        }
    }

    async fn check_order_task(&mut self, ) -> Result<()> {
        let (health_tx, health_rx) = oneshot::channel();
        self.order_tx.send(OrderRequest::Health(health_tx)).await?;
        health_rx.await?;
        Ok(())
    }

    async fn check_storage_task(&mut self, ) -> Result<()> {
        let (health_tx, health_rx) = oneshot::channel();
        self.storage_tx.send(StorageRequest::Health(health_tx)).await?;
        health_rx.await?;
        Ok(())
    }

    async fn check_delivery_task(&mut self, ) -> Result<()> {
        let (health_tx, health_rx) = oneshot::channel();
        self.delivery_tx.send(DeliveryRequest::Health(health_tx)).await?;
        health_rx.await?;
        Ok(())
    }

    pub(crate) async fn app_context(main_tx: Sender<MainTaskRequest>) -> Result<Arc<ModelManager>> {
        let (tx, rx) = oneshot::channel();
        info!("sending context request");
        main_tx.send(MainTaskRequest::AppContext(tx)).await?;
        info!("send context request");
        Ok(rx.await?)
    }

    pub(crate) async fn order_sender(main_tx: Sender<MainTaskRequest>) -> Result<Sender<OrderRequest>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::OrderSender(tx)).await?;
        Ok(rx.await?)
    }

    pub(crate) async fn storage_sender(main_tx: Sender<MainTaskRequest>) -> Result<Sender<StorageRequest>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::StorageSender(tx)).await?;
        Ok(rx.await?)
    }

    pub(crate) async fn delivery_sender(main_tx: Sender<MainTaskRequest>) -> Result<Sender<DeliveryRequest>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::DeliverySender(tx)).await?;
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

