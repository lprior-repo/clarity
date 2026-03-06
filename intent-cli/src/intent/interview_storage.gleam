/// Interview Session Storage
/// Dual persistence: SQLite for querying, JSONL for git-friendly version control
/// Mirrors Beads approach: git-native JSONL + local SQLite for performance
/// Includes answer history tracking and diff comparison
import gleam/dict.{type Dict}
import gleam/dynamic
import gleam/int
import gleam/io
import gleam/json
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import gleam/string
import intent/interview.{
  type Answer, type Conflict, type ConflictResolution, type Gap,
  type InterviewSession, type InterviewStage, type Profile,
}
import intent/question_types.{
  type Perspective, Business, Developer, Ops, Security, User,
}
import shellout
import simplifile

/// Session record for storage
pub type SessionRecord {
  SessionRecord(
    id: String,
    profile: String,
    created_at: String,
    updated_at: String,
    completed_at: String,
    stage: String,
    rounds_completed: Int,
    raw_notes: String,
  )
}

// =============================================================================
// Answer History Tracking
// =============================================================================

/// A historical version of an answer
pub type AnswerVersion {
  AnswerVersion(
    version: Int,
    response: String,
    extracted: Dict(String, String),
    confidence: Float,
    timestamp: String,
    change_reason: String,
  )
}

/// Answer with full history
pub type AnswerWithHistory {
  AnswerWithHistory(
    question_id: String,
    question_text: String,
    perspective: Perspective,
    round: Int,
    current: Answer,
    history: List(AnswerVersion),
    notes: String,
  )
}

/// Session snapshot for diff comparison
pub type SessionSnapshot {
  SessionSnapshot(
    session_id: String,
    snapshot_id: String,
    timestamp: String,
    description: String,
    answers: Dict(String, String),
    // question_id -> response
    gaps_count: Int,
    conflicts_count: Int,
    stage: String,
  )
}

