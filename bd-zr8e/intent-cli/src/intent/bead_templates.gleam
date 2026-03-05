/// Bead Templates
/// Generates work items (beads) from interview sessions for the `br` issue tracker
import gleam/dict.{type Dict}
import gleam/int
import gleam/json
import gleam/list
import gleam/result
import gleam/string
import intent/case_insensitive.{contains_any_ignore_case}
import intent/interview.{type InterviewSession, type Profile}

/// A single work item (bead) record
pub type BeadRecord {
  BeadRecord(
    title: String,
    description: String,
    profile_type: String,
    priority: Int,
    issue_type: String,
    labels: List(String),
    ai_hints: String,
    acceptance_criteria: List(String),
    dependencies: List(String),
  )
}

/// Generate beads from a completed interview session
pub fn generate_beads_from_session(
  session: InterviewSession,
) -> List(BeadRecord) {
  let profile_str = profile_to_string(session.profile)

  case session.profile {
    interview.Api -> generate_api_beads(session, profile_str)
    interview.Cli -> generate_cli_beads(session, profile_str)
    interview.Event -> generate_event_beads(session, profile_str)
    interview.Data -> generate_data_beads(session, profile_str)
    interview.Workflow -> generate_workflow_beads(session, profile_str)
    interview.UI -> generate_ui_beads(session, profile_str)
  }
}

/// Generate API endpoint beads
fn generate_api_beads(
  session: InterviewSession,
  profile: String,
) -> List(BeadRecord) {
  let endpoint_answers =
    list.filter(session.answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, ["endpoint", "path"])
    })

  list.map(endpoint_answers, fn(answer) {
    BeadRecord(
      title: "Implement API endpoint",
      description: answer.response,
      profile_type: profile,
      priority: 3,
      issue_type: "api_endpoint",
      labels: ["api", "endpoint", "implementation"],
      ai_hints: "Use interview response to build OpenAPI spec and implementation",
      acceptance_criteria: [
        "Endpoint responds with correct status code",
        "Response schema matches spec", "Error handling implemented",
        "Documentation added",
      ],
      dependencies: [],
    )
  })
}

/// Generate CLI command beads
fn generate_cli_beads(
  session: InterviewSession,
  profile: String,
) -> List(BeadRecord) {
  let command_answers =
    list.filter(session.answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, ["command", "subcommand"])
    })

  list.map(command_answers, fn(answer) {
    BeadRecord(
      title: "Implement CLI command",
      description: answer.response,
      profile_type: profile,
      priority: 3,
      issue_type: "cli_command",
      labels: ["cli", "command", "implementation"],
      ai_hints: "Review interview response for command syntax, options, and behavior",
      acceptance_criteria: [
        "Command parses arguments correctly", "Output format matches spec",
        "Help text is clear", "Error messages are helpful",
      ],
      dependencies: [],
    )
  })
}

/// Generate event beads
fn generate_event_beads(
  session: InterviewSession,
  profile: String,
) -> List(BeadRecord) {
  let event_answers =
    list.filter(session.answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, ["event", "message"])
    })

  list.map(event_answers, fn(answer) {
    BeadRecord(
      title: "Define and emit event",
      description: answer.response,
      profile_type: profile,
      priority: 2,
      issue_type: "event",
      labels: ["event", "messaging", "integration"],
      ai_hints: "Create event schema and producer/consumer implementation",
      acceptance_criteria: [
        "Event schema defined", "Producer implementation complete",
        "Consumer can subscribe", "Event routing working",
      ],
      dependencies: [],
    )
  })
}

/// Generate data model beads
fn generate_data_beads(
  session: InterviewSession,
  profile: String,
) -> List(BeadRecord) {
  let entity_answers =
    list.filter(session.answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, [
        "entity", "data model", "schema",
      ])
    })

  list.map(entity_answers, fn(answer) {
    BeadRecord(
      title: "Implement data model",
      description: answer.response,
      profile_type: profile,
      priority: 4,
      issue_type: "data_model",
      labels: ["data", "schema", "storage"],
      ai_hints: "Generate database schema and ORM/repository layer",
      acceptance_criteria: [
        "Schema migrations ready", "Validation rules implemented",
        "Indexes optimized", "Tests cover all fields",
      ],
      dependencies: [],
    )
  })
}

