use std::collections::HashMap;
use std::sync::Arc;
use sqlx::{Postgres, Transaction};
use tracing::{info, instrument};
use lib_dto::book::{BookInfo, BookList, BookStorageInfo};
use lib_dto::order::{OrderItem, OrderStatus, OrderStored};
use crate::bmc::order::OrderBmc;
use crate::context::app_context::ModelManager;
use crate::error::Result;

pub struct StorageBmc;

const SELECT_JOIN_STORAGE: &str = r#"
SELECT
    bi.id,
    bs.quantity
FROM
    book_info as bi
FULL OUTER JOIN book_storage as bs
    ON bi.id = bs.book_id
WHERE bi.id=ANY($1)
"#;

const UPDATE_STORAGE: &str = r#"
INSERT INTO book_storage (book_id, quantity) values ($1, $2)
ON CONFLICT (book_id) DO UPDATE SET quantity = $2;
"#;

const TX_ISOLATION_LEVEL: &str = r#"
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
"#;

const CLEANUP_STORAGE: &str = r#"
TRUNCATE book_storage CASCADE;
"#;

pub(crate) enum UpdateType {
    Add,
    Remove,
}

impl StorageBmc {
    pub async fn get_quantity(
        mm: &ModelManager,
        book_id: i64,
    ) -> Result<BookStorageInfo> {
        let book_storage: BookStorageInfo = sqlx::query_as(SELECT_JOIN_STORAGE)
            .bind(&book_id)
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(book_storage)
    }

    pub async fn get_quantity_tx(
        tx: &mut Transaction<'_, Postgres>,
        book_ids: Vec<i64>,
    ) -> Result<Vec<BookStorageInfo>> {
        let book_storage: Vec<BookStorageInfo> = sqlx::query_as(SELECT_JOIN_STORAGE)
            .bind(&book_ids)
            .fetch_all(& mut **tx)
            .await?;

        Ok(book_storage)
    }

    pub async fn update_storage(
        mm: &ModelManager,
        item: &OrderItem,
    ) -> Result<()> {
        sqlx::query(UPDATE_STORAGE)
            .bind(&item.book_id())
            .bind(&item.quantity())
            .fetch_optional(mm.pg_pool())
            .await?;

        Ok(())
    }

    pub async fn update_storage_tx(
        tx: &mut Transaction<'_, Postgres>,
        item: &OrderItem,
    ) -> Result<()> {
        sqlx::query(UPDATE_STORAGE)
            .bind(&item.book_id())
            .bind(&item.quantity())
            .fetch_optional(& mut **tx)
            .await?;

        Ok(())
    }

    pub async fn tx_isolation_level(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<()> {
        sqlx::query(TX_ISOLATION_LEVEL)
            .fetch_optional(& mut **tx)
            .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub(crate) async fn update_storage_and_order(
        app_context: Arc<ModelManager>,
        order: &OrderStored,
        update_type: UpdateType,
        new_status: OrderStatus,
    ) -> Result<()> {
        let mut book_ids = Vec::with_capacity(order.content().len());
        for order_item in order.content() {
            let book_id = order_item.book_id();
            book_ids.push(book_id);
        }

        //let _guard = app_context.db_mutex().lock().await;

        let mut tx = app_context.pg_pool()
            .begin()
            .await?;

        Self::tx_isolation_level(&mut tx).await?;

        let book_storage_infos = StorageBmc::get_quantity_tx(&mut tx, book_ids).await?;
        info!("book_storage_info: {:?}", book_storage_infos);
        let map: HashMap<i64, i64> = book_storage_infos
            .into_iter()
            .map(|info| (info.id(), info.quantity().unwrap_or(0)))
            .collect();

        for order_item in order.content() {
            let old_quantity = map.get(&order_item.book_id()).unwrap_or(&0i64);
            let new_quantity = match update_type {
                UpdateType::Add => {old_quantity + order_item.quantity()}
                UpdateType::Remove => {old_quantity - order_item.quantity()}
            } ;
            let new_order_item = OrderItem::new(order_item.book_id(), new_quantity);
            StorageBmc::update_storage_tx(&mut tx, &new_order_item).await?;
        }

        OrderBmc::update_status_tx(&mut tx, order.order_id(), new_status).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn cleanup_storage(
        mm: &ModelManager,
    ) -> Result<()> {
        sqlx::query(CLEANUP_STORAGE)
            .fetch_optional(mm.pg_pool())
            .await?;

        Ok(())
    }
}