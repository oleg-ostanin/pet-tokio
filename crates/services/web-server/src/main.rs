use std::error::Error;
use std::sync::Arc;
use axum::routing::trace;
use dotenv::dotenv;
use opentelemetry::{global, KeyValue, runtime};
use opentelemetry::sdk::{Resource, trace as sdktrace, trace};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use sqlx::__rt::JoinHandle::Tokio;
use tokio::{select};
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;

use lib_core::context::app_context::ModelManager;
use lib_web::app::context::create_app_context;
use lib_web::app::web_app::web_app;

use axum::http::StatusCode;
use axum::Router;
use axum::routing::get;
use tracing::{error, event, Level, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    // tracing_subscriber::fmt()
    //     //.without_time() // For early local development.
    //     .with_target(false)
    //     .init();

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = init_trace().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("starting web server");
    let main_task_channel = tokio::sync::mpsc::channel(64);

    let app_context: Arc<ModelManager> = create_app_context(main_task_channel.0.clone()).await;

    let app = web_app(app_context.clone()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let cancellation_token: CancellationToken = app_context.cancellation_token();

    select! {
        _ = lib_core::task::main::TaskManager::start(main_task_channel, app_context) => {}
        _ = axum::serve(listener, app) => {}
        _ = cancellation_token.cancelled() => {
                info!("Cancelled by cancellation token.")
            }
    };

    Ok(())
}

fn init_trace() -> Result<sdktrace::Tracer, TraceError> {
    // Configure the Jaeger exporter
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "pet-tokio-web-server",
            )]))
        )
        .install_batch(runtime::Tokio)
}