/// Generate workflow beads
fn generate_workflow_beads(
  session: InterviewSession,
  profile: String,
) -> List(BeadRecord) {
  let workflow_answers =
    list.filter(session.answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, [
        "workflow", "process", "step",
      ])
    })

  list.map(workflow_answers, fn(answer) {
    BeadRecord(
      title: "Implement workflow step",
      description: answer.response,
      profile_type: profile,
      priority: 2,
      issue_type: "workflow",
      labels: ["workflow", "orchestration", "automation"],
      ai_hints: "Design state machine and implement step logic",
      acceptance_criteria: [
        "State transitions working", "Error handling and retries",
        "Step completion detection", "Monitoring/logging implemented",
      ],
      dependencies: [],
    )
  })
}

/// Generate UI screen beads
fn generate_ui_beads(
  session: InterviewSession,
  profile: String,
) -> List(BeadRecord) {
  let screen_answers =
    list.filter(session.answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, [
        "screen", "view", "interface",
      ])
    })

  list.map(screen_answers, fn(answer) {
    BeadRecord(
      title: "Build UI screen",
      description: answer.response,
      profile_type: profile,
      priority: 2,
      issue_type: "ui_screen",
      labels: ["ui", "frontend", "component"],
      ai_hints: "Create mockup, component hierarchy, and responsive design",
      acceptance_criteria: [
        "All required fields present", "Responsive on mobile/tablet/desktop",
        "Accessibility standards met", "User testing completed",
      ],
      dependencies: [],
    )
  })
}

/// Convert bead record to JSONL line format (for .beads/issues.jsonl)
pub fn bead_to_jsonl_line(bead: BeadRecord) -> String {
  let json_list = [
    #("title", json.string(bead.title)),
    #("description", json.string(bead.description)),
    #("profile_type", json.string(bead.profile_type)),
    #("priority", json.int(bead.priority)),
    #("issue_type", json.string(bead.issue_type)),
    #("labels", json.array(bead.labels, json.string)),
    #("ai_hints", json.string(bead.ai_hints)),
    #("acceptance_criteria", json.array(bead.acceptance_criteria, json.string)),
    #("dependencies", json.array(bead.dependencies, json.string)),
  ]

  json.object(json_list)
  |> json.to_string
}

/// Format beads for output as JSONL (newline-delimited JSON)
pub fn beads_to_jsonl(beads: List(BeadRecord)) -> String {
  beads
  |> list.map(bead_to_jsonl_line)
  |> string.join("\n")
}

/// Render generated beads as schema-validated CUE entries.
/// Each entry is typed as #EnhancedBead so `cue vet schema/enhanced-bead.cue <file>`
/// validates every generated bead before persistence.
pub fn beads_to_enhanced_cue(beads: List(BeadRecord)) -> String {
  let entries =
    beads
    |> list.fold(#(1, []), fn(acc, bead) {
      let #(index, rendered) = acc
      #(index + 1, [render_enhanced_bead(bead, index), ..rendered])
    })
    |> fn(acc) {
      let #(_, rendered) = acc
      list.reverse(rendered)
    }

  "package schema\n\n" <> string.join(entries, "\n\n") <> "\n"
}

/// Deterministic bead ID used for schema files and validation references.
pub fn bead_id_for_index(bead: BeadRecord, index: Int) -> String {
  let component = profile_component(bead.profile_type)
  "intent-cli-" <> string.lowercase(component) <> string.inspect(index)
}

/// Render one bead entry suitable for a standalone CUE file.
pub fn enhanced_bead_entry(bead: BeadRecord, index: Int) -> String {
  render_enhanced_bead(bead, index)
}

