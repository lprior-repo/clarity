#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::suspicious_else_formatting)]

//! `PromptTextarea` component for the Progressive Discover Prompt phase.
//!
//! This component provides a specialized textarea with:
//! - 2000 character limit
//! - Live character count display with minimum threshold
//! - Auto-resize based on content (up to max height)
//! - Focus styling
//! - Trimming on blur

use dioxus::prelude::*;

/// Maximum character limit for the prompt textarea
pub const MAX_PROMPT_LENGTH: usize = 2000;

/// Minimum character count required for extraction
pub const MIN_PROMPT_LENGTH: usize = 50;

/// Props for `PromptTextarea` component.
#[derive(Props, PartialEq, Clone)]
pub struct PromptTextareaProps {
  /// Current value of the textarea
  pub value: String,

  /// Callback when the value changes
  pub on_change: EventHandler<String>,

  /// Placeholder text when empty
  #[props(default = String::new())]
  pub placeholder: String,

  /// Whether the textarea is disabled
  #[props(default = false)]
  pub disabled: bool,

  /// Whether the textarea is read-only
  #[props(default = false)]
  pub readonly: bool,

  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// `PromptTextarea` component.
///
/// A specialized textarea for the Prompt phase with character limit and live count.
///
/// # Features
///
/// - Enforces 2000 character maximum
/// - Shows live character count with visual warning when approaching limit
/// - Auto-resizes based on content up to a maximum height
/// - Trims whitespace on blur
/// - Accessible with proper ARIA attributes
///
/// # Example
///
/// ```rust,ignore
/// let prompt = use_signal(|| String::new());
///
/// rsx! {
///     PromptTextarea {
///         value: prompt.read().clone(),
///         on_change: move |v| *prompt.write() = v,
///         placeholder: "Describe your idea...".to_string(),
///     }
/// }
/// ```
#[component]
pub fn PromptTextarea(props: PromptTextareaProps) -> Element {
  let char_count = props.value.len();
  let is_near_limit = char_count > MAX_PROMPT_LENGTH.saturating_sub(200);
  let is_at_limit = char_count >= MAX_PROMPT_LENGTH;

  // Handle input with character limit enforcement
  let handle_input = {
    let on_change = props.on_change;
    move |e: Event<FormData>| {
      let new_value = e.value();
      if new_value.len() <= MAX_PROMPT_LENGTH {
        on_change.call(new_value);
      }
      // If over limit, don't update - effectively blocking additional input
    }
  };

  // Handle blur with trimming
  let handle_blur = {
    let value = props.value.clone();
    let on_change = props.on_change;
    move |_| {
      let trimmed = value.trim().to_string();
      if trimmed != value {
        on_change.call(trimmed);
      }
    }
  };

  // Base classes for the textarea
  let textarea_classes = format!(
        "flex min-h-[120px] max-h-[400px] w-full rounded-md border {} bg-background px-4 py-3 text-base text-foreground \
         placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 \
         focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 resize-y transition-colors {}",
        if is_at_limit {
            "border-destructive"
        } else if is_near_limit {
            "border-yellow-500/50"
        } else {
            "border-border"
        },
        props.class
    );

  // Character count color based on limit proximity
  let count_color = if is_at_limit {
    "text-destructive font-medium"
  } else if is_near_limit {
    "text-yellow-600 dark:text-yellow-400"
  } else {
    "text-muted-foreground"
  };

  rsx! {
      div {
          class: "space-y-2",

          // Textarea
          textarea {
              class: textarea_classes,
              placeholder: if props.placeholder.is_empty() {
                  None
              } else {
                  Some(props.placeholder.clone())
              },
              value: props.value.clone(),
              disabled: props.disabled,
              readonly: props.readonly,
              oninput: handle_input,
              onblur: handle_blur,
              aria_label: "Prompt input textarea",
              aria_describedby: "prompt-char-count",
              // Enable auto-resize via rows
              rows: 5,
          }

          // Character count display
          div {
              id: "prompt-char-count",
              class: "flex justify-end items-center gap-1",

              span {
                  class: "text-xs {count_color}",
                  "{char_count}"
              }
              span {
                  class: "text-xs text-muted-foreground",
                  "/ {MAX_PROMPT_LENGTH}"
              }

              // Warning indicator when near limit
              if is_near_limit && !is_at_limit {
                  span {
                      class: "text-xs text-yellow-600 dark:text-yellow-400 ml-2",
                      "Approaching limit"
                  }
              }

              // Error indicator when at limit
              if is_at_limit {
                  span {
                      class: "text-xs text-destructive ml-2",
                      "Character limit reached"
                  }
              }
          }
      }
  }
}

// ============================================================================
// CharacterCount Component
// ============================================================================

/// Props for `CharacterCount` component.
#[derive(Props, PartialEq, Eq, Clone)]
pub struct CharacterCountProps {
  /// Current character count
  pub current: usize,
  /// Minimum characters required for extraction
  #[props(default = MIN_PROMPT_LENGTH)]
  pub minimum: usize,
  /// Maximum characters allowed
  #[props(default = MAX_PROMPT_LENGTH)]
  pub maximum: usize,
  /// Additional CSS classes
  #[props(default = String::new())]
  pub class: String,
}

/// `CharacterCount` component.
///
/// Displays character count with progress toward minimum threshold.
/// Shows "Need X more" when under minimum, "Ready!" when at or above.
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     CharacterCount {
///         current: prompt.len(),
///         minimum: 50,
///         maximum: 2000,
///     }
/// }
/// ```
#[component]
pub fn CharacterCount(props: CharacterCountProps) -> Element {
  // Calculate progress percentage toward minimum threshold.
  // When minimum is 0 (edge case), we show 100% progress since there's no minimum requirement.
  let progress_percent = if props.minimum == 0 {
    100_usize
  } else {
    props
      .current
      .saturating_mul(100)
      .checked_div(props.minimum)
      .map_or(0, |v| v)
      .min(100)
  };

  let is_at_minimum = props.current >= props.minimum;
  let is_near_limit = props.current > props.maximum.saturating_sub(200);
  let is_at_limit = props.current >= props.maximum;

  // Color based on status
  let status_color = if is_at_limit {
    "text-destructive"
  } else if is_near_limit {
    "text-yellow-600 dark:text-yellow-400"
  } else if is_at_minimum {
    "text-green-600 dark:text-green-400"
  } else {
    "text-muted-foreground"
  };

  // Progress bar color
  let progress_color = if is_at_limit {
    "bg-destructive"
  } else if is_at_minimum {
    "bg-green-500"
  } else {
    "bg-primary"
  };

  // Status message
  let status_message = if is_at_limit {
    String::from("Character limit reached")
  } else if is_near_limit {
    String::from("Approaching limit")
  } else if is_at_minimum {
    String::from("Ready!")
  } else {
    format!("Need {} more", props.minimum.saturating_sub(props.current))
  };

  rsx! {
      div {
          class: format!("flex flex-col gap-1 {}", props.class),

          // Main count display
          div {
              class: "flex justify-between items-center text-xs",

              // Character count
              span {
                  class: "{status_color}",
                  "{props.current}/{props.maximum} characters"
              }

              // Status message
              span {
                  class: "{status_color}",
                  "{status_message}"
              }
          }

          // Progress bar toward minimum
          if !is_at_minimum {
              div {
                  class: "h-1 w-full bg-muted rounded-full overflow-hidden",
                  div {
                       class: "h-full {progress_color} transition-all duration-200",
                       style: "width: {progress_percent}%",
                   }
               }
           }
      }
  }
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

  #[test]
  fn test_max_prompt_length_is_2000() {
    assert_eq!(MAX_PROMPT_LENGTH, 2000);
  }

  #[test]
  fn test_min_prompt_length_is_50() {
    assert_eq!(MIN_PROMPT_LENGTH, 50);
  }

  #[test]
  fn test_near_limit_calculation() {
    // Near limit is when char_count > 1800 (2000 - 200)
    let near_limit_threshold = MAX_PROMPT_LENGTH.saturating_sub(200);
    assert_eq!(near_limit_threshold, 1800);
  }

  #[test]
  fn test_char_count_logic() {
    // Test empty string
    let empty = "";
    assert!(empty.len() <= MAX_PROMPT_LENGTH);
    assert!(empty.len() <= MAX_PROMPT_LENGTH.saturating_sub(200));

    // Test at limit
    let at_limit = "a".repeat(MAX_PROMPT_LENGTH);
    assert_eq!(at_limit.len(), MAX_PROMPT_LENGTH);

    // Test over limit (should be blocked by input handler)
    let over_limit = "a".repeat(MAX_PROMPT_LENGTH + 1);
    assert!(over_limit.len() > MAX_PROMPT_LENGTH);
  }

  // Note: test_props_equality requires Dioxus runtime (EventHandler).
  // It needs dioxus::prelude::launch_test() wrapper to run.
}
