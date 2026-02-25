#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::types::{get_steps_for_phase, prompt_steps, Answer, PromptStep};

const PHASES: &[&str] = &["discover", "define", "develop", "deliver"];

/// Phase state colors
fn phase_colors(phase: &str) -> (&'static str, &'static str, &'static str) {
    match phase {
        "discover" => ("ring-chart-1/50", "bg-chart-1/10", "text-chart-1"),
        "define" => ("ring-chart-5/50", "bg-chart-5/10", "text-chart-5"),
        "develop" => ("ring-chart-3/50", "bg-chart-3/10", "text-chart-3"),
        "deliver" => ("ring-chart-2/50", "bg-chart-2/10", "text-chart-2"),
        _ => (
            "ring-border/50",
            "bg-secondary/10",
            "text-muted-foreground",
        ),
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

    let step_elements: Vec<Element> = data
        .step_states
        .iter()
        .map(render_step_state)
        .collect();

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

/// StateMachine component - visualizes planning progress as a state machine
#[component]
pub fn StateMachine(answers: Signal<Vec<Answer>>, active_phase: Signal<String>) -> Element {
    let answers_guard = answers.read();
    let active_phase_str = active_phase.read();

    let phase_states = build_phase_states(&answers_guard, &active_phase_str);
    let (completed_steps, total_steps, progress_steps) = build_progress_steps(&answers_guard);
    let phase_render_data = build_phase_render_data(&phase_states, &answers_guard);

    drop(answers_guard);
    drop(active_phase_str);

    let progress_elements: Vec<Element> = progress_steps
        .iter()
        .map(render_progress_step)
        .collect();

    let phase_elements: Vec<Element> = phase_render_data
        .iter()
        .map(render_phase_card)
        .collect();

    rsx! {
        div { class: "flex h-full flex-col gap-6 p-4",
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

            // Phase state cards
            div { class: "flex flex-1 flex-col gap-3",
                for phase in phase_elements.iter() {
                    {phase.clone()}
                }
            }
        }
    }
}
