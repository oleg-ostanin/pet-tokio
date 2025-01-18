
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .init();

    println!("starts")
}