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
import intent/batch
import intent/bead_templates
import intent/cli_ui
import intent/effects_analyzer.{
  type EffectType, type SpecAnalysis, Cascade, High, Low, Medium, Notification,
  RaceCondition, RollbackRequired, StateChange,
}
import intent/env
import intent/flag_suggestions
import intent/init_prompt
import intent/interview
import intent/interview_storage
import intent/loader
import intent/parser
import intent/plan_emit_beads
import intent/plan_mode
import intent/ready_document
import intent/security
import intent/spec_templates
import intent/spec_validator
import intent/validation
import intent/vision_document
import shellout
import simplifile

/// Exit codes
const exit_pass = 0

const exit_fail = 1

@external(erlang, "erlang", "halt")
fn exit(code: Int) -> Nil

/// List of all available commands
const available_commands = [
  "init", "interview", "beads", "bead-status", "history", "version", "diff",
  "sessions", "plan", "plan-next", "plan-approve", "plan-emit-beads",
  "beads-regenerate", "vision", "ready", "effects", "validate", "batch",
]

/// Check if a command name exists in the available commands list
pub fn command_exists(name: String) -> Bool {
  case list.find(available_commands, fn(cmd) { cmd == name }) {
    Ok(_) -> True
    Error(_) -> False
  }
}

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
        ["help", command, ..rest] -> {
          // Validate that command exists before adding --help
          case command_exists(command) {
            True -> [command, "--help", ..rest]
            False -> {
              io.println_error("error: unknown command: " <> command)
              io.println("")
              io.println(
                "Available commands: init, interview, beads, bead-status, history, version, diff, sessions, plan, plan-next, plan-approve, plan-emit-beads, beads-regenerate, vision, ready, effects, validate, batch",
              )
              exit(exit_fail)
              ["--help"]
            }
          }
        }
        _ -> ["--help"]
      }
    }
    _ -> normalize_cli_args(args)
  }

  let app =
    glint.new()
    |> glint.with_name("intent")
    |> glint.with_pretty_help(glint.default_pretty_help())
    |> glint.add(at: ["init"], do: init_command())
    |> glint.add(at: ["interview"], do: interview_command())
    |> glint.add(at: ["beads"], do: beads_command())
    |> glint.add(at: ["bead-status"], do: bead_status_command())
    |> glint.add(at: ["history"], do: history_command())
    |> glint.add(at: ["version"], do: version_command())
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
    // Validation command
    |> glint.add(at: ["validate"], do: validate_command())
    // Batch command
    |> glint.add(at: ["batch"], do: batch_command())

  // Validate flags before executing glint
  case flag_suggestions.validate_flags(processed_args) {
    Error(message) -> {
      cli_ui.print_error(message)
      io.println("")
      exit(exit_fail)
    }
    Ok(_) -> {
      // Flags are valid, proceed with glint execution
      case glint.execute(app, processed_args) {
    Ok(glint.Out(_)) -> {
      // Extract the command name from args to show in error
      let assert [cmd, ..] = processed_args
      let command_name = case cmd, cmd != "--help", cmd != "-" {
        c, True, True -> c
        _, _, _ -> "unknown"
      }
      io.println_error("error: command not found: " <> command_name)
      io.println("")
      io.println(
        "Available commands: init, interview, beads, bead-status, history, version, diff, sessions, plan, plan-next, plan-approve, plan-emit-beads, beads-regenerate, vision, ready, effects, validate, batch",
      )
      exit(exit_fail)
    }
    Ok(glint.Help(help_text)) -> {
      io.println(help_text)
      exit(exit_pass)
    }
    Error(_err) -> {
      // Extract the command name from args to show in error
      let command_name = case processed_args {
        [cmd, ..] -> {
          case cmd, cmd == "--help", string.starts_with(cmd, "-") {
            c, False, False -> c
            _, _, _ -> "unknown"
          }
        }
        _ -> "unknown"
      }
      io.println_error("error: command not found: " <> command_name)
      io.println("")
      io.println(
        "Available commands: init, interview, beads, bead-status, history, version, diff, sessions, plan, plan-next, plan-approve, plan-emit-beads, beads-regenerate, vision, ready, effects, validate, batch",
      )
      exit(exit_fail)
    }
      }
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
    "no-config" -> True
    "parallel" -> True
    "continue-on-error" -> True
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
    "feature" -> True
    "export-answers-template" -> True
    "vision" -> True
    "dir" -> True
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
/// INIT COMMAND
/// ============================================================================
fn init_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let template =
      flag.get_string(input.flags, "profile")
      |> result.unwrap("")

    let output =
      flag.get_string(input.flags, "output")
      |> result.unwrap("")

    // Validate argument count (0 or 1 - spec name)
    case input.args {
      [] -> {
        // Interactive mode - prompt for everything
        run_init_interactive(template, output)
      }
      [spec_name] -> {
        // Use provided spec name
        run_init_with_name(spec_name, template, output)
      }
      _ -> {
        cli_ui.print_error("Error: init takes at most one argument (spec name)")
        io.println(
          "\nUsage: intent init [<name>] [--profile <template>] [--output <file>]",
        )
        exit(exit_fail)
      }
    }
  })
  |> glint.description(
    "Initialize a new Intent spec from a template

Creates a new Intent specification file (.cue) from a template. Templates
provide a starting point with common patterns, examples, and best practices
pre-configured for different project types.

Interactive mode (no arguments):
  intent init

Quick start with flags:
  intent init \"My API\" --profile api-spec --output my-api.cue

Available templates:
  api-spec       - REST/GraphQL API with endpoints and auth
  cli-tool       - Command-line tool with commands and flags
  data-pipeline  - Data processing with ETL operations
  workflow       - Business workflow with states and transitions

The generated spec includes:
  - Pre-configured features and behaviors
  - Helpful comments and TODO markers
  - Anti-patterns with good/bad examples
  - AI implementation hints
  - Security best practices

Next steps:
  1. Customize the spec for your needs
  2. Run: intent validate <spec-file>
  3. Generate docs: intent vision <spec-file>",
  )
  |> glint.flag(
    "profile",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Template to use (api-spec|cli-tool|data-pipeline|workflow)

Skips interactive template selection when provided.",
      ),
  )
  |> glint.flag(
    "output",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Output filename (default: <name>.cue)

Skips interactive filename prompt when provided.",
      ),
  )
}