/// Prefix description with planner-style CUE validation instructions.
pub fn with_validation_header(
  bead: BeadRecord,
  schema_path: String,
) -> BeadRecord {
  let prefixed =
    "# CUE Validation Schema\n"
    <> "# Validate implementation: cue vet "
    <> schema_path
    <> " implementation.cue\n"
    <> "# Schema location: "
    <> schema_path
    <> "\n\n"
    <> bead.description

  BeadRecord(..bead, description: prefixed)
}

fn render_enhanced_bead(bead: BeadRecord, index: Int) -> String {
  let component = profile_component(bead.profile_type)
  let normalized_title = component <> ": " <> fallback_title(bead.title)
  let id = bead_id_for_index(bead, index)
  let description = fallback_description(bead.description)
  let priority = normalize_priority(bead.priority)
  let labels = case bead.labels {
    [] -> [bead.profile_type, "generated"]
    existing -> existing
  }

  "bead_"
  <> string.inspect(index)
  <> ": #EnhancedBead & {\n"
  <> "  id: "
  <> cue_string(id)
  <> "\n"
  <> "  title: "
  <> cue_string(normalized_title)
  <> "\n"
  <> "  type: \"task\"\n"
  <> "  priority: "
  <> string.inspect(priority)
  <> "\n"
  <> "  effort_estimate: \"1hr\"\n"
  <> "  labels: "
  <> cue_string_list(labels)
  <> "\n\n"
  <> "  clarifications: {\n"
  <> "    clarification_status: \"RESOLVED\"\n"
  <> "  }\n\n"
  <> "  ears_requirements: {\n"
  <> "    ubiquitous: ["
  <> cue_string(
    "THE SYSTEM SHALL implement " <> string.lowercase(component) <> " behavior",
  )
  <> "]\n"
  <> "    event_driven: [{trigger: "
  <> cue_string("WHEN " <> string.lowercase(component) <> " work is executed")
  <> ", shall: "
  <> cue_string("THE SYSTEM SHALL complete the requested outcome")
  <> "}]\n"
  <> "    unwanted: [{condition: "
  <> cue_string("IF required inputs are missing")
  <> ", shall_not: "
  <> cue_string("THE SYSTEM SHALL NOT continue with invalid state")
  <> ", because: "
  <> cue_string("Invalid state causes unreliable execution")
  <> "}]\n"
  <> "  }\n\n"
  <> "  contracts: {\n"
  <> "    preconditions: {\n"
  <> "      auth_required: false\n"
  <> "      required_inputs: []\n"
  <> "      system_state: ["
  <> cue_string("Target codebase is available")
  <> "]\n"
  <> "    }\n"
  <> "    postconditions: {\n"
  <> "      state_changes: ["
  <> cue_string("Implementation state updated")
  <> "]\n"
  <> "      return_guarantees: []\n"
  <> "    }\n"
  <> "    invariants: ["
  <> cue_string("No silent failures are accepted")
  <> "]\n"
  <> "  }\n\n"
  <> "  research_requirements: {\n"
  <> "    files_to_read: [{\n"
  <> "      path: "
  <> cue_string("src/intent.gleam")
  <> "\n"
  <> "      what_to_extract: "
  <> cue_string("Existing CLI flow and command behavior")
  <> "\n"
  <> "      document_in: "
  <> cue_string("research_notes.md")
  <> "\n"
  <> "    }]\n"
  <> "    research_questions: [{\n"
  <> "      question: "
  <> cue_string("What existing pattern should this bead follow?")
  <> "\n"
  <> "      answered: false\n"
  <> "    }]\n"
  <> "    research_complete_when: ["
  <> cue_string("Key patterns are documented before changes")
  <> "]\n"
  <> "  }\n\n"
  <> "  inversions: {\n"
  <> "    usability_failures: [{\n"
  <> "      failure: "
  <> cue_string("User receives unclear output")
  <> "\n"
  <> "      prevention: "
  <> cue_string("Return actionable error and usage guidance")
  <> "\n"
  <> "      test_for_it: "
  <> cue_string("test_error_output_is_actionable")
  <> "\n"
  <> "    }]\n"
  <> "  }\n\n"
  <> "  acceptance_tests: {\n"
  <> "    happy_paths: [{\n"
  <> "      name: "
  <> cue_string("test_happy_path")
  <> "\n"
  <> "      given: "
  <> cue_string("Valid inputs")
  <> "\n"
  <> "      when: "
  <> cue_string("User runs the command")
  <> "\n"
  <> "      then: ["
  <> cue_string("Exit code is 0")
  <> ", "
  <> cue_string("Output matches expected behavior")
  <> "]\n"
  <> "      real_input: "
  <> cue_string(description)
  <> "\n"
  <> "      expected_output: "
  <> cue_string("Expected successful execution")
  <> "\n"
  <> "    }]\n"
  <> "    error_paths: [{\n"
  <> "      name: "
  <> cue_string("test_error_path")
  <> "\n"
  <> "      given: "
  <> cue_string("Invalid input")
  <> "\n"
  <> "      when: "
  <> cue_string("User runs the command")
  <> "\n"
  <> "      then: ["
  <> cue_string("Exit code is non-zero")
  <> ", "
  <> cue_string("Error message is clear")
  <> "]\n"
  <> "      real_input: "
  <> cue_string("invalid input")
  <> "\n"
  <> "      expected_output: null\n"
  <> "      expected_error: "
  <> cue_string("Actionable validation error")
  <> "\n"
  <> "    }]\n"
  <> "  }\n\n"
  <> "  e2e_tests: {\n"
  <> "    pipeline_test: {\n"
  <> "      name: "
  <> cue_string("test_full_pipeline")
  <> "\n"
  <> "      description: "
  <> cue_string("Validate full CLI workflow")
  <> "\n"
  <> "      setup: {}\n"
  <> "      execute: { command: "
  <> cue_string(
    "intent check examples/user-api.cue --target http://localhost:8080",
  )
  <> " }\n"
  <> "      verify: { exit_code: 0 }\n"
  <> "    }\n"
  <> "  }\n\n"
  <> "  verification_checkpoints: {\n"
  <> "    gate_0_research: {\n"
  <> "      name: "
  <> cue_string("Research Gate")
  <> "\n"
  <> "      must_pass_before: "
  <> cue_string("Writing code")
  <> "\n"
  <> "      checks: ["
  <> cue_string("Relevant files reviewed")
  <> "]\n"
  <> "      evidence_required: ["
  <> cue_string("Research notes recorded")
  <> "]\n"
  <> "    }\n"
  <> "    gate_1_tests: {\n"
  <> "      name: "
  <> cue_string("Test Gate")
  <> "\n"
  <> "      must_pass_before: "
  <> cue_string("Implementation")
  <> "\n"
  <> "      checks: ["
  <> cue_string("Failing tests exist")
  <> "]\n"
  <> "      evidence_required: ["
  <> cue_string("Test file added")
  <> "]\n"
  <> "    }\n"
  <> "    gate_2_implementation: {\n"
  <> "      name: "
  <> cue_string("Implementation Gate")
  <> "\n"
  <> "      must_pass_before: "
  <> cue_string("Completion")
  <> "\n"
  <> "      checks: ["
  <> cue_string("Tests pass")
  <> "]\n"
  <> "      evidence_required: ["
  <> cue_string("Test output captured")
  <> "]\n"
  <> "    }\n"
  <> "    gate_3_integration: {\n"
  <> "      name: "
  <> cue_string("Integration Gate")
  <> "\n"
  <> "      must_pass_before: "
  <> cue_string("Closing bead")
  <> "\n"
  <> "      checks: ["
  <> cue_string("Integration flow verified")
  <> "]\n"
  <> "      evidence_required: ["
  <> cue_string("Manual verification complete")
  <> "]\n"
  <> "    }\n"
  <> "  }\n\n"
  <> "  implementation_tasks: {\n"
  <> "    phase_0_research: {\n"
  <> "      parallelizable: true\n"
  <> "      tasks: [{ task: "
  <> cue_string("Review existing behavior")
  <> ", done_when: "
  <> cue_string("Research complete")
  <> ", parallel_group: "
  <> cue_string("research")
  <> " }]\n"
  <> "    }\n"
  <> "    phase_1_tests_first: {\n"
  <> "      parallelizable: true\n"
  <> "      gate_required: "
  <> cue_string("gate_0_research")
  <> "\n"
  <> "      tasks: [{ task: "
  <> cue_string("Write failing tests")
  <> ", done_when: "
  <> cue_string("Tests fail for expected reason")
  <> ", parallel_group: "
  <> cue_string("tests")
  <> " }]\n"
  <> "    }\n"
  <> "    phase_2_implementation: {\n"
  <> "      parallelizable: false\n"
  <> "      gate_required: "
  <> cue_string("gate_1_tests")
  <> "\n"
  <> "      tasks: [{ task: "
  <> cue_string("Implement required behavior")
  <> ", done_when: "
  <> cue_string("Tests pass")
  <> " }]\n"
  <> "    }\n"
  <> "    phase_4_verification: {\n"
  <> "      parallelizable: true\n"
  <> "      gate_required: "
  <> cue_string("gate_2_implementation")
  <> "\n"
  <> "      tasks: [{ task: "
  <> cue_string("Run CI verification")
  <> ", done_when: "
  <> cue_string("CI passes")
  <> ", parallel_group: "
  <> cue_string("verification")
  <> " }]\n"
  <> "    }\n"
  <> "  }\n\n"
  <> "  failure_modes: {\n"
  <> "    failure_modes: [{\n"
  <> "      symptom: "
  <> cue_string("Feature does not behave as expected")
  <> "\n"
  <> "      likely_cause: "
  <> cue_string("Implementation diverged from contract")
  <> "\n"
  <> "      where_to_look: [{\n"
  <> "        file: "
  <> cue_string("src/intent.gleam")
  <> "\n"
  <> "        what_to_check: "
  <> cue_string("Command execution logic")
  <> "\n"
  <> "      }]\n"
  <> "      fix_pattern: "
  <> cue_string("Align implementation with tests and contracts")
  <> "\n"
  <> "    }]\n"
  <> "  }\n\n"
  <> "  anti_hallucination: {\n"
  <> "    read_before_write: [{\n"
  <> "      file: "
  <> cue_string("src/intent.gleam")
  <> "\n"
  <> "      must_read_first: true\n"
  <> "      key_sections_to_understand: ["
  <> cue_string("Command registration and handlers")
  <> "]\n"
  <> "    }]\n"
  <> "    apis_that_exist: []\n"
  <> "    no_placeholder_values: ["
  <> cue_string("All values must be derived from real code context")
  <> "]\n"
  <> "    git_verification: {\n"
  <> "      before_claiming_done: "
  <> cue_string("git status && git diff && gleam test")
  <> "\n"
  <> "    }\n"
  <> "  }\n\n"
  <> "  context_survival: {\n"
  <> "    progress_file: {\n"
  <> "      path: "
  <> cue_string(".bead-progress/" <> id <> "/progress.txt")
  <> "\n"
  <> "      format: "
  <> cue_string("Markdown checklist")
  <> "\n"
  <> "    }\n"
  <> "    recovery_instructions: "
  <> cue_string("Read progress file and continue incomplete tasks")
  <> "\n"
  <> "  }\n\n"
  <> "  completion_checklist: {\n"
  <> "    tests: [\n"
  <> "      \"[ ] All acceptance tests written and passing\",\n"
  <> "      \"[ ] All error path tests written and passing\",\n"
  <> "      \"[ ] E2E pipeline test passing with real data\",\n"
  <> "      \"[ ] No mocks or fake data in any test\"\n"
  <> "    ]\n"
  <> "    code: [\n"
  <> "      \"[ ] Implementation uses Result<T, Error> throughout\",\n"
  <> "      \"[ ] Zero unwrap() or expect() calls\"\n"
  <> "    ]\n"
  <> "    ci: [\n"
  <> "      \"[ ] moon run :ci passes\"\n"
  <> "    ]\n"
  <> "  }\n\n"
  <> "  context: {\n"
  <> "    related_files: [{\n"
  <> "      path: "
  <> cue_string("src/intent.gleam")
  <> "\n"
  <> "      relevance: "
  <> cue_string("Primary CLI flow")
  <> "\n"
  <> "    }]\n"
  <> "  }\n\n"
  <> "  ai_hints: {\n"
  <> "    do: ["
  <> cue_string("Follow existing command and output patterns")
  <> "]\n"
  <> "    do_not: ["
  <> cue_string("Do not skip validation gates")
  <> "]\n"
  <> "    constitution: ["
  <> cue_string(
    "Coder liability is absolute: verify every bead before persistence",
  )
  <> "]\n"
  <> "  }\n"
  <> "}"
}

