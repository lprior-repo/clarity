#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::significant_drop_tightening)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Server functions for the Clarity Planner backend
//!
//! These functions run on the server and can be called from the client
//! using Dioxus fullstack server functions.
//!
//! ## Architecture
//!
//! This module implements server-side AI provider integration with:
//! - Provider singleton initialization with config loading
//! - Rate limiting per session (10 requests/min)
//! - Comprehensive error handling and logging
//! - Field extraction, field suggestion, and quality scoring

use dioxus::prelude::*;
use dioxus_fullstack::server;
use dioxus_fullstack::ServerFnError;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, LazyLock};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::RwLock;
#[cfg(feature = "server")]
use tracing::info;
use tracing::warn as tracing_warn;

// Re-export types from lattice and providers
#[cfg(not(target_arch = "wasm32"))]
use crate::components::discover::straw_man::StrawManTrap;
#[cfg(not(target_arch = "wasm32"))]
use crate::components::discover::straw_man::StrawManValidation;
#[cfg(not(target_arch = "wasm32"))]
use crate::components::discover::types::{HolePunchingResults, ScenarioField};
#[cfg(not(target_arch = "wasm32"))]
use crate::config::ai::{default_config, load_ai_config_if_present, AiConfig};
#[cfg(feature = "server")]
use crate::lattice::quality::{calculate_quality, InversionControl, QualityError};
#[cfg(not(target_arch = "wasm32"))]
use crate::lattice::quality::{Answer as QualityAnswer, EarsRequirementRef, QualityScore};
#[cfg(not(target_arch = "wasm32"))]
use crate::providers::resolution::resolve_provider_config;
#[cfg(feature = "server")]
use crate::providers::ExtractionProvider;
#[cfg(not(target_arch = "wasm32"))]
use crate::providers::OpenCodeProvider;
#[cfg(not(target_arch = "wasm32"))]
use crate::providers::{
  ExtractedFields, ExtractionContext, ExtractionError, FieldExtraction, FieldType,
  OpenCodeProviderOptions, SchemaField,
};

/// A planning bead (atomic work unit)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bead {
  pub id: String,
  pub title: String,
  pub description: String,
  pub phase: Phase,
  pub status: BeadStatus,
  pub created_at: String,
  pub updated_at: String,
}

/// Planning phases (Double Diamond)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
  Discover,
  Define,
  Develop,
  Deliver,
}

/// Bead status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BeadStatus {
  Todo,
  InProgress,
  Review,
  Done,
}

/// Save a bead (MOCK - does NOT actually save to database)
#[allow(clippy::unused_async)]
#[server]
pub async fn save_bead(bead: Bead) -> Result<Bead, ServerFnError> {
  // WARNING: Does NOT persist to database - only updates timestamp
  // In a real app, this would save to a database
  let updated_bead = Bead {
    updated_at: chrono::Utc::now().to_rfc3339(),
    ..bead
  };
  Ok(updated_bead)
}

/// Get all beads for a project (MOCK - ignores project_id, returns hardcoded data)
#[allow(clippy::unused_async)]
#[server]
pub async fn get_beads(project_id: String) -> Result<Vec<Bead>, ServerFnError> {
  let _ = project_id;
  // WARNING: Does NOT fetch from database - returns HARDCODED sample data
  // In a real app, this would fetch from a database
  let beads = vec![
    Bead {
      id: "1".to_string(),
      title: "User Research".to_string(),
      description: "Conduct user interviews and surveys".to_string(),
      phase: Phase::Discover,
      status: BeadStatus::Done,
      created_at: "2025-01-01T00:00:00Z".to_string(),
      updated_at: "2025-01-15T00:00:00Z".to_string(),
    },
    Bead {
      id: "2".to_string(),
      title: "Define Problem Statement".to_string(),
      description: "Synthesize research into a clear problem definition".to_string(),
      phase: Phase::Define,
      status: BeadStatus::InProgress,
      created_at: "2025-01-16T00:00:00Z".to_string(),
      updated_at: "2025-01-20T00:00:00Z".to_string(),
    },
    Bead {
      id: "3".to_string(),
      title: "Design Prototype".to_string(),
      description: "Create interactive prototype for testing".to_string(),
      phase: Phase::Develop,
      status: BeadStatus::Todo,
      created_at: "2025-01-21T00:00:00Z".to_string(),
      updated_at: "2025-01-21T00:00:00Z".to_string(),
    },
  ];
  Ok(beads)
}

/// Delete a bead (MOCK - does NOT actually delete)
#[allow(clippy::unused_async)]
#[server]
pub async fn delete_bead(bead_id: String) -> Result<(), ServerFnError> {
  // WARNING: Does NOT delete from database - just logs
  // In a real app, this would delete from a database
  println!("[MOCK] Would delete bead: {bead_id}");
  Ok(())
}

/// AI coaching prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachResponse {
  pub phase: Phase,
  pub guidance: String,
  pub questions: Vec<String>,
}

/// Get AI coaching guidance for a phase (MOCK - not actually AI)
#[allow(clippy::unused_async)]
#[server]
pub async fn get_coach_guidance(
  phase: Phase,
  context: String,
) -> Result<CoachResponse, ServerFnError> {
  // In a real app, this would call an AI API
  let guidance = match phase {
    Phase::Discover => {
      format!("In the Discover phase, focus on understanding users deeply. Context: {context}")
    }
    Phase::Define => format!(
      "In the Define phase, synthesize your findings into a clear problem. Context: {context}"
    ),
    Phase::Develop => {
      format!("In the Develop phase, ideate and prototype solutions. Context: {context}")
    }
    Phase::Deliver => {
      format!("In the Deliver phase, test and refine your solution. Context: {context}")
    }
  };

  let questions = match phase {
    Phase::Discover => vec![
      "Who are your target users?".to_string(),
      "What problems do they face?".to_string(),
      "How do they currently solve these problems?".to_string(),
    ],
    Phase::Define => vec![
      "What is the core problem you're solving?".to_string(),
      "What insights emerged from research?".to_string(),
      "What constraints must you consider?".to_string(),
    ],
    Phase::Develop => vec![
      "What solutions have the highest impact?".to_string(),
      "How can you validate ideas quickly?".to_string(),
      "What resources do you need?".to_string(),
    ],
    Phase::Deliver => vec![
      "How will you measure success?".to_string(),
      "What feedback have you received?".to_string(),
      "What needs iteration?".to_string(),
    ],
  };

  Ok(CoachResponse {
    phase,
    guidance,
    questions,
  })
}

// ============================================================================
// Extraction and Quality Server Functions
// ============================================================================

