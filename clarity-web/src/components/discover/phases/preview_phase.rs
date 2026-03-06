#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Preview Phase component for the Progressive Discover flow.
//!
//! This module implements the Preview phase (bd-3fz2) which is the final review
//! before locking in the plan. It includes:
//!
//! - **`TranscriptSummary`** (bd-3h2v): Displays all 5 confirmed fields
//! - **`BrutalTruthsChecklist`** (bd-2k1q): The Four Brutal Truths checklist
//! - **`PreviewPhase`** (bd-3fz2): Main component composing summary and navigation

use dioxus::prelude::*;

use crate::components::discover::brutal_truths::{BrutalTruthsChecklist, BrutalTruthsState};
use crate::components::discover::preview_summary::PreviewSummary;
use crate::hooks::{ProgressiveDiscoverActions, ProgressiveDiscoverState};
use crate::storage::transcript_store::InterrogationTranscript;
use crate::ui::button::ButtonVariant;
use crate::ui::Button;

// ============================================================================
// TranscriptSummary Component (bd-3h2v)
// ============================================================================

/// Props for `TranscriptSummary` component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct TranscriptSummaryProps {
  /// The interrogation transcript to display
  pub transcript: InterrogationTranscript,
  /// Callback when a field edit is requested
  pub on_edit: Option<EventHandler<String>>,
}

/// `TranscriptSummary` component (bd-3h2v)
///
/// Displays all 5 confirmed fields from the interrogation transcript
/// in a clean, readable summary format.
///
/// # Fields Displayed
///
/// 1. Problem - The core problem being solved
/// 2. Persona - Target user description
/// 3. Solution - The proposed solution
/// 4. Nonpersona - Who the solution is NOT for
/// 5. Scenario - Trigger, value moment, and feeling
#[component]
pub fn TranscriptSummary(props: TranscriptSummaryProps) -> Element {
  let transcript = &props.transcript;

  rsx! {
      div {
          class: "space-y-4",

          // Header
          div {
              class: "border-b border-border/50 pb-3",
              h3 {
                  class: "text-lg font-semibold text-foreground",
                  "Confirmed Fields"
              }
              p {
                  class: "text-sm text-muted-foreground",
                  "Review your confirmed artifacts before locking."
              }
          }

          // Field summaries
          SummaryField {
              label: "Problem",
              value: transcript.problem.content.clone(),
              confidence: transcript.problem.confidence,
              on_edit: props.on_edit,
              field_id: "problem",
          }

          SummaryField {
              label: "Persona",
              value: transcript.persona.content.clone(),
              confidence: transcript.persona.confidence,
              on_edit: props.on_edit,
              field_id: "persona",
          }

          SummaryField {
              label: "Solution",
              value: transcript.solution.content.clone(),
              confidence: transcript.solution.confidence,
              on_edit: props.on_edit,
              field_id: "solution",
          }

          SummaryField {
              label: "Nonpersona",
              value: transcript.nonpersona.content.clone(),
              confidence: transcript.nonpersona.confidence,
              on_edit: props.on_edit,
              field_id: "nonpersona",
          }

          // Scenario is a special field with multiple parts
          div {
              class: "rounded-lg border border-border/50 bg-muted/20 p-4",
              div {
                  class: "flex items-start justify-between gap-3",
                  div {
                      class: "flex-1",
                      label {
                          class: "text-sm font-medium text-foreground",
                          "Scenario"
                      }
                      p {
                          class: "mt-1 text-sm text-muted-foreground",
                          "Trigger: {transcript.scenario.trigger}"
                      }
                      p {
                          class: "mt-1 text-sm text-muted-foreground",
                          "Value Moment: {transcript.scenario.value_moment}"
                      }
                      p {
                          class: "mt-1 text-sm text-muted-foreground",
                          "Feeling: {transcript.scenario.feeling}"
                      }
                  }
                  if props.on_edit.is_some() {
                      button {
                          class: "text-xs text-primary hover:text-primary/80",
                          onclick: {
                              let on_edit = props.on_edit;
                              move |_| {
                                  if let Some(handler) = on_edit.as_ref() {
                                      handler.call("scenario".to_string());
                                  }
                              }
                          },
                          "Edit"
                      }
                  }
              }
          }
      }
  }
}

