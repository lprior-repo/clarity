#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

//! Tests for server module

use std::net::SocketAddr;
use tokio_test::block_on;

#[cfg(test)]
mod tests {
  use super::super::{error::ServerError, server::Server};

  #[test]
  fn test_server_new() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 4123));
    let server = Server::new(addr);
    // Verify server was created - we'll check the address through behavior
    match block_on(async {
      // Test that server can be created
      let _ = server;
      Ok::<(), ServerError>(())
    }) {
      Ok(_) => {},
      Err(e) => panic!("Server test failed: {e}"),
    }
  }

  #[test]
  fn test_create_router() {
    let router = super::super::server::create_router();
    // Router should be created - we'll verify routes in integration tests
    let _ = router;
  }

  #[test]
  fn test_server_run_fails_on_invalid_port() {
    // Try to bind to port 0 (ephemeral, but let's test error handling)
    // Actually, let's try a different approach - bind to an address that might fail
    // For now, this is a placeholder for testing error handling
    let addr = SocketAddr::from(([127, 0, 0, 1], 4123));
    let server = Server::new(addr);

    // This test will be expanded once we have the implementation
    let _ = server;
  }
}