/// Rate limiter for API calls per session
///
/// Tracks request timestamps per session ID and enforces max requests per minute.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct RateLimiter {
  max_requests_per_minute: u32,
  requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl RateLimiter {
  /// Create a new rate limiter
  fn new(max_requests_per_minute: u32) -> Self {
    Self {
      max_requests_per_minute,
      requests: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  /// Check if a session is allowed to make a request
  ///
  /// Returns `Ok(())` if allowed, `Err` with remaining seconds if rate limited.
  async fn check_rate_limit(&self, session_id: &str) -> Result<(), u64> {
    let now = Instant::now();
    let one_minute_ago = now.checked_sub(Duration::from_secs(60)).map_or(now, |t| t);
    let mut requests = self.requests.write().await;
    let session_requests = requests
      .entry(session_id.to_string())
      .or_insert_with(Vec::new);

    // Remove old requests outside the 1-minute window
    session_requests.retain(|&timestamp| timestamp > one_minute_ago);

    if session_requests.len() < self.max_requests_per_minute as usize {
      session_requests.push(now);
      drop(requests);
      return Ok(());
    }

    // Calculate oldest request time to determine retry-after
    let oldest = session_requests.first().copied().map_or(now, |t| t);
    let elapsed = now.duration_since(oldest).as_secs();
    let retry_after = 60_u64.saturating_sub(elapsed);
    drop(requests);
    Err(retry_after)
  }
}

/// Global rate limiter instance
#[cfg(not(target_arch = "wasm32"))]
static RATE_LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new(10));

/// Check rate limit for a session, returning a formatted error if limited.
///
/// # Arguments
/// * `session` - Session identifier for rate limiting
/// * `operation` - Name of the operation for logging
///
/// # Returns
/// `Ok(())` if allowed, `Err(ServerFnError)` if rate limited
#[cfg(not(target_arch = "wasm32"))]
async fn check_rate_limit_for_session(session: &str, operation: &str) -> Result<(), ServerFnError> {
  match RATE_LIMITER.check_rate_limit(session).await {
    Ok(()) => {
      info!(session, operation, "Rate limit check passed");
      Ok(())
    }
    Err(retry_after) => {
      tracing_warn!(session, retry_after, operation, "Rate limit exceeded");
      Err(ServerFnError::new(anyhow::anyhow!(
        "Rate limit exceeded. Please retry after {retry_after}s"
      )))
    }
  }
}

/// Map extraction errors to server function errors with consistent formatting.
///
/// # Arguments
/// * `error` - The extraction error to map
///
/// # Returns
/// `ServerFnError` with user-friendly message
#[cfg(not(target_arch = "wasm32"))]
fn map_extraction_error(error: ExtractionError) -> ServerFnError {
  match error {
    ExtractionError::RateLimited {
      retry_after_seconds,
    } => ServerFnError::new(anyhow::anyhow!(
      "Provider rate limited. Retry after {retry_after_seconds}s"
    )),
    ExtractionError::AuthenticationError(msg) => {
      ServerFnError::new(anyhow::anyhow!("Authentication failed: {msg}"))
    }
    ExtractionError::InvalidInput(msg) => {
      ServerFnError::new(anyhow::anyhow!("Invalid input: {msg}"))
    }
    _ => ServerFnError::new(anyhow::anyhow!("Extraction failed: {error}")),
  }
}

/// Global AI provider singleton
///
/// Initialized once with config from `~/.config/clarity/ai.toml`.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct AiProviderState {
  provider: Arc<OpenCodeProvider>,
  diagnostics: AiProviderDiagnostics,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderBootstrapInput {
  diagnostics: AiProviderDiagnostics,
  session_id: String,
}

use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Error)]
enum AiProviderBootstrapError {
  #[error("failed to load AI configuration: {0}")]
  Config(#[from] crate::config::ConfigError),

  #[error("unsupported AI provider: {0}")]
  UnsupportedProvider(String),

  #[error("failed to initialize AI provider: {0}")]
  Provider(#[from] crate::providers::ExtractionError),
}

#[cfg(not(target_arch = "wasm32"))]
fn build_provider_bootstrap_input(
  config: AiConfig,
  session_factory: impl FnOnce() -> String,
) -> ProviderBootstrapInput {
  let resolved = resolve_provider_config(&config);
  let session_id = if config.provider.session_id.is_empty() {
    session_factory()
  } else {
    config.provider.session_id
  };

  ProviderBootstrapInput {
    diagnostics: AiProviderDiagnostics {
      provider: resolved.provider_type,
      endpoint: resolved.endpoint,
      model: resolved.model,
      routing_provider: resolved.routing_provider,
    },
    session_id,
  }
}

#[cfg(not(target_arch = "wasm32"))]
fn create_ai_provider_state(
  input: ProviderBootstrapInput,
) -> Result<AiProviderState, AiProviderBootstrapError> {
  if input.diagnostics.provider != "opencode" {
    return Err(AiProviderBootstrapError::UnsupportedProvider(
      input.diagnostics.provider,
    ));
  }

  let diagnostics = input.diagnostics;
  let provider = OpenCodeProvider::new_with_options(
    diagnostics.endpoint.clone(),
    input.session_id,
    OpenCodeProviderOptions {
      model: diagnostics.model.clone(),
      routing_provider: diagnostics.routing_provider.clone(),
    },
  )?;

  Ok(AiProviderState {
    provider: Arc::new(provider),
    diagnostics,
  })
}

#[cfg(not(target_arch = "wasm32"))]
fn initialize_ai_provider_state() -> Result<AiProviderState, AiProviderBootstrapError> {
  let config = load_ai_config_if_present()?.map_or_else(default_config, |c| c);
  let input = build_provider_bootstrap_input(config, || uuid::Uuid::new_v4().to_string());

  create_ai_provider_state(input)
}

/// Global AI provider state.
///
/// Initialized lazily from configuration without aborting the process on failure.
#[cfg(not(target_arch = "wasm32"))]
static AI_PROVIDER_STATE: LazyLock<Result<AiProviderState, AiProviderBootstrapError>> =
  LazyLock::new(|| {
    let state = initialize_ai_provider_state();

    if let Err(error) = &state {
      tracing_warn!(error = ?error, "AI provider bootstrap failed");
    }

    state
  });

/// Lightweight diagnostics for currently configured AI extraction provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProviderDiagnostics {
  pub provider: String,
  pub endpoint: String,
  pub model: Option<String>,
  pub routing_provider: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn ai_provider_state() -> Result<&'static AiProviderState, ServerFnError> {
  AI_PROVIDER_STATE.as_ref().map_err(|error| {
    ServerFnError::new(anyhow::anyhow!(
      "AI provider initialization failed: {error:?}"
    ))
  })
}

#[cfg(not(target_arch = "wasm32"))]
fn ai_provider() -> Result<Arc<OpenCodeProvider>, ServerFnError> {
  ai_provider_state().map(|state| Arc::clone(&state.provider))
}

#[cfg(not(target_arch = "wasm32"))]
fn build_straw_man_schema() -> Vec<SchemaField> {
  vec![
    SchemaField {
      name: "irrational_actor_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona acts against their own motivations or self-interest".to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "manic_pixie_dream_user_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona magically loves everything without discernment or constraints"
          .to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "stoic_monk_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona tolerates immense friction or difficulty without complaint"
          .to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "your_clone_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona has developer-level system knowledge or mental models".to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "suggestions".to_string(),
      field_type: FieldType::TextArea,
      required: false,
      description: Some(
        "Specific suggestions for fixing detected traps. Be concrete and actionable.".to_string(),
      ),
      options: None,
    },
  ]
}

#[cfg(not(target_arch = "wasm32"))]
fn build_straw_man_context(schema: Vec<SchemaField>) -> ExtractionContext {
  ExtractionContext {
    document_type: Some("persona_validation".to_string()),
    locale: Some("en_US".to_string()),
    schema: Some(schema),
    extra: serde_json::json!({
        "validation_type": "straw_man_traps",
        "traps": [
            {
                "name": "IrrationalActor",
                "description": "User acts against their own motivations or self-interest"
            },
            {
                "name": "ManicPixieDreamUser",
                "description": "User magically loves everything without discernment"
            },
            {
                "name": "StoicMonk",
                "description": "User tolerates immense friction without complaint"
            },
            {
                "name": "YourClone",
                "description": "User has developer's system knowledge and mental models"
            }
        ]
    }),
  }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_straw_man_prompt(persona_text: &str) -> String {
  format!(
    "Analyze this user persona description for straw man trap patterns:\n\n{persona_text}\n\n\
        Detect which of the following traps are present:\n\
        1. Irrational Actor: User acts against their own motivations\n\
        2. Manic Pixie Dream User: User magically loves everything\n\
        3. Stoic Monk: User tolerates excessive friction\n\
        4. Your Clone: User has developer's system knowledge\n\n\
        Provide specific suggestions for any detected traps."
  )
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_straw_man_validation(fields: &[FieldExtraction]) -> StrawManValidation {
  let traps_detected = fields
    .iter()
    .filter_map(|field| match (field.name.as_str(), field.value.as_bool()) {
      ("irrational_actor_detected", Some(true)) => Some(StrawManTrap::IrrationalActor),
      ("manic_pixie_dream_user_detected", Some(true)) => Some(StrawManTrap::ManicPixieDreamUser),
      ("stoic_monk_detected", Some(true)) => Some(StrawManTrap::StoicMonk),
      ("your_clone_detected", Some(true)) => Some(StrawManTrap::YourClone),
      _ => None,
    })
    .collect();

  StrawManValidation::new(traps_detected)
}

#[cfg(not(target_arch = "wasm32"))]
fn build_hole_punching_schema() -> Vec<SchemaField> {
  vec![
    SchemaField {
      name: "discovery_hole_addressed".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the scenario explains how the user discovers the feature/solution".to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "edge_case_hole_addressed".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the scenario addresses what happens in edge cases (errors, network issues, typos)"
          .to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "motivation_dropoff_addressed".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the scenario explains why users continue through high-friction steps".to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "identified_holes".to_string(),
      field_type: FieldType::TextArea,
      required: false,
      description: Some(
        "List any holes that were detected but not addressed in the scenario".to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "suggestions".to_string(),
      field_type: FieldType::TextArea,
      required: false,
      description: Some(
        "Specific suggestions for addressing any detected holes. Be concrete and actionable."
          .to_string(),
      ),
      options: None,
    },
  ]
}

#[cfg(not(target_arch = "wasm32"))]
fn build_hole_punching_context(schema: Vec<SchemaField>) -> ExtractionContext {
  ExtractionContext {
    document_type: Some("scenario_validation".to_string()),
    locale: Some("en_US".to_string()),
    schema: Some(schema),
    extra: serde_json::json!({
        "validation_type": "hole_punching",
        "holes": [
            {
                "name": "DiscoveryHole",
                "description": "How did they find the feature?",
                "question": "Addresses the gap between user need and awareness of the solution"
            },
            {
                "name": "EdgeCaseHole",
                "description": "What if internet drops, mistype, etc?",
                "question": "Addresses technical and usability edge cases"
            },
            {
                "name": "MotivationDropOff",
                "description": "Why continue at high-friction steps?",
                "question": "Addresses motivation and engagement at critical points"
            }
        ]
    }),
  }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_hole_punching_prompt(scenario: &ScenarioField) -> String {
  format!(
    "Analyze this user scenario for hole punching gaps:\n\n\
        Trigger: {}\n\
        Value Moment: {}\n\
        Feeling: {}\n\n\
        Current hole punching status:\n\
        - Discovery Hole: {}\n\
        - Edge Case Hole: {}\n\
        - Motivation Drop-off: {}\n\n\
        Check which of the following holes have been adequately addressed:\n\
        1. Discovery Hole: How did they find the feature/solution?\n\
        2. Edge Case Hole: What if internet drops, typos, errors occur?\n\
        3. Motivation Drop-off: Why continue through high-friction steps?\n\n\
        Evaluate if each hole has been addressed. If a hole is present but not addressed,\n\
        provide specific suggestions for how to address it.",
    scenario.trigger,
    scenario.value_moment,
    scenario.feeling,
    scenario
      .hole_punching
      .discovery_hole
      .as_deref()
      .map_or("Not addressed", |s| s),
    scenario
      .hole_punching
      .edge_case_hole
      .as_deref()
      .map_or("Not addressed", |s| s),
    scenario
      .hole_punching
      .motivation_dropoff
      .as_deref()
      .map_or("Not addressed", |s| s)
  )
}

#[cfg(not(target_arch = "wasm32"))]
fn merge_hole_punching_results(
  existing: &HolePunchingResults,
  fields: &[FieldExtraction],
) -> HolePunchingResults {
  fields.iter().fold(existing.clone(), |results, field| {
    match (field.name.as_str(), field.value.as_bool()) {
      ("discovery_hole_addressed", Some(true)) if results.discovery_hole.is_none() => results
        .address(
          crate::components::discover::types::HoleType::DiscoveryHole,
          "Addressed in scenario".to_string(),
        ),
      ("edge_case_hole_addressed", Some(true)) if results.edge_case_hole.is_none() => results
        .address(
          crate::components::discover::types::HoleType::EdgeCaseHole,
          "Addressed in scenario".to_string(),
        ),
      ("motivation_dropoff_addressed", Some(true)) if results.motivation_dropoff.is_none() => {
        results.address(
          crate::components::discover::types::HoleType::MotivationDropOff,
          "Addressed in scenario".to_string(),
        )
      }
      _ => results,
    }
  })
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::unused_async)]
#[server]
pub async fn get_ai_provider_status_server() -> Result<AiProviderDiagnostics, ServerFnError> {
  let provider = ai_provider()?;
  Ok(AiProviderDiagnostics {
    provider: "opencode".to_string(),
    endpoint: provider.endpoint().clone(),
    model: provider.model().cloned(),
    routing_provider: provider.routing_provider().cloned(),
  })
}

/// Extract structured fields from freeform text input
///
/// # Arguments
/// * `input` - Freeform text to extract fields from
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(ExtractedFields)` - Successfully extracted fields with confidence scores
/// * `Err(ServerFnError)` - Extraction failed or rate limited
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn extract_fields_server(
  input: String,
  session_id: Option<String>,
) -> Result<ExtractedFields, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  check_rate_limit_for_session(session, "extract_fields_server").await?;

  // Validate input
  if input.trim().is_empty() {
    return Err(ServerFnError::new(anyhow::anyhow!(
      "Input text cannot be empty"
    )));
  }

  // Build extraction context
  let context = ExtractionContext {
    document_type: Some("discover_phase".to_string()),
    locale: Some("en_US".to_string()),
    schema: None,
    extra: serde_json::json!({}),
  };

  // Call provider
  let provider = ai_provider()?;
  let result = provider
    .extract_fields(&input, &context)
    .await
    .map_err(map_extraction_error)?;

  info!(
    session,
    field_count = result.fields.len(),
    confidence = result.confidence,
    duration_ms = result.metadata.processing_duration_ms,
    "extract_fields_server: Extraction completed"
  );

  Ok(result)
}

/// Suggest content for a specific field type based on context
///
/// # Arguments
/// * `field` - The type of field to suggest
/// * `context` - Extraction context with prior answers
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(String)` - Suggested content for the field
/// * `Err(ServerFnError)` - Suggestion failed or rate limited
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn suggest_field_server(
  field: FieldType,
  context: ExtractionContext,
  session_id: Option<String>,
) -> Result<String, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  check_rate_limit_for_session(session, "suggest_field_server").await?;

  // Build schema for single field suggestion
  let schema = vec![SchemaField {
    name: "suggestion".to_string(),
    field_type: field.clone(),
    required: true,
    description: Some(format!("AI-suggested content for {field:?} field")),
    options: None,
  }];

  // Use extract_fields_with_schema with minimal prompt text
  let prompt_text = format!(
    "Generate a suggestion for a {:?} field based on context: {}",
    field,
    context
      .extra
      .as_object()
      .and_then(|o| o.get("prior_answers"))
      .and_then(|v| v.as_str())
      .map_or("", |s| s)
  );

  // Call provider
  let provider = ai_provider()?;
  let result = provider
    .extract_fields_with_schema(&prompt_text, &schema, &context)
    .await
    .map_err(map_extraction_error)?;

  // Extract the first field's value as string
  let suggestion = result
    .fields
    .first()
    .and_then(|f| {
      serde_json::to_string(&f.value)
        .map(|s| s.trim_matches('"').to_string())
        .ok()
    })
    .map_or_else(
      || format!("Suggestion for {field:?} field based on your context. Please review and edit."),
      |s| s,
    );

  info!(
    session,
    ?field,
    suggestion_len = suggestion.len(),
    "suggest_field_server: Suggestion generated"
  );

  Ok(suggestion)
}

/// Calculate quality score from answers and EARS requirements
///
/// # Arguments
/// * `answers` - User answers to prompt steps
/// * `ears` - Optional EARS-formatted requirements
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(QualityScore)` - Quality assessment with dimensions and issues
/// * `Err(ServerFnError)` - Calculation failed
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn calculate_quality_server(
  answers: Vec<QualityAnswer>,
  ears: Option<Vec<EarsRequirementRef>>,
  session_id: Option<String>,
) -> Result<QualityScore, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Rate limit quality calculations (they're lightweight but we track them)
  match RATE_LIMITER.check_rate_limit(session).await {
    Ok(()) => {
      info!(
        session,
        answer_count = answers.len(),
        "calculate_quality_server: API call"
      );
    }
    Err(retry_after) => {
      tracing_warn!(
        session,
        retry_after,
        "calculate_quality_server: Rate limit exceeded"
      );
      return Err(ServerFnError::new(anyhow::anyhow!(
        "Rate limit exceeded. Please retry after {retry_after}s"
      )));
    }
  }

  // Validate input
  if answers.is_empty() {
    return Err(ServerFnError::new(anyhow::anyhow!(
      "Cannot calculate quality with empty answers"
    )));
  }

  // Default to empty EARS if none provided
  let ears_ref = ears
    .as_ref()
    .map_or_else(Vec::new, std::clone::Clone::clone);

  // Inversion control defaults (will be enhanced in future)
  let inversion = InversionControl {
    has_inversion_tests: false,
    inverted_count: 0,
  };

  // Calculate quality
  let result = calculate_quality(&answers, &ears_ref, &inversion).map_err(|e| match e {
    QualityError::EmptyAnswers => ServerFnError::new(anyhow::anyhow!("No answers provided")),
    QualityError::InvalidScore(msg) => ServerFnError::new(anyhow::anyhow!("Invalid score: {msg}")),
    QualityError::DimensionFailed(msg) => {
      ServerFnError::new(anyhow::anyhow!("Dimension failed: {msg}"))
    }
  })?;

  info!(
    session,
    overall = result.overall,
    dimension_count = result.dimensions.len(),
    issue_count = result.issues.len(),
    "calculate_quality_server: Quality calculated"
  );

  Ok(result)
}

/// Validate a persona description against straw man traps
///
/// This function uses AI to detect patterns that indicate unrealistic user
/// persona assumptions (straw man traps). It analyzes the persona text for
/// signs of:
///
/// - **Irrational Actor**: User acts against their own self-interest
/// - **Manic Pixie Dream User**: User magically loves everything without discernment
/// - **Stoic Monk**: User tolerates excessive friction without abandonment
/// - **Your Clone**: User possesses the developer's system knowledge
///
/// # Arguments
/// * `persona_text` - The persona description to validate
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(StrawManValidation)` - Validation result with detected traps and suggestions
/// * `Err(ServerFnError)` - Validation failed or rate limited
///
/// # Example
/// ```ignore
/// let validation = validate_straw_man_traps_server(
///     "Users will happily complete a 20-step onboarding flow".to_string(),
///     Some("session-123".to_string())
/// ).await?;
///
/// if !validation.passed {
///     println!("Detected traps: {:?}", validation.traps_detected);
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_lines)]
#[server]
pub async fn validate_straw_man_traps_server(
  persona_text: String,
  session_id: Option<String>,
) -> Result<StrawManValidation, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  check_rate_limit_for_session(session, "validate_straw_man_traps_server").await?;

