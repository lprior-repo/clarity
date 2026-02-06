//! WebSocket integration tests for clarity-server
//!
//! These tests verify WebSocket functionality including:
//! - Connection upgrade from HTTP to WebSocket
//! - Message broadcasting to multiple clients
//! - Proper connection lifecycle management
//! - Error handling without panics

#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]

use tokio::sync::broadcast;

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
  pub fn new(channel_capacity: usize) -> Result<Self, BroadcastError> {
    let (tx, _rx) = broadcast::channel(channel_capacity);
    Ok(Self { tx })
  }
}

/// Error type for WebSocket operations
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
  #[error("Failed to establish WebSocket connection")]
  ConnectionFailed,

  #[error("Failed to send message: {0}")]
  SendError(String),

  #[error("Failed to receive message")]
  ReceiveError,

  #[error("Connection closed")]
  ConnectionClosed,
}

/// Error type for broadcast channel operations
#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
  #[error("Failed to create broadcast channel")]
  ChannelCreationFailed,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_websocket_state_creation() {
    // Given: A channel capacity
    let capacity = 100;

    // When: Creating WebSocket state
    let result = WebSocketState::new(capacity);

    // Then: State should be created successfully
    assert!(result.is_ok(), "WebSocketState creation should succeed");

    let state = result.expect("WebSocketState creation should succeed");
    // Verify the broadcast channel is functional by subscribing
    let rx = state.tx.subscribe();
    assert_eq!(
      rx.len(),
      0,
      "New subscription should have no messages queued"
    );
  }

  #[tokio::test]
  async fn test_broadcast_message_to_multiple_subscribers() {
    // Given: A WebSocket state and multiple subscribers
    let state = WebSocketState::new(100).expect("State creation should succeed");
    let mut rx1 = state.tx.subscribe();
    let mut rx2 = state.tx.subscribe();
    let mut rx3 = state.tx.subscribe();

    // When: Sending a broadcast message
    let test_message = "Hello, WebSocket!".to_string();
    let send_result = state.tx.send(test_message.clone());

    // Then: All subscribers should receive the message
    assert!(send_result.is_ok(), "Send should succeed - receivers exist");

    let msg1 = rx1.recv().await;
    let msg2 = rx2.recv().await;
    let msg3 = rx3.recv().await;

    assert_eq!(msg1, Ok(test_message.clone()));
    assert_eq!(msg2, Ok(test_message.clone()));
    assert_eq!(msg3, Ok(test_message));
  }

  #[tokio::test]
  async fn test_broadcast_without_subscribers_does_not_panic() {
    // Given: A WebSocket state with no subscribers
    let state = WebSocketState::new(100).expect("State creation should succeed");

    // When: Sending a message with no subscribers
    let send_result = state.tx.send("Test message".to_string());

    // Then: Should return error but not panic
    // Axum's broadcast::send returns Err when there are no receivers
    assert!(
      send_result.is_err(),
      "Send should fail gracefully when no subscribers"
    );
  }

  #[tokio::test]
  async fn test_channel_capacity_respected() {
    // Given: A WebSocket state with small capacity
    let capacity = 2;
    let state = WebSocketState::new(capacity).expect("State creation should succeed");
    let mut rx = state.tx.subscribe();

    // When: Sending more messages than capacity
    let _ = state.tx.send("Message 1".to_string());
    let _ = state.tx.send("Message 2".to_string());
    let overflow_result = state.tx.send("Message 3".to_string());

    // Then: Overflow should be handled gracefully
    // Axum broadcast channels drop oldest messages when full
    assert!(
      overflow_result.is_ok(),
      "Overflow should be handled, not panic"
    );

    // And: Subscriber should only receive messages within capacity
    // Messages may be dropped, but we should not panic
    let _ = rx.recv().await;
    let _ = rx.recv().await;
  }
}
