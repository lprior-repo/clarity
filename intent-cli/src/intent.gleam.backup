/// Intent CLI - Planning and bead generation tool
import argv
import gleam/dynamic
import gleam/int
import gleam/io
import gleam/json
import gleam/list
import gleam/result
import gleam/string
import gleam_community/ansi
import glint
import glint/flag
import intent/cli_ui
import intent/effects_analyzer.{Cascade, Notification, RaceCondition, RollbackRequired, High, Low, Medium, StateChange, type EffectType, type SpecAnalysis}
import intent/interview
import intent/interview_storage
import intent/loader
import intent/parser
import intent/plan_emit_beads
import intent/ready_document
import intent/security
import intent/vision_document
import intent/validation
import shellout
import simplifile

/// Exit codes
const exit_pass = 0

const exit_fail = 1

pub fn main() {
  let args = argv.load().arguments

  // Check for --version flag (before normalization)
  case
    list.find(args, fn(arg) {
      arg == "--version"
      || arg == "-v"
      || arg == "--version=true"
      || arg == "--version=True"
    })
  {
    Ok(_) -> {
      io.println("intent v0.1.0")
      exit(exit_pass)
    }
    Error(_) -> Nil
  }

  // Handle 'help' command specially - convert to --help for the target command
  let processed_args = case args {
    ["help", ..] -> {
      // Convert "help <command>" to "<command> --help"
      case args {
        ["help"] -> ["--help"]
        ["help", command, ..rest] -> [command, "--help", ..rest]
        _ -> ["--help"]
      }
    }
    _ -> normalize_cli_args(args)
  }

  let app =
    glint.new()
    |> glint.with_name("intent")
    |> glint.with_pretty_help(glint.default_pretty_help())
    |> glint.add(at: ["interview"], do: interview_command())
    |> glint.add(at: ["beads"], do: beads_command())
    |> glint.add(at: ["bead-status"], do: bead_status_command())
    |> glint.add(at: ["history"], do: history_command())
    |> glint.add(at: ["diff"], do: diff_command())
    |> glint.add(at: ["sessions"], do: sessions_command())
    // Plan commands
    |> glint.add(at: ["plan"], do: plan_command())
    |> glint.add(at: ["plan-next"], do: plan_next_command())
    |> glint.add(at: ["plan-approve"], do: plan_approve_command())
    |> glint.add(at: ["plan-emit-beads"], do: plan_emit_beads_command())
    |> glint.add(at: ["beads-regenerate"], do: beads_regenerate_command())
    // Vision and Ready commands
    |> glint.add(at: ["vision"], do: vision_command())
    |> glint.add(at: ["ready"], do: ready_command())
    // KIRK commands
    |> glint.add(at: ["effects"], do: effects_command())

  case glint.execute(app, processed_args) {
    Ok(glint.Out(_)) -> {
      io.println_error(
        "error: failed to run command
cause:
  0: command not found",
      )
      exit(exit_fail)
    }
    Ok(glint.Help(help_text)) -> {
      io.println(help_text)
      exit(exit_pass)
    }
    Error(err) -> {
      io.println_error(err)
      exit(exit_fail)
    }
  }
}

pub fn normalize_cli_args(args: List(String)) -> List(String) {
  case args {
    [arg, next, ..rest] -> {
      case classify_flag(arg) {
        BoolFlag -> {
          case is_bool_literal(next) {
            True -> [
              arg <> "=" <> string.lowercase(next),
              ..normalize_cli_args(rest)
            ]
            False -> [arg <> "=true", ..normalize_cli_args([next, ..rest])]
          }
        }
        ValueFlag -> {
          case is_flag_token(next) {
            True -> [arg, ..normalize_cli_args([next, ..rest])]
            False -> [arg <> "=" <> next, ..normalize_cli_args(rest)]
          }
        }
        UnknownFlag -> [arg, ..normalize_cli_args([next, ..rest])]
      }
    }
    [arg] -> {
      case classify_flag(arg) {
        BoolFlag -> [arg <> "=true"]
        _ -> [arg]
      }
    }
    [] -> []
  }
}

type FlagKind {
  BoolFlag
  ValueFlag
  UnknownFlag
}

fn classify_flag(arg: String) -> FlagKind {
  let is_candidate = string.starts_with(arg, "--") && !string.contains(arg, "=")
  bool_to_flag_kind(is_candidate, string.drop_left(arg, 2))
}

fn bool_to_flag_kind(is_candidate: Bool, flag_name: String) -> FlagKind {
  case is_candidate {
    False -> UnknownFlag
    True -> {
      case is_known_bool_flag(flag_name) {
        True -> BoolFlag
        False -> {
          case is_known_value_flag(flag_name) {
            True -> ValueFlag
            False -> UnknownFlag
          }
        }
      }
    }
  }
}

