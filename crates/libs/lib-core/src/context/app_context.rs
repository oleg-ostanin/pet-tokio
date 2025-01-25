use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use sqlx::postgres::PgPool;


pub struct ModelManager {
    pool: Arc<PgPool>,
    cache: Arc<RwLock<HashMap<String, String>>>
}

impl ModelManager {
    pub fn create(pool: Arc<PgPool>) -> ModelManager {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        ModelManager {
            pool,
            cache,
        }
    }

    pub fn pg_pool(&self) -> &PgPool {
        self.pool.deref()
    }

    pub fn cache(&self) -> &Arc<RwLock<HashMap<String, String>>> {
        &self.cache
    }
}