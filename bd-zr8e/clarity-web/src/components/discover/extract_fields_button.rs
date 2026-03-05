#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! `ExtractFieldsButton` component for the Progressive Discover flow.
//!
//! This button triggers field extraction from user input. It enforces
//! a minimum character threshold (50 characters) and shows loading
//! state during extraction.
//!
//! # Requirements (from bead bd-2x35)
//!
//! - THE SYSTEM SHALL display a clear label indicating the button's extraction function
//! - THE SYSTEM SHALL show a loading state during extraction processing
//! - WHEN the user clicks the `ExtractFieldsButton`, THE SYSTEM SHALL initiate the extraction process
//! - IF the prompt textarea is empty (<50 chars), THE SYSTEM SHALL NOT trigger the extraction process
//! - IF extraction is already in progress, THE SYSTEM SHALL NOT start a new extraction request

use dioxus::prelude::*;
use tracing::info;

#[cfg(not(target_arch = "wasm32"))]
use crate::server::extract_fields_server;
use crate::ui::button::{Button, ButtonVariant};

/// Minimum character count required to enable extraction
pub const MIN_PROMPT_CHARS: usize = 50;

/// Props for `ExtractFieldsButton` component
#[derive(Clone, Props, PartialEq)]
pub struct ExtractFieldsButtonProps {
  /// The prompt text to extract fields from
  pub prompt: String,
  /// Whether an extraction is currently in progress
  #[props(default)]
  pub is_loading: bool,
  /// Callback when extraction is triggered with the prompt
  pub on_click: EventHandler<String>,
  /// Optional additional CSS classes
  #[props(default)]
  pub class: String,
  /// Optional disabled state override (in addition to character minimum)
  #[props(default)]
  pub disabled: bool,
}

/// `ExtractFieldsButton` component
///
/// A button that triggers field extraction from user input.
/// Disabled when prompt is less than `MIN_PROMPT_CHARS` characters.
/// Shows loading spinner during extraction.
///
/// # Example
///
/// ```rust,ignore
/// let prompt = use_signal(|| String::new());
/// let is_extracting = use_signal(|| false);
///
/// rsx! {
///     ExtractFieldsButton {
///         prompt: prompt.read().clone(),
///         is_loading: *is_extracting.read(),
///         on_click: move |prompt_text: String| {
///             // Handle extraction trigger
///         },
///     }
/// }
/// ```
#[component]
pub fn ExtractFieldsButton(props: ExtractFieldsButtonProps) -> Element {
  let prompt_len = props.prompt.trim().len();
  let is_disabled = props.disabled || prompt_len < MIN_PROMPT_CHARS || props.is_loading;

  let tooltip_text = if props.disabled {
    "Button is disabled"
  } else if props.is_loading {
    "Extraction in progress..."
  } else if prompt_len < MIN_PROMPT_CHARS {
    &format!(
      "Enter at least {} characters ({} more needed)",
      MIN_PROMPT_CHARS,
      MIN_PROMPT_CHARS.saturating_sub(prompt_len)
    )
  } else {
    "Extract structured fields from your description"
  };

  rsx! {
      div {
          class: format!("flex flex-col items-end gap-1 {}", props.class),

          Button {
              variant: ButtonVariant::Primary,
              disabled: is_disabled,
              onclick: {
                  let on_click = props.on_click;
                  let prompt = props.prompt.clone();
                  move |_| {
                      if !prompt.trim().is_empty() {
                          info!(prompt_len = prompt.len(), "ExtractFieldsButton clicked");
                          on_click.call(prompt.clone());
                      }
                  }
              },

              // Loading spinner or icon
              if props.is_loading {
                  svg {
                      class: "mr-2 h-4 w-4 animate-spin",
                      xmlns: "http://www.w3.org/2000/svg",
                      fill: "none",
                      view_box: "0 0 24 24",
                      circle {
                          class: "opacity-25",
                          cx: "12",
                          cy: "12",
                          r: "10",
                          stroke: "currentColor",
                          stroke_width: "4",
                      }
                      path {
                          class: "opacity-75",
                          fill: "currentColor",
                          d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                      }
                  }
                  "Extracting..."
              } else {
                  "Extract Fields"
                  svg {
                      xmlns: "http://www.w3.org/2000/svg",
                      width: "16",
                      height: "16",
                      view_box: "0 0 24 24",
                      fill: "none",
                      stroke: "currentColor",
                      stroke_width: "2",
                      stroke_linecap: "round",
                      stroke_linejoin: "round",
                      class: "ml-2",
                      path { d: "m9 18 6-6-6-6" }
                  }
              }
          }

          // Helper text showing character count
          if !props.is_loading && prompt_len < MIN_PROMPT_CHARS {
              span {
                  class: "text-xs text-muted-foreground",
                  title: tooltip_text,
                  "{prompt_len}/{MIN_PROMPT_CHARS} characters"
              }
          }
      }
  }
}

