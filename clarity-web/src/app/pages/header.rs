#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::components::quality::MINIMUM_GATE;

pub struct HeaderRenderData {
  pub phase_buttons: Vec<Element>,
  pub total_done: usize,
  pub total_required: usize,
  pub overall_score: Option<u8>,
  pub quality_badge_class: &'static str,
  pub active_phase: Signal<String>,
}

pub fn render_header(data: HeaderRenderData) -> Element {
  let HeaderRenderData {
    phase_buttons,
    total_done,
    total_required,
    overall_score,
    quality_badge_class,
    active_phase,
  } = data;

  rsx! {
      header {
          class: "flex shrink-0 items-center justify-between border-b border-border px-5 py-2",
          div { class: "flex items-center gap-6",
              div { class: "flex items-center gap-2",
                  div {
                      class: "flex h-6 w-6 items-center justify-center rounded-md bg-primary",
                      svg {
                          width: "14",
                          height: "14",
                          view_box: "0 0 14 14",
                          fill: "none",
                          class: "text-primary-foreground",
                          circle { cx: "4", cy: "4", r: "2", fill: "currentColor" }
                          circle { cx: "10", cy: "4", r: "2", fill: "currentColor" }
                          circle { cx: "7", cy: "10", r: "2", fill: "currentColor" }
                          path {
                              d: "M4 4L10 4M4 4L7 10M10 4L7 10",
                              stroke: "currentColor",
                              "stroke-width": "1",
                              opacity: "0.5"
                          }
                      }
                  }
                  span { class: "text-sm font-bold tracking-tight text-foreground",
                      "Clarity Planner"
                  }
              }
              nav { class: "flex items-center", "aria-label": "Planning phases",
                  for button in phase_buttons.iter() {
                      {button.clone()}
                  }
              }
          }
          div {
              class: "flex items-center gap-4",
              span { class: "font-mono text-xs text-muted-foreground",
                  "{total_done}/{total_required}"
              }
              if *active_phase.read() == "discover" {
                  div {
                      class: "flex items-center gap-2",
                      if let Some(score) = overall_score {
                          span {
                              class: format!("inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-semibold {}", quality_badge_class),
                              span { class: "opacity-70", "Quality:" }
                              span { class: "font-mono", "{score}" }
                              if score < MINIMUM_GATE {
                                  span {
                                      class: "ml-1 opacity-70",
                                      "(need {MINIMUM_GATE})"
                                  }
                              }
                          }
                      }
                  }
              }
          }
      }
  }
}
