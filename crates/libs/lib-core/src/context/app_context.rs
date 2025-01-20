use std::ops::Deref;
use std::sync::Arc;

use sqlx::postgres::PgPool;
use tokio_postgres::Client;


pub struct ModelManager {
    client: Arc<Client>,
    pool: Arc<PgPool>,
}

impl ModelManager {
    pub fn create(client: Arc<Client>, pool: Arc<PgPool>) -> ModelManager {
        ModelManager {
            client,
            pool,
        }
    }

    pub fn client(&self) -> &Client {
        self.client.deref()
    }

    pub fn pool(&self) -> &PgPool {
        self.pool.deref()
    }
}