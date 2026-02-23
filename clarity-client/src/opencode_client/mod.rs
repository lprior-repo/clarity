//! OpenCode Server API Client
//!
//! Correct HTTP client for OpenCode server.
//!
//! ## Architecture
//!
//! `POST /session/{id}/message` is a **synchronous** endpoint — it blocks until the
//! model finishes and returns the completed `AssistantMessage` JSON.  We parse
//! the `parts` array out of that response to build `TerminalLine`s.
//!
//! There is no need to parse an SSE stream from this endpoint.  Real streaming
//! via `GET /event` is not used here — the sync approach is simpler and correct
//! for this prototype.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Default OpenCode server URL
pub const DEFAULT_URL: &str = "http://localhost:4096";

// ─────────────────────────────────────────────────────────────────────────────
// Connection status
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConnectionStatus {
  Connected,
  Connecting,
  Disconnected,
  Error,
}

impl fmt::Display for ConnectionStatus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Connected => write!(f, "Connected"),
      Self::Connecting => write!(f, "Connecting..."),
      Self::Disconnected => write!(f, "Disconnected"),
      Self::Error => write!(f, "Error"),
    }
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Terminal line
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalLineType {
  Cmd,
  Output,
  Comment,
  Separator,
  Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerminalLine {
  pub line_type: TerminalLineType,
  pub text: String,
  pub agent: Option<String>,
  pub timestamp: DateTime<Utc>,
  pub executed: bool,
}

impl TerminalLine {
  #[must_use]
  pub fn cmd(text: String) -> Self {
    Self {
      line_type: TerminalLineType::Cmd,
      text,
      agent: None,
      timestamp: Utc::now(),
      executed: false,
    }
  }

  #[must_use]
  pub fn output(text: String) -> Self {
    Self {
      line_type: TerminalLineType::Output,
      text,
      agent: None,
      timestamp: Utc::now(),
      executed: false,
    }
  }

  #[must_use]
  pub fn comment(text: String) -> Self {
    Self {
      line_type: TerminalLineType::Comment,
      text,
      agent: None,
      timestamp: Utc::now(),
      executed: false,
    }
  }

  #[must_use]
  pub fn separator() -> Self {
    Self {
      line_type: TerminalLineType::Separator,
      text: String::new(),
      agent: None,
      timestamp: Utc::now(),
      executed: false,
    }
  }

  #[must_use]
  pub fn error(text: String) -> Self {
    Self {
      line_type: TerminalLineType::Error,
      text,
      agent: None,
      timestamp: Utc::now(),
      executed: false,
    }
  }

  #[must_use]
  pub fn with_agent(mut self, agent: String) -> Self {
    self.agent = Some(agent);
    self
  }

  #[must_use]
  pub fn executed(mut self) -> Self {
    self.executed = true;
    self
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// API types
// ─────────────────────────────────────────────────────────────────────────────

/// OpenCode session (mirrors the server's `Session` type)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
  pub id: String,
  pub title: Option<String>,
  #[serde(default)]
  pub created_at: DateTime<Utc>,
  #[serde(default)]
  pub updated_at: DateTime<Utc>,
}

/// Model identifier
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelId {
  #[serde(rename = "providerID")]
  pub provider_id: String,
  #[serde(rename = "modelID")]
  pub model_id: String,
}

impl ModelId {
  /// GLM 4.7 via zai-coding-plan — confirmed working via live test
  #[must_use]
  pub fn glm_4_7() -> Self {
    Self {
      provider_id: "zai-coding-plan".to_string(),
      model_id: "glm-4.7".to_string(),
    }
  }
}

impl Default for ModelId {
  fn default() -> Self {
    Self::glm_4_7()
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Response types for POST /session/{id}/message
// ─────────────────────────────────────────────────────────────────────────────

/// A single part inside an AssistantMessage
#[derive(Debug, Deserialize)]
struct MessagePart {
  #[serde(rename = "type")]
  part_type: String,
  // present on text / reasoning parts
  text: Option<String>,
  // present on tool parts
  tool: Option<String>,
}

/// Top-level response from POST /session/{id}/message
#[derive(Debug, Deserialize)]
struct AssistantMessage {
  #[serde(default)]
  parts: Vec<MessagePart>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct OpenCodeConfig {
  pub url: String,
  pub password: Option<String>,
  pub model: ModelId,
  /// Timeout for the blocking `/session/{id}/message` call (default: 5 min)
  pub message_timeout_secs: u64,
}

impl Default for OpenCodeConfig {
  fn default() -> Self {
    Self {
      url: DEFAULT_URL.to_string(),
      password: None,
      model: ModelId::default(),
      message_timeout_secs: 300,
    }
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Client
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct OpenCodeClient {
  config: OpenCodeConfig,
  http: reqwest::Client,
  status: Arc<RwLock<ConnectionStatus>>,
  session: Arc<RwLock<Option<Session>>>,
}

impl OpenCodeClient {
  #[must_use]
  pub fn new(config: OpenCodeConfig) -> Self {
    // The message call can take minutes — use a generous timeout.
    let http = reqwest::Client::builder()
      .timeout(Duration::from_secs(config.message_timeout_secs + 10))
      .connect_timeout(Duration::from_secs(5))
      .tcp_nodelay(true)
      .build()
      .unwrap_or_default();

    Self {
      config,
      http,
      status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
      session: Arc::new(RwLock::new(None)),
    }
  }

  #[must_use]
  pub fn default_client() -> Self {
    Self::new(OpenCodeConfig::default())
  }

  // ── Accessors ────────────────────────────────────────────────────────────

  pub async fn status(&self) -> ConnectionStatus {
    *self.status.read().await
  }

  pub async fn session(&self) -> Option<Session> {
    self.session.read().await.clone()
  }

  #[must_use]
  pub fn url(&self) -> &str {
    &self.config.url
  }

  #[must_use]
  pub fn model(&self) -> &ModelId {
    &self.config.model
  }

  // ── Health ───────────────────────────────────────────────────────────────

  /// `GET /global/health` — returns true when the server is reachable.
  pub async fn check_health(&self) -> bool {
    {
      let mut s = self.status.write().await;
      *s = ConnectionStatus::Connecting;
    }
    let url = format!("{}/global/health", self.config.url);
    match self.http.get(&url).send().await {
      Ok(r) if r.status().is_success() => {
        *self.status.write().await = ConnectionStatus::Connected;
        true
      }
      _ => {
        *self.status.write().await = ConnectionStatus::Disconnected;
        false
      }
    }
  }

  // ── Session ──────────────────────────────────────────────────────────────

  /// `POST /session` — create a new session and store it.
  pub async fn create_session(&self, title: &str) -> Option<Session> {
    let url = format!("{}/session", self.config.url);
    let body = serde_json::json!({ "title": title });
    let resp = self.http.post(&url).json(&body).send().await.ok()?;
    if !resp.status().is_success() {
      return None;
    }
    let session: Session = resp.json().await.ok()?;
    *self.session.write().await = Some(session.clone());
    Some(session)
  }

  // ── Message ──────────────────────────────────────────────────────────────

  /// `POST /session/{id}/message` (synchronous) — send `prompt` and collect
  /// the model's full response as a list of [`TerminalLine`]s.
  ///
  /// The endpoint blocks until the model finishes and returns the complete
  /// `AssistantMessage` JSON.  We parse `parts` out of it.
  ///
  /// Returns `Ok(lines)` on success, `Err(description)` on any failure.
  pub async fn send_message(&self, prompt: &str) -> Result<Vec<TerminalLine>, String> {
    let session = self
      .session
      .read()
      .await
      .clone()
      .ok_or_else(|| "no active session".to_string())?;

    let url = format!("{}/session/{}/message", self.config.url, session.id);
    let body = serde_json::json!({
        "model": {
            "providerID": self.config.model.provider_id,
            "modelID":    self.config.model.model_id
        },
        "agent": "build",
        "parts": [{ "type": "text", "text": prompt }]
    });

    let resp = self
      .http
      .post(&url)
      .json(&body)
      .send()
      .await
      .map_err(|e| format!("HTTP send error: {e}"))?;

    if !resp.status().is_success() {
      let status = resp.status();
      let body_text = resp.text().await.unwrap_or_default();
      return Err(format!("server {status}: {body_text}"));
    }

    // The response is the full AssistantMessage JSON (not SSE).
    let msg: AssistantMessage = resp
      .json()
      .await
      .map_err(|e| format!("JSON parse error: {e}"))?;

    Ok(Self::parts_to_lines(&msg.parts))
  }

  /// Convert `MessagePart`s into display lines.
  fn parts_to_lines(parts: &[MessagePart]) -> Vec<TerminalLine> {
    parts
      .iter()
      .filter_map(|p| match p.part_type.as_str() {
        "text" => p
          .text
          .as_deref()
          .filter(|t| !t.is_empty())
          .map(|t| TerminalLine::output(t.to_string())),
        "reasoning" => p
          .text
          .as_deref()
          .filter(|t| !t.is_empty())
          .map(|t| TerminalLine::comment(format!("[thinking] {t}"))),
        "tool" => p
          .tool
          .as_deref()
          .map(|name| TerminalLine::comment(format!("[{name}]"))),
        _ => None,
      })
      .collect()
  }

  // ── Legacy compat ────────────────────────────────────────────────────────

  /// Convenience wrapper — sends a command and calls `on_line` for each line.
  ///
  /// Emits the prompt as a `Cmd` line first, then streams the response lines.
  /// Returns `true` on success, `false` on error (errors are emitted too).
  pub async fn send_command_streaming<F>(&self, command: &str, mut on_line: F) -> bool
  where
    F: FnMut(TerminalLine) + Send,
  {
    on_line(TerminalLine::cmd(command.to_string()));
    match self.send_message(command).await {
      Ok(lines) => {
        for line in lines {
          on_line(line);
        }
        true
      }
      Err(e) => {
        on_line(TerminalLine::error(e));
        false
      }
    }
  }
}

impl Default for OpenCodeClient {
  fn default() -> Self {
    Self::default_client()
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_terminal_line_cmd() {
    let line = TerminalLine::cmd("br list".to_string());
    assert_eq!(line.line_type, TerminalLineType::Cmd);
    assert_eq!(line.text, "br list");
  }

  #[test]
  fn test_connection_status_display() {
    assert_eq!(ConnectionStatus::Connected.to_string(), "Connected");
    assert_eq!(ConnectionStatus::Connecting.to_string(), "Connecting...");
    assert_eq!(ConnectionStatus::Disconnected.to_string(), "Disconnected");
    assert_eq!(ConnectionStatus::Error.to_string(), "Error");
  }

  #[test]
  fn test_client_default_url() {
    let client = OpenCodeClient::default();
    assert_eq!(client.url(), DEFAULT_URL);
  }

  #[test]
  fn test_default_model_is_glm_4_7() {
    let client = OpenCodeClient::default();
    assert_eq!(client.model().provider_id, "zai-coding-plan");
    assert_eq!(client.model().model_id, "glm-4.7");
  }

  #[test]
  fn test_model_id_serializes_correctly() {
    let m = ModelId::glm_4_7();
    let json = serde_json::to_value(&m).expect("serialise");
    assert_eq!(json["providerID"], "zai-coding-plan");
    assert_eq!(json["modelID"], "glm-4.7");
  }

  #[test]
  fn test_parts_to_lines_text() {
    let parts = vec![MessagePart {
      part_type: "text".to_string(),
      text: Some("hello world".to_string()),
      tool: None,
    }];
    let lines = OpenCodeClient::parts_to_lines(&parts);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].line_type, TerminalLineType::Output);
    assert_eq!(lines[0].text, "hello world");
  }

  #[test]
  fn test_parts_to_lines_reasoning() {
    let parts = vec![MessagePart {
      part_type: "reasoning".to_string(),
      text: Some("thinking...".to_string()),
      tool: None,
    }];
    let lines = OpenCodeClient::parts_to_lines(&parts);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].line_type, TerminalLineType::Comment);
    assert!(lines[0].text.contains("thinking..."));
  }

  #[test]
  fn test_parts_to_lines_skips_empty_text() {
    let parts = vec![MessagePart {
      part_type: "text".to_string(),
      text: Some(String::new()),
      tool: None,
    }];
    let lines = OpenCodeClient::parts_to_lines(&parts);
    assert!(lines.is_empty());
  }

  #[test]
  fn test_parts_to_lines_tool() {
    let parts = vec![MessagePart {
      part_type: "tool".to_string(),
      text: None,
      tool: Some("bash".to_string()),
    }];
    let lines = OpenCodeClient::parts_to_lines(&parts);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].line_type, TerminalLineType::Comment);
    assert!(lines[0].text.contains("bash"));
  }

  #[test]
  fn test_parts_to_lines_unknown_type_ignored() {
    let parts = vec![MessagePart {
      part_type: "step-start".to_string(),
      text: None,
      tool: None,
    }];
    let lines = OpenCodeClient::parts_to_lines(&parts);
    assert!(lines.is_empty());
  }
}
