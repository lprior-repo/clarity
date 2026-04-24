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

//! Preview Summary component for displaying all extracted artifacts.
//!
//! This component renders a comprehensive summary of the interrogation transcript,
//! displaying all extracted fields in organized categories with edit capabilities.

use dioxus::prelude::*;

use super::field_card::{Confidence, FieldCard, FieldData};
use super::types::ScenarioField;
use crate::storage::transcript_store::{
  AntithesisResponse, ExtractedField, InterrogationTranscript, StrawManValidation,
};

/// Props for `PreviewSummary` component
#[derive(Clone, Props, PartialEq)]
pub struct PreviewSummaryProps {
  /// The interrogation transcript to display
  pub transcript: Signal<InterrogationTranscript>,
  /// Callback when a field is edited
  pub on_change: Option<EventHandler<InterrogationTranscript>>,
}

/// Preview Summary component
///
/// Displays all extracted fields from the interrogation transcript in a
/// comprehensive summary format with editable fields.
///
/// # Fields Displayed
///
/// - Problem statement
/// - Antithesis points (3 null hypothesis points)
/// - Solution description
/// - Target persona (User)
/// - Nonpersona (who it's NOT for)
/// - Scenario (trigger, value moment, feeling)
/// - VORP justification
#[component]
pub fn PreviewSummary(props: PreviewSummaryProps) -> Element {
  let transcript = props.transcript;

  // Create signals for each editable field
  let problem_field = use_signal(|| {
    let t = transcript.read();
    field_from_extracted("problem", "Problem", &t.problem)
  });

  let solution_field = use_signal(|| {
    let t = transcript.read();
    field_from_extracted("solution", "Solution", &t.solution)
  });

  let persona_field = use_signal(|| {
    let t = transcript.read();
    field_from_extracted("persona", "Target User", &t.persona)
  });

  let nonpersona_field = use_signal(|| {
    let t = transcript.read();
    field_from_extracted("nonpersona", "Nonpersona", &t.nonpersona)
  });

  let vorp_field = use_signal(|| {
    let t = transcript.read();
    FieldData {
      id: "vorp".to_string(),
      title: "VORP Justification".to_string(),
      content: t.vorp_justification.clone(),
      confidence: Confidence::Medium,
      locked: false,
    }
  });

  // Read current values for display
  let antithesis_points = {
    let t = transcript.read();
    t.antithesis.points.clone()
  };

  let scenario = {
    let t = transcript.read();
    t.scenario.clone()
  };

  let straw_man_passed = {
    let t = transcript.read();
    t.straw_man_validation.passed
  };

  rsx! {
      div {
          class: "space-y-6",

          // Header
          div {
              class: "border-b border-border/50 pb-4",
              h2 {
                  class: "text-xl font-semibold text-foreground",
                  "Plan Summary"
              }
              p {
                  class: "text-sm text-muted-foreground mt-1",
                  "Review and edit your extracted artifacts before locking."
              }
          }

          // Problem Section
          section {
              class: "space-y-4",
              h3 {
                  class: "text-sm font-medium text-muted-foreground uppercase tracking-wide",
                  "Problem & Antithesis"
              }
              FieldCard {
                  field: problem_field,
                  on_edit: None,
              }

              // Antithesis Points (read-only display)
              div {
                  class: "rounded-lg border border-border/50 bg-muted/20 p-4",
                  div {
                      class: "flex items-center justify-between mb-3",
                      h4 {
                          class: "text-sm font-medium text-foreground",
                          "Antithesis Points"
                      }
                      span {
                          class: format!(
                              "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium {}",
                              if antithesis_points.iter().all(|p| !p.trim().is_empty()) {
                                  "bg-chart-2/10 text-chart-2 border-chart-2/20"
                              } else {
                                  "bg-chart-4/10 text-chart-4 border-chart-4/20"
                              }
                          ),
                          if antithesis_points.iter().all(|p| !p.trim().is_empty()) {
                              "Complete"
                          } else {
                              "Incomplete"
                          }
                      }
                  }
                  ol {
                      class: "space-y-2 list-decimal list-inside",
                      for point in antithesis_points.iter() {
                          li {
                              class: "text-sm text-foreground/80",
                              if point.trim().is_empty() {
                                  span {
                                      class: "italic text-muted-foreground/50",
                                      "No point provided"
                                  }
                              } else {
                                  "{point}"
                              }
                          }
                      }
                  }
              }
          }

          // Solution Section
          section {
              class: "space-y-4",
              h3 {
                  class: "text-sm font-medium text-muted-foreground uppercase tracking-wide",
                  "Solution"
              }
              FieldCard {
                  field: solution_field,
                  on_edit: None,
              }

              // VORP Justification
              FieldCard {
                  field: vorp_field,
                  on_edit: None,
              }
          }

          // User Section
          section {
              class: "space-y-4",
              h3 {
                  class: "text-sm font-medium text-muted-foreground uppercase tracking-wide",
                  "Target User"
              }
              FieldCard {
                  field: persona_field,
                  on_edit: None,
              }

              // Straw Man Validation Badge
              div {
                  class: "flex items-center gap-2 text-sm",
                  span {
                      class: "text-muted-foreground",
                      "Straw Man Validation:"
                  }
                  span {
                      class: format!(
                          "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium {}",
                          if straw_man_passed {
                              "bg-chart-2/10 text-chart-2 border-chart-2/20"
                          } else {
                              "bg-destructive/10 text-destructive border-destructive/20"
                          }
                      ),
                      if straw_man_passed {
                          "Passed"
                      } else {
                          "Traps Detected"
                      }
                  }
              }

              FieldCard {
                  field: nonpersona_field,
                  on_edit: None,
              }
          }

          // Scenario Section
          section {
              class: "space-y-4",
              h3 {
                  class: "text-sm font-medium text-muted-foreground uppercase tracking-wide",
                  "North Star Scenario"
              }

              div {
                  class: "rounded-lg border border-border/50 bg-card overflow-hidden",

                  // Scenario header
                  div {
                      class: "border-b border-border/50 bg-muted/30 px-4 py-3",
                      div {
                          class: "flex items-center justify-between",
                          h4 {
                              class: "text-sm font-semibold text-foreground",
                              "Scenario"
                          }
                          span {
                              class: format!(
                                  "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium {}",
                                  if scenario.is_bullets_complete() {
                                      "bg-chart-2/10 text-chart-2 border-chart-2/20"
                                  } else {
                                      "bg-chart-4/10 text-chart-4 border-chart-4/20"
                                  }
                              ),
                              if scenario.is_bullets_complete() {
                                  "Complete"
                              } else {
                                  "Incomplete"
                              }
                          }
                      }
                  }

                  // Scenario bullets
                  div {
                      class: "p-4 space-y-4",

                      // Trigger
                      div {
                          class: "space-y-1",
                          label {
                              class: "text-xs font-medium text-muted-foreground",
                              "Trigger"
                          }
                          p {
                              class: "text-sm text-foreground/80",
                              if scenario.trigger.trim().is_empty() {
                                  span {
                                      class: "italic text-muted-foreground/50",
                                      "Not specified"
                                  }
                              } else {
                                  "{scenario.trigger}"
                              }
                          }
                      }

                      // Value Moment
                      div {
                          class: "space-y-1",
                          label {
                              class: "text-xs font-medium text-muted-foreground",
                              "Value Moment"
                          }
                          p {
                              class: "text-sm text-foreground/80",
                              if scenario.value_moment.trim().is_empty() {
                                  span {
                                      class: "italic text-muted-foreground/50",
                                      "Not specified"
                                  }
                              } else {
                                  "{scenario.value_moment}"
                              }
                          }
                      }

                      // Feeling
                      div {
                          class: "space-y-1",
                          label {
                              class: "text-xs font-medium text-muted-foreground",
                              "Feeling"
                          }
                          p {
                              class: "text-sm text-foreground/80",
                              if scenario.feeling.trim().is_empty() {
                                  span {
                                      class: "italic text-muted-foreground/50",
                                      "Not specified"
                                  }
                              } else {
                                  "{scenario.feeling}"
                              }
                          }
                      }
                  }

                  // Hole Punching Status
                  div {
                      class: "border-t border-border/50 bg-muted/20 px-4 py-3",
                      h5 {
                          class: "text-xs font-medium text-muted-foreground mb-2",
                          "Hole Punching Status"
                      }
                      div {
                          class: "flex flex-wrap gap-2",
                          HoleStatusBadge {
                              label: "Discovery",
                              addressed: scenario.hole_punching.discovery_hole.is_some(),
                          }
                          HoleStatusBadge {
                              label: "Edge Case",
                              addressed: scenario.hole_punching.edge_case_hole.is_some(),
                          }
                          HoleStatusBadge {
                              label: "Motivation",
                              addressed: scenario.hole_punching.motivation_dropoff.is_some(),
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Convert an `ExtractedField` to `FieldData` for display
fn field_from_extracted(id: &str, title: &str, field: &ExtractedField) -> FieldData {
  FieldData {
    id: id.to_string(),
    title: title.to_string(),
    content: field.content.clone(),
    confidence: confidence_from_score(field.confidence),
    locked: false,
  }
}

/// Convert a confidence score (0.0-1.0) to Confidence enum
fn confidence_from_score(score: f64) -> Confidence {
  match score {
    s if s >= 0.7 => Confidence::High,
    s if s >= 0.4 => Confidence::Medium,
    _ => Confidence::Low,
  }
}

/// Props for `HoleStatusBadge` component
#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct HoleStatusBadgeProps {
  /// Label for the badge
  pub label: String,
  /// Whether the hole has been addressed
  pub addressed: bool,
}

/// Badge component showing hole punching status
#[component]
pub fn HoleStatusBadge(props: HoleStatusBadgeProps) -> Element {
  let addressed = props.addressed;
  let label = props.label;

  let (icon, classes) = if addressed {
    (
      // Check icon
      rsx! {
          svg {
              xmlns: "http://www.w3.org/2000/svg",
              width: "12",
              height: "12",
              view_box: "0 0 24 24",
              fill: "none",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
              stroke_linejoin: "round",
              polyline { points: "20 6 9 17 4 12" }
          }
      },
      "bg-chart-2/10 text-chart-2 border-chart-2/20",
    )
  } else {
    (
      // X icon
      rsx! {
          svg {
              xmlns: "http://www.w3.org/2000/svg",
              width: "12",
              height: "12",
              view_box: "0 0 24 24",
              fill: "none",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
              stroke_linejoin: "round",
              line { x1: "18", y1: "6", x2: "6", y2: "18" }
              line { x1: "6", y1: "6", x2: "18", y2: "18" }
          }
      },
      "bg-muted/50 text-muted-foreground border-border/50",
    )
  };

  rsx! {
      span {
          class: format!(
              "inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs font-medium {}",
              classes
          ),
          {icon}
          "{label}"
      }
  }
}

/// Create a sample transcript for testing/preview
#[must_use]
pub fn sample_transcript() -> InterrogationTranscript {
  let mut transcript = InterrogationTranscript::from_prompt(
    "I want to build a meditation app for busy professionals".to_string(),
  );

  transcript.problem = ExtractedField::new(
        "Busy professionals struggle to maintain meditation habits due to lack of time and inconsistent schedules".to_string(),
        0.85,
        "ai".to_string(),
    );

  transcript.antithesis = AntithesisResponse::new(
    "They already tried Calm and Headspace and abandoned them".to_string(),
    "5 minutes still feels like too much time during busy days".to_string(),
    "They don't believe meditation actually helps their productivity".to_string(),
    0.85,
  );

  transcript.persona = ExtractedField::new(
    "Tech workers aged 25-40 who work long hours and feel burned out".to_string(),
    0.9,
    "ai".to_string(),
  );

  transcript.solution = ExtractedField::new(
    "Micro-meditation app with 60-second sessions and smart reminders based on calendar"
      .to_string(),
    0.8,
    "ai".to_string(),
  );

  transcript.vorp_justification = "Unlike existing apps, we integrate with their calendar to find genuine micro-moments, and our sessions are truly 60 seconds without preamble.".to_string();

  transcript.nonpersona = ExtractedField::new(
        "People who are already meditation practitioners, or those looking for spiritual/religious meditation content".to_string(),
        0.75,
        "ai".to_string(),
    );

  transcript.scenario = ScenarioField {
    trigger: "User receives a notification 5 minutes before their next meeting".to_string(),
    value_moment: "Completes a 60-second breathing exercise that actually calms their nerves"
      .to_string(),
    feeling: "Relieved and more focused for their upcoming meeting".to_string(),
    hole_punching: super::types::HolePunchingResults::new()
      .address(
        super::types::HoleType::DiscoveryHole,
        "App Store search and productivity blogs".to_string(),
      )
      .address(
        super::types::HoleType::EdgeCaseHole,
        "Offline mode with cached sessions".to_string(),
      )
      .address(
        super::types::HoleType::MotivationDropOff,
        "Streak counter and gentle nudges".to_string(),
      ),
  };

  transcript.straw_man_validation = StrawManValidation::passing();

  transcript
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
  fn test_confidence_from_score_high() {
    assert_eq!(confidence_from_score(0.9), Confidence::High);
    assert_eq!(confidence_from_score(0.7), Confidence::High);
    assert_eq!(confidence_from_score(0.85), Confidence::High);
  }

  #[test]
  fn test_confidence_from_score_medium() {
    assert_eq!(confidence_from_score(0.4), Confidence::Medium);
    assert_eq!(confidence_from_score(0.5), Confidence::Medium);
    assert_eq!(confidence_from_score(0.69), Confidence::Medium);
  }

  #[test]
  fn test_confidence_from_score_low() {
    assert_eq!(confidence_from_score(0.0), Confidence::Low);
    assert_eq!(confidence_from_score(0.1), Confidence::Low);
    assert_eq!(confidence_from_score(0.39), Confidence::Low);
  }

  #[test]
  fn test_field_from_extracted() {
    let field = ExtractedField::new("Test content".to_string(), 0.8, "ai".to_string());

    let field_data = field_from_extracted("test", "Test Title", &field);

    assert_eq!(field_data.id, "test");
    assert_eq!(field_data.title, "Test Title");
    assert_eq!(field_data.content, "Test content");
    assert_eq!(field_data.confidence, Confidence::High);
    assert!(!field_data.locked);
  }

  #[test]
  fn test_sample_transcript_is_valid() {
    let transcript = sample_transcript();

    assert!(!transcript.problem.content.is_empty());
    assert_eq!(transcript.antithesis.points.len(), 3);
    assert!(!transcript.persona.content.is_empty());
    assert!(!transcript.solution.content.is_empty());
    assert!(!transcript.vorp_justification.is_empty());
    assert!(!transcript.nonpersona.content.is_empty());
    assert!(!transcript.scenario.trigger.is_empty());
    assert!(transcript.straw_man_validation.passed);
  }

  #[test]
  fn test_sample_transcript_scenario_complete() {
    let transcript = sample_transcript();
    assert!(transcript.scenario.is_bullets_complete());
    assert!(transcript.scenario.hole_punching.is_complete());
  }

  #[test]
  fn test_hole_status_badge_props_equality() {
    let props1 = HoleStatusBadgeProps {
      label: "Test".to_string(),
      addressed: true,
    };
    let props2 = HoleStatusBadgeProps {
      label: "Test".to_string(),
      addressed: true,
    };
    assert_eq!(props1, props2);
  }
}