/// Props for `SummaryField` component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct SummaryFieldProps {
  /// Field label
  pub label: String,
  /// Field value
  pub value: String,
  /// Confidence score (0.0-1.0)
  #[props(default = 0.0)]
  pub confidence: f64,
  /// Callback when edit is requested
  pub on_edit: Option<EventHandler<String>>,
  /// Field identifier for edit callback
  pub field_id: String,
}

/// `SummaryField` component
///
/// Displays a single field in the summary with optional edit capability.
#[component]
pub fn SummaryField(props: SummaryFieldProps) -> Element {
  let confidence_class = if props.confidence >= 0.7 {
    "bg-chart-2/10 text-chart-2"
  } else if props.confidence >= 0.4 {
    "bg-amber-500/10 text-amber-600 dark:text-amber-400"
  } else {
    "bg-chart-4/10 text-chart-4"
  };

  let confidence_label = if props.confidence >= 0.7 {
    "High"
  } else if props.confidence >= 0.4 {
    "Medium"
  } else {
    "Low"
  };

  rsx! {
      div {
          class: "rounded-lg border border-border/50 bg-muted/20 p-4",
          div {
              class: "flex items-start justify-between gap-3",
              div {
                  class: "flex-1",
                  div {
                      class: "flex items-center gap-2",
                      label {
                          class: "text-sm font-medium text-foreground",
                          "{props.label}"
                      }
                      span {
                          class: "inline-flex items-center rounded px-1.5 py-0.5 text-xs font-medium {confidence_class}",
                          "{confidence_label}"
                      }
                  }
                  p {
                      class: "mt-1 text-sm text-muted-foreground",
                      if props.value.is_empty() {
                          span {
                              class: "italic",
                              "Not specified"
                          }
                      } else {
                          "{props.value}"
                      }
                  }
              }
              if props.on_edit.is_some() {
                  button {
                      class: "text-xs text-primary hover:text-primary/80",
                      onclick: {
                          let on_edit = props.on_edit;
                          let field_id = props.field_id;
                          move |_| {
                              if let Some(handler) = on_edit.as_ref() {
                                  handler.call(field_id.clone());
                              }
                          }
                      },
                      "Edit"
                  }
              }
          }
      }
  }
}

// ============================================================================
// PreviewPhase Component (bd-3fz2)
// ============================================================================

/// Props for `PreviewPhase` component
#[derive(Clone, Props, PartialEq)]
pub struct PreviewPhaseProps {
  /// State signal
  pub state: Signal<ProgressiveDiscoverState>,
  /// Actions for state manipulation
  pub actions: ProgressiveDiscoverActions,
  /// Callback when user wants to refine (returns to Prompt phase)
  pub on_refine: Option<EventHandler<()>>,
  /// Callback when flow completes (proceeds to Kirk compilation)
  pub on_complete: Option<EventHandler<InterrogationTranscript>>,
}

