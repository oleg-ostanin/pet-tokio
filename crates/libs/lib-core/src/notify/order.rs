use std::fmt::Debug;
use std::sync::Arc;
use serde::Deserialize;

use serde::de::DeserializeOwned;
use sqlx::error::Error;
use sqlx::postgres::PgListener;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::{error, info, instrument};
use lib_dto::order::{OrderContent, OrderId, OrderStatus, OrderStored};

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use crate::context::app_context::ModelManager;
use crate::task::main::{MainTaskRequest, TaskManager};
use crate::task::order::OrderRequest;

pub(crate) struct NotifyTask {}

impl NotifyTask {
    #[instrument(skip_all)]
    pub(crate) async fn start(main_tx: Sender<MainTaskRequest>) {
        info!("Starting notify task");

        tokio::spawn(handle_notify(main_tx));
        // match jh.await {
        //     Ok(_) => {
        //         info!("Task completed.")
        //     }
        //     Err(e) => {
        //         error!("Error during task: {:?}", e)
        //     }
        // }
    }
}


#[derive(Deserialize, Debug)]
pub enum ActionType {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(Deserialize, Debug)]
struct OrderPayload {
    table: String,
    action_type: ActionType,
}

#[instrument(skip_all)]
pub async fn handle_notify(
    main_tx: Sender<MainTaskRequest>
) -> Result<()> {
    info!("Starting handle_notify");
    info!("before getting context");
    let app_context = TaskManager::app_context(main_tx.clone()).await?;
    info!("after getting context");

    let channels = vec!["table_update"];

    let mut listener = PgListener::connect_with(app_context.pg_pool()).await.unwrap();
    listener.listen_all(channels).await.expect("error");

    info!("before getting sender");
    let mut order_tx = TaskManager::order_sender(main_tx.clone()).await?;
    info!("after getting sender");

    loop {
        while let Some(notification) = listener.try_recv().await.expect("error") {
            info!(
                "Getting notification with payload: {:?} from channel {:?}",
                notification.payload(),
                notification.channel()
            );

            let strr = notification.payload().to_owned();
            let payload: OrderPayload = serde_json::from_str::<OrderPayload>(&strr).unwrap();
            let order_stored: OrderStored = serde_json::from_str::<OrderStored>(&strr).unwrap();
            info!("the payload is {:?}", &payload);
            info!("the order stored is {:?}", &order_stored);

            match payload.action_type {
                ActionType::INSERT => {
                    let (result_tx, result_rx) = oneshot::channel();
                    order_tx.send(OrderRequest::ProcessOrder(order_stored, result_tx)).await
                        .expect("TODO: panic message");
                    let res = result_rx.await.expect("failed to receive result");
                    info!("Received order response: {:?}", res);
                }
                ActionType::UPDATE => {

                }
                ActionType::DELETE => {

                }
            };
            info!(" ");
        }
    }
}


pub async fn notify_order(
    app_context: Arc<ModelManager>,
    order_payload_tx: Sender<OrderStored>,
) -> Result<()> {
    let channels = vec!["table_update"];

    let mut listener = PgListener::connect_with(app_context.pg_pool()).await.unwrap();
    listener.listen_all(channels).await?;
    loop {
        while let Some(notification) = listener.try_recv().await? {
            info!(
                "Getting notification with payload: {:?} from channel {:?}",
                notification.payload(),
                notification.channel()
            );

            let strr = notification.payload().to_owned();
            let payload: OrderPayload = serde_json::from_str::<OrderPayload>(&strr).unwrap();
            let order_stored: OrderStored = serde_json::from_str::<OrderStored>(&strr).unwrap();
            info!("the payload is {:?}", &payload);
            info!("the order stored is {:?}", &order_stored);

            match payload.action_type {
                ActionType::INSERT => {
                    _ = order_payload_tx.send(order_stored).await;
                }
                ActionType::UPDATE => {

                }
                ActionType::DELETE => {

                }
            };
            info!(" ");
        }
    }
}
