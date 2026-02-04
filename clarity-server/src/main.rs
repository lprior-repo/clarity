use axum::response::Html;
use axum::routing::get;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{self, filter::LevelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // Create a new Axum router
    let app = axum::Router::new()
        .route("/", get(root));

    // Bind to the address
    let addr = SocketAddr::from(([127, 0, 0, 1], 4123));
    let listener = TcpListener::bind(addr).await?;
    
    println!("Server starting on http://{}", addr);
    
    // Start the server
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> Html<&'static str> {
    Html("<h1>Clarity Application</h1><p>Welcome to the modern fullstack Dioxus application!</p>")
}