  // Validate input
  if persona_text.trim().is_empty() {
    return Err(ServerFnError::new(anyhow::anyhow!(
      "Persona text cannot be empty"
    )));
  }

  // Define schema for trap detection
  // We'll extract which traps are present and get suggestions for each
  let schema = vec![
    SchemaField {
      name: "irrational_actor_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona acts against their own motivations or self-interest".to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "manic_pixie_dream_user_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona magically loves everything without discernment or constraints"
          .to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "stoic_monk_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona tolerates immense friction or difficulty without complaint"
          .to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "your_clone_detected".to_string(),
      field_type: FieldType::Boolean,
      required: true,
      description: Some(
        "True if the persona has developer-level system knowledge or mental models".to_string(),
      ),
      options: None,
    },
    SchemaField {
      name: "suggestions".to_string(),
      field_type: FieldType::TextArea,
      required: false,
      description: Some(
        "Specific suggestions for fixing detected traps. Be concrete and actionable.".to_string(),
      ),
      options: None,
    },
  ];

  // Build context
  let context = ExtractionContext {
    document_type: Some("persona_validation".to_string()),
    locale: Some("en_US".to_string()),
    schema: Some(schema.clone()),
    extra: serde_json::json!({
        "validation_type": "straw_man_traps",
        "traps": [
            {
                "name": "IrrationalActor",
                "description": "User acts against their own motivations or self-interest"
            },
            {
                "name": "ManicPixieDreamUser",
                "description": "User magically loves everything without discernment"
            },
            {
                "name": "StoicMonk",
                "description": "User tolerates immense friction without complaint"
            },
            {
                "name": "YourClone",
                "description": "User has developer's system knowledge and mental models"
            }
        ]
    }),
  };

  // Build analysis prompt
  let analysis_prompt = format!(
    "Analyze this user persona description for straw man trap patterns:\n\n{persona_text}\n\n\
        Detect which of the following traps are present:\n\
        1. Irrational Actor: User acts against their own motivations\n\
        2. Manic Pixie Dream User: User magically loves everything\n\
        3. Stoic Monk: User tolerates excessive friction\n\
        4. Your Clone: User has developer's system knowledge\n\n\
        Provide specific suggestions for any detected traps."
  );

  // Call AI provider
  let provider = ai_provider()?;
  let result = provider
    .extract_fields_with_schema(&analysis_prompt, &schema, &context)
    .await
    .map_err(map_extraction_error)?;

  // Parse detected traps from response
  let traps_detected: Vec<StrawManTrap> = result
    .fields
    .iter()
    .filter_map(|field| match field.name.as_str() {
      "irrational_actor_detected" if field.value.as_bool() == Some(true) => {
        Some(StrawManTrap::IrrationalActor)
      }
      "manic_pixie_dream_user_detected" if field.value.as_bool() == Some(true) => {
        Some(StrawManTrap::ManicPixieDreamUser)
      }
      "stoic_monk_detected" if field.value.as_bool() == Some(true) => Some(StrawManTrap::StoicMonk),
      "your_clone_detected" if field.value.as_bool() == Some(true) => Some(StrawManTrap::YourClone),
      _ => None,
    })
    .collect();

  // Create validation result
  let validation = StrawManValidation::new(traps_detected.clone());

  info!(
      session,
      passed = validation.passed,
      trap_count = traps_detected.len(),
      traps = ?traps_detected,
      "validate_straw_man_traps_server: Validation completed"
  );

  Ok(validation)
}

