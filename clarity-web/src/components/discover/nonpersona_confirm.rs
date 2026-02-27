#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

use super::quality_score::{QualityDimension, QualityScore, QualityScoreBar};
use crate::ui::button::ButtonVariant;
use crate::ui::{Button, Textarea};

/// Props for NonpersonaDisplay component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct NonpersonaDisplayProps {
  /// The nonpersona text to display/edit
  pub nonpersona: Signal<String>,
  /// Placeholder text for the textarea
  #[props(default = String::from("Describe who you are NOT building for..."))]
  pub placeholder: String,
  /// Whether the nonpersona text is editable
  #[props(default = true)]
  pub editable: bool,
}

/// NonpersonaDisplay component
///
/// Displays and allows editing of the nonpersona description.
/// Nonpersonas define who you are explicitly NOT building for,
/// which helps maintain focus and prevent scope creep.
#[component]
pub fn NonpersonaDisplay(props: NonpersonaDisplayProps) -> Element {
  let nonpersona = props.nonpersona;
  let mut local_nonpersona = use_signal(|| nonpersona.read().clone());

  // Sync local nonpersona when external signal changes
  use_effect({
    let nonpersona = nonpersona.clone();
    move || {
      let external = nonpersona.read().clone();
      let local = local_nonpersona.read().clone();
      if external != local {
        *local_nonpersona.write() = external;
      }
    }
  });

  let on_input = {
    let mut nonpersona = nonpersona.clone();
    move |value: String| {
      *local_nonpersona.write() = value.clone();
      *nonpersona.write() = value;
    }
  };

  rsx! {
      div {
          class: "space-y-2",
          label {
              class: "text-sm font-medium text-foreground",
              "Based on what you wrote, here's who you are NOT building for:"
          }
          Textarea {
              value: local_nonpersona.read().clone(),
              placeholder: props.placeholder.clone(),
              disabled: !props.editable,
              rows: 4,
              oninput: on_input,
          }
      }
  }
}

/// Props for NonpersonaGuidance component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct NonpersonaGuidanceProps {
  /// Additional guidance text
  #[props(default = String::new())]
  pub guidance: String,
}

/// NonpersonaGuidance component
///
/// Displays guidance and examples for defining nonpersonas.
#[allow(non_snake_case)]
pub fn NonpersonaGuidance(_props: NonpersonaGuidanceProps) -> Element {
  rsx! {
      div {
          class: "space-y-3 rounded-lg border border-border/50 bg-muted/20 p-4",
          h4 {
              class: "text-sm font-medium text-foreground",
              "Why define nonpersonas?"
          }
          p {
              class: "text-xs text-muted-foreground",
              "Explicitly stating who you're NOT building for helps:"
          }
          ul {
              class: "text-xs text-muted-foreground space-y-1 list-disc list-inside mt-2",
              li {
                  "Prevent scope creep and feature bloat"
              }
              li {
                  "Keep your product focused and coherent"
              }
              li {
                  "Make trade-off decisions easier"
              }
              li {
                  "Help stakeholders understand product boundaries"
              }
          }
          div {
              class: "mt-3 pt-3 border-t border-border/30",
              p {
                  class: "text-xs text-muted-foreground",
                  span {
                      class: "font-medium text-foreground",
                      "Example: "
                  }
                  "If you're building a tool for software developers, you might explicitly exclude: managers who don't code, designers, marketing teams, or enterprise users requiring SSO."
              }
          }
      }
  }
}

/// Props for NonpersonaQuality component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct NonpersonaQualityProps {
  /// The nonpersona text to evaluate
  pub nonpersona: Signal<String>,
  /// Whether to show expanded details
  #[props(default = false)]
  pub expanded: bool,
}

