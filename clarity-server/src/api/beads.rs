//! Bead management API endpoints
//!
//! This module provides HTTP handlers for bead CRUD operations.

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{IntoResponse, Json},
  routing::{get, post},
  Router,
};
use clarity_core::db::models::{Bead, BeadId, BeadPriority, BeadStatus, BeadType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// API state shared across handlers
#[derive(Clone)]
pub struct ApiState {
  // In a real implementation, this would hold a database connection pool
  // For now, we'll use in-memory storage
  pub beads: Arc<tokio::sync::RwLock<Vec<Bead>>>,
}

impl ApiState {
  /// Create a new API state
  #[must_use]
  pub fn new() -> Self {
    Self {
      beads: Arc::new(tokio::sync::RwLock::new(Vec::new())),
    }
  }
}

impl Default for ApiState {
  fn default() -> Self {
    Self::new()
  }
}

/// Query parameters for listing beads
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListBeadQuery {
  /// Filter by status
  pub status: Option<String>,
  /// Filter by type
  pub bead_type: Option<String>,
  /// Filter by priority
  pub priority: Option<i16>,
  /// Search query for title/description
  pub search: Option<String>,
}

/// Response for listing beads
#[derive(Serialize, ToSchema)]
pub struct ListBeadsResponse {
  pub beads: Vec<BeadSummary>,
  pub total: usize,
}

/// Bead summary for list views
#[derive(Serialize, ToSchema, Clone, Debug, PartialEq, Eq)]
pub struct BeadSummary {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub priority: i16,
  pub bead_type: String,
  pub created_at: String,
}

impl From<Bead> for BeadSummary {
  fn from(bead: Bead) -> Self {
    Self {
      id: bead.id.to_string(),
      title: bead.title,
      description: bead.description,
      status: bead.status.as_str().to_string(),
      priority: bead.priority.0,
      bead_type: bead.bead_type.as_str().to_string(),
      created_at: bead.created_at.to_rfc3339(),
    }
  }
}

/// Create bead request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBeadRequest {
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub priority: i16,
  pub bead_type: String,
}

/// Update bead request
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBeadRequest {
  pub title: Option<String>,
  pub description: Option<String>,
  pub status: Option<String>,
  pub priority: Option<i16>,
  pub bead_type: Option<String>,
}

/// Error response
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
  pub error: String,
}

/// Create a router for bead endpoints
#[must_use]
pub fn create_router() -> Router<ApiState> {
  Router::new()
    .route("/api/beads", get(list_beads).post(create_bead))
    .route(
      "/api/beads/:id",
      get(get_bead).put(update_bead).delete(delete_bead),
    )
}

/// List beads with optional filtering
///
/// # Errors
///
/// Returns an error response if:
/// - Query parameters are invalid
/// - Internal server error occurs
#[utoipa::path(
  get,
  path = "/api/beads",
  params(
    ("status" = Option<String>, Query, description = "Filter by status"),
    ("bead_type" = Option<String>, Query, description = "Filter by bead type"),
    ("priority" = Option<i16>, Query, description = "Filter by priority"),
    ("search" = Option<String>, Query, description = "Search in title/description")
  ),
  responses(
    (status = 200, description = "List of beads", body = ListBeadsResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "beads"
)]
async fn list_beads(
  Query(params): Query<ListBeadQuery>,
  State(state): State<ApiState>,
) -> Result<Json<ListBeadsResponse>, (StatusCode, Json<ErrorResponse>)> {
  let beads = state
    .beads
    .read()
    .await
    .iter()
    .filter(|bead| {
      if let Some(ref status_str) = params.status {
        if bead.status.as_str() != status_str {
          return false;
        }
      }
      if let Some(ref type_str) = params.bead_type {
        if bead.bead_type.as_str() != type_str {
          return false;
        }
      }
      if let Some(priority_val) = params.priority {
        if bead.priority.0 != priority_val {
          return false;
        }
      }
      if let Some(ref search_query) = params.search {
        let title_matches = bead
          .title
          .to_lowercase()
          .contains(&search_query.to_lowercase());
        let desc_matches = bead
          .description
          .as_ref()
          .map(|d| d.to_lowercase().contains(&search_query.to_lowercase()))
          .unwrap_or(false);
        if !title_matches && !desc_matches {
          return false;
        }
      }
      true
    })
    .cloned()
    .collect::<Vec<_>>();

  let summaries = beads.into_iter().map(BeadSummary::from).collect();

  Ok(Json(ListBeadsResponse {
    total: summaries.len(),
    beads: summaries,
  }))
}

/// Get a single bead by ID
///
/// # Errors
///
/// Returns a 404 error if the bead is not found
#[utoipa::path(
  get,
  path = "/api/beads/{id}",
  params(
    ("id" = String, Path, description = "Bead ID")
  ),
  responses(
    (status = 200, description = "Bead details", body = Bead),
    (status = 404, description = "Bead not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "beads"
)]
async fn get_bead(
  Path(id): Path<String>,
  State(state): State<ApiState>,
) -> Result<Json<Bead>, (StatusCode, Json<ErrorResponse>)> {
  let beads = state.beads.read().await;

  let bead = beads
    .iter()
    .find(|b| b.id.to_string() == id)
    .cloned()
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          error: format!("Bead {id} not found"),
        }),
      )
    })?;

  Ok(Json(bead))
}