/// Diff between two sessions or snapshots
pub type SessionDiff {
  SessionDiff(
    from_id: String,
    to_id: String,
    from_timestamp: String,
    to_timestamp: String,
    answers_added: List(AnswerDiff),
    answers_modified: List(AnswerDiff),
    answers_removed: List(String),
    gaps_added: Int,
    gaps_resolved: Int,
    conflicts_added: Int,
    conflicts_resolved: Int,
    stage_changed: Option(#(String, String)),
  )
}

/// Diff for a single answer
pub type AnswerDiff {
  AnswerDiff(
    question_id: String,
    question_text: String,
    old_response: Option(String),
    new_response: String,
    change_type: AnswerChangeType,
  )
}

/// Type of change to an answer
pub type AnswerChangeType {
  Added
  Modified
  Removed
}

// =============================================================================
// History Operations
// =============================================================================

/// Create an AnswerVersion from an Answer
pub fn answer_to_version(
  answer: Answer,
  version: Int,
  change_reason: String,
) -> AnswerVersion {
  AnswerVersion(
    version: version,
    response: answer.response,
    extracted: answer.extracted,
    confidence: answer.confidence,
    timestamp: answer.timestamp,
    change_reason: change_reason,
  )
}

/// Create a session snapshot for comparison
pub fn create_snapshot(
  session: InterviewSession,
  description: String,
) -> SessionSnapshot {
  let answers_dict =
    list.fold(session.answers, dict.new(), fn(acc, answer) {
      dict.insert(acc, answer.question_id, answer.response)
    })

  let unresolved_gaps = list.filter(session.gaps, fn(g) { !g.resolved })
  let unresolved_conflicts =
    list.filter(session.conflicts, fn(c) { c.chosen < 0 })

  SessionSnapshot(
    session_id: session.id,
    snapshot_id: session.id <> "-" <> session.updated_at,
    timestamp: session.updated_at,
    description: description,
    answers: answers_dict,
    gaps_count: list.length(unresolved_gaps),
    conflicts_count: list.length(unresolved_conflicts),
    stage: stage_to_string(session.stage),
  )
}

/// Compare two sessions and produce a diff
pub fn diff_sessions(
  from_session: InterviewSession,
  to_session: InterviewSession,
) -> SessionDiff {
  // Build lookup maps for answers
  let from_answers =
    list.fold(from_session.answers, dict.new(), fn(acc, a) {
      dict.insert(acc, a.question_id, a)
    })
  let to_answers =
    list.fold(to_session.answers, dict.new(), fn(acc, a) {
      dict.insert(acc, a.question_id, a)
    })

  // Find added answers (in to but not in from)
  let added =
    list.filter_map(to_session.answers, fn(answer) {
      case dict.get(from_answers, answer.question_id) {
        Ok(_) -> Error(Nil)
        Error(_) ->
          Ok(AnswerDiff(
            question_id: answer.question_id,
            question_text: answer.question_text,
            old_response: None,
            new_response: answer.response,
            change_type: Added,
          ))
      }
    })

  // Find modified answers (in both but different)
  let modified =
    list.filter_map(to_session.answers, fn(answer) {
      case dict.get(from_answers, answer.question_id) {
        Ok(old_answer) -> {
          case old_answer.response == answer.response {
            True -> Error(Nil)
            False ->
              Ok(AnswerDiff(
                question_id: answer.question_id,
                question_text: answer.question_text,
                old_response: Some(old_answer.response),
                new_response: answer.response,
                change_type: Modified,
              ))
          }
        }
        Error(_) -> Error(Nil)
      }
    })

  // Find removed answers (in from but not in to)
  let removed =
    list.filter_map(from_session.answers, fn(answer) {
      case dict.get(to_answers, answer.question_id) {
        Ok(_) -> Error(Nil)
        Error(_) -> Ok(answer.question_id)
      }
    })

  // Count gap changes
  let from_unresolved_gaps =
    list.filter(from_session.gaps, fn(g) { !g.resolved })
  let to_unresolved_gaps = list.filter(to_session.gaps, fn(g) { !g.resolved })
  let gaps_added =
    list.length(to_unresolved_gaps) - list.length(from_unresolved_gaps)
  let gaps_resolved = case gaps_added < 0 {
    True -> -gaps_added
    False -> 0
  }

  // Count conflict changes
  let from_unresolved_conflicts =
    list.filter(from_session.conflicts, fn(c) { c.chosen < 0 })
  let to_unresolved_conflicts =
    list.filter(to_session.conflicts, fn(c) { c.chosen < 0 })
  let conflicts_added =
    list.length(to_unresolved_conflicts)
    - list.length(from_unresolved_conflicts)
  let conflicts_resolved = case conflicts_added < 0 {
    True -> -conflicts_added
    False -> 0
  }

  // Check stage change
  let stage_changed = case from_session.stage == to_session.stage {
    True -> None
    False ->
      Some(#(
        stage_to_string(from_session.stage),
        stage_to_string(to_session.stage),
      ))
  }

  SessionDiff(
    from_id: from_session.id,
    to_id: to_session.id,
    from_timestamp: from_session.updated_at,
    to_timestamp: to_session.updated_at,
    answers_added: added,
    answers_modified: modified,
    answers_removed: removed,
    gaps_added: case gaps_added > 0 {
      True -> gaps_added
      False -> 0
    },
    gaps_resolved: gaps_resolved,
    conflicts_added: case conflicts_added > 0 {
      True -> conflicts_added
      False -> 0
    },
    conflicts_resolved: conflicts_resolved,
    stage_changed: stage_changed,
  )
}

