//! API types for Clarity client
//!
//! This module defines common types used for API communication.

use serde::{Deserialize, Serialize};

/// Bead summary for list views
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadSummary {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub priority: i16,
  pub bead_type: String,
  pub created_at: String,
}

/// Response for listing beads
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListBeadsResponse {
  pub beads: Vec<BeadSummary>,
  pub total: usize,
}

/// Create bead request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateBeadRequest {
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub priority: i16,
  pub bead_type: String,
}

/// Update bead request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateBeadRequest {
  pub title: Option<String>,
  pub description: Option<String>,
  pub status: Option<String>,
  pub priority: Option<i16>,
  pub bead_type: Option<String>,
}

/// Session summary for list views
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
  pub id: String,
  pub kind: String,
  pub state: String,
  pub title: Option<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// Response for listing sessions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListSessionsResponse {
  pub sessions: Vec<SessionSummary>,
  pub total: usize,
}

/// Create session request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSessionRequest {
  pub kind: String,
  pub title: Option<String>,
  pub description: Option<String>,
}

/// API error response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
  pub error: String,
}

/// Health check response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResponse {
  pub status: String,
  pub version: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bead_summary_serialization() {
    let bead = BeadSummary {
      id: "test-id".to_string(),
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&bead);
    assert!(json.is_ok());
  }

  #[test]
  fn test_list_beads_response_serialization() {
    let response = ListBeadsResponse {
      beads: vec![],
      total: 0,
    };

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());
  }

  #[test]
  fn test_create_bead_request_serialization() {
    let request = CreateBeadRequest {
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
    };

    let json = serde_json::to_string(&request);
    assert!(json.is_ok());
  }

  #[test]
  fn test_error_response_serialization() {
    let response = ErrorResponse {
      error: "Test error".to_string(),
    };

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());
  }

  #[test]
  fn test_health_response_serialization() {
    let response = HealthResponse {
      status: "ok".to_string(),
      version: "1.0.0".to_string(),
    };

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());
  }
}