/// NonpersonaQuality component
///
/// Displays quality metrics for the nonpersona description.
#[component]
pub fn NonpersonaQuality(props: NonpersonaQualityProps) -> Element {
  let nonpersona = props.nonpersona;

  // Calculate quality dimensions based on nonpersona
  let quality_score = use_memo({
    let nonpersona = nonpersona.clone();
    move || {
      let text = nonpersona.read();
      let overall = calculate_nonpersona_score(&text);

      let clarity = calculate_clarity_score(&text);
      let specificity = calculate_specificity_score(&text);
      let completeness = calculate_completeness_score(&text);

      QualityScore::new(overall).with_dimensions(vec![
        QualityDimension::new("Clarity", clarity).with_issues(get_clarity_issues(&text)),
        QualityDimension::new("Specificity", specificity)
          .with_issues(get_specificity_issues(&text)),
        QualityDimension::new("Completeness", completeness)
          .with_issues(get_completeness_issues(&text)),
      ])
    }
  });

  rsx! {
      div {
          class: "mt-4",
          QualityScoreBar {
              score: quality_score,
              expanded: props.expanded,
          }
      }
  }
}

/// Calculate overall nonpersona score (0-100)
fn calculate_nonpersona_score(text: &str) -> u8 {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return 0;
  }

  let word_count = trimmed.split_whitespace().count();
  match word_count {
    0..=9 => 30,
    10..=24 => 50,
    25..=49 => 70,
    50..=99 => 85,
    _ => 80,
  }
}

/// Calculate clarity score (0-100)
fn calculate_clarity_score(text: &str) -> u8 {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return 0;
  }

  let base = 50u8;

  // Check for clear exclusion language
  let lower = trimmed.to_lowercase();
  let clarity_bonus = if lower.contains("not") || lower.contains("excluding") {
    20
  } else if lower.contains("avoid") || lower.contains("won't") {
    15
  } else {
    0
  };

  (base + clarity_bonus).min(100)
}

/// Calculate specificity score (0-100)
fn calculate_specificity_score(text: &str) -> u8 {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return 0;
  }

  let word_count = trimmed.split_whitespace().count();
  match word_count {
    0..=9 => 30,
    10..=24 => 55,
    25..=49 => 75,
    _ => 70,
  }
}

/// Calculate completeness score (0-100)
fn calculate_completeness_score(text: &str) -> u8 {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return 0;
  }

  // Check for multiple exclusions (comma, semicolon, or "and" indicate multiple items)
  let lower = trimmed.to_lowercase();
  let has_multiple = lower.contains(',') || lower.contains(';') || lower.contains(" and ");

  if has_multiple {
    80
  } else {
    50
  }
}

/// Get clarity issues
fn get_clarity_issues(text: &str) -> Vec<String> {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return vec!["Nonpersona description is required".to_string()];
  }

  let lower = trimmed.to_lowercase();
  if !lower.contains("not") && !lower.contains("excluding") && !lower.contains("avoid") {
    return vec!["Use clear exclusion language (e.g., 'not for', 'excluding')".to_string()];
  }

  Vec::new()
}

/// Get specificity issues
fn get_specificity_issues(text: &str) -> Vec<String> {
  let word_count = text.trim().split_whitespace().count();
  if word_count < 10 {
    vec!["Add more specific details about who you're excluding".to_string()]
  } else {
    Vec::new()
  }
}

/// Get completeness issues
fn get_completeness_issues(text: &str) -> Vec<String> {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return vec!["Define at least one nonpersona group".to_string()];
  }

  let lower = trimmed.to_lowercase();
  if !lower.contains(',') && !lower.contains(';') && !lower.contains(" and ") {
    return vec!["Consider defining multiple nonpersona groups".to_string()];
  }

  Vec::new()
}

/// Props for NonpersonaConfirm component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct NonpersonaConfirmProps {
  /// The nonpersona text
  pub nonpersona: Signal<String>,
  /// Current step number (1-5)
  #[props(default = 4)]
  pub step: u8,
  /// Total steps in the confirmation flow
  #[props(default = 5)]
  pub total_steps: u8,
  /// Callback when Next is clicked
  pub on_next: Option<EventHandler<Event<MouseData>>>,
  /// Callback when Back is clicked
  pub on_back: Option<EventHandler<Event<MouseData>>>,
  /// Whether the Next button should be disabled
  #[props(default = false)]
  pub next_disabled: bool,
  /// Whether the Back button should be disabled
  #[props(default = false)]
  pub back_disabled: bool,
}

