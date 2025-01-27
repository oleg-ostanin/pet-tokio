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
use tokio_postgres::{Client, NoTls};
use sqlx::postgres::PgPoolOptions;
use tower_cookies::{CookieManagerLayer, Cookies};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lib_core::context::app_context::{AppConfig, ModelManager};
use crate::handlers::login::login;
use crate::handlers::rpc::rpc;
use crate::middleware::mw_ctx::{mw_ctx_check, mw_ctx_create};
use crate::middleware::mw_req_stamp::mw_req_stamp_resolver;
use crate::middleware::mw_res_map::mw_response_map;

pub async fn web_app(app_context: Arc<ModelManager>) -> Router {
    let routes_rpc = Router::new()
        .route("/rpc", post(rpc))
        .route_layer(middleware::from_fn(mw_ctx_check));

    Router::new()
        .nest("/api", routes_rpc)
        .route("/login", post(login))
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn_with_state(app_context.clone(), mw_ctx_create))
        .layer(CookieManagerLayer::new())
        .layer(middleware::from_fn(mw_req_stamp_resolver))
        .with_state(app_context)
}

