#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use clarity_web::storage::{EventEnvelope, FjallEventStore};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Parser)]
#[command(
  name = "clarity",
  version,
  about = "Clarity - Rust planning and interrogation CLI"
)]
struct Cli {
  #[arg(long, global = true)]
  json: bool,

  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Interview {
    #[command(subcommand)]
    command: InterviewCommand,
  },
  Gates {
    #[command(subcommand)]
    command: GatesCommand,
  },
  Spec {
    #[command(subcommand)]
    command: SpecCommand,
  },
  Beads {
    #[command(subcommand)]
    command: BeadsCommand,
  },
  Sessions {
    #[command(subcommand)]
    command: SessionsCommand,
  },
}

#[derive(Subcommand)]
enum InterviewCommand {
  Start {
    #[arg(long)]
    profile: RustProfile,
  },
  Resume {
    session_id: String,
  },
  Status {
    session_id: String,
  },
  Export {
    session_id: String,
    #[arg(long)]
    raw: bool,
  },
  Abort {
    session_id: String,
  },
}

#[derive(Subcommand)]
enum GatesCommand {
  Run { session_id: String },
}

#[derive(Subcommand)]
enum SpecCommand {
  Compile { session_id: String },
}

#[derive(Subcommand)]
enum BeadsCommand {
  Generate { session_id: String },
  Emit { session_id: String },
}

#[derive(Subcommand)]
enum SessionsCommand {
  List,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum RustProfile {
  RustCli,
  RustLibrary,
  RustWebService,
  RustAsyncService,
  RustStorage,
  RustUi,
  RustRefactor,
}

impl RustProfile {
  const fn as_str(self) -> &'static str {
    match self {
      Self::RustCli => "rust-cli",
      Self::RustLibrary => "rust-library",
      Self::RustWebService => "rust-web-service",
      Self::RustAsyncService => "rust-async-service",
      Self::RustStorage => "rust-storage",
      Self::RustUi => "rust-ui",
      Self::RustRefactor => "rust-refactor",
    }
  }
}

struct CliOutput {
  command: &'static str,
  session_id: Option<String>,
  state: Option<&'static str>,
  last_seq: u64,
  data: Value,
  warnings: Vec<String>,
  human: String,
}

struct CliFailure {
  command: &'static str,
  session_id: Option<String>,
  state: Option<&'static str>,
  last_seq: u64,
  exit_code: u8,
  error_code: &'static str,
  message: String,
  remediation: Option<&'static str>,
}

#[tokio::main]
async fn main() -> ExitCode {
  let cli = Cli::parse();
  let json_mode = cli.json;
  match run_cli(cli) {
    Ok(output) => print_success(&output, json_mode),
    Err(failure) => print_failure(&failure, json_mode),
  }
}

fn run_cli(cli: Cli) -> Result<CliOutput, CliFailure> {
  match cli.command {
    Commands::Interview { command } => run_interview(command),
    Commands::Gates { command } => run_gates(command),
    Commands::Spec { command } => run_spec(command),
    Commands::Beads { command } => run_beads(command),
    Commands::Sessions { command } => run_sessions(command),
  }
}

fn run_interview(command: InterviewCommand) -> Result<CliOutput, CliFailure> {
  match command {
    InterviewCommand::Start { profile } => start_interview(profile),
    InterviewCommand::Resume { session_id } => summarize_session("interview resume", session_id),
    InterviewCommand::Status { session_id } => summarize_session("interview status", session_id),
    InterviewCommand::Export { session_id, raw } => export_not_ready(session_id, raw),
    InterviewCommand::Abort { session_id } => abort_session(session_id),
  }
}

fn run_gates(command: GatesCommand) -> Result<CliOutput, CliFailure> {
  match command {
    GatesCommand::Run { session_id } => Err(foundation_pending(
      "gates run",
      Some(session_id),
      "GateScoreBelowThreshold",
      "deterministic gate evaluator implementation is not complete",
      4,
    )),
  }
}

fn run_spec(command: SpecCommand) -> Result<CliOutput, CliFailure> {
  match command {
    SpecCommand::Compile { session_id } => Err(foundation_pending(
      "spec compile",
      Some(session_id),
      "ArtifactGenerationIncomplete",
      "KIRK16 and CUE artifact compiler implementation is not complete",
      5,
    )),
  }
}

fn run_beads(command: BeadsCommand) -> Result<CliOutput, CliFailure> {
  match command {
    BeadsCommand::Generate { session_id } => Err(foundation_pending(
      "beads generate",
      Some(session_id),
      "ArtifactGenerationIncomplete",
      "enhanced bead generator implementation is not complete",
      5,
    )),
    BeadsCommand::Emit { session_id } => Err(foundation_pending(
      "beads emit",
      Some(session_id),
      "RawExportRequiresExplicitConsent",
      "bd emission requires validated local beads and explicit privacy consent",
      9,
    )),
  }
}

