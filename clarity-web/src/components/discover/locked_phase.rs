#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! `LockedPhase` component for the Progressive Discover flow.
//!
//! This is the final phase in the Progressive Discover workflow, displayed after
//! KIRK compilation is complete. It shows:
//! - Completion summary with artifact statistics
//! - Navigation buttons to view Plan, Graph, or State
//! - Export and restart actions

use dioxus::prelude::*;

use super::locked_summary::{ArtifactStats, LockedSummary};
use crate::types::RightTab;

/// Props for `LockedPhase` component
#[derive(Clone, Props, PartialEq)]
pub struct LockedPhaseProps {
  /// Artifact statistics to display in the summary
  pub stats: Signal<ArtifactStats>,
  /// Callback when user navigates to a tab
  #[props(default)]
  pub on_navigate: Option<Callback<RightTab>>,
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

/// `LockedPhase` component
///
/// Displays the final locked state after KIRK compilation is complete.
/// Shows the completion summary with artifact statistics and provides
/// navigation to view the compiled artifacts in different formats.
///
/// # Features
///
/// - `LockedSummary` with bead count, field count, validations
/// - Navigation buttons: View Plan, View Graph, View State
/// - Export button for downloading KIRK JSON
/// - Restart button to begin a new session
///
/// # Accessibility
///
/// Uses semantic HTML and ARIA attributes. The component is announced
/// as a "region" when rendered for screen reader users.
#[component]
pub fn LockedPhase(props: LockedPhaseProps) -> Element {
  let LockedPhaseProps {
    stats,
    on_navigate,
    on_export,
    on_restart,
    is_exporting,
  } = props;

  rsx! {
      div {
          class: "flex flex-col gap-8 w-full max-w-4xl mx-auto",
          role: "region",
          "aria-label": "Locked phase - plan complete",

          // Locked Summary
          LockedSummary {
              stats,
              on_export,
              on_restart,
              is_exporting,
          }

          // Navigation section
          if let Some(navigate_callback) = on_navigate {
              div {
                  class: "rounded-lg border border-border bg-card p-6 shadow-sm",

                  h3 {
                      class: "text-sm font-medium text-muted-foreground mb-4 uppercase tracking-wide",
                      "View Compiled Artifacts"
                  }

                  div {
                      class: "flex flex-col sm:flex-row gap-3",

                      // View Plan button
                      NavButton {
                          label: "View Plan",
                          description: "See the generated beads and tasks",
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
                                  path { d: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" },
                              }
                          },
                          on_click: move |()| navigate_callback.call(RightTab::Plan),
                      }

                      // View Graph button
                      NavButton {
                          label: "View Graph",
                          description: "Visualize the dependency graph",
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
                                  circle { cx: "12", cy: "5", r: "3" },
                                  line { x1: "12", y1: "8", x2: "12", y2: "16" },
                                  circle { cx: "6", cy: "19", r: "3" },
                                  circle { cx: "18", cy: "19", r: "3" },
                                  line { x1: "12", y1: "16", x2: "6", y2: "16" },
                                  line { x1: "12", y1: "16", x2: "18", y2: "16" },
                              }
                          },
                          on_click: move |()| navigate_callback.call(RightTab::Graph),
                      }

                      // View State button
                      NavButton {
                          label: "View State",
                          description: "Inspect the full KIRK contract",
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
                                  path { d: "M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" },
                              }
                          },
                          on_click: move |()| navigate_callback.call(RightTab::State),
                      }
                  }
              }
          }

          // Help text
          p {
              class: "text-center text-xs text-muted-foreground",
              "Your plan is now locked and ready for implementation. Use the navigation buttons above to explore your compiled artifacts."
          }
      }
  }
}

/// Props for `NavButton` component
#[derive(Clone, Props, PartialEq)]
struct NavButtonProps {
  /// Button label
  label: String,
  /// Description text
  description: String,
  /// Icon element
  icon: Element,
  /// Click handler
  on_click: Callback<()>,
}

/// Navigation button with icon and description
#[component]
fn NavButton(props: NavButtonProps) -> Element {
  let NavButtonProps {
    label,
    description,
    icon,
    on_click,
  } = props;

  rsx! {
      button {
          "type": "button",
          onclick: move |_| on_click.call(()),
          class: "flex-1 flex items-start gap-3 p-4 rounded-lg border border-border bg-background hover:bg-accent hover:text-accent-foreground transition-colors text-left",

          // Icon
          div {
              class: "flex-shrink-0 text-primary mt-0.5",
              {icon}
          }

          // Text content
          div {
              class: "flex-1 min-w-0",
              div {
                  class: "font-medium text-foreground",
                  "{label}"
              }
              div {
                  class: "text-xs text-muted-foreground mt-0.5",
                  "{description}"
              }
          }

          // Arrow indicator
          svg {
              class: "w-4 h-4 flex-shrink-0 text-muted-foreground self-center",
              xmlns: "http://www.w3.org/2000/svg",
              fill: "none",
              view_box: "0 0 24 24",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
              stroke_linejoin: "round",
              path { d: "M9 5l7 7-7 7" },
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_right_tab_variants() {
    // Verify RightTab variants exist and can be used
    let plan = RightTab::Plan;
    let graph = RightTab::Graph;
    let state = RightTab::State;

    assert_ne!(plan, graph);
    assert_ne!(graph, state);
    assert_ne!(plan, state);
  }

  #[test]
  fn test_right_tab_default() {
    let default_tab = RightTab::default();
    assert_eq!(default_tab, RightTab::Plan);
  }

  #[test]
  fn test_artifact_stats_integration() {
    let stats = ArtifactStats::new(5, 10, 3);
    assert!(stats.has_artifacts());
    assert_eq!(stats.bead_count, 5);
    assert_eq!(stats.field_count, 10);
    assert_eq!(stats.validation_count, 3);
  }

  // Note: Tests requiring Dioxus runtime (Signal, Callback, rsx!) are skipped.
  // The following tests require dioxus::prelude::launch_test() wrapper:
  // - test_locked_phase_props_default
  // - test_locked_phase_props_with_callbacks
  // - test_locked_phase_props_clone
  // - test_nav_button_props_clone
  // - test_nav_button_props_equality
}
