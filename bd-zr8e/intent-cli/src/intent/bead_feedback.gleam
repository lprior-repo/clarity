//// Track bead execution results via append-only CUE files.
////
//// Implements Option B architecture from beads.cue:
//// - Feedback appended to .intent/feedback-{session_id}.cue
//// - CUE unification automatically merges with session file
//// - No JSONL - pure CUE state for all operations

import gleam/list
import gleam/option
import gleam/result
import gleam/string
import simplifile

/// Represents the result of bead execution
pub type BeadResult {
  Success
  Failed
  Blocked
  Skipped
}

/// Feedback for a single bead execution
pub type BeadFeedback {
  BeadFeedback(
    bead_id: String,
    result: BeadResult,
    reason: String,
    executed_at: String,
    duration_ms: Int,
    error: option.Option(BeadError),
    blocked_by: option.Option(BlockedReason),
  )
}

pub type BeadError {
  BeadError(error_type: String, message: String, trace: option.Option(String))
}

pub type BlockedReason {
  BlockedReason(blocker_type: String, details: String, unblocks_when: String)
}

pub type FeedbackError {
  SessionNotFound(session_id: String)
  WriteError(path: String, message: String)
  ValidationError(message: String)
}

/// Mark a bead as executed with success status.
pub fn mark_bead_executed(
  session_id: String,
  bead_id: String,
  result: BeadResult,
  reason: String,
  duration_ms: Int,
) -> Result(Nil, FeedbackError) {
  mark_bead_with_details(
    session_id,
    bead_id,
    result,
    reason,
    duration_ms,
    option.None,
    option.None,
  )
}

/// Mark a bead as blocked with additional details.
pub fn mark_bead_blocked(
  session_id: String,
  bead_id: String,
  reason: String,
  blocker_type: String,
  blocker_details: String,
  unblocks_when: String,
  duration_ms: Int,
) -> Result(Nil, FeedbackError) {
  let blocked_reason =
    option.Some(BlockedReason(
      blocker_type: blocker_type,
      details: blocker_details,
      unblocks_when: unblocks_when,
    ))
  mark_bead_with_details(
    session_id,
    bead_id,
    Blocked,
    reason,
    duration_ms,
    option.None,
    blocked_reason,
  )
}

/// Mark a bead as failed with error details.
pub fn mark_bead_failed(
  session_id: String,
  bead_id: String,
  reason: String,
  error_type: String,
  error_message: String,
  trace: option.Option(String),
  duration_ms: Int,
) -> Result(Nil, FeedbackError) {
  let error =
    option.Some(BeadError(
      error_type: error_type,
      message: error_message,
      trace: trace,
    ))
  mark_bead_with_details(
    session_id,
    bead_id,
    Failed,
    reason,
    duration_ms,
    error,
    option.None,
  )
}

/// Internal: Mark bead with all details.
fn mark_bead_with_details(
  session_id: String,
  bead_id: String,
  result: BeadResult,
  reason: String,
  duration_ms: Int,
  error: option.Option(BeadError),
  blocked_by: option.Option(BlockedReason),
) -> Result(Nil, FeedbackError) {
  // Validate session ID format (alphanumeric + hyphen)
  case validate_session_id(session_id) {
    False -> Error(ValidationError("Invalid session ID format: " <> session_id))
    True -> {
      // Validate bead ID format
      case validate_bead_id(bead_id) {
        False -> Error(ValidationError("Invalid bead ID format: " <> bead_id))
        True -> {
          let feedback_path = ".intent/feedback-" <> session_id <> ".cue"
          let cue_entry =
            feedback_to_cue(
              bead_id,
              result,
              reason,
              duration_ms,
              error,
              blocked_by,
            )
          append_to_file(feedback_path, cue_entry)
        }
      }
    }
  }
}

/// Load all feedback for a session from CUE file.
pub fn load_feedback_for_session(
  session_id: String,
) -> Result(List(BeadFeedback), FeedbackError) {
  let feedback_path = ".intent/feedback-" <> session_id <> ".cue"
  case simplifile.read(feedback_path) {
    Ok(content) -> parse_feedback_from_cue(content)
    Error(_) -> {
      // File doesn't exist yet (empty feedback)
      Ok([])
    }
  }
}

// =============================================================================
// PRIVATE: CUE Serialization
// =============================================================================

