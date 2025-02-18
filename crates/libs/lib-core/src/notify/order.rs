use std::fmt::Debug;
use std::sync::Arc;
use serde::Deserialize;

use serde::de::DeserializeOwned;
use sqlx::error::Error;
use sqlx::postgres::PgListener;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::info;
use lib_dto::order::{OrderContent, OrderStatus};

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use crate::context::app_context::ModelManager;

#[derive(Deserialize, Debug)]
pub enum ActionType {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(Deserialize, Debug)]
pub struct OrderPayload {
    table: String,
    action_type: ActionType,
    order_id: i64,
    user_id: i64,
    content: sqlx::types::Json<OrderContent>,
    status: OrderStatus,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: DateTime<Utc>,
}

impl OrderPayload {
    pub fn content(&self) -> &sqlx::types::Json<OrderContent> {
        &self.content
    }
}

pub async fn notify_order(order_payload_tx: Sender<OrderPayload>, app_context: Arc<ModelManager>) -> Result<()>{
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
            info!("the payload is {:?}", &payload);

            match payload.action_type {
                ActionType::INSERT => {
                    _ = order_payload_tx.send(payload).await;
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
