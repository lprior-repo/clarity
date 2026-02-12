//! Customer Discovery Interview Logger Component

#![allow(clippy::disallowed_methods)]

use crate::pme::state::PmeDiscoverSignals;
use crate::pme::types::{
  CustomerDiscoveryInterview, SignalIntensity, SignalObservation, SignalType,
};
use dioxus::prelude::*;
use uuid::Uuid;

/// Interview Logger Component
#[component]
pub fn InterviewLogger(signals: Signal<PmeDiscoverSignals>) -> Element {
  let mut participant_id = use_signal(String::new);
  let mut show_new = use_signal(|| false);

  let start_interview = move |_| {
    if participant_id.read().trim().is_empty() {
      return;
    }
    let interview = CustomerDiscoveryInterview::new(participant_id.read().clone());
    signals.write().add_interview(interview);
    participant_id.set(String::new());
    show_new.set(false);
  };

  let interview_list = signals.read().interviews_list();

  rsx! {
    div { class: "interview-logger",
      div { class: "logger-header",
        h3 { "Customer Discovery Interview Logger" }
        p { class: "hint", "Track signals from user interviews" }
      }

      {if *show_new.read() {
        rsx! {
          div { class: "new-interview-form",
            input {
              r#type: "text",
              class: "form-control",
              placeholder: "Participant ID",
              value: "{participant_id}",
              oninput: move |evt| participant_id.set(evt.value()),
            }
            button {
              class: "btn btn-primary",
              onclick: start_interview,
              disabled: participant_id.read().trim().is_empty(),
              "Start Interview"
            }
            button {
              class: "btn btn-secondary",
              onclick: move |_| show_new.set(false),
              "Cancel"
            }
          }
        }
      } else {
        rsx! {
          button {
            class: "btn btn-primary",
            onclick: move |_| show_new.set(true),
            "+ New Interview"
          }
        }
      }}

      // Interview list
      div { class: "interview-list",
        h4 { "Interviews ({interview_list.len()})" }

        {if interview_list.is_empty() {
          rsx! {
            div { class: "empty-state",
              p { "No interviews conducted yet" }
            }
          }
        } else {
          rsx! {
            for interview in interview_list {
              div { class: "interview-card",
                key: "{interview.id}",
                div { class: "card-header",
                  h4 { "{interview.participant_id}" }
                  span { "Strength: {(interview.signal_strength * 100.0) as i32}%" }
                }
                div { class: "card-body",
                  p { "Questions: {interview.questions_asked.len()}" }
                  p { "Signals: {interview.signals.len()}" }
                }
              }
            }
          }
        }}
      }
    }
  }
}

/// Interview summary for dashboard
#[component]
pub fn InterviewSummary(signals: Signal<PmeDiscoverSignals>) -> Element {
  let s = signals.read();
  let total = s.interview_count();
  let total_signals = s.total_signal_count();
  let strong_signals = s.strong_interview_count();

  rsx! {
    div { class: "interview-summary",
      div { class: "summary-stat",
        span { class: "stat-value", "{total}" }
        span { class: "stat-label", "Interviews" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{total_signals}" }
        span { class: "stat-label", "Signals" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{strong_signals}" }
        span { class: "stat-label", "Strong" }
      }
    }
  }
}
