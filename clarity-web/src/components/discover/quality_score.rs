#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

/// Quality score threshold constants
const QUALITY_GATE_THRESHOLD: u8 = 70;

/// Color classes based on score
fn score_color_classes(score: u8) -> &'static str {
  match score {
    70..=100 => "bg-emerald-500/60",
    50..=69 => "bg-amber-500/60",
    _ => "bg-red-500/60",
  }
}

/// Text color based on score
fn score_text_color_classes(score: u8) -> &'static str {
  match score {
    70..=100 => "text-emerald-400",
    50..=69 => "text-amber-400",
    _ => "text-red-400",
  }
}

/// Ring color based on score
fn score_ring_classes(score: u8) -> &'static str {
  match score {
    70..=100 => "ring-emerald-500/30",
    50..=69 => "ring-amber-500/30",
    _ => "ring-red-500/30",
  }
}

/// Quality dimension scores
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualityDimension {
  pub name: String,
  pub score: u8,
  pub issues: Vec<String>,
}

impl QualityDimension {
  /// Create a new quality dimension
  pub fn new(name: impl Into<String>, score: u8) -> Self {
    Self {
      name: name.into(),
      score,
      issues: Vec::new(),
    }
  }

  /// Add issues to the dimension
  pub fn with_issues(mut self, issues: Vec<String>) -> Self {
    self.issues = issues;
    self
  }
}

/// Overall quality assessment
#[derive(Clone, Debug, PartialEq)]
pub struct QualityScore {
  pub overall: u8,
  pub dimensions: Vec<QualityDimension>,
}

impl QualityScore {
  /// Create a new quality score with default dimensions
  pub fn new(overall: u8) -> Self {
    Self {
      overall,
      dimensions: Vec::new(),
    }
  }

  /// Create a new quality score with dimensions
  pub fn with_dimensions(mut self, dimensions: Vec<QualityDimension>) -> Self {
    self.dimensions = dimensions;
    self
  }

  /// Check if quality gate passes
  pub fn gate_passes(&self) -> bool {
    self.overall >= QUALITY_GATE_THRESHOLD
  }

  /// Get gate status message
  pub fn gate_message(&self) -> String {
    if self.gate_passes() {
      "Quality gate: PASS".to_string()
    } else {
      format!(
        "Quality gate: FAIL (need {QUALITY_GATE_THRESHOLD}, have {})",
        self.overall
      )
    }
  }
}

impl Default for QualityScore {
  fn default() -> Self {
    Self {
      overall: 0,
      dimensions: Vec::new(),
    }
  }
}

/// Props for QualityScoreBar component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct QualityScoreBarProps {
  /// Quality score data (accepts Memo for computed scores)
  pub score: Memo<QualityScore>,
  /// Initially expanded state
  pub expanded: bool,
}

