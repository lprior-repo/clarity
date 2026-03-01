//! Timestamp utility functions for planning module
//!
//! Provides ISO 8601 / RFC 3339 formatted timestamps for bead emission
//! and other planning operations.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

/// Returns the current UTC timestamp in ISO 8601 / RFC 3339 format.
///
/// # Format
///
/// The returned string is in RFC 3339 format: `YYYY-MM-DDTHH:MM:SS.sss+00:00`
///
/// # Example
///
/// ```
/// use clarity_web::intent::plan::timestamp::current_iso8601_timestamp;
///
/// let ts = current_iso8601_timestamp();
/// assert!(ts.ends_with("+00:00"));
/// assert!(ts.contains('T'));
/// ```
#[must_use]
pub fn current_iso8601_timestamp() -> String {
  chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_timestamp_format() {
    let ts = current_iso8601_timestamp();

    // Should end with '+00:00' for UTC (chrono's RFC 3339 format)
    assert!(
      ts.ends_with("+00:00"),
      "Timestamp should end with +00:00: {ts}"
    );

    // Should contain 'T' separator between date and time
    assert!(ts.contains('T'), "Timestamp should contain T: {ts}");

    // Should have correct length for RFC 3339 with microseconds
    // Format: YYYY-MM-DDTHH:MM:SS.ssssss+00:00 (32 chars)
    assert!(
      ts.len() >= 25 && ts.len() <= 35,
      "Timestamp length unexpected: {ts} (len={})",
      ts.len()
    );
  }

  #[test]
  fn test_timestamp_is_parseable() {
    let ts = current_iso8601_timestamp();

    // Should be parseable as RFC 3339
    let parsed = chrono::DateTime::parse_from_rfc3339(&ts);
    assert!(parsed.is_ok(), "Failed to parse timestamp: {ts}");
  }

  #[test]
  fn test_timestamp_is_recent() {
    let ts = current_iso8601_timestamp();
    let parsed = chrono::DateTime::parse_from_rfc3339(&ts);
    let parsed = match parsed {
      Ok(dt) => dt.with_timezone(&chrono::Utc),
      Err(_) => panic!("Failed to parse timestamp"),
    };

    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(parsed);

    // Should be within 1 second of current time
    assert!(diff.num_seconds().abs() <= 1, "Timestamp not recent: {ts}");
  }

  #[test]
  fn test_multiple_calls_differ() {
    let ts1 = current_iso8601_timestamp();

    // Small delay to ensure different timestamp
    std::thread::sleep(std::time::Duration::from_millis(10));

    let ts2 = current_iso8601_timestamp();

    // Multiple calls should produce different timestamps
    // (with enough precision, they will differ)
    assert_ne!(ts1, ts2, "Consecutive timestamps should differ");
  }
}
