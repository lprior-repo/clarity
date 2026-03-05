//// Execution Plan Display for Intent CLI
////
//// Reads beads from .intent/session-{id}.cue, builds dependency graph,
//// outputs execution phases in human-readable or JSON format.
////
//// Implements #ExecutionPlan from schema/beads.cue:
//// - Groups beads into dependency-ordered phases
//// - Calculates total effort and risk assessment
//// - Supports both human (ASCII tree) and JSON output

import gleam/dict.{type Dict}
import gleam/dynamic
import gleam/int
import gleam/json
import gleam/list

// Option not used - all fields required
import gleam/option
import gleam/result

// Set not needed - using dict for lookups
import gleam/string
import intent/security
import shellout
import simplifile

// =============================================================================
// TYPES - Matching #ExecutionPlan from schema/beads.cue
// =============================================================================

/// Execution plan computed from session beads
pub type ExecutionPlan {
  ExecutionPlan(
    session_id: String,
    generated_at: String,
    phases: List(ExecutionPhase),
    total_beads: Int,
    total_effort: String,
    risk: RiskLevel,
    blockers: List(String),
  )
}

/// A phase of execution containing beads that can run in parallel
pub type ExecutionPhase {
  ExecutionPhase(
    phase_number: Int,
    title: String,
    beads: List(PlanBead),
    can_parallel: Bool,
    effort: String,
  )
}

/// Risk level assessment
pub type RiskLevel {
  Low
  Medium
  High
  Critical
}

/// Bead effort estimate
pub type Effort {
  Effort5min
  Effort10min
  Effort15min
  Effort20min
  Effort30min
}

/// Bead status
pub type BeadStatus {
  Pending
  InProgress
  Blocked
  Completed
  Failed
}

/// Minimal bead representation for planning
pub type PlanBead {
  PlanBead(
    id: String,
    title: String,
    requires: List(String),
    effort: Effort,
    status: BeadStatus,
  )
}

pub type PlanError {
  SessionNotFound(session_id: String)
  ParseError(message: String)
  CueExportError(message: String)
  JsonParseError(message: String)
  CyclicDependency(beads: List(String))
  MissingDependency(bead: String, missing: String)
}

pub fn effort_to_label(effort: Effort) -> String {
  case effort {
    Effort5min -> "5min"
    Effort10min -> "10min"
    Effort15min -> "15min"
    Effort20min -> "20min"
    Effort30min -> "30min"
  }
}

