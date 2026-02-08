//! Session management API endpoints
//!
//! This module provides HTTP handlers for session CRUD operations.

use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Json},
  routing::{get, post},
  Router,
};
use clarity_core::session::{Session, SessionId, SessionKind, SessionState, Timestamp};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use super::beads::ApiState;

/// Create session request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
  pub kind: String,
  pub title: Option<String>,
  pub description: Option<String>,
}

/// Session summary for list views
#[derive(Serialize, ToSchema, Clone, Debug, PartialEq, Eq)]
pub struct SessionSummary {
  pub id: String,
  pub kind: String,
  pub state: String,
  pub title: Option<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

impl From<Session> for SessionSummary {
  fn from(session: Session) -> Self {
    Self {
      id: session.id.to_string(),
      kind: session.kind.to_string(),
      state: session.state.to_string(),
      title: session.title,
      created_at: session.created_at.as_secs(),
      updated_at: session.updated_at.as_secs(),
    }
  }
}

/// Response for listing sessions
#[derive(Serialize, ToSchema)]
pub struct ListSessionsResponse {
  pub sessions: Vec<SessionSummary>,
  pub total: usize,
}

/// Error response
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
  pub error: String,
}

/// Create a router for session endpoints
#[must_use]
pub fn create_router() -> Router<ApiState> {
  Router::new()
    .route("/api/sessions", get(list_sessions).post(create_session))
    .route("/api/sessions/:id", get(get_session))
}

/// List all sessions
///
/// # Errors
///
/// Returns an error response if internal server error occurs
#[utoipa::path(
  get,
  path = "/api/sessions",
  responses(
    (status = 200, description = "List of sessions", body = ListSessionsResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "sessions"
)]
async fn list_sessions(
  State(_state): State<ApiState>,
) -> Result<Json<ListSessionsResponse>, (StatusCode, Json<ErrorResponse>)> {
  // For now, return empty list - in real implementation, would query database
  Ok(Json(ListSessionsResponse {
    sessions: Vec::new(),
    total: 0,
  }))
}

/// Get a single session by ID
///
/// # Errors
///
/// Returns a 404 error if the session is not found
#[utoipa::path(
  get,
  path = "/api/sessions/{id}",
  params(
    ("id" = String, Path, description = "Session ID")
  ),
  responses(
    (status = 200, description = "Session details", body = Session),
    (status = 404, description = "Session not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "sessions"
)]
async fn get_session(
  Path(id): Path<String>,
  State(_state): State<ApiState>,
) -> Result<Json<Session>, (StatusCode, Json<ErrorResponse>)> {
  // Parse session ID
  let session_id = SessionId::new(id.clone()).map_err(|_| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!("Invalid session ID format: {id}"),
      }),
    )
  })?;

  // For now, return error - in real implementation, would query database
  Err((
    StatusCode::NOT_FOUND,
    Json(ErrorResponse {
      error: format!("Session {session_id} not found"),
    }),
  ))
}

/// Create a new session
///
/// # Errors
///
/// Returns a 400 error if request validation fails
/// Returns a 500 error if creation fails
#[utoipa::path(
  post,
  path = "/api/sessions",
  request_body = CreateSessionRequest,
  responses(
    (status = 201, description = "Session created", body = SessionSummary),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "sessions"
)]
async fn create_session(
  State(_state): State<ApiState>,
  Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionSummary>), (StatusCode, Json<ErrorResponse>)> {
  // Parse session kind
  let kind = match req.kind.to_lowercase().as_str() {
    "interview" => SessionKind::Interview,
    "analysis" => SessionKind::Analysis,
    "planning" => SessionKind::Planning,
    _ => {
      return Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!("Invalid session kind: {}", req.kind),
        }),
      ))
    }
  };

  // Generate UUID for session
  let id = uuid::Uuid::new_v4().to_string();

  // Create timestamp
  let timestamp = Timestamp::now().map_err(|_| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        error: "Failed to create timestamp".to_string(),
      }),
    )
  })?;

  // Create session
  let session_id = SessionId::new(id).map_err(|_| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        error: "Failed to create session ID".to_string(),
      }),
    )
  })?;

  let mut session = Session::new(session_id, kind, timestamp).map_err(|_| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        error: "Failed to create session".to_string(),
      }),
    )
  })?;

  session.title = req.title;
  session.description = req.description;

  let summary = SessionSummary::from(session);
  Ok((StatusCode::CREATED, Json(summary)))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_session_summary_from_session() {
    let id = SessionId::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
    let kind = SessionKind::Interview;
    let created_at = Timestamp::from_secs(1_234_567_890);

    let session = Session::new(id, kind, created_at).unwrap();
    let summary = SessionSummary::from(session.clone());

    assert_eq!(summary.id, session.id.to_string());
    assert_eq!(summary.kind, session.kind.to_string());
    assert_eq!(summary.state, session.state.to_string());
    assert_eq!(summary.created_at, session.created_at.as_secs());
    assert_eq!(summary.updated_at, session.updated_at.as_secs());
  }

  #[test]
  fn test_list_sessions_response_serialization() {
    let response = ListSessionsResponse {
      sessions: vec![],
      total: 0,
    };

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());
  }

  #[tokio::test]
  async fn test_list_sessions_empty() {
    let state = ApiState::new();
    let result = list_sessions(State(state)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.0.total, 0);
    assert!(response.0.sessions.is_empty());
  }

  #[tokio::test]
  async fn test_create_session_interview() {
    let state = ApiState::new();

    let req = CreateSessionRequest {
      kind: "interview".to_string(),
      title: Some("Test Interview".to_string()),
      description: Some("Test description".to_string()),
    };

    let result = create_session(State(state), Json(req)).await;
    assert!(result.is_ok());

    let (status, summary) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(summary.kind, "interview");
    assert_eq!(summary.title, Some("Test Interview".to_string()));
  }

  #[tokio::test]
  async fn test_create_session_invalid_kind() {
    let state = ApiState::new();

    let req = CreateSessionRequest {
      kind: "invalid".to_string(),
      title: None,
      description: None,
    };

    let result = create_session(State(state), Json(req)).await;
    assert!(result.is_err());

    let (status, error) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(error.0.error.contains("Invalid session kind"));
  }
}