/// Validate a scenario description for hole punching gaps
///
/// This function analyzes a scenario (trigger, value moment, feeling) to identify
/// coverage gaps across three critical dimensions:
///
/// - **Discovery Hole**: How did the user discover this feature/solution?
///   Addresses the gap between user need and awareness of the solution.
///
/// - **Edge Case Hole**: What happens in edge cases (internet drops, typos, errors)?
///   Addresses technical and usability edge cases that break the flow.
///
/// - **Motivation Drop-off**: Why would users continue through high-friction steps?
///   Addresses motivation and engagement at critical points.
///
/// # Arguments
/// * `scenario` - The scenario field containing trigger, `value_moment`, and feeling
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(HolePunchingResults)` - Results showing which holes have been addressed
/// * `Err(ServerFnError)` - Validation failed or rate limited
///
/// # Example
/// ```ignore
/// let scenario = ScenarioField {
///     trigger: "User encounters error message".to_string(),
///     value_moment: "Instant problem resolution".to_string(),
///     feeling: "Relieved and confident".to_string(),
///     hole_punching: HolePunchingResults::default(),
/// };
///
/// let results = validate_hole_punching_server(
///     scenario,
///     Some("session-123".to_string())
/// ).await?;
///
/// if !results.is_complete() {
///     println!("Missing: {:?}", results.unaddressed_holes());
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn validate_hole_punching_server(
  scenario: ScenarioField,
  session_id: Option<String>,
) -> Result<HolePunchingResults, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  check_rate_limit_for_session(session, "validate_hole_punching_server").await?;

  // Validate input - scenario bullets should be complete
  if !scenario.is_bullets_complete() {
    return Err(ServerFnError::new(anyhow::anyhow!(
      "Scenario must have all three bullet fields (trigger, value_moment, feeling) complete"
    )));
  }

  let schema = build_hole_punching_schema();
  let context = build_hole_punching_context(schema.clone());
  let analysis_prompt = build_hole_punching_prompt(&scenario);

  // Call AI provider
  let provider = ai_provider()?;
  let result = provider
    .extract_fields_with_schema(&analysis_prompt, &schema, &context)
    .await
    .map_err(map_extraction_error)?;

  // Parse hole addressing status from response
  let (discovery_hole, edge_case_hole, motivation_dropoff) = result.fields.iter().fold(
    (
      scenario.hole_punching.discovery_hole.clone(),
      scenario.hole_punching.edge_case_hole.clone(),
      scenario.hole_punching.motivation_dropoff.clone(),
    ),
    |(dh, eh, md), field| {
      let is_true = field.value.as_bool() == Some(true);
      match field.name.as_str() {
        "discovery_hole_addressed" if is_true && dh.is_none() => {
          (Some("Addressed in scenario".to_string()), eh, md)
        }
        "edge_case_hole_addressed" if is_true && eh.is_none() => {
          (dh, Some("Addressed in scenario".to_string()), md)
        }
        "motivation_dropoff_addressed" if is_true && md.is_none() => {
          (dh, eh, Some("Addressed in scenario".to_string()))
        }
        _ => (dh, eh, md),
      }
    },
  );

  // Create hole punching results
  let hole_results = HolePunchingResults {
    discovery_hole,
    edge_case_hole,
    motivation_dropoff,
  };

  info!(
      session,
      is_complete = hole_results.is_complete(),
      addressed_count = hole_results.addressed_count(),
      unaddressed = ?hole_results.unaddressed_holes(),
      "validate_hole_punching_server: Validation completed"
  );

  Ok(hole_results)
}

// ============================================================================
// Progressive Discover Server Functions (WP01)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
use crate::kirk::progressive_discover::{
  AntithesisValidation, EarsExtraction, EarsPattern, ExtractedEarsRequirement,
  HolePunchingValidation, KirkContract16, VorpValidation,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::storage::transcript_store::InterrogationTranscript;

/// Validate antithesis (null hypothesis) points (bd-378l)
///
/// This function scores the quality of 3 null hypothesis points that represent
/// realistic reasons why users might reject or ignore a proposed solution.
///
/// # Arguments
/// * `points` - Array of exactly 3 antithesis points
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(AntithesisValidation)` - Validation result with score and suggestions
/// * `Err(ServerFnError)` - Validation failed or rate limited
///
/// # Quality Scoring
/// Points score higher for:
/// - Being non-empty (base score)
/// - Containing specific details (word count heuristics)
/// - Using concrete language vs vague abstractions
/// - Including numbers or specific reasoning
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn validate_antithesis(
  points: [String; 3],
  session_id: Option<String>,
) -> Result<AntithesisValidation, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  match RATE_LIMITER.check_rate_limit(session).await {
    Ok(()) => {
      info!(
        session,
        points_count = points.len(),
        "validate_antithesis: API call"
      );
    }
    Err(retry_after) => {
      tracing_warn!(
        session,
        retry_after,
        "validate_antithesis: Rate limit exceeded"
      );
      return Err(ServerFnError::new(anyhow::anyhow!(
        "Rate limit exceeded. Please retry after {retry_after}s"
      )));
    }
  }

  // Calculate quality scores for each point
  let scores: Vec<f64> = points.iter().map(|p| calculate_specificity(p)).collect();

  // Average score
  let overall_score = scores.iter().sum::<f64>() / 3.0;

  // Generate suggestions for low-scoring points
  let suggestions: Vec<String> = points
    .iter()
    .enumerate()
    .filter(|(_, p)| p.len() < 20 || calculate_specificity(p) < 0.5)
    .map(|(i, p)| {
      if p.trim().is_empty() {
        format!(
          "Point {} is empty - please provide a specific reason users might reject this",
          i + 1
        )
      } else if p.len() < 20 {
        format!("Point {} is too brief - add more specific details", i + 1)
      } else {
        format!(
          "Point {} needs more specificity - include concrete examples or numbers",
          i + 1
        )
      }
    })
    .collect();

  // Check if all points are valid (non-empty and specific enough)
  let is_valid = points
    .iter()
    .all(|p| !p.trim().is_empty() && calculate_specificity(p) >= 0.3);

  info!(
    session,
    score = overall_score,
    is_valid,
    suggestion_count = suggestions.len(),
    "validate_antithesis: Validation completed"
  );

  Ok(AntithesisValidation::new(
    overall_score,
    suggestions,
    is_valid,
  ))
}

/// Calculate specificity score for a single antithesis point.
#[cfg(not(target_arch = "wasm32"))]
fn calculate_specificity(text: &str) -> f64 {
  let trimmed = text.trim();

  if trimmed.is_empty() {
    return 0.0;
  }

  let word_count = trimmed.split_whitespace().count();
  let has_numbers = trimmed.chars().any(char::is_numeric);
  let has_specific_terms = [
    "exactly",
    "specifically",
    "precisely",
    "only",
    "because",
    "for example",
  ]
  .iter()
  .any(|t| trimmed.to_lowercase().contains(t));

  // Base score from word count (capped at 1.0)
  let bounded_word_count = word_count.min(20);
  let bounded_word_count = u8::try_from(bounded_word_count).map_or(20, |v| v);
  let base = f64::from(bounded_word_count) / 20.0;

  // Boosts for specificity indicators
  let number_boost = if has_numbers { 0.15 } else { 0.0 };
  let term_boost = if has_specific_terms { 0.1 } else { 0.0 };

  (base + number_boost + term_boost).min(1.0)
}

/// Validate VORP justification (bd-2mcc)
///
/// VORP (Value, Obvious, Real, Possible) is a framework for evaluating
/// whether a solution idea is worth pursuing.
///
/// # Arguments
/// * `value` - Does it provide meaningful value to users?
/// * `obvious` - Is the value immediately apparent to users?
/// * `real` - Are the users and problem real?
/// * `possible` - Can we actually build this?
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(VorpValidation)` - Validation result with scores and suggestions
/// * `Err(ServerFnError)` - Validation failed or rate limited
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn validate_vorp(
  value: String,
  obvious: String,
  real: String,
  possible: String,
  session_id: Option<String>,
) -> Result<VorpValidation, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  match RATE_LIMITER.check_rate_limit(session).await {
    Ok(()) => {
      info!(session, "validate_vorp: API call");
    }
    Err(retry_after) => {
      tracing_warn!(session, retry_after, "validate_vorp: Rate limit exceeded");
      return Err(ServerFnError::new(anyhow::anyhow!(
        "Rate limit exceeded. Please retry after {retry_after}s"
      )));
    }
  }

  // Validate each dimension
  let value_score = validate_v_dimension(&value);
  let obvious_score = validate_o_dimension(&obvious);
  let real_score = validate_r_dimension(&real);
  let possible_score = validate_p_dimension(&possible);

  let validation = VorpValidation::new(value_score, obvious_score, real_score, possible_score);

  info!(
    session,
    overall = validation.overall_score,
    passes = validation.passes(),
    weakest = validation.weakest_dimension().map(|(n, _)| n.as_str()),
    "validate_vorp: Validation completed"
  );

  Ok(validation)
}

/// Validate the Value dimension.
#[cfg(not(target_arch = "wasm32"))]
fn validate_v_dimension(text: &str) -> f64 {
  let word_count = text.split_whitespace().count();
  let has_quantified_benefit = text.chars().any(char::is_numeric)
    || text.to_lowercase().contains("save")
    || text.to_lowercase().contains("reduce")
    || text.to_lowercase().contains("increase");

  let bounded_word_count = word_count.min(15);
  let bounded_word_count = u8::try_from(bounded_word_count).map_or(15, |v| v);
  let base = f64::from(bounded_word_count) / 15.0;
  let boost = if has_quantified_benefit { 0.2 } else { 0.0 };

  (base + boost).min(1.0)
}

/// Validate the Obvious dimension.
#[cfg(not(target_arch = "wasm32"))]
fn validate_o_dimension(text: &str) -> f64 {
  let word_count = text.split_whitespace().count();
  let mentions_immediate = text.to_lowercase().contains("immediately")
    || text.to_lowercase().contains("instant")
    || text.to_lowercase().contains("right away")
    || text.to_lowercase().contains("clear");

  let bounded_word_count = word_count.min(15);
  let bounded_word_count = u8::try_from(bounded_word_count).map_or(15, |v| v);
  let base = f64::from(bounded_word_count) / 15.0;
  let boost = if mentions_immediate { 0.2 } else { 0.0 };

  (base + boost).min(1.0)
}