pub fn bead_status_to_string(status: BeadStatus) -> String {
  case status {
    Pending -> "pending"
    InProgress -> "in_progress"
    Blocked -> "blocked"
    Completed -> "completed"
    Failed -> "failed"
  }
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Load beads from session file and compute execution plan
pub fn compute_plan(session_id: String) -> Result(ExecutionPlan, PlanError) {
  let session_path = ".intent/session-" <> session_id <> ".cue"

  case simplifile.verify_is_file(session_path) {
    Ok(True) -> {
      use beads <- result.try(parse_beads_from_cue(session_path))
      use phases <- result.try(detect_dependency_graph(beads))

      let total_effort = calculate_total_effort(beads)
      let risk = assess_risk(beads, phases)
      let blockers = find_blockers(beads)
      let timestamp = current_iso8601_timestamp()

      Ok(ExecutionPlan(
        session_id: session_id,
        generated_at: timestamp,
        phases: phases,
        total_beads: list.length(beads),
        total_effort: total_effort,
        risk: risk,
        blockers: blockers,
      ))
    }
    _ -> Error(SessionNotFound(session_id))
  }
}

/// Build execution phases from bead dependencies using topological sort
pub fn detect_dependency_graph(
  beads: List(PlanBead),
) -> Result(List(ExecutionPhase), PlanError) {
  // Build a map from id to bead
  let by_id =
    beads
    |> list.map(fn(b) { #(b.id, b) })
    |> dict.from_list

  // Check all dependencies exist
  use _ <- result.try(validate_dependencies(beads, by_id))

  // Group beads into phases based on dependency levels
  build_phases(beads, by_id)
}

/// Format plan as human-readable ASCII tree
pub fn format_plan_human(plan: ExecutionPlan) -> String {
  let header =
    "╔══════════════════════════════════════════════════════════════╗\n"
    <> "║                    EXECUTION PLAN                            ║\n"
    <> "╠══════════════════════════════════════════════════════════════╣\n"
    <> "║ Session: "
    <> pad_right(plan.session_id, 51)
    <> "║\n"
    <> "║ Generated: "
    <> pad_right(plan.generated_at, 49)
    <> "║\n"
    <> "║ Total Beads: "
    <> pad_right(int.to_string(plan.total_beads), 47)
    <> "║\n"
    <> "║ Total Effort: "
    <> pad_right(plan.total_effort, 46)
    <> "║\n"
    <> "║ Risk Level: "
    <> pad_right(risk_to_string(plan.risk), 48)
    <> "║\n"
    <> "╚══════════════════════════════════════════════════════════════╝\n\n"

  let blockers_section = case list.is_empty(plan.blockers) {
    True -> ""
    False ->
      "⚠ BLOCKERS:\n"
      <> {
        plan.blockers |> list.map(fn(b) { "  • " <> b }) |> string.join("\n")
      }
      <> "\n\n"
  }

  let phases_section =
    plan.phases
    |> list.map(format_phase_human)
    |> string.join("\n")

  header <> blockers_section <> phases_section
}

/// Format plan as JSON (compatible with cue export format)
pub fn format_plan_json(plan: ExecutionPlan) -> String {
  let phases_json =
    plan.phases
    |> list.map(format_phase_json)
    |> string.join(",\n    ")

  let blockers_json =
    plan.blockers
    |> list.map(fn(b) { "\"" <> escape_json_string(b) <> "\"" })
    |> string.join(", ")

  "{\n"
  <> "  \"session_id\": \""
  <> escape_json_string(plan.session_id)
  <> "\",\n"
  <> "  \"generated_at\": \""
  <> plan.generated_at
  <> "\",\n"
  <> "  \"phases\": [\n    "
  <> phases_json
  <> "\n  ],\n"
  <> "  \"total_beads\": "
  <> int.to_string(plan.total_beads)
  <> ",\n"
  <> "  \"total_effort\": \""
  <> plan.total_effort
  <> "\",\n"
  <> "  \"risk\": \""
  <> risk_to_string(plan.risk)
  <> "\",\n"
  <> "  \"blockers\": ["
  <> blockers_json
  <> "]\n"
  <> "}"
}

/// Format plan as AI-friendly JSON with bead details
pub fn format_plan_ai(plan: ExecutionPlan) -> String {
  let phases_json =
    plan.phases
    |> list.map(fn(phase) {
      json.object([
        #("phase_number", json.int(phase.phase_number)),
        #("title", json.string(phase.title)),
        #("can_parallel", json.bool(phase.can_parallel)),
        #("effort", json.string(phase.effort)),
        #("beads", json.array(phase.beads, plan_bead_to_json)),
      ])
    })

  let handoff =
    json.object([
      #(
        "why_this_plan",
        json.string(
          "Execution plan for session. Review phases and execute beads in order.",
        ),
      ),
      #("changed_since_last_run", json.string("Plan regenerated.")),
    ])

  let acceptance_tests =
    plan.blockers
    |> list.map(fn(blocker) { json.string("Verify: " <> blocker) })
    |> json.array(of: fn(x) { x })

  let next_commands =
    ["intent plan " <> plan.session_id, "intent plan-next " <> plan.session_id]
    |> list.map(fn(cmd) { json.string(cmd) })
    |> json.array(of: fn(x) { x })

  json.object([
    #("action", json.string("emit_plan")),
    #("contract_version", json.string("1.0")),
    #("session_id", json.string(plan.session_id)),
    #(
      "session",
      json.object([
        #("id", json.string(plan.session_id)),
        #("profile", json.string("cli")),
        #("created_at", json.string(plan.generated_at)),
        #("updated_at", json.string(plan.generated_at)),
        #("stage", json.string("complete")),
      ]),
    ),
    #("phases", json.array(phases_json, of: fn(x) { x })),
    #("planning_focus", json.string("Execute the plan in phases")),
    #(
      "assumptions",
      json.array(
        [
          json.string("Session beads are valid and complete"),
          json.string("Phase ordering respects dependencies"),
          json.string("Bead execution order is sequential within phases"),
        ],
        of: fn(x) { x },
      ),
    ),
    #("open_questions", json.array([], of: fn(x) { x })),
    #(
      "risks",
      json.array(
        [
          json.string("Some beads may fail - use beads-regenerate to fix"),
          json.string("Parallel execution may reveal race conditions"),
        ],
        of: fn(x) { x },
      ),
    ),
    #("acceptance_tests", acceptance_tests),
    #("handoff", handoff),
    #("next_commands", next_commands),
    #(
      "plan",
      json.object([
        #("session_id", json.string(plan.session_id)),
        #("total_beads", json.int(plan.total_beads)),
        #("total_effort", json.string(plan.total_effort)),
        #("risk", json.string(risk_to_string(plan.risk))),
        #("phase_count", json.int(list.length(plan.phases))),
        #("critical_path_phases", json.int(list.length(plan.phases))),
        #("blockers", json.array(plan.blockers, json.string)),
      ]),
    ),
  ])
  |> json.to_string
}

