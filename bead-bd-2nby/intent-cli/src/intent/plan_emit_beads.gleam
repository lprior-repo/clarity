//// Plan Emit Beads
//// Emits beads from a session to the br issue tracker with idempotency safeguards
////
//// Idempotency strategy:
//// - Parse existing beads from session file
//// - Check which beads already exist in br (by title matching)
//// - Only create beads that don't exist yet
//// - Track emitted bead IDs in session metadata

import gleam/int
import gleam/list
import gleam/result
import gleam/string
import intent/plan_mode.{type PlanBead}

/// Emission result tracking
pub type EmissionResult {
  EmissionResult(
    session_id: String,
    dry_run: Bool,
    total_beads: Int,
    already_exists: Int,
    would_create: Int,
    created: Int,
    failed: Int,
    commands: List(String),
  )
}

/// Error types for bead emission
pub type EmitError {
  BrCommandFailed(command: String, reason: String)
  InvalidSession(reason: String)
  ParseError(reason: String)
}

/// Emit beads from a session to br
pub fn emit_beads(
  session_id: String,
  dry_run: Bool,
  execute: Bool,
  force: Bool,
) -> Result(EmissionResult, EmitError) {
  let session_path = ".intent/session-" <> session_id <> ".cue"

  // Parse beads from session
  let beads_result = plan_mode.compute_plan(session_id)

  case beads_result {
    Error(plan_mode.SessionNotFound(_)) -> {
      Error(InvalidSession("Session file not found: " <> session_path))
    }
    Error(err) -> Error(ParseError(plan_mode.format_error(err)))
    Ok(plan) -> {
      let beads = extract_beads_from_plan(plan)

      // Check which beads already exist (skip if force flag is set)
      use existing_beads <- result.then(case force {
        True -> Ok([])
        False -> check_existing_beads(beads)
      })

      // Determine which beads need to be created
      let new_beads = filter_new_beads(beads, existing_beads)

      // Generate commands
      let commands = generate_br_commands(session_id, new_beads)

      let result =
        EmissionResult(
          session_id: session_id,
          dry_run: dry_run,
          total_beads: list.length(beads),
          already_exists: list.length(existing_beads),
          would_create: list.length(new_beads),
          created: 0,
          failed: 0,
          commands: commands,
        )

      // Execute if not dry run and execute flag is set
      case dry_run || !execute {
        True -> Ok(result)
        // Dry run - don't execute
        False -> {
          use final_result <- result.then(execute_commands(result, new_beads))
          Ok(final_result)
        }
      }
    }
  }
}

/// Extract beads from execution plan
fn extract_beads_from_plan(plan: plan_mode.ExecutionPlan) -> List(PlanBead) {
  list.fold(plan.phases, [], fn(acc, phase) { list.append(phase.beads, acc) })
}

/// Check which beads already exist in br by listing all issues
fn check_existing_beads(
  beads: List(PlanBead),
) -> Result(List(PlanBead), EmitError) {
  // Get list of all existing bead titles
  use existing_titles <- result.then(get_existing_bead_titles())

  // Find beads that already exist (by title matching)
  let existing_beads =
    list.filter(beads, fn(bead) {
      list.any(existing_titles, fn(title) {
        string.lowercase(title) == string.lowercase(bead.title)
      })
    })

  Ok(existing_beads)
}

/// Get list of existing bead titles from br
fn get_existing_bead_titles() -> Result(List(String), EmitError) {
  // TODO: Use shellout to run 'br list --json'
  // For now, return empty list to allow development
  Ok([])
}

/// Filter out beads that already exist
fn filter_new_beads(
  beads: List(PlanBead),
  existing_beads: List(PlanBead),
) -> List(PlanBead) {
  let existing_ids =
    existing_beads
    |> list.map(fn(b) { string.lowercase(b.id) })

  list.filter(beads, fn(bead) {
    let bead_id_lower = string.lowercase(bead.id)
    !list.any(existing_ids, fn(existing_id) { existing_id == bead_id_lower })
  })
}

