use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{self, filter::LevelFilter};

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
async fn serve_css() -> impl IntoResponse {
    let css_path = "../clarity-client/assets/responsive.css";

    match tokio::fs::read_to_string(css_path).await {
        Ok(css_content) => {
            let headers = [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")];
            (headers, css_content).into_response()
        }
        Err(_) => {
            let error_msg = "CSS file not found";
            let headers = [(axum::http::header::CONTENT_TYPE, "text/plain")];
            (axum::http::StatusCode::NOT_FOUND, headers, error_msg).into_response()
        }
    }
}
