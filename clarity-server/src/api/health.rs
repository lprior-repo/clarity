//! Health check endpoint
//!
//! Provides a simple health check endpoint for monitoring and load balancers.

use axum::{
  response::{IntoResponse, Json},
  routing::get,
  Router,
};
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct HealthResponse {
  pub status: String,
  pub version: String,
}

/// Create a router for health check endpoints
#[must_use]
pub fn create_router() -> Router {
  Router::new().route("/health", get(health_check))
}

/// Health check handler
///
/// Returns a simple JSON response indicating the server is healthy.
async fn health_check() -> impl IntoResponse {
  Json(HealthResponse {
    status: "ok".to_string(),
    version: env!("CARGO_PKG_VERSION").to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use tower_service::Service;

  #[test]
  fn test_health_response_serialization() {
    let response = HealthResponse {
      status: "ok".to_string(),
      version: "1.0.0".to_string(),
    };

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("\"status\":\"ok\""));
    assert!(json_str.contains("\"version\":\"1.0.0\""));
  }

  #[tokio::test]
  async fn test_health_check_endpoint() {
    let app = create_router();

    let request = Request::builder()
      .uri("/health")
      .body(Body::empty())
      .unwrap();

    let response = app
      .oneshot(request)
      .await
      .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(health.status, "ok");
    assert!(!health.version.is_empty());
  }
}
