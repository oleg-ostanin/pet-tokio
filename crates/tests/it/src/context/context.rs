use core::net::SocketAddr;
use std::sync::Arc;

use axum::{body::Body, Error, http::{self, Request, StatusCode}};
use axum::response::Response;
use axum::routing::get;
use http_body_util::BodyExt;
use hyper::body::{Buf, Incoming};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
// for `collect`
use serde_json::json;
use sqlx::Pool;
use sqlx::postgres::PgPoolOptions;
use testcontainers::{clients, Container, images::postgres::Postgres};
use tokio::net::TcpListener;
use tokio_postgres::NoTls;
use tower::builder;
use lib_core::context::app_context::ModelManager;
use lib_web::app::web_app::create_app_context;
use lib_web::app::web_app::web_app;

use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;
//use lib_core::model::user::{UserForCreate, UserForLogin, UserForSignIn, UserStored};
use crate::context::sql::{CREATE_IDENTITY_TYPE, CREATE_USER_TABLE};
// for `call`, `oneshot`, and `ready`

#[derive(Debug, Clone)]
struct HeaderWrapper {
    key: String,
    value: String,
}

pub(crate) struct TestContext<> {
    docker: &'static clients::Cli,
    pg_container: &'static(Container<'static, Postgres>),
    client: Client<HttpConnector, Body>,
    socket_addr: SocketAddr,
    auth_token: Option<String>,
    headers: Vec<HeaderWrapper>,
}

impl TestContext {
    pub(crate) async fn new() -> Self {
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
                eprintln!("Connection error: {}", error);
            }
        });

        init_db(&pg_client).await;

        let db_url = format!("postgresql://postgres:root@localhost:{pg_port}/postgres");
        let pool = get_pool(&db_url).await;
        let app_context: Arc<ModelManager> = Arc::new(
            ModelManager::create(
                Arc::new(pg_client),
                Arc::new(pool),
            ));

        let app = web_app(app_context).await;

        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let socket_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client: Client<HttpConnector, Body> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        Self {
            docker,
            pg_container,
            client,
            socket_addr,
            auth_token: None,
            headers: Vec::new(),
        }
    }

    pub(crate) fn invalidate_token(&mut self) -> Option<String> {
        self.auth_token.take()
    }

    pub(crate) async fn get_books(&self) -> () {
        let addr = &self.socket_addr;

        let mut cookie = Cookie::new("AUTH_TOKEN", "token".to_string());

        let get_response = self.client
            .request(Request::builder()
                .method(http::Method::GET)
                .uri(format!("http://{addr}/get-books"))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .header("cookie", "auth-token=token".to_string())
                .header("cookie", "new-auth-token=new-token".to_string())

                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);
    }

    pub(crate) async fn get_user_response_by_id(&self, user_id: i64) -> Response<Incoming> {
        let path = format!("/get-by-id/{user_id}");

        self.client
            .request(self.get_builder(path))
            .await
            .unwrap()
    }

    // pub(crate) async fn get_user_by_id(&self, user_id: i64) -> Option<UserStored> {
    //     let response = self.get_user_response_by_id(user_id).await;
    //     Self::user_from_response(response).await
    // }
    //
    // async fn user_from_response(response: Response<Incoming>) -> Option<UserStored> {
    //     let body = response.collect().await.unwrap().aggregate();
    //     let user: UserStored = serde_json::from_reader(body.reader()).unwrap();
    //     Some(user)
    // }
    //
    // pub(crate) async fn create_user(&self, user_body: &UserForCreate) -> Response<Incoming> {
    //     let addr = &self.socket_addr;
    //
    //     self.client.request(Request::builder()
    //             .method(http::Method::POST)
    //             .uri(format!("http://{addr}/sign-up"))
    //             .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    //             .header("meta-cookie", "create_user")
    //             .body(Body::from(serde_json::to_string(&json!(user_body)).unwrap()))
    //             .unwrap())
    //         .await
    //         .unwrap()
    // }
    //
    // pub(crate) async fn sign_in_user(&mut self, user_body: UserForSignIn) -> Response<Incoming> {
    //     let addr = &self.socket_addr;
    //
    //     self.client
    //         .request(Request::builder()
    //             .method(http::Method::POST)
    //             .uri(format!("http://{addr}/sign-in"))
    //             .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    //             .body(Body::from(serde_json::to_string(&json!(user_body)).unwrap()))
    //             .unwrap())
    //         .await
    //         .unwrap()
    // }

    pub(crate) async fn get_auth_cookie(&mut self, sign_in_response: &Response<Incoming>) -> Option<String> {
        let sc = sign_in_response.headers().get("set-cookie");
        if let Some(hv) = sc {
            let hv_str = hv.to_str().unwrap().to_string();
            self.auth_token = Some(hv_str.clone());
            Some(hv_str.to_string())
        } else {
            None
        }
    }

    fn get_builder(&self, path: impl Into<String>) -> Request<Body> {
        let addr = &self.socket_addr;
        let path: String = path.into();

        let mut builder = Request::builder()
            .method(http::Method::GET)
            .uri(format!("http://{addr}{path}"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref());

        if let Some(auth_token) = self.auth_token.clone() {
            builder = builder.header("cookie", auth_token)
        }

        builder.body(Body::empty()).unwrap()
    }
}

async fn get_pool(db_url: &String) -> Pool<sqlx::Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .unwrap();

    sqlx::migrate!("../../../db/migrations").run(&pool).await.unwrap();

    pool
}

async fn init_db(pg_client: &tokio_postgres::Client) {
    pg_client.execute(CREATE_IDENTITY_TYPE, &[]).await.unwrap();
    pg_client.execute(CREATE_USER_TABLE, &[]).await.unwrap();
}