fn is_known_bool_flag(flag_name: String) -> Bool {
  case flag_name {
    "json" -> True
    "verbose" -> True
    "quiet" -> True
    "yes" -> True
    "draft" -> True
    "confirm" -> True
    "dry-run" -> True
    "execute" -> True
    _ -> False
  }
}

fn is_known_value_flag(flag_name: String) -> Bool {
  case flag_name {
    "only" -> True
    "profile" -> True
    "resume" -> True
    "answer" -> True
    "bead-id" -> True
    "status" -> True
    "reason" -> True
    "session" -> True
    "format" -> True
    "notes" -> True
    "strategy" -> True
    "output" -> True
    "out" -> True
    "name" -> True
    "target" -> True
    _ -> False
  }
}

fn is_flag_token(value: String) -> Bool {
  string.starts_with(value, "--")
}

fn is_bool_literal(value: String) -> Bool {
  case string.lowercase(value) {
    "true" -> True
    "false" -> True
    _ -> False
  }
}

/// ============================================================================
/// INTERVIEW COMMAND
/// ============================================================================
fn interview_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let profile =
      flag.get_string(input.flags, "profile")
      |> result.unwrap("")

    let resume_session =
      flag.get_string(input.flags, "resume")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "interview") {
      Ok(Nil) -> {
        case resume_session {
          "" -> {
            // Validate profile when not resuming
            case validation.validate_profile(profile) {
              Ok(valid_profile) -> run_interview(valid_profile, "")
              Error(err) -> {
                cli_ui.print_error(err)
                exit(exit_fail)
              }
            }
          }
          session_id -> run_interview(profile, session_id)
        }
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description(
    "Run interactive interview session to capture requirements",
  )
  |> glint.flag(
    "profile",
    flag.string()
      |> flag.default("")
      |> flag.description("Profile type: api, cli, event, data, workflow, ui"),
  )
  |> glint.flag(
    "resume",
    flag.string()
      |> flag.default("")
      |> flag.description("Resume a previous session by ID"),
  )
}

fn run_interview(profile: String, session_id: String) -> Nil {
  // TODO: Implement interview functionality
  cli_ui.print_header("Interview")
  io.println("Profile: " <> profile)
  case session_id {
    "" -> Nil
    _ -> io.println("Resume session: " <> session_id)
  }
  cli_ui.print_success("Interview command - implementation needed")
  exit(exit_pass)
}

/// ============================================================================
/// BEADS COMMAND
/// ============================================================================
fn beads_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let format =
      flag.get_string(input.flags, "format")
      |> result.unwrap("")

    let output_dir =
      flag.get_string(input.flags, "out")
      |> result.unwrap("")

    let session_id =
      flag.get_string(input.flags, "session")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "beads") {
      Ok(Nil) -> {
        // Validate session_id is required
        case validation.validate_required_flag("session", session_id) {
          Ok(valid_session) -> {
            // Validate format
            case validation.validate_format(format) {
              Ok(valid_format) ->
                generate_beads(valid_session, valid_format, output_dir)
              Error(err) -> {
                cli_ui.print_error(err)
                exit(exit_fail)
              }
            }
          }
          Error(err) -> {
            cli_ui.print_error(err)
            exit(exit_fail)
          }
        }
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Generate beads from interview session")
  |> glint.flag(
    "session",
    flag.string()
      |> flag.default("")
      |> flag.description("Session ID to generate beads from"),
  )
  |> glint.flag(
    "format",
    flag.string()
      |> flag.default("json")
      |> flag.description("Output format: json, jsonl, markdown"),
  )
  |> glint.flag(
    "out",
    flag.string()
      |> flag.default("")
      |> flag.description("Output directory (default: current directory)"),
  )
}

fn generate_beads(
  session_id: String,
  format: String,
  _output_dir: String,
) -> Nil {
  // TODO: Implement bead generation
  cli_ui.print_header("Generate Beads")
  io.println("Session: " <> session_id)
  io.println("Format: " <> format)
  cli_ui.print_success("Beads command - implementation needed")
  exit(exit_pass)
}

/// ============================================================================
/// BEAD STATUS COMMAND
/// ============================================================================
fn bead_status_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let bead_id =
      flag.get_string(input.flags, "bead-id")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "bead-status") {
      Ok(Nil) -> {
        // Validate bead_id is required
        case validation.validate_required_flag("bead-id", bead_id) {
          Ok(valid_bead_id) -> check_bead_status(valid_bead_id)
          Error(err) -> {
            cli_ui.print_error(err)
            exit(exit_fail)
          }
        }
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Check status of a bead")
  |> glint.flag(
    "bead-id",
    flag.string()
      |> flag.default("")
      |> flag.description("Bead ID to check"),
  )
}

fn check_bead_status(bead_id: String) -> Nil {
  // TODO: Implement bead status check
  cli_ui.print_header("Bead Status")
  io.println("Bead ID: " <> bead_id)
  cli_ui.print_success("Bead status command - implementation needed")
  exit(exit_pass)
}

/// ============================================================================
/// HISTORY COMMAND
/// ============================================================================
fn history_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    // Validate no extra arguments
    case validation.validate_no_args(input.args, "history") {
      Ok(Nil) -> show_history()
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("List all interview sessions")
}

fn show_history() -> Nil {
  cli_ui.print_header("Interview History")

  let sessions_path = ".interview/sessions.jsonl"

  case interview_storage.list_sessions_from_jsonl(sessions_path) {
    Ok(sessions) -> {
      case sessions {
        [] -> {
          cli_ui.print_warning("No sessions found")
          io.println("")
          io.println("Start a new session with:")
          io.println("  intent interview --profile <profile>")
        }
        _ -> {
          // Sort by created_at (newest first)
          let sorted =
            sessions
            |> list.sort(by: fn(a, b) {
              string.compare(b.created_at, a.created_at)
            })

          // Display sessions as a formatted table
          display_sessions_table(sorted)

          // Show summary
          io.println("")
          cli_ui.print_info(
            "Total sessions: " <> int.to_string(list.length(sorted)),
          )
        }
      }
      exit(exit_pass)
    }
    Error(err) -> {
      // Check if it's a "file not found" error
      case string.contains(err, "No such file")
        || string.contains(err, "not found")
        || string.contains(err, "Enoent") {
        True -> {
          cli_ui.print_warning("No sessions found")
          io.println("")
          io.println("Start a new session with:")
          io.println("  intent interview --profile <profile>")
          exit(exit_pass)
        }
        False -> {
          cli_ui.print_error("Failed to load sessions: " <> err)
          exit(exit_fail)
        }
      }
    }
  }
}

fn display_sessions_table(sessions: List(interview.InterviewSession)) -> Nil {
  // Calculate column widths
  let max_id_width =
    sessions
    |> list.map(fn(s) { string.length(s.id) })
    |> list.fold(from: 0, with: fn(acc, width) { int.max(acc, width) })

  let max_profile_width =
    sessions
    |> list.map(fn(s) {
      s.profile
      |> interview.profile_to_string
      |> string.length()
    })
    |> list.fold(from: 0, with: fn(acc, width) { int.max(acc, width) })

  // Ensure minimum widths for headers
  let id_width = int.max(max_id_width, 10)
  let profile_width = int.max(max_profile_width, 7)

  // Print header
  let header =
    "Session ID"
    <> string.repeat(" ", id_width - string.length("Session ID") + 2)
    <> "Profile"
    <> string.repeat(" ", profile_width - string.length("Profile") + 2)
    <> "Created            "
    <> "Answers"

  io.println(ansi.bold(ansi.cyan(header)))
  io.println(
    string.repeat("─", id_width)
    <> "--"
    <> string.repeat("─", profile_width)
    <> "--"
    <> "────────────────────"
    <> "────────",
  )

  // Print each session
  list.each(sessions, fn(session) {
    let profile_str = interview.profile_to_string(session.profile)
    let id_padding = id_width - string.length(session.id) + 2
    let profile_padding = profile_width - string.length(profile_str) + 2
    let answers_count = int.to_string(list.length(session.answers))

    let row =
      session.id
      <> string.repeat(" ", id_padding)
      <> profile_str
      <> string.repeat(" ", profile_padding)
      <> format_timestamp(session.created_at)
      <> "  "
      <> answers_count

    io.println(row)
  })
}

fn format_timestamp(timestamp: String) -> String {
  // Parse ISO timestamp and format as YYYY-MM-DD HH:MM
  // For now, just return the timestamp as-is (simplified)
  // In a full implementation, would parse and reformat
  let parts = string.split(timestamp, "T")
  case parts {
    [date, time] -> {
      let time_parts = string.split(time, ":")
      case time_parts {
        [hour, minute, _] -> date <> " " <> hour <> ":" <> minute
        _ -> timestamp
      }
    }
    _ -> timestamp
  }
}

/// ============================================================================
/// DIFF COMMAND
/// ============================================================================
fn diff_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let session_id =
      flag.get_string(input.flags, "session")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "diff") {
      Ok(Nil) -> {
        // Validate session_id is required
        case validation.validate_required_flag("session", session_id) {
          Ok(valid_session) -> show_session_diff(valid_session)
          Error(err) -> {
            cli_ui.print_error(err)
            exit(exit_fail)
          }
        }
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Show diff for session changes")
  |> glint.flag(
    "session",
    flag.string()
      |> flag.default("")
      |> flag.description("Session ID to show diff for"),
  )
}

fn show_session_diff(session_id: String) -> Nil {
  cli_ui.print_header("Session Diff")

  let sessions_path = ".interview/sessions.jsonl"
  let history_path = ".interview/history.jsonl"

  // Load current session
  case interview_storage.get_session_from_jsonl(sessions_path, session_id) {
    Error(err) -> {
      cli_ui.print_error("Session not found: " <> session_id)
      io.println("\n" <> err)
      io.println("\nAvailable sessions:")
      case interview_storage.list_sessions_from_jsonl(sessions_path) {
        Ok(sessions) -> {
          case sessions {
            [] -> io.println("  No sessions found")
            _ -> {
              list.each(sessions, fn(s) {
                io.println(
                  "  - "
                  <> s.id
                  <> " ("
                  <> interview.profile_to_string(s.profile)
                  <> ", "
                  <> interview.stage_to_string(s.stage)
                  <> ")",
                )
              })
            }
          }
        }
        Error(_) -> io.println("  Unable to load sessions")
      }
      exit(exit_fail)
    }
    Ok(current_session) -> {
      // Load session history to find previous snapshot
      case interview_storage.list_session_history(history_path, session_id) {
        Error(_) -> {
          // No history available yet - show basic session info
          cli_ui.print_warning("No previous snapshots found for this session")
          io.println("\nCurrent session state:")
          io.println("  Profile: " <> interview.profile_to_string(current_session.profile))
          io.println("  Stage: " <> interview.stage_to_string(current_session.stage))
          io.println("  Updated: " <> current_session.updated_at)
          io.println("  Answers: " <> int.to_string(list.length(current_session.answers)))
          io.println("  Unresolved gaps: " <> int.to_string(list.length(list.filter(current_session.gaps, fn(g) { !g.resolved }))))
          io.println("  Unresolved conflicts: " <> int.to_string(list.length(list.filter(current_session.conflicts, fn(c) { c.chosen < 0 }))))
          exit(exit_pass)
        }
        Ok(snapshots) -> {
          case snapshots {
            [] -> {
              // No snapshots yet
              cli_ui.print_warning("No previous snapshots found for this session")
              io.println("\nThis is the first version of this session.")
              exit(exit_pass)
            }
            _ -> {
              // Get the most recent snapshot (last in list)
              let snapshots_sorted =
                list.sort(snapshots, fn(a, b) {
                  string.compare(a.timestamp, b.timestamp)
                })

              let previous_snapshot = list.last(snapshots_sorted)

              case previous_snapshot {
                Error(Nil) -> {
                  cli_ui.print_error("Unable to load previous snapshot")
                  exit(exit_fail)
                }
                Ok(prev_snap) -> {
                  // We need to load the previous session state
                  // For now, compare with an empty session to show what was added
                  let empty_session = interview.InterviewSession(
                    id: session_id <> "-previous",
                    profile: current_session.profile,
                    created_at: prev_snap.timestamp,
                    updated_at: prev_snap.timestamp,
                    completed_at: "",
                    stage: interview.Discovery,
                    rounds_completed: 0,
                    answers: [],
                    gaps: [],
                    conflicts: [],
                    raw_notes: "",
                    current_phase: 1,
                    completed_phases: [],
                  )

                  // Generate diff
                  let diff =
                    interview_storage.diff_sessions(empty_session, current_session)

                  // Format and display
                  io.println(interview_storage.format_diff(diff))

                  // Show summary
                  io.println("\nPrevious snapshot: " <> prev_snap.snapshot_id)
                  io.println("  Description: " <> prev_snap.description)
                  io.println("  Timestamp: " <> prev_snap.timestamp)

                  cli_ui.print_success("Diff complete")
                  exit(exit_pass)
                }
              }
            }
          }
        }
      }
    }
  }
}

/// ============================================================================
/// SESSIONS COMMAND
/// ============================================================================
fn sessions_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let profile =
      flag.get_string(input.flags, "profile")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "sessions") {
      Ok(Nil) -> list_sessions(profile)
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("List interview sessions")
  |> glint.flag(
    "profile",
    flag.string()
      |> flag.default("")
      |> flag.description("Filter by profile type"),
  )
}

