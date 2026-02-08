//! HTTP client for Clarity API
//!
//! This module provides a client for communicating with the Clarity backend.

use crate::api::types::*;
use thiserror::Error;

/// Default server address
const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:4123";

/// API client for Clarity backend
#[derive(Debug, Clone)]
pub struct ApiClient {
  client: reqwest::Client,
  base_url: String,
}

impl ApiClient {
  /// Create a new API client with default server URL
  #[must_use]
  pub fn new() -> Self {
    Self::with_base_url(DEFAULT_SERVER_URL.to_string())
  }

  /// Create a new API client with custom server URL
  #[must_use]
  pub fn with_base_url(base_url: String) -> Self {
    Self {
      client: reqwest::Client::new(),
      base_url,
    }
  }

  /// Get the base URL
  #[must_use]
  pub fn base_url(&self) -> &str {
    &self.base_url
  }

  /// Check server health
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Server returns non-OK status
  /// - Response cannot be parsed
  pub async fn health(&self) -> Result<HealthResponse, ApiError> {
    let url = format!("{}/health", self.base_url);
    let response = self.client.get(&url).send().await?;

    if !response.status().is_success() {
      return Err(ApiError::HttpError(response.status().as_u16()));
    }

    let health = response.json().await?;
    Ok(health)
  }

  /// List beads with optional filtering
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Server returns non-OK status
  /// - Response cannot be parsed
  pub async fn list_beads(
    &self,
    status: Option<&str>,
    bead_type: Option<&str>,
    priority: Option<i16>,
    search: Option<&str>,
  ) -> Result<ListBeadsResponse, ApiError> {
    let mut url = format!("{}/api/beads", self.base_url);
    let mut params = Vec::new();

    if let Some(s) = status {
      params.push(format!("status={s}"));
    }
    if let Some(t) = bead_type {
      params.push(format!("bead_type={t}"));
    }
    if let Some(p) = priority {
      params.push(format!("priority={p}"));
    }
    if let Some(q) = search {
      params.push(format!("search={q}"));
    }

    if !params.is_empty() {
      url.push('?');
      url.push_str(&params.join("&"));
    }

    let response = self.client.get(&url).send().await?;

    if !response.status().is_success() {
      return Err(ApiError::HttpError(response.status().as_u16()));
    }

    let beads_response = response.json().await?;
    Ok(beads_response)
  }

  /// Get a single bead by ID
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Bead not found (404)
  /// - Response cannot be parsed
  pub async fn get_bead(&self, id: &str) -> Result<BeadSummary, ApiError> {
    let url = format!("{}/api/beads/{}", self.base_url, id);
    let response = self.client.get(&url).send().await?;

    match response.status().as_u16() {
      404 => Err(ApiError::NotFound(id.to_string())),
      status if !response.status().is_success() => Err(ApiError::HttpError(status)),
      _ => {
        let bead = response.json().await?;
        Ok(bead)
      }
    }
  }

  /// Create a new bead
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Server returns non-success status
  /// - Response cannot be parsed
  pub async fn create_bead(&self, request: CreateBeadRequest) -> Result<BeadSummary, ApiError> {
    let url = format!("{}/api/beads", self.base_url);
    let response = self.client.post(&url).json(&request).send().await?;

    if !response.status().is_success() {
      let status = response.status().as_u16();
      // Try to parse error response
      if let Ok(error_resp) = response.json::<ErrorResponse>().await {
        return Err(ApiError::ServerError(error_resp.error));
      }
      return Err(ApiError::HttpError(status));
    }

    let bead = response.json().await?;
    Ok(bead)
  }

  /// Update an existing bead
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Bead not found (404)
  /// - Response cannot be parsed
  pub async fn update_bead(
    &self,
    id: &str,
    request: UpdateBeadRequest,
  ) -> Result<BeadSummary, ApiError> {
    let url = format!("{}/api/beads/{}", self.base_url, id);
    let response = self.client.put(&url).json(&request).send().await?;

    match response.status().as_u16() {
      404 => Err(ApiError::NotFound(id.to_string())),
      status if !response.status().is_success() => Err(ApiError::HttpError(status)),
      _ => {
        let bead = response.json().await?;
        Ok(bead)
      }
    }
  }

