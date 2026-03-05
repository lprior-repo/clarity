#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::lattice::ears::parse_requirements;
use crate::lattice::quality::{
  calculate_quality, DimensionScore, EarsRequirementRef, InversionControl, QualityDimension,
  QualityIssue, QualityScore,
};
use crate::types::{get_steps_for_phase, prompt_steps, Answer, PromptStep};

const PHASES: &[&str] = &["discover", "define", "develop", "deliver"];

/// Phase state colors
fn phase_colors(phase: &str) -> (&'static str, &'static str, &'static str) {
  match phase {
    "discover" => ("ring-chart-1/50", "bg-chart-1/10", "text-chart-1"),
    "define" => ("ring-chart-5/50", "bg-chart-5/10", "text-chart-5"),
    "develop" => ("ring-chart-3/50", "bg-chart-3/10", "text-chart-3"),
    "deliver" => ("ring-chart-2/50", "bg-chart-2/10", "text-chart-2"),
    _ => ("ring-border/50", "bg-secondary/10", "text-muted-foreground"),
  }
}

/// Phase state information
#[derive(Clone, Debug)]
struct PhaseState {
  phase: &'static str,
  total: usize,
  done: usize,
  is_complete: bool,
  is_active: bool,
  steps: Vec<&'static PromptStep>,
}

/// Step state for rendering
#[derive(Clone, Debug)]
struct StepRenderState {
  step: &'static PromptStep,
  is_done: bool,
  is_current: bool,
}

/// Progress bar step data
#[derive(Clone, Debug)]
struct ProgressStepData {
  index: usize,
  class_name: String,
}

/// Phase render data
#[derive(Clone, Debug)]
struct PhaseRenderData {
  phase_idx: usize,
  phase: &'static str,
  ring: &'static str,
  bg: &'static str,
  text: &'static str,
  is_complete: bool,
  is_active: bool,
  done: usize,
  total: usize,
  step_states: Vec<StepRenderState>,
}

/// Build phase states from answers and active phase
fn build_phase_states(answers: &[Answer], active_phase: &str) -> Vec<PhaseState> {
  let completed_ids: Vec<&str> = answers.iter().map(|a| a.step_id.as_str()).collect();

  PHASES
    .iter()
    .map(|&phase| {
      let steps = get_steps_for_phase(phase);
      let required: Vec<_> = steps.iter().filter(|s| s.required).collect();
      let done_count = required
        .iter()
        .filter(|&&s| completed_ids.contains(&s.id.as_str()))
        .count();
      let is_complete = !required.is_empty()
        && required
          .iter()
          .all(|&&s| completed_ids.contains(&s.id.as_str()));
      let is_active = active_phase == phase;

      PhaseState {
        phase,
        total: required.len(),
        done: done_count,
        is_complete,
        is_active,
        steps,
      }
    })
    .collect()
}

/// Build progress step data
fn build_progress_steps(answers: &[Answer]) -> (usize, usize, Vec<ProgressStepData>) {
  let completed_ids: Vec<&str> = answers.iter().map(|a| a.step_id.as_str()).collect();

  let all_steps: Vec<_> = prompt_steps().iter().filter(|s| s.required).collect();
  let total_steps = all_steps.len();
  let completed_steps = all_steps
    .iter()
    .filter(|&&s| completed_ids.contains(&s.id.as_str()))
    .count();

  let current_global_idx = all_steps
    .iter()
    .position(|s| !completed_ids.contains(&s.id.as_str()));

  let progress_steps: Vec<ProgressStepData> = all_steps
    .iter()
    .enumerate()
    .map(|(i, step)| {
      let is_completed = completed_ids.contains(&step.id.as_str());
      let is_current = current_global_idx == Some(i);
      let class_name = if is_completed {
        "bg-primary".to_string()
      } else if is_current {
        "animate-pulse-glow bg-primary/40".to_string()
      } else {
        "bg-secondary".to_string()
      };
      ProgressStepData {
        index: i,
        class_name,
      }
    })
    .collect();

  (completed_steps, total_steps, progress_steps)
}

/// Build phase render data
fn build_phase_render_data(
  phase_states: &[PhaseState],
  answers: &[Answer],
) -> Vec<PhaseRenderData> {
  let completed_ids: Vec<&str> = answers.iter().map(|a| a.step_id.as_str()).collect();

  phase_states
    .iter()
    .enumerate()
    .map(|(phase_idx, ps)| {
      let (ring, bg, text) = phase_colors(ps.phase);
      let step_states: Vec<StepRenderState> = ps
        .steps
        .iter()
        .map(|step| {
          let is_done = completed_ids.contains(&step.id.as_str());
          let is_current = !is_done;
          StepRenderState {
            step,
            is_done,
            is_current,
          }
        })
        .collect();

      PhaseRenderData {
        phase_idx,
        phase: ps.phase,
        ring,
        bg,
        text,
        is_complete: ps.is_complete,
        is_active: ps.is_active,
        done: ps.done,
        total: ps.total,
        step_states,
      }
    })
    .collect()
}