fn list_sessions(profile_filter: String) -> Nil {
  cli_ui.print_header("Sessions")

  let jsonl_path = ".interview/sessions.jsonl"

  case interview_storage.list_sessions_from_jsonl(jsonl_path) {
    Ok(sessions) -> {
      // Filter by profile if provided
      let filtered_sessions = case profile_filter {
        "" -> sessions
        filter ->
          list.filter(sessions, fn(session) {
            interview.profile_to_string(session.profile) == filter
          })
      }

      case filtered_sessions {
        [] -> {
          case profile_filter {
            "" -> io.println("No sessions found")
            _ ->
              io.println(
                "No sessions found for profile: "
                <> profile_filter
                <> "\n\nTry running without --profile to see all sessions",
              )
          }
        }
        _ -> {
          // Display sessions
          io.println("")
          list.each(filtered_sessions, fn(session) {
            display_session(session)
            io.println("")
          })

          // Summary
          case profile_filter {
            "" ->
              io.println(
                "Total: "
                <> int.to_string(list.length(filtered_sessions))
                <> " session(s)",
              )
            _ ->
              io.println(
                "Total: "
                <> int.to_string(list.length(filtered_sessions))
                <> " session(s) for profile: "
                <> profile_filter,
              )
          }
        }
      }

      exit(exit_pass)
    }
    Error(err) -> {
      cli_ui.print_error("Failed to load sessions: " <> err)
      exit(exit_fail)
    }
  }
}

