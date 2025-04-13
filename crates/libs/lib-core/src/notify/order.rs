use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;
use sqlx::postgres::PgListener;
use tokio::select;
use tokio::sync::oneshot;
use tracing::{error, info, instrument};

use lib_dto::order::OrderStored;

use crate::context::app_context::ModelManager;
use crate::task::main_task::{TaskManager};
use crate::task::order::OrderRequest;

pub(crate) struct NotifyTask {}

impl NotifyTask {
    #[instrument(skip_all)]
    pub(crate) async fn start(app_context: Arc<ModelManager>) {
        info!("Starting notify task");

        let cancellation_token = app_context.cancellation_token();
        let jh = tokio::spawn(async move {
            select! {
                _ = handle_notify(app_context) => {}
                _ = cancellation_token.cancelled() => {
                    info!("Cancelled by cancellation token.")
                }
            }
        }).await;

        match jh {
            Ok(_) => {
                info!("Notify task completed.")
            }
            Err(e) => {
                error!("Error during notify task: {:#?}", e)
            }
        }
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
    app_context: Arc<ModelManager>
) -> Result<()> {
    info!("Starting handle_notify");
    let main_tx = app_context.main_tx();

    let channels = vec!["table_update"];

    let mut listener = PgListener::connect_with(app_context.pg_pool()).await.unwrap();
    listener.listen_all(channels).await.expect("error");
    info!("Getting order tx");
    let order_tx = TaskManager::order_sender(main_tx.clone()).await?;
    info!("Got order tx");

    loop {
        while let Some(notification) = listener.try_recv().await.expect("error") {
            let strr = notification.payload().to_owned();
            let payload: OrderPayload = serde_json::from_str::<OrderPayload>(&strr).unwrap();
            let order_stored: OrderStored = serde_json::from_str::<OrderStored>(&strr).unwrap();
            info!("the payload is {:#?}", &payload);
            info!("the order stored is {:#?}", &order_stored);

            match payload.action_type {
                ActionType::INSERT => {
                    let (result_tx, result_rx) = oneshot::channel();
                    order_tx.send(OrderRequest::ProcessOrder(order_stored, result_tx)).await
                        .expect("TODO: panic message");
                    let res = result_rx.await.expect("failed to receive result");
                    info!("Received order response: {:#?}", res);
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
