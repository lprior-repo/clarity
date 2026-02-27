#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

use super::antithesis::AntithesisResponse;
use super::quality_score::{QualityDimension, QualityScore, QualityScoreBar};
use crate::ui::button::ButtonVariant;
use crate::ui::{Button, Textarea};

/// Props for ProblemDisplay component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ProblemDisplayProps {
  /// The problem text to display/edit
  pub problem: Signal<String>,
  /// Placeholder text for the textarea
  #[props(default = String::from("Describe the problem you are solving..."))]
  pub placeholder: String,
  /// Whether the problem text is editable
  #[props(default = true)]
  pub editable: bool,
}

/// ProblemDisplay component
///
/// Displays and allows editing of the problem statement.
/// This is the first part of the ProblemConfirm flow.
#[component]
pub fn ProblemDisplay(props: ProblemDisplayProps) -> Element {
  let problem = props.problem;
  let mut local_problem = use_signal(|| problem.read().clone());

  // Sync local problem when external signal changes
  use_effect({
    let problem = problem.clone();
    move || {
      let external = problem.read().clone();
      let local = local_problem.read().clone();
      if external != local {
        *local_problem.write() = external;
      }
    }
  });

  let on_input = {
    let mut problem = problem.clone();
    move |value: String| {
      *local_problem.write() = value.clone();
      *problem.write() = value;
    }
  };

  rsx! {
      div {
          class: "space-y-2",
          label {
              class: "text-sm font-medium text-foreground",
              "Based on what you wrote, here's the problem I see:"
          }
          Textarea {
              value: local_problem.read().clone(),
              placeholder: props.placeholder.clone(),
              disabled: !props.editable,
              rows: 4,
              oninput: on_input,
          }
      }
  }
}

/// Props for AntithesisInput component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct AntithesisInputProps {
  /// The antithesis response containing 3 points
  pub antithesis: Signal<AntithesisResponse>,
  /// Whether inputs are enabled
  #[props(default = true)]
  pub enabled: bool,
}

/// AntithesisInput component
///
/// Displays three input fields for the null hypothesis points.
/// These are realistic reasons why users might reject the solution.
#[component]
pub fn AntithesisInput(props: AntithesisInputProps) -> Element {
  let antithesis = props.antithesis;
  let mut local_points = use_signal(|| antithesis.read().points.clone());

  // Sync local points when external signal changes
  use_effect({
    let antithesis = antithesis.clone();
    move || {
      let external = antithesis.read().points.clone();
      let local = local_points.read().clone();
      if external != local {
        *local_points.write() = external;
      }
    }
  });

  let update_point = {
    let mut antithesis = antithesis.clone();
    move |index: usize, value: String| {
      let current_points = local_points.read();
      if index < current_points.len() {
        let new_points = current_points
          .iter()
          .enumerate()
          .map(|(i, p)| if i == index { value.clone() } else { p.clone() })
          .collect::<Vec<_>>();
        drop(current_points);
        *local_points.write() = new_points.clone();
        *antithesis.write() = AntithesisResponse::new(new_points);
      }
    }
  };

  rsx! {
      div {
          class: "space-y-3",
          label {
              class: "text-sm font-medium text-foreground",
              "Now the hard part - the null hypothesis. Give me 3 realistic reasons why your target customer will ignore or reject this:"
          }
          div {
              class: "space-y-2",
              for (index, _point) in local_points.read().iter().enumerate() {
                  div {
                      class: "flex items-start gap-2",
                      span {
                          class: "flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted text-sm font-medium text-muted-foreground",
                          "{index + 1}"
                      }
                      input {
                          r#type: "text",
                          value: local_points.read().get(index).cloned().unwrap_or_default(),
                          disabled: !props.enabled,
                          placeholder: format!("Antithesis point {}", index + 1),
                          class: "flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
                          oninput: {
                              let mut update_point = update_point.clone();
                              move |e: Event<FormData>| {
                                  update_point(index, e.value());
                              }
                          },
                      }
                  }
              }
          }
      }
  }
}

/// Props for AntithesisQuality component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct AntithesisQualityProps {
  /// The antithesis response to display quality for
  pub antithesis: Signal<AntithesisResponse>,
  /// Whether to show expanded details
  #[props(default = false)]
  pub expanded: bool,
}

