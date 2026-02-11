//! PlanningCoach component - Interactive guided planning interface
//!
//! Ported from planning-coach.tsx with functional Rust patterns,
//! signal-based chat, use_future for terminal animation, and debounced input.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::planner::types_coach::{CoachAnswer, CoachStep};
use crate::planner::prompts::get_steps_for_phase;
use crate::planner::types::DiamondPhase;
use dioxus::prelude::*;
use std::collections::HashMap;

/// Terminal command structure
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalCommand {
    /// Agent name (e.g., "planner", "claude-code")
    pub agent: String,

    /// Command to execute
    pub cmd: String,

    /// Expected output
    pub output: String,
}

/// Get commands for a coaching step based on the user's answer
///
/// Generates terminal commands that would be executed based on the step ID
/// and user input value.
#[must_use]
pub fn get_commands_for_step(step_id: &str, value: &str) -> Vec<TerminalCommand> {
    let v = value.chars().take(60).collect::<String>();

    match step_id {
        "problem" => vec![
            TerminalCommand {
                agent: "planner".into(),
                cmd: format!("bd init --project beads-plan"),
                output: "Initialized .beads/ in current directory".into(),
            },
            TerminalCommand {
                agent: "planner".into(),
                cmd: format!("bd create --type epic --title \"Problem: {}...\"", v),
                output: "Created bd-a1f0  Problem Statement".into(),
            },
        ],
        "antithesis" => vec![
            TerminalCommand {
                agent: "planner".into(),
                cmd: format!("bd update bd-a1f0 --label antithesis --note \"{}...\"", v),
                output: "Updated bd-a1f0  +label:antithesis".into(),
            },
        ],
        "solution" => vec![
            TerminalCommand {
                agent: "planner".into(),
                cmd: format!("bd create --type epic --title \"Solution: {}...\"", v),
                output: "Created bd-b2e1  Solution".into(),
            },
            TerminalCommand {
                agent: "planner".into(),
                cmd: "bd dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from".into(),
                output: "Linked bd-b2e1 -> bd-a1f0 (discovered-from)".into(),
            },
        ],
        "persona" => vec![
            TerminalCommand {
                agent: "planner".into(),
                cmd: format!("bd create --type task --parent bd-b2e1 --title \"Persona: {}...\"", v),
                output: "Created bd-b2e1.1  Persona definition".into(),
            },
        ],
        "scenario" => vec![
            TerminalCommand {
                agent: "planner".into(),
                cmd: "bd create --type task --parent bd-b2e1 --title \"North Star Scenario\"".into(),
                output: "Created bd-b2e1.2  North Star Scenario".into(),
            },
            TerminalCommand {
                agent: "planner".into(),
                cmd: "bd dep add bd-b2e1.2 --related bd-b2e1.1".into(),
                output: "Linked bd-b2e1.2 -> bd-b2e1.1 (related)".into(),
            },
        ],
        "use-cases" => {
            let lines = value
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>();

            lines
                .iter()
                .enumerate()
                .map(|(i, &uc)| TerminalCommand {
                    agent: "planner".into(),
                    cmd: format!("bd create --type feature --title \"{}...\"", uc.chars().take(55).collect::<String>()),
                    output: format!("Created bd-c{i}d{i}  {}...", uc.chars().take(30).collect::<String>()),
                })
                .collect()
        },
        "constraints" => vec![
            TerminalCommand {
                agent: "planner".into(),
                cmd: format!("bd update bd-b2e1 --label stack --note \"{}...\"", v),
                output: "Updated bd-b2e1  +label:stack".into(),
            },
        ],
        "bd-show" => vec![
            TerminalCommand {
                agent: "planner".into(),
                cmd: "bd show --json".into(),
                output: r#"[{"id":"bd-a1f0","title":"Problem Statement","type":"epic","status":"completed","labels":["problem","core"]},{"id":"bd-b2e1","title":"Solution","type":"epic","status":"in-progress","labels":["solution","core"]},{"id":"bd-b2e1.1","title":"Persona definition","type":"task","status":"completed"},{"id":"bd-b2e1.2","title":"North Star Scenario","type":"task","status":"completed"}]"#.into(),
            },
            TerminalCommand {
                agent: "planner".into(),
                cmd: "bd show --tree".into(),
                output: "bd-a1f0 [epic] └─┬ bd-b2e1 [epic]\n                            ├─ bd-b2e1.1 [task]\n                            └─ bd-b2e1.2 [task]".into(),
            },
            TerminalCommand {
                agent: "planner".into(),
                cmd: "bd show --status".into(),
                output: "📊 Bead Status: 2 completed, 1 in-progress, 0 blocked".into(),
            },
        ],
        "tasks" => {
            let lines = value
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>();

            let mut cmds = Vec::new();

            for (i, t) in lines.iter().enumerate() {
                let parts = t.split(':').collect::<Vec<_>>();
                let (mod_name, title) = if parts.len() > 1 {
                    (parts[0].trim(), parts[1..].join(":").trim())
                } else {
                    ("core", t)
                };

                cmds.push(TerminalCommand {
                    agent: "claude-code".into(),
                    cmd: format!("bd create --type task --title \"{}\" --label \"{}\" --priority P2",
                                title.chars().take(50).collect::<String>(), mod_name),
                    output: format!("Created bd-d{i}e{i}  [{}] {}", mod_name, title.chars().take(25).collect::<String>()),
                });
            }

            cmds.push(TerminalCommand {
                agent: "claude-code".into(),
                cmd: "bd ready --json".into(),
                output: format!("[{} task(s) ready for execution]", lines.len()),
            });

            cmds
        },
        _ => Vec::new(),
    }
}

