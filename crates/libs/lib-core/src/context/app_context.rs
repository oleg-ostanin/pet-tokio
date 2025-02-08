use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use axum::body::Body;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct ModelManager {
    pool: Arc<PgPool>,
    cache: Arc<RwLock<HashMap<String, String>>>,
    web_client: Client<HttpConnector, Body>,
    app_config: AppConfig,
}

impl ModelManager {
    pub fn create(app_config: AppConfig, pool: Arc<PgPool>) -> ModelManager {
        let cache = Arc::new(RwLock::new(HashMap::new()));

        let web_client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        ModelManager {
            pool,
            cache,
            web_client,
            app_config,
        }
    }

    pub fn pg_pool(&self) -> &PgPool {
        self.pool.deref()
    }

    pub fn cache(&self) -> &Arc<RwLock<HashMap<String, String>>> {
        &self.cache
    }

    pub fn web_client(&self) -> &Client<HttpConnector, Body> {
        &self.web_client
    }

    pub fn app_config(&self) -> &AppConfig {
        &self.app_config
    }
}

#[derive(Clone)]
pub struct AppConfig {
    pub auth_url: Arc<String>,
}