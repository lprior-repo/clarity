//! Spec Templates - Template generation for specifications (WP32)
//!
//! This module provides spec template generation functionality:
//! - `generate_spec_template` - Generate boilerplate spec structure for each profile type
//! - `fill_template` - Fill in template with session data
//!
//! ## Supported Profiles
//!
//! - **Api**: REST/GraphQL API specifications with endpoints, auth, error handling
//! - **Cli**: Command-line interface specifications with commands, flags, exit codes
//! - **Event**: Event-driven specifications with event types, payloads, triggers
//! - **Data**: Data model specifications with entities, relationships, queries
//! - **Workflow**: Workflow specifications with steps, transitions, error recovery
//! - **Ui**: User interface specifications with flows, states, components
//!
//! ## Design Principles
//!
//! - Zero panics: All fallible operations return `Result<T, E>`
//! - Pure functions: Core logic is deterministic and side-effect free
//! - Type safety: Leverage Rust's type system for compile-time guarantees

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use std::collections::HashMap;
use thiserror::Error;

use crate::intent::interview::types::{InterviewSession, Profile};

/// Error type for spec template operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SpecTemplateError {
  /// Template placeholder could not be found
  #[error("placeholder not found: {0}")]
  PlaceholderNotFound(String),

  /// Template is empty
  #[error("template is empty")]
  EmptyTemplate,

  /// Session has no answers to fill template
  #[error("session has no answers")]
  NoAnswers,

  /// Required field is missing from session
  #[error("missing required field: {0}")]
  MissingField(String),

  /// JSON serialization failed
  #[error("JSON serialization failed: {0}")]
  JsonError(String),

  /// Template rendering failed
  #[error("template rendering failed: {0}")]
  RenderingError(String),
}

/// Placeholder pattern for template variables
const PLACEHOLDER_START: &str = "{{";
const PLACEHOLDER_END: &str = "}}";

/// Generate a boilerplate spec structure for the given profile type.
///
/// Creates a JSON-formatted spec template with:
/// - Profile-specific required sections
/// - Placeholder fields for customization
/// - Documentation comments
///
/// # Errors
/// Returns `SpecTemplateError::RenderingError` if template generation fails.
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::templates::spec_templates::{generate_spec_template, SpecTemplateError};
/// use clarity_web::intent::Profile;
///
/// let template = generate_spec_template(Profile::Api)?;
/// assert!(template.contains("base_url"));
/// assert!(template.contains("auth_method"));
/// ```
pub fn generate_spec_template(profile: Profile) -> Result<String, SpecTemplateError> {
  let template = match profile {
    Profile::Api => generate_api_template(),
    Profile::Cli => generate_cli_template(),
    Profile::Event => generate_event_template(),
    Profile::Data => generate_data_template(),
    Profile::Workflow => generate_workflow_template(),
    Profile::Ui => generate_ui_template(),
  };

  if template.trim().is_empty() {
    return Err(SpecTemplateError::EmptyTemplate);
  }

  Ok(template)
}

/// Fill a template with data from an interview session.
///
/// Replaces placeholder variables in the template with values extracted
/// from the session's answers. Placeholders use the `{{field_name}}` syntax.
///
/// # Errors
/// - `SpecTemplateError::EmptyTemplate` if template is empty
/// - `SpecTemplateError::NoAnswers` if session has no answers
/// - `SpecTemplateError::PlaceholderNotFound` if a required placeholder cannot be filled
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::templates::spec_templates::{fill_template, SpecTemplateError};
/// use clarity_web::intent::InterviewSession;
///
/// let template = r#"{"name": "{{spec_name}}", "url": "{{base_url}}"}"#;
/// let session = create_session_with_answers();
///
/// let filled = fill_template(template, &session)?;
/// assert!(filled.contains("https://api.example.com"));
/// ```
pub fn fill_template(
  template: &str,
  session: &InterviewSession,
) -> Result<String, SpecTemplateError> {
  if template.trim().is_empty() {
    return Err(SpecTemplateError::EmptyTemplate);
  }

  // Build a map of field names to values from session answers
  let field_values = extract_field_values(session);

  // Replace placeholders with values
  let mut result = template.to_string();

  for (field, value) in &field_values {
    let placeholder = format!("{PLACEHOLDER_START}{field}{PLACEHOLDER_END}");
    result = result.replace(&placeholder, value);
  }

  // Check for unfilled placeholders (still contain {{...}})
  if result.contains(PLACEHOLDER_START) && result.contains(PLACEHOLDER_END) {
    // Extract remaining placeholders for error message
    let remaining = extract_placeholders(&result);
    if !remaining.is_empty() {
      return Err(SpecTemplateError::PlaceholderNotFound(remaining.join(", ")));
    }
  }

  Ok(result)
}

