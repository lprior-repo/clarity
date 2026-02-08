#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

//! Tests for error module

#[cfg(test)]
mod tests {
  use super::super::error::ServerError;

  #[test]
  fn test_address_bind_error_display() {
    let address = match "127.0.0.1:8080".parse::<std::net::SocketAddr>() {
      Ok(addr) => addr,
      Err(e) => panic!("Failed to parse socket address: {e}"),
    };
    let error = ServerError::AddressBind {
      address,
      source: std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address already in use"),
    };
    let msg = format!("{}", error);
    assert!(msg.contains("Failed to bind to address"));
    assert!(msg.contains("127.0.0.1:8080"));
  }

  #[test]
  fn test_server_start_error_display() {
    let error = ServerError::ServerStart {
      source: std::io::Error::new(std::io::ErrorKind::Other, "Server error"),
    };
    let msg = format!("{}", error);
    assert!(msg.contains("Server error"));
  }

  #[test]
  fn test_error_is_send() {
    // Verify error can be sent across threads
    fn assert_send<T: Send>() {}
    assert_send::<ServerError>();
  }

  #[test]
  fn test_error_is_sync() {
    // Verify error is thread-safe
    fn assert_sync<T: Sync>() {}
    assert_sync::<ServerError>();
  }
}