fn normalize_priority(priority: Int) -> Int {
  case priority < 0 {
    True -> 0
    False ->
      case priority > 4 {
        True -> 4
        False -> priority
      }
  }
}

fn profile_component(profile: String) -> String {
  case string.lowercase(profile) {
    "api" -> "API"
    "cli" -> "CLI"
    "event" -> "EVENT"
    "data" -> "DATA"
    "workflow" -> "WORKFLOW"
    "ui" -> "UI"
    _ -> "GENERAL"
  }
}

fn fallback_title(title: String) -> String {
  case string.is_empty(string.trim(title)) {
    True -> "Generated work item"
    False -> title
  }
}

fn fallback_description(description: String) -> String {
  case string.is_empty(string.trim(description)) {
    True -> "Generated from interview session"
    False -> description
  }
}

fn cue_string(value: String) -> String {
  let escaped =
    value
    |> string.replace("\\", "\\\\")
    |> string.replace("\"", "\\\"")
    |> string.replace("\n", "\\n")
    |> string.replace("\t", "\\t")

  "\"" <> escaped <> "\""
}

fn cue_string_list(items: List(String)) -> String {
  let rendered = items |> list.map(cue_string) |> string.join(", ")

  "[" <> rendered <> "]"
}

/// Extract beads with specific issue type
pub fn filter_beads_by_type(
  beads: List(BeadRecord),
  issue_type: String,
) -> List(BeadRecord) {
  list.filter(beads, fn(bead) { bead.issue_type == issue_type })
}