/// Extract field values from session answers into a flat map.
fn extract_field_values(session: &InterviewSession) -> HashMap<String, String> {
  let mut values = HashMap::new();

  // Add session metadata
  values.insert("session_id".to_string(), session.id.clone());
  values.insert("profile".to_string(), session.profile.as_str().to_string());
  values.insert("created_at".to_string(), session.created_at.clone());
  values.insert("updated_at".to_string(), session.updated_at.clone());

  // Add answers and extracted fields
  for answer in &session.answers {
    // Add the response as a field if question_id maps to a known field
    let question_key = answer.question_id.replace(' ', "_").to_lowercase();
    values.insert(question_key.clone(), answer.response.clone());

    // Add all extracted fields
    for (key, value) in &answer.extracted {
      values.insert(key.clone(), value.clone());
    }

    // Add notes if present
    if !answer.notes.is_empty() {
      values.insert(format!("{question_key}_notes"), answer.notes.clone());
    }
  }

  // Add raw notes
  values.insert("raw_notes".to_string(), session.raw_notes.clone());

  values
}

/// Extract placeholder names from a template string.
fn extract_placeholders(template: &str) -> Vec<String> {
  let mut placeholders = Vec::new();
  let mut start_idx = None;

  for (i, ch) in template.char_indices() {
    match ch {
      '{' if template[i..].starts_with(PLACEHOLDER_START) => {
        start_idx = Some(i + PLACEHOLDER_START.len());
      }
      '}' if template[i..].starts_with(PLACEHOLDER_END) => {
        if let Some(start) = start_idx {
          let placeholder: String = template[start..i].chars().collect();
          let trimmed = placeholder.trim().to_string();
          if !trimmed.is_empty() {
            placeholders.push(trimmed);
          }
        }
        start_idx = None;
      }
      _ => {}
    }
  }

  placeholders.into_iter().unique().collect()
}

/// Generate API profile template.
fn generate_api_template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "api",
  "features": [
    {
      "name": "authentication",
      "description": "API authentication and authorization",
      "behaviors": [
        {
          "name": "authenticate_request",
          "description": "Validate authentication credentials",
          "verification": {
            "verification_type": "integration_test",
            "description": "Test authentication flow",
            "example": "assert!(authenticate(valid_token).is_ok())"
          }
        }
      ]
    }
  ],
  "api_spec": {
    "base_url": "{{base_url}}",
    "auth_method": "{{auth_method}}",
    "response_format": "{{response_format}}",
    "versioning": "{{versioning}}"
  },
  "endpoints": [
    {
      "path": "{{endpoint_path}}",
      "method": "{{http_method}}",
      "description": "{{endpoint_description}}",
      "request_schema": {},
      "response_schema": {}
    }
  ],
  "error_cases": [
    {
      "code": 400,
      "description": "Bad Request",
      "recovery": "Validate input parameters"
    },
    {
      "code": 401,
      "description": "Unauthorized",
      "recovery": "Provide valid authentication"
    },
    {
      "code": 404,
      "description": "Not Found",
      "recovery": "Check resource identifier"
    },
    {
      "code": 500,
      "description": "Internal Server Error",
      "recovery": "Retry with exponential backoff"
    }
  ],
  "happy_path": "{{happy_path}}",
  "invariants": [
    {
      "name": "authenticated_access",
      "description": "All endpoints require authentication",
      "constraint": "request.auth != null"
    }
  ],
  "ai_hints": {
    "implementation": {
      "architecture": "REST API with middleware pipeline",
      "performance_notes": "Use connection pooling and caching",
      "error_handling": "Centralized error handling middleware"
    },
    "security": {
      "authentication": "{{auth_method}}",
      "authorization": "Role-based access control",
      "data_sensitivity": "Classify data by sensitivity level"
    }
  }
}
"#
  .to_string()
}

