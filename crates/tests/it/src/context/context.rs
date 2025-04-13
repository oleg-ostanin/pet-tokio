use core::net::SocketAddr;
use std::fmt::Debug;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::{body::Body, Error, http::{self, Request, StatusCode}};
use axum::http::HeaderValue;
use axum::response::Response;
use axum::routing::get;
use dotenv::dotenv;
use http_body_util::BodyExt;
use hyper::body::{Buf, Incoming};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use serde::Deserialize;
// for `collect`
use serde_json::{json, Value};
use sqlx::{PgPool, Pool};
use sqlx::postgres::PgPoolOptions;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers::core::ContainerPort;
// use testcontainers::{clients, Container, images::postgres::Postgres};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::kafka::Kafka;
use testcontainers_modules::postgres::Postgres;
use tokio::net::TcpListener;
use tokio::select;
use tokio_postgres::NoTls;
use tokio_util::sync::CancellationToken;
use tower::builder;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info, subscriber};
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::layer::SubscriberExt;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{body_json, method, path};

use lib_core::context::app_context::{AppConfig, ModelManager};
use lib_dto::user::{AuthCode, UserForCreate, UserForSignIn};
use lib_load::requests::user_context::UserContext;
use lib_load::utils::body_utils::message_and_detail;
use lib_utils::json::result;
use lib_utils::rpc::request;
use lib_web::app::auth_app::auth_app;
use lib_web::app::web_app::web_app;

#[derive(Debug, Clone)]
struct HeaderWrapper {
    key: String,
    value: String,
}

pub(crate) struct TestContext<> {
    app_context: Arc<ModelManager>,
    pg_container: ContainerAsync<Postgres>,
    kafka_container: ContainerAsync<Kafka>,
    pool: PgPool,
    pub(crate) client: Client<HttpConnector, Body>,
    pub(crate) mock_server: MockServer,
    pub(crate) socket_addr: SocketAddr,
    auth_token: Option<String>,
    headers: Vec<HeaderWrapper>,
}

pub(crate) enum ServiceType {
    Auth,
    Web,
}

static TRACING: OnceLock<()> = OnceLock::new();

impl TestContext {
    pub(crate) async fn new(service_type: ServiceType) -> Self {
        dotenv().ok();
        // for setting subscriber only once
        TRACING.get_or_init(|| {
            // tracing_subscriber::fmt()
            //     .without_time() // For early local development.
            //     .with_target(false)
            //     .init();

            let filter_layer = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("info"))
                .unwrap();

            let fmt_layer = fmt::layer()
                //.without_time() // For tests.
                .compact();

            let console_layer = console_subscriber::spawn();
            let subscriber = tracing_subscriber::Registry::default()
                .with(console_layer)
                .with(filter_layer)
                .with(fmt_layer);
            subscriber::set_global_default(subscriber).unwrap();

            info!("subscriber initialized");
            }
        );

        let mock_server = MockServer::start().await;

        let pg_container = Postgres::default()
            .with_db_name("postgres")
            .with_user("postgres")
            .with_password("root")
            .start().await.unwrap();

        // Get the PostgreSQL port
        let pg_port = pg_container.get_host_port_ipv4(5432).await.unwrap();

        // Define the connection to the Postgress client
        let (pg_client, connection) = tokio_postgres::Config::new()
            .user("postgres")
            .password("root")
            .host("localhost")
            .port(pg_port)
            .connect(NoTls)
            .await
            .unwrap();

