//! Interactive Prompts for Spec Initialization
//!
//! Ported from intent-cli/src/intent/init_prompt.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use thiserror::Error;

/// Errors during initialization prompts
#[derive(Debug, Clone, Error)]
pub enum InitPromptError {
  #[error("spec name cannot be empty")]
  EmptySpecName,

  #[error("invalid template selection")]
  InvalidTemplateSelection,

  #[error("invalid number format")]
  InvalidNumberFormat,

  #[error("failed to read input: {0}")]
  InputError(String),
}

/// Template type for spec initialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
  Api,
  Cli,
  Event,
  Data,
  Workflow,
  Ui,
}

impl TemplateType {
  /// Get template name
  #[must_use]
  pub const fn name(&self) -> &'static str {
    match self {
      Self::Api => "API Service",
      Self::Cli => "CLI Tool",
      Self::Event => "Event Processor",
      Self::Data => "Data Pipeline",
      Self::Workflow => "Workflow Engine",
      Self::Ui => "UI Application",
    }
  }

  /// Get template description
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Api => "REST or GraphQL API service",
      Self::Cli => "Command-line interface tool",
      Self::Event => "Event-driven processor",
      Self::Data => "Data processing pipeline",
      Self::Workflow => "Workflow orchestration engine",
      Self::Ui => "User interface application",
    }
  }
}

/// Template information for display
#[derive(Debug, Clone)]
pub struct Template {
  pub type_: TemplateType,
  pub name: String,
  pub description: String,
}

impl Template {
  /// Create a new template
  #[must_use]
  pub fn new(type_: TemplateType) -> Self {
    Self {
      type_,
      name: type_.name().to_string(),
      description: type_.description().to_string(),
    }
  }
}

/// Get all available templates
#[must_use]
pub fn get_all_templates() -> Vec<Template> {
  [
    TemplateType::Api,
    TemplateType::Cli,
    TemplateType::Event,
    TemplateType::Data,
    TemplateType::Workflow,
    TemplateType::Ui,
  ]
  .iter()
  .map(|&t| Template::new(t))
  .collect()
}

/// Validate spec name
///
/// # Errors
/// Returns `InitPromptError` if the spec name is empty.
pub fn validate_spec_name(name: &str) -> Result<String, InitPromptError> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    Err(InitPromptError::EmptySpecName)
  } else {
    Ok(trimmed.to_string())
  }
}

/// Validate template selection
///
/// # Errors
/// Returns `InitPromptError` if the selection is invalid.
pub fn validate_template_selection(
  selection: usize,
  templates: &[Template],
) -> Result<TemplateType, InitPromptError> {
  if selection == 0 || selection > templates.len() {
    Err(InitPromptError::InvalidTemplateSelection)
  } else {
    Ok(templates[selection - 1].type_)
  }
}

/// Parse template selection from string
///
/// # Errors
/// Returns `InitPromptError` if the input is not a valid number.
pub fn parse_template_selection(input: &str) -> Result<usize, InitPromptError> {
  input
    .trim()
    .parse::<usize>()
    .map_err(|_| InitPromptError::InvalidNumberFormat)
}

/// Validate output filename and add .cue extension if needed
#[must_use]
pub fn validate_output_filename(input: &str, default_name: &str) -> String {
  let trimmed = input.trim();
  if trimmed.is_empty() {
    default_name.to_string()
  } else if trimmed.to_lowercase().ends_with(".cue") {
    trimmed.to_string()
  } else {
    format!("{trimmed}.cue")
  }
}

/// Generate default filename from spec name
#[must_use]
pub fn generate_default_filename(spec_name: &str) -> String {
  let sanitized: String = spec_name
    .to_lowercase()
    .chars()
    .map(|c| if c.is_alphanumeric() { c } else { '-' })
    .collect();

  let compressed: String = sanitized
    .split('-')
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("-");

  format!("{compressed}.cue")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_template_type_name() {
    assert_eq!(TemplateType::Api.name(), "API Service");
    assert_eq!(TemplateType::Cli.name(), "CLI Tool");
  }

  #[test]
  fn test_template_type_description() {
    assert_eq!(
      TemplateType::Api.description(),
      "REST or GraphQL API service"
    );
  }

  #[test]
  fn test_template_new() {
    let template = Template::new(TemplateType::Api);
    assert_eq!(template.name, "API Service");
    assert_eq!(template.description, "REST or GraphQL API service");
  }

  #[test]
  fn test_get_all_templates() {
    let templates = get_all_templates();
    assert_eq!(templates.len(), 6);
  }

  #[test]
  fn test_validate_spec_name_valid() {
    let result = validate_spec_name("My API");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "My API");
  }

  #[test]
  fn test_validate_spec_name_empty() {
    let result = validate_spec_name("");
    assert!(matches!(result, Err(InitPromptError::EmptySpecName)));

    let result = validate_spec_name("   ");
    assert!(matches!(result, Err(InitPromptError::EmptySpecName)));
  }

  #[test]
  fn test_validate_template_selection_valid() {
    let templates = get_all_templates();
    let result = validate_template_selection(1, &templates);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TemplateType::Api);
  }

  #[test]
  fn test_validate_template_selection_zero() {
    let templates = get_all_templates();
    let result = validate_template_selection(0, &templates);
    assert!(matches!(
      result,
      Err(InitPromptError::InvalidTemplateSelection)
    ));
  }

  #[test]
  fn test_validate_template_selection_too_high() {
    let templates = get_all_templates();
    let result = validate_template_selection(100, &templates);
    assert!(matches!(
      result,
      Err(InitPromptError::InvalidTemplateSelection)
    ));
  }

  #[test]
  fn test_parse_template_selection_valid() {
    let result = parse_template_selection("3");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
  }

  #[test]
  fn test_parse_template_selection_invalid() {
    let result = parse_template_selection("abc");
    assert!(matches!(result, Err(InitPromptError::InvalidNumberFormat)));
  }

  #[test]
  fn test_validate_output_filename_empty() {
    let result = validate_output_filename("", "default.cue");
    assert_eq!(result, "default.cue");
  }

  #[test]
  fn test_validate_output_filename_adds_extension() {
    let result = validate_output_filename("myspec", "default.cue");
    assert_eq!(result, "myspec.cue");
  }

  #[test]
  fn test_validate_output_filename_keeps_extension() {
    let result = validate_output_filename("myspec.cue", "default.cue");
    assert_eq!(result, "myspec.cue");
  }

  #[test]
  fn test_validate_output_filename_case_insensitive() {
    let result = validate_output_filename("myspec.CUE", "default.cue");
    assert_eq!(result, "myspec.CUE");
  }

  #[test]
  fn test_generate_default_filename() {
    assert_eq!(generate_default_filename("My API"), "my-api.cue");
    assert_eq!(generate_default_filename("Test Spec"), "test-spec.cue");
    assert_eq!(
      generate_default_filename("  Multiple   Spaces  "),
      "multiple-spaces.cue"
    );
  }

  #[test]
  fn test_init_prompt_error_display() {
    let err = InitPromptError::EmptySpecName;
    assert!(err.to_string().contains("empty"));

    let err = InitPromptError::InvalidTemplateSelection;
    assert!(err.to_string().contains("invalid"));
  }
}