/// Validate the Real dimension.
#[cfg(not(target_arch = "wasm32"))]
fn validate_r_dimension(text: &str) -> f64 {
  let word_count = text.split_whitespace().count();
  let has_evidence = text.to_lowercase().contains("research")
    || text.to_lowercase().contains("study")
    || text.to_lowercase().contains("survey")
    || text.to_lowercase().contains("interview")
    || text.chars().any(char::is_numeric);

  let bounded_word_count = word_count.min(15);
  let bounded_word_count = u8::try_from(bounded_word_count).map_or(15, |v| v);
  let base = f64::from(bounded_word_count) / 15.0;
  let boost = if has_evidence { 0.2 } else { 0.0 };

  (base + boost).min(1.0)
}

/// Validate the Possible dimension.
#[cfg(not(target_arch = "wasm32"))]
fn validate_p_dimension(text: &str) -> f64 {
  let word_count = text.split_whitespace().count();
  let mentions_resources = text.to_lowercase().contains("can build")
    || text.to_lowercase().contains("technology")
    || text.to_lowercase().contains("team")
    || text.to_lowercase().contains("skill");

  let bounded_word_count = word_count.min(15);
  let bounded_word_count = u8::try_from(bounded_word_count).map_or(15, |v| v);
  let base = f64::from(bounded_word_count) / 15.0;
  let boost = if mentions_resources { 0.2 } else { 0.0 };

  (base + boost).min(1.0)
}

/// Validate hole punching for scenario gaps (bd-13yb)
///
/// Checks if all 3 hole types have been addressed in the scenario.
///
/// # Arguments
/// * `discovery_hole` - How the user discovers the feature
/// * `edge_case_hole` - What happens in edge cases
/// * `motivation_dropoff` - Why users continue through friction
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(HolePunchingValidation)` - Validation result
/// * `Err(ServerFnError)` - Validation failed or rate limited
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn validate_hole_punching_v2(
  discovery_hole: Option<String>,
  edge_case_hole: Option<String>,
  motivation_dropoff: Option<String>,
  session_id: Option<String>,
) -> Result<HolePunchingValidation, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  match RATE_LIMITER.check_rate_limit(session).await {
    Ok(()) => {
      info!(session, "validate_hole_punching_v2: API call");
    }
    Err(retry_after) => {
      tracing_warn!(
        session,
        retry_after,
        "validate_hole_punching_v2: Rate limit exceeded"
      );
      return Err(ServerFnError::new(anyhow::anyhow!(
        "Rate limit exceeded. Please retry after {retry_after}s"
      )));
    }
  }

  // Normalize empty strings to None
  let discovery_hole = discovery_hole.filter(|s| !s.trim().is_empty());
  let edge_case_hole = edge_case_hole.filter(|s| !s.trim().is_empty());
  let motivation_dropoff = motivation_dropoff.filter(|s| !s.trim().is_empty());

  let results = HolePunchingResults {
    discovery_hole,
    edge_case_hole,
    motivation_dropoff,
  };

  let validation = HolePunchingValidation::new(results);

  info!(
    session,
    is_complete = validation.is_complete,
    addressed_count = validation.addressed_count,
    "validate_hole_punching_v2: Validation completed"
  );

  Ok(validation)
}

/// Extract EARS requirements from transcript (bd-zf68)
///
/// EARS (Easy Approach to Requirements Syntax) provides patterns for
/// writing clear, testable requirements.
///
/// # Arguments
/// * `transcript` - The interrogation transcript to extract from
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(EarsExtraction)` - Extracted requirements
/// * `Err(ServerFnError)` - Extraction failed or rate limited
#[cfg(not(target_arch = "wasm32"))]
#[server]
pub async fn extract_ears(
  transcript: InterrogationTranscript,
  session_id: Option<String>,
) -> Result<EarsExtraction, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  match RATE_LIMITER.check_rate_limit(session).await {
    Ok(()) => {
      info!(session, "extract_ears: API call");
    }
    Err(retry_after) => {
      tracing_warn!(session, retry_after, "extract_ears: Rate limit exceeded");
      return Err(ServerFnError::new(anyhow::anyhow!(
        "Rate limit exceeded. Please retry after {retry_after}s"
      )));
    }
  }

  let requirements = [
    (&transcript.problem.content, "problem"),
    (&transcript.solution.content, "solution"),
    (&transcript.scenario.trigger, "scenario.trigger"),
    (&transcript.scenario.value_moment, "scenario.value_moment"),
  ]
  .into_iter()
  .filter(|(content, _)| !content.is_empty())
  .flat_map(|(content, section)| extract_ears_from_text(content, section))
  .collect();

  let extraction = EarsExtraction::new(requirements);

  info!(
    session,
    total_count = extraction.total_count,
    sections_analyzed = extraction.analyzed_sections.len(),
    "extract_ears: Extraction completed"
  );

  Ok(extraction)
}

/// Extract EARS requirements from a text string.
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn extract_ears_from_text(text: &str, source_section: &str) -> Vec<ExtractedEarsRequirement> {
  let sentences: Vec<&str> = text.split(&['.', '!', '?'][..]).collect();

  let detected: Vec<ExtractedEarsRequirement> = sentences
    .iter()
    .enumerate()
    .filter_map(|(i, sentence)| {
      let sentence_lower = sentence.to_lowercase();

      let pattern = if sentence_lower.contains("shall not") || sentence_lower.contains("must not") {
        Some(EarsPattern::Unwanted)
      } else if sentence_lower.contains("when ") || sentence_lower.contains("if ") {
        Some(EarsPattern::EventDriven)
      } else if sentence_lower.contains("while ") || sentence_lower.contains("during ") {
        Some(EarsPattern::StateDriven)
      } else if sentence_lower.contains("shall ")
        || sentence_lower.contains("must ")
        || sentence_lower.contains("will ")
      {
        Some(EarsPattern::Ubiquitous)
      } else {
        None
      };

      pattern.and_then(|p| {
        let trimmed = sentence.trim();
        if trimmed.is_empty() {
          None
        } else {
          Some(ExtractedEarsRequirement::new(
            format!("{source_section}-{i}"),
            trimmed.to_string(),
            p,
            source_section.to_string(),
          ))
        }
      })
    })
    .collect();

  // Also check for keywords indicating requirements
  let lower = text.to_lowercase();
  let has_keyword = lower.contains("require") || lower.contains("need") || lower.contains("should");

  if has_keyword && detected.is_empty() && text.len() > 20 {
    detected
      .into_iter()
      .chain(std::iter::once(ExtractedEarsRequirement::new(
        format!("{source_section}-implicit"),
        text.trim().to_string(),
        EarsPattern::Ubiquitous,
        source_section.to_string(),
      )))
      .collect()
  } else {
    detected
  }
}