/// Quality Score Bar component
///
/// Displays:
/// - Overall score (large, color-coded)
/// - 5 dimension bars (mini)
/// - Gate status indicator
/// - Color coding: >=70 green, 50-69 yellow, <50 red
/// - Expandable dimension breakdown
/// - Issues list as tooltips
#[component]
pub fn QualityScoreBar(props: QualityScoreBarProps) -> Element {
  let score = props.score;
  let mut expanded = use_signal(|| props.expanded);

  let on_toggle = {
    move |_| {
      expanded.toggle();
    }
  };

  rsx! {
      div {
          class: "rounded-lg border border-border/50 bg-card shadow-sm overflow-hidden",

          // Main score display
          div {
              class: "p-6",
              div {
                  class: "flex items-center justify-between",

                  // Overall score
                  div {
                      class: "flex items-center gap-4",
                      div {
                          class: format!(
                              "flex h-20 w-20 items-center justify-center rounded-full {} ring-2 {} transition-all",
                              score_color_classes(score.read().overall),
                              score_ring_classes(score.read().overall)
                          ),
                          span {
                              class: format!(
                                  "text-3xl font-bold tabular-nums {}",
                                  score_text_color_classes(score.read().overall)
                              ),
                              "{score.read().overall}"
                          }
                      }

                      // Gate status
                      div {
                          class: "space-y-1",
                          div {
                              class: "text-sm font-medium text-foreground",
                              "Overall Quality"
                          }
                          div {
                              class: format!(
                                  "text-xs font-medium {}",
                                  if score.read().gate_passes() {
                                      "text-emerald-400"
                                  } else {
                                      "text-red-400"
                                  }
                              ),
                              "{score.read().gate_message()}"
                          }
                      }
                  }

                  // Expand/collapse button
                  button {
                      "type": "button",
                      onclick: on_toggle,
                      class: "shrink-0 rounded p-2 text-muted-foreground/60 transition-colors hover:bg-secondary hover:text-foreground",
                      aria_label: if *expanded.read() { "Collapse details" } else { "Expand details" },
                      if *expanded.read() {
                          // Chevron up
                          svg {
                              xmlns: "http://www.w3.org/2000/svg",
                              width: "20",
                              height: "20",
                              view_box: "0 0 24 24",
                              fill: "none",
                              stroke: "currentColor",
                              stroke_width: "2",
                              stroke_linecap: "round",
                              stroke_linejoin: "round",
                              path { d: "m18 15-6-6-6 6" }
                          }
                      } else {
                          // Chevron down
                          svg {
                              xmlns: "http://www.w3.org/2000/svg",
                              width: "20",
                              height: "20",
                              view_box: "0 0 24 24",
                              fill: "none",
                              stroke: "currentColor",
                              stroke_width: "2",
                              stroke_linecap: "round",
                              stroke_linejoin: "round",
                              path { d: "m6 9 6 6 6-6" }
                          }
                      }
                  }
              }

              // Dimension bars (always visible)
              div {
                  class: "mt-4 space-y-2",
                  {score.read().dimensions.iter().map(|dimension| {
                      let dim_name = dimension.name.clone();
                      let dim_score = dimension.score;
                      let dim_issues = dimension.issues.clone();
                      let has_issues = !dim_issues.is_empty();

                      rsx! {
                          div {
                              class: "group relative flex items-center gap-3",
                              // Tooltip for issues
                              if has_issues {
                                  div {
                                      class: "invisible absolute -top-8 left-0 z-50 w-64 rounded-md bg-popover border border-border px-3 py-2 text-xs text-popover-foreground shadow-lg group-hover:visible transition-opacity",
                                      div {
                                          class: "font-medium text-destructive mb-1",
                                          "Issues:"
                                      }
                                      ul {
                                          class: "space-y-0.5 list-disc list-inside",
                                          {dim_issues.iter().map(|issue| {
                                              rsx! {
                                                  li {
                                                      class: "text-muted-foreground",
                                                      "{issue}"
                                                  }
                                              }
                                          })}
                                      }
                                      // Arrow
                                      div {
                                          class: "absolute -bottom-1 left-4 h-2 w-2 rotate-45 bg-popover border-r border-b border-border"
                                      }
                                  }
                              }

                              // Dimension name
                              div {
                                  class: "w-24 text-xs text-muted-foreground truncate",
                                  title: "{dim_name}",
                                  "{dim_name}"
                              }

                              // Mini bar
                              div {
                                  class: "flex-1 h-2 bg-secondary/30 rounded-full overflow-hidden",
                                  div {
                                      class: format!(
                                          "h-full rounded-full transition-all duration-500 {}",
                                          score_color_classes(dim_score)
                                      ),
                                      style: "width: {dim_score}%",
                                  }
                              }

                              // Score
                              div {
                                  class: "w-8 text-xs font-medium tabular-nums text-right {score_text_color_classes(dim_score)}",
                                  "{dim_score}"
                              }

                              // Warning indicator if has issues
                              if has_issues {
                                  div {
                                      class: "relative group/indicator",
                                      class: "w-4 h-4 rounded-full bg-amber-500/20 border border-amber-500/50 flex items-center justify-center",
                                      svg {
                                          xmlns: "http://www.w3.org/2000/svg",
                                          width: "10",
                                          height: "10",
                                          view_box: "0 0 24 24",
                                          fill: "none",
                                          stroke: "currentColor",
                                          stroke_width: "2",
                                          stroke_linecap: "round",
                                          stroke_linejoin: "round",
                                          class: "text-amber-400",
                                          path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" }
                                          line { x1: "12", y1: "9", x2: "12", y2: "13" }
                                          line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
                                      }
                                  }
                              }
                          }
                      }
                  })}
              }
          }

          // Expanded details section
          if *expanded.read() {
              div {
                  class: "border-t border-border/50 bg-muted/20 p-6",
                  div {
                      class: "space-y-4",
                      h4 {
                          class: "text-sm font-semibold text-foreground mb-3",
                          "Dimension Breakdown"
                      }

                      {score.read().dimensions.iter().map(|dimension| {
                          let dim_name = dimension.name.clone();
                          let dim_score = dimension.score;
                          let dim_issues = dimension.issues.clone();

                          rsx! {
                              div {
                                  class: "space-y-2",
                                  // Dimension header
                                  div {
                                      class: "flex items-center justify-between",
                                      div {
                                          class: "flex items-center gap-2",
                                          span {
                                              class: "text-sm font-medium text-foreground",
                                              "{dim_name}"
                                          }
                                          span {
                                              class: format!(
                                                  "text-sm font-bold tabular-nums {}",
                                                  score_text_color_classes(dim_score)
                                              ),
                                              "{dim_score}"
                                          }
                                      }
                                      div {
                                          class: "text-xs text-muted-foreground",
                                          "out of 100"
                                      }
                                  }

                                  // Full-width bar
                                  div {
                                      class: "h-3 bg-secondary/30 rounded-full overflow-hidden",
                                      div {
                                          class: format!(
                                              "h-full rounded-full transition-all duration-500 {}",
                                              score_color_classes(dim_score)
                                          ),
                                          style: "width: {dim_score}%",
                                      }
                                  }

                                  // Issues list
                                  if !dim_issues.is_empty() {
                                      div {
                                          class: "mt-2 rounded-md bg-amber-500/10 border border-amber-500/20 p-3",
                                          div {
                                              class: "flex items-center gap-2 mb-2",
                                              svg {
                                                  xmlns: "http://www.w3.org/2000/svg",
                                                  width: "14",
                                                  height: "14",
                                                  view_box: "0 0 24 24",
                                                  fill: "none",
                                                  stroke: "currentColor",
                                                  stroke_width: "2",
                                                  stroke_linecap: "round",
                                                  stroke_linejoin: "round",
                                                  class: "text-amber-400 shrink-0",
                                                  path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" }
                                                  line { x1: "12", y1: "9", x2: "12", y2: "13" }
                                                  line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
                                              }
                                              span {
                                                  class: "text-xs font-medium text-amber-400",
                                                  "Improvement needed"
                                              }
                                          }
                                          ul {
                                              class: "space-y-1 list-disc list-inside",
                                              {dim_issues.iter().map(|issue| {
                                                  rsx! {
                                                      li {
                                                          class: "text-xs text-muted-foreground",
                                                          "{issue}"
                                                      }
                                                  }
                                              })}
                                          }
                                      }
                                  } else {
                                      div {
                                          class: "mt-2 rounded-md bg-emerald-500/10 border border-emerald-500/20 p-3",
                                          div {
                                              class: "flex items-center gap-2",
                                              svg {
                                                  xmlns: "http://www.w3.org/2000/svg",
                                                  width: "14",
                                                  height: "14",
                                                  view_box: "0 0 24 24",
                                                  fill: "none",
                                                  stroke: "currentColor",
                                                  stroke_width: "2",
                                                  stroke_linecap: "round",
                                                  stroke_linejoin: "round",
                                                  class: "text-emerald-400 shrink-0",
                                                  path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
                                                  path { d: "m9 11 3 3L22 4" }
                                              }
                                              span {
                                                  class: "text-xs font-medium text-emerald-400",
                                                  "No issues — meets quality standards"
                                              }
                                          }
                                      }
                                  }
                              }
                          }
                      })}
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
  fn test_score_color_classes() {
    assert_eq!(score_color_classes(85), "bg-emerald-500/60");
    assert_eq!(score_color_classes(70), "bg-emerald-500/60");
    assert_eq!(score_color_classes(60), "bg-amber-500/60");
    assert_eq!(score_color_classes(50), "bg-amber-500/60");
    assert_eq!(score_color_classes(30), "bg-red-500/60");
    assert_eq!(score_color_classes(0), "bg-red-500/60");
  }

  #[test]
  fn test_score_text_color_classes() {
    assert_eq!(score_text_color_classes(85), "text-emerald-400");
    assert_eq!(score_text_color_classes(70), "text-emerald-400");
    assert_eq!(score_text_color_classes(60), "text-amber-400");
    assert_eq!(score_text_color_classes(50), "text-amber-400");
    assert_eq!(score_text_color_classes(30), "text-red-400");
    assert_eq!(score_text_color_classes(0), "text-red-400");
  }

  #[test]
  fn test_score_ring_classes() {
    assert_eq!(score_ring_classes(85), "ring-emerald-500/30");
    assert_eq!(score_ring_classes(70), "ring-emerald-500/30");
    assert_eq!(score_ring_classes(60), "ring-amber-500/30");
    assert_eq!(score_ring_classes(50), "ring-amber-500/30");
    assert_eq!(score_ring_classes(30), "ring-red-500/30");
    assert_eq!(score_ring_classes(0), "ring-red-500/30");
  }

  #[test]
  fn test_quality_dimension_new() {
    let dim = QualityDimension::new("Clarity", 80);
    assert_eq!(dim.name, "Clarity");
    assert_eq!(dim.score, 80);
    assert!(dim.issues.is_empty());
  }

  #[test]
  fn test_quality_dimension_with_issues() {
    let dim = QualityDimension::new("Clarity", 60).with_issues(vec![
      "Add more detail".to_string(),
      "Clarify assumptions".to_string(),
    ]);
    assert_eq!(dim.issues.len(), 2);
    assert_eq!(dim.issues[0], "Add more detail");
    assert_eq!(dim.issues[1], "Clarify assumptions");
  }

  #[test]
  fn test_quality_score_new() {
    let score = QualityScore::new(75);
    assert_eq!(score.overall, 75);
    assert!(score.dimensions.is_empty());
  }

  #[test]
  fn test_quality_score_with_dimensions() {
    let dimensions = vec![
      QualityDimension::new("Clarity", 80),
      QualityDimension::new("Completeness", 70),
      QualityDimension::new("Accuracy", 90),
    ];
    let score = QualityScore::new(80).with_dimensions(dimensions.clone());
    assert_eq!(score.overall, 80);
    assert_eq!(score.dimensions.len(), 3);
  }

  #[test]
  fn test_quality_score_gate_passes() {
    let pass = QualityScore::new(70);
    assert!(pass.gate_passes());

    let pass_high = QualityScore::new(85);
    assert!(pass_high.gate_passes());

    let fail = QualityScore::new(69);
    assert!(!fail.gate_passes());

    let fail_zero = QualityScore::new(0);
    assert!(!fail_zero.gate_passes());
  }

  #[test]
  fn test_quality_score_gate_message() {
    let pass = QualityScore::new(75);
    assert_eq!(pass.gate_message(), "Quality gate: PASS");

    let fail = QualityScore::new(55);
    assert_eq!(fail.gate_message(), "Quality gate: FAIL (need 70, have 55)");

    let fail_exact = QualityScore::new(69);
    assert_eq!(
      fail_exact.gate_message(),
      "Quality gate: FAIL (need 70, have 69)"
    );
  }

  #[test]
  fn test_quality_score_default() {
    let score = QualityScore::default();
    assert_eq!(score.overall, 0);
    assert!(score.dimensions.is_empty());
  }

  #[test]
  fn test_quality_dimension_equality() {
    let dim1 = QualityDimension::new("Test", 75);
    let dim2 = QualityDimension::new("Test", 75);
    assert_eq!(dim1, dim2);

    let dim3 = QualityDimension::new("Test", 80);
    assert_ne!(dim1, dim3);
  }

  #[test]
  fn test_quality_score_equality() {
    let score1 = QualityScore {
      overall: 75,
      dimensions: vec![QualityDimension::new("Test", 80)],
    };
    let score2 = QualityScore {
      overall: 75,
      dimensions: vec![QualityDimension::new("Test", 80)],
    };
    assert_eq!(score1, score2);

    let score3 = QualityScore {
      overall: 80,
      dimensions: vec![QualityDimension::new("Test", 80)],
    };
    assert_ne!(score1, score3);
  }

  #[test]
  fn test_quality_gate_threshold_constant() {
    assert_eq!(QUALITY_GATE_THRESHOLD, 70);
  }

  #[test]
  fn test_five_dimensions() {
    let dimensions = vec![
      QualityDimension::new("Clarity", 85),
      QualityDimension::new("Completeness", 70),
      QualityDimension::new("Accuracy", 90),
      QualityDimension::new("Relevance", 75),
      QualityDimension::new("Structure", 80),
    ];
    assert_eq!(dimensions.len(), 5);
  }

  #[test]
  fn test_boundary_scores() {
    // Test boundary conditions for color coding
    assert_eq!(score_color_classes(100), "bg-emerald-500/60");
    assert_eq!(score_color_classes(70), "bg-emerald-500/60");
    assert_eq!(score_color_classes(69), "bg-amber-500/60");
    assert_eq!(score_color_classes(50), "bg-amber-500/60");
    assert_eq!(score_color_classes(49), "bg-red-500/60");
    assert_eq!(score_color_classes(1), "bg-red-500/60");
  }

  #[test]
  fn test_gate_boundary() {
    // Exact threshold should pass
    let pass = QualityScore::new(70);
    assert!(pass.gate_passes());

    // One below should fail
    let fail = QualityScore::new(69);
    assert!(!fail.gate_passes());
  }
}
