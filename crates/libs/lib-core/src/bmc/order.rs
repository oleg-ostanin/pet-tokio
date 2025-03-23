use chrono::prelude::*;
use sqlx::{Postgres, Transaction};
use tracing::log::info;
use lib_dto::order::{OrderForCreate, OrderId, OrderStatus, OrderStored};

use crate::context::app_context::ModelManager;
use crate::error::Result;

pub struct OrderBmc;
const INSERT_ORDER: &str = r#"
INSERT INTO order_info
(user_id, content, status, created_at, updated_at)
VALUES
($1, $2, $3, $4, $5)
RETURNING order_id;
"#;

const SELECT_BY_ID: &str = r#"
SELECT * FROM order_info WHERE order_id=$1;
"#;

const UPDATE_STATUS: &str = r#"
UPDATE order_info
SET status = $1
WHERE order_id = $2
RETURNING order_id;
"#;

const CLEANUP_ORDERS: &str = r#"
TRUNCATE order_info CASCADE;
"#;

impl OrderBmc {
    pub async fn create(
        mm: &ModelManager,
        order: OrderForCreate,
    ) -> Result<OrderId> {

        let json = sqlx::types::Json::from(order.content());
        let order_id = sqlx::query_as(INSERT_ORDER)
            .bind(order.user_id())
            .bind(json)
            .bind(OrderStatus::New)
            .bind(Utc::now())
            .bind(Utc::now())
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(order_id)
    }

    pub async fn get_by_id(
        mm: &ModelManager,
        order_id: i64,
    ) -> Result<OrderStored> {
        info!("Trying to get order by id: {:#?}", order_id);
        let order: OrderStored = sqlx::query_as(SELECT_BY_ID)
            .bind(order_id)
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(order)
    }

    pub async fn update_status(
        mm: &ModelManager,
        order_id: i64,
        order_status: OrderStatus,
    ) -> Result<()> {
        info!("Trying to update order with id: {:#?}, new status: {:#?}", &order_id, &order_status);
        sqlx::query_as(UPDATE_STATUS)
            .bind(&order_status)
            .bind(order_id)
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(())
    }

    pub async fn update_status_tx(
        tx: &mut Transaction<'_, Postgres>,
        order_id: i64,
        order_status: OrderStatus,
    ) -> Result<()> {
        info!("Trying to update order with id: {:#?}, new status: {:#?}", &order_id, &order_status);
        sqlx::query_as(UPDATE_STATUS)
            .bind(&order_status)
            .bind(order_id)
            .fetch_one(&mut **tx)
            .await?;

        Ok(())
    }

    pub async fn cleanup_orders(
        mm: &ModelManager,
    ) -> Result<()> {
        sqlx::query(CLEANUP_ORDERS)
            .fetch_optional(mm.pg_pool())
            .await?;

        Ok(())
    }
}


