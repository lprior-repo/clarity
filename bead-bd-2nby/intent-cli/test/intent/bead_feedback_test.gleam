// Test suite for bead_feedback module
// Tests validation logic and error handling
import gleam/option
import gleeunit
import gleeunit/should
import intent/bead_feedback

pub fn main() -> Nil {
  gleeunit.main()
}

// =============================================================================
// BEAD RESULT TYPE TESTS
// =============================================================================

pub fn bead_result_creation_test() {
  // Test that BeadResult types can be created
  let _success = bead_feedback.Success
  let _failed = bead_feedback.Failed
  let _blocked = bead_feedback.Blocked
  let _skipped = bead_feedback.Skipped

  // If we get here, types exist
  should.equal(True, True)
}

// =============================================================================
// ERROR TYPE TESTS
// =============================================================================

pub fn feedback_error_session_not_found_test() {
  let error = bead_feedback.SessionNotFound(session_id: "test-session")
  case error {
    bead_feedback.SessionNotFound(sid) -> {
      sid
      |> should.equal("test-session")
    }
    _ -> should.fail()
  }
}

pub fn feedback_error_write_error_test() {
  let error =
    bead_feedback.WriteError(
      path: "/tmp/test.cue",
      message: "Permission denied",
    )
  case error {
    bead_feedback.WriteError(path, msg) -> {
      path
      |> should.equal("/tmp/test.cue")
      msg
      |> should.equal("Permission denied")
    }
    _ -> should.fail()
  }
}

pub fn feedback_error_validation_error_test() {
  let error = bead_feedback.ValidationError(message: "Invalid format")
  case error {
    bead_feedback.ValidationError(msg) -> {
      msg
      |> should.equal("Invalid format")
    }
    _ -> should.fail()
  }
}

// =============================================================================
// BEAD ERROR TYPE TESTS
// =============================================================================

pub fn bead_error_creation_test() {
  let error =
    bead_feedback.BeadError(
      error_type: "Timeout",
      message: "Request timed out",
      trace: option.Some("stack trace here"),
    )

  error.error_type
  |> should.equal("Timeout")

  error.message
  |> should.equal("Request timed out")

  case error.trace {
    option.Some(trace) -> {
      trace
      |> should.equal("stack trace here")
    }
    option.None -> should.fail()
  }
}

pub fn bead_error_without_trace_test() {
  let error =
    bead_feedback.BeadError(
      error_type: "ParseError",
      message: "Invalid JSON",
      trace: option.None,
    )

  error.error_type
  |> should.equal("ParseError")

  error.message
  |> should.equal("Invalid JSON")

  case error.trace {
    option.Some(_) -> should.fail()
    option.None -> should.equal(True, True)
  }
}

// =============================================================================
// BLOCKED REASON TYPE TESTS
// =============================================================================

pub fn blocked_reason_creation_test() {
  let reason =
    bead_feedback.BlockedReason(
      blocker_type: "dependency",
      details: "Waiting for AUTH-001 to complete",
      unblocks_when: "When AUTH-001 is marked complete",
    )

  reason.blocker_type
  |> should.equal("dependency")

  reason.details
  |> should.equal("Waiting for AUTH-001 to complete")

  reason.unblocks_when
  |> should.equal("When AUTH-001 is marked complete")
}

// =============================================================================
// BEAD FEEDBACK TYPE TESTS
// =============================================================================

pub fn bead_feedback_creation_test() {
  let feedback =
    bead_feedback.BeadFeedback(
      bead_id: "AUTH-001",
      result: bead_feedback.Success,
      reason: "Completed successfully",
      executed_at: "2024-02-08T14:30:00Z",
      duration_ms: 1500,
      error: option.None,
      blocked_by: option.None,
    )

  feedback.bead_id
  |> should.equal("AUTH-001")

  feedback.reason
  |> should.equal("Completed successfully")

  feedback.duration_ms
  |> should.equal(1500)
}

pub fn bead_feedback_with_error_test() {
  let error =
    option.Some(bead_feedback.BeadError(
      error_type: "NetworkError",
      message: "Connection refused",
      trace: option.None,
    ))

  let feedback =
    bead_feedback.BeadFeedback(
      bead_id: "API-042",
      result: bead_feedback.Failed,
      reason: "API call failed",
      executed_at: "2024-02-08T14:31:00Z",
      duration_ms: 5000,
      error: error,
      blocked_by: option.None,
    )

  feedback.result
  |> should.equal(bead_feedback.Failed)

  case feedback.error {
    option.Some(err) -> {
      err.error_type
      |> should.equal("NetworkError")
    }
    option.None -> should.fail()
  }
}

pub fn bead_feedback_blocked_test() {
  let blocked =
    option.Some(bead_feedback.BlockedReason(
      blocker_type: "missing_config",
      details: "API key not configured",
      unblocks_when: "After API key is set",
    ))

  let feedback =
    bead_feedback.BeadFeedback(
      bead_id: "CONFIG-001",
      result: bead_feedback.Blocked,
      reason: "Waiting for configuration",
      executed_at: "2024-02-08T14:32:00Z",
      duration_ms: 100,
      error: option.None,
      blocked_by: blocked,
    )

  feedback.result
  |> should.equal(bead_feedback.Blocked)

  case feedback.blocked_by {
    option.Some(reason) -> {
      reason.blocker_type
      |> should.equal("missing_config")
    }
    option.None -> should.fail()
  }
}
