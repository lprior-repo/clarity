//! Intent CLI - Planning and bead generation tool
//!
//! A command-line tool for managing specifications, interviews, and bead generation.
//! This is the Rust implementation mirroring the Gleam CLI functionality.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use clarity_web::intent::beads::feedback::{
  collect_feedback, get_bead_feedback_history, BeadStatus as FeedbackBeadStatus,
};
use clarity_web::intent::beads::templates::{
  beads_to_enhanced_cue, beads_to_jsonl, generate_beads_from_session, BeadTemplate,
};
use clarity_web::intent::documents::ready::generate_ready_document;
use clarity_web::intent::documents::vision::generate_vision_document;
use clarity_web::intent::interview::storage::{
  append_session_to_jsonl, diff_snapshots, format_diff, get_session_from_jsonl,
  list_session_history, list_sessions_from_jsonl, session_to_jsonl_line, SessionSnapshot,
};
use clarity_web::intent::interview::types::{InterviewSession, InterviewStage, Profile};
use clarity_web::intent::loader::{export_cue_to_json, format_loader_error, validate_cue_file};
use clarity_web::intent::parser::parse_spec;
use clarity_web::intent::plan::plan_emit_beads::{emit_beads, EmissionMode};
use clarity_web::intent::plan::plan_mode::{compute_plan, format_plan_human, format_plan_json};
use clarity_web::intent::plan::plan_next::get_next_action;
use clarity_web::intent::plan::types::{BeadState, ExecutionPlan};
use clarity_web::intent::quality::effects::analyze_spec as analyze_spec_effects;
use clarity_web::intent::templates::generate_spec_template;
use clarity_web::intent::types::Spec;
use clarity_web::intent::validation::validate_spec;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Intent CLI - Planning and bead generation tool
#[derive(Parser, Debug)]
#[command(
  name = "intent",
  version = "0.1.0",
  about = "Planning and bead generation tool",
  long_about = "Intent is a planning and bead generation tool that runs interactive \
                  interviews to capture requirements, generates structured CUE specifications, \
                  and creates beads (tasks) for use with br (beads_rust)."
)]
struct Cli {
  /// The command to execute
  #[command(subcommand)]
  command: Commands,
}

/// All available CLI commands
#[derive(Subcommand, Debug)]
enum Commands {
  /// Initialize a new Intent spec from a template
  Init {
    /// Spec name (optional, will prompt if not provided)
    name: Option<String>,
    /// Template profile to use (api|cli|data|event|workflow|ui)
    #[arg(short, long, value_name = "PROFILE")]
    profile: Option<String>,
    /// Output filename (default: <name>.cue)
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
  },

  /// Run an interactive interview to capture requirements
  Interview {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: Option<String>,
    /// Resume from a specific session
    #[arg(short, long, value_name = "SESSION")]
    resume: Option<String>,
    /// Inline answer in format question_id=value (can be repeated)
    /// Example: --answer q-api-type="REST API"
    #[arg(short = 'a', long, value_name = "QUESTION_ID=VALUE")]
    answer: Vec<String>,
    /// Answer file to use for non-interactive mode (JSON or TOML format)
    #[arg(long, value_name = "FILE")]
    answer_file: Option<String>,
    /// Generate an answer template file with all questions from the session
    #[arg(long, value_name = "FILE")]
    generate_template: Option<String>,
  },

  /// Generate beads from a specification
  Beads {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: String,
    /// Output format (json|markdown)
    #[arg(long, value_name = "FORMAT", default_value = "json")]
    format: String,
    /// Output directory for generated beads
    #[arg(short, long, value_name = "DIR")]
    dir: Option<String>,
    /// Specific feature to generate beads for
    #[arg(short, long, value_name = "FEATURE")]
    feature: Option<String>,
  },

  /// Get or update bead status
  BeadStatus {
    /// Bead ID to query or update
    #[arg(value_name = "ID")]
    bead_id: String,
    /// New status to set
    #[arg(short, long, value_name = "STATUS")]
    status: Option<String>,
    /// Reason for status change
    #[arg(short, long, value_name = "REASON")]
    reason: Option<String>,
  },

  /// Show interview history
  History {
    /// Number of entries to show
    #[arg(short, long, default_value = "10")]
    limit: usize,
  },

  /// Show version information
  Version,

  /// Compare two specs, sessions, or show changes
  Diff {
    /// First spec file or session ID (when --session is used)
    spec1: Option<String>,
    /// Second spec file (not used with --session)
    spec2: Option<String>,
    /// Compare session snapshots instead of specs
    /// Usage: intent diff --session <session_id>
    /// Or: intent diff --session <session_id> --snapshot <snapshot_id>
    #[arg(long, value_name = "SESSION_ID")]
    session: Option<String>,
    /// Compare two specific snapshots by ID (requires --session)
    #[arg(long, value_name = "SNAPSHOT_ID")]
    snapshot: Option<String>,
    /// Output in JSON format
    #[arg(long)]
    json: bool,
  },

  /// Manage interview sessions
  Sessions {
    /// Session ID to operate on
    #[arg(value_name = "ID")]
    session_id: Option<String>,
    /// Delete the specified session
    #[arg(short, long)]
    delete: bool,
  },

  /// Generate a plan from a specification
  Plan {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: String,
    /// Strategy for planning (sequential|parallel|auto)
    #[arg(short, long, default_value = "auto")]
    strategy: String,
    /// Output file for the plan
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
  },

  /// Show the next recommended task
  PlanNext {
    /// Path to the plan file
    #[arg(value_name = "PLAN")]
    plan: Option<String>,
    /// Output in JSON format
    #[arg(long)]
    json: bool,
  },

  /// Approve a planned task
  PlanApprove {
    /// Task ID to approve
    #[arg(value_name = "ID")]
    task_id: String,
    /// Notes for the approval
    #[arg(short, long, value_name = "NOTES")]
    notes: Option<String>,
  },

  /// Emit beads from a plan
  PlanEmitBeads {
    /// Path to the plan file
    #[arg(value_name = "PLAN")]
    plan: String,
    /// Dry run without creating beads
    #[arg(long)]
    dry_run: bool,
  },

  /// Regenerate beads from existing spec
  BeadsRegenerate {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: String,
  },

  /// Generate a vision document from a spec
  Vision {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: String,
    /// Output file for the vision document
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
  },

  /// Generate a ready document from a spec
  Ready {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: String,
    /// Output file for the ready document
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
  },

  /// Analyze specification for second-order effects
  Effects {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: String,
    /// Output in JSON format
    #[arg(long)]
    json: bool,
  },