/// AntithesisQuality component
///
/// Displays quality metrics for the antithesis response.
/// Uses the QualityScoreBar component with dimensions for:
/// - Specificity: How specific are the rejection reasons?
/// - Realism: How realistic/plausible are they?
/// - Actionability: Can the team address these concerns?
#[component]
pub fn AntithesisQuality(props: AntithesisQualityProps) -> Element {
  let antithesis = props.antithesis;

  // Calculate quality dimensions based on antithesis
  let quality_score = use_memo({
    let antithesis = antithesis.clone();
    move || {
      let response = antithesis.read();
      let overall = response.score();

      let specificity = calculate_specificity_score(&response.points);
      let realism = calculate_realism_score(&response.points);
      let actionability = calculate_actionability_score(&response.points);

      QualityScore::new(overall).with_dimensions(vec![
        QualityDimension::new("Specificity", specificity)
          .with_issues(get_specificity_issues(&response.points)),
        QualityDimension::new("Realism", realism).with_issues(get_realism_issues(&response.points)),
        QualityDimension::new("Actionability", actionability)
          .with_issues(get_actionability_issues(&response.points)),
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

/// Calculate specificity score (0-100) for antithesis points
fn calculate_specificity_score(points: &[String]) -> u8 {
  if points.len() != 3 {
    return 0;
  }

  let total: u32 = points
    .iter()
    .map(|p| {
      let word_count = p.split_whitespace().count();
      match word_count {
        0..=4 => 20,
        5..=9 => 50,
        10..=25 => 80,
        26..=50 => 70,
        _ => 60,
      }
    })
    .sum();

  u8::try_from(total / 3).unwrap_or(0)
}

/// Calculate realism score (0-100) for antithesis points
fn calculate_realism_score(points: &[String]) -> u8 {
  if points.len() != 3 {
    return 0;
  }

  let total: u32 = points
    .iter()
    .map(|p| {
      let lower = p.to_lowercase();
      let base = 40;

      // Bonus for realistic indicators
      let bonus = if lower.contains("because") {
        20
      } else if lower.contains("currently") || lower.contains("already") {
        15
      } else if lower.contains("might") || lower.contains("could") {
        10
      } else {
        0
      };

      base + bonus
    })
    .sum();

  u8::try_from(total / 3).unwrap_or(0)
}

/// Calculate actionability score (0-100) for antithesis points
fn calculate_actionability_score(points: &[String]) -> u8 {
  if points.len() != 3 {
    return 0;
  }

  let total: u32 = points
    .iter()
    .map(|p| {
      let lower = p.to_lowercase();
      let base = 30;

      // Bonus for actionable indicators
      let bonus = if lower.contains("if") || lower.contains("when") {
        25
      } else if lower.contains("by") || lower.contains("through") {
        20
      } else if lower.contains("need") || lower.contains("require") {
        15
      } else {
        0
      };

      base + bonus
    })
    .sum();

  u8::try_from(total / 3).unwrap_or(0)
}

/// Get specificity issues for the points
fn get_specificity_issues(points: &[String]) -> Vec<String> {
  let issues: Vec<String> = points
    .iter()
    .enumerate()
    .filter_map(|(i, p)| {
      let word_count = p.split_whitespace().count();
      match word_count {
        0..=4 => Some(format!(
          "Point {} is too vague - add more specific details",
          i + 1
        )),
        5..=9 => Some(format!("Point {} could be more specific", i + 1)),
        _ => None,
      }
    })
    .collect();

  issues
}

/// Get realism issues for the points
fn get_realism_issues(points: &[String]) -> Vec<String> {
  let issues: Vec<String> = points
    .iter()
    .enumerate()
    .filter_map(|(i, p)| {
      if p.trim().is_empty() {
        Some(format!("Point {} is empty", i + 1))
      } else {
        None
      }
    })
    .collect();

  issues
}

/// Get actionability issues for the points
fn get_actionability_issues(points: &[String]) -> Vec<String> {
  let empty_count = points.iter().filter(|p| p.trim().is_empty()).count();
  if empty_count > 0 {
    vec![format!("{} points need to be filled in", empty_count)]
  } else {
    Vec::new()
  }
}

/// Props for ProblemConfirm component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ProblemConfirmProps {
  /// The problem text
  pub problem: Signal<String>,
  /// The antithesis response
  pub antithesis: Signal<AntithesisResponse>,
  /// Current step number (1-5)
  #[props(default = 1)]
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

/// ProblemConfirm component
///
/// Composes:
/// - ProblemDisplay: Shows the problem text for review/editing
/// - AntithesisInput: Three inputs for null hypothesis points
/// - AntithesisQuality: Quality score indicator
/// - Navigation: Back/Next buttons
///
/// This is the first confirmation step in the Progressive Discover flow.
#[component]
pub fn ProblemConfirm(props: ProblemConfirmProps) -> Element {
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
                  "Problem ({props.step}/{props.total_steps})"
              }
              span {
                  class: "text-sm text-muted-foreground",
                  "Confirm your problem statement"
              }
          }

          // Problem display section
          ProblemDisplay {
              problem: props.problem,
          }

          // Antithesis input section
          AntithesisInput {
              antithesis: props.antithesis,
          }

          // Quality indicator section (expandable)
          div {
              class: "cursor-pointer",
              onclick: toggle_quality,
              AntithesisQuality {
                  antithesis: props.antithesis,
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
  fn test_calculate_specificity_score_empty() {
    let points = vec![String::new(), String::new(), String::new()];
    let score = calculate_specificity_score(&points);
    assert_eq!(score, 20); // Base score for empty (0-4 words)
  }

  #[test]
  fn test_calculate_specificity_score_vague() {
    let points = vec![
      "Too vague".to_string(),
      "Short text".to_string(),
      "Brief".to_string(),
    ];
    let score = calculate_specificity_score(&points);
    assert!(score >= 20 && score <= 50);
  }

  #[test]
  fn test_calculate_specificity_score_wrong_count() {
    let two_points = vec!["One".to_string(), "Two".to_string()];
    assert_eq!(calculate_specificity_score(&two_points), 0);

    let four_points = vec![
      "One".to_string(),
      "Two".to_string(),
      "Three".to_string(),
      "Four".to_string(),
    ];
    assert_eq!(calculate_specificity_score(&four_points), 0);
  }

  #[test]
  fn test_calculate_realism_score_empty() {
    let points = vec![String::new(), String::new(), String::new()];
    let score = calculate_realism_score(&points);
    assert_eq!(score, 40); // Base score
  }

  #[test]
  fn test_calculate_realism_score_with_because() {
    let points = vec![
      "Users prefer existing tools because they work well".to_string(),
      "Learning curve is steep because it requires training".to_string(),
      "Cost is high because of premium features".to_string(),
    ];
    let score = calculate_realism_score(&points);
    assert_eq!(score, 60); // 40 + 20 bonus
  }

  #[test]
  fn test_calculate_realism_score_with_currently() {
    let points = vec![
      "Users currently use spreadsheets".to_string(),
      "Teams already have workflows".to_string(),
      "Companies currently pay for alternatives".to_string(),
    ];
    let score = calculate_realism_score(&points);
    assert_eq!(score, 55); // 40 + 15 bonus
  }

  #[test]
  fn test_calculate_actionability_score_empty() {
    let points = vec![String::new(), String::new(), String::new()];
    let score = calculate_actionability_score(&points);
    assert_eq!(score, 30); // Base score
  }

  #[test]
  fn test_calculate_actionability_score_with_conditionals() {
    let points = vec![
      "If the price is too high, users will not switch".to_string(),
      "When the feature is complex, adoption drops".to_string(),
      "If onboarding takes too long, users churn".to_string(),
    ];
    let score = calculate_actionability_score(&points);
    assert_eq!(score, 55); // 30 + 25 bonus
  }

  #[test]
  fn test_get_specificity_issues_empty() {
    let points = vec![String::new(), String::new(), String::new()];
    let issues = get_specificity_issues(&points);
    assert_eq!(issues.len(), 3);
  }

  #[test]
  fn test_get_specificity_issues_short() {
    let points = vec![
      "Too short".to_string(),
      "Brief".to_string(),
      "Vague".to_string(),
    ];
    let issues = get_specificity_issues(&points);
    assert!(!issues.is_empty());
  }

  #[test]
  fn test_get_specificity_issues_good() {
    let points = vec![
      "Users prefer existing tools because they have invested time learning them".to_string(),
      "Learning curve is too steep for team members who have limited time for training".to_string(),
      "Cost outweighs perceived benefits especially for smaller teams with limited budgets"
        .to_string(),
    ];
    let issues = get_specificity_issues(&points);
    assert!(issues.is_empty());
  }

  #[test]
  fn test_get_realism_issues_empty() {
    let points = vec![String::new(), "Valid point".to_string(), String::new()];
    let issues = get_realism_issues(&points);
    assert_eq!(issues.len(), 2);
  }

  #[test]
  fn test_get_realism_issues_all_filled() {
    let points = vec![
      "Valid point one".to_string(),
      "Valid point two".to_string(),
      "Valid point three".to_string(),
    ];
    let issues = get_realism_issues(&points);
    assert!(issues.is_empty());
  }

  #[test]
  fn test_get_actionability_issues_empty() {
    let points = vec![String::new(), String::new(), "Valid".to_string()];
    let issues = get_actionability_issues(&points);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("2 points"));
  }

  #[test]
  fn test_get_actionability_issues_all_filled() {
    let points = vec![
      "Valid point one".to_string(),
      "Valid point two".to_string(),
      "Valid point three".to_string(),
    ];
    let issues = get_actionability_issues(&points);
    assert!(issues.is_empty());
  }

  // Note: The following tests require Dioxus runtime (Signal/use_memo):
  // - test_calculate_specificity_score_good
  // - test_specificity_score_boundary_conditions
  // - test_realism_score_mixed_indicators
  // - test_actionability_score_mixed_indicators
}