fn display_session(session: interview.InterviewSession) -> Nil {
  let profile_str = interview.profile_to_string(session.profile)
  let stage_str = interview.stage_to_string(session.stage)

  // Session ID and profile
  io.println("ID:       " <> session.id)
  io.println("Profile:  " <> profile_str)
  io.println("Stage:    " <> stage_str)

  // Timestamps
  io.println("Created:  " <> session.created_at)
  case session.updated_at != session.created_at {
    True -> io.println("Updated:  " <> session.updated_at)
    False -> Nil
  }
  case session.completed_at != "" {
    True -> io.println("Completed: " <> session.completed_at)
    False -> Nil
  }

  // Progress info
  case session.rounds_completed {
    0 -> Nil
    n -> io.println("Rounds:   " <> int.to_string(n))
  }

  // Status indicators
  let gaps_count = list.length(list.filter(session.gaps, fn(g) { !g.resolved }))
  let conflicts_count =
    list.length(list.filter(session.conflicts, fn(c) { c.chosen < 0 }))

  let status = case gaps_count, conflicts_count {
    0, 0 -> "✓ Ready"
    _, 0 -> "⚠ " <> int.to_string(gaps_count) <> " gap(s)"
    0, _ -> "⚠ " <> int.to_string(conflicts_count) <> " conflict(s)"
    _, _ -> {
      "⚠ "
      <> int.to_string(gaps_count)
      <> " gap(s), "
      <> int.to_string(conflicts_count)
      <> " conflict(s)"
    }
  }
  io.println("Status:   " <> status)
}