/// Inline terminal component with animated command display
#[component]
pub fn InlineTerminal(commands: Vec<TerminalCommand>) -> Element {
    let visible_count = use_signal(|| 0);
    let is_running = use_memo(|| visible_count() < commands.len() * 2);

    // Future for animation
    use_future(
        || {
            async move {
                if visible_count() >= commands.len() * 2 {
                    return;
                }

                let delay = if visible_count() % 2 == 0 { 300 } else { 150 };
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                visible_count.set(visible_count() + 1);
            }
        },
        (),
        |_, _| {},
    );

    rsx! {
        div {
            class: "mx-2 my-1.5 overflow-hidden rounded-lg border border-border/60 bg-[hsl(0,0%,3%)]",

            // Mini header bar
            div {
                class: "flex items-center gap-2 border-b border-border/40 px-3 py-1.5",

                div {
                    class: "flex gap-1",

                    span {
                        class: "h-2 w-2 rounded-full bg-chart-4/60",
                    }
                    span {
                        class: "h-2 w-2 rounded-full bg-chart-3/60",
                    }
                    span {
                        class: "h-2 w-2 rounded-full bg-chart-2/60",
                    }
                }
                span {
                    class: "font-mono text-[10px] text-muted-foreground/40",
                    "beads-cli"
                }
                if is_running() {
                    span {
                        class: "ml-auto flex items-center gap-1",
                        span {
                            class: "h-1.5 w-1.5 animate-pulse rounded-full bg-primary",
                        }
                        span {
                            class: "font-mono text-[10px] text-chart-2/70",
                            "running"
                        }
                    }
                }
            }

            // Command lines
            div {
                class: "px-3 py-2 font-mono text-xs leading-relaxed",

                for (i, entry) in commands.iter().enumerate() {
                    let cmd_visible = visible_count() > i * 2;
                    let out_visible = visible_count() > i * 2 + 1;

                    if !cmd_visible {
                        continue;
                    }

                    rsx! {
                        Fragment {
                            key: "{i}",

                            div {
                                class: "flex items-start gap-1.5 animate-fade-up",

                                span {
                                    class: format!("mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium {}",
                                                if entry.agent == "claude-code" {
                                                    "bg-chart-3/15 text-chart-3"
                                                } else {
                                                    "bg-primary/15 text-primary"
                                                }),
                                    "{entry.agent}"
                                }
                                span {
                                    class: "text-chart-2",
                                    "$"
                                }
                                span {
                                    class: "text-foreground/90",
                                    "{entry.cmd}"
                                }
                            }
                            if out_visible {
                                div {
                                    class: "animate-fade-up pl-4 text-muted-foreground/50 pb-1",
                                    "{entry.output}"
                                }
                            }
                        }
                    }
                }

                // Blinking cursor at end
                if !is_running() {
                    div {
                        class: "flex items-center gap-1 pt-0.5",
                        span {
                            class: "text-chart-2",
                            "$"
                        }
                        span {
                            class: "inline-block h-3 w-1.5 animate-terminal-blink bg-foreground/60",
                        }
                    }
                }
            }
        }
    }
}

/// Coach bubble component
#[component]
pub fn CoachBubble(children: Element) -> Element {
    rsx! {
        div {
            class: "flex gap-3 animate-fade-up",

            div {
                class: "flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-primary/15 text-xs font-bold text-primary",
                "B"
            }
            div {
                class: "max-w-lg text-sm leading-relaxed text-foreground",
                {children}
            }
        }
    }
}

