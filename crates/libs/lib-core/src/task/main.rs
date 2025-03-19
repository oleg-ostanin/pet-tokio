use std::ops::Deref;
use std::sync::Arc;
use std::thread::sleep;
use crate::context::app_context::ModelManager;

use anyhow::{bail, Result};
use chrono::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{error, info, instrument};
use lib_dto::order::OrderStored;
use crate::bmc::book_info::BookBmc;
use crate::notify::order::{NotifyTask};
use crate::task::delivery::{DeliveryRequest, DeliveryTask};
use crate::task::order::{OrderRequest, OrderResponse, OrderTask};
use crate::task::storage::{StorageRequest, StorageTask};

pub enum MainTaskRequest {
    Health(oneshot::Sender<MainTaskResponse>),
    AppContext(oneshot::Sender<Arc<ModelManager>>),
    OrderSender(oneshot::Sender<Sender<OrderRequest>>),
    StorageSender(oneshot::Sender<Sender<StorageRequest>>),
    DeliverySender(oneshot::Sender<Sender<DeliveryRequest>>),
}

#[derive(Debug)]
pub enum MainTaskResponse {
    HealthOk,
}

#[derive(Clone)]
pub struct TaskManager {
    app_context: Arc<ModelManager>,
    tx: Sender<MainTaskRequest>,
    order_tx: Option<Sender<OrderRequest>>,
    storage_tx: Option<Sender<StorageRequest>>,
    delivery_tx: Option<Sender<DeliveryRequest>>,
}

impl TaskManager {
    #[instrument(skip_all)]
    pub async fn start(
        main_task_channel: (Sender<MainTaskRequest>, Receiver<MainTaskRequest>),
        app_context: Arc<ModelManager>
    ) -> Result<()> {
        let (tx, rx) = main_task_channel;

        info!("creating NotifyTask");
        NotifyTask::start(tx.clone()).await;
        info!("creating OrderTask");

        let order_tx = OrderTask::start(tx.clone());
        let storage_tx = StorageTask::start(tx.clone());
        let delivery_tx = DeliveryTask::start(tx.clone());

        info!("creating MainTaskRequest");

        let mut main_task = TaskManager {
            app_context,
            tx: tx.clone(),
            order_tx: None,
            storage_tx: None,
            delivery_tx: None,
        };

        info!("spawning MainTaskRequest");

        let jh = tokio::spawn(main_task.handle_requests(rx));
        jh.await.expect("TODO: panic message").expect("TODO: panic message");

        tokio::time::sleep(core::time::Duration::from_secs(20)).await;
        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_requests(
        mut self,
        mut rx: Receiver<MainTaskRequest>,
    ) -> Result<()> {
        info!("starting MainTaskRequest");
        while let Some(request) = rx.recv().await {
            info!("got MainTaskRequest");
            self.match_requests(request).await;
        }
        Ok(())
    }

    #[instrument(skip_all)]
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
                if let Err(e) = self.check_order_task().await {
                    error!("Failed to check task: {:?}", e);
                    self.order_tx = Some(OrderTask::start(self.tx.clone()));
                }
                tx.send(self.order_tx.as_ref().expect("must be some").clone()).expect("should be ok");
            }
            MainTaskRequest::StorageSender(tx) => {
                if let Err(e) = self.check_storage_task().await {
                    error!("Failed to check task: {:?}", e);
                    self.storage_tx = Some(StorageTask::start(self.tx.clone()));
                }
                tx.send(self.storage_tx.as_ref().expect("must be some").clone()).expect("TODO: panic message");
            }
            MainTaskRequest::DeliverySender(tx) => {
                if let Err(e) = self.check_delivery_task().await {
                    error!("Failed to check task: {:?}", e);
                    self.delivery_tx = Some(DeliveryTask::start(self.tx.clone()));
                }
                tx.send(self.delivery_tx.as_ref().expect("must be some").clone()).expect("TODO: panic message");
            }
        }
    }

    #[instrument(skip_all)]
    async fn check_order_task(&mut self, ) -> Result<()> {
        let (health_tx, health_rx) = oneshot::channel();
        if let Some(tx) = self.order_tx.as_ref() {
            tx.send(OrderRequest::Health(health_tx)).await?;
            health_rx.await?;
            return Ok(());
        }
        bail!("order tx none")
    }

    #[instrument(skip_all)]
    async fn check_storage_task(&mut self, ) -> Result<()> {
        let (health_tx, health_rx) = oneshot::channel();
        if let Some(tx) = self.storage_tx.as_ref() {
            tx.send(StorageRequest::Health(health_tx)).await?;
            health_rx.await?;
            return Ok(());
        }
        bail!("storage tx none")
    }

    #[instrument(skip_all)]
    async fn check_delivery_task(&mut self, ) -> Result<()> {
        let (health_tx, health_rx) = oneshot::channel();
        if let Some(tx) = self.delivery_tx.as_ref() {
            tx.send(DeliveryRequest::Health(health_tx)).await?;
            health_rx.await?;
            return Ok(());
        }
        bail!("delivery tx none")
    }

    #[instrument(skip_all)]
    pub(crate) async fn app_context(main_tx: Sender<MainTaskRequest>) -> Result<Arc<ModelManager>> {
        let (tx, rx) = oneshot::channel();
        info!("sending context request");
        main_tx.send(MainTaskRequest::AppContext(tx)).await?;
        info!("send context request");
        Ok(rx.await?)
    }

    #[instrument(skip_all)]
    pub(crate) async fn order_sender(main_tx: Sender<MainTaskRequest>) -> Result<Sender<OrderRequest>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::OrderSender(tx)).await?;
        Ok(rx.await?)
    }

    #[instrument(skip_all)]
    pub(crate) async fn storage_sender(main_tx: Sender<MainTaskRequest>) -> Result<Sender<StorageRequest>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::StorageSender(tx)).await?;
        Ok(rx.await?)
    }

    #[instrument(skip_all)]
    pub(crate) async fn delivery_sender(main_tx: Sender<MainTaskRequest>) -> Result<Sender<DeliveryRequest>> {
        let (tx, rx) = oneshot::channel();
        main_tx.send(MainTaskRequest::DeliverySender(tx)).await?;
        Ok(rx.await?)
    }
}

