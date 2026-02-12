//! Scenario Validator Component

#![allow(clippy::disallowed_methods)]

use crate::planner::types::NorthStarScenario;
use crate::pme::state::PmeDiscoverSignals;
use crate::pme::types::PlotHoleSeverity;
use dioxus::prelude::*;

/// Scenario Validator Component
#[component]
pub fn ScenarioValidator(
  signals: Signal<PmeDiscoverSignals>,
  scenarios: Signal<Vec<NorthStarScenario>>,
) -> Element {
  let scenarios_list = scenarios.read();
  let blocking_count = signals.read().blocking_plot_hole_count();

  rsx! {
    div { class: "scenario-validator",
      div { class: "validator-header",
        h3 { "North Star Scenario Validator" }
        p { class: "hint", "Detect plot holes in user journey scenarios" }
      }

      {if blocking_count > 0 {
        rsx! {
          div { class: "warning-banner",
            "{blocking_count} blocking plot hole(s) must be resolved"
          }
        }
      } else {
        rsx! {}
      }}

      div { class: "scenario-list",
        for scenario in scenarios_list.iter() {
          ScenarioCard {
            key: "{scenario.id}",
            scenario: scenario.clone(),
            signals
          }
        }
      }
    }
  }
}

/// Scenario card
#[component]
fn ScenarioCard(
  scenario: NorthStarScenario,
  signals: Signal<PmeDiscoverSignals>,
) -> Element {
  let holes = signals.read().plot_holes_for_scenario(scenario.id);
  let blocking = holes.iter().filter(|p| p.is_blocking()).count();
  let total = holes.len();

  let status_class = if blocking > 0 {
    "status-blocking"
  } else if !holes.is_empty() {
    "status-warnings"
  } else {
    "status-clean"
  };

  rsx! {
    div { class: "scenario-card {status_class}",
      div { class: "card-header",
        h4 { "{scenario.title}" }
        span { "{total} holes ({blocking} blocking)" }
      }
      div { class: "card-body",
        p { "{scenario.narrative}" }
        p { "Steps: {scenario.steps.len()}" }
      }
    }
  }
}

/// Scenario summary for dashboard
#[component]
pub fn ScenarioSummary(signals: Signal<PmeDiscoverSignals>) -> Element {
  let (total, blocking, fatal) = signals.read().plot_hole_counts();

  rsx! {
    div { class: "scenario-summary",
      div { class: "summary-stat",
        span { class: "stat-value", "{total}" }
        span { class: "stat-label", "Plot Holes" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{blocking}" }
        span { class: "stat-label", "Blocking" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{fatal}" }
        span { class: "stat-label", "Fatal" }
      }
    }
  }
}