/// Format error as human-readable string
pub fn format_error(error: PlanError) -> String {
  case error {
    SessionNotFound(id) ->
      "Session not found: "
      <> id
      <> "\n"
      <> "Expected file: .intent/session-"
      <> id
      <> ".cue"
    ParseError(msg) -> "Failed to parse session: " <> msg
    CueExportError(msg) -> "CUE export failed:\n" <> msg
    JsonParseError(msg) -> "JSON parse error: " <> msg
    CyclicDependency(beads) ->
      "Cyclic dependency detected involving: " <> string.join(beads, ", ")
    MissingDependency(bead, missing) ->
      "Bead '" <> bead <> "' requires '" <> missing <> "' which does not exist"
  }
}

// =============================================================================
// PRIVATE: Dependency Analysis
// =============================================================================

fn validate_dependencies(
  beads: List(PlanBead),
  by_id: Dict(String, PlanBead),
) -> Result(Nil, PlanError) {
  list.try_each(beads, fn(bead) {
    list.try_each(bead.requires, fn(dep) {
      case dict.has_key(by_id, dep) {
        True -> Ok(Nil)
        False -> Error(MissingDependency(bead.id, dep))
      }
    })
  })
}

fn build_phases(
  beads: List(PlanBead),
  by_id: Dict(String, PlanBead),
) -> Result(List(ExecutionPhase), PlanError) {
  // Calculate dependency depth for each bead
  let depths = calculate_depths(beads, by_id)

  // Check for cycles (depths will be incomplete)
  let max_expected = list.length(beads)
  case dict.size(depths) < max_expected {
    True -> {
      let missing =
        beads
        |> list.filter(fn(b) { !dict.has_key(depths, b.id) })
        |> list.map(fn(b) { b.id })
      Error(CyclicDependency(missing))
    }
    False -> {
      // Group by depth level
      let grouped = group_by_depth(beads, depths)

      // Convert to phases
      let phases =
        grouped
        |> dict.to_list
        |> list.sort(fn(a, b) { int.compare(a.0, b.0) })
        |> list.index_map(fn(pair, idx) {
          let #(_, bead_ids) = pair
          let phase_beads =
            bead_ids
            |> list.filter_map(fn(id) {
              dict.get(by_id, id) |> result.nil_error
            })

          ExecutionPhase(
            phase_number: idx + 1,
            title: "Phase " <> int.to_string(idx + 1),
            beads: phase_beads,
            can_parallel: list.length(phase_beads) > 1,
            effort: calculate_phase_effort(phase_beads),
          )
        })

      Ok(phases)
    }
  }
}