fn run_init_interactive(template_flag: String, output_flag: String) -> Nil {
  cli_ui.print_header("Initialize New Spec")

  // Prompt for spec name
  let spec_name = case init_prompt.prompt_spec_name() {
    Ok(name) -> name
    Error(err) -> {
      cli_ui.print_error("Failed to get spec name: " <> err)
      exit(exit_fail)
      ""
      // Never reached, but needed for type consistency
    }
  }

  run_init_with_name(spec_name, template_flag, output_flag)
}

fn run_init_with_name(
  spec_name: String,
  template_flag: String,
  output_flag: String,
) -> Nil {
  cli_ui.print_header("Initialize: " <> spec_name)

  // Validate spec name
  let valid_name = case spec_templates.validate_spec_name(spec_name) {
    Ok(name) -> name
    Error(err) -> {
      cli_ui.print_error(err)
      exit(exit_fail)
      ""
      // Never reached, but needed for type consistency
    }
  }

  // Determine template type
  let template_type = case template_flag {
    "" -> {
      // Interactive: prompt for template
      case init_prompt.prompt_template(spec_templates.list_templates()) {
        Ok(t) -> t
        Error(err) -> {
          cli_ui.print_error("Failed to get template selection: " <> err)
          exit(exit_fail)
          spec_templates.ApiSpec
          // Never reached, but needed for type consistency
        }
      }
    }
    _ -> {
      // Use template from flag
      case spec_templates.find_template(template_flag) {
        Ok(t) -> t
        Error(err) -> {
          cli_ui.print_error(err)
          exit(exit_fail)
          spec_templates.ApiSpec
          // Never reached, but needed for type consistency
        }
      }
    }
  }

  // Generate package name
  let package_name = spec_templates.generate_package_name(valid_name)

  // Generate spec content
  let spec_content =
    spec_templates.generate_spec(template_type, valid_name, package_name)

  // Determine output filename
  let output_file = case output_flag {
    "" -> {
      // Interactive: prompt for filename
      let default_name = string.lowercase(package_name) <> ".cue"

      case init_prompt.prompt_output_filename(default_name) {
        Ok(name) -> name
        Error(err) -> {
          cli_ui.print_error("Failed to get output filename: " <> err)
          exit(exit_fail)
          ""
          // Never reached, but needed for type consistency
        }
      }
    }
    _ -> {
      // Use output from flag
      case string.ends_with(output_flag, ".cue") {
        True -> output_flag
        False -> output_flag <> ".cue"
      }
    }
  }

  // Write spec file
  case simplifile.write(output_file, spec_content) {
    Ok(_) -> {
      io.println("")
      io.println("Created: " <> ansi.green(output_file))
      io.println("")

      cli_ui.print_success("Spec initialized successfully!")
      io.println("")
      io.println(
        "Template: " <> spec_templates.format_template_type(template_type),
      )
      io.println("")
      io.println("Next steps:")
      io.println("  1. Review and customize the spec:")
      io.println("     " <> ansi.cyan("cat " <> output_file))
      io.println("")
      io.println("  2. Validate the spec:")
      io.println("     " <> ansi.cyan("intent validate " <> output_file))
      io.println("")
      io.println("  3. Generate documentation:")
      io.println("     " <> ansi.cyan("intent vision " <> output_file))
      io.println("")
      exit(exit_pass)
    }
    Error(_err) -> {
      cli_ui.print_error("Failed to write spec file")
      io.println("\nError details:")
      io.println("\nTroubleshooting:")
      io.println("  1. Check file permissions: ls -l " <> output_file)
      io.println("  2. Verify directory exists: ls .")
      io.println("  3. Check disk space: df -h")
      exit(exit_fail)
    }
  }
}

/// ============================================================================
/// INTERVIEW COMMAND
/// ============================================================================
fn interview_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let profile =
      flag.get_string(input.flags, "profile")
      |> result.unwrap(env.get_default_profile() |> result.unwrap(""))

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
    "Run interactive interview session to capture requirements

Starts an interactive questionnaire that captures project requirements and
generates structured specifications. Supports different profiles for various
project types (API, CLI, event-driven, data pipelines, workflows, UI).

Common workflows:
  - New API project:    intent interview --profile api
  - Resume session:     intent interview --resume <session-id>
  - List sessions:      intent history

