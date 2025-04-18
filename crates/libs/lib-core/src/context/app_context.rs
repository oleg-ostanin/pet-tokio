use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use axum::body::Body;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use sqlx::postgres::PgPool;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::task::main_task::MainTaskRequest;

#[derive(Clone)]
pub struct ModelManager {
    main_tx: Sender<MainTaskRequest>,
    pg_pool: Arc<PgPool>,
    cache: Arc<RwLock<HashMap<String, String>>>,
    web_client: Client<HttpConnector, Body>,
    app_config: AppConfig,
    cancellation_token: CancellationToken,
    db_mutex: Arc<Mutex<()>>,
}

impl ModelManager {
    pub fn create(
        main_tx: Sender<MainTaskRequest>,
        app_config: AppConfig,
        pg_pool: Arc<PgPool>
    ) -> ModelManager {
        let cache = Arc::new(RwLock::new(HashMap::new()));

        let web_client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        let cancellation_token: CancellationToken = CancellationToken::new();

        let db_mutex = Arc::new(Mutex::new(()));

        ModelManager {
            main_tx,
            pg_pool,
            cache,
            web_client,
            app_config,
            cancellation_token,
            db_mutex,
        }
    }



    pub fn pg_pool(&self) -> &PgPool {
        self.pg_pool.deref()
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

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub fn db_mutex(&self) -> &Arc<Mutex<()>> {
        &self.db_mutex
    }

    pub fn main_tx(&self) -> Sender<MainTaskRequest> {
        self.main_tx.clone()
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub auth_url: Arc<String>,
    pub kafka_url: Arc<String>,
}

impl Debug for ModelManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ModeManager")
    }
}