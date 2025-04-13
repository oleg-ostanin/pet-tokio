#![allow(unused_imports)]
#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use axum::{extract::{Json, State}, middleware, Router, routing::{get, post}};
use axum::http::StatusCode;
use java_properties::read;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::mpsc::Sender;
use tokio_postgres::{Client, NoTls};
use tower_cookies::{CookieManagerLayer, Cookies};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lib_core::context::app_context::{AppConfig, ModelManager};
use lib_core::task::main_task::MainTaskRequest;

use crate::handlers::login::login;
use crate::handlers::rpc::rpc;
use crate::middleware::mw_ctx::{mw_ctx_check, mw_ctx_create};
use crate::middleware::mw_req_stamp::mw_req_stamp_resolver;
use crate::middleware::mw_res_map::mw_response_map;

pub async fn create_app_context(main_tx: Sender<MainTaskRequest>) -> Arc<ModelManager> {
    let db_url = read_db_url("local.properties"); // todo from env
    let pool = get_pool(&db_url).await;

    let kafka_url = env::var("KAFKA_URL").expect("must be ok");
    let app_config: AppConfig = AppConfig {
        auth_url: Arc::new("http://127.0.0.1:3001".to_string()), //todo from env
        kafka_url: Arc::new(kafka_url),
    };

    let app_context: Arc<ModelManager> = Arc::new(ModelManager::create(
        main_tx,
        app_config,
        Arc::new(pool),
    ));

    app_context
}

async fn get_client(db_url: &String) -> Client {
    //Unwrap because if we can't connect we must fail at once
    let (client, connection) =
        tokio_postgres::connect(db_url, NoTls).await.unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    client
}

async fn get_pool(db_url: &String) -> Pool<Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .unwrap();

    sqlx::migrate!("../../../db/migrations-auth").run(&pool).await.unwrap();

    pool
}

fn read_db_url(path: &str) -> String {

    // Reading
    let f = File::open(path).unwrap();
    let map2 = read(BufReader::new(f)).unwrap();
    let db_url = map2.get("db.url").unwrap().to_string();
    db_url
}