Related commands:
  - history     View all interview sessions
  - diff        Show changes in a session
  - beads       Generate beads from completed session",
  )
  |> glint.flag(
    "profile",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Profile type for interview questions (api|cli|event|data|workflow|ui)

Profiles customize questions for different project types:
  api      - REST/GraphQL APIs (endpoints, data models, authentication)
  cli      - Command-line tools (commands, arguments, exit codes)
  event    - Event-driven systems (events, handlers, subscriptions)
  data     - Data pipelines (sources, transformations, destinations)
  workflow - Business workflows (states, transitions, actors)
  ui       - User interfaces (components, interactions, states)",
      ),
  )
  |> glint.flag(
    "resume",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Resume a previous interview session by ID

Use 'intent history' to list available sessions, then resume with the
session ID to continue answering questions or modify previous answers.",
      ),
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
      |> result.unwrap(env.get_default_format())

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
  |> glint.description(
    "Generate beads (tasks) from completed interview session

Converts interview answers into structured task beads for use with br
(beads issue tracking). Each behavior becomes a bead with appropriate
priority, dependencies, and descriptions.

Workflow:
  1. Complete interview:   intent interview --profile api
  2. Generate beads:       intent beads --session <id> --format json
  3. Review beads:         Check generated beads in output
  4. Emit to br:           intent plan-emit-beads <session-id> --execute

Output formats:
  json     - JSON array of bead objects (default)
  jsonl    - JSON Lines (one bead per line)
  markdown - Human-readable bead list

Related commands:
  - beads-regenerate  Regenerate beads with updated templates
  - plan-emit-beads   Emit beads to br (idempotent)",
  )
  |> glint.flag(
    "session",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Session ID to generate beads from (required)

Find session IDs using: intent history or intent sessions",
      ),
  )
  |> glint.flag(
    "format",
    flag.string()
      |> flag.default("json")
      |> flag.description(
        "Output format for beads (default: json)

Formats:
  json     - JSON array with all beads (default)
  jsonl    - JSON Lines (one JSON object per line)
  markdown - Human-readable formatted list

Example output files: beads.json, beads.jsonl, beads.md",
      ),
  )
  |> glint.flag(
    "out",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Output directory for generated beads (default: current directory)

Beads are written to <outdir>/beads.<format> (e.g., beads.json)",
      ),
  )
}

fn generate_beads(
  session_id: String,
  format: String,
  _output_dir: String,
) -> Nil {
  cli_ui.print_header("Generate Beads")

  let sessions_path = ".interview/sessions.jsonl"

  // Validate session exists before proceeding
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
    Ok(_current_session) -> {
      // Session exists, proceed with bead generation
      io.println("Session: " <> session_id)
      io.println("Format: " <> format)
      cli_ui.print_success("Beads command - implementation needed")
      exit(exit_pass)
    }
  }
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
  |> glint.description(
    "Check the status of a bead in br (beads)

Queries br for detailed information about a specific bead, including:
- Current status (pending, in_progress, completed, blocked, etc.)
- Priority level (P1-P4)
- Title and description
- Creation and update timestamps
- Labels and metadata

Example:
  intent bead-status --bead-id intent-cli-113

Related commands:
  - br list    List all beads
  - br ready   Find ready beads",
  )
  |> glint.flag(
    "bead-id",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Bead ID to check (required)

Use full bead ID (e.g., intent-cli-113). Partial IDs may be ambiguous.
Find bead IDs with: br list",
      ),
  )
}