/// ============================================================================
/// PLAN COMMAND
/// ============================================================================
fn plan_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let notes =
      flag.get_string(input.flags, "notes")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "plan") {
      Ok(Nil) -> {
        case notes {
          "" -> io.println("Plan generation - notes: none")
          _ -> io.println("Plan generation - notes: " <> notes)
        }
        cli_ui.print_success("Plan command - implementation needed")
        exit(exit_pass)
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Generate plan from current context")
  |> glint.flag(
    "notes",
    flag.string()
      |> flag.default("")
      |> flag.description("Additional notes for plan generation"),
  )
}

/// ============================================================================
/// PLAN NEXT COMMAND
/// ============================================================================
fn plan_next_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let strategy =
      flag.get_string(input.flags, "strategy")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "plan-next") {
      Ok(Nil) -> {
        // Validate strategy
        case validation.validate_strategy(strategy) {
          Ok(valid_strategy) -> {
            case valid_strategy {
              "" -> io.println("Suggest next task")
              _ ->
                io.println("Suggest next task - strategy: " <> valid_strategy)
            }
            cli_ui.print_success("Plan next command - implementation needed")
            exit(exit_pass)
          }
          Error(err) -> {
            cli_ui.print_error(err)
            exit(exit_fail)
          }
        }
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Suggest next task to work on")
  |> glint.flag(
    "strategy",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Selection strategy: page_rank, critical_path, shortest, risk_first",
      ),
  )
}

