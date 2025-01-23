use std::ops::Deref;
use std::sync::Arc;

use sqlx::postgres::PgPool;


pub struct ModelManager {
    pool: Arc<PgPool>,
}

impl ModelManager {
    pub fn create(pool: Arc<PgPool>) -> ModelManager {
        ModelManager {
            pool,
        }
    }

    pub fn pg_pool(&self) -> &PgPool {
        self.pool.deref()
    }
}