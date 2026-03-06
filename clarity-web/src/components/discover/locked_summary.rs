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

use dioxus::prelude::*;

/// Artifact statistics for the locked summary
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ArtifactStats {
  /// Number of beads generated
  pub bead_count: usize,
  /// Number of fields extracted
  pub field_count: usize,
  /// Number of validation checks passed
  pub validation_count: usize,
  /// Compilation timestamp (ISO 8601)
  pub compiled_at: String,
}

impl ArtifactStats {
  /// Create new artifact stats
  #[must_use]
  pub fn new(bead_count: usize, field_count: usize, validation_count: usize) -> Self {
    Self {
      bead_count,
      field_count,
      validation_count,
      compiled_at: chrono::Utc::now().to_rfc3339(),
    }
  }

  /// Create stats with a specific timestamp
  #[must_use]
  pub fn with_timestamp(mut self, timestamp: String) -> Self {
    self.compiled_at = timestamp;
    self
  }

  /// Check if any artifacts were generated
  #[must_use]
  pub const fn has_artifacts(&self) -> bool {
    self.bead_count > 0
  }

  /// Get a summary string for display
  #[must_use]
  pub fn summary(&self) -> String {
    format!(
      "{} bead{}, {} field{}, {} validation{}",
      self.bead_count,
      if self.bead_count == 1 { "" } else { "s" },
      self.field_count,
      if self.field_count == 1 { "" } else { "s" },
      self.validation_count,
      if self.validation_count == 1 { "" } else { "s" },
    )
  }
}

/// Props for `LockedSummary` component
#[derive(Clone, Props, PartialEq)]
pub struct LockedSummaryProps {
  /// Artifact statistics to display
  pub stats: Signal<ArtifactStats>,
  /// Callback when user clicks export
  #[props(default)]
  pub on_export: Option<Callback<()>>,
  /// Callback when user clicks restart
  #[props(default)]
  pub on_restart: Option<Callback<()>>,
  /// Whether export is in progress
  #[props(default)]
  pub is_exporting: bool,
}