/// Render a progress step
fn render_progress_step(data: &ProgressStepData) -> Element {
  rsx! {
      div {
          key: "{data.index}",
          class: format!("h-1.5 flex-1 rounded-full transition-all duration-500 {}", data.class_name)
      }
  }
}

/// Render a step sub-state
fn render_step_state(data: &StepRenderState) -> Element {
  let text_class = if data.is_done {
    "text-muted-foreground line-through"
  } else if data.is_current {
    "font-medium text-foreground"
  } else {
    "text-muted-foreground/40"
  };

  rsx! {
      div { class: "flex items-center gap-2",
          if data.is_done {
              svg {
                  width: "12",
                  height: "12",
                  view_box: "0 0 12 12",
                  fill: "none",
                  class: "text-chart-2 shrink-0",
                  path {
                      d: "M3 6L5 8L9 4",
                      stroke: "currentColor",
                      "stroke-width": "1.5",
                      "stroke-linecap": "round",
                      "stroke-linejoin": "round"
                  }
              }
          } else if data.is_current {
              span {
                  class: "relative flex h-3 w-3 shrink-0",
                  span { class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-primary/40" }
                  span { class: "relative inline-flex h-3 w-3 rounded-full bg-primary" }
              }
          } else {
              span { class: "h-3 w-3 shrink-0 rounded-full border border-border" }
          }
          span {
              class: format!("text-xs {}", text_class),
              "{data.step.title}"
          }
      }
  }
}

/// Render a phase card
fn render_phase_card(data: &PhaseRenderData) -> Element {
  let animation_delay = data.phase_idx * 80;

  let container_class = format!(
    "rounded-lg border p-3 transition-all duration-300 {}",
    if data.is_active {
      format!("ring-2 {} border-transparent {}", data.ring, data.bg)
    } else if data.is_complete {
      "border-border bg-card/50".to_string()
    } else {
      "border-border/50 bg-transparent".to_string()
    }
  );

  let phase_text_class = format!(
    "text-sm font-medium capitalize {}",
    if data.is_active {
      data.text
    } else if data.is_complete {
      "text-foreground/70"
    } else {
      "text-muted-foreground/50"
    }
  );

  let step_elements: Vec<Element> = data.step_states.iter().map(render_step_state).collect();

  rsx! {
      div {
          key: "{data.phase}",
          class: "animate-fade-up",
          style: format!("animation-delay: {}ms; animation-fill-mode: both;", animation_delay),
          div { class: "{container_class}",

              // Phase header
              div { class: "flex items-center justify-between",
                  div { class: "flex items-center gap-2",
                      if data.is_complete {
                          svg {
                              width: "16",
                              height: "16",
                              view_box: "0 0 16 16",
                              fill: "none",
                              class: "text-chart-2",
                              circle { cx: "8", cy: "8", r: "7", stroke: "currentColor", "stroke-width": "1.5" }
                              path {
                                  d: "M5 8L7 10L11 6",
                                  stroke: "currentColor",
                                  "stroke-width": "1.5",
                                  "stroke-linecap": "round",
                                  "stroke-linejoin": "round"
                              }
                          }
                      } else if data.is_active {
                          span {
                              class: format!("flex h-4 w-4 items-center justify-center rounded-full {}", data.bg),
                              span { class: format!("h-2 w-2 rounded-full bg-current {} animate-pulse", data.text) }
                          }
                      } else {
                          span {
                              class: "flex h-4 w-4 items-center justify-center rounded-full bg-secondary",
                              span { class: "h-1.5 w-1.5 rounded-full bg-muted-foreground/30" }
                          }
                      }
                      span { class: "{phase_text_class}", "{data.phase}" }
                  }
                  span { class: "font-mono text-xs text-muted-foreground/50", "{data.done}/{data.total}" }
              }

              // Step sub-states
              if data.is_active && !data.step_states.is_empty() {
                  div { class: "mt-3 space-y-1.5 pl-6",
                      for step in step_elements.iter() {
                          {step.clone()}
                      }
                  }
              }

              // Transition arrow
              if data.phase_idx < PHASES.len() - 1 {
                  div { class: "mt-2 flex justify-center",
                      svg {
                          width: "12",
                          height: "16",
                          view_box: "0 0 12 16",
                          fill: "none",
                          class: "text-border",
                          path {
                              d: "M6 0V12M2 8L6 12L10 8",
                              stroke: "currentColor",
                              "stroke-width": "1.5",
                              "stroke-linecap": "round",
                              "stroke-linejoin": "round"
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Build EARS requirements from answers for quality scoring
fn build_ears_requirements(answers: &[Answer]) -> Vec<EarsRequirementRef> {
  // Extract requirements from answers
  let requirement_text: String = answers
    .iter()
    .filter(|a| {
      let lower = a.value.to_lowercase();
      lower.contains("shall")
        || lower.contains("when")
        || lower.contains("during")
        || lower.contains("if")
        || lower.contains("where")
    })
    .map(|a| a.value.as_str())
    .collect::<Vec<_>>()
    .join("\n");

  if requirement_text.is_empty() {
    return Vec::new();
  }

  let ears_output = parse_requirements(&requirement_text);

  ears_output
    .requirements
    .iter()
    .enumerate()
    .map(|(i, req)| {
      let text = match req {
        crate::lattice::ears::EarsRequirement::Ubiquitous { actor, action } => {
          format!("{actor} shall {action}")
        }
        crate::lattice::ears::EarsRequirement::StateDriven {
          actor,
          trigger,
          action,
        } => {
          format!("When {trigger}, {actor} shall {action}")
        }
        crate::lattice::ears::EarsRequirement::EventDriven {
          actor,
          trigger,
          action,
        } => {
          format!("During {trigger}, {actor} shall {action}")
        }
        crate::lattice::ears::EarsRequirement::Unwanted {
          actor,
          condition,
          action,
        } => {
          format!("If {condition}, {actor} shall NOT {action}")
        }
        crate::lattice::ears::EarsRequirement::Optional {
          actor,
          condition,
          action,
        } => {
          format!("Where {condition}, {actor} shall {action}")
        }
      };

      EarsRequirementRef {
        id: format!("req-{i}"),
        text,
        has_acceptance_criteria: false, // Will be detected in real implementation
      }
    })
    .collect()
}

/// Calculate quality invariants from answers
fn build_quality_invariants(answers: &[Answer]) -> Option<QualityScore> {
  if answers.is_empty() {
    return None;
  }

  let ears_requirements = build_ears_requirements(answers);
  let inversion = InversionControl {
    has_inversion_tests: false,
    inverted_count: 0,
  };

  // Convert Answer types to quality module's Answer type
  let quality_answers: Vec<crate::lattice::quality::Answer> = answers
    .iter()
    .map(|a| crate::lattice::quality::Answer {
      step_id: a.step_id.clone(),
      value: a.value.clone(),
      timestamp: a.timestamp.clone(),
    })
    .collect();

  calculate_quality(&quality_answers, &ears_requirements, &inversion).ok()
}

/// KIRK invariant display card
#[component]
fn InvariantCard(
  dimension: QualityDimension,
  score: Option<DimensionScore>,
  issues: Vec<QualityIssue>,
) -> Element {
  let dimension_label = dimension.label();
  let dimension_desc = dimension.description();

  let score_value = score.map_or(0, |s| s.score);
  let score_color = if score_value >= 80 {
    "text-chart-2"
  } else if score_value >= 50 {
    "text-chart-3"
  } else {
    "text-chart-4"
  };

  let relevant_issues: Vec<_> = issues.iter().filter(|i| i.dimension == dimension).collect();

  rsx! {
      div { class: "rounded-lg border border-border bg-card px-3 py-2.5",
          div { class: "flex items-center justify-between",
              div { class: "min-w-0 flex-1",
                  span { class: "block text-xs font-medium uppercase tracking-wider text-muted-foreground/70", "{dimension_label}" }
                  p { class: "mt-0.5 text-xs text-muted-foreground/60", "{dimension_desc}" }
              }
              div { class: "shrink-0 text-right",
                  span { class: "font-mono text-2xl font-semibold {score_color}", "{score_value}" }
                  span { class: "text-xs text-muted-foreground", "%" }
              }
          }
          if !relevant_issues.is_empty() {
              div { class: "mt-2 space-y-1 border-t border-border/50 pt-2",
                  for issue in relevant_issues.iter() {
                      div { class: "flex items-start gap-2 text-xs",
                           svg {
                              width: "12",
                              height: "12",
                              view_box: "0 0 12 12",
                              fill: "none",
                               class: format!(
                                   "shrink-0 mt-0.5 {}",
                                   match issue.severity {
                                       crate::lattice::quality::IssueSeverity::Critical
                                       | crate::lattice::quality::IssueSeverity::Error => "text-chart-4",
                                       crate::lattice::quality::IssueSeverity::Warning => "text-chart-3",
                                   }
                               ),
                              path {
                                  d: "M6 1C3.2 1 1 3.2 1 6C1 8.8 3.2 11 6 11C8.8 11 11 8.8 11 6C11 3.2 8.8 1 6 1ZM6 8.5C5.4 8.5 5 8.1 5 7.5C5 6.9 5.4 6.5 6 6.5C6.6 6.5 7 6.9 7 7.5C7 8.1 6.6 8.5 6 8.5ZM6 5.5C5.4 5.5 5 5.1 5 4.5V3C5 2.4 5.4 2 6 2C6.6 2 7 2.4 7 3V4.5C7 5.1 6.6 5.5 6 5.5Z",
                                  fill: "currentColor"
                              }
                          }
                          span { class: "text-muted-foreground", "{issue.message}" }
                      }
                  }
              }
          }
      }
  }
}

/// `StateMachine` component - visualizes planning progress as a state machine
#[component]
pub fn StateMachine(answers: Signal<Vec<Answer>>, active_phase: Signal<String>) -> Element {
  let answers_guard = answers.read();
  let active_phase_str = active_phase.read();

  let phase_states = build_phase_states(&answers_guard, &active_phase_str);
  let (completed_steps, total_steps, progress_steps) = build_progress_steps(&answers_guard);
  let phase_render_data = build_phase_render_data(&phase_states, &answers_guard);
  let quality_invariants = build_quality_invariants(&answers_guard);

  drop(answers_guard);
  drop(active_phase_str);

  let progress_elements: Vec<Element> = progress_steps.iter().map(render_progress_step).collect();

  let phase_elements: Vec<Element> = phase_render_data.iter().map(render_phase_card).collect();

  // Build invariants section
  let invariants_section = quality_invariants.as_ref().map(|score| {
      let overall_score = score.overall;
      let issues = &score.issues;

      let invariant_cards: Vec<Element> = QualityDimension::all()
        .iter()
        .map(|dim| {
          let dim_score = score.get_dimension(*dim).copied();
          let dim_issues: Vec<QualityIssue> = issues
            .iter()
            .filter(|i| i.dimension == *dim)
            .cloned()
            .collect();
          rsx! {
              InvariantCard {
                  dimension: *dim,
                  score: dim_score,
                  issues: dim_issues
              }
          }
        })
        .collect();

      rsx! {
          div { class: "space-y-3",
              // Overall quality header
              div { class: "flex items-center justify-between border-b border-border pb-2",
                  div { class: "flex items-center gap-2",
                      span { class: "text-xs font-medium uppercase tracking-widest text-muted-foreground/70", "KIRK Invariants" }
                      div {
                          class: "group relative",
                          svg {
                              width: "14",
                              height: "14",
                              view_box: "0 0 14 14",
                              fill: "none",
                              class: "text-muted-foreground/40 cursor-help",
                              circle { cx: "7", cy: "7", r: "6", stroke: "currentColor", "stroke-width": "1" }
                              path { d: "M7 4V7M7 10H7.01", stroke: "currentColor", "stroke-width": "1.5", "stroke-linecap": "round" }
                          }
                          div {
                              class: "absolute left-full ml-2 hidden w-48 rounded-md bg-popover px-3 py-2 text-xs text-popover-foreground shadow-md group-hover:block z-50",
                              "Keep Invariants Regular and Known - Quality metrics for your requirements"
                          }
                      }
                  }
                  div { class: "flex items-center gap-2",
                      span {
                          class: format!("font-mono text-2xl font-semibold {}",
                              if overall_score >= 80 { "text-chart-2" }
                              else if overall_score >= 50 { "text-chart-3" }
                              else { "text-chart-4" }),
                          "{overall_score}"
                      }
                      span { class: "text-xs text-muted-foreground", "overall" }
                  }
              }

              // Dimension cards
              div { class: "grid grid-cols-1 gap-2",
                  for card in invariant_cards.iter() {
                      {card.clone()}
                  }
              }
          }
      }
  });

  rsx! {
      div { class: "flex h-full flex-col overflow-y-auto",
          div { class: "flex flex-col gap-6 p-4",
              // Overall progress
              div { class: "space-y-2",
                  div { class: "flex items-center justify-between",
                      span { class: "text-xs font-medium uppercase tracking-widest text-muted-foreground/70", "Progress" }
                      span { class: "font-mono text-xs text-muted-foreground", "{completed_steps}/{total_steps}" }
                  }
                  div { class: "flex gap-1",
                      for step in progress_elements.iter() {
                          {step.clone()}
                      }
                  }
              }

              // KIRK Invariants section
              if let Some(invariants) = invariants_section {
                  {invariants}
              }

              // Phase state cards
              div { class: "space-y-3",
                  div { class: "text-xs font-medium uppercase tracking-widest text-muted-foreground/70", "Phase States" }
                  div { class: "flex flex-col gap-3",
                      for phase in phase_elements.iter() {
                          {phase.clone()}
                      }
                  }
              }
          }
      }
  }
}
