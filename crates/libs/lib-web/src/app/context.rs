#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use axum::{extract::{Json, State}, middleware, Router, routing::{get, post}};
use axum::http::StatusCode;
use java_properties::read;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tokio_postgres::{Client, NoTls};
use tower_cookies::{CookieManagerLayer, Cookies};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lib_core::context::app_context::{AppConfig, ModelManager};

use crate::handlers::login::login;
use crate::handlers::rpc::rpc;
use crate::middleware::mw_ctx::{mw_ctx_check, mw_ctx_create};
use crate::middleware::mw_req_stamp::mw_req_stamp_resolver;
use crate::middleware::mw_res_map::mw_response_map;

pub async fn create_app_context() -> Arc<ModelManager> {
    let db_url = read_db_url("local.properties");
    let client = get_client(&db_url).await;
    let pool = get_pool(&db_url).await;

    let app_config: AppConfig = AppConfig { auth_url: Arc::new("http://127.0.0.1:3001".to_string())};

    let app_context: Arc<ModelManager> = Arc::new(ModelManager::create(
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

    let db_migrations = read_db_migrations("local.properties");
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

fn read_db_migrations(path: &str) -> String {

    // Reading
    let f = File::open(path).unwrap();
    let map2 = read(BufReader::new(f)).unwrap();
    let db_migrations = map2.get("db.migrations").unwrap().to_string();
    db_migrations
}

