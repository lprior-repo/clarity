//! PME Discover Panel
//!
//! Main panel component for the Product-Market Engineer Discover phase.
//! Integrates hypothesis editor, interview logger, persona forge, and scenario validator.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]
// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::planner::types::{NorthStarScenario, Persona};
use crate::pme::components::{
  HypothesisEditor, HypothesisSummary, InterviewLogger, InterviewSummary, LatticeAuditSummaryCard,
  PersonaForge, PersonaSummary, ScenarioSummary, ScenarioValidator,
};
use crate::pme::state::PmeDiscoverSignals;
use dioxus::prelude::*;

/// Tab navigation for PME Discover phase
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiscoverTab {
  /// Overview dashboard with all summaries
  Dashboard,
  /// Hypothesis editor
  Hypotheses,
  /// Interview logger
  Interviews,
  /// Persona forge
  Personas,
  /// Scenario validator
  Scenarios,
}

impl DiscoverTab {
  /// Get display label for tab
  const fn label(&self) -> &'static str {
    match self {
      Self::Dashboard => "Dashboard",
      Self::Hypotheses => "Hypotheses",
      Self::Interviews => "Interviews",
      Self::Personas => "Personas",
      Self::Scenarios => "Scenarios",
    }
  }

  /// All tabs in order
  const fn all() -> [Self; 5] {
    [
      Self::Dashboard,
      Self::Hypotheses,
      Self::Interviews,
      Self::Personas,
      Self::Scenarios,
    ]
  }
}

/// PME Discover Panel Props
#[derive(Props, Clone, PartialEq)]
pub struct PmeDiscoverPanelProps {
  /// Planner personas signal
  pub personas: Signal<Vec<Persona>>,
  /// Planner scenarios signal
  pub scenarios: Signal<Vec<NorthStarScenario>>,
}

/// Main PME Discover Panel Component
///
/// Integrates all PME Discover sub-components with tab navigation.
#[component]
pub fn PmeDiscoverPanel(props: PmeDiscoverPanelProps) -> Element {
  let mut active_tab = use_signal(|| DiscoverTab::Dashboard);
  let pme_signals = use_context::<Signal<PmeDiscoverSignals>>();

  let health_score = pme_signals.read().health_score();
  let can_proceed = pme_signals.read().can_proceed_to_define();

  rsx! {
    div { class: "pme-discover-panel",
      // Header with health score
      div { class: "panel-header",
        div { class: "header-title",
          h2 { "PME Discover Phase" }
          p { class: "subtitle", "Scientific rigor for turning rough ideas into vision" }
        }

        // Health score gauge
        div { class: "health-gauge",
          HealthGauge { score: health_score }
        }

        // Proceed indicator
        div { class: "proceed-indicator",
          {if can_proceed {
            rsx! {
              div { class: "proceed-status can-proceed",
                span { class: "icon", "✓" }
                span { "Ready to proceed to Define phase" }
              }
            }
          } else {
            rsx! {
              div { class: "proceed-status blocked",
                span { class: "icon", "○" }
                span { "Complete discovery requirements to proceed" }
              }
            }
          }}
        }
      }

      // Tab navigation
      div { class: "tab-navigation",
        for tab in DiscoverTab::all() {
          button {
            key: "{tab:?}",
            class: format!(
              "tab-button {}",
              if *active_tab.read() == tab { "active" } else { "" }
            ),
            onclick: move |_| active_tab.set(tab),
            "{tab.label()}"
          }
        }
      }

      // Tab content
      div { class: "tab-content",
        {match *active_tab.read() {
          DiscoverTab::Dashboard => rsx! {
            PmeDiscoverDashboard {
              pme_signals,
              personas: props.personas,
              scenarios: props.scenarios,
            }
          },
          DiscoverTab::Hypotheses => rsx! {
            HypothesisEditor { signals: pme_signals }
          },
          DiscoverTab::Interviews => rsx! {
            InterviewLogger { signals: pme_signals }
          },
          DiscoverTab::Personas => rsx! {
            PersonaForge {
              signals: pme_signals,
              personas: props.personas,
            }
          },
          DiscoverTab::Scenarios => rsx! {
            ScenarioValidator {
              signals: pme_signals,
              scenarios: props.scenarios,
            }
          },
        }}
      }
    }
  }
}