/// ============================================================================
/// PLAN APPROVE COMMAND
/// ============================================================================
fn plan_approve_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    // Validate single argument
    case validation.validate_single_arg(input.args, "plan-approve") {
      Ok(plan_id) -> {
        cli_ui.print_success("Plan approved: " <> plan_id)
        exit(exit_pass)
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Approve a generated plan")
}

/// ============================================================================
/// BEADS REGENERATE COMMAND
/// ============================================================================
fn beads_regenerate_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let session_id =
      flag.get_string(input.flags, "session")
      |> result.unwrap("")

    // Validate no extra arguments
    case validation.validate_no_args(input.args, "beads-regenerate") {
      Ok(Nil) -> {
        // Validate session_id is required
        case validation.validate_required_flag("session", session_id) {
          Ok(valid_session) -> regenerate_beads(valid_session)
          Error(err) -> {
            cli_ui.print_error(err)
            exit(exit_fail)
          }
        }
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Regenerate beads from session")
  |> glint.flag(
    "session",
    flag.string()
      |> flag.default("")
      |> flag.description("Session ID to regenerate beads from"),
  )
}

fn regenerate_beads(session_id: String) -> Nil {
  // TODO: Implement bead regeneration
  cli_ui.print_header("Regenerate Beads")
  io.println("Session: " <> session_id)
  cli_ui.print_success("Beads regenerate command - implementation needed")
  exit(exit_pass)
}

/// ============================================================================
/// PLAN EMIT BEADS COMMAND
/// ============================================================================
fn plan_emit_beads_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let dry_run =
      flag.get_bool(input.flags, "dry-run")
      |> result.unwrap(False)

    let execute =
      flag.get_bool(input.flags, "execute")
      |> result.unwrap(False)

    let force =
      flag.get_bool(input.flags, "force")
      |> result.unwrap(False)

    let target =
      flag.get_string(input.flags, "target")
      |> result.unwrap("br")

    // Validate single argument (session_id)
    case input.args {
      [] -> {
        cli_ui.print_error("Error: session ID required")
        io.println(
          "\nUsage: intent plan-emit-beads <session-id> [--dry-run] [--execute] [--force] [--target br]",
        )
        exit(exit_fail)
      }
      [session_id] -> {
        // Validate target is "br" (only supported target for now)
        case target {
          "br" -> emit_beads_to_br(session_id, dry_run, execute, force)
          _ -> {
            cli_ui.print_error("Error: unsupported target '" <> target <> "'")
            io.println("Supported targets: br")
            exit(exit_fail)
          }
        }
      }
      _ -> {
        cli_ui.print_error(
          "Error: plan-emit-beads takes exactly one argument (session-id)",
        )
        exit(exit_fail)
      }
    }
  })
  |> glint.description(
    "Emit beads from session to br (idempotent - won't create duplicates)",
  )
  |> glint.flag(
    "dry-run",
    flag.bool()
      |> flag.default(True)
      |> flag.description(
        "Show what would be created without creating beads (default: true)",
      ),
  )
  |> glint.flag(
    "execute",
    flag.bool()
      |> flag.default(False)
      |> flag.description(
        "Actually create beads in br (requires explicit confirmation)",
      ),
  )
  |> glint.flag(
    "force",
    flag.bool()
      |> flag.default(False)
      |> flag.description(
        "Bypass idempotency checks and create all beads (use with caution)",
      ),
  )
  |> glint.flag(
    "target",
    flag.string()
      |> flag.default("br")
      |> flag.description("Target system (default: br)"),
  )
}

fn emit_beads_to_br(
  session_id: String,
  dry_run: Bool,
  execute: Bool,
  force: Bool,
) -> Nil {
  cli_ui.print_header("Emit Beads to br")

  // Safety check: require --execute flag to actually create beads
  case !dry_run && !execute {
    True -> {
      cli_ui.print_error("Error: --execute flag required to create beads")
      io.println(
        "\nThis command will create beads in br using the session: "
        <> session_id,
      )
      io.println("\nTo see what would be created (dry run):")
      io.println("  intent plan-emit-beads " <> session_id)
      io.println("\nTo actually create beads:")
      io.println("  intent plan-emit-beads " <> session_id <> " --execute")
      io.println("\nTo bypass idempotency checks (force recreation):")
      io.println(
        "  intent plan-emit-beads " <> session_id <> " --execute --force",
      )
      exit(exit_fail)
    }
    False -> {
      case plan_emit_beads.emit_beads(session_id, dry_run, execute, force) {
        Ok(result) -> {
          io.println(plan_emit_beads.format_result(result))
          exit(exit_pass)
        }
        Error(err) -> {
          cli_ui.print_error(plan_emit_beads.format_error(err))
          exit(exit_fail)
        }
      }
    }
  }
}