        // Spawn connection
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                error!("Connection error: {}", error);
            }
        });

        let kafka_container = Kafka::default()
            .start()
            .await
            .unwrap();
        let kafka_port = kafka_container.get_host_port_ipv4(9093).await.unwrap();
        let kafka_url = format!{"localhost:{kafka_port}"};
        info!("kafka_url: {}", kafka_url);

        let mock_auth_url = mock_server.uri();
        info!("mock_auth_url: {:#?}", &mock_auth_url);
        let app_config: AppConfig = AppConfig {
            auth_url: Arc::new(mock_auth_url),
            kafka_url: Arc::new(kafka_url),
        };

        let db_url = format!("postgresql://postgres:root@localhost:{pg_port}/postgres");
        let pool = get_pool(&db_url).await;
        let main_task_channel = tokio::sync::mpsc::channel(64);
        let app_context: Arc<ModelManager> = Arc::new(
            ModelManager::create(
                main_task_channel.0.clone(),
                app_config,
                Arc::new(pool.clone()),
            ));


        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let socket_addr = listener.local_addr().unwrap();

        let app = match service_type {
            ServiceType::Auth => auth_app(app_context.clone()).await,
            ServiceType::Web => web_app(app_context.clone()).await,
        };

        let client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        let cancellation_token: CancellationToken = app_context.cancellation_token();
        let cancellation_token_cloned: CancellationToken = cancellation_token.clone();
        let app_context_cloned = app_context.clone();
        tokio::spawn(async move {
            select! {
                _ = lib_core::task::main_task::TaskManager::start(main_task_channel, app_context_cloned) => {}
                _ = axum::serve(listener, app) => {}
                _ = cancellation_token_cloned.cancelled() => {
                    info!("Cancelled by cancellation token.")
                }
            };
        });

        Self {
            app_context,
            //docker,
            pg_container,
            kafka_container,
            pool,
            client,
            mock_server,
            socket_addr,
            auth_token: None,
            headers: Vec::new(),
        }
    }

    pub(crate) fn user(&self, idx: usize) -> UserContext {
        UserContext::with_socket_address(idx, Some(self.socket_addr.to_string()))
    }

    pub(crate) async fn cancel(&self) {
        info!("Canceling test context.");
        //let _ = self.pg_container.stop().await;
        //let _ = self.kafka_container.stop().await;
        self.app_context.cancellation_token().cancel()
    }

    pub(crate) async fn mock_ok(&self, value: Value) {
        Mock::given(method("POST"))
            .and(path("/check-code"))
            .and(body_json(&value))
            .respond_with(ResponseTemplate::new(200))
            .mount(&self.mock_server)
            .await;
    }

    pub(crate) async fn mock_forbidden(&self, value: Value) {
        Mock::given(method("POST"))
            .and(path("/check-code"))
            .and(body_json(&value))
            .respond_with(ResponseTemplate::new(403))
            .mount(&self.mock_server)
            .await;
    }

    pub(crate) async fn create_user(&mut self, user_body: &UserForCreate) -> Response<Incoming> {
        self.post("/sign-up", json!(user_body)).await
    }

    pub(crate) async fn sign_in_user(&mut self, user_body: UserForSignIn) -> Response<Incoming> {
        self.post("/sign-in", json!(user_body)).await
    }

    pub(crate) async fn check_code(&mut self, user_body: AuthCode) -> Response<Incoming> {
        self.post("/check-code", json!(user_body)).await
    }

    pub(crate) async fn post(&mut self, path: impl Into<String>, body: Value) -> Response<Incoming> {
        let addr = &self.socket_addr;
        let path: String = path.into();

        let mut builder = Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{addr}{path}"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref());

        if let Some(auth_token) = self.auth_token.clone() {
            builder = builder.header("cookie", auth_token)
        }

        let builder = builder.body(Body::from(serde_json::to_string(&json!(body)).unwrap())).unwrap();
        let response = self.client
            .request(builder)
            .await
            .unwrap();

        let token = extract_token(&response);
        if let Some(token) = token {
            self.auth_token = Some(token);
        }

        response
    }

    pub fn app_context(&self) -> &Arc<ModelManager> {
        &self.app_context
    }
}

pub(crate) fn extract_token(response: &Response<Incoming>) -> Option<String> {
    let headers = response.headers();
    let value: Option<&HeaderValue> = headers.get("set-cookie");
    if let Some(value) = value {
        return Some(value.to_str().unwrap().to_string())
    }
    None
}

async fn get_pool(db_url: &str) -> Pool<sqlx::Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .unwrap();

    sqlx::migrate!("../../../db/migrations-auth").run(&pool).await.unwrap();

    pool
}