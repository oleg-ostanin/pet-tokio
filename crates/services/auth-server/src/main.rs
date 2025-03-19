use std::error::Error;
use std::sync::Arc;

use dotenv::dotenv;
use tracing::info;

use lib_core::context::app_context::ModelManager;
use lib_web::app::auth_app::auth_app;
use lib_web::app::context::create_app_context;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();
    info!("Starting auth server");
    let main_task_channel = tokio::sync::mpsc::channel(64);
    let app_context: Arc<ModelManager> = create_app_context(main_task_channel.0).await;
    let app = auth_app(app_context).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    Ok(axum::serve(listener, app).await?)
}



