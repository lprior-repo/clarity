#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;
use tracing;

use crate::components::quality::{QualityScoreBar, MINIMUM_GATE};
use crate::components::{ArtifactPanel, GraphVisualizer, PlanningCoach, StateMachine};
// use crate::hooks::{use_quality_score, use_cached_quality_score};
use crate::lattice::quality::{
  calculate_quality, EarsRequirementRef, InversionControl, QualityScore,
};
use crate::types::{get_steps_for_phase, prompt_steps, Answer, RightTab, PHASES, TABS};

/// Check if a phase is complete based on answers
fn is_phase_done(phase_key: &str, answers: &[Answer]) -> bool {
  let steps = get_steps_for_phase(phase_key);
  let required_steps: Vec<_> = steps.iter().filter(|s| s.required).collect();
  if required_steps.is_empty() {
    return false;
  }
  required_steps
    .iter()
    .all(|s| answers.iter().any(|a| a.step_id == s.id))
}

/// Phase button data for rendering
#[derive(Clone, Debug)]
struct PhaseButtonData {
  key: &'static str,
  label: String,
  index: usize,
  is_done: bool,
  is_active: bool,
  is_disabled: bool,
  disabled_reason: Option<String>,
}

/// Create phase button element from data
fn render_phase_button(data: &PhaseButtonData, mut active_phase: Signal<String>) -> Element {
  let number_class = if data.is_active {
    "bg-primary/20 text-primary"
  } else if data.is_disabled {
    "bg-muted text-muted-foreground/50"
  } else {
    "bg-secondary text-muted-foreground"
  };

  let text_class = if data.is_active {
    "font-medium"
  } else if data.is_disabled {
    "text-muted-foreground/50"
  } else {
    ""
  };

  let button_class = format!(
    "relative flex items-center gap-1.5 px-3 py-2 text-sm transition-colors {}",
    if data.is_active {
      "text-foreground"
    } else if data.is_disabled {
      "text-muted-foreground/50 cursor-not-allowed"
    } else {
      "text-muted-foreground hover:text-foreground/70"
    }
  );

  let key_owned = data.key;
  let is_disabled = data.is_disabled;
  let aria_label: String = data
    .disabled_reason
    .as_ref()
    .map_or_else(|| data.label.clone(), |reason| format!("{} - {reason}", data.label));

  rsx! {
      div {
          class: "relative",
          button {
              key: "{data.key}",
              "type": "button",
              onclick: move |_| {
                  if !is_disabled {
                      active_phase.set(key_owned.to_string());
                  }
              },
              disabled: data.is_disabled,
              class: "{button_class}",
              aria_label: aria_label,
              if data.is_done {
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
                       "{data.index + 1}"
                   }
               }
               span { class: "{text_class}", "{data.label}" }
               if data.is_active {
                   span { class: "absolute inset-x-0 -bottom-[9px] h-0.5 bg-primary" }
               }
           }
           // Tooltip for disabled button
           if data.is_disabled {
               if let Some(reason) = &data.disabled_reason {
                   div {
                       class: "absolute left-0 top-full mt-2 z-50 w-64 rounded-md bg-popover px-3 py-2 text-xs text-popover-foreground shadow-md border border-border",
                       "{reason}"
                   }
               }
           }
      }
  }
}

/// Tab button data for rendering
#[derive(Clone, Debug)]
struct TabButtonData {
  key: RightTab,
  label: String,
  is_active: bool,
  right_tab_signal: Signal<RightTab>,
}

