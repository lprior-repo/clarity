#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::components::{ArtifactPanel, GraphVisualizer, PlanningCoach, StateMachine};
use crate::types::{get_steps_for_phase, prompt_steps, Answer, PHASES, RightTab, TABS};

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
    key: String,
    label: String,
    index: usize,
    is_done: bool,
    is_active: bool,
}

/// Create phase button element from data
fn render_phase_button(data: &PhaseButtonData, mut active_phase: Signal<String>) -> Element {
    let PhaseButtonData {
        key,
        label,
        index,
        is_done,
        is_active,
    } = data.clone();

    let number_class = if is_active {
        "bg-primary/20 text-primary"
    } else {
        "bg-secondary text-muted-foreground"
    };

    let text_class = if is_active { "font-medium" } else { "" };

    let button_class = format!(
        "relative flex items-center gap-1.5 px-3 py-2 text-sm transition-colors {}",
        if is_active {
            "text-foreground"
        } else {
            "text-muted-foreground hover:text-foreground/70"
        }
    );

    rsx! {
        button {
            key: "{key}",
            "type": "button",
            onclick: move |_| active_phase.set(key.clone()),
            class: "{button_class}",
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
                answers: answers.clone(),
                active_phase: active_phase.clone()
            }
        },
        RightTab::Graph => rsx! {
            GraphVisualizer { answers: answers.clone() }
        },
        RightTab::State => rsx! {
            StateMachine {
                answers: answers.clone(),
                active_phase: active_phase.clone()
            }
        },
    }
}

/// Main home page - the Beads Planner UI
#[component]
pub fn HomePage() -> Element {
    let active_phase = use_signal(|| String::from("discover"));
    let answers = use_signal(|| Vec::<Answer>::new());
    let right_tab = use_signal(|| RightTab::Plan);

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

    // Pre-calculate phase states
    let active_phase_val = active_phase.read();
    let phase_buttons_data: Vec<PhaseButtonData> = PHASES
        .iter()
        .enumerate()
        .map(|(i, phase)| {
            let is_done = is_phase_done(phase.key, &answers.read());
            let is_active = *active_phase_val == phase.key;
            PhaseButtonData {
                key: phase.key.to_string(),
                label: phase.label.to_string(),
                index: i,
                is_done,
                is_active,
            }
        })
        .collect();
    drop(active_phase_val);

    // Pre-render phase buttons
    let active_phase_for_buttons = active_phase;
    let phase_buttons: Vec<Element> = phase_buttons_data
        .iter()
        .map(|data| render_phase_button(data, active_phase_for_buttons))
        .collect();

    // Pre-calculate tab states
    let right_tab_val = right_tab();
    let tab_buttons_data: Vec<TabButtonData> = TABS
        .iter()
        .map(|tab| TabButtonData {
            key: tab.key,
            label: tab.label.to_string(),
            is_active: right_tab_val == tab.key,
            right_tab_signal: right_tab,
        })
        .collect();

    // Pre-render tab buttons
    let tab_buttons: Vec<Element> = tab_buttons_data.into_iter().map(render_tab_button).collect();

    // Get current tab for content rendering
    let current_tab = right_tab();

    rsx! {
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
                        for button in phase_buttons.iter() {
                            {button.clone()}
                        }
                    }
                }

                // Progress counter
                span { class: "font-mono text-xs text-muted-foreground",
                    "{total_done}/{total_required}"
                }
            }

            // Main content
            div { class: "flex flex-1 overflow-hidden",
                // Left: Coach panel
                main { class: "flex-1 overflow-hidden border-r border-border",
                    PlanningCoach {
                        active_phase: active_phase.clone(),
                        answers: answers.clone(),
                        mut_answers: answers.clone(),
                        mut_active_phase: active_phase.clone()
                    }
                }

                // Right: Tabbed panel
                div { class: "flex w-[440px] shrink-0 flex-col lg:w-[500px]",
                    // Tab headers
                    div { class: "flex shrink-0 items-center border-b border-border",
                        for button in tab_buttons.iter() {
                            {button.clone()}
                        }
                    }

                    // Tab content
                    div { class: "flex-1 overflow-hidden",
                        {render_tab_content(current_tab, answers.clone(), active_phase.clone())}
                    }
                }
            }
        }
    }
}