/// Generate CLI profile template.
fn generate_cli_template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "cli",
  "features": [
    {
      "name": "command_parsing",
      "description": "Parse command-line arguments",
      "behaviors": [
        {
          "name": "parse_arguments",
          "description": "Parse and validate CLI arguments",
          "verification": {
            "verification_type": "unit_test",
            "description": "Test argument parsing",
            "example": "assert!(parse_args([\"--help\"]).is_ok())"
          }
        }
      ]
    }
  ],
  "cli_spec": {
    "command_name": "{{command_name}}",
    "version": "{{version}}",
    "help_text": "{{help_text}}"
  },
  "commands": [
    {
      "name": "{{command}}",
      "description": "{{command_description}}",
      "flags": [
        {
          "short": "-h",
          "long": "--help",
          "description": "Show help message"
        }
      ],
      "arguments": []
    }
  ],
  "exit_codes": [
    {
      "code": 0,
      "description": "Success"
    },
    {
      "code": 1,
      "description": "General error"
    },
    {
      "code": 2,
      "description": "Invalid arguments"
    }
  ],
  "happy_path": "{{happy_path}}",
  "invariants": [
    {
      "name": "zero_exit_on_success",
      "description": "Exit code 0 only on success",
      "constraint": "success => exit_code == 0"
    }
  ],
  "ai_hints": {
    "implementation": {
      "architecture": "Command pattern with subcommand dispatch",
      "performance_notes": "Lazy load subcommands",
      "error_handling": "User-friendly error messages with suggestions"
    }
  }
}
"#
  .to_string()
}

/// Generate Event profile template.
fn generate_event_template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "event",
  "features": [
    {
      "name": "event_production",
      "description": "Produce events to message broker",
      "behaviors": [
        {
          "name": "publish_event",
          "description": "Publish event to topic",
          "verification": {
            "verification_type": "integration_test",
            "description": "Test event publishing",
            "example": "assert!(publish(event).is_ok())"
          }
        }
      ]
    }
  ],
  "event_spec": {
    "event_type": "{{event_type}}",
    "payload_schema": {{payload_schema}},
    "trigger": "{{trigger}}"
  },
  "events": [
    {
      "type": "{{event_type}}",
      "version": "1.0",
      "description": "{{event_description}}",
      "payload": {
        "fields": []
      },
      "metadata": {
        "timestamp": "ISO8601",
        "source": "{{source}}",
        "correlation_id": "UUID"
      }
    }
  ],
  "subscriptions": [
    {
      "topic": "{{topic}}",
      "handler": "{{handler}}",
      "retry_policy": {
        "max_retries": 3,
        "backoff_ms": 1000
      }
    }
  ],
  "invariants": [
    {
      "name": "event_ordering",
      "description": "Events are ordered by timestamp",
      "constraint": "event_a.timestamp < event_b.timestamp => event_a.sequence < event_b.sequence"
    }
  ],
  "ai_hints": {
    "implementation": {
      "architecture": "Event sourcing with CQRS",
      "performance_notes": "Use event batching for throughput",
      "error_handling": "Dead letter queue for failed events"
    }
  }
}
"#
  .to_string()
}

/// Generate Data profile template.
fn generate_data_template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "data",
  "features": [
    {
      "name": "data_access",
      "description": "Data storage and retrieval",
      "behaviors": [
        {
          "name": "query_data",
          "description": "Query data with filters",
          "verification": {
            "verification_type": "integration_test",
            "description": "Test data queries",
            "example": "assert!(query(filter).len() > 0)"
          }
        }
      ]
    }
  ],
  "data_spec": {
    "data_model": {{data_model}},
    "access_patterns": "{{access_patterns}}",
    "retention": "{{retention}}"
  },
  "entities": [
    {
      "name": "{{entity_name}}",
      "description": "{{entity_description}}",
      "fields": [
        {
          "name": "id",
          "type": "UUID",
          "required": true,
          "primary_key": true
        }
      ],
      "indexes": [],
      "relationships": []
    }
  ],
  "queries": [
    {
      "name": "{{query_name}}",
      "description": "{{query_description}}",
      "parameters": [],
      "returns": "List<Entity>"
    }
  ],
  "migrations": {
    "strategy": "versioned",
    "rollback_supported": true
  },
  "invariants": [
    {
      "name": "referential_integrity",
      "description": "Foreign keys must reference existing records",
      "constraint": "FOREIGN KEY references must exist"
    }
  ],
  "ai_hints": {
    "implementation": {
      "architecture": "Repository pattern with unit of work",
      "performance_notes": "Use read replicas for queries",
      "error_handling": "Constraint violation mapping to domain errors"
    }
  }
}
"#
  .to_string()
}

