#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::types::{get_steps_for_phase, Answer};

pub fn is_phase_done(phase_key: &str, answers: &[Answer]) -> bool {
  let steps = get_steps_for_phase(phase_key);
  let required_steps: Vec<_> = steps.iter().filter(|s| s.required).collect();
  if required_steps.is_empty() {
    return false;
  }
  required_steps
    .iter()
    .all(|s| answers.iter().any(|a| a.step_id == s.id))
}

#[derive(Clone, Debug)]
pub struct PhaseButtonData {
  pub key: String,
  pub label: String,
  pub index: usize,
  pub is_done: bool,
  pub is_active: bool,
  pub is_disabled: bool,
  pub disabled_reason: Option<String>,
}

pub fn render_phase_button(
  data: &PhaseButtonData,
  mut active_phase: Signal<String>,
) -> Element {
  let PhaseButtonData {
    key,
    label,
    index,
    is_done,
    is_active,
    is_disabled,
    disabled_reason,
  } = data.clone();

  let number_class = if is_active {
    "bg-primary/20 text-primary"
  } else if is_disabled {
    "bg-muted text-muted-foreground/50"
  } else {
    "bg-secondary text-muted-foreground"
  };

  let text_class = if is_active {
    "font-medium"
  } else if is_disabled {
    "text-muted-foreground/50"
  } else {
    ""
  };

  let button_class = format!(
    "relative flex items-center gap-1.5 px-3 py-2 text-sm transition-colors {}",
    if is_active {
      "text-foreground"
    } else if is_disabled {
      "text-muted-foreground/50 cursor-not-allowed"
    } else {
      "text-muted-foreground hover:text-foreground/70"
    }
  );

  rsx! {
      div {
          class: "relative",
          button {
              key: "{key}",
              "type": "button",
              onclick: move |_| {
                  if !is_disabled {
                      active_phase.set(key.clone());
                  }
              },
              disabled: is_disabled,
              class: "{button_class}",
              aria_label: disabled_reason.as_ref().map_or_else(|| label.clone(), |reason| format!("{label} - {reason}")),
              if is_done {
                  svg {
                      width: "14",
                      height: "14",
                      view_box: "0 0 14 14",
                      fill: "none",
                      class: "text-chart-2",
                      path {
                          d: "M3.5 7L6 9.5L10.5 4.5",
                          stroke: "currentColor",
                          "stroke-width": "1.5",
                          "stroke-linecap": "round",
                          "stroke-linejoin": "round"
                      }
                  }
              } else {
                  span {
                      class: "flex h-4 w-4 items-center justify-center rounded-full text-xs {number_class}",
                      "{index + 1}"
                  }
              }
              span { class: "{text_class}", "{label}" }
              if is_active {
                  span { class: "absolute inset-x-0 -bottom-[9px] h-0.5 bg-primary" }
              }
          }
          if is_disabled {
              if let Some(reason) = &disabled_reason {
                  div {
                      class: "absolute left-0 top-full mt-2 z-50 w-64 rounded-md bg-popover px-3 py-2 text-xs text-popover-foreground shadow-md border border-border",
                      "{reason}"
                  }
              }
          }
      }
  }
}
