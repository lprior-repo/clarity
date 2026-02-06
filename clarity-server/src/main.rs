use axum::{
  response::{Html, IntoResponse},
  routing::get,
  Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{self, filter::LevelFilter};

// Global allocator optimization: mimalloc provides 20-30% speedup
// over the default system allocator through better fragmentation
// reduction and thread-local caching strategies.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Embed CSS at compile time to avoid fragile runtime path dependencies
const CSS: &str = include_str!("../../clarity-client/assets/responsive.css");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Set up logging
  tracing_subscriber::fmt()
    .with_max_level(LevelFilter::INFO)
    .init();

  // Create a new Axum router with CSS serving
  let app = Router::new()
    .route("/", get(root))
    .route("/assets/responsive.css", get(serve_css));

  // Bind to the address
  let addr = SocketAddr::from(([127, 0, 0, 1], 4123));
  let listener = TcpListener::bind(addr).await?;

  println!("Server starting on http://{}", addr);

  // Start the server
  axum::serve(listener, app).await?;

  Ok(())
}

async fn root() -> Html<&'static str> {
  Html("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>Clarity Application</title><link rel=\"stylesheet\" href=\"/assets/responsive.css\"></head><body><div class=\"container\"><h1>Clarity Application</h1><p>Welcome to Clarity with responsive design!</p></div></body></html>")
}

/// Serve the responsive CSS file with proper content type
///
/// CSS is embedded at compile time using `include_str!()` to avoid
/// fragile runtime path dependencies.
async fn serve_css() -> impl IntoResponse {
  let headers = [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")];
  (headers, CSS).into_response()
}
