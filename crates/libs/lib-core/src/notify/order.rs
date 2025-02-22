use std::fmt::Debug;
use std::sync::Arc;
use serde::Deserialize;

use serde::de::DeserializeOwned;
use sqlx::error::Error;
use sqlx::postgres::PgListener;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::info;
use lib_dto::order::{OrderContent, OrderId, OrderStatus, OrderStored};

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::Sender;
use crate::context::app_context::ModelManager;

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

pub async fn notify_order(
    app_context: Arc<ModelManager>,
    order_payload_tx: Sender<OrderStored>,
) -> Result<()>{
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
