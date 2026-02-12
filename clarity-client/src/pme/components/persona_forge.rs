//! Persona Forge Component

#![allow(clippy::disallowed_methods)]

use crate::planner::types::Persona;
use crate::pme::state::PmeDiscoverSignals;
use dioxus::prelude::*;

/// Persona Forge Component
#[component]
pub fn PersonaForge(
  signals: Signal<PmeDiscoverSignals>,
  personas: Signal<Vec<Persona>>,
) -> Element {
  let personas_list = personas.read();
  let straw_men_count = signals.read().straw_men_count(&personas_list);

  rsx! {
    div { class: "persona-forge",
      div { class: "forge-header",
        h3 { "Persona Forge" }
        p { class: "hint", "Evidence-based personas prevent straw man assumptions" }
      }

      {if straw_men_count > 0 {
        rsx! {
          div { class: "warning-banner",
            "{straw_men_count} persona(s) may be straw men - not backed by sufficient evidence"
          }
        }
      } else {
        rsx! {}
      }}

      div { class: "persona-list",
        for persona in personas_list.iter() {
          PersonaEvidenceCard {
            key: "{persona.id}",
            persona: persona.clone(),
            signals
          }
        }
      }
    }
  }
}

/// Persona evidence card
#[component]
fn PersonaEvidenceCard(
  persona: Persona,
  signals: Signal<PmeDiscoverSignals>,
) -> Element {
  let (confidence, is_validated, is_straw_man) = signals.read().persona_evidence_stats(persona.id);

  let status_class = if is_validated {
    "status-validated"
  } else if is_straw_man {
    "status-straw-man"
  } else {
    "status-pending"
  };

  rsx! {
    div { class: "persona-card {status_class}",
      div { class: "card-header",
        h4 { "{persona.name}" }
        span { class: "role", "{persona.role}" }
      }
      div { class: "card-body",
        p { "{persona.description}" }
        div { class: "confidence",
          "Confidence: {(confidence * 100.0) as i32}%"
        }
        {if is_straw_man {
          rsx! { div { class: "warning", "Needs validation" } }
        } else if is_validated {
          rsx! { div { class: "success", "Validated" } }
        } else {
          rsx! {}
        }}
      }
    }
  }
}

/// Persona summary for dashboard
#[component]
pub fn PersonaSummary(
  signals: Signal<PmeDiscoverSignals>,
  personas: Signal<Vec<Persona>>,
) -> Element {
  let personas_list = personas.read();
  let (total, validated, straw_men) = signals.read().persona_stats(&personas_list);

  rsx! {
    div { class: "persona-summary",
      div { class: "summary-stat",
        span { class: "stat-value", "{total}" }
        span { class: "stat-label", "Personas" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{validated}" }
        span { class: "stat-label", "Validated" }
      }
      div { class: "summary-stat warning",
        span { class: "stat-value", "{straw_men}" }
        span { class: "stat-label", "Straw Men" }
      }
    }
  }
}
