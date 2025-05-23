use std::error::Error;
use std::sync::Arc;

use console_subscriber;
use dotenv::dotenv;
use opentelemetry::{global, KeyValue, runtime};
use opentelemetry::sdk::{Resource, trace as sdktrace, trace};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::trace::{FutureExt, TraceError};
use opentelemetry_otlp::WithExportConfig;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, filter, fmt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use lib_core::context::app_context::ModelManager;
use lib_web::app::context::create_app_context;
use lib_web::app::web_app::web_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    // tracing_subscriber::fmt()
    //     //.without_time() // For early local development.
    //     .with_target(false)
    //     .init();

    // log level filtering here
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let fmt_layer = fmt::layer()
        .with_filter(filter::LevelFilter::from_level(Level::INFO));
        //.compact();

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = init_trace().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let console_layer = console_subscriber::spawn();
    let subscriber = tracing_subscriber::Registry::default()
        .with(console_layer)
        //.with(filter_layer)
        .with(fmt_layer)
        .with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("starting web server");
    let (tx, rx) = tokio::sync::mpsc::channel(64);

    let app_context: Arc<ModelManager> = create_app_context(tx).await;

    let app = web_app(app_context.clone()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let cancellation_token: CancellationToken = app_context.cancellation_token();

    select! {
        _ = lib_core::task::main_task::TaskManager::start(rx, app_context) => {}
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