/// `LockedSummary` component
///
/// Displays the completion summary when the plan is locked:
/// - "Plan Locked" header with success icon
/// - Artifact statistics (bead count, field count, validations)
/// - Export button for downloading KIRK JSON
/// - Restart button to begin a new session
///
/// # Accessibility
///
/// Uses semantic HTML and ARIA attributes for screen readers.
/// Focus management ensures the summary is announced when rendered.
#[component]
pub fn LockedSummary(props: LockedSummaryProps) -> Element {
  let LockedSummaryProps {
    stats,
    on_export,
    on_restart,
    is_exporting,
  } = props;

  let stats_read = stats.read();
  let bead_count = stats_read.bead_count;
  let field_count = stats_read.field_count;
  let validation_count = stats_read.validation_count;
  let compiled_at = stats_read.compiled_at.clone();
  let has_artifacts = stats_read.has_artifacts();
  drop(stats_read);

  // Format timestamp for display
  let formatted_time = format_timestamp(&compiled_at);

  rsx! {
      div {
          class: "flex flex-col gap-6 w-full max-w-2xl mx-auto",
          role: "region",
          "aria-label": "Plan locked summary",

          // Success header
          div {
              class: "flex flex-col items-center gap-4 text-center",

              // Success icon
              div {
                  class: "flex items-center justify-center w-16 h-16 rounded-full bg-emerald-500/20 ring-4 ring-emerald-500/30",
                  svg {
                      class: "w-8 h-8 text-emerald-400",
                      xmlns: "http://www.w3.org/2000/svg",
                      fill: "none",
                      view_box: "0 0 24 24",
                      stroke: "currentColor",
                      stroke_width: "2",
                      stroke_linecap: "round",
                      stroke_linejoin: "round",
                      path { d: "M5 13l4 4L19 7" },
                  }
              }

              // Header text
              h2 {
                  class: "text-2xl font-bold text-foreground",
                  "Plan Locked"
              }
              p {
                  class: "text-muted-foreground",
                  "Your plan has been compiled and is ready for implementation."
              }
          }

          // Statistics card
          div {
              class: "rounded-lg border border-border bg-card p-6 shadow-sm",

              h3 {
                  class: "text-sm font-medium text-muted-foreground mb-4 uppercase tracking-wide",
                  "Compiled Artifacts"
              }

              // Stats grid
              div {
                  class: "grid grid-cols-3 gap-4",

                  // Bead count
                  StatItem {
                      label: "Beads Generated",
                      value: "{bead_count}",
                      icon: rsx! {
                          svg {
                              class: "w-5 h-5",
                              xmlns: "http://www.w3.org/2000/svg",
                              fill: "none",
                              view_box: "0 0 24 24",
                              stroke: "currentColor",
                              stroke_width: "2",
                              stroke_linecap: "round",
                              stroke_linejoin: "round",
                              circle { cx: "12", cy: "12", r: "10" },
                              path { d: "M12 6v6l4 2" },
                          }
                      },
                  },

                  // Field count
                  StatItem {
                      label: "Fields Extracted",
                      value: "{field_count}",
                      icon: rsx! {
                          svg {
                              class: "w-5 h-5",
                              xmlns: "http://www.w3.org/2000/svg",
                              fill: "none",
                              view_box: "0 0 24 24",
                              stroke: "currentColor",
                              stroke_width: "2",
                              stroke_linecap: "round",
                              stroke_linejoin: "round",
                              path { d: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" },
                          }
                      },
                  },

                  // Validation count
                  StatItem {
                      label: "Validations Passed",
                      value: "{validation_count}",
                      icon: rsx! {
                          svg {
                              class: "w-5 h-5",
                              xmlns: "http://www.w3.org/2000/svg",
                              fill: "none",
                              view_box: "0 0 24 24",
                              stroke: "currentColor",
                              stroke_width: "2",
                              stroke_linecap: "round",
                              stroke_linejoin: "round",
                              path { d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" },
                          }
                      },
                  },
              }

              // Compilation timestamp
              div {
                  class: "mt-4 pt-4 border-t border-border/50",
                  div {
                      class: "flex items-center gap-2 text-xs text-muted-foreground",
                      svg {
                          class: "w-4 h-4",
                          xmlns: "http://www.w3.org/2000/svg",
                          fill: "none",
                          view_box: "0 0 24 24",
                          stroke: "currentColor",
                          stroke_width: "2",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          path { d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" },
                      }
                      span { "Compiled: {formatted_time}" }
                  }
              }
          }

          // Warning if no artifacts
          if !has_artifacts {
              div {
                  class: "rounded-lg border border-amber-500/30 bg-amber-500/10 p-4",
                  role: "alert",
                  div {
                      class: "flex items-start gap-3",
                      svg {
                          class: "w-5 h-5 text-amber-400 flex-shrink-0 mt-0.5",
                          xmlns: "http://www.w3.org/2000/svg",
                          fill: "none",
                          view_box: "0 0 24 24",
                          stroke: "currentColor",
                          stroke_width: "2",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          path { d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" },
                      }
                      div {
                          p {
                              class: "text-sm font-medium text-amber-400",
                              "No artifacts generated"
                          }
                          p {
                              class: "text-xs text-amber-400/70 mt-1",
                              "Your plan was locked but no beads were generated. Consider restarting to add more detail."
                          }
                      }
                  }
              }
          }

          // Action buttons
          div {
              class: "flex flex-col sm:flex-row gap-3 justify-center",

              // Export button
              if let Some(on_export_callback) = on_export {
                  button {
                      "type": "button",
                      onclick: move |_| on_export_callback.call(()),
                      disabled: is_exporting || !has_artifacts,
                      class: format!(
                          "flex items-center justify-center gap-2 px-6 py-3 rounded-lg font-medium transition-colors {}",
                          if is_exporting || !has_artifacts {
                              "bg-muted text-muted-foreground cursor-not-allowed"
                          } else {
                              "bg-primary text-primary-foreground hover:bg-primary/90"
                          }
                      ),
                      "aria-busy": is_exporting,

                      if is_exporting {
                          // Loading spinner
                          svg {
                              class: "w-5 h-5 animate-spin",
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
                      } else {
                          // Download icon
                          svg {
                              class: "w-5 h-5",
                              xmlns: "http://www.w3.org/2000/svg",
                              fill: "none",
                              view_box: "0 0 24 24",
                              stroke: "currentColor",
                              stroke_width: "2",
                              stroke_linecap: "round",
                              stroke_linejoin: "round",
                              path { d: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" },
                          }
                      }
                      span {
                          if is_exporting {
                              "Exporting..."
                          } else if has_artifacts {
                              "Export KIRK JSON"
                          } else {
                              "No artifacts to export"
                          }
                      }
                  }
              }

              // Restart button
              if let Some(on_restart_callback) = on_restart {
                  button {
                      "type": "button",
                      onclick: move |_| on_restart_callback.call(()),
                      class: "flex items-center justify-center gap-2 px-6 py-3 rounded-lg font-medium border border-border bg-background text-foreground hover:bg-accent hover:text-accent-foreground transition-colors",

                      // Restart icon
                      svg {
                          class: "w-5 h-5",
                          xmlns: "http://www.w3.org/2000/svg",
                          fill: "none",
                          view_box: "0 0 24 24",
                          stroke: "currentColor",
                          stroke_width: "2",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          path { d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" },
                      }
                      span { "Start New Plan" }
                  }
              }
          }
      }
  }
}

/// Format an ISO 8601 timestamp for display
fn format_timestamp(iso_timestamp: &str) -> String {
  // Try to parse and format nicely, fall back to original string on error
  chrono::DateTime::parse_from_rfc3339(iso_timestamp).map_or_else(
    |_| iso_timestamp.to_string(),
    |dt| dt.format("%B %d, %Y at %I:%M %p").to_string(),
  )
}

/// Props for `StatItem` component
#[derive(Clone, Props, PartialEq)]
struct StatItemProps {
  /// Label for the stat
  label: String,
  /// Value to display
  value: String,
  /// Optional icon
  icon: Element,
}

/// Individual stat item in the summary
#[component]
fn StatItem(props: StatItemProps) -> Element {
  let StatItemProps { label, value, icon } = props;

  rsx! {
      div {
          class: "flex flex-col items-center gap-2 p-4 rounded-lg bg-muted/30",

          // Icon
          div {
              class: "text-primary",
              {icon}
          }

          // Value
          span {
              class: "text-2xl font-bold text-foreground",
              "{value}"
          }

          // Label
          span {
              class: "text-xs text-muted-foreground text-center",
              "{label}"
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
  fn test_artifact_stats_new() {
    let stats = ArtifactStats::new(5, 10, 3);
    assert_eq!(stats.bead_count, 5);
    assert_eq!(stats.field_count, 10);
    assert_eq!(stats.validation_count, 3);
    assert!(stats.has_artifacts());
  }

  #[test]
  fn test_artifact_stats_default() {
    let stats = ArtifactStats::default();
    assert_eq!(stats.bead_count, 0);
    assert_eq!(stats.field_count, 0);
    assert_eq!(stats.validation_count, 0);
    assert!(!stats.has_artifacts());
  }

  #[test]
  fn test_artifact_stats_with_timestamp() {
    let stats = ArtifactStats::new(1, 1, 1).with_timestamp("2024-01-01T00:00:00Z".to_string());
    assert_eq!(stats.compiled_at, "2024-01-01T00:00:00Z");
  }

  #[test]
  fn test_artifact_stats_has_artifacts() {
    let with_artifacts = ArtifactStats::new(1, 0, 0);
    assert!(with_artifacts.has_artifacts());

    let without_artifacts = ArtifactStats::new(0, 5, 3);
    assert!(!without_artifacts.has_artifacts());
  }

  #[test]
  fn test_artifact_stats_summary_singular() {
    let stats = ArtifactStats::new(1, 1, 1);
    let summary = stats.summary();
    assert!(summary.contains("1 bead"));
    assert!(summary.contains("1 field"));
    assert!(summary.contains("1 validation"));
    assert!(!summary.contains("1 beads"));
  }

  #[test]
  fn test_artifact_stats_summary_plural() {
    let stats = ArtifactStats::new(2, 3, 4);
    let summary = stats.summary();
    assert!(summary.contains("2 beads"));
    assert!(summary.contains("3 fields"));
    assert!(summary.contains("4 validations"));
  }

  #[test]
  fn test_artifact_stats_summary_zero() {
    let stats = ArtifactStats::new(0, 0, 0);
    let summary = stats.summary();
    assert!(summary.contains("0 beads"));
    assert!(summary.contains("0 fields"));
    assert!(summary.contains("0 validations"));
  }

  #[test]
  fn test_format_timestamp_valid() {
    let iso = "2024-06-15T14:30:00Z";
    let formatted = format_timestamp(iso);
    // The exact format depends on timezone, but should contain month/day/year info
    assert!(!formatted.is_empty());
    assert_ne!(formatted, iso); // Should be reformatted
  }

  #[test]
  fn test_format_timestamp_invalid() {
    let invalid = "not-a-timestamp";
    let formatted = format_timestamp(invalid);
    assert_eq!(formatted, invalid); // Falls back to original string
  }

  #[test]
  fn test_format_timestamp_empty() {
    let formatted = format_timestamp("");
    assert_eq!(formatted, "");
  }

  // Note: Tests requiring Dioxus runtime (Signal, Callback, rsx!) are skipped.
  // The following tests require dioxus::prelude::launch_test() wrapper:
  // - test_stat_item_props_clone
  // - test_stat_item_props_equality
  // - test_locked_summary_props_default
  // - test_locked_summary_props_with_callbacks

  #[test]
  fn test_artifact_stats_equality() {
    let stats1 = ArtifactStats::new(1, 2, 3);
    let stats2 = ArtifactStats::new(1, 2, 3).with_timestamp(stats1.compiled_at.clone());
    assert_eq!(stats1, stats2);

    let stats3 = ArtifactStats::new(1, 2, 4);
    assert_ne!(stats1, stats3);
  }

  #[test]
  fn test_artifact_stats_clone() {
    let stats = ArtifactStats::new(1, 2, 3);
    let cloned = stats.clone();
    assert_eq!(stats, cloned);
  }

  #[test]
  fn test_artifact_stats_debug() {
    let stats = ArtifactStats::new(1, 2, 3);
    let debug_str = format!("{stats:?}");
    assert!(debug_str.contains("bead_count"));
    assert!(debug_str.contains("field_count"));
    assert!(debug_str.contains("validation_count"));
  }
}
