//! OpenCode Server API Client
//!
//! Fast, real HTTP client for OpenCode server with SSE streaming.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Default OpenCode server URL
pub const DEFAULT_URL: &str = "http://localhost:4096";

/// Connection status
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

/// Terminal line type
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalLineType {
  Cmd,
  Output,
  Comment,
  Separator,
  Error,
}

/// A line in the terminal feed
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

/// OpenCode session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
  pub id: String,
  pub title: Option<String>,
  #[serde(default)]
  pub created_at: DateTime<Utc>,
  #[serde(default)]
  pub updated_at: DateTime<Utc>,
}

/// OpenCode configuration
#[derive(Clone, Debug)]
pub struct OpenCodeConfig {
  pub url: String,
  pub password: Option<String>,
}

impl Default for OpenCodeConfig {
  fn default() -> Self {
    Self {
      url: DEFAULT_URL.to_string(),
      password: None,
    }
  }
}

/// Fast HTTP client for OpenCode server
#[derive(Clone, Debug)]
pub struct OpenCodeClient {
  config: OpenCodeConfig,
  http: reqwest::Client,
  status: Arc<RwLock<ConnectionStatus>>,
  session: Arc<RwLock<Option<Session>>>,
}

impl OpenCodeClient {
  /// Create a new OpenCode client
  #[must_use]
  pub fn new(config: OpenCodeConfig) -> Self {
    let http = reqwest::Client::builder()
      .timeout(Duration::from_secs(60))
      .connect_timeout(Duration::from_secs(3))
      .tcp_nodelay(true)
      .build()
      .unwrap_or_else(|_| reqwest::Client::new());

    Self {
      config,
      http,
      status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
      session: Arc::new(RwLock::new(None)),
    }
  }

  /// Create with default config
  #[must_use]
  pub fn default_client() -> Self {
    Self::new(OpenCodeConfig::default())
  }

  /// Get current connection status
  pub async fn status(&self) -> ConnectionStatus {
    *self.status.read().await
  }

  /// Get current session
  pub async fn session(&self) -> Option<Session> {
    self.session.read().await.clone()
  }

  /// Check server health - ACTUAL HTTP CALL
  pub async fn check_health(&self) -> bool {
    {
      let mut status = self.status.write().await;
      *status = ConnectionStatus::Connecting;
    }

    let url = format!("{}/global/health", self.config.url);

    match self.http.get(&url).send().await {
      Ok(response) => {
        let connected = response.status().is_success();
        let mut status = self.status.write().await;
        *status = if connected {
          ConnectionStatus::Connected
        } else {
          ConnectionStatus::Error
        };
        connected
      }
      Err(_) => {
        let mut status = self.status.write().await;
        *status = ConnectionStatus::Disconnected;
        false
      }
    }
  }

  /// Create a new session - ACTUAL HTTP CALL
  pub async fn create_session(&self, title: &str) -> Option<Session> {
    let url = format!("{}/session", self.config.url);
    let body = serde_json::json!({ "title": title });

    let response = self.http.post(&url).json(&body).send().await.ok()?;

    if !response.status().is_success() {
      return None;
    }

    let session: Session = response.json().await.ok()?;
    let mut current = self.session.write().await;
    *current = Some(session.clone());

    Some(session)
  }

  /// Send a command and stream responses - ACTUAL HTTP WITH SSE
  pub async fn send_command_streaming<F>(&self, command: &str, mut on_line: F) -> bool
  where
    F: FnMut(TerminalLine) + Send,
  {
    let session = match self.session.read().await.clone() {
      Some(s) => s,
      None => return false,
    };

    let url = format!("{}/session/{}/message", self.config.url, session.id);
    let body = serde_json::json!({
      "parts": [{"type": "text", "text": command}]
    });

    // Emit command immediately
    on_line(TerminalLine::cmd(command.to_string()));

    let response = match self.http.post(&url).json(&body).send().await {
      Ok(r) => r,
      Err(e) => {
        on_line(TerminalLine::error(format!("HTTP error: {e}")));
        return false;
      }
    };

    if !response.status().is_success() {
      on_line(TerminalLine::error(format!(
        "Server: {}",
        response.status()
      )));
      return false;
    }

    // Stream SSE
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
      match chunk_result {
        Ok(chunk) => {
          if let Ok(text) = std::str::from_utf8(&chunk) {
            buffer.push_str(text);
            while let Some(event_end) = buffer.find("\n\n") {
              let event = buffer[..event_end].to_string();
              buffer = buffer[event_end + 2..].to_string();
              self.process_sse(&event, &mut on_line);
            }
          }
        }
        Err(e) => {
          on_line(TerminalLine::error(format!("Stream: {e}")));
          return false;
        }
      }
    }
    true
  }

  fn process_sse<F>(&self, event: &str, on_line: &mut F)
  where
    F: FnMut(TerminalLine),
  {
    for line in event.lines() {
      if let Some(data) = line.strip_prefix("data: ") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
          if let Some(t) = json.get("text").and_then(|v| v.as_str()) {
            on_line(TerminalLine::output(t.to_string()));
          } else if let Some(c) = json.get("content").and_then(|v| v.as_str()) {
            on_line(TerminalLine::output(c.to_string()));
          } else if let Some(tool) = json.get("tool").and_then(|v| v.as_str()) {
            on_line(TerminalLine::output(format!("[{tool}]")));
          } else if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            on_line(TerminalLine::error(err.to_string()));
          }
        } else {
          on_line(TerminalLine::output(data.to_string()));
        }
      }
    }
  }

  /// Get the server URL
  #[must_use]
  pub fn url(&self) -> &str {
    &self.config.url
  }
}

impl Default for OpenCodeClient {
  fn default() -> Self {
    Self::default_client()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_terminal_line_cmd() {
    let line = TerminalLine::cmd("br list".to_string());
    assert_eq!(line.line_type, TerminalLineType::Cmd);
  }

  #[test]
  fn test_connection_status_display() {
    assert_eq!(ConnectionStatus::Connected.to_string(), "Connected");
  }

  #[test]
  fn test_client_default() {
    let client = OpenCodeClient::default();
    assert_eq!(client.url(), DEFAULT_URL);
  }
}
