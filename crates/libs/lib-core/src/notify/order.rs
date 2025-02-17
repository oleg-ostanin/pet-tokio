use std::fmt::Debug;
use serde::Deserialize;

//use log::info;
use serde::de::DeserializeOwned;
use sqlx::error::Error;
use sqlx::postgres::PgListener;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::info;

use anyhow::Result;

#[derive(Deserialize, Debug)]
pub enum ActionType {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(Deserialize, Debug)]
pub struct OrderPayload {
    pub table: String,
    pub action_type: ActionType,
    pub order_id: i64,
    pub user_id: i64,
    pub content: sqlx::types::Json<OrderContent>,
    pub status: OrderStatus,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: DateTime<Utc>,
}

pub async fn start_listening<T: DeserializeOwned + Sized + Debug>(
    pool: &Pool<Postgres>,
    channels: Vec<&str>,
    call_back: impl Fn(T),
) -> Result<(), Error> {
    let mut listener = PgListener::connect_with(pool).await.unwrap();
    listener.listen_all(channels).await?;
    loop {
        while let Some(notification) = listener.try_recv().await? {
            info!(
                "Getting notification with payload: {:?} from channel {:?}",
                notification.payload(),
                notification.channel()
            );

            let strr = notification.payload().to_owned();
            let payload: T = serde_json::from_str::<T>(&strr).unwrap();
            info!("the payload is {:?}", payload);

            call_back(payload);
            return Ok(());
        }
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use lib_dto::order::{OrderContent, OrderStatus};

pub async fn notify(pool: sqlx::PgPool) -> Result<()>{

    let channels = vec!["table_update"];

    let hm: HashMap<String, String> = HashMap::new();
    let constants = Arc::new(RwLock::new(hm));

    let call_back = |payload: OrderPayload| {
        info!("payload: {:?}", &payload);

        match payload.action_type {
            ActionType::INSERT => {

            }
            ActionType::UPDATE => {

            }
            ActionType::DELETE => {

            }
        };
        info!(" ");
    };

    let mut listener = PgListener::connect_with(&pool).await.unwrap();
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
            info!("the payload is {:?}", payload);

            info!("payload: {:?}", &payload);

            match payload.action_type {
                ActionType::INSERT => {

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