fn calculate_depths(
  beads: List(PlanBead),
  _by_id: Dict(String, PlanBead),
) -> Dict(String, Int) {
  // Iteratively calculate depths until no changes
  let initial_depths =
    beads
    |> list.filter(fn(b) { list.is_empty(b.requires) })
    |> list.map(fn(b) { #(b.id, 0) })
    |> dict.from_list

  calculate_depths_loop(beads, initial_depths, list.length(beads))
}

fn calculate_depths_loop(
  beads: List(PlanBead),
  depths: Dict(String, Int),
  max_iterations: Int,
) -> Dict(String, Int) {
  case max_iterations <= 0 {
    True -> depths
    False -> {
      // Try to calculate depth for beads without depths
      let new_depths =
        beads
        |> list.fold(depths, fn(acc, bead) {
          case dict.has_key(acc, bead.id) {
            True -> acc
            False -> {
              // Check if all dependencies have depths
              let dep_depths =
                bead.requires
                |> list.filter_map(fn(dep) {
                  dict.get(acc, dep) |> result.nil_error
                })

              case list.length(dep_depths) == list.length(bead.requires) {
                False -> acc
                True -> {
                  let max_dep = case dep_depths {
                    [] -> 0
                    _ -> list.fold(dep_depths, 0, int.max)
                  }
                  dict.insert(acc, bead.id, max_dep + 1)
                }
              }
            }
          }
        })

      // Check if we made progress
      case dict.size(new_depths) == dict.size(depths) {
        True -> depths
        False -> calculate_depths_loop(beads, new_depths, max_iterations - 1)
      }
    }
  }
}

fn group_by_depth(
  beads: List(PlanBead),
  depths: Dict(String, Int),
) -> Dict(Int, List(String)) {
  beads
  |> list.fold(dict.new(), fn(acc, bead) {
    case dict.get(depths, bead.id) {
      Error(_) -> acc
      Ok(depth) -> {
        let existing = dict.get(acc, depth) |> result.unwrap([])
        dict.insert(acc, depth, [bead.id, ..existing])
      }
    }
  })
}

// =============================================================================
// PRIVATE: Effort Calculation
// =============================================================================

fn calculate_total_effort(beads: List(PlanBead)) -> String {
  let minutes =
    beads
    |> list.map(fn(b) { effort_to_minutes(b.effort) })
    |> list.fold(0, int.add)

  format_duration(minutes)
}

fn calculate_phase_effort(beads: List(PlanBead)) -> String {
  let minutes =
    beads
    |> list.map(fn(b) { effort_to_minutes(b.effort) })
    |> list.fold(0, int.add)

  format_duration(minutes)
}

fn effort_to_minutes(effort: Effort) -> Int {
  case effort {
    Effort5min -> 5
    Effort10min -> 10
    Effort15min -> 15
    Effort20min -> 20
    Effort30min -> 30
  }
}

fn format_duration(minutes: Int) -> String {
  let hours = minutes / 60
  let mins = minutes % 60

  case hours, mins {
    0, m -> int.to_string(m) <> "min"
    h, 0 -> int.to_string(h) <> "h"
    h, m -> int.to_string(h) <> "h " <> int.to_string(m) <> "min"
  }
}

// =============================================================================
// PRIVATE: Risk Assessment
// =============================================================================

fn assess_risk(beads: List(PlanBead), phases: List(ExecutionPhase)) -> RiskLevel {
  let total = list.length(beads)
  let blocked_count =
    beads
    |> list.filter(fn(b) { b.status == Blocked || b.status == Failed })
    |> list.length

  let phase_count = list.length(phases)

  // Risk factors:
  // 1. Percentage of blocked/failed beads
  // 2. Number of phases (more phases = more complexity)
  // 3. Total beads (larger scope = more risk)

  let blocked_ratio = case total {
    0 -> 0.0
    _ -> int.to_float(blocked_count) /. int.to_float(total)
  }

  case blocked_ratio, phase_count, total {
    r, _, _ if r >. 0.3 -> Critical
    r, p, _ if r >. 0.1 || p > 10 -> High
    _, p, t if p > 5 || t > 50 -> Medium
    _, _, _ -> Low
  }
}

fn find_blockers(beads: List(PlanBead)) -> List(String) {
  beads
  |> list.filter(fn(b) { b.status == Blocked || b.status == Failed })
  |> list.map(fn(b) { b.id <> ": " <> b.title })
}

// =============================================================================
// PRIVATE: Formatting
// =============================================================================

fn format_phase_human(phase: ExecutionPhase) -> String {
  let parallel_indicator = case phase.can_parallel {
    True -> " [can run in parallel]"
    False -> ""
  }

  let header =
    "┌─ "
    <> phase.title
    <> " ("
    <> phase.effort
    <> ")"
    <> parallel_indicator
    <> "\n"

  let beads_list =
    phase.beads
    |> list.map(format_bead_line)
    |> string.join("\n")

  let footer =
    "└────────────────────────────────────────\n"

  header <> beads_list <> "\n" <> footer
}

fn format_phase_json(phase: ExecutionPhase) -> String {
  let beads_json =
    phase.beads
    |> list.map(fn(bead) { "\"" <> escape_json_string(bead.id) <> "\"" })
    |> string.join(", ")

  "{\n"
  <> "      \"phase_number\": "
  <> int.to_string(phase.phase_number)
  <> ",\n"
  <> "      \"title\": \""
  <> escape_json_string(phase.title)
  <> "\",\n"
  <> "      \"beads\": ["
  <> beads_json
  <> "],\n"
  <> "      \"can_parallel\": "
  <> bool_to_string(phase.can_parallel)
  <> ",\n"
  <> "      \"effort\": \""
  <> phase.effort
  <> "\"\n"
  <> "    }"
}

fn plan_bead_to_json(bead: PlanBead) -> json.Json {
  json.object([
    #("id", json.string(bead.id)),
    #("title", json.string(bead.title)),
    #("requires", json.array(bead.requires, json.string)),
    #("effort", json.string(effort_to_label(bead.effort))),
    #("status", json.string(bead_status_to_string(bead.status))),
  ])
}

fn format_bead_line(bead: PlanBead) -> String {
  "│  • "
  <> bead.id
  <> " - "
  <> bead.title
  <> " ["
  <> bead_status_to_string(bead.status)
  <> ", "
  <> effort_to_label(bead.effort)
  <> "]"
}

pub fn risk_to_string(risk: RiskLevel) -> String {
  case risk {
    Low -> "low"
    Medium -> "medium"
    High -> "high"
    Critical -> "critical"
  }
}

fn bool_to_string(b: Bool) -> String {
  case b {
    True -> "true"
    False -> "false"
  }
}

fn pad_right(s: String, width: Int) -> String {
  let len = string.length(s)
  case len >= width {
    True -> s
    False -> s <> string.repeat(" ", width - len)
  }
}

fn escape_json_string(s: String) -> String {
  s
  |> string.replace("\\", "\\\\")
  |> string.replace("\"", "\\\"")
  |> string.replace("\n", "\\n")
  |> string.replace("\t", "\\t")
}

// =============================================================================
// PRIVATE: CUE Parsing (Simplified)
// =============================================================================

/// Parse beads from a session CUE file using cue export
fn parse_beads_from_cue(
  session_path: String,
) -> Result(List(PlanBead), PlanError) {
  case security.validate_file_path(session_path) {
    Ok(validated_path) -> export_beads_from_cue(validated_path)
    Error(err) -> Error(ParseError(security.format_security_error(err)))
  }
}

fn export_beads_from_cue(path: String) -> Result(List(PlanBead), PlanError) {
  case
    shellout.command("cue", ["export", path, "-e", "session.beads"], ".", [])
  {
    Ok(json_str) -> decode_beads_json(json_str)
    Error(#(_, stderr)) -> {
      case shellout.command("cue", ["export", path, "-e", "beads"], ".", []) {
        Ok(json_str) -> decode_beads_json(json_str)
        Error(#(_, stderr_fallback)) ->
          Error(CueExportError(stderr <> "\n" <> stderr_fallback))
      }
    }
  }
}

/// Decode beads from JSON (exported via cue)
pub fn decode_beads_json(json_str: String) -> Result(List(PlanBead), PlanError) {
  use data <- result.try(
    json.decode(json_str, dynamic.dynamic)
    |> result.map_error(fn(_) { JsonParseError("Failed to decode beads JSON") }),
  )

  use bead_values <- result.try(extract_bead_values(data))
  list_try_map(bead_values, parse_plan_bead)
}

fn list_try_map(
  list: List(a),
  fun: fn(a) -> Result(b, PlanError),
) -> Result(List(b), PlanError) {
  case list {
    [] -> Ok([])
    [head, ..tail] -> {
      use value <- result.try(fun(head))
      use rest <- result.try(list_try_map(tail, fun))
      Ok([value, ..rest])
    }
  }
}

fn extract_bead_values(
  data: dynamic.Dynamic,
) -> Result(List(dynamic.Dynamic), PlanError) {
  case dynamic.list(dynamic.dynamic)(data) {
    Ok(values) -> Ok(values)
    Error(_) -> {
      case dynamic.field("beads", dynamic.list(dynamic.dynamic))(data) {
        Ok(values) -> Ok(values)
        Error(_) ->
          case
            dynamic.field(
              "session",
              dynamic.field("beads", dynamic.list(dynamic.dynamic)),
            )(data)
          {
            Ok(values) -> Ok(values)
            Error(_) -> Error(ParseError("Beads list not found in JSON"))
          }
      }
    }
  }
}

fn parse_plan_bead(data: dynamic.Dynamic) -> Result(PlanBead, PlanError) {
  use id <- result.try(require_string_field(data, ["id", "bead_id", "name"]))

  let title =
    get_string_field(data, ["title", "summary", "name"])
    |> option.unwrap(id)

  let requires =
    get_string_list_field(data, ["requires", "dependencies", "depends_on"])
    |> option.unwrap([])

  let effort = get_effort_field(data)
  let status = get_status_field(data)

  Ok(PlanBead(
    id: id,
    title: title,
    requires: requires,
    effort: effort,
    status: status,
  ))
}

fn require_string_field(
  data: dynamic.Dynamic,
  keys: List(String),
) -> Result(String, PlanError) {
  case get_string_field(data, keys) {
    option.Some(value) -> Ok(value)
    option.None -> Error(ParseError("Bead is missing required id field"))
  }
}

fn get_string_field(
  data: dynamic.Dynamic,
  keys: List(String),
) -> option.Option(String) {
  keys
  |> list.fold(option.None, fn(acc, key) {
    case acc {
      option.Some(_) -> acc
      option.None ->
        case dynamic.field(key, dynamic.string)(data) {
          Ok(value) -> option.Some(value)
          Error(_) -> option.None
        }
    }
  })
}

fn get_string_list_field(
  data: dynamic.Dynamic,
  keys: List(String),
) -> option.Option(List(String)) {
  keys
  |> list.fold(option.None, fn(acc, key) {
    case acc {
      option.Some(_) -> acc
      option.None ->
        case dynamic.field(key, dynamic.list(dynamic.string))(data) {
          Ok(value) -> option.Some(value)
          Error(_) -> option.None
        }
    }
  })
}

fn get_effort_field(data: dynamic.Dynamic) -> Effort {
  case dynamic.field("effort", dynamic.string)(data) {
    Ok(value) -> parse_effort_string(value)
    Error(_) ->
      case dynamic.field("effort_minutes", dynamic.int)(data) {
        Ok(minutes) -> effort_from_minutes(minutes)
        Error(_) -> Effort15min
      }
  }
}

fn parse_effort_string(value: String) -> Effort {
  let cleaned =
    value
    |> string.lowercase
    |> string.replace(" ", "")

  case cleaned {
    "5min" -> Effort5min
    "10min" -> Effort10min
    "15min" -> Effort15min
    "20min" -> Effort20min
    "30min" -> Effort30min
    _ -> {
      let digits =
        cleaned
        |> string.to_graphemes
        |> list.filter(fn(char) {
          case char {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" -> True
            _ -> False
          }
        })
        |> string.join("")

      case int.parse(digits) {
        Ok(minutes) -> effort_from_minutes(minutes)
        Error(_) -> Effort15min
      }
    }
  }
}

fn effort_from_minutes(minutes: Int) -> Effort {
  case minutes {
    m if m <= 7 -> Effort5min
    m if m <= 12 -> Effort10min
    m if m <= 17 -> Effort15min
    m if m <= 25 -> Effort20min
    _ -> Effort30min
  }
}

fn get_status_field(data: dynamic.Dynamic) -> BeadStatus {
  case get_string_field(data, ["status", "state"]) {
    option.Some(value) -> bead_status_from_string(value)
    option.None -> Pending
  }
}

fn bead_status_from_string(value: String) -> BeadStatus {
  case string.lowercase(value) {
    "open" -> Pending
    "pending" -> Pending
    "ready" -> Pending
    "in_progress" -> InProgress
    "in-progress" -> InProgress
    "inprogress" -> InProgress
    "in progress" -> InProgress
    "blocked" -> Blocked
    "completed" -> Completed
    "complete" -> Completed
    "closed" -> Completed
    "done" -> Completed
    "success" -> Completed
    "succeeded" -> Completed
    "skipped" -> Completed
    "failed" -> Failed
    "error" -> Failed
    _ -> Pending
  }
}

// =============================================================================
// PRIVATE: External Functions
// =============================================================================

@external(erlang, "intent_ffi", "current_iso8601_timestamp")
fn current_iso8601_timestamp() -> String