/// `PreviewPhase` component (bd-3fz2)
///
/// The Preview phase is the final review before locking in the plan.
/// Users see all their confirmed data and must acknowledge the Four Brutal Truths.
///
/// # Features
///
/// - Displays summary of all 5 confirmed fields
/// - Shows Four Brutal Truths checklist
/// - "Refine" button returns to Prompt phase
/// - "Lock In" button proceeds to Kirk compilation
/// - Lock In only enabled when all truths acknowledged
#[component]
pub fn PreviewPhase(props: PreviewPhaseProps) -> Element {
  // Signal for brutal truths state
  let brutal_truths = use_signal(BrutalTruthsState::new);

  // Create transcript signal for PreviewSummary
  let transcript_signal = use_signal(|| {
    let state = props.state.read();
    state.transcript.clone()
  });

  // Check if all truths are acknowledged
  let all_acknowledged = brutal_truths.read().is_complete();

  // Handler for refine button
  let on_refine = {
    let mut actions = props.actions;
    let on_refine = props.on_refine;
    move |_| {
      // Go back to Prompt phase (3 phases back)
      actions.regress_phase();
      actions.regress_phase();
      actions.regress_phase();
      if let Some(handler) = on_refine {
        handler.call(());
      }
    }
  };

  // Handler for lock in button
  let on_lock_in = {
    let mut actions = props.actions;
    let on_complete = props.on_complete;
    let state = props.state;
    move |_| {
      // Advance to KirkCompilation
      actions.advance_phase();
      if let Some(handler) = on_complete {
        let transcript = state.read().transcript.clone();
        handler.call(transcript);
      }
    }
  };

  // Handler for field edit (returns to confirm phase)
  // This is available for future use when edit functionality is implemented
  let _on_field_edit = {
    let mut actions = props.actions;
    move |_field_id: String| {
      // Navigate to the appropriate confirm sub-phase
      // Future implementation will navigate to specific sub-phases
      actions.regress_phase();
    }
  };

  rsx! {
      div {
          class: "space-y-6 p-6 rounded-lg border border-border/50 bg-card shadow-sm",

          // Header
          div {
              class: "border-b border-border/50 pb-4",
              h2 {
                  class: "text-xl font-bold text-foreground",
                  "Review Your Plan"
              }
              p {
                  class: "text-sm text-muted-foreground mt-1",
                  "Review your confirmed artifacts and validate the Four Brutal Truths before locking."
              }
          }

          // Preview summary using existing component
          PreviewSummary {
              transcript: transcript_signal,
              on_change: None,
          }

          // Four Brutal Truths checklist (bd-2k1q)
          div {
              class: "rounded-lg border border-border/50 bg-card p-6",
              BrutalTruthsChecklist {
                  checked: brutal_truths,
                  enabled: true,
                  show_help: true,
                  show_status: true,
              }
          }

          // Action buttons
          div {
              class: "flex items-center justify-between border-t border-border/50 pt-4",

              // Refine button
              Button {
                  variant: ButtonVariant::Secondary,
                  onclick: on_refine,
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
                      path { d: "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" }
                      path { d: "M3 3v5h5" }
                  }
                  "Refine"
              }

              // Lock In button (only enabled when all truths acknowledged)
              Button {
                  variant: ButtonVariant::Primary,
                  disabled: !all_acknowledged,
                  onclick: on_lock_in,
                  if all_acknowledged {
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
                          rect {
                              x: "3",
                              y: "11",
                              width: "18",
                              height: "11",
                              rx: "2",
                              ry: "2",
                          }
                          path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
                      }
                  }
                  "Lock In"
                  if !all_acknowledged {
                      span {
                          class: "ml-2 text-xs opacity-70",
                          "(Acknowledge all truths)"
                      }
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
  fn test_summary_field_props_equality() {
    let props1 = SummaryFieldProps {
      label: "Test".to_string(),
      value: "Value".to_string(),
      confidence: 0.8,
      on_edit: None,
      field_id: "test".to_string(),
    };
    let props2 = SummaryFieldProps {
      label: "Test".to_string(),
      value: "Value".to_string(),
      confidence: 0.8,
      on_edit: None,
      field_id: "test".to_string(),
    };
    assert_eq!(props1, props2);
  }

  #[test]
  fn test_transcript_summary_props_equality() {
    let transcript = InterrogationTranscript::from_prompt("Test".to_string());
    let props1 = TranscriptSummaryProps {
      transcript: transcript.clone(),
      on_edit: None,
    };
    let props2 = TranscriptSummaryProps {
      transcript,
      on_edit: None,
    };
    assert_eq!(props1, props2);
  }

  #[test]
  fn test_brutal_truths_initial_state() {
    let state = BrutalTruthsState::new();
    assert!(!state.is_complete());
    assert_eq!(state.checked_count(), 0);
  }

  #[test]
  fn test_brutal_truths_all_checked() {
    let state = BrutalTruthsState::all_checked();
    assert!(state.is_complete());
    assert_eq!(state.checked_count(), 4);
  }
}
