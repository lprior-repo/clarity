#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

use super::quality_score::{QualityDimension, QualityScore, QualityScoreBar};
use super::types::{HolePunchingResults, HoleType};
use crate::ui::button::ButtonVariant;
use crate::ui::Button;

/// Props for ScenarioBulletInput component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ScenarioBulletInputProps {
  /// The trigger text (what triggers them to look for a solution?)
  pub trigger: Signal<String>,
  /// The value moment text (what's the key moment of value?)
  pub value_moment: Signal<String>,
  /// The feeling text (how do they feel after?)
  pub feeling: Signal<String>,
  /// Whether inputs are enabled
  #[props(default = true)]
  pub enabled: bool,
}

/// ScenarioBulletInput component
///
/// Displays three input fields for the North Star Scenario bullet prompts:
/// 1. Trigger: What triggers them to look for a solution?
/// 2. Value moment: What's the key moment of value?
/// 3. Feeling: How do they feel after?
#[component]
pub fn ScenarioBulletInput(props: ScenarioBulletInputProps) -> Element {
  let mut local_trigger = use_signal(|| props.trigger.read().clone());
  let mut local_value_moment = use_signal(|| props.value_moment.read().clone());
  let mut local_feeling = use_signal(|| props.feeling.read().clone());

  // Sync local values when external signals change
  use_effect({
    let trigger = props.trigger.clone();
    move || {
      let external = trigger.read().clone();
      let local = local_trigger.read().clone();
      if external != local {
        *local_trigger.write() = external;
      }
    }
  });

  use_effect({
    let value_moment = props.value_moment.clone();
    move || {
      let external = value_moment.read().clone();
      let local = local_value_moment.read().clone();
      if external != local {
        *local_value_moment.write() = external;
      }
    }
  });

  use_effect({
    let feeling = props.feeling.clone();
    move || {
      let external = feeling.read().clone();
      let local = local_feeling.read().clone();
      if external != local {
        *local_feeling.write() = external;
      }
    }
  });

  let update_trigger = {
    let mut trigger = props.trigger.clone();
    move |value: String| {
      *local_trigger.write() = value.clone();
      *trigger.write() = value;
    }
  };

  let update_value_moment = {
    let mut value_moment = props.value_moment.clone();
    move |value: String| {
      *local_value_moment.write() = value.clone();
      *value_moment.write() = value;
    }
  };

  let update_feeling = {
    let mut feeling = props.feeling.clone();
    move |value: String| {
      *local_feeling.write() = value.clone();
      *feeling.write() = value;
    }
  };

  rsx! {
      div {
          class: "space-y-3",
          label {
              class: "text-sm font-medium text-foreground",
              "Define your North Star Scenario:"
          }
          div {
              class: "space-y-3 rounded-lg border border-border/50 bg-muted/20 p-4",
              p {
                  class: "text-xs text-muted-foreground mb-3",
                  "Paint a complete picture of the user journey with these three prompts:"
              }

              // Trigger
              div {
                  class: "space-y-1",
                  div {
                      class: "flex items-center gap-2",
                      div {
                          class: "flex h-6 w-6 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary",
                          "1"
                      }
                      span {
                          class: "text-sm font-medium text-foreground",
                          "Trigger"
                      }
                  }
                  p {
                      class: "text-xs text-muted-foreground ml-8",
                      "What triggers them to look for a solution?"
                  }
                  input {
                      r#type: "text",
                      value: local_trigger.read().clone(),
                      disabled: !props.enabled,
                      placeholder: "e.g., User gets an error message they don't understand",
                      class: "ml-8 w-[calc(100%-2rem)] rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
                      oninput: {
                          let mut update_trigger = update_trigger.clone();
                          move |e: Event<FormData>| {
                              update_trigger(e.value());
                          }
                      },
                  }
              }

              // Value moment
              div {
                  class: "space-y-1",
                  div {
                      class: "flex items-center gap-2",
                      div {
                          class: "flex h-6 w-6 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary",
                          "2"
                      }
                      span {
                          class: "text-sm font-medium text-foreground",
                          "Value Moment"
                      }
                  }
                  p {
                      class: "text-xs text-muted-foreground ml-8",
                      "What's the key moment of value?"
                  }
                  input {
                      r#type: "text",
                      value: local_value_moment.read().clone(),
                      disabled: !props.enabled,
                      placeholder: "e.g., Problem is resolved instantly with one click",
                      class: "ml-8 w-[calc(100%-2rem)] rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
                      oninput: {
                          let mut update_value_moment = update_value_moment.clone();
                          move |e: Event<FormData>| {
                              update_value_moment(e.value());
                          }
                      },
                  }
              }

              // Feeling
              div {
                  class: "space-y-1",
                  div {
                      class: "flex items-center gap-2",
                      div {
                          class: "flex h-6 w-6 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary",
                          "3"
                      }
                      span {
                          class: "text-sm font-medium text-foreground",
                          "Feeling"
                      }
                  }
                  p {
                      class: "text-xs text-muted-foreground ml-8",
                      "How do they feel after?"
                  }
                  input {
                      r#type: "text",
                      value: local_feeling.read().clone(),
                      disabled: !props.enabled,
                      placeholder: "e.g., Relieved and confident in the product",
                      class: "ml-8 w-[calc(100%-2rem)] rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
                      oninput: {
                          let mut update_feeling = update_feeling.clone();
                          move |e: Event<FormData>| {
                              update_feeling(e.value());
                          }
                      },
                  }
              }
          }
      }
  }
}