/// Create a new bead
///
/// # Errors
///
/// Returns a 400 error if request validation fails
/// Returns a 500 error if creation fails
#[utoipa::path(
  post,
  path = "/api/beads",
  request_body = CreateBeadRequest,
  responses(
    (status = 201, description = "Bead created", body = BeadSummary),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "beads"
)]
async fn create_bead(
  State(state): State<ApiState>,
  Json(req): Json<CreateBeadRequest>,
) -> Result<(StatusCode, Json<BeadSummary>), (StatusCode, Json<ErrorResponse>)> {
  // Validate and parse status
  let status = BeadStatus::from_str(&req.status).map_err(|e| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!("Invalid status: {e}"),
      }),
    )
  })?;

  // Validate and parse bead type
  let bead_type = BeadType::from_str(&req.bead_type).map_err(|e| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!("Invalid bead type: {e}"),
      }),
    )
  })?;

  // Validate and parse priority
  let priority = BeadPriority::new(req.priority).map_err(|e| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!("Invalid priority: {e}"),
      }),
    )
  })?;

  // Create new bead
  let new_bead = Bead {
    id: BeadId::new(),
    title: req.title,
    description: req.description,
    status,
    priority,
    bead_type,
    created_by: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  // Save to storage
  let mut beads = state.beads.write().await;
  beads.push(new_bead.clone());

  let summary = BeadSummary::from(new_bead);
  Ok((StatusCode::CREATED, Json(summary)))
}

/// Update an existing bead
///
/// # Errors
///
/// Returns a 404 error if the bead is not found
/// Returns a 400 error if validation fails
#[utoipa::path(
  put,
  path = "/api/beads/{id}",
  params(
    ("id" = String, Path, description = "Bead ID")
  ),
  request_body = UpdateBeadRequest,
  responses(
    (status = 200, description = "Bead updated", body = BeadSummary),
    (status = 404, description = "Bead not found", body = ErrorResponse),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "beads"
)]
async fn update_bead(
  Path(id): Path<String>,
  State(state): State<ApiState>,
  Json(req): Json<UpdateBeadRequest>,
) -> Result<Json<BeadSummary>, (StatusCode, Json<ErrorResponse>)> {
  let mut beads = state.beads.write().await;

  let bead_index = beads
    .iter()
    .position(|b| b.id.to_string() == id)
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          error: format!("Bead {id} not found"),
        }),
      )
    })?;

  let bead = &mut beads[bead_index];

  // Update fields if provided
  if let Some(title) = req.title {
    bead.title = title;
  }
  if let Some(description) = req.description {
    bead.description = Some(description);
  }
  if let Some(status_str) = req.status {
    bead.status = BeadStatus::from_str(&status_str).map_err(|e| {
      (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!("Invalid status: {e}"),
        }),
      )
    })?;
  }
  if let Some(priority_val) = req.priority {
    bead.priority = BeadPriority::new(priority_val).map_err(|e| {
      (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!("Invalid priority: {e}"),
        }),
      )
    })?;
  }
  if let Some(type_str) = req.bead_type {
    bead.bead_type = BeadType::from_str(&type_str).map_err(|e| {
      (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!("Invalid bead type: {e}"),
        }),
      )
    })?;
  }

  bead.updated_at = chrono::Utc::now();

  let summary = BeadSummary::from(bead.clone());
  Ok(Json(summary))
}

/// Delete a bead
///
/// # Errors
///
/// Returns a 404 error if the bead is not found
#[utoipa::path(
  delete,
  path = "/api/beads/{id}",
  params(
    ("id" = String, Path, description = "Bead ID")
  ),
  responses(
    (status = 204, description = "Bead deleted"),
    (status = 404, description = "Bead not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "beads"
)]
async fn delete_bead(
  Path(id): Path<String>,
  State(state): State<ApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  let mut beads = state.beads.write().await;

  let bead_index = beads
    .iter()
    .position(|b| b.id.to_string() == id)
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          error: format!("Bead {id} not found"),
        }),
      )
    })?;

  beads.remove(bead_index);
  Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_bead() -> Bead {
    Bead {
      id: BeadId::new(),
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::HIGH,
      bead_type: BeadType::Feature,
      created_by: None,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    }
  }

  #[test]
  fn test_api_state_new() {
    let state = ApiState::new();
    assert!(state.beads.read().now_or_never().is_some());
  }

  #[test]
  fn test_api_state_default() {
    let state = ApiState::default();
    assert!(state.beads.read().now_or_never().is_some());
  }

  #[test]
  fn test_bead_summary_from_bead() {
    let bead = create_test_bead();
    let summary = BeadSummary::from(bead.clone());

    assert_eq!(summary.id, bead.id.to_string());
    assert_eq!(summary.title, bead.title);
    assert_eq!(summary.description, bead.description);
    assert_eq!(summary.status, bead.status.as_str());
    assert_eq!(summary.priority, bead.priority.0);
    assert_eq!(summary.bead_type, bead.bead_type.as_str());
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
  fn test_error_response_serialization() {
    let response = ErrorResponse {
      error: "Test error".to_string(),
    };

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("\"error\":\"Test error\""));
  }

  #[tokio::test]
  async fn test_list_beads_empty() {
    let state = ApiState::new();
    let params = ListBeadQuery {
      status: None,
      bead_type: None,
      priority: None,
      search: None,
    };

    let result = list_beads(Query(params), State(state)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.0.total, 0);
    assert!(response.0.beads.is_empty());
  }

  #[tokio::test]
  async fn test_create_and_list_beads() {
    let state = ApiState::new();

    let req = CreateBeadRequest {
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
    };

    let result = create_bead(State(state.clone()), Json(req)).await;
    assert!(result.is_ok());

    let (status, summary) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(summary.title, "Test Bead");

    // List beads
    let params = ListBeadQuery {
      status: None,
      bead_type: None,
      priority: None,
      search: None,
    };

    let result = list_beads(Query(params), State(state)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.0.total, 1);
  }
}
