use std::fmt::Debug;
use serde::Deserialize;

use sqlx::postgres::PgListener;
use tracing::{info, instrument};
use lib_dto::order::OrderStored;

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use crate::task::main_task::{MainTaskRequest, TaskManager};
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
        //         error!("Error during task: {:#?}", e)
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
    let app_context = TaskManager::app_context(main_tx.clone()).await?;

    let channels = vec!["table_update"];

    let mut listener = PgListener::connect_with(app_context.pg_pool()).await.unwrap();
    listener.listen_all(channels).await.expect("error");
    let order_tx = TaskManager::order_sender(main_tx.clone()).await?;

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
