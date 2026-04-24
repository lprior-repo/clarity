#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::components::{ArtifactPanel, GraphVisualizer, StateMachine};
use crate::types::{Answer, RightTab};

#[derive(Clone, Debug)]
pub struct TabButtonData {
  pub key: RightTab,
  pub label: String,
  pub is_active: bool,
  pub right_tab_signal: Signal<RightTab>,
}

pub fn render_tab_button(data: TabButtonData) -> Element {
  let TabButtonData {
    key,
    label,
    is_active,
    mut right_tab_signal,
  } = data;

  let button_class = format!(
    "relative flex items-center gap-1.5 px-4 py-2.5 text-xs font-medium transition-colors {}",
    if is_active {
      "text-foreground"
    } else {
      "text-muted-foreground hover:text-foreground/70"
    }
  );

  let icon = match key {
    RightTab::Graph => rsx! {
        svg {
            width: "12",
            height: "12",
            view_box: "0 0 16 16",
            fill: "none",
            class: "shrink-0",
            circle { cx: "4", cy: "4", r: "2", stroke: "currentColor", "stroke-width": "1.2" }
            circle { cx: "12", cy: "4", r: "2", stroke: "currentColor", "stroke-width": "1.2" }
            circle { cx: "8", cy: "12", r: "2", stroke: "currentColor", "stroke-width": "1.2" }
            path { d: "M5.5 5.5L7 10.5M10.5 5.5L9 10.5", stroke: "currentColor", "stroke-width": "1", opacity: "0.5" }
        }
    },
    RightTab::State => rsx! {
        svg {
            width: "12",
            height: "12",
            view_box: "0 0 16 16",
            fill: "none",
            class: "shrink-0",
            rect { x: "2", y: "2", width: "5", height: "5", rx: "1", stroke: "currentColor", "stroke-width": "1.2" }
            rect { x: "9", y: "9", width: "5", height: "5", rx: "1", stroke: "currentColor", "stroke-width": "1.2" }
            path { d: "M7 4.5H9.5V9.5H11.5", stroke: "currentColor", "stroke-width": "1", "stroke-linecap": "round" }
        }
    },
    RightTab::Plan => rsx! {
        svg {
            width: "12",
            height: "12",
            view_box: "0 0 16 16",
            fill: "none",
            class: "shrink-0",
            rect { x: "2", y: "2", width: "12", height: "12", rx: "2", stroke: "currentColor", "stroke-width": "1.2" }
            path { d: "M5 6H11M5 8.5H9M5 11H7", stroke: "currentColor", "stroke-width": "1", "stroke-linecap": "round", opacity: "0.6" }
        }
    },
  };

  rsx! {
      button {
          key: "{key:?}",
          "type": "button",
          onclick: move |_| right_tab_signal.set(key),
          class: "{button_class}",
          {icon}
          "{label}"
          if is_active {
              span { class: "absolute inset-x-0 -bottom-px h-0.5 bg-primary" }
          }
      }
  }
}

pub fn render_tab_content(
  tab: RightTab,
  answers: Signal<Vec<Answer>>,
  active_phase: Signal<String>,
) -> Element {
  match tab {
    RightTab::Plan => rsx! {
        ArtifactPanel {
            answers: answers,
            active_phase: active_phase
        }
    },
    RightTab::Graph => rsx! {
        GraphVisualizer { answers: answers }
    },
    RightTab::State => rsx! {
        StateMachine {
            answers: answers,
            active_phase: active_phase
        }
    },
  }
}