  /// Delete a bead
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Bead not found (404)
  pub async fn delete_bead(&self, id: &str) -> Result<(), ApiError> {
    let url = format!("{}/api/beads/{}", self.base_url, id);
    let response = self.client.delete(&url).send().await?;

    match response.status().as_u16() {
      404 => Err(ApiError::NotFound(id.to_string())),
      204 => Ok(()),
      status => Err(ApiError::HttpError(status)),
    }
  }

  /// List sessions
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Server returns non-OK status
  /// - Response cannot be parsed
  pub async fn list_sessions(&self) -> Result<ListSessionsResponse, ApiError> {
    let url = format!("{}/api/sessions", self.base_url);
    let response = self.client.get(&url).send().await?;

    if !response.status().is_success() {
      return Err(ApiError::HttpError(response.status().as_u16()));
    }

    let sessions_response = response.json().await?;
    Ok(sessions_response)
  }

  /// Create a new session
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Network request fails
  /// - Server returns non-success status
  /// - Response cannot be parsed
  pub async fn create_session(
    &self,
    request: CreateSessionRequest,
  ) -> Result<SessionSummary, ApiError> {
    let url = format!("{}/api/sessions", self.base_url);
    let response = self.client.post(&url).json(&request).send().await?;

    if !response.status().is_success() {
      let status = response.status().as_u16();
      // Try to parse error response
      if let Ok(error_resp) = response.json::<ErrorResponse>().await {
        return Err(ApiError::ServerError(error_resp.error));
      }
      return Err(ApiError::HttpError(status));
    }

    let session = response.json().await?;
    Ok(session)
  }
}

impl Default for ApiClient {
  fn default() -> Self {
    Self::new()
  }
}

/// API error types
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ApiError {
  /// Network error
  #[error("Network error: {0}")]
  NetworkError(String),

  /// HTTP error with status code
  #[error("HTTP error: {0}")]
  HttpError(u16),

  /// Resource not found
  #[error("Resource not found: {0}")]
  NotFound(String),

  /// Server error with message
  #[error("Server error: {0}")]
  ServerError(String),

  /// JSON parse error
  #[error("Failed to parse JSON response")]
  JsonError,
}

// Implement conversion from reqwest::Error
impl From<reqwest::Error> for ApiError {
  fn from(err: reqwest::Error) -> Self {
    if err.is_timeout() || err.is_connect() {
      ApiError::NetworkError(err.to_string())
    } else if err.is_decode() || err.is_body() {
      ApiError::JsonError
    } else {
      ApiError::NetworkError(err.to_string())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_api_client_new() {
    let client = ApiClient::new();
    assert_eq!(client.base_url(), DEFAULT_SERVER_URL);
  }

  #[test]
  fn test_api_client_with_base_url() {
    let client = ApiClient::with_base_url("http://localhost:8080".to_string());
    assert_eq!(client.base_url(), "http://localhost:8080");
  }

  #[test]
  fn test_api_client_default() {
    let client = ApiClient::default();
    assert_eq!(client.base_url(), DEFAULT_SERVER_URL);
  }

  #[test]
  fn test_api_error_display() {
    let err = ApiError::NetworkError("connection refused".to_string());
    assert_eq!(err.to_string(), "Network error: connection refused");
  }

  #[test]
  fn test_api_error_http() {
    let err = ApiError::HttpError(404);
    assert_eq!(err.to_string(), "HTTP error: 404");
  }

  #[test]
  fn test_api_error_not_found() {
    let err = ApiError::NotFound("bd-123".to_string());
    assert_eq!(err.to_string(), "Resource not found: bd-123");
  }

  #[test]
  fn test_api_error_server() {
    let err = ApiError::ServerError("Internal server error".to_string());
    assert_eq!(err.to_string(), "Server error: Internal server error");
  }

  #[test]
  fn test_api_error_json() {
    let err = ApiError::JsonError;
    assert_eq!(err.to_string(), "Failed to parse JSON response");
  }

  #[test]
  fn test_bead_summary_equality() {
    let bead1 = BeadSummary {
      id: "test-id".to_string(),
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let bead2 = BeadSummary {
      id: "test-id".to_string(),
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_at: "2024-01-01T00:00:00Z".to_string(),
    };

    assert_eq!(bead1, bead2);
  }

  #[test]
  fn test_create_bead_request_equality() {
    let req1 = CreateBeadRequest {
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
    };

    let req2 = CreateBeadRequest {
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
    };

    assert_eq!(req1, req2);
  }
}