fn run_sessions(command: SessionsCommand) -> Result<CliOutput, CliFailure> {
  match command {
    SessionsCommand::List => Err(foundation_pending(
      "sessions list",
      None,
      "ArtifactGenerationIncomplete",
      "session index keyspace read model is not complete",
      5,
    )),
  }
}

fn start_interview(profile: RustProfile) -> Result<CliOutput, CliFailure> {
  let session_id = format!("clarity-{}", Uuid::new_v4());
  let store = open_store("interview start", None)?;
  let event = interview_started_event(&session_id, profile);
  store.append_event_sync_all(&event).map_err(|error| {
    storage_failure(
      "interview start",
      Some(session_id.clone()),
      error.to_string(),
    )
  })?;
  Ok(CliOutput {
    command: "interview start",
    session_id: Some(session_id.clone()),
    state: Some("Interviewing"),
    last_seq: 1,
    data: json!({
      "session_id": session_id,
      "profile": profile.as_str(),
      "state": "Interviewing",
      "next_action": "answer the first Double Diamond prompt",
      "storage": "fjall"
    }),
    warnings: Vec::new(),
    human: "Started Clarity interview session in Fjall event store.".to_string(),
  })
}

fn summarize_session(command: &'static str, session_id: String) -> Result<CliOutput, CliFailure> {
  let store = open_store(command, Some(session_id.clone()))?;
  let events = store
    .load_events(&session_id)
    .map_err(|error| storage_failure(command, Some(session_id.clone()), error.to_string()))?;
  let last_seq = events
    .iter()
    .map(|event| event.seq)
    .max()
    .map_or(0, |seq| seq);
  let state = events
    .last()
    .map_or("New", |event| state_after_event(&event.event_type));
  Ok(CliOutput {
    command,
    session_id: Some(session_id.clone()),
    state: Some(state),
    last_seq,
    data: json!({
      "session_id": session_id,
      "state": state,
      "last_seq": last_seq,
      "event_count": events.len(),
      "storage": "fjall"
    }),
    warnings: Vec::new(),
    human: format!("Session state: {state}; last_seq: {last_seq}"),
  })
}

fn abort_session(session_id: String) -> Result<CliOutput, CliFailure> {
  let store = open_store("interview abort", Some(session_id.clone()))?;
  let events = store.load_events(&session_id).map_err(|error| {
    storage_failure(
      "interview abort",
      Some(session_id.clone()),
      error.to_string(),
    )
  })?;
  let next_seq = events
    .iter()
    .map(|event| event.seq)
    .max()
    .map_or(1, |seq| seq.saturating_add(1));
  let event = session_event(
    &session_id,
    next_seq,
    "InterviewAborted",
    "interview abort",
    "Aborted",
  );
  store.append_event_sync_all(&event).map_err(|error| {
    storage_failure(
      "interview abort",
      Some(session_id.clone()),
      error.to_string(),
    )
  })?;
  Ok(CliOutput {
    command: "interview abort",
    session_id: Some(session_id.clone()),
    state: Some("Aborted"),
    last_seq: next_seq,
    data: json!({"session_id": session_id, "state": "Aborted", "last_seq": next_seq}),
    warnings: Vec::new(),
    human: "Session aborted.".to_string(),
  })
}

fn export_not_ready(session_id: String, raw: bool) -> Result<CliOutput, CliFailure> {
  let error_code = if raw {
    "RawExportRequiresExplicitConsent"
  } else {
    "ArtifactGenerationIncomplete"
  };
  Err(foundation_pending(
    "interview export",
    Some(session_id),
    error_code,
    "export writer and redaction pipeline are not complete",
    9,
  ))
}

fn interview_started_event(session_id: &str, profile: RustProfile) -> EventEnvelope {
  EventEnvelope {
    session_id: session_id.to_string(),
    seq: 1,
    event_id: Uuid::new_v4().to_string(),
    event_type: "InterviewStarted".to_string(),
    payload: json!({
      "kind": "session",
      "command": "interview start",
      "source_state": "New",
      "destination_state": "Interviewing",
      "profile": profile.as_str()
    }),
    created_at: Utc::now().to_rfc3339(),
    idempotency_key: format!("session-start:{session_id}"),
    schema_version: "1.0.0".to_string(),
    actor: "System".to_string(),
    prev_event_hash: None,
    event_hash: None,
  }
}

fn session_event(
  session_id: &str,
  seq: u64,
  event_type: &str,
  command: &str,
  destination_state: &str,
) -> EventEnvelope {
  EventEnvelope {
    session_id: session_id.to_string(),
    seq,
    event_id: Uuid::new_v4().to_string(),
    event_type: event_type.to_string(),
    payload: json!({"kind": "session", "command": command, "destination_state": destination_state}),
    created_at: Utc::now().to_rfc3339(),
    idempotency_key: format!("{command}:{session_id}:{seq}"),
    schema_version: "1.0.0".to_string(),
    actor: "System".to_string(),
    prev_event_hash: None,
    event_hash: None,
  }
}