/// User bubble component
#[component]
pub fn UserBubble(children: Element) -> Element {
    rsx! {
        div {
            class: "flex justify-end animate-fade-up",
            div {
                class: "max-w-lg rounded-lg bg-primary/10 px-4 py-2.5 text-sm leading-relaxed text-foreground",
                {children}
            }
        }
    }
}

/// Main PlanningCoach component
#[component]
pub fn PlanningCoach(
    active_phase: String,
    answers: Vec<CoachAnswer>,
    on_answer: Callback<(String, String)>,
    on_phase_change: Callback<String>,
) -> Element {
    let draft = use_signal(String::new);
    let scroll_ref = use_node::<Element>();
    let input_ref = use_node::<web_sys::HtmlTextAreaElement>();

    let completed_ids = use_memo(|| answers.iter().map(|a| a.step_id.clone()).collect::<Vec<_>>());
    let phase_steps = use_memo(|| get_steps_for_phase_string(&active_phase));
    let current_step = use_memo(|| {
        phase_steps().iter().find(|s| !completed_ids().contains(&s.step_id))
    });

    // Build conversation thread
    let thread = use_memo(|| {
        let mut thread = Vec::new();
        let step_ids = completed_ids();

        for step in phase_steps().iter() {
            thread.push(ThreadEntry::Coach {
                content: step.question,
                step_title: Some(step.title),
            });

            if let Some(answer) = answers.iter().find(|a| a.step_id == step.id) {
                thread.push(ThreadEntry::User {
                    content: answer.value.clone(),
                });

                // Insert terminal block showing the commands that fired
                let cmds = get_commands_for_step(step.id, &answer.value);
                if !cmds.is_empty() {
                    thread.push(ThreadEntry::Terminal { commands: cmds });
                }

                if let Some(follow_up) = step.follow_up {
                    thread.push(ThreadEntry::Coach {
                        content: follow_up,
                        step_title: None,
                    });
                }
            } else {
                break;
            }
        }

        thread
    });

    // Scroll to bottom
    use_effect(move || {
        if let Some(element) = scroll_ref() {
            let element = element.clone();
            spawn(async move {
                element.scroll_to_with_y(element.scroll_height());
            });
        }
    }, [thread.len(), active_phase.clone()]);

    // Focus input when step changes
    use_effect(move || {
        if let Some(element) = input_ref() {
            let element = element.clone();
            spawn(async move {
                element.focus();
            });
        }
    }, [current_step.as_ref().map(|s| s.map(|s| s.id))]);

    let handle_submit = move |()| {
        let current_step = current_step();
        let draft = draft();

        if !draft.trim().is_empty() && current_step.is_some() {
            on_answer.call((current_step.unwrap().id.clone(), draft.trim().to_string()));
            draft.set(String::new());
        }
    };

    let handle_key_down = move |event: KeyboardEvent| {
        if (event.meta_key() || event.ctrl_key()) && event.key() == "Enter" {
            event.prevent_default();
            handle_submit(());
        }
    };

    let phase_complete = use_memo(|| {
        phase_steps().iter().all(|s| !s.required || completed_ids().contains(&s.step_id))
    });

    let phases = ["discover", "define", "develop", "deliver"];
    let next_phase = phases.get(phases.iter().position(|p| p == &active_phase).map(|i| i + 1).unwrap_or(0));

    rsx! {
        div {
            class: "flex h-full flex-col",

            // Scrollable area
            div {
                ref: scroll_ref,
                class: "flex-1 overflow-y-auto px-6 py-6",

                div {
                    class: "mx-auto max-w-xl space-y-4",

                    for (i, entry) in thread().iter().enumerate() {
                        match entry {
                            ThreadEntry::Coach { content, step_title } => rsx! {
                                div {
                                    class: "space-y-1",

                                    if step_title.is_some() {
                                        span {
                                            class: "ml-10 text-[10px] font-medium uppercase tracking-widest text-muted-foreground/50",
                                            "{step_title.as_ref().unwrap()}"
                                        }
                                    }
                                    CoachBubble {
                                        p { "{content}" }
                                    }
                                }
                            },
                            ThreadEntry::User { content } => rsx! {
                                UserBubble {
                                    "{content}"
                                }
                            },
                            ThreadEntry::Terminal { commands } => rsx! {
                                InlineTerminal { commands: commands.clone() }
                            },
                        }
                    }

                    // Hint
                    if current_step().is_some() && !phase_complete() {
                        rsx! {
                            div {
                                class: "ml-10 rounded-md border border-dashed border-border px-3 py-2 text-xs leading-relaxed text-muted-foreground animate-fade-up",
                                "{current_step().unwrap().hint}"
                            }
                        }
                    }

                    // Phase complete
                    if phase_complete() {
                        rsx! {
                            div {
                                class: "space-y-3 pt-2",

                                CoachBubble {
                                    p {
                                        if next_phase.is_some() {
                                            "This phase is locked in. Ready to continue?"
                                        } else {
                                            "Your plan is fully specified. Review the tasks in the sidebar, then hand off to agents."
                                        }
                                    }
                                }

                                if next_phase.is_some() {
                                    rsx! {
                                        div {
                                            class: "ml-10",
                                            button {
                                                onclick: move |_| on_phase_change.call(next_phase.unwrap().to_string()),
                                                class: "rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90",
                                                "Continue to {next_phase.unwrap().charAt(0).toUpperCase() + next_phase.unwrap().slice(1)}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Input area
            if current_step().is_some() && !phase_complete() {
                rsx! {
                    div {
                        class: "shrink-0 border-t border-border px-6 py-4",

                        div {
                            class: "mx-auto max-w-xl",

                            div {
                                class: "overflow-hidden rounded-lg border border-border bg-card transition-colors focus-within:border-primary/50",

                                textarea {
                                    ref: input_ref,
                                    value: draft(),
                                    oninput: move |event| draft.set(event.value()),
                                    onkeydown: handle_key_down,
                                    placeholder: format!("{}...", current_step().unwrap().title),
                                    rows: 3,
                                    class: "w-full resize-none bg-transparent px-4 py-3 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none",
                                }

                                div {
                                    class: "flex items-center justify-between px-4 py-2",

                                    div {
                                        class: "flex items-center gap-2",

                                        if !current_step().unwrap().required {
                                            rsx! {
                                                button {
                                                    onclick: move |_| {
                                                        on_answer.call((current_step().unwrap().id.clone(), "(skipped)".to_string()));
                                                        draft.set(String::new());
                                                    },
                                                    class: "text-xs text-muted-foreground hover:text-foreground",
                                                    "Skip"
                                                }
                                            }
                                        }
                                    }

                                    div {
                                        class: "flex items-center gap-2",

                                        span {
                                            class: "hidden rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground sm:inline",
                                            "Cmd+Enter"
                                        }
                                        button {
                                            onclick: handle_submit,
                                            disabled: draft().trim().is_empty(),
                                            class: "rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground transition-opacity disabled:opacity-30",
                                            "Send"
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
}

/// Generate conversation thread from steps and answers
#[must_use]
pub fn build_thread(
    phase_steps: &[CoachStep],
    answers: &[CoachAnswer],
) -> Vec<ThreadEntry> {
    let mut thread = Vec::new();
    let completed_ids: std::collections::HashSet<String> = answers
        .iter()
        .map(|a| a.step_id.clone())
        .collect();

    for step in phase_steps {
        thread.push(ThreadEntry::Coach {
            content: step.question,
            step_title: Some(step.title),
        });

        if let Some(answer) = answers.iter().find(|a| a.step_id == step.id) {
            thread.push(ThreadEntry::User {
                content: answer.value.clone(),
            });

            // Insert terminal block showing the commands that fired
            let cmds = get_commands_for_step(step.id, &answer.value);
            if !cmds.is_empty() {
                thread.push(ThreadEntry::Terminal { commands: cmds });
            }

            if let Some(follow_up) = step.follow_up {
                thread.push(ThreadEntry::Coach {
                    content: follow_up,
                    step_title: None,
                });
            }
        } else {
            break;
        }
    }

    thread
}

/// Check if a phase is complete based on required steps
#[must_use]
pub fn is_phase_complete(phase_steps: &[CoachStep], answers: &[CoachAnswer]) -> bool {
    let completed_ids: std::collections::HashSet<String> = answers
        .iter()
        .map(|a| a.step_id.clone())
        .collect();

    phase_steps.iter().all(|s| !s.required || completed_ids.contains(&s.step_id))
}

// Thread entry types for the conversation
#[derive(Clone)]
pub enum ThreadEntry {
    Coach { content: String, step_title: Option<&'static str> },
    User { content: String },
    Terminal { commands: Vec<TerminalCommand> },
}

// Helper function to get steps for a phase from string
fn get_steps_for_phase_string(phase: &str) -> Vec<CoachStep> {
    match phase {
        "discover" => get_steps_for_phase(crate::planner::types::DiamondPhase::Top).to_vec(),
        "define" => get_steps_for_phase(crate::planner::types::DiamondPhase::Right).to_vec(),
        "develop" => get_steps_for_phase(crate::planner::types::DiamondPhase::Bottom).to_vec(),
        "deliver" => get_steps_for_phase(crate::planner::types::DiamondPhase::Left).to_vec(),
        _ => Vec::new(),
    }
}