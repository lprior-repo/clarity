#![warn(clippy::all)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

mod api;

use api::{beads::ApiState, health, sessions};
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
#[allow(clippy::disallowed_methods)] // False positive on Ok(()) - not actually calling expect
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Set up logging
  tracing_subscriber::fmt()
    .with_max_level(LevelFilter::INFO)
    .init();

  // Create API state
  let api_state = ApiState::new();

  // Create a new Axum router with all endpoints
  let app = Router::new()
    // Static routes
    .route("/", get(root))
    .route("/assets/responsive.css", get(serve_css))
    // API routes
    .merge(health::create_router())
    .nest_service(api::beads::create_router().with_state(api_state.clone()))
    .nest_service(api::sessions::create_router().with_state(api_state));

  // Bind to the address
  let addr = SocketAddr::from(([127, 0, 0, 1], 4123));
  let listener = TcpListener::bind(addr).await?;

  println!("Server starting on http://{}", addr);
  println!("Available endpoints:");
  println!("  GET  http://{}/", addr);
  println!("  GET  http://{}/health", addr);
  println!("  GET  http://{}/api/beads", addr);
  println!("  POST http://{}/api/beads", addr);
  println!("  GET  http://{}/api/sessions", addr);
  println!("  POST http://{}/api/sessions", addr);

  // Start the server
  axum::serve(listener, app).await?;

  Ok(())
}

async fn root() -> Html<&'static str> {
  Html("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>Clarity Application</title><link rel=\"stylesheet\" href=\"/assets/responsive.css\"></head><body><div class=\"container\"><h1>Clarity Application</h1><p>Welcome to Clarity with responsive design!</p><div class=\"api-info\"><h2>API Endpoints</h2><ul><li><a href=\"/health\">GET /health</a> - Health check</li><li><a href=\"/api/beads\">GET /api/beads</a> - List beads</li><li>POST /api/beads - Create bead</li><li><a href=\"/api/sessions\">GET /api/sessions</a> - List sessions</li><li>POST /api/sessions - Create session</li></ul></div></div></body></html>")
}

/// Serve the responsive CSS file with proper content type
///
/// CSS is embedded at compile time using `include_str!()` to avoid
/// fragile runtime path dependencies.
async fn serve_css() -> impl IntoResponse {
  let headers = [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")];
  (headers, CSS).into_response()
}