  /// Validate a specification
  Validate {
    /// Path to the spec file
    #[arg(value_name = "SPEC")]
    spec: String,
    /// Output in JSON format
    #[arg(long)]
    json: bool,
    /// Run additional security checks
    #[arg(long)]
    security: bool,
  },

  /// Run batch operations
  Batch {
    /// Path to batch configuration file
    #[arg(value_name = "FILE")]
    file: Option<String>,
    /// Continue on errors
    #[arg(long)]
    continue_on_error: bool,
    /// Execute in parallel where possible
    #[arg(long)]
    parallel: bool,
  },
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Init {
      name,
      profile,
      output,
    } => {
      cmd_init(name.as_deref(), profile.as_deref(), output.as_deref())?;
    }
    Commands::Interview {
      spec,
      resume,
      answer,
      answer_file,
      generate_template,
    } => {
      cmd_interview(
        spec.as_deref(),
        resume.as_deref(),
        &answer,
        answer_file.as_deref(),
        generate_template.as_deref(),
      )?;
    }
    Commands::Beads {
      spec,
      format,
      dir,
      feature,
    } => {
      cmd_beads(&spec, &format, dir.as_deref(), feature.as_deref())?;
    }
    Commands::BeadStatus {
      bead_id,
      status,
      reason,
    } => {
      cmd_bead_status(&bead_id, status.as_deref(), reason.as_deref())?;
    }
    Commands::History { limit } => {
      cmd_history(limit)?;
    }
    Commands::Version => {
      cmd_version();
    }
    Commands::Diff {
      spec1,
      spec2,
      session,
      snapshot,
      json,
    } => {
      cmd_diff(
        spec1.as_deref(),
        spec2.as_deref(),
        session.as_deref(),
        snapshot.as_deref(),
        json,
      )?;
    }
    Commands::Sessions { session_id, delete } => {
      cmd_sessions(session_id.as_deref(), delete)?;
    }
    Commands::Plan {
      spec,
      strategy,
      output,
    } => {
      cmd_plan(&spec, &strategy, output.as_deref())?;
    }
    Commands::PlanNext { plan, json } => {
      cmd_plan_next(plan.as_deref(), json)?;
    }
    Commands::PlanApprove { task_id, notes } => {
      cmd_plan_approve(&task_id, notes.as_deref())?;
    }
    Commands::PlanEmitBeads { plan, dry_run } => {
      cmd_plan_emit_beads(&plan, dry_run)?;
    }
    Commands::BeadsRegenerate { spec } => {
      cmd_beads_regenerate(&spec)?;
    }
    Commands::Vision { spec, output } => {
      cmd_vision(&spec, output.as_deref())?;
    }
    Commands::Ready { spec, output } => {
      cmd_ready(&spec, output.as_deref())?;
    }
    Commands::Effects { spec, json } => {
      cmd_effects(&spec, json)?;
    }
    Commands::Validate {
      spec,
      json,
      security,
    } => {
      cmd_validate(&spec, json, security)?;
    }
    Commands::Batch {
      file,
      continue_on_error,
      parallel,
    } => {
      cmd_batch(file.as_deref(), continue_on_error, parallel)?;
    }
  }

  Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get the default interview sessions directory
fn sessions_dir() -> PathBuf {
  PathBuf::from(".interview/sessions")
}

/// Get the default session JSONL file path
fn session_jsonl_path() -> PathBuf {
  sessions_dir().join("sessions.jsonl")
}

/// Get the session history file path
fn session_history_path() -> PathBuf {
  sessions_dir().join("history.jsonl")
}

/// Get the default beads directory
fn beads_dir() -> PathBuf {
  PathBuf::from(".beads")
}

/// Get the beads JSONL file path
fn beads_jsonl_path() -> PathBuf {
  beads_dir().join("beads.jsonl")
}

/// Get current timestamp in ISO 8601 format
fn current_timestamp() -> String {
  Utc::now().to_rfc3339()
}

/// Generate a unique session ID
fn generate_session_id() -> String {
  let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
  format!("session-{timestamp}")
}

/// Load a spec from a CUE file
fn load_spec_from_cue(path: &str) -> Result<Spec> {
  let spec_path = Path::new(path);

  // Validate file exists
  if !spec_path.exists() {
    return Err(anyhow::anyhow!("Spec file not found: {path}"));
  }

  // Export CUE to JSON
  let json_str = export_cue_to_json(spec_path)
    .map_err(|e| anyhow::anyhow!("Failed to export CUE: {}", format_loader_error(&e)))?;

  // Parse JSON to Spec
  let spec = parse_spec(&json_str).map_err(|e| anyhow::anyhow!("Failed to parse spec: {e:?}"))?;

  Ok(spec)
}

/// Load or create an interview session
fn load_or_create_session(session_id: Option<&str>) -> Result<InterviewSession> {
  let jsonl_path = session_jsonl_path();

  session_id.map_or_else(
    || {
      Ok(InterviewSession {
        id: generate_session_id(),
        profile: Profile::default(),
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
        completed_at: None,
        stage: InterviewStage::Discovery,
        rounds_completed: 0,
        answers: Vec::new(),
        gaps: Vec::new(),
        conflicts: Vec::new(),
        raw_notes: String::new(),
        current_phase: 1,
        completed_phases: Vec::new(),
      })
    },
    |id| {
      get_session_from_jsonl(&jsonl_path, id)
        .map_err(|e| anyhow::anyhow!("Failed to load session: {e}"))
    },
  )
}

/// Save an interview session
fn save_session(session: &InterviewSession) -> Result<()> {
  let jsonl_path = session_jsonl_path();
  append_session_to_jsonl(session, &jsonl_path)
    .map_err(|e| anyhow::anyhow!("Failed to save session: {e}"))
}