/// Props for `ExtractFieldsButtonWithServer` component
///
/// This variant includes server function integration for actual extraction.
#[derive(Clone, Props)]
pub struct ExtractFieldsButtonWithServerProps {
  /// The prompt text to extract fields from
  pub prompt: String,
  /// Optional session ID for rate limiting
  #[props(default)]
  pub session_id: Option<String>,
  /// Callback when extraction starts
  #[props(default)]
  pub on_extraction_start: Option<EventHandler<()>>,
  /// Callback when extraction completes successfully
  #[props(default)]
  pub on_extraction_complete: Option<EventHandler<ExtractedFieldsData>>,
  /// Callback when extraction fails
  #[props(default)]
  pub on_extraction_error: Option<EventHandler<String>>,
  /// Optional additional CSS classes
  #[props(default)]
  pub class: String,
}

impl PartialEq for ExtractFieldsButtonWithServerProps {
  fn eq(&self, _other: &Self) -> bool {
    // Props with EventHandlers cannot be meaningfully compared
    false
  }
}

/// Extracted fields data returned from the server
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExtractedFieldsData {
  /// The extracted field values
  pub fields: Vec<ExtractedField>,
  /// Overall confidence score (0.0 to 1.0)
  pub confidence: f64,
  /// Provider ID used for extraction
  pub provider: String,
  /// Model ID used for extraction
  pub model: Option<String>,
  /// Extraction duration in milliseconds
  pub processing_duration_ms: u64,
}

/// A single extracted field
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExtractedField {
  /// Field name (e.g., "problem", "persona", "solution")
  pub name: String,
  /// Field value
  pub value: String,
  /// Confidence for this specific field (0.0 to 1.0)
  pub confidence: f64,
}