/// Props for HolePunchingChecklist component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct HolePunchingChecklistProps {
  /// The hole punching results
  pub holes: Signal<HolePunchingResults>,
  /// Whether inputs are enabled
  #[props(default = true)]
  pub enabled: bool,
}

/// HolePunchingChecklist component
///
/// Displays three input fields for addressing scenario holes:
/// 1. Discovery Hole: How did they find the feature?
/// 2. Edge Case Hole: What if internet drops, mistype, etc?
/// 3. Motivation Drop-off: Why continue at high-friction steps?
#[component]
pub fn HolePunchingChecklist(props: HolePunchingChecklistProps) -> Element {
  let holes = props.holes;
  let mut local_holes = use_signal(|| holes.read().clone());

  // Sync local holes when external signal changes
  use_effect({
    let holes = holes.clone();
    move || {
      let external = holes.read().clone();
      let local = local_holes.read().clone();
      if external != local {
        *local_holes.write() = external;
      }
    }
  });

  let update_hole = {
    let mut holes = holes.clone();
    move |hole_type: HoleType, value: String| {
      let current = local_holes.read().clone();
      let new_holes = current.address(hole_type, value);
      *local_holes.write() = new_holes.clone();
      *holes.write() = new_holes;
    }
  };

  rsx! {
      div {
          class: "space-y-3",
          label {
              class: "text-sm font-medium text-foreground",
                  "Hole Punching Checklist:"
          }
          div {
              class: "space-y-3 rounded-lg border border-border/50 bg-muted/20 p-4",
              p {
                  class: "text-xs text-muted-foreground mb-3",
                  "Address these gaps in your scenario to make it bulletproof:"
              }

              for hole_type in HoleType::all() {
                  div {
                      class: "space-y-1",
                      div {
                          class: "flex items-center gap-2",
                          div {
                              class: format!(
                                      "flex h-5 w-5 items-center justify-center rounded-full {}",
                                      if local_holes.read().is_addressed(*hole_type) {
                                          "bg-emerald-500/20 text-emerald-500"
                                      } else {
                                          "bg-amber-500/20 text-amber-500"
                                      }
                                  ),
                                  if local_holes.read().is_addressed(*hole_type) {
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
                                  } else {
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
                                          circle { cx: "12", cy: "12", r: "10" }
                                          line { x1: "12", y1: "8", x2: "12", y2: "12" }
                                          line { x1: "12", y1: "16", x2: "12.01", y2: "16" }
                                      }
                                  }
                              }
                          span {
                              class: "text-sm font-medium text-foreground",
                              "{hole_type.label()}"
                          }
                          p {
                              class: "text-xs text-muted-foreground ml-7",
                              "{hole_type.description()}"
                          }
                          input {
                              r#type: "text",
                              value: local_holes.read().explanation(*hole_type).map_or(String::new(), |s| s.to_string()),
                              disabled: !props.enabled,
                              placeholder: "Explain how this is addressed...",
                              class: "ml-7 w-[calc(100%-1.75rem)] rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
                              oninput: {
                                  let mut update_hole = update_hole.clone();
                                  move |e: Event<FormData>| {
                                      update_hole(*hole_type, e.value());
                                  }
                              },
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Props for ScenarioQuality component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ScenarioQualityProps {
  /// The trigger text
  pub trigger: Signal<String>,
  /// The value moment text
  pub value_moment: Signal<String>,
  /// The feeling text
  pub feeling: Signal<String>,
  /// The hole punching results
  pub holes: Signal<HolePunchingResults>,
  /// Whether to show expanded details
  #[props(default = false)]
  pub expanded: bool,
}

/// ScenarioQuality component
///
/// Displays quality metrics for the scenario.
#[component]
pub fn ScenarioQuality(props: ScenarioQualityProps) -> Element {
  let trigger = props.trigger;
  let value_moment = props.value_moment;
  let feeling = props.feeling;
  let holes = props.holes;

  // Calculate quality dimensions based on scenario
  let quality_score = use_memo({
    let trigger = trigger.clone();
    let value_moment = value_moment.clone();
    let feeling = feeling.clone();
    let holes = holes.clone();
    move || {
      let trigger_text = trigger.read();
      let value_moment_text = value_moment.read();
      let feeling_text = feeling.read();
      let holes_data = holes.read();

      let bullets_score = calculate_bullets_score(&trigger_text, &value_moment_text, &feeling_text);
      let holes_score = calculate_holes_score(&holes_data);

      let overall = ((bullets_score as u16 + holes_score as u16) / 2) as u8;

      QualityScore::new(overall).with_dimensions(vec![
        QualityDimension::new("Scenario Bullets", bullets_score).with_issues(get_bullets_issues(
          &trigger_text,
          &value_moment_text,
          &feeling_text,
        )),
        QualityDimension::new("Hole Punching", holes_score)
          .with_issues(get_holes_issues(&holes_data)),
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

/// Calculate bullets score (0-100)
fn calculate_bullets_score(trigger: &str, value_moment: &str, feeling: &str) -> u8 {
  let scores = [
    calculate_single_bullet_score(trigger),
    calculate_single_bullet_score(value_moment),
    calculate_single_bullet_score(feeling),
  ];

  let sum: u16 = scores.iter().map(|&s| s as u16).sum();
  (sum / 3) as u8
}

/// Calculate score for a single bullet (0-100)
fn calculate_single_bullet_score(text: &str) -> u8 {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return 0;
  }

  let word_count = trimmed.split_whitespace().count();
  match word_count {
    0..=4 => 30,
    5..=9 => 50,
    10..=19 => 75,
    _ => 85,
  }
}

/// Calculate holes score (0-100)
fn calculate_holes_score(holes: &HolePunchingResults) -> u8 {
  let addressed = holes.addressed_count();
  match addressed {
    0 => 0,
    1 => 35,
    2 => 70,
    3 => 100,
    _ => 100,
  }
}

/// Get issues for scenario bullets
fn get_bullets_issues(trigger: &str, value_moment: &str, feeling: &str) -> Vec<String> {
  let mut issues = Vec::new();

  if trigger.trim().is_empty() {
    issues.push("Trigger is required".to_string());
  } else if trigger.split_whitespace().count() < 5 {
    issues.push("Trigger needs more detail".to_string());
  }

  if value_moment.trim().is_empty() {
    issues.push("Value moment is required".to_string());
  } else if value_moment.split_whitespace().count() < 5 {
    issues.push("Value moment needs more detail".to_string());
  }

  if feeling.trim().is_empty() {
    issues.push("Feeling is required".to_string());
  } else if feeling.split_whitespace().count() < 3 {
    issues.push("Feeling needs more detail".to_string());
  }

  issues
}

/// Get issues for hole punching
fn get_holes_issues(holes: &HolePunchingResults) -> Vec<String> {
  let unaddressed = holes.unaddressed_holes();
  unaddressed
    .iter()
    .map(|hole| format!("{}: {}", hole.label(), hole.description()))
    .collect()
}

/// Props for ScenarioConfirm component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ScenarioConfirmProps {
  /// The trigger text
  pub trigger: Signal<String>,
  /// The value moment text
  pub value_moment: Signal<String>,
  /// The feeling text
  pub feeling: Signal<String>,
  /// The hole punching results
  pub holes: Signal<HolePunchingResults>,
  /// Current step number (1-5)
  #[props(default = 5)]
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

/// ScenarioConfirm component
///
/// Composes:
/// - ScenarioBulletInput: Three bullet prompts for North Star Scenario
/// - HolePunchingChecklist: Address gaps in the scenario
/// - ScenarioQuality: Quality score indicator
/// - Navigation: Back/Next buttons
///
/// This is the fifth and final confirmation step in the Progressive Discover flow.
#[component]
pub fn ScenarioConfirm(props: ScenarioConfirmProps) -> Element {
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
                  "Scenario ({props.step}/{props.total_steps})"
              }
              span {
                  class: "text-sm text-muted-foreground",
                  "Define your North Star Scenario"
              }
          }

          // Scenario bullet inputs
          ScenarioBulletInput {
              trigger: props.trigger,
              value_moment: props.value_moment,
              feeling: props.feeling,
          }

          // Hole punching checklist
          HolePunchingChecklist {
              holes: props.holes,
          }

          // Quality indicator section (expandable)
          div {
              class: "cursor-pointer",
              onclick: toggle_quality,
              ScenarioQuality {
                  trigger: props.trigger,
                  value_moment: props.value_moment,
                  feeling: props.feeling,
                  holes: props.holes,
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
  fn test_calculate_single_bullet_score_empty() {
    let score = calculate_single_bullet_score("");
    assert_eq!(score, 0);
  }

  #[test]
  fn test_calculate_single_bullet_score_short() {
    let score = calculate_single_bullet_score("Short text");
    assert_eq!(score, 30);
  }

  #[test]
  fn test_calculate_single_bullet_score_medium() {
    let score = calculate_single_bullet_score("This is a medium length text");
    assert_eq!(score, 50);
  }

  #[test]
  fn test_calculate_single_bullet_score_long() {
    let score = calculate_single_bullet_score("This is a longer text with enough words to be good");
    assert_eq!(score, 75);
  }

  #[test]
  fn test_calculate_bullets_score_all_empty() {
    let score = calculate_bullets_score("", "", "");
    assert_eq!(score, 0);
  }

  #[test]
  fn test_calculate_bullets_score_all_filled() {
    let score = calculate_bullets_score(
      "User gets error message they don't understand",
      "Problem is resolved instantly with one click",
      "Relieved and confident in the product",
    );
    assert!(score >= 50);
  }

  #[test]
  fn test_calculate_holes_score_none() {
    let holes = HolePunchingResults::default();
    let score = calculate_holes_score(&holes);
    assert_eq!(score, 0);
  }

  #[test]
  fn test_calculate_holes_score_partial() {
    let holes =
      HolePunchingResults::new().address(HoleType::DiscoveryHole, "Found via search".to_string());
    let score = calculate_holes_score(&holes);
    assert_eq!(score, 35);
  }

  #[test]
  fn test_calculate_holes_score_complete() {
    let holes = HolePunchingResults::new()
      .address(HoleType::DiscoveryHole, "Found via search".to_string())
      .address(HoleType::EdgeCaseHole, "Handles offline".to_string())
      .address(
        HoleType::MotivationDropOff,
        "Progress indicator".to_string(),
      );
    let score = calculate_holes_score(&holes);
    assert_eq!(score, 100);
  }

  #[test]
  fn test_get_bullets_issues_all_empty() {
    let issues = get_bullets_issues("", "", "");
    assert_eq!(issues.len(), 3);
  }

  #[test]
  fn test_get_bullets_issues_all_filled() {
    let issues = get_bullets_issues(
      "This is a detailed trigger description",
      "This is a detailed value moment",
      "This is a detailed feeling",
    );
    assert!(issues.is_empty());
  }

  #[test]
  fn test_get_holes_issues_none() {
    let holes = HolePunchingResults::default();
    let issues = get_holes_issues(&holes);
    assert_eq!(issues.len(), 3);
  }

  #[test]
  fn test_get_holes_issues_complete() {
    let holes = HolePunchingResults::new()
      .address(HoleType::DiscoveryHole, "Found via search".to_string())
      .address(HoleType::EdgeCaseHole, "Handles offline".to_string())
      .address(
        HoleType::MotivationDropOff,
        "Progress indicator".to_string(),
      );
    let issues = get_holes_issues(&holes);
    assert!(issues.is_empty());
  }
}
