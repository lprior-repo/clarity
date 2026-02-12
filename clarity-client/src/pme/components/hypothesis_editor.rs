//! Hypothesis Editor Component
//!
//! UI for creating and managing scientific hypotheses with required null hypothesis.

#![allow(clippy::disallowed_methods)]

use crate::pme::state::PmeDiscoverSignals;
use crate::pme::types::{Hypothesis, HypothesisStatus, PmeError};
use dioxus::prelude::*;

/// Hypothesis Editor Component
#[component]
pub fn HypothesisEditor(signals: Signal<PmeDiscoverSignals>) -> Element {
  let mut thesis = use_signal(String::new);
  let mut null_hypothesis = use_signal(String::new);
  let mut validation_criterion = use_signal(String::new);
  let mut error_msg = use_signal(|| None::<String>);

  let create_hypothesis = move |_| {
    error_msg.set(None);

    let thesis_val = thesis.read().clone();
    let null_val = null_hypothesis.read().clone();
    let criterion_val = validation_criterion.read().clone();

    match Hypothesis::new(thesis_val, null_val) {
      Ok(h) => {
        let h = if !criterion_val.trim().is_empty() {
          h.with_validation_criterion(criterion_val)
        } else {
          h
        };

        signals.write().add_hypothesis(h);
        thesis.set(String::new());
        null_hypothesis.set(String::new());
        validation_criterion.set(String::new());
      }
      Err(PmeError::EmptyField(field)) => {
        error_msg.set(Some(format!("{} is required", field)));
      }
      Err(e) => {
        error_msg.set(Some(e.to_string()));
      }
    }
  };

  let hypothesis_list = signals.read().hypotheses_list();

  rsx! {
    div { class: "hypothesis-editor",
      div { class: "editor-header",
        h3 { "Thesis & Antithesis Generator" }
        p { class: "hint", "Scientific rigor: every hypothesis needs a null hypothesis to prevent confirmation bias" }
      }

      // Error display
      {error_msg.read().as_ref().map(|msg| rsx! {
        div { class: "error-banner", "{msg}" }
      })}

      // Create new hypothesis form
      div { class: "hypothesis-form",
        div { class: "form-group",
          label { "Thesis Statement *" }
          textarea {
            class: "form-control",
            placeholder: "What do you believe is true?",
            value: "{thesis}",
            oninput: move |evt| thesis.set(evt.value()),
            rows: 3,
          }
        }

        div { class: "form-group",
          label { "Null Hypothesis *" }
          textarea {
            class: "form-control",
            placeholder: "What would prove you wrong?",
            value: "{null_hypothesis}",
            oninput: move |evt| null_hypothesis.set(evt.value()),
            rows: 3,
          }
        }

        div { class: "form-group",
          label { "Validation Criterion" }
          input {
            r#type: "text",
            class: "form-control",
            placeholder: "How will you test this? (optional)",
            value: "{validation_criterion}",
            oninput: move |evt| validation_criterion.set(evt.value()),
          }
        }

        button {
          class: "btn btn-primary",
          onclick: create_hypothesis,
          disabled: thesis.read().trim().is_empty() || null_hypothesis.read().trim().is_empty(),
          "Add Hypothesis"
        }
      }

      // Hypothesis list
      div { class: "hypothesis-list",
        h4 { "Your Hypotheses ({hypothesis_list.len()})" }

        {if hypothesis_list.is_empty() {
          rsx! {
            div { class: "empty-state",
              p { "No hypotheses yet. Start by stating what you believe and what would prove you wrong." }
            }
          }
        } else {
          rsx! {
            for hypothesis in hypothesis_list {
              HypothesisCard {
                key: "{hypothesis.id}",
                hypothesis,
                signals
              }
            }
          }
        }}
      }
    }
  }
}

/// Individual hypothesis card
#[component]
fn HypothesisCard(hypothesis: Hypothesis, signals: Signal<PmeDiscoverSignals>) -> Element {
  let id = hypothesis.id;
  let confidence = use_signal(|| hypothesis.confidence_score);

  let status_class = match hypothesis.status {
    HypothesisStatus::Validated => "status-validated",
    HypothesisStatus::Refuted => "status-refuted",
    HypothesisStatus::Testing => "status-testing",
    HypothesisStatus::Inconclusive => "status-inconclusive",
    HypothesisStatus::Formulating => "status-formulating",
  };

  let delete_hypothesis = move |_| {
    signals.write().remove_hypothesis(id);
  };

  rsx! {
    div { class: "hypothesis-card {status_class}",
      div { class: "card-header",
        span { class: "status-badge {status_class}", "{hypothesis.status}" }
        button {
          class: "btn btn-icon btn-danger",
          onclick: delete_hypothesis,
          "×"
        }
      }

      div { class: "card-body",
        div { class: "thesis",
          strong { "Thesis: " }
          "{hypothesis.thesis_statement}"
        }

        div { class: "null-hypothesis",
          strong { "Null: " }
          "{hypothesis.null_hypothesis}"
        }

        div { class: "confidence-control",
          label { "Confidence: {(*confidence.read() * 100.0) as i32}%" }
        }
      }
    }
  }
}

/// Hypothesis status summary for dashboard
#[component]
pub fn HypothesisSummary(signals: Signal<PmeDiscoverSignals>) -> Element {
  let s = signals.read();
  let validated = s.validated_hypothesis_count();
  let refuted = s.refuted_hypothesis_count();
  let testing = s.testing_hypothesis_count();
  let total = s.hypothesis_count();

  rsx! {
    div { class: "hypothesis-summary",
      div { class: "summary-stat",
        span { class: "stat-value", "{validated}" }
        span { class: "stat-label", "Validated" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{refuted}" }
        span { class: "stat-label", "Refuted" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{testing}" }
        span { class: "stat-label", "Testing" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{total}" }
        span { class: "stat-label", "Total" }
      }
    }
  }
}