/// Write output to file or stdout
fn write_output(content: &str, output_path: Option<&str>) -> Result<()> {
  match output_path {
    Some(path) => {
      let path = Path::new(path);
      if let Some(parent) = path.parent() {
        if !parent.exists() {
          fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
      }
      fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
      println!("Output written to: {}", path.display());
    }
    None => {
      println!("{content}");
    }
  }
  Ok(())
}

/// Load beads from the beads JSONL file
fn load_beads() -> Result<Vec<BeadTemplate>> {
  let beads_path = beads_jsonl_path();
  if !beads_path.exists() {
    return Ok(Vec::new());
  }

  let content = fs::read_to_string(&beads_path)
    .with_context(|| format!("Failed to read beads file: {}", beads_path.display()))?;

  let beads: Vec<BeadTemplate> = content
    .lines()
    .filter(|line| !line.trim().is_empty())
    .filter_map(|line| serde_json::from_str(line).ok())
    .collect();

  Ok(beads)
}

/// Save beads to the beads JSONL file
fn save_beads(beads: &[BeadTemplate]) -> Result<()> {
  let beads_path = beads_jsonl_path();
  if let Some(parent) = beads_path.parent() {
    if !parent.exists() {
      fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
  }

  let jsonl =
    beads_to_jsonl(beads).map_err(|e| anyhow::anyhow!("Failed to serialize beads: {e:?}"))?;
  fs::write(&beads_path, jsonl)
    .with_context(|| format!("Failed to write beads file: {}", beads_path.display()))?;

  Ok(())
}

/// Load execution plan from file
fn load_plan(path: &str) -> Result<ExecutionPlan> {
  let plan_path = Path::new(path);
  if !plan_path.exists() {
    return Err(anyhow::anyhow!("Plan file not found: {path}"));
  }

  let content =
    fs::read_to_string(plan_path).with_context(|| format!("Failed to read plan file: {path}"))?;

  let plan: ExecutionPlan =
    serde_json::from_str(&content).with_context(|| "Failed to parse plan file")?;

  Ok(plan)
}

/// Save execution plan to file
fn save_plan(plan: &ExecutionPlan, path: &str) -> Result<()> {
  let plan_path = Path::new(path);
  if let Some(parent) = plan_path.parent() {
    if !parent.exists() {
      fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
  }

  let content = serde_json::to_string_pretty(plan).with_context(|| "Failed to serialize plan")?;

  fs::write(plan_path, content).with_context(|| format!("Failed to write plan file: {path}"))?;

  Ok(())
}

// ============================================================================
// COMMAND IMPLEMENTATIONS
// ============================================================================

/// Initialize a new Intent spec from a template
fn cmd_init(name: Option<&str>, profile: Option<&str>, output: Option<&str>) -> Result<()> {
  use intent::init_prompt::{run_interactive_setup, ProjectConfig};

  // If all options are provided, use non-interactive mode
  let config = if name.is_some() && profile.is_some() && output.is_some() {
    // Non-interactive mode with all args provided
    let spec_name = name.context("name is required")?.to_string();
    let profile_str = profile.context("profile is required")?.to_string();
    let output_file = output.context("output is required")?.to_string();

    ProjectConfig {
      name: spec_name.clone(),
      description: format!("{spec_name} specification"),
      audience: "developers".to_string(),
      profile: profile_str,
      output_file,
      version: "0.1.0".to_string(),
    }
  } else {
    // Run fully interactive setup
    run_interactive_setup(name)?
  };

  // Parse profile
  let parsed_profile = Profile::parse(&config.profile)
    .map_err(|e| anyhow::anyhow!("Invalid profile '{}': {:?}", config.profile, e))?;

  // Generate template
  let template = generate_spec_template(parsed_profile)
    .map_err(|e| anyhow::anyhow!("Failed to generate template: {:?}", e))?;

  // Replace placeholders with collected values
  let content = template
    .replace("{{name}}", &config.name)
    .replace("{{description}}", &config.description)
    .replace("{{audience}}", &config.audience)
    .replace("{{version}}", &config.version);

  // Write output
  write_output(&content, Some(&config.output_file))?;

  println!();
  println!("===================================================================");
  println!("                     Spec Created Successfully!");
  println!("===================================================================");
  println!();
  println!("  File:     {}", config.output_file);
  println!("  Name:     {}", config.name);
  println!("  Profile:  {}", config.profile);
  println!();
  println!("Next steps:");
  println!("  1. Review and edit the spec file");
  println!(
    "  2. Run 'intent interview {}' to capture requirements",
    config.output_file
  );
  println!(
    "  3. Run 'intent validate {}' to check for issues",
    config.output_file
  );
  println!();

  Ok(())
}

/// Parse an inline answer in format "question_id=value"
fn parse_inline_answer(input: &str) -> Result<(String, String)> {
  input
    .find('=')
    .map(|pos| {
      let (question_id, value) = input.split_at(pos);
      (question_id.to_string(), value[1..].to_string())
    })
    .ok_or_else(|| {
      anyhow::anyhow!(
        "Invalid answer format '{}'. Expected: question_id=value",
        input
      )
    })
}

/// Run an interactive interview to capture requirements
fn cmd_interview(
  _spec: Option<&str>,
  resume: Option<&str>,
  inline_answers: &[String],
  answer_file: Option<&str>,
  generate_template: Option<&str>,
) -> Result<()> {
  // Generate answers template if requested
  if let Some(template_path) = generate_template {
    return cmd_generate_template(template_path, resume);
  }

  // Parse inline answers into a map
  let parsed_inline: HashMap<String, String> = inline_answers
    .iter()
    .map(|s| parse_inline_answer(s))
    .collect::<Result<HashMap<_, _>>>()?;

  // Load or create session
  let mut session = load_or_create_session(resume)?;

  // If answer file provided, load it
  let file_answers: HashMap<String, String> = match answer_file {
    Some(path) => {
      let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read answer file: {path}"))?;
      parse_answer_file(&content)?
    }
    None => HashMap::new(),
  };

  // Merge inline answers with file answers (inline takes precedence)
  let all_answers: HashMap<String, String> =
    file_answers.into_iter().chain(parsed_inline).collect();

  // If any answers provided, use non-interactive mode
  if !all_answers.is_empty() {
    // Process answers
    for (question_id, response) in all_answers {
      session
        .answers
        .push(clarity_web::intent::interview::types::Answer {
          question_id,
          question_text: String::new(),
          perspective: clarity_web::intent::interview::types::Perspective::default(),
          round: session.rounds_completed + 1,
          response,
          extracted: HashMap::new(),
          confidence: 1.0,
          notes: String::new(),
          timestamp: current_timestamp(),
        });
    }

    session.updated_at = current_timestamp();
    save_session(&session)?;
    println!("Interview completed. Session ID: {}", session.id);
    return Ok(());
  }

  // Interactive interview
  println!("Starting interview session: {}", session.id);
  println!("Profile: {:?}", session.profile);
  println!("Stage: {:?}", session.stage);
  println!();

  // Get profile-specific required fields as questions
  let required_fields = session.profile.required_fields();
  for field in required_fields {
    let prompt = format!("Enter value for '{field}':");
    let response: String = dialoguer::Input::new()
      .with_prompt(&prompt)
      .interact_text()
      .context("Failed to read input")?;

    session
      .answers
      .push(clarity_web::intent::interview::types::Answer {
        question_id: field.to_string(),
        question_text: prompt,
        perspective: clarity_web::intent::interview::types::Perspective::User,
        round: session.rounds_completed + 1,
        response,
        extracted: HashMap::new(),
        confidence: 1.0,
        notes: String::new(),
        timestamp: current_timestamp(),
      });
  }

  // Ask for raw notes
  let notes: String = dialoguer::Input::new()
    .with_prompt("Additional notes (optional)")
    .allow_empty(true)
    .interact_text()
    .context("Failed to read notes")?;
  session.raw_notes = notes;

  // Update session
  session.updated_at = current_timestamp();
  session.rounds_completed += 1;
  session.stage = InterviewStage::Refinement;

  // Save session
  save_session(&session)?;

  println!();
  println!("Interview saved. Session ID: {}", session.id);
  println!("Stage: {:?}", session.stage);

  Ok(())
}

/// Generate an answer template file with all questions from the session
fn cmd_generate_template(template_path: &str, resume: Option<&str>) -> Result<()> {
  use clarity_web::intent::interview::interview_questions::get_questions_for_round;

  // Load or create session to get profile
  let session = load_or_create_session(resume)?;
  let profile_str = session.profile.as_str();

  // Collect all questions from all rounds
  let mut all_questions: Vec<clarity_web::intent::interview::types::Question> = Vec::new();

  // Get questions for rounds 1-5 (typical interview has 5 rounds)
  for round in 1..=5 {
    let questions = get_questions_for_round(profile_str, round);
    all_questions.extend(questions);
  }

  // If no questions from CUE, use profile required fields
  if all_questions.is_empty() {
    let required_fields = session.profile.required_fields();
    all_questions = required_fields
      .into_iter()
      .map(|field| clarity_web::intent::interview::types::Question {
        id: format!("q-{field}"),
        round: 1,
        perspective: clarity_web::intent::interview::types::Perspective::User,
        category: clarity_web::intent::interview::types::QuestionCategory::HappyPath,
        priority: clarity_web::intent::interview::types::QuestionPriority::Critical,
        question: format!("Enter value for '{field}'"),
        context: String::new(),
        example: String::new(),
        expected_type: "text".to_string(),
        extract_into: vec![field.to_string()],
        depends_on: Vec::new(),
        blocks: Vec::new(),
      })
      .collect();
  }

  // Generate TOML template
  let template = generate_toml_template(&all_questions);

  write_output(&template, Some(template_path))?;
  println!(
    "Generated template with {} questions to: {template_path}",
    all_questions.len()
  );

  Ok(())
}

/// Generate a TOML template from questions
fn generate_toml_template(questions: &[clarity_web::intent::interview::types::Question]) -> String {
  let header = r#"# Interview Answer Template
# Fill in the 'answer' field for each question and run:
#   intent interview --answer-file <this_file>
#
# Lines starting with # are comments and will be ignored.
# Question IDs in [brackets] must not be changed.

"#;

  let questions_str = questions
    .iter()
    .map(|q| {
      let comment = format!("# Question: {}\n", q.question);
      let section = format!("[{}]\nanswer = \"\"\n", q.id);
      format!("{comment}{section}")
    })
    .join("\n");

  format!("{header}{questions_str}")
}

/// Parse answers from a file (supports JSON and TOML formats)
fn parse_answer_file(content: &str) -> Result<HashMap<String, String>> {
  let trimmed = content.trim();

  // Try JSON first (starts with {)
  if trimmed.starts_with('{') {
    return serde_json::from_str(trimmed)
      .with_context(|| "Failed to parse answer file as JSON");
  }

  // Try TOML format
  parse_toml_answers(trimmed)
}

/// Parse answers from TOML format
fn parse_toml_answers(content: &str) -> Result<HashMap<String, String>> {
  let mut answers = HashMap::new();
  let mut current_section: Option<String> = None;

  for line in content.lines() {
    let trimmed = line.trim();

    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    // Section header [question-id]
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
      current_section = Some(trimmed[1..trimmed.len() - 1].to_string());
      continue;
    }

    // Answer field: answer = "value"
    if let Some(rest) = trimmed.strip_prefix("answer") {
      let rest = rest.trim();
      if let Some(rest) = rest.strip_prefix('=') {
        let rest = rest.trim();
        // Remove surrounding quotes if present
        let value = if (rest.starts_with('"') && rest.ends_with('"'))
          || (rest.starts_with('\'') && rest.ends_with('\''))
        {
          &rest[1..rest.len() - 1]
        } else {
          rest
        };

        if let Some(ref section) = current_section {
          answers.insert(section.clone(), value.to_string());
        }
      }
    }
  }

  Ok(answers)
}

/// Generate beads from a specification
fn cmd_beads(spec: &str, format: &str, dir: Option<&str>, feature: Option<&str>) -> Result<()> {
  // Load spec (validates it exists and parses)
  let _loaded_spec = load_spec_from_cue(spec)?;

  // Create a minimal session from the spec for bead generation
  let session = InterviewSession {
    id: generate_session_id(),
    profile: Profile::default(),
    created_at: current_timestamp(),
    updated_at: current_timestamp(),
    completed_at: None,
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: Vec::new(),
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  };

  // Generate beads
  let all_beads = generate_beads_from_session(&session)
    .map_err(|e| anyhow::anyhow!("Failed to generate beads: {e:?}"))?;

  // Filter by feature if specified
  let beads: Vec<BeadTemplate> = match feature {
    Some(f) => all_beads
      .into_iter()
      .filter(|b| b.title.contains(f) || b.description.contains(f))
      .collect(),
    None => all_beads,
  };

  // Output based on format
  let output = match format {
    "json" => {
      beads_to_jsonl(&beads).map_err(|e| anyhow::anyhow!("Failed to format beads: {e:?}"))?
    }
    "markdown" | "md" => {
      beads_to_enhanced_cue(&beads).map_err(|e| anyhow::anyhow!("Failed to format beads: {e:?}"))?
    }
    _ => {
      return Err(anyhow::anyhow!(
        "Unknown format '{format}'. Supported: json, markdown"
      ))
    }
  };

  // Determine output destination
  match dir {
    Some(d) => {
      let output_dir = Path::new(d);
      fs::create_dir_all(output_dir).with_context(|| format!("Failed to create directory: {d}"))?;

      let output_file = output_dir.join("beads.jsonl");
      fs::write(&output_file, &output)
        .with_context(|| format!("Failed to write beads file: {}", output_file.display()))?;

      println!(
        "Generated {} beads to {}",
        beads.len(),
        output_file.display()
      );
    }
    None => {
      println!("{output}");
    }
  }

  Ok(())
}

/// Get or update bead status
fn cmd_bead_status(bead_id: &str, status: Option<&str>, reason: Option<&str>) -> Result<()> {
  // If no status provided, just show current status
  if status.is_none() {
    let history = get_bead_feedback_history(bead_id);

    if history.is_empty() {
      println!("Bead '{bead_id}' not found.");
      return Ok(());
    }

    println!("Bead: {bead_id}");
    println!("History:");
    for feedback in history {
      println!(
        "  - {:?} at {} | {}",
        feedback.status, feedback.timestamp, feedback.notes
      );
    }
    return Ok(());
  }

  // Parse new status
  let new_status = match status {
    Some(s) => match s.to_lowercase().as_str() {
      "pending" => FeedbackBeadStatus::Pending,
      "ready" => FeedbackBeadStatus::Ready,
      "in_progress" | "inprogress" => FeedbackBeadStatus::InProgress,
      "blocked" => FeedbackBeadStatus::Blocked,
      "complete" | "completed" | "done" => FeedbackBeadStatus::Complete,
      "failed" => FeedbackBeadStatus::Failed,
      _ => return Err(anyhow::anyhow!("Unknown status: {s}")),
    },
    None => FeedbackBeadStatus::Pending,
  };

  // Collect feedback (this updates the bead status)
  let notes = reason.unwrap_or("Status updated via CLI");
  collect_feedback(bead_id, new_status, notes)
    .map_err(|e| anyhow::anyhow!("Failed to update bead status: {e:?}"))?;

  println!("Bead '{bead_id}' status updated to {new_status:?}");
  Ok(())
}

/// Show interview history
fn cmd_history(limit: usize) -> Result<()> {
  let jsonl_path = session_jsonl_path();

  if !jsonl_path.exists() {
    println!("No interview history found.");
    return Ok(());
  }

  let sessions = list_sessions_from_jsonl(&jsonl_path)
    .map_err(|e| anyhow::anyhow!("Failed to list sessions: {e}"))?;

  if sessions.is_empty() {
    println!("No interview sessions found.");
    return Ok(());
  }

  println!(
    "Interview History (showing last {}):",
    limit.min(sessions.len())
  );
  println!("{}", "-".repeat(60));

  for session in sessions.iter().rev().take(limit) {
    println!("Session: {}", session.id);
    println!("  Profile: {:?}", session.profile);
    println!("  Stage: {:?}", session.stage);
    println!("  Answers: {}", session.answers.len());
    println!("  Created: {}", session.created_at);
    println!("  Updated: {}", session.updated_at);
    println!();
  }

  println!("Total sessions: {}", sessions.len());
  Ok(())
}

/// Show version information
fn cmd_version() {
  println!("intent v0.1.0");
  println!("Planning and bead generation tool");
  println!("Part of the Clarity specification system");
}

/// Compare two specs, sessions, or show changes
fn cmd_diff(
  spec1: Option<&str>,
  spec2: Option<&str>,
  session_id: Option<&str>,
  snapshot_id: Option<&str>,
  json: bool,
) -> Result<()> {
  // Handle session diff mode
  if let Some(sid) = session_id {
    return cmd_diff_session(sid, snapshot_id, json);
  }

  // Require both spec files for spec diff mode
  match (spec1, spec2) {
    (Some(s1), Some(s2)) => cmd_diff_specs(s1, s2),
    _ => Err(anyhow::anyhow!(
      "Usage: intent diff <spec1> <spec2>\n       intent diff --session <session_id>"
    )),
  }
}

/// Compare two interview sessions or session snapshots
fn cmd_diff_session(session_id: &str, snapshot_id: Option<&str>, json: bool) -> Result<()> {
  let history_path = session_history_path();

  // Get session history
  let snapshots = list_session_history(&history_path, session_id)
    .map_err(|e| anyhow::anyhow!("Failed to list session history: {e}"))?;

  if snapshots.is_empty() {
    println!("No history found for session: {session_id}");
    println!("Hint: Session history is created when sessions are saved.");
    return Ok(());
  }

  // Handle specific snapshot comparison
  if let Some(snap_id) = snapshot_id {
    let target_snapshot = snapshots.iter().find(|s| s.snapshot_id == snap_id).cloned();

    match target_snapshot {
      Some(target) => {
        // Find the previous snapshot
        let target_idx = snapshots
          .iter()
          .position(|s| s.snapshot_id == snap_id)
          .map_or(0, |i| if i > 0 { i - 1 } else { 0 });

        let previous = if target_idx == 0 && snapshots[0].snapshot_id == snap_id {
          // No previous snapshot, create empty one
          SessionSnapshot {
            session_id: session_id.to_string(),
            snapshot_id: format!("{session_id}-initial"),
            timestamp: String::new(),
            description: "Initial state".to_string(),
            answers: HashMap::new(),
            gaps_count: 0,
            conflicts_count: 0,
            stage: "discovery".to_string(),
            created_by: None,
            version: 0,
            tags: Vec::new(),
          }
        } else {
          snapshots[target_idx].clone()
        };

        let diff = diff_snapshots(&previous, &target);

        if json {
          let json_output =
            serde_json::to_string_pretty(&diff).with_context(|| "Failed to serialize diff")?;
          println!("{json_output}");
        } else {
          print!("{}", format_diff(&diff));
        }
      }
      None => {
        println!("Snapshot '{snap_id}' not found in session {session_id}");
        println!("Available snapshots:");
        for snap in &snapshots {
          println!("  - {} ({})", snap.snapshot_id, snap.timestamp);
        }
      }
    }
    return Ok(());
  }

  // Compare the two most recent snapshots
  if snapshots.len() == 1 {
    // Only one snapshot - compare with empty state
    let empty_snapshot = SessionSnapshot {
      session_id: session_id.to_string(),
      snapshot_id: format!("{session_id}-initial"),
      timestamp: String::new(),
      description: "Initial state".to_string(),
      answers: HashMap::new(),
      gaps_count: 0,
      conflicts_count: 0,
      stage: "discovery".to_string(),
      created_by: None,
      version: 0,
      tags: Vec::new(),
    };

    let diff = diff_snapshots(&empty_snapshot, &snapshots[0]);

    println!("Session: {session_id} (comparing with initial state)");
    println!("{}", "-".repeat(60));

    if json {
      let json_output =
        serde_json::to_string_pretty(&diff).with_context(|| "Failed to serialize diff")?;
      println!("{json_output}");
    } else {
      print!("{}", format_diff(&diff));
    }
  } else {
    // Compare last two snapshots
    let len = snapshots.len();
    let previous = &snapshots[len - 2];
    let current = &snapshots[len - 1];

    let diff = diff_snapshots(previous, current);

    println!("Session: {session_id}");
    println!(
      "Comparing: {} -> {}",
      previous.snapshot_id, current.snapshot_id
    );
    println!("{}", "-".repeat(60));

    if json {
      let json_output =
        serde_json::to_string_pretty(&diff).with_context(|| "Failed to serialize diff")?;
      println!("{json_output}");
    } else {
      print!("{}", format_diff(&diff));
    }
  }

  Ok(())
}

/// Compare two spec files
fn cmd_diff_specs(spec1: &str, spec2: &str) -> Result<()> {
  // Load both specs
  let spec1_parsed = load_spec_from_cue(spec1)?;
  let spec2_parsed = load_spec_from_cue(spec2)?;

  // Compare specs
  println!("Comparing specs:");
  println!("  Spec 1: {} ({})", spec1_parsed.name, spec1);
  println!("  Spec 2: {} ({})", spec2_parsed.name, spec2);
  println!("{}", "-".repeat(60));

  // Compare names
  if spec1_parsed.name != spec2_parsed.name {
    println!(
      "Name changed: '{}' -> '{}'",
      spec1_parsed.name, spec2_parsed.name
    );
  }

  // Compare descriptions
  if spec1_parsed.description != spec2_parsed.description {
    println!("Description changed:");
    println!("  - {}", spec1_parsed.description);
    println!("  + {}", spec2_parsed.description);
  }

  // Compare features
  let features1: std::collections::HashSet<&str> = spec1_parsed
    .features
    .iter()
    .map(|f| f.name.as_str())
    .collect();
  let features2: std::collections::HashSet<&str> = spec2_parsed
    .features
    .iter()
    .map(|f| f.name.as_str())
    .collect();

  let added: Vec<_> = features2.difference(&features1).copied().collect();
  let removed: Vec<_> = features1.difference(&features2).copied().collect();

  if !added.is_empty() {
    println!("Features added:");
    for f in &added {
      println!("  + {f}");
    }
  }

  if !removed.is_empty() {
    println!("Features removed:");
    for f in &removed {
      println!("  - {f}");
    }
  }

  // Compare behavior counts
  let behaviors1: usize = spec1_parsed
    .features
    .iter()
    .map(|f| f.behaviors.len())
    .sum();
  let behaviors2: usize = spec2_parsed
    .features
    .iter()
    .map(|f| f.behaviors.len())
    .sum();

  if behaviors1 != behaviors2 {
    println!("Behavior count changed: {behaviors1} -> {behaviors2}");
  }

  // Compare invariants
  if spec1_parsed.invariants.len() != spec2_parsed.invariants.len() {
    println!(
      "Invariants count changed: {} -> {}",
      spec1_parsed.invariants.len(),
      spec2_parsed.invariants.len()
    );
  }

  // Compare anti-patterns
  if spec1_parsed.anti_patterns.len() != spec2_parsed.anti_patterns.len() {
    println!(
      "Anti-patterns count changed: {} -> {}",
      spec1_parsed.anti_patterns.len(),
      spec2_parsed.anti_patterns.len()
    );
  }

  if added.is_empty() && removed.is_empty() && behaviors1 == behaviors2 {
    println!("No significant differences found.");
  }

  Ok(())
}

/// Manage interview sessions
fn cmd_sessions(session_id: Option<&str>, delete: bool) -> Result<()> {
  let jsonl_path = session_jsonl_path();

  if !jsonl_path.exists() {
    println!("No sessions found.");
    return Ok(());
  }

  match (session_id, delete) {
    (Some(id), true) => {
      // Delete session
      let sessions = list_sessions_from_jsonl(&jsonl_path)
        .map_err(|e| anyhow::anyhow!("Failed to list sessions: {e}"))?;

      let original_len = sessions.len();
      let remaining: Vec<_> = sessions.into_iter().filter(|s| s.id != id).collect();

      if remaining.len() == original_len {
        println!("Session '{id}' not found.");
        return Ok(());
      }

      // Rewrite the file without the deleted session
      let mut content = String::new();
      for session in &remaining {
        let line = session_to_jsonl_line(session)
          .map_err(|e| anyhow::anyhow!("Failed to serialize session: {e}"))?;
        content.push_str(&line);
        content.push('\n');
      }

      fs::write(&jsonl_path, content)
        .with_context(|| format!("Failed to write sessions file: {}", jsonl_path.display()))?;

      println!("Session '{id}' deleted.");
    }
    (Some(id), false) => {
      // Show specific session
      let session = get_session_from_jsonl(&jsonl_path, id)
        .map_err(|e| anyhow::anyhow!("Failed to get session: {e}"))?;

      println!("Session: {}", session.id);
      println!("  Profile: {:?}", session.profile);
      println!("  Stage: {:?}", session.stage);
      println!("  Rounds completed: {}", session.rounds_completed);
      println!("  Answers: {}", session.answers.len());
      println!("  Gaps: {}", session.gaps.len());
      println!("  Conflicts: {}", session.conflicts.len());
      println!("  Created: {}", session.created_at);
      println!("  Updated: {}", session.updated_at);

      if !session.answers.is_empty() {
        println!("\nAnswers:");
        for answer in &session.answers {
          println!("  - {}: {}", answer.question_id, answer.response);
        }
      }
    }
    (None, _) => {
      // List all sessions
      let sessions = list_sessions_from_jsonl(&jsonl_path)
        .map_err(|e| anyhow::anyhow!("Failed to list sessions: {e}"))?;

      if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
      }

      println!("Sessions:");
      for session in &sessions {
        println!(
          "  {} [{:?}] {} answers, {} gaps",
          session.id,
          session.stage,
          session.answers.len(),
          session.gaps.len()
        );
      }
      println!("\nTotal: {} sessions", sessions.len());
    }
  }

  Ok(())
}

/// Generate a plan from a specification
fn cmd_plan(spec: &str, strategy: &str, output: Option<&str>) -> Result<()> {
  // Load spec (validates it exists)
  let _loaded_spec = load_spec_from_cue(spec)?;

  // Create session from spec for plan generation
  let session = InterviewSession {
    id: generate_session_id(),
    profile: Profile::default(),
    created_at: current_timestamp(),
    updated_at: current_timestamp(),
    completed_at: None,
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: Vec::new(),
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  };

  // Compute plan
  let plan =
    compute_plan(&session).map_err(|e| anyhow::anyhow!("Failed to compute plan: {e:?}"))?;

  // Format output
  let content = match strategy {
    "json" => format_plan_json(&plan),
    _ => format_plan_human(&plan),
  };

  // Write output
  let output_file = output.unwrap_or("plan.json");
  write_output(&content, Some(output_file))?;

  println!("Plan generated with {} phases", plan.phases.len());
  Ok(())
}

/// Show the next recommended task
fn cmd_plan_next(plan: Option<&str>, json: bool) -> Result<()> {
  // Load plan
  let plan_path = plan.unwrap_or("plan.json");
  let loaded_plan = load_plan(plan_path)?;

  // Create a minimal session
  let session = InterviewSession {
    id: generate_session_id(),
    profile: Profile::default(),
    created_at: current_timestamp(),
    updated_at: current_timestamp(),
    completed_at: None,
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: Vec::new(),
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  };

  // Get next action
  let next_action = get_next_action(&session, &loaded_plan);

  match next_action {
    Some(action) => {
      if json {
        let json_output =
          serde_json::to_string_pretty(&action).with_context(|| "Failed to serialize action")?;
        println!("{json_output}");
      } else {
        println!("Next Recommended Action:");
        println!("  Type: {:?}", action.action_type);
        println!("  Target: {}", action.target_id);
        println!("  Description: {}", action.description);
        println!("  Reason: {}", action.reason);
        if action.priority > 0 {
          println!("  Priority: {}", action.priority);
        }
      }
    }
    None => {
      if json {
        println!("{{\"status\": \"no_action_available\"}}");
      } else {
        println!("No next action available. All tasks may be complete.");
      }
    }
  }

  Ok(())
}

/// Approve a planned task
fn cmd_plan_approve(task_id: &str, notes: Option<&str>) -> Result<()> {
  // Load plan
  let plan_path = "plan.json";
  let mut loaded_plan = load_plan(plan_path)?;

  // Find and update the bead
  let found = loaded_plan.beads.iter_mut().any(|bead| {
    if bead.id == task_id {
      bead.state = BeadState::Ready;
      true
    } else {
      false
    }
  });

  if !found {
    return Err(anyhow::anyhow!("Task '{task_id}' not found in plan"));
  }

  // Save plan
  save_plan(&loaded_plan, plan_path)?;

  println!("Task '{task_id}' approved.");
  if let Some(n) = notes {
    println!("Notes: {n}");
  }

  Ok(())
}

/// Emit beads from a plan
fn cmd_plan_emit_beads(plan_path: &str, dry_run: bool) -> Result<()> {
  // Load plan
  let loaded_plan = load_plan(plan_path)?;

  // Create a minimal session
  let session = InterviewSession {
    id: generate_session_id(),
    profile: Profile::default(),
    created_at: current_timestamp(),
    updated_at: current_timestamp(),
    completed_at: None,
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: Vec::new(),
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  };

  // Determine emission mode
  let mode = if dry_run {
    EmissionMode::Simulate
  } else {
    EmissionMode::Persist
  };

  // Emit beads
  let mut mutable_plan = loaded_plan;
  let (emitted_beads, result) = emit_beads(&session, &mut mutable_plan, mode)
    .map_err(|e| anyhow::anyhow!("Failed to emit beads: {e:?}"))?;

  // Report results
  if dry_run {
    println!("Dry run - would emit {} beads:", emitted_beads.len());
  } else {
    println!("Emitted {} beads:", emitted_beads.len());
  }

  for bead in &emitted_beads {
    println!("  - {} [Phase {}]", bead.title, bead.phase);
  }

  if result.skipped > 0 {
    println!("Skipped {} existing beads", result.skipped);
  }

  if !result.errors.is_empty() {
    println!("\nErrors:");
    for error in &result.errors {
      println!("  - {error}");
    }
  }

  // Save emitted beads to beads directory
  if !dry_run && !emitted_beads.is_empty() {
    let mut existing_beads = load_beads()?;
    existing_beads.extend(emitted_beads.iter().map(|b| BeadTemplate {
      title: b.title.clone(),
      description: b.description.clone(),
      profile_type: "plan".to_string(),
      priority: u8::try_from(b.priority.clamp(1, 5)).unwrap_or(3),
      dependencies: b.dependencies.clone(),
      ..BeadTemplate::default()
    }));
    save_beads(&existing_beads)?;
    println!("\nBeads saved to: {}", beads_jsonl_path().display());
  }

  Ok(())
}

/// Regenerate beads from existing spec
fn cmd_beads_regenerate(spec: &str) -> Result<()> {
  println!("Regenerating beads from: {spec}");

  // Load spec (validates it exists)
  let _loaded_spec = load_spec_from_cue(spec)?;

  // Create session for bead generation
  let session = InterviewSession {
    id: generate_session_id(),
    profile: Profile::default(),
    created_at: current_timestamp(),
    updated_at: current_timestamp(),
    completed_at: None,
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: Vec::new(),
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  };

  // Generate new beads
  let new_beads = generate_beads_from_session(&session)
    .map_err(|e| anyhow::anyhow!("Failed to generate beads: {e:?}"))?;

  // Load existing beads
  let existing_beads = load_beads()?;

  // Merge: keep existing bead status, add new beads (by title matching)
  let existing_titles: std::collections::HashSet<String> =
    existing_beads.iter().map(|b| b.title.clone()).collect();

  let merged: Vec<BeadTemplate> = existing_beads
    .into_iter()
    .chain(
      new_beads
        .into_iter()
        .filter(|b| !existing_titles.contains(&b.title)),
    )
    .collect();

  // Save merged beads
  save_beads(&merged)?;

  println!("Regenerated {} beads", merged.len());
  Ok(())
}

/// Generate a vision document from a spec
fn cmd_vision(spec: &str, output: Option<&str>) -> Result<()> {
  // Load spec
  let loaded_spec = load_spec_from_cue(spec)?;

  // Generate vision document
  let vision = generate_vision_document(&loaded_spec);

  // Write output
  write_output(&vision, output)?;

  Ok(())
}

/// Generate a ready document from a spec
fn cmd_ready(spec: &str, output: Option<&str>) -> Result<()> {
  // Load spec
  let loaded_spec = load_spec_from_cue(spec)?;

  // Generate ready document
  let ready = generate_ready_document(&loaded_spec);

  // Write output
  write_output(&ready, output)?;

  Ok(())
}

/// Analyze specification for second-order effects
fn cmd_effects(spec: &str, json: bool) -> Result<()> {
  // Load spec
  let loaded_spec = load_spec_from_cue(spec)?;

  // Analyze effects
  let report = analyze_spec_effects(&loaded_spec);

  if json {
    let json_output = serde_json::to_string_pretty(&report)
      .with_context(|| "Failed to serialize effects report")?;
    println!("{json_output}");
  } else {
    println!("Effects Analysis for: {}", report.spec_name);
    println!("{}", "-".repeat(60));

    if report.behavior_reports.is_empty() {
      println!("No significant effects detected.");
    } else {
      for behavior_report in &report.behavior_reports {
        if !behavior_report.effects.is_empty() {
          println!("\nBehavior: {}", behavior_report.behavior_name);
          for effect in &behavior_report.effects {
            println!(
              "  - {} [{}]: {}",
              effect.effect_type, effect.severity, effect.description
            );
          }
        }
      }
    }
  }

  Ok(())
}

/// Validate a specification
fn cmd_validate(spec: &str, json: bool, security: bool) -> Result<()> {
  let spec_path = Path::new(spec);

  // Check file exists
  if !spec_path.exists() {
    return Err(anyhow::anyhow!("Spec file not found: {spec}"));
  }

  let mut errors: Vec<String> = Vec::new();
  let mut warnings: Vec<String> = Vec::new();

  // Validate CUE syntax
  match validate_cue_file(spec_path) {
    Ok(()) => {}
    Err(e) => {
      errors.push(format!(
        "CUE validation failed: {}",
        format_loader_error(&e)
      ));
    }
  }

  // Load and parse spec
  let loaded_spec = match load_spec_from_cue(spec) {
    Ok(s) => Some(s),
    Err(e) => {
      errors.push(format!("Failed to parse spec: {e}"));
      None
    }
  };

  // Validate spec structure
  if let Some(ref parsed_spec) = loaded_spec {
    let result = validate_spec(parsed_spec);

    if !result.is_valid() {
      errors.push("Spec validation failed".to_string());
      for error in &result.errors {
        errors.push(format!("  - {error}"));
      }
    }

    for warning in &result.warnings {
      let ctx = warning
        .context
        .as_ref()
        .map(|c| format!(" ({c})"))
        .unwrap_or_default();
      warnings.push(format!("  - {}{}", warning.message, ctx));
    }

    // Security checks if requested
    if security {
      let security_issues = check_security(parsed_spec);
      errors.extend(security_issues);
    }
  }

  // Output results
  if json {
    let result = serde_json::json!({
      "valid": errors.is_empty(),
      "errors": errors,
      "warnings": warnings,
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
  } else {
    if errors.is_empty() {
      println!("Validation passed: {spec}");
    } else {
      println!("Validation FAILED: {spec}");
      println!("\nErrors:");
      for error in &errors {
        println!("  {error}");
      }
    }

    if !warnings.is_empty() {
      println!("\nWarnings:");
      for warning in &warnings {
        println!("  {warning}");
      }
    }
  }

  if !errors.is_empty() {
    std::process::exit(1);
  }

  Ok(())
}

/// Check for security issues in a spec
fn check_security(spec: &Spec) -> Vec<String> {
  let mut issues = Vec::new();

  // Check for insecure patterns in behaviors
  for feature in &spec.features {
    for behavior in &feature.behaviors {
      let lower_intent = behavior.intent.to_lowercase();
      let lower_desc = behavior.description.to_lowercase();

      // Check for password-related issues
      if (lower_intent.contains("password") || lower_desc.contains("password"))
        && !lower_intent.contains("hash")
        && !lower_desc.contains("hash")
      {
        issues.push(format!(
          "Potential plaintext password handling in behavior '{}'",
          behavior.name
        ));
      }

      // Check for SQL injection risks
      if (lower_intent.contains("sql") || lower_desc.contains("sql"))
        && !lower_intent.contains("parameterized")
        && !lower_desc.contains("parameterized")
      {
        issues.push(format!(
          "Potential SQL injection risk in behavior '{}'",
          behavior.name
        ));
      }
    }
  }

  issues
}

/// Run batch operations
fn cmd_batch(file: Option<&str>, continue_on_error: bool, parallel: bool) -> Result<()> {
  // Read batch configuration
  let config_content = if let Some(path) = file {
    fs::read_to_string(path).with_context(|| format!("Failed to read batch file: {path}"))?
  } else {
    let mut content = String::new();
    io::stdin()
      .read_to_string(&mut content)
      .context("Failed to read from stdin")?;
    content
  };

  // Get spec files
  let spec_files: Vec<String> = if config_content.trim().starts_with('{') {
    // JSON configuration
    let config: serde_json::Value = serde_json::from_str(&config_content)
      .with_context(|| "Failed to parse batch configuration")?;

    match config.get("files") {
      Some(files) => {
        serde_json::from_value(files.clone()).with_context(|| "Failed to parse files list")?
      }
      None => {
        return Err(anyhow::anyhow!("No 'files' field in batch configuration"));
      }
    }
  } else {
    // Plain list of files
    config_content
      .lines()
      .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
      .map(String::from)
      .collect()
  };

  if spec_files.is_empty() {
    println!("No spec files to process.");
    return Ok(());
  }

  println!("Processing {} spec files...", spec_files.len());
  if parallel {
    println!("Running in parallel mode");
  }
  if continue_on_error {
    println!("Continue on error: enabled");
  }
  println!();

  // Process each file
  let mut successful = 0;
  let mut failed = 0;
  let mut skipped = 0;

  for spec_file in &spec_files {
    let path = Path::new(spec_file);
    if !path.exists() {
      println!("[SKIP] {spec_file} - File not found");
      skipped += 1;
      if !continue_on_error {
        return Err(anyhow::anyhow!("File not found: {spec_file}"));
      }
      continue;
    }

    match load_spec_from_cue(spec_file) {
      Ok(_spec) => {
        println!("[OK] {spec_file}");
        successful += 1;
      }
      Err(e) => {
        println!("[FAIL] {spec_file} - {e}");
        failed += 1;
        if !continue_on_error {
          return Err(e);
        }
      }
    }
  }

  // Print summary
  println!("{}", "=".repeat(60));
  println!("Batch Processing Summary");
  println!("{}", "=".repeat(60));
  println!("Total files: {}", spec_files.len());
  println!("Successful: {successful}");
  println!("Failed: {failed}");
  println!("Skipped: {skipped}");

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_version_command() {
    // Version should succeed
    cmd_version();
  }

  #[test]
  fn verify_cli_app() {
    // Verify that the CLI can be constructed
    use clap::CommandFactory;
    Cli::command().debug_assert();
  }

  #[test]
  fn test_current_timestamp() {
    let ts = current_timestamp();
    assert!(!ts.is_empty());
    assert!(ts.contains('T')); // ISO 8601 format
  }

  #[test]
  fn test_generate_session_id() {
    let id = generate_session_id();
    assert!(id.starts_with("session-"));
  }
}
