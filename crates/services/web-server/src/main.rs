use std::error::Error;
use std::sync::Arc;

use dotenv::dotenv;
use tokio::select;
use tokio_util::sync::CancellationToken;

use tracing::info;

use lib_core::context::app_context::ModelManager;
use lib_web::app::context::create_app_context;
use lib_web::app::web_app::web_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        //.without_time() // For early local development.
        .with_target(false)
        .init();
    info!("starting web server");
    let app_context: Arc<ModelManager> = create_app_context().await;

    let app = web_app(app_context.clone()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let cancellation_token: CancellationToken = app_context.cancellation_token();

    select! {
        _ = lib_core::task::main::main(app_context) => {}
        _ = axum::serve(listener, app) => {}
        _ = cancellation_token.cancelled() => {
                info!("Cancelled by cancellation token.")
            }
    };

    Ok(())
}