/// Create tab button element from data
fn render_tab_button(data: TabButtonData) -> Element {
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

/// Render the right panel content based on active tab
fn render_tab_content(
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

/// Main home page - the Beads Planner UI
#[component]
pub fn HomePage() -> Element {
  let active_phase = use_signal(|| String::from("discover"));
  let answers = use_signal(Vec::<Answer>::new);
  let right_tab = use_signal(|| RightTab::Plan);

  // EARS requirements (empty for now - will be populated by lattice processing)
  let ears_requirements = use_signal(Vec::<EarsRequirementRef>::new);

  // Quality scoring - manual implementation to avoid type inference issues
  let quality_score = use_signal(|| Option::<QualityScore>::None);

  // Recalculate quality score when answers change
  use_effect({
    let mut quality_score = quality_score;
    move || {
      let answers_clone = answers.read().clone();
      let ears_clone = ears_requirements.read().clone();

      if answers_clone.is_empty() {
        *quality_score.write() = None;
        return;
      }

      let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
      };

      let result = calculate_quality(&answers_clone, &ears_clone, &inversion);
      match result {
        Ok(score) => quality_score.set(Some(score)),
        Err(ref e) => {
          tracing::debug!("quality calculation failed: {:?}", e);
          quality_score.set(None)
        }
      }
    }
  });

  // Calculate progress using functional patterns
  let total_required = prompt_steps().iter().filter(|s| s.required).count();

  let total_done = answers
    .read()
    .iter()
    .filter(|a| {
      prompt_steps()
        .iter()
        .any(|s| s.id == a.step_id && s.required && a.value != "(skipped)")
    })
    .count();

  // Check if quality gate is passed
  let quality_score_ref: &Signal<Option<crate::lattice::quality::QualityScore>> = &quality_score;
  let score_read = quality_score_ref.read();
  let passes_gate: bool = score_read
    .as_ref()
    .is_some_and(|s: &crate::lattice::quality::QualityScore| s.passes(MINIMUM_GATE));
  drop(score_read);

  // Pre-calculate phase states
  let active_phase_val = active_phase.read();
  let phase_buttons_data: Vec<PhaseButtonData> = PHASES
    .iter()
    .enumerate()
    .map(|(i, phase)| {
      let is_done = is_phase_done(phase.key, &answers.read());
      let is_active = *active_phase_val == phase.key;

      // Check if phase should be disabled due to quality gate
      let (is_disabled, disabled_reason) = if phase.key == "develop" || phase.key == "deliver" {
        if is_phase_done("discover", &answers.read()) && !passes_gate {
          (
            true,
            Some(format!(
              "Quality score must be at least {MINIMUM_GATE} to proceed"
            )),
          )
        } else {
          (false, None)
        }
      } else {
        (false, None)
      };

      PhaseButtonData {
        key: phase.key,
        label: phase.label.to_string(),
        index: i,
        is_done,
        is_active,
        is_disabled,
        disabled_reason,
      }
    })
    .collect();
  drop(active_phase_val);

  // Pre-calculate tab states
  let right_tab_val = right_tab();

  // Get current tab for content rendering
  let current_tab = right_tab();

  // Quality details toggle
  let show_quality_details = use_signal(|| false);

  // Get overall score for header display
  let overall_score: Option<u8> = quality_score.read().as_ref().map(|s| s.overall);

  // Calculate quality badge color class
  let quality_badge_class: &'static str = overall_score.map_or("", |score| {
    if score >= MINIMUM_GATE {
      "text-chart-2 border-chart-2/30 bg-chart-2/10"
    } else if score >= 50 {
      "text-chart-3 border-chart-3/30 bg-chart-3/10"
    } else {
      "text-chart-4 border-chart-4/30 bg-chart-4/10"
    }
  });

  let result: Element = rsx! {
      div { class: "flex h-screen flex-col overflow-hidden bg-background",
          // Top bar
          header {
              class: "flex shrink-0 items-center justify-between border-b border-border px-5 py-2",
              div { class: "flex items-center gap-6",
                  // Logo
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

                  // Phase navigation
                  nav { class: "flex items-center", "aria-label": "Planning phases",
                      for data in phase_buttons_data.iter() {
                          {render_phase_button(data, active_phase)}
                      }
                  }
              }

              // Progress counter and quality score
              div {
                  class: "flex items-center gap-4",
                  span { class: "font-mono text-xs text-muted-foreground",
                      "{total_done}/{total_required}"
                  }
                  // Quality score badge (shown when in Discover phase)
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

          // Main content
          div { class: "flex flex-1 overflow-hidden",
              // Left: Coach panel
              main { class: "flex-1 overflow-hidden border-r border-border flex flex-col",
                  // Quality score bar (shown in Discover phase)
                  if *active_phase.read() == "discover" {
                      div {
                          class: "shrink-0 border-b border-border bg-muted/30 px-6 py-4",
                          QualityScoreBar {
                              score: quality_score,
                              minimum_gate: MINIMUM_GATE,
                              show_details: show_quality_details,
                          }
                      }
                  }

                  // Planning coach interface
                  div {
                      class: "flex-1 overflow-hidden",
                      PlanningCoach {
                          active_phase: active_phase,
                          answers: answers,
                          mut_answers: answers,
                          mut_active_phase: active_phase
                      }
                  }
              }

              // Right: Tabbed panel
              div { class: "flex w-[440px] shrink-0 flex-col lg:w-[500px]",
                  // Tab headers
                  div { class: "flex shrink-0 items-center border-b border-border",
                      for tab in TABS.iter() {
                          {render_tab_button(TabButtonData {
                              key: tab.key,
                              label: tab.label.to_string(),
                              is_active: right_tab_val == tab.key,
                              right_tab_signal: right_tab,
                          })}
                      }
                  }

                  // Tab content
                  div { class: "flex-1 overflow-hidden",
                      {render_tab_content(current_tab, answers, active_phase)}
                  }
              }
          }
      }
  };

  result
}