/// Generate Workflow profile template.
fn generate_workflow_template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "workflow",
  "features": [
    {
      "name": "workflow_execution",
      "description": "Execute workflow steps",
      "behaviors": [
        {
          "name": "execute_step",
          "description": "Execute a workflow step",
          "verification": {
            "verification_type": "integration_test",
            "description": "Test workflow execution",
            "example": "assert!(execute_step(step_id).is_ok())"
          }
        }
      ]
    }
  ],
  "workflow_spec": {
    "steps": {{steps}},
    "happy_path": "{{happy_path}}",
    "error_recovery": "{{error_recovery}}"
  },
  "states": [
    {
      "name": "pending",
      "description": "Workflow is pending"
    },
    {
      "name": "running",
      "description": "Workflow is running"
    },
    {
      "name": "completed",
      "description": "Workflow completed successfully"
    },
    {
      "name": "failed",
      "description": "Workflow failed"
    }
  ],
  "transitions": [
    {
      "from": "pending",
      "to": "running",
      "trigger": "start_workflow"
    },
    {
      "from": "running",
      "to": "completed",
      "trigger": "all_steps_done"
    },
    {
      "from": "running",
      "to": "failed",
      "trigger": "unrecoverable_error"
    }
  ],
  "error_handling": {
    "retry_strategy": "exponential_backoff",
    "max_retries": 3,
    "compensation_enabled": true
  },
  "invariants": [
    {
      "name": "state_consistency",
      "description": "Workflow state is always consistent",
      "constraint": "state in valid_states"
    }
  ],
  "ai_hints": {
    "implementation": {
      "architecture": "State machine with saga pattern",
      "performance_notes": "Use async execution for long-running steps",
      "error_handling": "Compensation actions for rollback"
    }
  }
}
"#
  .to_string()
}