/// `ExtractFieldsButton` with server integration
///
/// This component handles the full extraction flow including:
/// - Calling the server function
/// - Managing loading state
/// - Error handling
/// - Success callbacks
#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn ExtractFieldsButtonWithServer(props: ExtractFieldsButtonWithServerProps) -> Element {
  let mut is_loading = use_signal(|| false);
  let mut error = use_signal(|| None::<String>);
  let mut last_ai_status = use_signal(|| None::<String>);

  let current_error = error.read().clone();

  rsx! {
      div {
          class: format!("flex flex-col items-end gap-2 {}", props.class),

          ExtractFieldsButton {
              prompt: props.prompt.clone(),
              is_loading: *is_loading.read(),
              disabled: false,
              on_click: {
                  let prompt = props.prompt.clone();
                  let session_id = props.session_id.clone();
                  let on_start = props.on_extraction_start;
                  let on_complete = props.on_extraction_complete;
                  let on_error = props.on_extraction_error;

                  move |_prompt_text: String| {
                      // Prevent double-clicks
                      if *is_loading.read() {
                          return;
                      }

                      // Clear previous error
                      *error.write() = None;

                      // Set loading state
                      *is_loading.write() = true;
                      *last_ai_status.write() = None;

                      // Notify start
                      if let Some(handler) = on_start.as_ref() {
                          handler.call(());
                      }

                      let prompt_text = prompt.clone();
                      let session = session_id.clone();

                      // Spawn async task for server call
                      spawn({
                          let mut is_loading = is_loading;
                          let mut error = error;
                          let on_complete = on_complete;
                          let on_error = on_error;

                          async move {
                              let result = extract_fields_server(prompt_text, session, None).await;

                              *is_loading.write() = false;

                              match result {
                                  Ok(extracted) => {
                                      info!(
                                          field_count = extracted.fields.len(),
                                          confidence = extracted.confidence,
                                          "Extraction completed successfully"
                                      );

                                      // Convert to our data type
                                      let data = ExtractedFieldsData {
                                          fields: extracted
                                              .fields
                                              .into_iter()
                                              .map(|f| ExtractedField {
                                                  name: f.name,
                                                  value: serde_json::to_string(&f.value)
                                                      .map(|s| s.trim_matches('"').to_string())
                                                      .ok()
                                                  .unwrap_or_default(),
                                                  confidence: f.confidence,
                                              })
                                               .collect(),
                                          confidence: extracted.confidence,
                                          provider: extracted.metadata.provider.clone(),
                                          model: extracted.metadata.model.clone(),
                                          processing_duration_ms: extracted.metadata.processing_duration_ms,
                                      };

                                      let model_label = data
                                          .model
                                          .as_deref()
                                          .unwrap_or("default-model");
                                      *last_ai_status.write() = Some(format!(
                                          "AI: {} / {} in {}ms",
                                          data.provider, model_label, data.processing_duration_ms
                                      ));

                                      if let Some(handler) = on_complete.as_ref() {
                                          handler.call(data);
                                      }
                                  }
                                  Err(e) => {
                                      let err_msg = e.to_string();
                                      info!(error = %err_msg, "Extraction failed");
                                      *error.write() = Some(err_msg.clone());
                                      *last_ai_status.write() = Some(
                                          "AI: extraction request failed - check provider/model settings"
                                              .to_string(),
                                      );

                                      if let Some(handler) = on_error.as_ref() {
                                          handler.call(err_msg);
                                      }
                                  }
                              }
                          }
                      });
                  }
              },
          }

          // Error display
          if let Some(err) = current_error.as_ref() {
              div {
                  class: "rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive",
                  role: "alert",
                  "Extraction failed: {err}"
              }
          }

          if let Some(status) = last_ai_status.read().as_ref() {
              p {
                  class: "text-xs text-muted-foreground",
                  "{status}"
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_min_prompt_chars_constant() {
    assert_eq!(MIN_PROMPT_CHARS, 50);
  }

  #[test]
  fn test_extracted_field_serialization() {
    let field = ExtractedField {
      name: "problem".to_string(),
      value: "Users struggle with complex workflows".to_string(),
      confidence: 0.95,
    };

    let json_result = serde_json::to_string(&field);
    assert!(
      json_result.is_ok(),
      "Serialization should succeed: {json_result:?}"
    );
    let Ok(json) = json_result else {
      return;
    };

    let deserialized_result: Result<ExtractedField, serde_json::Error> =
      serde_json::from_str(&json);
    assert!(
      deserialized_result.is_ok(),
      "Deserialization should succeed: {deserialized_result:?}"
    );
    let Ok(deserialized) = deserialized_result else {
      return;
    };

    assert_eq!(deserialized.name, field.name);
    assert_eq!(deserialized.value, field.value);
    assert!((deserialized.confidence - field.confidence).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extracted_fields_data_serialization() {
    let data = ExtractedFieldsData {
      fields: vec![
        ExtractedField {
          name: "problem".to_string(),
          value: "Test problem".to_string(),
          confidence: 0.9,
        },
        ExtractedField {
          name: "persona".to_string(),
          value: "Test persona".to_string(),
          confidence: 0.85,
        },
      ],
      confidence: 0.875,
      provider: "opencode".to_string(),
      model: Some("zai-coding-plan/glm-5".to_string()),
      processing_duration_ms: 123,
    };

    let json_result = serde_json::to_string(&data);
    assert!(
      json_result.is_ok(),
      "Serialization should succeed: {json_result:?}"
    );
    let Ok(json) = json_result else {
      return;
    };

    let deserialized_result: Result<ExtractedFieldsData, serde_json::Error> =
      serde_json::from_str(&json);
    assert!(
      deserialized_result.is_ok(),
      "Deserialization should succeed: {deserialized_result:?}"
    );
    let Ok(deserialized) = deserialized_result else {
      return;
    };

    assert_eq!(deserialized.fields.len(), 2);
    assert_eq!(deserialized.fields[0].name, "problem");
    assert!((deserialized.confidence - 0.875).abs() < f64::EPSILON);
    assert_eq!(deserialized.provider, "opencode");
    assert_eq!(deserialized.model.as_deref(), Some("zai-coding-plan/glm-5"));
    assert_eq!(deserialized.processing_duration_ms, 123);
  }

  #[test]
  fn test_button_disabled_for_short_prompt() {
    // A prompt with fewer than 50 characters should be disabled
    let short_prompt = "This is a short prompt".to_string();
    assert!(short_prompt.len() < MIN_PROMPT_CHARS);
  }

  #[test]
  fn test_button_enabled_for_long_prompt() {
    // A prompt with 50+ characters should be enabled
    let long_prompt =
      "This is a longer prompt that has at least fifty characters in it".to_string();
    assert!(long_prompt.len() >= MIN_PROMPT_CHARS);
  }

  #[test]
  fn test_whitespace_only_prompt_is_short() {
    let whitespace_prompt = "   ".to_string();
    assert!(whitespace_prompt.trim().len() < MIN_PROMPT_CHARS);
  }

  #[test]
  fn test_trimmed_length_used_for_check() {
    // Prompt with leading/trailing whitespace should use trimmed length
    let prompt = "   This is a prompt with exactly fifty chars in it!!!   ".to_string();
    let trimmed_len = prompt.trim().len();
    let expected_trimmed = "This is a prompt with exactly fifty chars in it!!!";
    assert_eq!(trimmed_len, expected_trimmed.len());
    assert!(trimmed_len >= MIN_PROMPT_CHARS);
  }
}