/// Sort beads by priority (higher number = higher priority)
pub fn sort_beads_by_priority(beads: List(BeadRecord)) -> List(BeadRecord) {
  list.sort(beads, fn(a, b) { int.compare(b.priority, a.priority) })
}

/// Add dependency between beads (updates beads in place)
pub fn add_dependency(
  beads: List(BeadRecord),
  from_title: String,
  to_title: String,
) -> List(BeadRecord) {
  list.map(beads, fn(bead) {
    case bead.title == from_title {
      True ->
        BeadRecord(
          ..bead,
          dependencies: list.append(bead.dependencies, [to_title]),
        )
      False -> bead
    }
  })
}

/// Helper: convert Profile to string
fn profile_to_string(profile: Profile) -> String {
  case profile {
    interview.Api -> "api"
    interview.Cli -> "cli"
    interview.Event -> "event"
    interview.Data -> "data"
    interview.Workflow -> "workflow"
    interview.UI -> "ui"
  }
}

/// Summary stats for beads
pub type BeadStats {
  BeadStats(total: Int, by_type: Dict(String, Int), by_priority: Dict(Int, Int))
}

/// Calculate stats for a list of beads
pub fn bead_stats(beads: List(BeadRecord)) -> BeadStats {
  let total = list.length(beads)

  let by_type =
    list.fold(beads, dict.new(), fn(acc, bead) {
      let current = dict.get(acc, bead.issue_type) |> result.unwrap(0)
      dict.insert(acc, bead.issue_type, current + 1)
    })

  let by_priority =
    list.fold(beads, dict.new(), fn(acc, bead) {
      let current = dict.get(acc, bead.priority) |> result.unwrap(0)
      dict.insert(acc, bead.priority, current + 1)
    })

  BeadStats(total: total, by_type: by_type, by_priority: by_priority)
}