/// ============================================================================
/// VISION COMMAND
/// ============================================================================
fn vision_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let output_dir =
      flag.get_string(input.flags, "out")
      |> result.unwrap("")

    // Validate argument count (0 or 1)
    case input.args {
      [] -> {
        cli_ui.print_error("Error: spec file required")
        io.println("\nUsage: intent vision <spec-file> [--out <dir>]")
        exit(exit_fail)
      }
      [spec_file] -> {
        generate_vision_document(spec_file, output_dir)
      }
      _ -> {
        cli_ui.print_error(
          "Error: vision command takes at most one argument (spec file)",
        )
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Generate vision document from spec")
  |> glint.flag(
    "out",
    flag.string()
      |> flag.default("")
      |> flag.description("Output directory (default: current directory)"),
  )
}

fn generate_vision_document(spec_file: String, output_dir: String) -> Nil {
  cli_ui.print_header("Vision Document Generation")

  // Validate file path for security
  case security.validate_file_path(spec_file) {
    Error(err) -> {
      cli_ui.print_error("Invalid file path: " <> security.format_security_error(err))
      exit(exit_fail)
    }
    Ok(validated_path) -> {
      // Verify file exists
      case simplifile.verify_is_file(validated_path) {
        Ok(False) -> {
          cli_ui.print_error("Spec file not found: " <> spec_file)
          exit(exit_fail)
        }
        Error(_) -> {
          cli_ui.print_error("Cannot access file: " <> spec_file)
          exit(exit_fail)
        }
        Ok(True) -> {
          // Export CUE to JSON
          case shellout.command("cue", ["export", validated_path], ".", []) {
            Ok(json_str) -> {
              // Parse JSON to get spec
              case json.decode(json_str, dynamic.dynamic) {
                Ok(json_data) -> {
                  case parser.decode_dynamic(json_data) {
                    Ok(spec) -> {
                      // Generate vision document
                      let vision_doc = vision_document.generate_vision_document(spec)

                      // Determine output path
                      let out_dir = case output_dir {
                        "" -> "."
                        _ -> output_dir
                      }

                      let output_path = out_dir <> "/" <> "vision.md"

                      // Write document
                      case simplifile.write(output_path, vision_doc) {
                        Ok(_) -> {
                          io.println("\nOutput: " <> output_path)
                          cli_ui.print_success("Vision document generated successfully")
                          exit(exit_pass)
                        }
                        Error(err) -> {
                          cli_ui.print_error("Failed to write vision document:")
                          io.println("Could not write file - check permissions and disk space")
                          exit(exit_fail)
                        }
                      }
                    }
                    Error(parse_errors) -> {
                      cli_ui.print_error("Failed to parse spec:")
                      io.println(format_parse_errors(parse_errors))
                      exit(exit_fail)
                    }
                  }
                }
                Error(_) -> {
                  cli_ui.print_error(
                    "Failed to decode JSON output from CUE export",
                  )
                  exit(exit_fail)
                }
              }
            }
            Error(#(_, stderr)) -> {
              cli_ui.print_error("Failed to export CUE to JSON:")
              io.println(stderr)
              exit(exit_fail)
            }
          }
        }
      }
    }
  }
}

/// ============================================================================
/// READY COMMAND
/// ============================================================================
fn ready_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let output_dir =
      flag.get_string(input.flags, "out")
      |> result.unwrap("")

    // Validate single argument (spec file)
    case input.args {
      [] -> {
        cli_ui.print_error("Error: spec file required")
        io.println("\nUsage: intent ready <spec-file> [--out <dir>]")
        exit(exit_fail)
      }
      [spec_file] -> {
        generate_ready_document(spec_file, output_dir)
      }
      _ -> {
        cli_ui.print_error(
          "Error: ready command takes exactly one argument (spec file)",
        )
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Generate ready document for implementation")
  |> glint.flag(
    "out",
    flag.string()
      |> flag.default("")
      |> flag.description("Output directory (default: current directory)"),
  )
}

fn generate_ready_document(spec_file: String, output_dir: String) -> Nil {
  cli_ui.print_header("Ready Document Generation")

  // Validate file path for security
  case security.validate_file_path(spec_file) {
    Error(err) -> {
      cli_ui.print_error("Invalid file path: " <> security.format_security_error(err))
      exit(exit_fail)
    }
    Ok(validated_path) -> {
      // Verify file exists
      case simplifile.verify_is_file(validated_path) {
        Ok(False) -> {
          cli_ui.print_error("Spec file not found: " <> spec_file)
          exit(exit_fail)
        }
        Error(_) -> {
          cli_ui.print_error("Cannot access file: " <> spec_file)
          exit(exit_fail)
        }
        Ok(True) -> {
          // Load spec
          case loader.load_spec_quiet(validated_path) {
            Error(err) -> {
              cli_ui.print_error("Failed to load spec:")
              io.println(loader.format_error(err))
              exit(exit_fail)
            }
            Ok(spec) -> {
              // Generate ready document
              let ready_doc = ready_document.generate_ready_document(spec)

              // Determine output path
              let out_dir = case output_dir {
                "" -> "."
                _ -> output_dir
              }

              let output_path =
                out_dir <> "/" <> "ready.md"

              // Write document
              case simplifile.write(output_path, ready_doc) {
                Ok(_) -> {
                  io.println("\nOutput: " <> output_path)
                  cli_ui.print_success("Ready document generated successfully")
                  exit(exit_pass)
                }
                Error(_) -> {
                  cli_ui.print_error("Failed to write ready document")
                  exit(exit_fail)
                }
              }
            }
          }
        }
      }
    }
  }
}

/// ============================================================================
/// EFFECTS COMMAND
/// ============================================================================
fn effects_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let json =
      flag.get_bool(input.flags, "json")
      |> result.unwrap(False)

    let behavior =
      flag.get_string(input.flags, "behavior")
      |> result.unwrap("")

    // Validate argument count (0 or 1)
    case input.args {
      [] -> {
        cli_ui.print_error("Error: spec file required")
        exit(exit_fail)
      }
      [spec_file] -> {
        analyze_effects(spec_file, behavior, json)
      }
      _ -> {
        cli_ui.print_error("Error: effects command takes at most one argument")
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Analyze behaviors for second-order effects")
  |> glint.flag(
    "behavior",
    flag.string()
      |> flag.default("")
      |> flag.description("Analyze specific behavior only"),
  )
  |> glint.flag(
    "json",
    flag.bool()
      |> flag.default(False)
      |> flag.description("Output as JSON"),
  )
}

fn analyze_effects(
  spec_file: String,
  behavior_filter: String,
  as_json: Bool,
) -> Nil {
  cli_ui.print_header("Second-Order Effects Analysis")

  // Validate file path for security
  case security.validate_file_path(spec_file) {
    Error(err) -> {
      cli_ui.print_error("Invalid file path: " <> security.format_security_error(err))
      exit(exit_fail)
    }
    Ok(validated_path) -> {
      // Verify file exists
      case simplifile.verify_is_file(validated_path) {
        Ok(False) -> {
          cli_ui.print_error("Spec file not found: " <> spec_file)
          exit(exit_fail)
        }
        Error(_) -> {
          cli_ui.print_error("Cannot access file: " <> spec_file)
          exit(exit_fail)
        }
        Ok(True) -> {
          // Export CUE to JSON
          case shellout.command("cue", ["export", validated_path], ".", []) {
            Ok(json_str) -> {
              // Parse JSON to get spec
              case json.decode(json_str, dynamic.dynamic) {
                Ok(json_data) -> {
                  case parser.decode_dynamic(json_data) {
                    Ok(spec) -> {
                      // Analyze effects
                      let analysis = effects_analyzer.analyze_spec(spec)

                      // Filter by behavior if requested
                      let filtered_analysis = case behavior_filter {
                        "" -> analysis
                        _ -> filter_by_behavior(analysis, behavior_filter)
                      }

                      // Output results
                      case as_json {
                        True -> output_effects_json(filtered_analysis)
                        False -> output_effects_cli(filtered_analysis)
                      }

                      cli_ui.print_success("Effects analysis complete")
                      exit(exit_pass)
                    }
                    Error(parse_errors) -> {
                      cli_ui.print_error("Failed to parse spec:")
                      io.println(format_parse_errors(parse_errors))
                      exit(exit_fail)
                    }
                  }
                }
                Error(_) -> {
                  cli_ui.print_error(
                    "Failed to decode JSON output from CUE export",
                  )
                  exit(exit_fail)
                }
              }
            }
            Error(#(_, stderr)) -> {
              cli_ui.print_error("Failed to export CUE to JSON:")
              io.println(stderr)
              exit(exit_fail)
            }
          }
        }
      }
    }
  }
}

fn filter_by_behavior(
  analysis: SpecAnalysis,
  behavior_name: String,
) -> SpecAnalysis {
  let filtered_behaviors =
    list.filter(analysis.behavior_effects, fn(behavior_effect) {
      behavior_effect.behavior_name == behavior_name
    })

  effects_analyzer.SpecAnalysis(
    spec_name: analysis.spec_name,
    behavior_effects: filtered_behaviors,
  )
}

fn output_effects_cli(analysis: SpecAnalysis) -> Nil {
  io.println("\nSpec: " <> analysis.spec_name)
  io.println("")

  case analysis.behavior_effects {
    [] -> {
      io.println("No behaviors found matching the criteria")
      cli_ui.print_warning("No behaviors to analyze")
    }
    _ -> {
      // Display each behavior's effects
      list.each(analysis.behavior_effects, fn(behavior_effect) {
        let output =
          effects_analyzer.format_effects_cli(
            behavior_effect.behavior_name,
            behavior_effect.effects,
          )
        io.println(output)
        io.println("")
      })
    }
  }
}

fn output_effects_json(analysis: SpecAnalysis) -> Nil {
  // Build JSON structure for all behaviors
  let behaviors_json =
    list.map(analysis.behavior_effects, fn(behavior_effect) {
      let effects_json =
        list.map(behavior_effect.effects, fn(effect) {
          json.object([
            #("type", json.string(effect_type_to_json_string(effect.type_))),
            #("description", json.string(effect.description)),
            #("severity", json.string(severity_to_json_string(effect.severity))),
            #("suggestion", json.string(effect.suggestion)),
          ])
        })

      #(
        behavior_effect.behavior_name,
        json.array(from: effects_json, of: fn(_) { json.object([]) }),
      )
    })

  let output_json =
    json.object([
      #("spec", json.string(analysis.spec_name)),
      ..behaviors_json,
    ])
    |> json.to_string()

  io.println(output_json)
}

fn effect_type_to_json_string(
  type_: EffectType,
) -> String {
  case type_ {
    StateChange -> "state_change"
    Notification -> "notification"
    Cascade -> "cascade"
    RaceCondition -> "race_condition"
    RollbackRequired -> "rollback_required"
  }
}

fn severity_to_json_string(severity: effects_analyzer.Severity) -> String {
  case severity {
    High -> "high"
    Medium -> "medium"
    Low -> "low"
  }
}

fn format_parse_errors(
  errors: List(dynamic.DecodeError),
) -> String {
  errors
  |> list.map(fn(err) {
    "  - "
    <> string.join(err.path, ".")
    <> ": expected "
    <> err.expected
    <> ", found "
    <> err.found
  })
  |> string.join("\n")
}

@external(erlang, "erlang", "halt")
fn exit(code: Int) -> Nil
