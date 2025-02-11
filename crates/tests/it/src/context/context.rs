use core::net::SocketAddr;
use std::fmt::Debug;
use std::sync::Arc;

use axum::{body::Body, Error, http::{self, Request, StatusCode}};
use axum::http::HeaderValue;
use axum::response::Response;
use axum::routing::get;
use dotenv::dotenv;
use http_body_util::BodyExt;
use hyper::body::{Buf, Incoming};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
// for `collect`
use serde_json::{json, Value};
use sqlx::{PgPool, Pool};
use sqlx::postgres::PgPoolOptions;
use testcontainers::{clients, Container, images::postgres::Postgres};
use tokio::net::TcpListener;
use tokio_postgres::NoTls;
use tower::builder;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info};
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{body_json, method, path};

use lib_core::context::app_context::{AppConfig, ModelManager};
use lib_dto::user::{AuthCode, UserForCreate, UserForSignIn};
use lib_web::app::auth_app::auth_app;
use lib_web::app::web_app::web_app;

//use lib_core::model::user::{UserForCreate, UserForLogin, UserForSignIn, UserStored};
use crate::context::sql::{CREATE_PHONE_TYPE, CREATE_USER_TABLE};

#[derive(Debug, Clone)]
struct HeaderWrapper {
    key: String,
    value: String,
}

pub(crate) struct TestContext<> {
    app_context: Arc<ModelManager>,
    docker: &'static clients::Cli,
    pg_container: &'static(Container<'static, Postgres>),
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

impl TestContext {
    pub(crate) async fn new(service_type: ServiceType) -> Self {
        dotenv().ok();
        tracing_subscriber::fmt()
            .without_time() // For early local development.
            .with_target(false)
            .init();
        info!("info");

        let mock_server = MockServer::start().await;

        let docker: &'static clients::Cli = Box::leak(Box::new(clients::Cli::default()));

        // Define a PostgreSQL container image
        let postgres_image = Postgres::default();

        let pg_container = docker.run(postgres_image);

        let pg_container: &'static(Container<Postgres>) = Box::leak(Box::new(pg_container));

        pg_container.start();

        // Get the PostgreSQL port
        let pg_port = pg_container.get_host_port_ipv4(5432);

        // Define the connection to the Postgress client
        let (pg_client, connection) = tokio_postgres::Config::new()
            .user("postgres")
            .password("postgres")
            .host("localhost")
            .port(pg_port)
            .dbname("postgres")
            .connect(NoTls)
            .await
            .unwrap();

        // Spawn connection
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                error!("Connection error: {}", error);
            }
        });

        //init_db(&pg_client).await;
        let mock_auth_url = mock_server.uri();
        info!("mock_auth_url: {:?}", &mock_auth_url);
        let app_config: AppConfig = AppConfig { auth_url: Arc::new(mock_auth_url)};

        let db_url = format!("postgresql://postgres:root@localhost:{pg_port}/postgres");
        let pool = get_pool(&db_url).await;
        let app_context: Arc<ModelManager> = Arc::new(
            ModelManager::create(
                app_config,
                Arc::new(pool.clone()),
            ));


        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let socket_addr = listener.local_addr().unwrap();

        let app = match service_type {
            ServiceType::Auth => auth_app(app_context.clone()).await,
            ServiceType::Web => web_app(app_context.clone()).await,
        };
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        Self {
            app_context,
            docker,
            pg_container,
            pool,
            client,
            mock_server,
            socket_addr,
            auth_token: None,
            headers: Vec::new(),
        }
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

    pub(crate) fn invalidate_token(&mut self) -> Option<String> {
        self.auth_token.take()
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

    pub fn pool(&self) -> &PgPool {
        &self.pool
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

async fn get_pool(db_url: &String) -> Pool<sqlx::Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .unwrap();

    sqlx::migrate!("../../../db/migrations-auth").run(&pool).await.unwrap();

    pool
}

async fn init_db(pg_client: &tokio_postgres::Client) {
    pg_client.execute(CREATE_PHONE_TYPE, &[]).await.unwrap();
    pg_client.execute(CREATE_USER_TABLE, &[]).await.unwrap();
}