fn check_bead_status(bead_id: String) -> Nil {
  cli_ui.print_header("Bead Status")

  // Query br for bead status
  case shellout.command("br", ["show", "--json", bead_id], ".", []) {
    Ok(json_output) -> {
      // Parse JSON response
      case json.decode(json_output, dynamic.dynamic) {
        Ok(json_data) -> {
          case parse_bead_response(json_data) {
            Ok(bead) -> display_bead_status(bead)
            Error(err) -> {
              cli_ui.print_error("Failed to parse bead response")
              io.println("
Specific error: " <> err)
              io.println(
                "
This usually indicates a problem with br's JSON output format.",
              )
              io.println(
                "
Troubleshooting:",
              )
              io.println("  1. Check br version: br --version")
              io.println("  2. Verify br is working: br list")
              io.println("  3. Check br logs: br log --tail 50")
              exit(exit_fail)
            }
          }
        }
        Error(_) -> {
          cli_ui.print_error("Failed to decode JSON response from br")
          io.println(
            "
Raw output:",
          )
          io.println(json_output)
          io.println(
            "
Troubleshooting:",
          )
          io.println("  1. Check br version: br --version")
          io.println("  2. Verify br is working: br list")
          exit(exit_fail)
        }
      }
    }
    Error(#(_, stderr)) -> {
      // Check if it's a "bead not found" error
      case
        string.contains(stderr, "not found")
        || string.contains(stderr, "No issue found")
        || string.contains(stderr, "AMBIGUOUS_ID")
      {
        True -> {
          cli_ui.print_error("Bead not found: " <> bead_id)
          io.println(
            "
Tip: Use a full bead ID (e.g., intent-cli-113)",
          )
          io.println("List all beads with: br list")
          io.println(
            "
If you expected this bead to exist:",
          )
          io.println("  1. Check the bead ID for typos")
          io.println("  2. Verify you're in the correct project")
          io.println("  3. Ensure beads have been emitted to br")
          exit(exit_fail)
        }
        False -> {
          format_br_command_error(stderr)
          exit(exit_fail)
        }
      }
    }
  }
}

/// Bead status from br
type BeadStatus {
  BeadStatus(
    id: String,
    title: String,
    description: String,
    status: String,
    priority: Int,
    issue_type: String,
    created_at: String,
    updated_at: String,
    closed_at: String,
    labels: List(String),
  )
}

/// Parse bead response from br show command
fn parse_bead_response(json_data: dynamic.Dynamic) -> Result(BeadStatus, String) {
  // br show returns an array, try to decode as list first
  case dynamic.list(dynamic.dynamic)(json_data) {
    Ok(beads_list) -> {
      case beads_list {
        [] -> Error("Empty bead list returned from br")
        [first_bead, ..] -> decode_bead_from_dynamic(first_bead)
      }
    }
    Error(_) -> {
      // If not an array, try to decode as a single bead
      decode_bead_from_dynamic(json_data)
    }
  }
}

/// Decode a single bead from dynamic
fn decode_bead_from_dynamic(
  bead_dynamic: dynamic.Dynamic,
) -> Result(BeadStatus, String) {
  let id = dynamic.field("id", dynamic.string)(bead_dynamic)
  let title = dynamic.field("title", dynamic.string)(bead_dynamic)
  let status = dynamic.field("status", dynamic.string)(bead_dynamic)
  let priority = dynamic.field("priority", dynamic.int)(bead_dynamic)
  let created_at = dynamic.field("created_at", dynamic.string)(bead_dynamic)
  let updated_at = dynamic.field("updated_at", dynamic.string)(bead_dynamic)

  // Optional fields - use field with fallback
  let description =
    dynamic.field("description", dynamic.string)(bead_dynamic)
    |> result.unwrap("")
  let issue_type =
    dynamic.field("issue_type", dynamic.string)(bead_dynamic)
    |> result.unwrap("unknown")
  let closed_at =
    dynamic.field("closed_at", dynamic.string)(bead_dynamic)
    |> result.unwrap("")
  let labels =
    dynamic.field("labels", dynamic.list(dynamic.string))(bead_dynamic)
    |> result.unwrap([])

  case id, title, status, priority, created_at, updated_at {
    Ok(id_val),
      Ok(title_val),
      Ok(status_val),
      Ok(priority_val),
      Ok(created_val),
      Ok(updated_val)
    -> {
      Ok(BeadStatus(
        id: id_val,
        title: title_val,
        description: description,
        status: status_val,
        priority: priority_val,
        issue_type: issue_type,
        created_at: created_val,
        updated_at: updated_val,
        closed_at: closed_at,
        labels: labels,
      ))
    }
    _, _, _, _, _, _ -> {
      Error("Failed to decode required bead fields")
    }
  }
}

/// Display bead status in formatted output
fn display_bead_status(bead: BeadStatus) -> Nil {
  // Bead ID and title
  cli_ui.print_labeled("ID", bead.id)
  cli_ui.print_labeled("Title", bead.title)

  // Status with color
  let status_display = case bead.status {
    "pending" -> ansi.green(bead.status)
    "in_progress" -> ansi.yellow(bead.status)
    "completed" -> ansi.green(bead.status)
    "closed" -> ansi.green(bead.status)
    "blocked" -> ansi.red(bead.status)
    _ -> ansi.white(bead.status)
  }
  cli_ui.print_labeled("Status", status_display)

  // Priority and type
  let priority_emoji = case bead.priority {
    1 -> "🔴 P1"
    2 -> "🟠 P2"
    3 -> "🟡 P3"
    4 -> "🟢 P4"
    _ -> "⚪ P" <> int.to_string(bead.priority)
  }
  cli_ui.print_labeled(
    "Priority",
    priority_emoji <> " (" <> int.to_string(bead.priority) <> ")",
  )
  cli_ui.print_labeled("Type", bead.issue_type)

  // Labels
  case bead.labels {
    [] -> Nil
    labels -> {
      let labels_str =
        labels
        |> list.map(fn(l) { "#" <> l })
        |> string.join(", ")
      cli_ui.print_labeled("Labels", labels_str)
    }
  }

  // Description (if present)
  case bead.description {
    "" -> Nil
    desc -> {
      io.println("")
      io.println(ansi.bold("Description:"))
      io.println(desc)
    }
  }

  // Timestamps
  io.println("")
  cli_ui.print_labeled("Created", format_iso_timestamp(bead.created_at))
  cli_ui.print_labeled("Updated", format_iso_timestamp(bead.updated_at))

  case bead.closed_at {
    "" -> Nil
    closed -> cli_ui.print_labeled("Closed", format_iso_timestamp(closed))
  }

  // Success
  io.println("")
  cli_ui.print_success("Bead status retrieved successfully")
  exit(exit_pass)
}

/// Format ISO timestamp for display
fn format_iso_timestamp(iso: String) -> String {
  // Parse ISO timestamp and format as YYYY-MM-DD HH:MM
  let parts = string.split(iso, "T")
  case parts {
    [date, time] -> {
      let time_parts = string.split(time, ":")
      case time_parts {
        [hour, minute, _] -> date <> " " <> hour <> ":" <> minute
        _ -> iso
      }
    }
    _ -> iso
  }
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
  |> glint.description(
    "List all interview sessions in chronological order

Displays a table of all interview sessions with:
- Session ID (for resuming or generating beads)
- Profile type (api, cli, event, etc.)
- Creation timestamp
- Number of answers completed

This is the primary command for finding session IDs to use with other commands.

Example output:
  Session ID    Profile  Created              Answers
  ─────────────────────────────────────────────────────
  abc123        api      2024-01-15 14:30     12
  def456        cli      2024-01-14 09:15     8

Related commands:
  - interview    Start a new session
  - diff         Show changes in a specific session
  - sessions     List sessions with filtering options

Flag shortcuts:
  -h, --help          Show help
  -v, --version       Show version",
  )
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
    Error(interview_storage.FileNotFound(_)) -> {
      // File doesn't exist - show friendly message
      cli_ui.print_warning("No sessions found")
      io.println("")
      io.println("Start a new session with:")
      io.println("  intent interview --profile <profile>")
      exit(exit_pass)
    }
    Error(interview_storage.FileCorrupt(_err)) -> {
      // File is corrupt - error already displayed by interview_storage
      // Just exit with failure
      exit(exit_fail)
    }
    Error(interview_storage.OtherError(err)) -> {
      cli_ui.print_error("Failed to load sessions: " <> err)
      exit(exit_fail)
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
/// VERSION COMMAND
/// ============================================================================
fn version_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    // Validate no extra arguments
    case validation.validate_no_args(input.args, "version") {
      Ok(Nil) -> {
        io.println("intent v0.1.0")
        exit(exit_pass)
      }
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description(
    "Show the version number

Displays the current version of intent.

Example:
  intent version

Related commands:
  - --version    Show version as a flag
  - --help       Show help information",
  )
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
  |> glint.description(
    "Show diff of changes for a specific session

Compares the current state of a session with its previous snapshot,
showing what answers were added, modified, or removed during the interview.

Use this to:
- Review changes made in a session
- Understand how requirements evolved
- Identify what answers were added in each round

Example:
  intent diff --session abc123

Output shows:
  + Added answers (new requirements)
  - Removed answers (changed requirements)
  ~ Modified answers (updated requirements)

Related commands:
  - history     List all sessions
  - interview   Resume and modify a session",
  )
  |> glint.flag(
    "session",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Session ID to show diff for (required)

Find session IDs with: intent history",
      ),
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
          io.println(
            "  Profile: "
            <> interview.profile_to_string(current_session.profile),
          )
          io.println(
            "  Stage: " <> interview.stage_to_string(current_session.stage),
          )
          io.println("  Updated: " <> current_session.updated_at)
          io.println(
            "  Answers: " <> int.to_string(list.length(current_session.answers)),
          )
          io.println(
            "  Unresolved gaps: "
            <> int.to_string(
              list.length(
                list.filter(current_session.gaps, fn(g) { !g.resolved }),
              ),
            ),
          )
          io.println(
            "  Unresolved conflicts: "
            <> int.to_string(
              list.length(
                list.filter(current_session.conflicts, fn(c) { c.chosen < 0 }),
              ),
            ),
          )
          exit(exit_pass)
        }
        Ok(snapshots) -> {
          case snapshots {
            [] -> {
              // No snapshots yet
              cli_ui.print_warning(
                "No previous snapshots found for this session",
              )
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
                  let empty_session =
                    interview.InterviewSession(
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
                    interview_storage.diff_sessions(
                      empty_session,
                      current_session,
                    )

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

    // Validate no extra arguments and profile value
    case validation.validate_no_args(input.args, "sessions") {
      Ok(Nil) -> {
        case validation.validate_profile_filter(profile) {
          Ok(valid_profile) -> list_sessions(valid_profile)
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
  |> glint.description(
    "List interview sessions with optional filtering

Displays detailed information about interview sessions including:
- Session ID and profile type
- Stage (discovery, refinement, etc.)
- Creation and update timestamps
- Completion status
- Unresolved gaps and conflicts

Similar to 'history' but with more detailed information and filtering options.

Examples:
  intent sessions                    # List all sessions
  intent sessions --profile api       # List only API sessions

Output shows status indicators:
  ✓ Ready       - No gaps or conflicts
  ⚠ N gap(s)    - Unresolved gaps
  ⚠ N conflict(s) - Unresolved conflicts

Related commands:
  - history    Simpler chronological list
  - diff       Show changes in a specific session

Flag shortcuts:
  -h, --help          Show help
  -v, --version       Show version
  -p, --profile       Filter by profile type",
  )
  |> glint.flag(
    "profile",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Filter by profile type (api|cli|event|data|workflow|ui)

Shows only sessions matching the specified profile type.",
      ),
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
    Error(interview_storage.FileNotFound(_)) -> {
      cli_ui.print_warning("No sessions found")
      exit(exit_pass)
    }
    Error(interview_storage.FileCorrupt(_err)) -> {
      // File is corrupt - error already displayed
      exit(exit_fail)
    }
    Error(interview_storage.OtherError(err)) -> {
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
  |> glint.description(
    "Generate execution plan from current project context

Analyzes the current project state and generates a structured execution plan
with phases, dependencies, and effort estimates. The plan considers:
- Unimplemented behaviors from specifications
- Current code state and gaps
- Dependencies between tasks
- Risk levels and blockers

Output includes:
- Total number of beads (tasks)
- Estimated effort
- Risk level assessment
- Phase breakdown with dependencies

Use --notes to provide additional context for plan generation.

Related commands:
  - plan-next        Suggest next task to work on
  - plan-approve     Approve a plan for execution
  - plan-emit-beads  Emit beads from plan",
  )
  |> glint.flag(
    "notes",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Additional context notes for plan generation

Use this to provide extra information about the current state,
constraints, or specific goals for the planning process.",
      ),
  )
}

/// ============================================================================
/// PLAN NEXT COMMAND
/// ============================================================================
fn plan_next_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let strategy =
      flag.get_string(input.flags, "strategy")
      |> result.unwrap(env.get_default_strategy())

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
  |> glint.description(
    "Suggest the next task to work on based on strategy

Analyzes the current plan and suggests the most appropriate next task
using one of several selection strategies. This helps maintain momentum
and ensures dependencies are properly respected.

Available strategies:
  page_rank       - Highest PageRank score (most connected/influential)
  critical_path   - Task on the critical path (affects overall timeline)
  shortest        - Quickest to complete (quick wins)
  risk_first      - Highest risk first (risk mitigation)

Example:
  intent plan-next --strategy critical_path

Output shows:
- Suggested task with ID
- Rationale for selection
- Dependencies and blockers
- Estimated effort

Related commands:
  - plan           Generate or update plan
  - plan-approve   Approve plan before execution",
  )
  |> glint.flag(
    "strategy",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Task selection strategy (default: page_rank)

Strategies:
  page_rank       - Most connected/influential task (default)
  critical_path   - Task affecting overall timeline
  shortest        - Quickest task to complete
  risk_first      - Highest risk task first

Each strategy optimizes for different goals:
  - page_rank: Maximizes downstream progress
  - critical_path: Minimizes overall timeline
  - shortest: Build momentum with quick wins
  - risk_first: Mitigate risks early",
      ),
  )
}

/// ============================================================================
/// PLAN APPROVE COMMAND
/// ============================================================================
fn plan_approve_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let notes =
      flag.get_string(input.flags, "notes")
      |> result.unwrap("")

    // Validate single argument
    case validation.validate_single_arg(input.args, "plan-approve") {
      Ok(plan_id) -> approve_plan(plan_id, notes)
      Error(err) -> {
        cli_ui.print_error(err)
        exit(exit_fail)
      }
    }
  })
  |> glint.description("Approve a generated plan for execution")
  |> glint.flag(
    "notes",
    flag.string()
      |> flag.default("")
      |> flag.description("Approval notes for documentation"),
  )
}

fn approve_plan(plan_id: String, notes: String) -> Nil {
  cli_ui.print_header("Approve Plan")

  // Compute the plan to validate it exists
  case plan_mode.compute_plan(plan_id) {
    Error(err) -> {
      cli_ui.print_error("Failed to load plan:")
      io.println(plan_mode.format_error(err))
      exit(exit_fail)
    }
    Ok(plan) -> {
      // Display plan summary
      io.println("")
      io.println("Session: " <> plan.session_id)
      io.println("Total Beads: " <> int.to_string(plan.total_beads))
      io.println("Total Effort: " <> plan.total_effort)
      io.println("Risk Level: " <> plan_mode.risk_to_string(plan.risk))
      io.println("Phases: " <> int.to_string(list.length(plan.phases)))

      // Show blockers if any
      case plan.blockers {
        [] -> Nil
        blockers -> {
          io.println("")
          cli_ui.print_warning("Blockers:")
          list.each(blockers, fn(blocker) { io.println("  • " <> blocker) })
        }
      }

      // Save approval
      save_plan_approval(plan_id, notes)
      display_approval_confirmation(plan_id)
      exit(exit_pass)
    }
  }
}

fn save_plan_approval(plan_id: String, notes: String) -> Nil {
  // Save approval metadata to .intent directory
  let approval_path = ".intent/plan-" <> plan_id <> "-approved.json"

  let timestamp = current_iso8601_timestamp()

  let approval_json =
    json.object([
      #("plan_id", json.string(plan_id)),
      #("approved_at", json.string(timestamp)),
      #("notes", json.string(notes)),
      #("status", json.string("approved")),
    ])
    |> json.to_string()

  case simplifile.write(approval_path, approval_json) {
    Ok(_) -> Nil
    Error(_err) -> {
      // Non-fatal: log warning but don't fail
      cli_ui.print_warning("Could not save approval metadata")
    }
  }
}

fn display_approval_confirmation(plan_id: String) -> Nil {
  io.println("")
  cli_ui.print_success("Plan approved: " <> plan_id)
  io.println("")
  io.println("Next steps:")
  io.println("  1. Review the plan phases")
  io.println("  2. Emit beads to br: intent plan-emit-beads " <> plan_id)
  io.println("  3. Execute beads in order")
  io.println("")
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
  cli_ui.print_header("Regenerate Beads")

  let sessions_path = ".interview/sessions.jsonl"

  // Try to load session from JSONL
  let load_result =
    interview_storage.get_session_from_jsonl(sessions_path, session_id)

  case load_result {
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
    Ok(session) -> {
      // Generate new beads from session
      let new_beads = bead_templates.generate_beads_from_session(session)

      // Display beads
      io.println("")
      io.println("Session: " <> session_id)
      io.println("")

      case list.is_empty(new_beads) {
        True -> cli_ui.print_warning("No beads generated from session")
        False -> {
          io.println("Generated beads:")
          list.each(new_beads, fn(bead) {
            io.println(
              "  - "
              <> bead.title
              <> " ["
              <> bead.profile_type
              <> " P"
              <> int.to_string(bead.priority)
              <> "]",
            )
          })

          // Show stats
          let stats = bead_templates.bead_stats(new_beads)
          io.println("")
          io.println("Bead Statistics:")
          io.println("  Total: " <> int.to_string(stats.total))
        }
      }

      cli_ui.print_success("Bead regeneration complete")
      exit(exit_pass)
    }
  }
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
      cli_ui.print_error(
        "Invalid file path: " <> security.format_security_error(err),
      )
      exit(exit_fail)
    }
    Ok(validated_path) -> {
      // Verify file exists
      case simplifile.verify_is_file(validated_path) {
        Ok(False) -> {
          format_file_not_found_with_context(spec_file, "Spec")
          exit(exit_fail)
        }
        Error(_) -> {
          cli_ui.print_error("Cannot access file: " <> spec_file)
          io.println(
            "
Troubleshooting:",
          )
          io.println("  1. Check file permissions: ls -l " <> spec_file)
          io.println("  2. Verify file is not locked by another process")
          io.println("  3. Ensure you have read access to the file")
          exit(exit_fail)
        }
        Ok(True) -> {
          // Export CUE to JSON
          case
            shellout.command(
              "cue",
              ["export", validated_path, "-e", "spec"],
              ".",
              [],
            )
          {
            Ok(json_str) -> {
              // Parse JSON to get spec
              case json.decode(json_str, dynamic.dynamic) {
                Ok(json_data) -> {
                  case parser.decode_dynamic(json_data) {
                    Ok(spec) -> {
                      // Generate vision document
                      let vision_doc =
                        vision_document.generate_vision_document(spec)

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
                          cli_ui.print_success(
                            "Vision document generated successfully",
                          )
                          exit(exit_pass)
                        }
                        Error(_err) -> {
                          cli_ui.print_error("Failed to write vision document:")
                          io.println(
                            "Could not write file - check permissions and disk space",
                          )
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
              cli_ui.print_error("Failed to export CUE spec to JSON")
              io.println("
Spec file: " <> spec_file)
              io.println(
                "
CUE error:",
              )
              io.println(stderr)
              io.println(
                "
Troubleshooting:",
              )
              io.println("  1. Validate CUE syntax: cue vet " <> spec_file)
              io.println("  2. Check CUE version: cue version")
              io.println("  3. Verify spec schema matches expected format")
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
      cli_ui.print_error(
        "Invalid file path: " <> security.format_security_error(err),
      )
      exit(exit_fail)
    }
    Ok(validated_path) -> {
      // Verify file exists
      case simplifile.verify_is_file(validated_path) {
        Ok(False) -> {
          format_file_not_found_with_context(spec_file, "Spec")
          exit(exit_fail)
        }
        Error(_) -> {
          cli_ui.print_error("Cannot access file: " <> spec_file)
          io.println(
            "
Troubleshooting:",
          )
          io.println("  1. Check file permissions: ls -l " <> spec_file)
          io.println("  2. Verify file is not locked by another process")
          io.println("  3. Ensure you have read access to the file")
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

              let output_path = out_dir <> "/" <> "ready.md"

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
      cli_ui.print_error(
        "Invalid file path: " <> security.format_security_error(err),
      )
      exit(exit_fail)
    }
    Ok(validated_path) -> {
      // Verify file exists
      case simplifile.verify_is_file(validated_path) {
        Ok(False) -> {
          format_file_not_found_with_context(spec_file, "Spec")
          exit(exit_fail)
        }
        Error(_) -> {
          cli_ui.print_error("Cannot access file: " <> spec_file)
          io.println(
            "
Troubleshooting:",
          )
          io.println("  1. Check file permissions: ls -l " <> spec_file)
          io.println("  2. Verify file is not locked by another process")
          io.println("  3. Ensure you have read access to the file")
          exit(exit_fail)
        }
        Ok(True) -> {
          // Export CUE to JSON
          case
            shellout.command(
              "cue",
              ["export", validated_path, "-e", "spec"],
              ".",
              [],
            )
          {
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
              cli_ui.print_error("Failed to export CUE spec to JSON")
              io.println("
Spec file: " <> spec_file)
              io.println(
                "
CUE error:",
              )
              io.println(stderr)
              io.println(
                "
Troubleshooting:",
              )
              io.println("  1. Validate CUE syntax: cue vet " <> spec_file)
              io.println("  2. Check CUE version: cue version")
              io.println("  3. Verify spec schema matches expected format")
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
    json.object([#("spec", json.string(analysis.spec_name)), ..behaviors_json])
    |> json.to_string()

  io.println(output_json)
}

fn effect_type_to_json_string(type_: EffectType) -> String {
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

fn format_parse_errors(errors: List(dynamic.DecodeError)) -> String {
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

/// ============================================================================
/// VALIDATE COMMAND
/// ============================================================================
fn validate_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let json =
      flag.get_bool(input.flags, "json")
      |> result.unwrap(False)

    // Validate single argument (spec file)
    case input.args {
      [] -> {
        cli_ui.print_error("Error: spec file required")
        io.println("\nUsage: intent validate <spec-file> [--json]")
        exit(exit_fail)
      }
      [spec_file] -> {
        validate_spec_file(spec_file, json)
      }
      _ -> {
        cli_ui.print_error(
          "Error: validate command takes exactly one argument (spec file)",
        )
        exit(exit_fail)
      }
    }
  })
  |> glint.description(
    "Validate a CUE spec file for syntax and schema compliance

Performs comprehensive validation of Intent spec files:
  • CUE syntax validity (file compiles)
  • Schema compliance (all required fields present)
  • No duplicate behavior names
  • No circular dependencies in requires
  • Valid behavior references (invariants exist, behaviors valid)

Exit codes:
  0 - Spec is valid
  1 - Spec has validation errors

Examples:
  intent validate examples/user-api.cue
  intent validate examples/user-api.cue --json

Output formats:
  Default - Human-readable error messages with line numbers
  -j, --json  - Machine-readable JSON output for CI/CD integration

Flag shortcuts:
  -j, --json          Output JSON",
  )
  |> glint.flag(
    "json",
    flag.bool()
      |> flag.default(False)
      |> flag.description("Output validation results as JSON"),
  )
}

fn validate_spec_file(spec_file: String, as_json: Bool) -> Nil {
  cli_ui.print_header("Spec Validation")

  // Validate file path for security
  case security.validate_file_path(spec_file) {
    Error(err) -> {
      cli_ui.print_error(
        "Invalid file path: " <> security.format_security_error(err),
      )
      exit(exit_fail)
    }
    Ok(validated_path) -> {
      // Run validation
      let result = spec_validator.validate_spec_file(validated_path)

      // Output results
      case as_json {
        True -> {
          io.println(spec_validator.format_validation_result_json(result))
        }
        False -> {
          io.println("\n" <> spec_validator.format_validation_result(result))

          // Show spec summary if valid
          case result {
            spec_validator.ValidationValid -> {
              case spec_validator.load_and_parse_spec(validated_path) {
                Ok(spec) -> {
                  io.println("\nSpec Summary:")
                  cli_ui.print_labeled("Name", spec.name)
                  cli_ui.print_labeled("Version", spec.version)
                  cli_ui.print_labeled(
                    "Features",
                    int.to_string(list.length(spec.features)),
                  )
                  let behavior_count =
                    spec.features
                    |> list.flat_map(fn(f) { f.behaviors })
                    |> list.length
                  cli_ui.print_labeled(
                    "Behaviors",
                    int.to_string(behavior_count),
                  )
                }
                Error(_) -> Nil
              }
            }
            spec_validator.ValidationInvalid(_) -> Nil
          }
        }
      }

      // Exit with appropriate code
      case result {
        spec_validator.ValidationValid -> {
          cli_ui.print_success("Validation passed")
          exit(exit_pass)
        }
        spec_validator.ValidationInvalid(_) -> {
          exit(exit_fail)
        }
      }
    }
  }
}

/// ============================================================================
/// ERROR HANDLING HELPERS
/// ============================================================================
/// Format file not found error with helpful suggestions
fn format_file_not_found_with_context(
  file_path: String,
  file_type: String,
) -> Nil {
  cli_ui.print_error(file_type <> " file not found: " <> file_path)

  io.println("\nTroubleshooting:")
  io.println("  1. Check the file path for typos")
  io.println("  2. Verify the file exists in the current directory")
  io.println("  3. List .cue files: find . -name '*.cue' -type f")
  io.println("  4. Check examples directory: ls examples/")
}

/// Format br command error with troubleshooting
fn format_br_command_error(stderr: String) -> Nil {
  cli_ui.print_error("Failed to execute br command")

  io.println("\nError details:")
  io.println(stderr)

  io.println("\nTroubleshooting:")
  io.println("  1. Ensure br is installed: br --version")
  io.println("  2. Check br configuration: br config show")
  io.println("  3. Verify repository is initialized: br list")
  io.println("  4. Check br logs: br log --tail 50")
}

@external(erlang, "intent_ffi", "current_iso8601_timestamp")
fn current_iso8601_timestamp() -> String

/// ============================================================================
/// BATCH COMMAND
/// ============================================================================
fn batch_command() -> glint.Command(Nil) {
  glint.command(fn(input: glint.CommandInput) {
    let output_dir =
      flag.get_string(input.flags, "out")
      |> result.unwrap("")

    let dir =
      flag.get_string(input.flags, "dir")
      |> result.unwrap("")

    let parallel =
      flag.get_bool(input.flags, "parallel")
      |> result.unwrap(False)

    let verbose =
      flag.get_bool(input.flags, "verbose")
      |> result.unwrap(False)

    let continue_on_error =
      flag.get_bool(input.flags, "continue-on-error")
      |> result.unwrap(False)

    // Determine files to process
    let files_result = case input.args {
      [] -> {
        // No files provided, check --dir flag
        case dir {
          "" -> Error("No spec files provided")
          _ -> batch.get_specs_from_dir(dir)
        }
      }
      args -> Ok(args)
    }

    // Handle file errors
    let files = case files_result {
      Ok(file_list) -> file_list
      Error(err) -> {
        cli_ui.print_error("Error: " <> err)
        io.println(
          "\nUsage: intent batch <spec1.cue> [<spec2.cue> ...] [--dir <directory>]",
        )
        io.println("\nExamples:")
        io.println("  intent batch spec1.cue spec2.cue spec3.cue")
        io.println("  intent batch --dir examples/")
        io.println("  intent batch --dir specs/ --verbose")
        exit(exit_fail)
        // This is unreachable but needed for type checking
        []
      }
    }

    // Create batch config
    let config =
      batch.BatchConfig(
        output_dir: output_dir,
        parallel: parallel,
        verbose: verbose,
        continue_on_error: continue_on_error,
      )

    // Process specs
    let summary = batch.process_specs(files, config)

    // Exit with appropriate code
    case summary.failed > 0 {
      True -> exit(exit_fail)
      False -> exit(exit_pass)
    }
  })
  |> glint.description(
    "Process multiple spec files and generate summary report

Processes multiple CUE spec files sequentially or in parallel, analyzing
each for quality and generating a comprehensive summary report.

Examples:
  # Process specific files
  intent batch spec1.cue spec2.cue spec3.cue

  # Process all .cue files in a directory
  intent batch --dir examples/

  # Verbose output with detailed results
  intent batch --dir specs/ --verbose

  # Continue processing even if some files fail
  intent batch *.cue --continue-on-error

  # Specify output directory for results
  intent batch --dir specs/ --out results/

Output includes:
  - Progress indicator: [1/3] processing spec1.cue...
  - Per-file status with behavior count and quality score
  - Summary statistics (successful, failed, skipped)
  - Average quality score across all specs
  - Detailed error messages for failed files (with --verbose)

Quality scoring:
  - 90-100: Excellent (comprehensive, well-tested)
  - 75-89:  Good (solid coverage, minor gaps)
  - 60-74:  Fair (basic coverage, needs improvement)
  - 0-59:   Poor (significant gaps or issues)

Related commands:
  - effects    Analyze second-order effects
  - vision     Generate vision document
  - ready      Generate ready document

Flag shortcuts:
  -d, --dir          Directory containing .cue spec files
  -o, --out          Output directory for results
  -j, --json         Output JSON (reserved for future use)",
  )
  |> glint.flag(
    "dir",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Directory containing .cue spec files

Process all .cue files in the specified directory. Hidden files
(starting with .) are ignored.

Example: intent batch --dir examples/",
      ),
  )
  |> glint.flag(
    "out",
    flag.string()
      |> flag.default("")
      |> flag.description(
        "Output directory for batch results (default: current directory)

Reserved for future use - currently results are printed to stdout.",
      ),
  )
  |> glint.flag(
    "parallel",
    flag.bool()
      |> flag.default(False)
      |> flag.description(
        "Process specs in parallel (experimental, not yet implemented)

Currently all specs are processed sequentially. This flag is reserved
for future parallel processing support.",
      ),
  )
  |> glint.flag(
    "verbose",
    flag.bool()
      |> flag.default(False)
      |> flag.description(
        "Enable verbose output with detailed results

Shows detailed breakdown for each spec including:
  - Quality issues found
  - Detailed error messages
  - Per-file result details",
      ),
  )
  |> glint.flag(
    "continue-on-error",
    flag.bool()
      |> flag.default(False)
      |> flag.description(
        "Continue processing remaining files if one fails

By default, batch processing stops on the first error. Use this flag
to continue processing all files and report all errors at the end.",
      ),
  )
}