/// Dashboard showing all PME summary widgets
#[component]
fn PmeDiscoverDashboard(
  pme_signals: Signal<PmeDiscoverSignals>,
  personas: Signal<Vec<Persona>>,
  scenarios: Signal<Vec<NorthStarScenario>>,
) -> Element {
  let health_score = pme_signals.read().health_score();
  let can_proceed = pme_signals.read().can_proceed_to_define();

  rsx! {
    div { class: "pme-dashboard",
      // Overall health card
      div { class: "health-card",
        h3 { "Discovery Health" }
        div { class: "health-score-large",
          span { class: "score-value", "{(health_score * 100.0) as i32}" }
          span { class: "score-unit", "%" }
        }
        div { class: "health-bar",
          div {
            class: "health-fill",
            style: "width: {health_score * 100.0}%"
          }
        }
        {if can_proceed {
          rsx! {
            p { class: "proceed-message success", "All requirements met - ready to proceed" }
          }
        } else {
          rsx! {
            BlockersList { pme_signals, personas }
          }
        }}
      }

      // Summary widgets grid
      div { class: "summary-grid",
        div { class: "summary-card",
          h4 { "Hypotheses" }
          HypothesisSummary { signals: pme_signals }
        }

        div { class: "summary-card",
          h4 { "Interviews" }
          InterviewSummary { signals: pme_signals }
        }

        div { class: "summary-card",
          h4 { "Personas" }
          PersonaSummary {
            signals: pme_signals,
            personas,
          }
        }

        div { class: "summary-card",
          h4 { "Scenarios" }
          ScenarioSummary { signals: pme_signals }
        }

        div { class: "summary-card",
          h4 { "Lattice Audit" }
          LatticeAuditSummaryCard { signals: pme_signals }
        }
      }

      // Quick actions
      div { class: "quick-actions",
        h4 { "Quick Actions" }
        div { class: "action-buttons",
          a { class: "action-btn", href: "#hypotheses", "Add Hypothesis" }
          a { class: "action-btn", href: "#interviews", "Start Interview" }
          a { class: "action-btn", href: "#personas", "Validate Persona" }
          a { class: "action-btn", href: "#scenarios", "Check Scenarios" }
        }
      }
    }
  }
}

/// List of blockers preventing progression
#[component]
fn BlockersList(
  pme_signals: Signal<PmeDiscoverSignals>,
  personas: Signal<Vec<Persona>>,
) -> Element {
  let s = pme_signals.read();
  let interviews_count = s.interview_count();
  let has_validated = s.validated_hypothesis_count() > 0;
  let (_, blocking_holes_count, _) = s.plot_hole_counts();
  let personas_list = personas.read();
  let (_, _, straw_men_count) = s.persona_stats(&personas_list);

  rsx! {
    div { class: "blockers-list",
      h4 { "Requirements to Proceed:" }
      ul {
        {if !has_validated {
          rsx! { li { class: "blocker", "At least one validated hypothesis required" } }
        } else {
          rsx! { li { class: "met", "✓ Has validated hypothesis" } }
        }}

        {if interviews_count < 2 {
          rsx! { li { class: "blocker", "At least 2 interviews required ({interviews_count}/2)" } }
        } else {
          rsx! { li { class: "met", "✓ Sufficient interviews ({interviews_count})" } }
        }}

        {if blocking_holes_count > 0 {
          rsx! { li { class: "blocker", "Resolve {blocking_holes_count} blocking plot hole(s)" } }
        } else {
          rsx! { li { class: "met", "✓ No blocking plot holes" } }
        }}

        {if straw_men_count > 0 {
          rsx! { li { class: "blocker", "Validate {straw_men_count} straw man persona(s)" } }
        } else {
          rsx! { li { class: "met", "✓ All personas validated" } }
        }}
      }
    }
  }
}

/// Health gauge component
#[component]
fn HealthGauge(score: f32) -> Element {
  let percentage = (score * 100.0) as i32;
  let color_class = health_color_class(score);

  rsx! {
    div { class: "health-gauge {color_class}",
      // Simple text-based gauge (SVG removed for Dioxus 0.7 compatibility)
      div { class: "gauge-bar",
        div {
          class: "gauge-fill {color_class}",
          style: "width: {percentage}%"
        }
      }
      div { class: "gauge-value",
        span { class: "number", "{percentage}" }
        span { class: "unit", "%" }
      }
    }
  }
}

/// Helper function to determine health color class from score
fn health_color_class(score: f32) -> &'static str {
  match score {
    s if s >= 0.8 => "excellent",
    s if s >= 0.6 => "good",
    s if s >= 0.4 => "fair",
    s if s >= 0.2 => "poor",
    _ => "critical",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn discover_tab_labels() {
    assert_eq!(DiscoverTab::Dashboard.label(), "Dashboard");
    assert_eq!(DiscoverTab::Hypotheses.label(), "Hypotheses");
    assert_eq!(DiscoverTab::Interviews.label(), "Interviews");
    assert_eq!(DiscoverTab::Personas.label(), "Personas");
    assert_eq!(DiscoverTab::Scenarios.label(), "Scenarios");
  }

  #[test]
  fn discover_tab_all_count() {
    assert_eq!(DiscoverTab::all().len(), 5);
  }

  #[test]
  fn health_color_class_boundaries() {
    assert_eq!(health_color_class(0.9), "excellent");
    assert_eq!(health_color_class(0.8), "excellent");
    assert_eq!(health_color_class(0.79), "good");
    assert_eq!(health_color_class(0.6), "good");
    assert_eq!(health_color_class(0.59), "fair");
    assert_eq!(health_color_class(0.4), "fair");
    assert_eq!(health_color_class(0.39), "poor");
    assert_eq!(health_color_class(0.2), "poor");
    assert_eq!(health_color_class(0.19), "critical");
    assert_eq!(health_color_class(0.0), "critical");
  }
}
