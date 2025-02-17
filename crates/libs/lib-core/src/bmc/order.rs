use chrono::prelude::*;
use tracing::log::info;
use lib_dto::order::{OrderForCreate, OrderId, OrderStatus, OrderStored};

use crate::context::app_context::ModelManager;
use crate::error::Result;

pub struct OrderBmc;
const INSERT_USER: &str = r#"
INSERT INTO order_info
(user_id, content, status, created_at, updated_at)
VALUES
($1, $2, $3, $4, $5)
RETURNING order_id;
"#;

const SELECT_BY_ID: &str = r#"
SELECT * FROM order_info WHERE order_id=$1;
"#;

impl OrderBmc {
    pub async fn create(
        mm: &ModelManager,
        order: OrderForCreate,
    ) -> Result<OrderId> {

        let json = sqlx::types::Json::from(order.content());
        let order_id = sqlx::query_as(INSERT_USER)
            .bind(&order.user_id())
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
        order_id: OrderId,
    ) -> Result<OrderStored> {
        info!("Trying to get order by id: {:?}", order_id);
        let order: OrderStored = sqlx::query_as(SELECT_BY_ID)
            .bind(&order_id.order_id())
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(order)
    }
}