/// NonpersonaConfirm component
///
/// Composes:
/// - NonpersonaDisplay: Shows the nonpersona text for review/editing
/// - NonpersonaGuidance: Guidance for defining nonpersonas
/// - NonpersonaQuality: Quality score indicator
/// - Navigation: Back/Next buttons
///
/// This is the fourth confirmation step in the Progressive Discover flow.
#[component]
pub fn NonpersonaConfirm(props: NonpersonaConfirmProps) -> Element {
  let mut quality_expanded = use_signal(|| false);

  let toggle_quality = {
    move |_| {
      quality_expanded.toggle();
    }
  };

  rsx! {
      div {
          class: "space-y-6 rounded-lg border border-border/50 bg-card p-6 shadow-sm",

          // Header with step indicator
          div {
              class: "flex items-center justify-between border-b border-border/50 pb-4",
              h2 {
                  class: "text-lg font-semibold text-foreground",
                  "Nonpersona ({props.step}/{props.total_steps})"
              }
              span {
                  class: "text-sm text-muted-foreground",
                  "Define who you are NOT building for"
              }
          }

          // Nonpersona display section
          NonpersonaDisplay {
              nonpersona: props.nonpersona,
          }

          // Guidance section
          NonpersonaGuidance {}

          // Quality indicator section (expandable)
          div {
              class: "cursor-pointer",
              onclick: toggle_quality,
              NonpersonaQuality {
                  nonpersona: props.nonpersona,
                  expanded: *quality_expanded.read(),
              }
          }

          // Navigation buttons
          div {
              class: "flex items-center justify-between border-t border-border/50 pt-4",
              Button {
                  variant: ButtonVariant::Secondary,
                  disabled: props.back_disabled,
                  onclick: {
                      move |e| {
                          if let Some(handler) = &props.on_back {
                              handler.call(e);
                          }
                      }
                  },
                  // Left arrow icon
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
                      class: "mr-2",
                      path { d: "m15 18-6-6 6-6" }
                  }
                  "Back"
              }

              Button {
                  variant: ButtonVariant::Primary,
                  disabled: props.next_disabled,
                  onclick: {
                      move |e| {
                          if let Some(handler) = &props.on_next {
                              handler.call(e);
                          }
                      }
                  },
                  "Next"
                  // Right arrow icon
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
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_calculate_nonpersona_score_empty() {
    let score = calculate_nonpersona_score("");
    assert_eq!(score, 0);
  }

  #[test]
  fn test_calculate_nonpersona_score_short() {
    let score = calculate_nonpersona_score("Short text");
    assert_eq!(score, 30);
  }

  #[test]
  fn test_calculate_nonpersona_score_medium() {
    let score = calculate_nonpersona_score("This is a medium length description of who we exclude");
    assert!(score >= 50 && score <= 85);
  }

  #[test]
  fn test_calculate_clarity_score_with_not() {
    let score = calculate_clarity_score("This is not for enterprise users");
    assert!(score >= 70);
  }

  #[test]
  fn test_calculate_clarity_score_without_exclusion() {
    let score = calculate_clarity_score("Enterprise users are different");
    assert!(score < 70);
  }

  #[test]
  fn test_calculate_specificity_score_empty() {
    let score = calculate_specificity_score("");
    assert_eq!(score, 0);
  }

  #[test]
  fn test_calculate_completeness_score_multiple() {
    let score = calculate_completeness_score("Not for managers, designers, and marketing teams");
    assert_eq!(score, 80);
  }

  #[test]
  fn test_calculate_completeness_score_single() {
    let score = calculate_completeness_score("Not for enterprise users");
    assert_eq!(score, 50);
  }

  #[test]
  fn test_get_clarity_issues_empty() {
    let issues = get_clarity_issues("");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("required"));
  }

  #[test]
  fn test_get_clarity_issues_no_exclusion() {
    let issues = get_clarity_issues("Enterprise users have different needs");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("exclusion"));
  }

  #[test]
  fn test_get_specificity_issues_short() {
    let issues = get_specificity_issues("Short");
    assert_eq!(issues.len(), 1);
  }

  #[test]
  fn test_get_completeness_issues_single() {
    let issues = get_completeness_issues("Not for enterprise users");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("multiple"));
  }
}
