use sqlx::{Postgres, Transaction};
use lib_dto::book::BookStorageInfo;
use lib_dto::order::OrderItem;
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
            .bind(book_id)
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
            .bind(item.book_id())
            .bind(item.quantity())
            .fetch_optional(mm.pg_pool())
            .await?;

        Ok(())
    }

    pub async fn update_storage_tx(
        tx: &mut Transaction<'_, Postgres>,
        item: &OrderItem,
    ) -> Result<()> {
        sqlx::query(UPDATE_STORAGE)
            .bind(item.book_id())
            .bind(item.quantity())
            .fetch_optional(& mut **tx)
            .await?;

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