fn state_after_event(event_type: &str) -> &'static str {
  match event_type {
    "InterviewStarted" => "Interviewing",
    "NormalQuestioningFrozen" => "NormalQuestioningFrozen",
    "ReviewerPanelStarted" => "Reviewing",
    "InterviewExhausted" => "InterviewExhausted",
    "SpecCompleted" => "SpecComplete",
    "InterviewAborted" => "Aborted",
    "RecoveredDegraded" => "RecoveredDegraded",
    _ => "Interviewing",
  }
}

fn open_store(
  command: &'static str,
  session_id: Option<String>,
) -> Result<FjallEventStore, CliFailure> {
  let db_root = default_db_root().map_err(|message| CliFailure {
    command,
    session_id: session_id.clone(),
    state: None,
    last_seq: 0,
    exit_code: 7,
    error_code: "FjallCommitFailed",
    message,
    remediation: Some("set CLARITY_FJALL_ROOT or fix local data directory permissions"),
  })?;
  std::fs::create_dir_all(&db_root).map_err(|error| CliFailure {
    command,
    session_id: session_id.clone(),
    state: None,
    last_seq: 0,
    exit_code: 7,
    error_code: "FjallCommitFailed",
    message: error.to_string(),
    remediation: Some("fix local data directory permissions"),
  })?;
  FjallEventStore::open(&db_root)
    .map_err(|error| storage_failure(command, session_id, error.to_string()))
}

fn default_db_root() -> Result<PathBuf, String> {
  match std::env::var_os("CLARITY_FJALL_ROOT") {
    Some(path) => Ok(PathBuf::from(path)),
    None => dirs::data_local_dir()
      .map(|path| path.join("clarity").join("fjall"))
      .ok_or_else(|| "could not resolve local data directory".to_string()),
  }
}

fn storage_failure(
  command: &'static str,
  session_id: Option<String>,
  message: String,
) -> CliFailure {
  CliFailure {
    command,
    session_id,
    state: None,
    last_seq: 0,
    exit_code: 7,
    error_code: "FjallCommitFailed",
    message,
    remediation: Some("inspect the Fjall database path and retry the command"),
  }
}

fn foundation_pending(
  command: &'static str,
  session_id: Option<String>,
  error_code: &'static str,
  message: &'static str,
  exit_code: u8,
) -> CliFailure {
  CliFailure {
    command,
    session_id,
    state: None,
    last_seq: 0,
    exit_code,
    error_code,
    message: message.to_string(),
    remediation: Some("complete the corresponding foundation bead from MASTER_DOC.md"),
  }
}

fn print_success(output: &CliOutput, json_mode: bool) -> ExitCode {
  if json_mode {
    match serde_json::to_string_pretty(&success_envelope(output)) {
      Ok(text) => println!("{text}"),
      Err(error) => return print_failure(&serialization_failure(error.to_string()), true),
    }
  } else {
    println!("{}", output.human);
  }
  ExitCode::SUCCESS
}

fn print_failure(failure: &CliFailure, json_mode: bool) -> ExitCode {
  if json_mode {
    match serde_json::to_string_pretty(&failure_envelope(failure)) {
      Ok(text) => eprintln!("{text}"),
      Err(error) => eprintln!("serialization failure: {error}"),
    }
  } else {
    eprintln!("{}: {}", failure.error_code, failure.message);
    if let Some(remediation) = failure.remediation {
      eprintln!("remediation: {remediation}");
    }
  }
  ExitCode::from(failure.exit_code)
}

fn success_envelope(output: &CliOutput) -> Value {
  json!({
    "ok": true,
    "command": output.command,
    "session_id": output.session_id,
    "state": output.state,
    "last_seq": output.last_seq,
    "data": output.data,
    "warnings": output.warnings
  })
}

fn failure_envelope(failure: &CliFailure) -> Value {
  json!({
    "ok": false,
    "command": failure.command,
    "session_id": failure.session_id,
    "state": failure.state,
    "last_seq": failure.last_seq,
    "error": {
      "error_code": failure.error_code,
      "message": failure.message,
      "remediation": failure.remediation,
      "evidence_event_ids": []
    },
    "warnings": []
  })
}

fn serialization_failure(message: String) -> CliFailure {
  CliFailure {
    command: "json serialization",
    session_id: None,
    state: None,
    last_seq: 0,
    exit_code: 1,
    error_code: "SerializationFailed",
    message,
    remediation: Some("report this bug with the command that produced invalid JSON output"),
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
  use super::*;

  #[test]
  fn rust_profile_serializes_to_master_doc_values() {
    assert_eq!(RustProfile::RustCli.as_str(), "rust-cli");
    assert_eq!(RustProfile::RustStorage.as_str(), "rust-storage");
  }

  #[test]
  fn success_envelope_uses_stable_shape() {
    let output = CliOutput {
      command: "interview status",
      session_id: Some("clarity-test".to_string()),
      state: Some("Interviewing"),
      last_seq: 1,
      data: json!({"event_count": 1}),
      warnings: Vec::new(),
      human: "ok".to_string(),
    };
    let envelope = success_envelope(&output);
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "interview status");
    assert_eq!(envelope["state"], "Interviewing");
  }
}