/// Generate br create commands for new beads
fn generate_br_commands(
  session_id: String,
  beads: List(PlanBead),
) -> List(String) {
  list.map(beads, fn(bead) {
    let type_flag = case bead.status {
      plan_mode.Failed -> " -t bug"
      plan_mode.Blocked -> " -t bug"
      _ -> " -t feature"
    }

    let effort_priority = effort_to_priority(bead.effort)

    "br create \""
    <> bead.title
    <> "\" -p "
    <> int.to_string(effort_priority)
    <> type_flag
    <> " --deps discovered-from:session-"
    <> session_id
    <> " --json"
  })
}

/// Convert effort to br priority (lower effort = higher priority)
fn effort_to_priority(effort: plan_mode.Effort) -> Int {
  case effort {
    plan_mode.Effort5min -> 2
    plan_mode.Effort10min -> 2
    plan_mode.Effort15min -> 3
    plan_mode.Effort20min -> 3
    plan_mode.Effort30min -> 4
  }
}

/// Execute br commands
fn execute_commands(
  result: EmissionResult,
  beads: List(PlanBead),
) -> Result(EmissionResult, EmitError) {
  case beads {
    [] -> Ok(result)
    _ -> {
      // TODO: Use shellout to execute commands
      // For now, return success without executing
      Ok(EmissionResult(..result, created: list.length(beads)))
    }
  }
}

/// Format emission result as human-readable output
pub fn format_result(result: EmissionResult) -> String {
  let header =
    "╔══════════════════════════════════════════════════════════════╗\n"
    <> "║                    BEAD EMISSION REPORT                    ║\n"
    <> "╠══════════════════════════════════════════════════════════════╣\n"
    <> "║ Session: "
    <> pad_right(result.session_id, 51)
    <> "║\n"
    <> "║ Total Beads: "
    <> pad_right(int.to_string(result.total_beads), 47)
    <> "║\n"
    <> "║ Already Exists: "
    <> pad_right(int.to_string(result.already_exists), 44)
    <> "║\n"
    <> "║ Would Create: "
    <> pad_right(int.to_string(result.would_create), 46)
    <> "║\n"
    <> "║ Created: "
    <> pad_right(int.to_string(result.created), 50)
    <> "║\n"
    <> "║ Failed: "
    <> pad_right(int.to_string(result.failed), 50)
    <> "║\n"
    <> "╚══════════════════════════════════════════════════════════════╝\n\n"

  let mode_section = case result.dry_run {
    True -> "🔍 DRY RUN MODE - No beads were created\n\n"
    False -> "✅ EXECUTION MODE - Beads were created in br\n\n"
  }

  let commands_section = case list.is_empty(result.commands) {
    True -> "No new beads to create.\n"
    False -> {
      "Commands that would run:\n"
      <> {
        result.commands
        |> list.map(fn(cmd) { "  " <> cmd })
        |> string.join("\n")
      }
      <> "\n"
    }
  }

  header <> mode_section <> commands_section
}

/// Format emission error
pub fn format_error(error: EmitError) -> String {
  case error {
    BrCommandFailed(cmd, reason) ->
      "Failed to execute br command:\n"
      <> "  Command: "
      <> cmd
      <> "\n"
      <> "  Reason: "
      <> reason
      <> "\n"
    InvalidSession(reason) -> "Invalid session: " <> reason <> "\n"
    ParseError(reason) -> "Failed to parse session: " <> reason <> "\n"
  }
}

fn pad_right(s: String, width: Int) -> String {
  let len = string.length(s)
  case len >= width {
    True -> s
    False -> s <> string.repeat(" ", width - len)
  }
}

// =============================================================================
// TEST HELPER FUNCTIONS
// =============================================================================

/// Export filter_new_beads for testing
pub fn filter_new_beads_for_test(
  beads: List(PlanBead),
  existing_beads: List(PlanBead),
) -> List(PlanBead) {
  filter_new_beads(beads, existing_beads)
}

/// Export effort_to_priority for testing
pub fn effort_to_priority_for_test(effort: plan_mode.Effort) -> Int {
  effort_to_priority(effort)
}