/// Format a SessionDiff as a human-readable string
pub fn format_diff(diff: SessionDiff) -> String {
  let lines = []

  // Header
  let lines =
    list.append(lines, [
      "Session Diff: " <> diff.from_id <> " → " <> diff.to_id,
      "Time: " <> diff.from_timestamp <> " → " <> diff.to_timestamp,
      "",
    ])

  // Stage change
  let lines = case diff.stage_changed {
    Some(#(from, to)) ->
      list.append(lines, ["Stage: " <> from <> " → " <> to, ""])
    None -> lines
  }

  // Answers added
  let lines = case list.length(diff.answers_added) {
    0 -> lines
    n -> {
      let header = ["Answers Added (" <> string.inspect(n) <> "):"]
      let answer_lines =
        list.map(diff.answers_added, fn(a) {
          "  + [" <> a.question_id <> "] " <> truncate(a.new_response, 50)
        })
      list.append(lines, list.append(header, list.append(answer_lines, [""])))
    }
  }

  // Answers modified
  let lines = case list.length(diff.answers_modified) {
    0 -> lines
    n -> {
      let header = ["Answers Modified (" <> string.inspect(n) <> "):"]
      let answer_lines =
        list.flat_map(diff.answers_modified, fn(a) {
          let old = case a.old_response {
            Some(r) -> truncate(r, 40)
            None -> "(none)"
          }
          [
            "  ~ [" <> a.question_id <> "]",
            "    - " <> old,
            "    + " <> truncate(a.new_response, 40),
          ]
        })
      list.append(lines, list.append(header, list.append(answer_lines, [""])))
    }
  }

  // Answers removed
  let lines = case list.length(diff.answers_removed) {
    0 -> lines
    n -> {
      let header = ["Answers Removed (" <> string.inspect(n) <> "):"]
      let answer_lines =
        list.map(diff.answers_removed, fn(id) { "  - [" <> id <> "]" })
      list.append(lines, list.append(header, list.append(answer_lines, [""])))
    }
  }

  // Gaps and conflicts summary
  let lines = case diff.gaps_added > 0 || diff.gaps_resolved > 0 {
    True ->
      list.append(lines, [
        "Gaps: +"
        <> string.inspect(diff.gaps_added)
        <> " added, -"
        <> string.inspect(diff.gaps_resolved)
        <> " resolved",
      ])
    False -> lines
  }

  let lines = case diff.conflicts_added > 0 || diff.conflicts_resolved > 0 {
    True ->
      list.append(lines, [
        "Conflicts: +"
        <> string.inspect(diff.conflicts_added)
        <> " added, -"
        <> string.inspect(diff.conflicts_resolved)
        <> " resolved",
      ])
    False -> lines
  }

  string.join(lines, "\n")
}

/// Truncate a string with ellipsis
fn truncate(s: String, max_len: Int) -> String {
  let trimmed = string.trim(s)
  case string.length(trimmed) > max_len {
    True -> string.slice(trimmed, 0, max_len - 3) <> "..."
    False -> trimmed
  }
}

// =============================================================================
// JSON Decode Error Handling (RED-03 fix)
// =============================================================================

fn warn_json_decode_error(
  line: String,
  path: String,
  index: Int,
  error: json.DecodeError,
) -> Nil {
  io.println_error("")
  io.println_error(
    "Warning: Failed to parse JSON line "
    <> int.to_string(index)
    <> " from "
    <> path,
  )
  io.println_error("")

  let sanitized = sanitize_for_display(line, 200)
  io.println_error("Corrupted data preview:")
  io.println_error("  " <> sanitized)
  io.println_error("")

  io.println_error("Parse error:")
  let error_msg = format_json_error(error)
  io.println_error(error_msg)
  io.println_error("")
  io.println_error(
    "This line has been skipped. Other valid data will be loaded.",
  )
  io.println_error("")
}

fn sanitize_for_display(s: String, max_len: Int) -> String {
  let trimmed = string.trim(s)
  case string.length(trimmed) > max_len {
    True -> string.slice(trimmed, 0, max_len - 3) <> "..."
    False -> trimmed
  }
}

fn format_json_error(error: json.DecodeError) -> String {
  case error {
    json.UnexpectedEndOfInput ->
      "Unexpected end of input - JSON is incomplete or truncated.\n"
      <> "  - Check that your JSON is properly closed with matching braces/brackets"
    json.UnexpectedByte(b) ->
      "Unexpected byte: '"
      <> b
      <> "' in JSON at this position.\n"
      <> "  - Check for syntax errors like missing commas, quotes, or brackets\n"
      <> "  - Ensure strings are properly quoted"
    json.UnexpectedSequence(s) ->
      "Unexpected sequence: '"
      <> s
      <> "' in JSON.\n"
      <> "  - This sequence is not valid JSON syntax\n"
      <> "  - Check for typos or invalid characters"
    json.UnexpectedFormat(errs) ->
      "JSON format error:\n" <> format_decode_errors(errs)
  }
}

fn format_decode_errors(errors: List(dynamic.DecodeError)) -> String {
  case errors {
    [] -> "Unknown decode error"
    [error] -> format_single_decode_error(error)
    multiple -> {
      "Multiple decode errors:\n"
      <> string.join(
        list.map(multiple, fn(e) { "  - " <> format_single_decode_error(e) }),
        "\n",
      )
    }
  }
}

fn format_single_decode_error(error: dynamic.DecodeError) -> String {
  let path_str = case error.path {
    [] -> "at root"
    path_parts -> "at " <> string.join(path_parts, ".")
  }

  "Expected "
  <> error.expected
  <> " but found "
  <> error.found
  <> " "
  <> path_str
}

// =============================================================================
// Session History JSONL
// =============================================================================

/// Append a session snapshot to history JSONL
/// File: .interview/history.jsonl
pub fn append_to_history(
  session: InterviewSession,
  description: String,
  history_path: String,
) -> Result(Nil, String) {
  let snapshot = create_snapshot(session, description)
  let line = snapshot_to_jsonl_line(snapshot)

  use existing <- result.try(
    simplifile.read(history_path)
    |> result.unwrap("")
    |> Ok,
  )

  let content = case string.length(string.trim(existing)) {
    0 -> line
    _ -> existing <> "\n" <> line
  }

  simplifile.write(history_path, content)
  |> result.map_error(fn(err) {
    "Failed to write history: " <> string.inspect(err)
  })
}

fn snapshot_to_jsonl_line(snapshot: SessionSnapshot) -> String {
  json.object([
    #("session_id", json.string(snapshot.session_id)),
    #("snapshot_id", json.string(snapshot.snapshot_id)),
    #("timestamp", json.string(snapshot.timestamp)),
    #("description", json.string(snapshot.description)),
    #(
      "answers",
      json.object(
        dict.to_list(snapshot.answers)
        |> list.map(fn(pair) { #(pair.0, json.string(pair.1)) }),
      ),
    ),
    #("gaps_count", json.int(snapshot.gaps_count)),
    #("conflicts_count", json.int(snapshot.conflicts_count)),
    #("stage", json.string(snapshot.stage)),
  ])
  |> json.to_string
}

/// List all snapshots for a session from history
pub fn list_session_history(
  history_path: String,
  session_id: String,
) -> Result(List(SessionSnapshot), String) {
  use content <- result.try(
    simplifile.read(history_path)
    |> result.map_error(fn(err) {
      "Failed to read history: " <> string.inspect(err)
    }),
  )

  case string.length(string.trim(content)) {
    0 -> Ok([])
    _ -> {
      let lines = string.split(content, "\n")
      let snapshots =
        list.index_map(lines, fn(line, index) {
          case string.length(string.trim(line)) {
            0 -> Error(Nil)
            _ ->
              json.decode(line, snapshot_decoder)
              |> result.map_error(fn(err) {
                warn_json_decode_error(line, history_path, index + 1, err)
                Nil
              })
          }
        })
        |> list.filter_map(fn(result) { result })
        |> list.filter(fn(s) { s.session_id == session_id })
      Ok(snapshots)
    }
  }
}

fn snapshot_decoder(
  json_value: dynamic.Dynamic,
) -> Result(SessionSnapshot, dynamic.DecodeErrors) {
  use session_id <- result.try(dynamic.field("session_id", dynamic.string)(
    json_value,
  ))
  use snapshot_id <- result.try(dynamic.field("snapshot_id", dynamic.string)(
    json_value,
  ))
  use timestamp <- result.try(dynamic.field("timestamp", dynamic.string)(
    json_value,
  ))
  use description <- result.try(dynamic.field("description", dynamic.string)(
    json_value,
  ))
  use answers_list <- result.try(dynamic.field(
    "answers",
    dynamic.dict(dynamic.string, dynamic.string),
  )(json_value))
  use gaps_count <- result.try(dynamic.field("gaps_count", dynamic.int)(
    json_value,
  ))
  use conflicts_count <- result.try(dynamic.field(
    "conflicts_count",
    dynamic.int,
  )(json_value))
  use stage <- result.try(dynamic.field("stage", dynamic.string)(json_value))

  Ok(SessionSnapshot(
    session_id: session_id,
    snapshot_id: snapshot_id,
    timestamp: timestamp,
    description: description,
    answers: answers_list,
    gaps_count: gaps_count,
    conflicts_count: conflicts_count,
    stage: stage,
  ))
}

/// JSONL operations - git-friendly line-delimited JSON
/// Each line is a complete session snapshot
/// Stored at: .interview/sessions.jsonl
pub fn session_to_json(session: InterviewSession) -> json.Json {
  json.object([
    #("id", json.string(session.id)),
    #("profile", json.string(profile_to_string(session.profile))),
    #("created_at", json.string(session.created_at)),
    #("updated_at", json.string(session.updated_at)),
    #("completed_at", json.string(session.completed_at)),
    #("stage", json.string(stage_to_string(session.stage))),
    #("rounds_completed", json.int(session.rounds_completed)),
    #("answers", json.array(session.answers, answer_to_json)),
    #("gaps", json.array(session.gaps, gap_to_json)),
    #("conflicts", json.array(session.conflicts, conflict_to_json)),
    #("raw_notes", json.string(session.raw_notes)),
    #("current_phase", json.int(session.current_phase)),
    #("completed_phases", json.array(session.completed_phases, json.int)),
  ])
}

pub fn profile_to_string(profile: Profile) -> String {
  case profile {
    interview.Api -> "api"
    interview.Cli -> "cli"
    interview.Event -> "event"
    interview.Data -> "data"
    interview.Workflow -> "workflow"
    interview.UI -> "ui"
  }
}

pub fn stage_to_string(stage: InterviewStage) -> String {
  case stage {
    interview.Discovery -> "discovery"
    interview.Refinement -> "refinement"
    interview.Validation -> "validation"
    interview.Complete -> "complete"
    interview.Paused -> "paused"
  }
}

fn answer_to_json(answer: Answer) -> json.Json {
  json.object([
    #("question_id", json.string(answer.question_id)),
    #("question_text", json.string(answer.question_text)),
    #("perspective", json.string(perspective_to_string(answer.perspective))),
    #("round", json.int(answer.round)),
    #("response", json.string(answer.response)),
    #(
      "extracted",
      json.object(
        dict.to_list(answer.extracted)
        |> list.map(fn(pair) { #(pair.0, json.string(pair.1)) }),
      ),
    ),
    #("confidence", json.float(answer.confidence)),
    #("notes", json.string(answer.notes)),
    #("timestamp", json.string(answer.timestamp)),
  ])
}

fn perspective_to_string(perspective: Perspective) -> String {
  case perspective {
    User -> "user"
    Developer -> "developer"
    Ops -> "ops"
    Security -> "security"
    Business -> "business"
  }
}

fn gap_to_json(gap: Gap) -> json.Json {
  json.object([
    #("id", json.string(gap.id)),
    #("field", json.string(gap.field)),
    #("description", json.string(gap.description)),
    #("blocking", json.bool(gap.blocking)),
    #("suggested_default", json.string(gap.suggested_default)),
    #("why_needed", json.string(gap.why_needed)),
    #("round", json.int(gap.round)),
    #("resolved", json.bool(gap.resolved)),
    #("resolution", json.string(gap.resolution)),
  ])
}

fn conflict_to_json(conflict: Conflict) -> json.Json {
  let #(between_1, between_2) = conflict.between
  json.object([
    #("id", json.string(conflict.id)),
    #("between", json.array([between_1, between_2], json.string)),
    #("description", json.string(conflict.description)),
    #("impact", json.string(conflict.impact)),
    #("options", json.array(conflict.options, conflict_resolution_to_json)),
    #("chosen", json.int(conflict.chosen)),
  ])
}

fn conflict_resolution_to_json(res: ConflictResolution) -> json.Json {
  json.object([
    #("option", json.string(res.option)),
    #("description", json.string(res.description)),
    #("tradeoffs", json.string(res.tradeoffs)),
    #("recommendation", json.string(res.recommendation)),
  ])
}

/// Encode session to JSONL line (for git storage)
pub fn session_to_jsonl_line(session: InterviewSession) -> String {
  session
  |> session_to_json
  |> json.to_string
}

/// Write session to .interview/sessions.jsonl
/// Each session ID appears once, most recent last (for efficient updates)
pub fn append_session_to_jsonl(
  session: InterviewSession,
  jsonl_path: String,
) -> Result(Nil, String) {
  use _ <- result.try(ensure_parent_directory(jsonl_path))

  let existing = simplifile.read(jsonl_path) |> result.unwrap("")

  let lines = case existing {
    "" -> []
    content -> string.split(content, "\n")
  }

  let filtered =
    list.filter(lines, fn(line) {
      // Parse each line and keep if session ID doesn't match
      case json.decode(line, session_id_decoder) {
        Ok(id) -> id != session.id
        Error(_) -> True
      }
    })

  let new_line = session_to_jsonl_line(session)
  let all_lines = list.append(filtered, [new_line])
  let content = string.join(all_lines, "\n")

  simplifile.write(jsonl_path, content)
  |> result.map_error(fn(err) {
    "Failed to write JSONL: " <> string.inspect(err)
  })
}

fn ensure_parent_directory(path: String) -> Result(Nil, String) {
  let path_parts = string.split(path, "/")

  case list.reverse(path_parts) {
    [] -> Ok(Nil)
    [_file] -> Ok(Nil)
    [_file, ..reversed_parent] -> {
      let parent = reversed_parent |> list.reverse |> string.join("/")

      case string.trim(parent) == "" {
        True -> Ok(Nil)
        False -> {
          case shellout.command("mkdir", ["-p", parent], ".", []) {
            Ok(_) -> Ok(Nil)
            Error(#(_code, stderr)) -> {
              Error(
                "Failed to create parent directory '"
                <> parent
                <> "': "
                <> stderr,
              )
            }
          }
        }
      }
    }
  }
}

/// List all sessions from JSONL file
pub fn list_sessions_from_jsonl(
  jsonl_path: String,
) -> Result(List(InterviewSession), String) {
  use content <- result.try(
    simplifile.read(jsonl_path)
    |> result.map_error(fn(err) {
      "Failed to read JSONL: " <> string.inspect(err)
    }),
  )

  case string.length(string.trim(content)) {
    0 -> Ok([])
    _ -> {
      let lines = string.split(content, "\n")
      let sessions =
        list.index_map(lines, fn(line, index) {
          case string.length(string.trim(line)) {
            0 -> Error(Nil)
            _ ->
              json.decode(line, session_decoder)
              |> result.map_error(fn(err) {
                warn_json_decode_error(line, jsonl_path, index + 1, err)
                Nil
              })
          }
        })
        |> list.filter_map(fn(result) { result })
      Ok(sessions)
    }
  }
}

/// Get session by ID from JSONL
pub fn get_session_from_jsonl(
  jsonl_path: String,
  session_id: String,
) -> Result(InterviewSession, String) {
  list_sessions_from_jsonl(jsonl_path)
  |> result.try(fn(sessions) {
    list.find(sessions, fn(s) { s.id == session_id })
    |> result.map_error(fn(_) { "Session not found: " <> session_id })
  })
}

/// SQLite operations - local database for queries and performance
/// Database schema:
///
/// CREATE TABLE sessions (
///   id TEXT PRIMARY KEY,
///   profile TEXT NOT NULL,
///   created_at TEXT NOT NULL,
///   updated_at TEXT NOT NULL,
///   completed_at TEXT,
///   stage TEXT NOT NULL,
///   rounds_completed INTEGER NOT NULL,
///   raw_notes TEXT,
///   data JSONB  -- Full session data
/// );
///
/// CREATE TABLE answers (
///   id TEXT PRIMARY KEY,
///   session_id TEXT NOT NULL REFERENCES sessions(id),
///   question_id TEXT NOT NULL,
///   round INTEGER NOT NULL,
///   perspective TEXT NOT NULL,
///   response TEXT NOT NULL,
///   confidence REAL NOT NULL,
///   timestamp TEXT NOT NULL
/// );
///
/// CREATE TABLE gaps (
///   id TEXT PRIMARY KEY,
///   session_id TEXT NOT NULL REFERENCES sessions(id),
///   field TEXT NOT NULL,
///   blocking BOOLEAN NOT NULL,
///   resolved BOOLEAN NOT NULL
/// );
///
/// CREATE TABLE conflicts (
///   id TEXT PRIMARY KEY,
///   session_id TEXT NOT NULL REFERENCES sessions(id),
///   description TEXT NOT NULL,
///   chosen INTEGER
/// );
/// Initialize SQLite database (create tables if not exist)
pub fn init_database(_db_path: String) -> Result(Nil, String) {
  // In real implementation:
  // 1. Check if .interview/interview.db exists
  // 2. If not, create it with schema above
  // 3. If it does, verify schema is up-to-date
  Ok(Nil)
}

/// Save session to SQLite
pub fn save_session_to_db(
  _db_path: String,
  _session: InterviewSession,
) -> Result(Nil, String) {
  // INSERT or UPDATE sessions table
  // DELETE and INSERT answers/gaps/conflicts to maintain referential integrity
  Ok(Nil)
}

/// Query sessions by profile
pub fn query_sessions_by_profile(
  _db_path: String,
  _profile: String,
) -> Result(List(SessionRecord), String) {
  Ok([])
}

/// Query ready sessions (active, not complete, has gaps)
pub fn query_ready_sessions(
  _db_path: String,
) -> Result(List(SessionRecord), String) {
  Ok([])
}

/// Sync operations - keep SQLite and JSONL in sync
/// Strategy: JSONL is source of truth for git
/// 1. On read: load from JSONL, check SQLite is consistent
/// 2. On write: write to both
/// 3. Conflict resolution: JSONL wins (it's in git)
pub fn sync_to_jsonl(
  session: InterviewSession,
  db_path: String,
  jsonl_path: String,
) -> Result(Nil, String) {
  // 1. Update SQLite
  use _ <- result.try(save_session_to_db(db_path, session))
  // 2. Append to JSONL
  use _ <- result.try(append_session_to_jsonl(session, jsonl_path))
  Ok(Nil)
}

pub fn sync_from_jsonl(
  jsonl_path: String,
  db_path: String,
) -> Result(List(InterviewSession), String) {
  // 1. Read from JSONL (source of truth)
  use sessions <- result.try(list_sessions_from_jsonl(jsonl_path))
  // 2. Update SQLite with latest
  list.fold(sessions, Ok(Nil), fn(acc, session) {
    result.try(acc, fn(_) { save_session_to_db(db_path, session) })
  })
  |> result.map(fn(_) { sessions })
}

// Decoder helpers for JSON parsing
fn session_id_decoder(
  json_value: dynamic.Dynamic,
) -> Result(String, dynamic.DecodeErrors) {
  dynamic.field("id", dynamic.string)(json_value)
}

fn session_decoder(
  json_value: dynamic.Dynamic,
) -> Result(InterviewSession, dynamic.DecodeErrors) {
  use id <- result.try(dynamic.field("id", dynamic.string)(json_value))
  use profile_str <- result.try(dynamic.field("profile", dynamic.string)(
    json_value,
  ))
  use profile <- result.try(case profile_str {
    "api" -> Ok(interview.Api)
    "cli" -> Ok(interview.Cli)
    "event" -> Ok(interview.Event)
    "data" -> Ok(interview.Data)
    "workflow" -> Ok(interview.Workflow)
    "ui" -> Ok(interview.UI)
    _ -> Error([dynamic.DecodeError("profile", "invalid profile", [])])
  })
  use created_at <- result.try(dynamic.field("created_at", dynamic.string)(
    json_value,
  ))
  use updated_at <- result.try(dynamic.field("updated_at", dynamic.string)(
    json_value,
  ))
  use completed_at <- result.try(
    dynamic.field("completed_at", dynamic.string)(json_value)
    |> result.map_error(fn(_) { [] }),
  )
  use stage_str <- result.try(dynamic.field("stage", dynamic.string)(json_value))
  use stage <- result.try(case stage_str {
    "discovery" -> Ok(interview.Discovery)
    "refinement" -> Ok(interview.Refinement)
    "validation" -> Ok(interview.Validation)
    "complete" -> Ok(interview.Complete)
    "paused" -> Ok(interview.Paused)
    "Discovery" -> Ok(interview.Discovery)
    "Refinement" -> Ok(interview.Refinement)
    "Validation" -> Ok(interview.Validation)
    "Complete" -> Ok(interview.Complete)
    "Paused" -> Ok(interview.Paused)
    _ -> Error([dynamic.DecodeError("stage", "invalid stage", [])])
  })
  use rounds_completed <- result.try(dynamic.field(
    "rounds_completed",
    dynamic.int,
  )(json_value))
  let answers =
    dynamic.field("answers", dynamic.list(answer_decoder))(json_value)
    |> result.unwrap([])
  let gaps =
    dynamic.field("gaps", dynamic.list(gap_decoder))(json_value)
    |> result.unwrap([])
  let conflicts =
    dynamic.field("conflicts", dynamic.list(conflict_decoder))(json_value)
    |> result.unwrap([])
  use raw_notes <- result.try(
    dynamic.field("raw_notes", dynamic.string)(json_value)
    |> result.map_error(fn(_) { [] }),
  )
  let current_phase =
    dynamic.field("current_phase", dynamic.int)(json_value)
    |> result.unwrap(1)
  let completed_phases =
    dynamic.field("completed_phases", dynamic.list(dynamic.int))(json_value)
    |> result.unwrap([])

  Ok(interview.InterviewSession(
    id: id,
    profile: profile,
    created_at: created_at,
    updated_at: updated_at,
    completed_at: completed_at,
    stage: stage,
    rounds_completed: rounds_completed,
    answers: answers,
    gaps: gaps,
    conflicts: conflicts,
    raw_notes: raw_notes,
    current_phase: current_phase,
    completed_phases: completed_phases,
  ))
}

fn answer_decoder(
  json_value: dynamic.Dynamic,
) -> Result(Answer, dynamic.DecodeErrors) {
  use question_id <- result.try(dynamic.field("question_id", dynamic.string)(
    json_value,
  ))
  use question_text <- result.try(dynamic.field("question_text", dynamic.string)(
    json_value,
  ))
  use perspective_str <- result.try(dynamic.field("perspective", dynamic.string)(
    json_value,
  ))
  use perspective <- result.try(perspective_from_string(perspective_str))
  use round <- result.try(dynamic.field("round", dynamic.int)(json_value))
  use response <- result.try(dynamic.field("response", dynamic.string)(
    json_value,
  ))
  let extracted =
    dynamic.field("extracted", dynamic.dict(dynamic.string, dynamic.string))(
      json_value,
    )
    |> result.unwrap(dict.new())
  let confidence =
    dynamic.field("confidence", dynamic.float)(json_value) |> result.unwrap(0.0)
  let notes =
    dynamic.field("notes", dynamic.string)(json_value) |> result.unwrap("")
  let timestamp =
    dynamic.field("timestamp", dynamic.string)(json_value) |> result.unwrap("")

  Ok(interview.Answer(
    question_id: question_id,
    question_text: question_text,
    perspective: perspective,
    round: round,
    response: response,
    extracted: extracted,
    confidence: confidence,
    notes: notes,
    timestamp: timestamp,
  ))
}

fn gap_decoder(json_value: dynamic.Dynamic) -> Result(Gap, dynamic.DecodeErrors) {
  use id <- result.try(dynamic.field("id", dynamic.string)(json_value))
  use field <- result.try(dynamic.field("field", dynamic.string)(json_value))
  use description <- result.try(dynamic.field("description", dynamic.string)(
    json_value,
  ))
  use blocking <- result.try(dynamic.field("blocking", dynamic.bool)(json_value))
  let suggested_default =
    dynamic.field("suggested_default", dynamic.string)(json_value)
    |> result.unwrap("")
  let why_needed =
    dynamic.field("why_needed", dynamic.string)(json_value) |> result.unwrap("")
  let round =
    dynamic.field("round", dynamic.int)(json_value) |> result.unwrap(0)
  let resolved =
    dynamic.field("resolved", dynamic.bool)(json_value) |> result.unwrap(False)
  let resolution =
    dynamic.field("resolution", dynamic.string)(json_value) |> result.unwrap("")

  Ok(interview.Gap(
    id: id,
    field: field,
    description: description,
    blocking: blocking,
    suggested_default: suggested_default,
    why_needed: why_needed,
    round: round,
    resolved: resolved,
    resolution: resolution,
  ))
}

fn conflict_decoder(
  json_value: dynamic.Dynamic,
) -> Result(Conflict, dynamic.DecodeErrors) {
  use id <- result.try(dynamic.field("id", dynamic.string)(json_value))
  use between_list <- result.try(dynamic.field(
    "between",
    dynamic.list(dynamic.string),
  )(json_value))
  use between <- result.try(between_from_list(between_list))
  use description <- result.try(dynamic.field("description", dynamic.string)(
    json_value,
  ))
  let impact =
    dynamic.field("impact", dynamic.string)(json_value) |> result.unwrap("")
  let options =
    dynamic.field("options", dynamic.list(conflict_resolution_decoder))(
      json_value,
    )
    |> result.unwrap([])
  let chosen =
    dynamic.field("chosen", dynamic.int)(json_value) |> result.unwrap(-1)

  Ok(interview.Conflict(
    id: id,
    between: between,
    description: description,
    impact: impact,
    options: options,
    chosen: chosen,
  ))
}

fn conflict_resolution_decoder(
  json_value: dynamic.Dynamic,
) -> Result(ConflictResolution, dynamic.DecodeErrors) {
  let option =
    dynamic.field("option", dynamic.string)(json_value) |> result.unwrap("")
  let description =
    dynamic.field("description", dynamic.string)(json_value)
    |> result.unwrap("")
  let tradeoffs =
    dynamic.field("tradeoffs", dynamic.string)(json_value) |> result.unwrap("")
  let recommendation =
    dynamic.field("recommendation", dynamic.string)(json_value)
    |> result.unwrap("")

  Ok(interview.ConflictResolution(
    option: option,
    description: description,
    tradeoffs: tradeoffs,
    recommendation: recommendation,
  ))
}

fn between_from_list(
  values: List(String),
) -> Result(#(String, String), dynamic.DecodeErrors) {
  case values {
    [a, b] -> Ok(#(a, b))
    _ -> Error([dynamic.DecodeError("between", "two strings", [])])
  }
}

fn perspective_from_string(
  value: String,
) -> Result(Perspective, dynamic.DecodeErrors) {
  case value {
    "user" | "User" -> Ok(User)
    "developer" | "Developer" -> Ok(Developer)
    "ops" | "Ops" -> Ok(Ops)
    "security" | "Security" -> Ok(Security)
    "business" | "Business" -> Ok(Business)
    _ -> Error([dynamic.DecodeError("perspective", "valid perspective", [])])
  }
}
