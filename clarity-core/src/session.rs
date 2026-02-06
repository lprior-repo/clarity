//! Session types and management for Clarity

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use std::sync::Arc;
use tokio::sync::broadcast;
use thiserror::Error;

/// Error type for session operations
#[derive(Debug, Error, Clone)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Session expired")]
    Expired,

    #[error("Invalid session token")]
    InvalidToken,
}

/// Shared application state for WebSocket connections
#[derive(Clone)]
pub struct WebSocketState {
    /// Broadcast channel for sending messages to all connected clients
    pub tx: broadcast::Sender<String>,
}

impl WebSocketState {
    /// Create a new WebSocket state with a broadcast channel
    ///
    /// # Errors
    ///
    /// Returns an error if the broadcast channel cannot be created
    pub fn new(channel_capacity: usize) -> Result<Self, SessionError> {
        let (tx, _rx) = broadcast::channel(channel_capacity);
        Ok(Self { tx })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_state_creation() {
        let capacity = 100;
        let result = WebSocketState::new(capacity);

        assert!(result.is_ok(), "WebSocketState creation should succeed");

        let state = result.unwrap();
        let rx = state.tx.subscribe();
        assert_eq!(rx.len(), 0, "New subscription should have no messages");
    }
}