/// Generate UI profile template.
fn generate_ui_template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "ui",
  "features": [
    {
      "name": "user_interaction",
      "description": "Handle user interactions",
      "behaviors": [
        {
          "name": "render_component",
          "description": "Render UI component",
          "verification": {
            "verification_type": "visual_test",
            "description": "Test component rendering",
            "example": "assert!(render(component).snapshot_matches())"
          }
        }
      ]
    }
  ],
  "ui_spec": {
    "user_flows": {{user_flows}},
    "happy_path": "{{happy_path}}",
    "states": "{{states}}"
  },
  "components": [
    {
      "name": "{{component_name}}",
      "description": "{{component_description}}",
      "props": [],
      "state": {
        "local": [],
        "derived": []
      },
      "events": []
    }
  ],
  "screens": [
    {
      "name": "{{screen_name}}",
      "route": "{{route}}",
      "components": [],
      "loading_state": {
        "strategy": "skeleton",
        "timeout_ms": 5000
      },
      "error_state": {
        "strategy": "retry_button",
        "message": "Something went wrong"
      }
    }
  ],
  "accessibility": {
    "wcag_level": "AA",
    "screen_reader_support": true,
    "keyboard_navigation": true
  },
  "invariants": [
    {
      "name": "responsive_design",
      "description": "UI works on all screen sizes",
      "constraint": "layout adapts to viewport width"
    }
  ],
  "ai_hints": {
    "implementation": {
      "architecture": "Component-based with unidirectional data flow",
      "performance_notes": "Virtual scrolling for large lists",
      "error_handling": "Error boundaries with fallback UI"
    }
  }
}
"#
  .to_string()
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;
  use crate::intent::interview::types::{Answer, InterviewStage, Perspective};
  use std::collections::HashMap;

  fn create_test_session() -> InterviewSession {
    InterviewSession::new(
      "test-session-123".to_string(),
      Profile::Api,
      "2026-02-27T00:00:00Z".to_string(),
    )
  }

  fn create_test_session_with_answers() -> InterviewSession {
    let mut session = create_test_session();

    let mut extracted = HashMap::new();
    extracted.insert(
      "base_url".to_string(),
      "https://api.example.com".to_string(),
    );
    extracted.insert("auth_method".to_string(), "Bearer".to_string());
    extracted.insert("response_format".to_string(), "JSON".to_string());
    extracted.insert(
      "happy_path".to_string(),
      "User authenticates and retrieves data".to_string(),
    );
    extracted.insert("spec_name".to_string(), "My API".to_string());
    extracted.insert("description".to_string(), "A sample API".to_string());

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "What is the API base URL?".to_string(),
      perspective: Perspective::Developer,
      round: 1,
      response: "The base URL is https://api.example.com".to_string(),
      extracted,
      confidence: 0.9,
      notes: "Production endpoint".to_string(),
      timestamp: "2026-02-27T00:01:00Z".to_string(),
    });

    session.stage = InterviewStage::Refinement;
    session
  }

  #[test]
  fn test_spec_template_error_display() {
    let err = SpecTemplateError::PlaceholderNotFound("base_url".to_string());
    assert!(format!("{err}").contains("base_url"));

    let err = SpecTemplateError::EmptyTemplate;
    assert!(format!("{err}").contains("empty"));

    let err = SpecTemplateError::NoAnswers;
    assert!(format!("{err}").contains("no answers"));

    let err = SpecTemplateError::MissingField("auth".to_string());
    assert!(format!("{err}").contains("auth"));

    let err = SpecTemplateError::JsonError("parse error".to_string());
    assert!(format!("{err}").contains("parse error"));

    let err = SpecTemplateError::RenderingError("failed".to_string());
    assert!(format!("{err}").contains("failed"));
  }

  #[test]
  fn test_generate_spec_template_api() {
    let result = generate_spec_template(Profile::Api);
    assert!(result.is_ok());

    let template = result.expect("template");
    assert!(template.contains("\"profile\": \"api\""));
    assert!(template.contains("base_url"));
    assert!(template.contains("auth_method"));
    assert!(template.contains("endpoints"));
    assert!(template.contains("error_cases"));
    assert!(template.contains("happy_path"));
  }

  #[test]
  fn test_generate_spec_template_cli() {
    let result = generate_spec_template(Profile::Cli);
    assert!(result.is_ok());

    let template = result.expect("template");
    assert!(template.contains("\"profile\": \"cli\""));
    assert!(template.contains("command_name"));
    assert!(template.contains("help_text"));
    assert!(template.contains("exit_codes"));
    assert!(template.contains("commands"));
  }

  #[test]
  fn test_generate_spec_template_event() {
    let result = generate_spec_template(Profile::Event);
    assert!(result.is_ok());

    let template = result.expect("template");
    assert!(template.contains("\"profile\": \"event\""));
    assert!(template.contains("event_type"));
    assert!(template.contains("payload_schema"));
    assert!(template.contains("trigger"));
    assert!(template.contains("subscriptions"));
  }

  #[test]
  fn test_generate_spec_template_data() {
    let result = generate_spec_template(Profile::Data);
    assert!(result.is_ok());

    let template = result.expect("template");
    assert!(template.contains("\"profile\": \"data\""));
    assert!(template.contains("data_model"));
    assert!(template.contains("access_patterns"));
    assert!(template.contains("retention"));
    assert!(template.contains("entities"));
    assert!(template.contains("queries"));
  }

  #[test]
  fn test_generate_spec_template_workflow() {
    let result = generate_spec_template(Profile::Workflow);
    assert!(result.is_ok());

    let template = result.expect("template");
    assert!(template.contains("\"profile\": \"workflow\""));
    assert!(template.contains("steps"));
    assert!(template.contains("transitions"));
    assert!(template.contains("error_recovery"));
    assert!(template.contains("states"));
  }

  #[test]
  fn test_generate_spec_template_ui() {
    let result = generate_spec_template(Profile::Ui);
    assert!(result.is_ok());

    let template = result.expect("template");
    assert!(template.contains("\"profile\": \"ui\""));
    assert!(template.contains("user_flows"));
    assert!(template.contains("components"));
    assert!(template.contains("screens"));
    assert!(template.contains("accessibility"));
  }

  #[test]
  fn test_generate_spec_template_all_profiles() {
    let profiles = [
      Profile::Api,
      Profile::Cli,
      Profile::Event,
      Profile::Data,
      Profile::Workflow,
      Profile::Ui,
    ];

    for profile in profiles {
      let result = generate_spec_template(profile);
      assert!(
        result.is_ok(),
        "Profile {profile:?} should generate template"
      );

      let template = result.expect("template");
      assert!(
        !template.is_empty(),
        "Profile {profile:?} template should not be empty"
      );
      assert!(
        template.contains("{{spec_name}}"),
        "Profile {profile:?} should have spec_name placeholder"
      );
      assert!(
        template.contains("{{description}}"),
        "Profile {profile:?} should have description placeholder"
      );
      assert!(
        template.contains("invariants"),
        "Profile {profile:?} should have invariants section"
      );
      assert!(
        template.contains("ai_hints"),
        "Profile {profile:?} should have ai_hints section"
      );
    }
  }

  #[test]
  fn test_fill_template_empty_template() {
    let session = create_test_session();
    let result = fill_template("", &session);
    assert!(matches!(result, Err(SpecTemplateError::EmptyTemplate)));

    let result = fill_template("   ", &session);
    assert!(matches!(result, Err(SpecTemplateError::EmptyTemplate)));
  }

  #[test]
  fn test_fill_template_no_placeholders() {
    let session = create_test_session();
    let template = r#"{"name": "fixed", "value": 123}"#;

    let result = fill_template(template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled template");
    assert_eq!(filled, template);
  }

  #[test]
  fn test_fill_template_with_answers() {
    let session = create_test_session_with_answers();
    let template = r#"{"name": "{{spec_name}}", "url": "{{base_url}}"}"#;

    let result = fill_template(template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled template");
    assert!(filled.contains("My API"));
    assert!(filled.contains("https://api.example.com"));
    assert!(!filled.contains("{{"));
  }

  #[test]
  fn test_fill_template_multiple_placeholders() {
    let session = create_test_session_with_answers();
    let template = r#"{"name": "{{spec_name}}", "url": "{{base_url}}", "auth": "{{auth_method}}"}"#;

    let result = fill_template(template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled template");
    assert!(filled.contains("My API"));
    assert!(filled.contains("https://api.example.com"));
    assert!(filled.contains("Bearer"));
  }

  #[test]
  fn test_fill_template_missing_placeholder() {
    let session = create_test_session(); // Session without answers
    let template = r#"{"name": "{{spec_name}}"}"#;

    let result = fill_template(template, &session);
    // Should fail because spec_name is not available
    assert!(matches!(
      result,
      Err(SpecTemplateError::PlaceholderNotFound(_))
    ));
  }

  #[test]
  fn test_fill_template_partial_fill() {
    let session = create_test_session_with_answers();
    // Template has both filled and unfilled placeholders
    let template = r#"{"name": "{{spec_name}}", "unknown": "{{unknown_field}}"}"#;

    let result = fill_template(template, &session);
    assert!(matches!(
      result,
      Err(SpecTemplateError::PlaceholderNotFound(_))
    ));

    if let Err(SpecTemplateError::PlaceholderNotFound(fields)) = result {
      assert!(fields.contains("unknown_field"));
    }
  }

  #[test]
  fn test_extract_placeholders_simple() {
    let template = r#"{"name": "{{spec_name}}"}"#;
    let placeholders = extract_placeholders(template);
    assert_eq!(placeholders, vec!["spec_name"]);
  }

  #[test]
  fn test_extract_placeholders_multiple() {
    let template = r#"{"a": "{{field_a}}", "b": "{{field_b}}"}"#;
    let placeholders = extract_placeholders(template);
    assert!(placeholders.contains(&"field_a".to_string()));
    assert!(placeholders.contains(&"field_b".to_string()));
  }

  #[test]
  fn test_extract_placeholders_with_whitespace() {
    let template = r#"{"name": "{{ spec_name }}"}"#;
    let placeholders = extract_placeholders(template);
    assert_eq!(placeholders, vec!["spec_name"]);
  }

  #[test]
  fn test_extract_placeholders_duplicates() {
    let template = r#"{"a": "{{field}}", "b": "{{field}}"}"#;
    let placeholders = extract_placeholders(template);
    assert_eq!(placeholders.len(), 1);
    assert_eq!(placeholders, vec!["field"]);
  }

  #[test]
  fn test_extract_placeholders_no_placeholders() {
    let template = r#"{"name": "fixed"}"#;
    let placeholders = extract_placeholders(template);
    assert!(placeholders.is_empty());
  }

  #[test]
  fn test_extract_field_values_empty_session() {
    let session = create_test_session();
    let values = extract_field_values(&session);

    assert_eq!(
      values.get("session_id"),
      Some(&"test-session-123".to_string())
    );
    assert_eq!(values.get("profile"), Some(&"api".to_string()));
  }

  #[test]
  fn test_extract_field_values_with_answers() {
    let session = create_test_session_with_answers();
    let values = extract_field_values(&session);

    assert_eq!(
      values.get("base_url"),
      Some(&"https://api.example.com".to_string())
    );
    assert_eq!(values.get("auth_method"), Some(&"Bearer".to_string()));
    assert_eq!(values.get("spec_name"), Some(&"My API".to_string()));
    assert_eq!(
      values.get("q1_notes"),
      Some(&"Production endpoint".to_string())
    );
  }

  #[test]
  fn test_full_workflow_api() {
    // Generate template
    let template = generate_spec_template(Profile::Api).expect("template");

    // Create session with all required fields
    let mut session = create_test_session();
    let mut extracted = HashMap::new();
    extracted.insert("spec_name".to_string(), "User API".to_string());
    extracted.insert("description".to_string(), "User management API".to_string());
    extracted.insert(
      "base_url".to_string(),
      "https://api.users.com/v1".to_string(),
    );
    extracted.insert("auth_method".to_string(), "OAuth2".to_string());
    extracted.insert("response_format".to_string(), "JSON".to_string());
    extracted.insert(
      "happy_path".to_string(),
      "Create, read, update, delete users".to_string(),
    );
    extracted.insert("endpoint_path".to_string(), "/users".to_string());
    extracted.insert("http_method".to_string(), "GET".to_string());
    extracted.insert(
      "endpoint_description".to_string(),
      "List all users".to_string(),
    );
    extracted.insert("versioning".to_string(), "URL path".to_string());

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "API Configuration".to_string(),
      perspective: Perspective::Developer,
      round: 1,
      response: "Configure the API".to_string(),
      extracted,
      confidence: 1.0,
      notes: String::new(),
      timestamp: "2026-02-27T00:00:00Z".to_string(),
    });

    // Fill template
    let result = fill_template(&template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled");
    assert!(filled.contains("User API"));
    assert!(filled.contains("https://api.users.com/v1"));
    assert!(filled.contains("OAuth2"));
  }

  #[test]
  fn test_full_workflow_cli() {
    let template = generate_spec_template(Profile::Cli).expect("template");

    let mut session = InterviewSession::new(
      "cli-session".to_string(),
      Profile::Cli,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let mut extracted = HashMap::new();
    extracted.insert("spec_name".to_string(), "MyCLI".to_string());
    extracted.insert("description".to_string(), "A CLI tool".to_string());
    extracted.insert("command_name".to_string(), "mycli".to_string());
    extracted.insert("help_text".to_string(), "A helpful CLI tool".to_string());
    extracted.insert(
      "happy_path".to_string(),
      "Run command successfully".to_string(),
    );
    extracted.insert("command".to_string(), "run".to_string());
    extracted.insert(
      "command_description".to_string(),
      "Run the tool".to_string(),
    );
    extracted.insert("version".to_string(), "1.0.0".to_string());

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "CLI Config".to_string(),
      perspective: Perspective::User,
      round: 1,
      response: "Configure CLI".to_string(),
      extracted,
      confidence: 1.0,
      notes: String::new(),
      timestamp: "2026-02-27T00:00:00Z".to_string(),
    });

    let result = fill_template(&template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled");
    assert!(filled.contains("MyCLI"));
    assert!(filled.contains("mycli"));
    assert!(filled.contains("1.0.0"));
  }

  #[test]
  fn test_full_workflow_event() {
    let template = generate_spec_template(Profile::Event).expect("template");

    let mut session = InterviewSession::new(
      "event-session".to_string(),
      Profile::Event,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let mut extracted = HashMap::new();
    extracted.insert("spec_name".to_string(), "OrderEvents".to_string());
    extracted.insert("description".to_string(), "Order event system".to_string());
    extracted.insert("event_type".to_string(), "order.created".to_string());
    extracted.insert("payload_schema".to_string(), "{}".to_string());
    extracted.insert("trigger".to_string(), "order submission".to_string());
    extracted.insert(
      "event_description".to_string(),
      "Order was created".to_string(),
    );
    extracted.insert("source".to_string(), "order-service".to_string());
    extracted.insert("topic".to_string(), "orders".to_string());
    extracted.insert("handler".to_string(), "OrderHandler".to_string());

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "Event Config".to_string(),
      perspective: Perspective::Developer,
      round: 1,
      response: "Configure events".to_string(),
      extracted,
      confidence: 1.0,
      notes: String::new(),
      timestamp: "2026-02-27T00:00:00Z".to_string(),
    });

    let result = fill_template(&template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled");
    assert!(filled.contains("OrderEvents"));
    assert!(filled.contains("order.created"));
  }

  #[test]
  fn test_full_workflow_data() {
    let template = generate_spec_template(Profile::Data).expect("template");

    let mut session = InterviewSession::new(
      "data-session".to_string(),
      Profile::Data,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let mut extracted = HashMap::new();
    extracted.insert("spec_name".to_string(), "UserData".to_string());
    extracted.insert("description".to_string(), "User data model".to_string());
    extracted.insert("data_model".to_string(), "relational".to_string());
    extracted.insert("access_patterns".to_string(), "CRUD".to_string());
    extracted.insert("retention".to_string(), "7 years".to_string());
    extracted.insert("entity_name".to_string(), "User".to_string());
    extracted.insert("entity_description".to_string(), "User entity".to_string());
    extracted.insert("query_name".to_string(), "find_by_email".to_string());
    extracted.insert(
      "query_description".to_string(),
      "Find user by email".to_string(),
    );

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "Data Config".to_string(),
      perspective: Perspective::Developer,
      round: 1,
      response: "Configure data".to_string(),
      extracted,
      confidence: 1.0,
      notes: String::new(),
      timestamp: "2026-02-27T00:00:00Z".to_string(),
    });

    let result = fill_template(&template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled");
    assert!(filled.contains("UserData"));
    assert!(filled.contains("User"));
  }

  #[test]
  fn test_full_workflow_workflow() {
    let template = generate_spec_template(Profile::Workflow).expect("template");

    let mut session = InterviewSession::new(
      "workflow-session".to_string(),
      Profile::Workflow,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let mut extracted = HashMap::new();
    extracted.insert("spec_name".to_string(), "OrderWorkflow".to_string());
    extracted.insert(
      "description".to_string(),
      "Order processing workflow".to_string(),
    );
    extracted.insert("steps".to_string(), "[validate, process, ship]".to_string());
    extracted.insert(
      "happy_path".to_string(),
      "Order processed and shipped".to_string(),
    );
    extracted.insert(
      "error_recovery".to_string(),
      "Retry with backoff".to_string(),
    );

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "Workflow Config".to_string(),
      perspective: Perspective::Ops,
      round: 1,
      response: "Configure workflow".to_string(),
      extracted,
      confidence: 1.0,
      notes: String::new(),
      timestamp: "2026-02-27T00:00:00Z".to_string(),
    });

    let result = fill_template(&template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled");
    assert!(filled.contains("OrderWorkflow"));
  }

  #[test]
  fn test_full_workflow_ui() {
    let template = generate_spec_template(Profile::Ui).expect("template");

    let mut session = InterviewSession::new(
      "ui-session".to_string(),
      Profile::Ui,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let mut extracted = HashMap::new();
    extracted.insert("spec_name".to_string(), "DashboardUI".to_string());
    extracted.insert("description".to_string(), "Admin dashboard".to_string());
    extracted.insert("user_flows".to_string(), "[login, view, edit]".to_string());
    extracted.insert(
      "happy_path".to_string(),
      "User logs in and manages data".to_string(),
    );
    extracted.insert(
      "states".to_string(),
      "[loading, success, error]".to_string(),
    );
    extracted.insert("component_name".to_string(), "DataTable".to_string());
    extracted.insert(
      "component_description".to_string(),
      "Data table component".to_string(),
    );
    extracted.insert("screen_name".to_string(), "Dashboard".to_string());
    extracted.insert("route".to_string(), "/dashboard".to_string());

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "UI Config".to_string(),
      perspective: Perspective::User,
      round: 1,
      response: "Configure UI".to_string(),
      extracted,
      confidence: 1.0,
      notes: String::new(),
      timestamp: "2026-02-27T00:00:00Z".to_string(),
    });

    let result = fill_template(&template, &session);
    assert!(result.is_ok());

    let filled = result.expect("filled");
    assert!(filled.contains("DashboardUI"));
    assert!(filled.contains("DataTable"));
  }

  #[test]
  fn test_template_structure_consistency() {
    // All templates should have consistent structure
    let profiles = [
      Profile::Api,
      Profile::Cli,
      Profile::Event,
      Profile::Data,
      Profile::Workflow,
      Profile::Ui,
    ];

    for profile in profiles {
      let template = generate_spec_template(profile).expect("template");

      // All templates should be valid JSON-like structure
      assert!(
        template.contains("\"name\":"),
        "Profile {profile:?} should have name field"
      );
      assert!(
        template.contains("\"description\":"),
        "Profile {profile:?} should have description field"
      );
      assert!(
        template.contains("\"profile\":"),
        "Profile {profile:?} should have profile field"
      );
      assert!(
        template.contains("\"features\":"),
        "Profile {profile:?} should have features field"
      );
      assert!(
        template.contains("\"invariants\":"),
        "Profile {profile:?} should have invariants field"
      );
      assert!(
        template.contains("\"ai_hints\":"),
        "Profile {profile:?} should have ai_hints field"
      );

      // All templates should have common placeholders
      assert!(
        template.contains("{{spec_name}}"),
        "Profile {profile:?} should have spec_name placeholder"
      );
      assert!(
        template.contains("{{description}}"),
        "Profile {profile:?} should have description placeholder"
      );
    }
  }
}