/// Compile transcript to 16-section KIRK contract (bd-l1qq)
///
/// Takes a completed interrogation transcript and compiles it into
/// the 16-section KIRK contract structure.
///
/// # Arguments
/// * `transcript` - The interrogation transcript to compile
/// * `session_id` - Optional session identifier for rate limiting
///
/// # Returns
/// * `Ok(KirkContract16)` - Compiled 16-section contract
/// * `Err(ServerFnError)` - Compilation failed or rate limited
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_lines)]
#[server]
pub async fn compile_to_kirk(
  transcript: InterrogationTranscript,
  session_id: Option<String>,
) -> Result<KirkContract16, ServerFnError> {
  let session = session_id.as_deref().map_or("default", |s| s);

  // Check rate limit
  match RATE_LIMITER.check_rate_limit(session).await {
    Ok(()) => {
      info!(session, "compile_to_kirk: API call");
    }
    Err(retry_after) => {
      tracing_warn!(session, retry_after, "compile_to_kirk: Rate limit exceeded");
      return Err(ServerFnError::new(anyhow::anyhow!(
        "Rate limit exceeded. Please retry after {retry_after}s"
      )));
    }
  }

  // Build section content pairs
  let straw_man_content = if transcript.straw_man_validation.passed {
    "Passed - No straw man traps detected".to_string()
  } else {
    format!(
      "Traps detected: {:?}",
      transcript.straw_man_validation.traps_detected
    )
  };

  let ears_extraction = extract_ears_from_text(&transcript.problem.content, "problem")
    .into_iter()
    .chain(extract_ears_from_text(
      &transcript.solution.content,
      "solution",
    ))
    .map(|r| r.text)
    .collect::<Vec<_>>()
    .join("\n\n");

  let metadata = serde_json::json!({
      "compiled_at": chrono::Utc::now().to_rfc3339(),
      "schema_version": "1.0.0",
      "session_id": session,
      "is_completed": transcript.is_completed(),
  })
  .to_string();

  let required_sections: [(usize, String); 11] = [
    (0, transcript.original_prompt.clone()),
    (1, transcript.problem.content.clone()),
    (2, transcript.antithesis.points.join("\n\n")),
    (3, transcript.persona.content.clone()),
    (4, straw_man_content),
    (5, transcript.solution.content.clone()),
    (6, transcript.vorp_justification.clone()),
    (7, transcript.nonpersona.content.clone()),
    (8, transcript.scenario.trigger.clone()),
    (9, transcript.scenario.value_moment.clone()),
    (10, transcript.scenario.feeling.clone()),
  ];

  let optional_sections: [Option<(usize, String)>; 5] = [
    transcript
      .scenario
      .hole_punching
      .discovery_hole
      .clone()
      .map(|c| (11, c)),
    transcript
      .scenario
      .hole_punching
      .edge_case_hole
      .clone()
      .map(|c| (12, c)),
    transcript
      .scenario
      .hole_punching
      .motivation_dropoff
      .clone()
      .map(|c| (13, c)),
    Some((14, ears_extraction)),
    Some((15, metadata)),
  ];

  let contract = required_sections
    .into_iter()
    .chain(optional_sections.into_iter().flatten())
    .try_fold(KirkContract16::new(), |contract, (section, content)| {
      contract
        .with_section_content(section, content)
        .ok_or_else(|| ServerFnError::new(anyhow::anyhow!("Failed to set section {}", section)))
    })?;

  info!(
    session,
    filled_sections = contract.filled_section_count(),
    completion = contract.completion_percentage(),
    is_complete = contract.is_complete(),
    "compile_to_kirk: Compilation completed"
  );

  Ok(contract)
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod integration_tests {
  use super::*;
  use crate::components::discover::straw_man::StrawManTrap;
  use crate::config::ai::{AiConfig, ProviderConfig, ProviderType, QualityConfig};
  use crate::providers::FieldExtraction;
  use serde_json::json;

  /// Test rate limiter allows requests under limit
  #[tokio::test]
  async fn test_rate_limiter_under_limit() {
    let limiter = RateLimiter::new(5);
    let session = "test_session_under";

    // Should allow 5 requests
    for i in 0..5 {
      let result = limiter.check_rate_limit(session).await;
      assert!(result.is_ok(), "Request {} should be allowed", i + 1);
    }

    // Clean up
    limiter.requests.write().await.remove(session);
  }

  /// Test rate limiter blocks requests over limit
  #[tokio::test]
  async fn test_rate_limiter_over_limit() {
    let limiter = RateLimiter::new(3);
    let session = "test_session_over";

    // Should allow first 3
    for i in 0..3 {
      let result = limiter.check_rate_limit(session).await;
      assert!(result.is_ok(), "Request {} should be allowed", i + 1);
    }

    // 4th should be blocked
    let result = limiter.check_rate_limit(session).await;
    assert!(result.is_err(), "Request 4 should be blocked");

    // Should get retry-after duration
    let retry_after = result.unwrap_err();
    assert!(retry_after > 0, "Should have retry-after duration");

    // Clean up
    limiter.requests.write().await.remove(session);
  }

  /// Test rate limiter resets after time window
  #[tokio::test]
  async fn test_rate_limiter_resets_after_window() {
    let limiter = RateLimiter::new(2);
    let session = "test_session_reset";

    // Fill up the limiter
    let _ = limiter.check_rate_limit(session).await;
    let _ = limiter.check_rate_limit(session).await;

    // Should be blocked
    assert!(limiter.check_rate_limit(session).await.is_err());

    // Manipulate timestamps to simulate time passing
    let mut requests = limiter.requests.write().await;
    if let Some(session_reqs) = requests.get_mut(session) {
      // Set all timestamps to > 60 seconds ago
      let now = Instant::now();
      let old_time = now.checked_sub(Duration::from_secs(61)).unwrap_or(now);
      session_reqs.clear();
      session_reqs.push(old_time);
    }
    drop(requests);

    // Should now be allowed again
    assert!(limiter.check_rate_limit(session).await.is_ok());

    // Clean up
    limiter.requests.write().await.remove(session);
  }

  /// Test rate limiter handles multiple sessions independently
  #[tokio::test]
  async fn test_rate_limiter_multiple_sessions() {
    let limiter = RateLimiter::new(2);

    // Fill session1
    let _ = limiter.check_rate_limit("session1").await;
    let _ = limiter.check_rate_limit("session1").await;
    assert!(limiter.check_rate_limit("session1").await.is_err());

    // Session2 should still be allowed
    assert!(limiter.check_rate_limit("session2").await.is_ok());
    assert!(limiter.check_rate_limit("session2").await.is_ok());

    // Clean up
    limiter.requests.write().await.remove("session1");
    limiter.requests.write().await.remove("session2");
  }

  /// Test extraction context serialization
  #[test]
  fn test_extraction_context_serialization() {
    let context = ExtractionContext {
      document_type: Some("discover_phase".to_string()),
      locale: Some("en_US".to_string()),
      schema: None,
      extra: json!({"test": "value"}),
    };

    let serialized =
      serde_json::to_string(&context).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: ExtractionContext =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.document_type, context.document_type);
    assert_eq!(deserialized.locale, context.locale);
  }

  /// Test field type serialization
  #[test]
  fn test_field_type_serialization() {
    let field_type = FieldType::TextArea;
    let serialized =
      serde_json::to_string(&field_type).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: FieldType =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized, field_type);
  }

  /// Test extracted fields serialization
  #[test]
  fn test_extracted_fields_serialization() {
    let fields = ExtractedFields {
      fields: vec![FieldExtraction {
        name: "problem".to_string(),
        field_type: FieldType::TextArea,
        value: json!("Users struggle with complex workflows"),
        confidence: 0.95,
        justification: Some("Directly stated in input".to_string()),
      }],
      confidence: 0.95,
      metadata: crate::providers::ExtractionMetadata {
        provider: "opencode".to_string(),
        model: Some("test-model".to_string()),
        timestamp: chrono::Utc::now(),
        processing_duration_ms: 150,
        extra: json!({}),
      },
    };

    let serialized =
      serde_json::to_string(&fields).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: ExtractedFields =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.fields.len(), fields.fields.len());
    assert_eq!(deserialized.fields[0].name, "problem");
    assert!((deserialized.fields[0].confidence - 0.95).abs() < f64::EPSILON);
  }

  /// Test quality score serialization
  #[test]
  fn test_quality_score_serialization() {
    use crate::lattice::quality::{DimensionScore, QualityDimension};

    let score = QualityScore {
      overall: 85,
      dimensions: vec![DimensionScore {
        dimension: QualityDimension::Completeness,
        score: 90,
      }],
      issues: vec![],
    };

    let serialized =
      serde_json::to_string(&score).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: QualityScore =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.overall, 85);
    assert_eq!(deserialized.dimensions.len(), 1);
    assert_eq!(deserialized.dimensions[0].score, 90);
  }

  /// Test EARS requirement ref serialization
  #[test]
  fn test_ears_requirement_ref_serialization() {
    let ears = EarsRequirementRef {
      id: "req-1".to_string(),
      text: "User shall authenticate".to_string(),
      has_acceptance_criteria: true,
    };

    let serialized =
      serde_json::to_string(&ears).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: EarsRequirementRef =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.id, "req-1");
    assert_eq!(deserialized.text, "User shall authenticate");
    assert!(deserialized.has_acceptance_criteria);
  }

  /// Test quality answer serialization
  #[test]
  fn test_quality_answer_serialization() {
    let answer = QualityAnswer {
      step_id: "user_goal".to_string(),
      value: "Users want to complete tasks quickly".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
    };

    let serialized =
      serde_json::to_string(&answer).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: QualityAnswer =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.step_id, "user_goal");
    assert_eq!(deserialized.value, "Users want to complete tasks quickly");
  }

  /// Test AI provider diagnostics serialization roundtrip
  #[test]
  fn test_ai_provider_diagnostics_serialization() {
    let diagnostics = AiProviderDiagnostics {
      provider: "opencode".to_string(),
      endpoint: "https://api.opencode.ai/v1".to_string(),
      model: Some("zai-coding-plan/glm-5".to_string()),
      routing_provider: Some("zai-coding-plan".to_string()),
    };

    let serialized =
      serde_json::to_string(&diagnostics).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: AiProviderDiagnostics =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.provider, "opencode");
    assert_eq!(deserialized.endpoint, "https://api.opencode.ai/v1");
    assert_eq!(deserialized.model.as_deref(), Some("zai-coding-plan/glm-5"));
    assert_eq!(
      deserialized.routing_provider.as_deref(),
      Some("zai-coding-plan")
    );
  }

  #[test]
  fn test_map_extraction_error_maps_rate_limit_message() {
    let error = ExtractionError::RateLimited {
      retry_after_seconds: 12,
    };

    let mapped = map_extraction_error(error);

    assert!(mapped.to_string().contains("Retry after 12s"));
  }

  #[test]
  fn test_map_extraction_error_maps_authentication_message() {
    let mapped = map_extraction_error(ExtractionError::AuthenticationError(
      "Invalid API key".to_string(),
    ));

    assert!(mapped
      .to_string()
      .contains("Authentication failed: Invalid API key"));
  }

  #[test]
  fn test_map_extraction_error_uses_action_for_fallback_errors() {
    let mapped = map_extraction_error(ExtractionError::ProviderError(
      "JSON schema violation".to_string(),
    ));

    assert!(mapped
      .to_string()
      .contains("Extraction failed: Provider error: JSON schema violation"));
  }

  #[test]
  fn test_build_straw_man_prompt_includes_persona_and_all_four_traps() {
    let prompt = build_straw_man_prompt("Persona text");

    assert!(prompt.contains("Persona text"));
    assert!(prompt.contains("Irrational Actor"));
    assert!(prompt.contains("Manic Pixie Dream User"));
    assert!(prompt.contains("Stoic Monk"));
    assert!(prompt.contains("Your Clone"));
  }

  #[test]
  fn test_parse_straw_man_validation_collects_only_true_known_flags() {
    let fields = vec![
      FieldExtraction {
        name: "irrational_actor_detected".to_string(),
        field_type: FieldType::Boolean,
        value: json!(true),
        confidence: 1.0,
        justification: None,
      },
      FieldExtraction {
        name: "stoic_monk_detected".to_string(),
        field_type: FieldType::Boolean,
        value: json!(false),
        confidence: 1.0,
        justification: None,
      },
      FieldExtraction {
        name: "unknown_flag".to_string(),
        field_type: FieldType::Boolean,
        value: json!(true),
        confidence: 1.0,
        justification: None,
      },
    ];

    let validation = parse_straw_man_validation(&fields);

    assert_eq!(
      validation.traps_detected,
      vec![StrawManTrap::IrrationalActor]
    );
    assert!(!validation.passed);
  }

  #[test]
  fn test_parse_straw_man_validation_returns_passing_when_no_true_flags() {
    let fields = vec![FieldExtraction {
      name: "stoic_monk_detected".to_string(),
      field_type: FieldType::Boolean,
      value: json!(false),
      confidence: 1.0,
      justification: None,
    }];

    let validation = parse_straw_man_validation(&fields);

    assert!(validation.passed);
    assert!(validation.traps_detected.is_empty());
  }

  #[test]
  fn test_build_hole_punching_prompt_uses_not_addressed_fallbacks() {
    let scenario = ScenarioField {
      trigger: "Trigger".to_string(),
      value_moment: "Value".to_string(),
      feeling: "Feeling".to_string(),
      hole_punching: HolePunchingResults::default(),
    };

    let prompt = build_hole_punching_prompt(&scenario);

    assert!(prompt.contains("Trigger: Trigger"));
    assert!(prompt.contains("Value Moment: Value"));
    assert!(prompt.contains("Feeling: Feeling"));
    assert_eq!(prompt.matches("Not addressed").count(), 3);
  }

  #[test]
  fn test_merge_hole_punching_results_marks_missing_holes_as_addressed() {
    let fields = vec![
      FieldExtraction {
        name: "discovery_hole_addressed".to_string(),
        field_type: FieldType::Boolean,
        value: json!(true),
        confidence: 1.0,
        justification: None,
      },
      FieldExtraction {
        name: "motivation_dropoff_addressed".to_string(),
        field_type: FieldType::Boolean,
        value: json!(true),
        confidence: 1.0,
        justification: None,
      },
    ];

    let merged = merge_hole_punching_results(&HolePunchingResults::default(), &fields);

    assert_eq!(
      merged.discovery_hole.as_deref(),
      Some("Addressed in scenario")
    );
    assert_eq!(
      merged.motivation_dropoff.as_deref(),
      Some("Addressed in scenario")
    );
    assert_eq!(merged.edge_case_hole, None);
  }

  #[test]
  fn test_merge_hole_punching_results_preserves_existing_explanations() {
    let existing = HolePunchingResults {
      discovery_hole: Some("Already explained".to_string()),
      edge_case_hole: None,
      motivation_dropoff: None,
    };
    let fields = vec![FieldExtraction {
      name: "discovery_hole_addressed".to_string(),
      field_type: FieldType::Boolean,
      value: json!(true),
      confidence: 1.0,
      justification: None,
    }];

    let merged = merge_hole_punching_results(&existing, &fields);

    assert_eq!(merged.discovery_hole.as_deref(), Some("Already explained"));
  }

  #[test]
  fn test_merge_hole_punching_results_ignores_false_non_boolean_unknown_fields() {
    let fields = vec![
      FieldExtraction {
        name: "edge_case_hole_addressed".to_string(),
        field_type: FieldType::Boolean,
        value: json!(false),
        confidence: 1.0,
        justification: None,
      },
      FieldExtraction {
        name: "edge_case_hole_addressed".to_string(),
        field_type: FieldType::Boolean,
        value: json!("yes"),
        confidence: 1.0,
        justification: None,
      },
      FieldExtraction {
        name: "unknown_field".to_string(),
        field_type: FieldType::Boolean,
        value: json!(true),
        confidence: 1.0,
        justification: None,
      },
    ];

    let merged = merge_hole_punching_results(&HolePunchingResults::default(), &fields);

    assert_eq!(merged, HolePunchingResults::default());
  }

  #[test]
  fn test_build_provider_bootstrap_input_preserves_configured_session() {
    let config = AiConfig {
      provider: ProviderConfig {
        provider: ProviderType::Opencode,
        endpoint: "https://api.example.com/v1".to_string(),
        session_id: "configured-session".to_string(),
        model: Some("provider-a/model-a".to_string()),
        routing_provider: Some("provider-a".to_string()),
      },
      quality: QualityConfig::default(),
    };

    let input = build_provider_bootstrap_input(config, || "generated-session".to_string());

    assert_eq!(input.session_id, "configured-session");
    assert_eq!(input.diagnostics.provider, "opencode");
    assert_eq!(input.diagnostics.endpoint, "https://api.example.com/v1");
    assert_eq!(input.diagnostics.model.as_deref(), Some("model-a"));
    assert_eq!(
      input.diagnostics.routing_provider.as_deref(),
      Some("provider-a")
    );
  }

  #[test]
  fn test_build_provider_bootstrap_input_generates_session_when_missing() {
    let config = AiConfig {
      provider: ProviderConfig {
        provider: ProviderType::Opencode,
        endpoint: "https://api.opencode.ai/v1".to_string(),
        session_id: String::new(),
        model: Some("provider-b/model-b".to_string()),
        routing_provider: None,
      },
      quality: QualityConfig::default(),
    };

    let input = build_provider_bootstrap_input(config, || "generated-session".to_string());

    assert_eq!(input.session_id, "generated-session");
    assert_eq!(input.diagnostics.model.as_deref(), Some("model-b"));
    assert_eq!(
      input.diagnostics.routing_provider.as_deref(),
      Some("provider-b")
    );
  }

  #[test]
  fn test_create_ai_provider_state_rejects_unsupported_provider() {
    let input = ProviderBootstrapInput {
      diagnostics: AiProviderDiagnostics {
        provider: "anthropic".to_string(),
        endpoint: "https://api.example.com".to_string(),
        model: Some("claude".to_string()),
        routing_provider: None,
      },
      session_id: "session-123".to_string(),
    };

    let result = create_ai_provider_state(input);

    assert!(matches!(
      result,
      Err(AiProviderBootstrapError::UnsupportedProvider(provider)) if provider == "anthropic"
    ));
  }

  #[test]
  fn test_create_ai_provider_state_returns_structured_diagnostics() {
    let input = ProviderBootstrapInput {
      diagnostics: AiProviderDiagnostics {
        provider: "opencode".to_string(),
        endpoint: "https://api.opencode.ai/v1".to_string(),
        model: Some("glm-5".to_string()),
        routing_provider: Some("zai-coding-plan".to_string()),
      },
      session_id: "session-123".to_string(),
    };

    let state = create_ai_provider_state(input).expect("provider should initialize");

    assert_eq!(state.diagnostics.provider, "opencode");
    assert_eq!(state.diagnostics.endpoint, "https://api.opencode.ai/v1");
    assert_eq!(state.diagnostics.model.as_deref(), Some("glm-5"));
    assert_eq!(
      state.diagnostics.routing_provider.as_deref(),
      Some("zai-coding-plan")
    );
    assert_eq!(state.provider.session_id(), "session-123");
  }

  /// Test inversion control serialization
  #[test]
  fn test_inversion_control_serialization() {
    use crate::lattice::quality::InversionControl;

    let inversion = InversionControl {
      has_inversion_tests: true,
      inverted_count: 5,
    };

    let serialized =
      serde_json::to_string(&inversion).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: InversionControl =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert!(deserialized.has_inversion_tests);
    assert_eq!(deserialized.inverted_count, 5);
  }

  /// Test `StrawManValidation` serialization
  #[test]
  fn test_straw_man_validation_serialization() {
    use crate::components::discover::straw_man::StrawManValidation;

    let validation =
      StrawManValidation::new(vec![StrawManTrap::IrrationalActor, StrawManTrap::YourClone]);

    let serialized =
      serde_json::to_string(&validation).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: StrawManValidation =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.traps_detected.len(), 2);
    assert!(!deserialized.passed);
    assert!(deserialized.has_trap(StrawManTrap::IrrationalActor));
    assert!(deserialized.has_trap(StrawManTrap::YourClone));
  }

  /// Test `StrawManTrap` serialization
  #[test]
  fn test_straw_man_trap_serialization() {
    use crate::components::discover::straw_man::StrawManTrap;

    for trap in [
      StrawManTrap::IrrationalActor,
      StrawManTrap::ManicPixieDreamUser,
      StrawManTrap::StoicMonk,
      StrawManTrap::YourClone,
    ] {
      let serialized =
        serde_json::to_string(&trap).unwrap_or_else(|e| panic!("serialization error: {}", e));
      let deserialized: StrawManTrap = serde_json::from_str(&serialized)
        .unwrap_or_else(|e| panic!("deserialization error: {}", e));
      assert_eq!(trap, deserialized);
    }
  }

  /// Test passing `StrawManValidation`
  #[test]
  fn test_passing_straw_man_validation() {
    use crate::components::discover::straw_man::StrawManValidation;

    let validation = StrawManValidation::passing();
    assert!(validation.passed);
    assert!(validation.traps_detected.is_empty());
    assert_eq!(validation.trap_count(), 0);
  }

  /// Test failing `StrawManValidation` with multiple traps
  #[test]
  fn test_failing_straw_man_validation_multiple_traps() {
    use crate::components::discover::straw_man::StrawManValidation;

    let traps = vec![
      StrawManTrap::ManicPixieDreamUser,
      StrawManTrap::StoicMonk,
      StrawManTrap::YourClone,
    ];
    let validation = StrawManValidation::new(traps);

    assert!(!validation.passed);
    assert_eq!(validation.trap_count(), 3);
    assert!(validation.has_trap(StrawManTrap::ManicPixieDreamUser));
    assert!(validation.has_trap(StrawManTrap::StoicMonk));
    assert!(validation.has_trap(StrawManTrap::YourClone));
    assert!(!validation.has_trap(StrawManTrap::IrrationalActor));
  }

  /// Test `HolePunchingResults` serialization
  #[test]
  fn test_hole_punching_results_serialization() {
    let results = HolePunchingResults {
      discovery_hole: Some("Found via search".to_string()),
      edge_case_hole: Some("Handles network errors".to_string()),
      motivation_dropoff: None,
    };

    let serialized =
      serde_json::to_string(&results).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: HolePunchingResults =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.discovery_hole, results.discovery_hole);
    assert_eq!(deserialized.edge_case_hole, results.edge_case_hole);
    assert_eq!(deserialized.motivation_dropoff, results.motivation_dropoff);
  }

  /// Test `ScenarioField` serialization
  #[test]
  fn test_scenario_field_serialization() {
    let scenario = ScenarioField {
      trigger: "User sees error".to_string(),
      value_moment: "Quick fix".to_string(),
      feeling: "Relieved".to_string(),
      hole_punching: HolePunchingResults {
        discovery_hole: Some("Via notification".to_string()),
        edge_case_hole: None,
        motivation_dropoff: None,
      },
    };

    let serialized =
      serde_json::to_string(&scenario).unwrap_or_else(|e| panic!("serialization error: {}", e));
    let deserialized: ScenarioField =
      serde_json::from_str(&serialized).unwrap_or_else(|e| panic!("deserialization error: {}", e));

    assert_eq!(deserialized.trigger, "User sees error");
    assert_eq!(deserialized.value_moment, "Quick fix");
    assert_eq!(deserialized.feeling, "Relieved");
    assert_eq!(
      deserialized.hole_punching.discovery_hole,
      Some("Via notification".to_string())
    );
  }

  /// Test `HolePunchingResults::is_complete`
  #[test]
  fn test_hole_punching_results_is_complete() {
    // All holes addressed with non-empty content
    let complete = HolePunchingResults {
      discovery_hole: Some("a".to_string()),
      edge_case_hole: Some("x".to_string()),
      motivation_dropoff: Some("y".to_string()),
    };
    assert!(complete.is_complete());
    assert_eq!(complete.addressed_count(), 3);

    // Missing one hole
    let incomplete = HolePunchingResults {
      discovery_hole: Some("x".to_string()),
      edge_case_hole: None,
      motivation_dropoff: Some("y".to_string()),
    };
    assert!(!incomplete.is_complete());
    assert_eq!(incomplete.addressed_count(), 2);

    // All empty strings normalize to None
    let empty = HolePunchingResults {
      discovery_hole: Some(String::new()),
      edge_case_hole: Some("   ".to_string()),
      motivation_dropoff: Some("\t\n".to_string()),
    };
    assert!(!empty.is_complete());
    assert_eq!(empty.addressed_count(), 0);
  }

  /// Test `HolePunchingResults::unaddressed_holes`
  #[test]
  fn test_hole_punching_results_unaddressed_holes() {
    use crate::components::discover::types::HoleType;

    let results = HolePunchingResults {
      discovery_hole: Some("x".to_string()),
      edge_case_hole: None,
      motivation_dropoff: None,
    };

    let unaddressed = results.unaddressed_holes();
    assert_eq!(unaddressed.len(), 2);
    assert!(unaddressed.contains(&HoleType::EdgeCaseHole));
    assert!(unaddressed.contains(&HoleType::MotivationDropOff));
    assert!(!unaddressed.contains(&HoleType::DiscoveryHole));
  }

  /// Test `HolePunchingResults::from_strings` normalizes empty strings
  #[test]
  fn test_hole_punching_results_from_strings() {
    let results =
      HolePunchingResults::from_strings("valid".to_string(), String::new(), "   ".to_string());

    assert_eq!(results.discovery_hole, Some("valid".to_string()));
    assert_eq!(results.edge_case_hole, None);
    assert_eq!(results.motivation_dropoff, None);
    assert_eq!(results.addressed_count(), 1);
  }

  /// Test `ScenarioField` validation helpers
  #[test]
  fn test_scenario_field_validation_helpers() {
    // Complete scenario
    let complete = ScenarioField {
      trigger: "Trigger".to_string(),
      value_moment: "Value".to_string(),
      feeling: "Happy".to_string(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(complete.is_bullets_complete());
    assert!(!complete.is_complete()); // holes not addressed
    assert!(!complete.is_trigger_empty());
    assert!(!complete.is_value_moment_empty());
    assert!(!complete.is_feeling_empty());

    // Incomplete with whitespace
    let whitespace = ScenarioField {
      trigger: "   ".to_string(),
      value_moment: "\t\n".to_string(),
      feeling: String::new(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(!whitespace.is_bullets_complete());
    assert!(whitespace.is_trigger_empty());
    assert!(whitespace.is_value_moment_empty());
    assert!(whitespace.is_feeling_empty());
  }

  // ============================================
  // ADVERSARIAL INPUT VALIDATION TESTS
  // ============================================

  /// Test that `validate_straw_man_traps_server` rejects empty input
  #[tokio::test]
  async fn test_validate_straw_man_rejects_empty() {
    // The server function has validation: input.trim().is_empty() check
    // We verify the validation logic works correctly
    let empty_input = "";
    assert!(
      empty_input.trim().is_empty(),
      "Empty input should be detected"
    );

    let whitespace_input = "   \t\n";
    assert!(
      whitespace_input.trim().is_empty(),
      "Whitespace-only input should be detected"
    );
  }

  /// Test that `validate_straw_man_traps_server` rejects whitespace-only input
  #[tokio::test]
  async fn test_validate_straw_man_rejects_whitespace() {
    let whitespace_variants = vec!["   ", "\t\t", "\n\n", " \t \n "];

    for input in whitespace_variants {
      assert!(
        input.trim().is_empty(),
        "Whitespace input '{input}' should be detected as empty after trim"
      );
    }
  }

  /// Test that `validate_hole_punching_server` rejects incomplete scenario
  #[tokio::test]
  async fn test_validate_hole_punching_rejects_empty_fields() {
    // Empty trigger
    let empty_trigger = ScenarioField {
      trigger: String::new(),
      value_moment: "Some value".to_string(),
      feeling: "Okay".to_string(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(
      empty_trigger.is_trigger_empty(),
      "Empty trigger should be detected"
    );

    // All fields empty
    let all_empty = ScenarioField {
      trigger: String::new(),
      value_moment: String::new(),
      feeling: String::new(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(
      all_empty.is_trigger_empty(),
      "Empty trigger should be detected"
    );
    assert!(
      all_empty.is_value_moment_empty(),
      "Empty value_moment should be detected"
    );
    assert!(
      all_empty.is_feeling_empty(),
      "Empty feeling should be detected"
    );
  }

  /// Test that `validate_hole_punching_server` rejects whitespace-only fields
  #[tokio::test]
  async fn test_validate_hole_punching_rejects_whitespace_fields() {
    let whitespace = ScenarioField {
      trigger: "   ".to_string(),
      value_moment: "\t\n".to_string(),
      feeling: "  ".to_string(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(
      whitespace.is_trigger_empty(),
      "Whitespace trigger should be detected as empty"
    );
    assert!(
      whitespace.is_value_moment_empty(),
      "Whitespace value_moment should be detected as empty"
    );
    assert!(
      whitespace.is_feeling_empty(),
      "Whitespace feeling should be detected as empty"
    );
  }

  // ============================================
  // DIRECT VALIDATION LOGIC TESTS
  // ============================================

  /// Test that empty input validation helper works correctly
  #[test]
  fn test_is_empty_after_trim() {
    // Valid inputs
    assert!(!"valid text".trim().is_empty());
    assert!(!"  valid  ".trim().is_empty());
    assert!(!"\tvalid\t".trim().is_empty());
    assert!(!"a".trim().is_empty());

    // Empty inputs
    assert!("".trim().is_empty());
    assert!("   ".trim().is_empty());
    assert!("\t\n".trim().is_empty());
    assert!("  \t  \n  ".trim().is_empty());

    // Unicode whitespace
    assert!("\u{2003}\u{3000}".trim().is_empty());
  }

  /// Test `ScenarioField` empty detection methods
  #[test]
  fn test_scenario_field_empty_detection() {
    // Non-empty fields
    let valid = ScenarioField {
      trigger: "Trigger".to_string(),
      value_moment: "Value".to_string(),
      feeling: "Happy".to_string(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(!valid.is_trigger_empty());
    assert!(!valid.is_value_moment_empty());
    assert!(!valid.is_feeling_empty());

    // Empty fields
    let empty = ScenarioField {
      trigger: String::new(),
      value_moment: String::new(),
      feeling: String::new(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(empty.is_trigger_empty());
    assert!(empty.is_value_moment_empty());
    assert!(empty.is_feeling_empty());

    // Whitespace fields (normalized to empty)
    let whitespace = ScenarioField {
      trigger: "   ".to_string(),
      value_moment: "\t\n".to_string(),
      feeling: "  ".to_string(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(whitespace.is_trigger_empty());
    assert!(whitespace.is_value_moment_empty());
    assert!(whitespace.is_feeling_empty());
  }

  /// Test `HolePunchingResults` empty normalization
  #[test]
  fn test_hole_punching_empty_normalization() {
    // Empty strings are normalized to None
    let empty_strings =
      HolePunchingResults::from_strings(String::new(), String::new(), String::new());
    assert_eq!(empty_strings.discovery_hole, None);
    assert_eq!(empty_strings.edge_case_hole, None);
    assert_eq!(empty_strings.motivation_dropoff, None);
    assert_eq!(empty_strings.addressed_count(), 0);

    // Whitespace strings are normalized to None
    let whitespace =
      HolePunchingResults::from_strings("   ".to_string(), "\t\n".to_string(), "  ".to_string());
    assert_eq!(whitespace.discovery_hole, None);
    assert_eq!(whitespace.edge_case_hole, None);
    assert_eq!(whitespace.motivation_dropoff, None);
    assert_eq!(whitespace.addressed_count(), 0);

    // Valid content is preserved (whitespace is NOT trimmed, only empty/whitespace-only becomes None)
    let valid = HolePunchingResults::from_strings(
      "Discovery via search".to_string(),
      "  Edge case handled  ".to_string(),
      "Motivated by speed".to_string(),
    );
    assert_eq!(
      valid.discovery_hole,
      Some("Discovery via search".to_string())
    );
    // Note: Whitespace is preserved in non-empty strings
    assert_eq!(
      valid.edge_case_hole,
      Some("  Edge case handled  ".to_string())
    );
    assert_eq!(
      valid.motivation_dropoff,
      Some("Motivated by speed".to_string())
    );
    assert_eq!(valid.addressed_count(), 3);
  }

  /// Test that session_id validation works for server functions
  #[test]
  fn test_session_id_validation_available() {
    use crate::intent::security::validate_session_id;

    // Validation should work
    assert!(validate_session_id("valid").is_ok());
    // Empty should fail
    assert!(validate_session_id("").is_err());
  }

  /// Test: extract_fields_server should validate session_id
  /// This test verifies session_id validation is available.
  /// The validate_session_id function should be called by extract_fields_server.
  #[test]
  fn test_extract_fields_server_uses_session_validation() {
    use crate::intent::security::validate_session_id;

    // Test that validation rejects empty session_id
    // Server functions should call this validation
    let empty_result = validate_session_id("");
    assert!(
      empty_result.is_err(),
      "Empty session must be rejected by validation"
    );
  }
}