fn feedback_to_cue(
  bead_id: String,
  result: BeadResult,
  reason: String,
  duration_ms: Int,
  error: option.Option(BeadError),
  blocked_by: option.Option(BlockedReason),
) -> String {
  let result_str = bead_result_to_string(result)
  let timestamp = current_iso8601_timestamp()
  let duration_str = string.inspect(duration_ms)

  let error_str = case error {
    option.None -> ""
    option.Some(err) -> {
      let trace_str = case err.trace {
        option.None -> ""
        option.Some(t) -> "\n\t\ttrace: \"\"\"" <> t <> "\"\"\""
      }
      "\n\terror: {\n\t\ttype: \""
      <> err.error_type
      <> "\"\n\t\tmessage: \""
      <> escape_cue_string(err.message)
      <> "\""
      <> trace_str
      <> "\n\t}"
    }
  }

  let blocked_str = case blocked_by {
    option.None -> ""
    option.Some(b) -> {
      "\n\tblocked_by: {\n\t\ttype: \""
      <> b.blocker_type
      <> "\"\n\t\tdetails: \""
      <> escape_cue_string(b.details)
      <> "\"\n\t\tunblocks_when: \""
      <> escape_cue_string(b.unblocks_when)
      <> "\"\n\t}"
    }
  }

  "{\n\tbead_id: \""
  <> bead_id
  <> "\"\n\tresult: \""
  <> result_str
  <> "\"\n\treason: \""
  <> escape_cue_string(reason)
  <> "\"\n\texecuted_at: \""
  <> timestamp
  <> "\"\n\tduration_ms: "
  <> duration_str
  <> error_str
  <> blocked_str
  <> "\n}\n"
}

fn bead_result_to_string(result: BeadResult) -> String {
  case result {
    Success -> "success"
    Failed -> "failed"
    Blocked -> "blocked"
    Skipped -> "skipped"
  }
}

fn escape_cue_string(s: String) -> String {
  s
  |> string.replace("\\", "\\\\")
  |> string.replace("\"", "\\\"")
  |> string.replace("\n", "\\n")
  |> string.replace("\t", "\\t")
}

// =============================================================================
// PRIVATE: File Operations
// =============================================================================

fn append_to_file(path: String, content: String) -> Result(Nil, FeedbackError) {
  // Read existing content (if file exists)
  let existing = case simplifile.read(path) {
    Ok(text) -> text
    Error(_) -> ""
  }

  // Append new content
  let updated = existing <> content

  // Write back (atomic replacement)
  simplifile.write(path, updated)
  |> result.map_error(fn(err) {
    let err_msg = case err {
      simplifile.Enoent -> "File not found"
      simplifile.Eacces -> "Permission denied"
      simplifile.Enospc -> "No space left on device"
      simplifile.Eio -> "I/O error"
      _ -> "Unknown error"
    }
    WriteError(path, err_msg)
  })
}

// =============================================================================
// PRIVATE: Validation
// =============================================================================

fn validate_session_id(id: String) -> Bool {
  let trimmed = string.trim(id)
  case string.length(trimmed) {
    0 -> False
    _ -> {
      trimmed
      |> string.to_graphemes
      |> list.all(fn(char) {
        case char {
          "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" -> True
          "a"
          | "b"
          | "c"
          | "d"
          | "e"
          | "f"
          | "g"
          | "h"
          | "i"
          | "j"
          | "k"
          | "l"
          | "m"
          | "n"
          | "o"
          | "p"
          | "q"
          | "r"
          | "s"
          | "t"
          | "u"
          | "v"
          | "w"
          | "x"
          | "y"
          | "z" -> True
          "A"
          | "B"
          | "C"
          | "D"
          | "E"
          | "F"
          | "G"
          | "H"
          | "I"
          | "J"
          | "K"
          | "L"
          | "M"
          | "N"
          | "O"
          | "P"
          | "Q"
          | "R"
          | "S"
          | "T"
          | "U"
          | "V"
          | "W"
          | "X"
          | "Y"
          | "Z" -> True
          "-" -> True
          "_" -> True
          _ -> False
        }
      })
    }
  }
}

fn validate_bead_id(id: String) -> Bool {
  // Format: PREFIX-NNN (e.g., AUTH-001, API-042)
  let trimmed = string.trim(id)
  case string.length(trimmed) >= 5 {
    False -> False
    True -> {
      case string.contains(trimmed, "-") {
        False -> False
        True -> {
          case string.split_once(trimmed, "-") {
            Error(Nil) -> False
            Ok(#(prefix, suffix)) -> {
              let prefix_ok =
                string.length(prefix) > 0 && string.length(prefix) <= 10
              let suffix_ok =
                string.length(suffix) == 3 && is_numeric_string(suffix)
              prefix_ok && suffix_ok
            }
          }
        }
      }
    }
  }
}

// =============================================================================
// PRIVATE: Helpers
// =============================================================================

/// Check if a string contains only numeric digits.
fn is_numeric_string(s: String) -> Bool {
  s
  |> string.to_graphemes
  |> list.all(fn(char) {
    case char {
      "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" -> True
      _ -> False
    }
  })
}

// =============================================================================
// PRIVATE: Parsing (Placeholder)
// =============================================================================

fn parse_feedback_from_cue(
  _content: String,
) -> Result(List(BeadFeedback), FeedbackError) {
  // TODO: Implement CUE parsing
  // For now, return empty list - CUE will be parsed via cue export
  Ok([])
}

// =============================================================================
// PRIVATE: Timestamp
// =============================================================================

@external(erlang, "intent_ffi", "current_timestamp")
fn current_iso8601_timestamp() -> String
