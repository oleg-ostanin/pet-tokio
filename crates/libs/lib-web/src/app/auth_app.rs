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
use crate::handlers::auth::{check_code, sign_in, sign_up};
use crate::handlers::login::login;
use crate::middleware::mw_req_stamp::mw_req_stamp_resolver;
use crate::middleware::mw_res_map::mw_response_map;

pub async fn auth_app(app_context: Arc<ModelManager>) -> Router {
    Router::new()
        .route("/sign-up", post(sign_up))
        .route("/sign-in", post(sign_in))
        .route("/check-code", post(check_code))
        .with_state(app_context)
}

