//! V2-Style Planner Page
//!
//! Clean split layout matching v2-typescript-sample:
//! - Top: Phase tabs with progress
//! - Left: Chat-style PlanningCoach
//! - Right: Tabbed panel (Plan/Graph/State)
//!
//! ## Micro-Interactions Implemented
//! - Focus states with ring styling for accessibility
//! - Hover/active states for buttons and cards
//! - Auto-scroll to bottom on new messages
//! - Auto-focus input on step change
//! - Scroll shadow indicators for overflow areas
//! - Keyboard navigation hints (Cmd/Ctrl+Enter)
//! - Tooltip patterns for contextual help

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::manual_map)]

use crate::opencode_client::{ConnectionStatus, OpenCodeClient, TerminalLine, TerminalLineType};
use crate::planner::prompts::{get_steps_for_phase_string, phase_done, total_done, total_required};
use crate::planner::types_coach::{CoachAnswer, CoachStep};
use dioxus::prelude::*;
use std::collections::HashSet;

const PHASES: &[(&str, &str)] = &[
  ("discover", "Discover"),
  ("define", "Define"),
  ("develop", "Develop"),
  ("deliver", "Deliver"),
];

const TABS: &[(&str, &str)] = &[("plan", "Plan"), ("graph", "Graph"), ("state", "State")];

/// V2-style planner page
#[component]
pub fn PlannerV2() -> Element {
  let mut active_phase = use_signal(|| "discover".to_string());
  let mut answers = use_signal(Vec::<CoachAnswer>::new);
  let mut right_tab = use_signal(|| "plan".to_string());
  let mut client = use_signal(OpenCodeClient::default);
  let mut connection_status = use_signal(|| ConnectionStatus::Disconnected);
  let mut terminal_lines = use_signal(Vec::<TerminalLine>::new);
  let mut executed_commands = use_signal(HashSet::<String>::new);

  // Check connection on mount
  use_effect({
    let mut connection_status = connection_status;
    move || {
      spawn(async move {
        // For now, just set to demo mode since check_health needs async
        connection_status.set(ConnectionStatus::Disconnected);
      });
    }
  });

  let handle_answer = move |(step_id, value): (String, String)| {
    let mut current = answers.write().clone();
    current.retain(|a| a.step_id != step_id);
    current.push(CoachAnswer {
      step_id: step_id.clone(),
      value: value.clone(),
    });
    answers.set(current);

    // Generate terminal commands for this answer
    let cmds = get_commands_for_step(&step_id, &value);
    let mut lines = terminal_lines.write().clone();
    for (agent, cmd, output) in cmds {
      lines.push(TerminalLine::cmd(cmd).with_agent(agent));
      lines.push(TerminalLine::output(output));
    }
    terminal_lines.set(lines);
  };

  let total_req = total_required();
  let total_complete = total_done(&answers.read());

  rsx! {
    div { class: "flex h-screen flex-col overflow-hidden bg-[hsl(0,0%,4%)] text-white font-sans",
      // Top header
      header { class: "flex shrink-0 items-center justify-between border-b border-white/10 px-5 py-2",
        div { class: "flex items-center gap-6",
          // Logo
          div { class: "flex items-center gap-2",
            div { class: "flex h-6 w-6 items-center justify-center rounded-md bg-blue-500",
              svg {
                width: "14",
                height: "14",
                view_box: "0 0 14 14",
                fill: "none",
                class: "text-white",
                circle { cx: "4", cy: "4", r: "2", fill: "currentColor" }
                circle { cx: "10", cy: "4", r: "2", fill: "currentColor" }
                circle { cx: "7", cy: "10", r: "2", fill: "currentColor" }
                path {
                  d: "M4 4L10 4M4 4L7 10M10 4L7 10",
                  stroke: "currentColor",
                  stroke_width: "1",
                  opacity: "0.5",
                }
              }
            }
            span { class: "text-sm font-bold tracking-tight", "Beads Planner" }
          }

          // Phase tabs
          nav { class: "flex items-center",
            for (phase_key, label) in PHASES.iter() {
              PhaseTab {
                key: "{phase_key}",
                phase_key: *phase_key,
                label: *label,
                active: *active_phase.read() == *phase_key,
                done: phase_done(*phase_key, &answers.read()),
                on_click: {
                  let mut active_phase = active_phase;
                  let phase_key = phase_key.to_string();
                  move |_| active_phase.set(phase_key.clone())
                },
              }
            }
          }
        }

        // Progress counter
        span { class: "font-mono text-xs text-white/50", "{total_complete}/{total_req}" }
      }

      // Main content
      div { class: "flex flex-1 overflow-hidden",
        // Left: Planning Coach
        main { class: "flex-1 overflow-hidden border-r border-white/10",
          PlanningCoach {
            active_phase: active_phase.read().clone(),
            answers: answers.read().clone(),
            on_answer: handle_answer,
            on_phase_change: {
              let mut active_phase = active_phase;
              move |phase| active_phase.set(phase)
            },
          }
        }

        // Right: Tabbed panel
        div { class: "flex w-[440px] shrink-0 flex-col",
          // Tabs
          div { class: "flex shrink-0 items-center border-b border-white/10",
            for (tab_key, label) in TABS.iter() {
              TabButton {
                key: "{tab_key}",
                tab_key: *tab_key,
                label: *label,
                active: *right_tab.read() == *tab_key,
                on_click: {
                  let mut right_tab = right_tab;
                  let tab_key = tab_key.to_string();
                  move |_| right_tab.set(tab_key.clone())
                },
              }
            }
          }

          // Panel content
          div { class: "flex-1 overflow-hidden",
            if *right_tab.read() == "plan" {
              ArtifactPanel {
                answers: answers.read().clone(),
                active_phase: active_phase.read().clone(),
                terminal_lines: terminal_lines.read().clone(),
                connection_status: *connection_status.read(),
              }
            } else if *right_tab.read() == "graph" {
              GraphPanel {
                answers: answers.read().clone(),
              }
            } else {
              StatePanel {
                answers: answers.read().clone(),
                active_phase: active_phase.read().clone(),
              }
            }
          }
        }
      }
    }
  }
}

/// Get phase index as string (1-indexed), or empty if not found
fn phase_index_str(phase_key: &str) -> String {
  PHASES
    .iter()
    .position(|(k, _)| *k == phase_key)
    .map_or_else(String::new, |i| (i + 1).to_string())
}

/// Phase tab button with focus-visible ring styling
#[component]
fn PhaseTab(
  phase_key: String,
  label: &'static str,
  active: bool,
  done: bool,
  on_click: EventHandler<()>,
) -> Element {
  let index_str = phase_index_str(&phase_key);
  // Focus-visible styling for keyboard navigation accessibility
  let focus_class = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[hsl(0,0%,4%)]";
  // Active state for press feedback
  let active_press_class = if active { "" } else { "active:scale-[0.98]" };
  rsx! {
    button {
      class: if active {
        format!("relative flex items-center gap-1.5 px-3 py-2 text-sm text-white rounded-sm {focus_class}")
      } else {
        format!("relative flex items-center gap-1.5 px-3 py-2 text-sm text-white/60 hover:text-white/80 transition-all duration-150 rounded-sm {focus_class} {active_press_class}")
      },
      onclick: move |_| on_click.call(()),
      if done {
        span { class: "text-green-400", "✓" }
      } else {
        span { class: if active { "flex h-4 w-4 items-center justify-center rounded-full bg-blue-500/20 text-blue-400 text-xs" }
                     else { "flex h-4 w-4 items-center justify-center rounded-full bg-white/10 text-white/50 text-xs" },
          "{index_str}"
        }
      }
      span { class: if active { "font-medium" } else { "" }, "{label}" }
      if active {
        span { class: "absolute inset-x-0 -bottom-[9px] h-0.5 bg-blue-500" }
      }
    }
  }
}

/// Tab button with focus-visible ring styling
#[component]
fn TabButton(
  tab_key: &'static str,
  label: &'static str,
  active: bool,
  on_click: EventHandler<()>,
) -> Element {
  // Focus-visible styling for keyboard navigation accessibility
  let focus_class = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[hsl(0,0%,4%)]";
  rsx! {
    button {
      class: if active {
        format!("relative flex items-center gap-1.5 px-4 py-2.5 text-xs font-medium text-white rounded-sm {focus_class}")
      } else {
        format!("relative flex items-center gap-1.5 px-4 py-2.5 text-xs font-medium text-white/60 hover:text-white/80 active:scale-[0.98] transition-all duration-150 rounded-sm {focus_class}")
      },
      onclick: move |_| on_click.call(()),
      // Tab icon based on tab_key
      if tab_key == "plan" {
        // Plan tab icon: document with lines
        svg {
          width: "12",
          height: "12",
          view_box: "0 0 16 16",
          fill: "none",
          class: "shrink-0",
          rect {
            x: "2",
            y: "2",
            width: "12",
            height: "12",
            rx: "2",
            stroke: "currentColor",
            stroke_width: "1.2",
          }
          path {
            d: "M5 6H11M5 8.5H9M5 11H7",
            stroke: "currentColor",
            stroke_width: "1",
            stroke_linecap: "round",
            opacity: "0.6",
          }
        }
      } else if tab_key == "graph" {
        // Graph tab icon: connected nodes
        svg {
          width: "12",
          height: "12",
          view_box: "0 0 16 16",
          fill: "none",
          class: "shrink-0",
          circle {
            cx: "4",
            cy: "4",
            r: "2",
            stroke: "currentColor",
            stroke_width: "1.2",
          }
          circle {
            cx: "12",
            cy: "4",
            r: "2",
            stroke: "currentColor",
            stroke_width: "1.2",
          }
          circle {
            cx: "8",
            cy: "12",
            r: "2",
            stroke: "currentColor",
            stroke_width: "1.2",
          }
          path {
            d: "M5.5 5.5L7 10.5M10.5 5.5L9 10.5",
            stroke: "currentColor",
            stroke_width: "1",
            opacity: "0.5",
          }
        }
      } else if tab_key == "state" {
        // State tab icon: connected boxes
        svg {
          width: "12",
          height: "12",
          view_box: "0 0 16 16",
          fill: "none",
          class: "shrink-0",
          rect {
            x: "2",
            y: "2",
            width: "5",
            height: "5",
            rx: "1",
            stroke: "currentColor",
            stroke_width: "1.2",
          }
          rect {
            x: "9",
            y: "9",
            width: "5",
            height: "5",
            rx: "1",
            stroke: "currentColor",
            stroke_width: "1.2",
          }
          path {
            d: "M7 4.5H9.5V9.5H11.5",
            stroke: "currentColor",
            stroke_width: "1",
            stroke_linecap: "round",
          }
        }
      }
      "{label}"
      if active {
        span { class: "absolute inset-x-0 -bottom-px h-0.5 bg-blue-500" }
      }
    }
  }
}

/// Thread entry types for the conversation
#[derive(Clone, Debug)]
enum ThreadEntry {
  Coach {
    content: String,
    step_title: Option<String>,
  },
  User {
    content: String,
  },
  Terminal {
    commands: Vec<(String, String, String)>,
  },
}

/// Build the conversation thread from steps and answers
fn build_thread(steps: &[CoachStep], answers: &[CoachAnswer]) -> Vec<ThreadEntry> {
  let mut thread = Vec::new();

  for step in steps {
    let answer_opt = answers.iter().find(|a| a.step_id == step.id);

    // Always add the coach question
    thread.push(ThreadEntry::Coach {
      content: step.question.clone(),
      step_title: Some(step.title.clone()),
    });

    if let Some(answer) = answer_opt {
      // Add user answer
      thread.push(ThreadEntry::User {
        content: answer.value.clone(),
      });

      // Add inline terminal showing commands
      let cmds = get_commands_for_step(&step.id, &answer.value);
      if !cmds.is_empty() {
        thread.push(ThreadEntry::Terminal { commands: cmds });
      }

      // Add follow-up if present
      if let Some(ref follow_up) = step.follow_up {
        thread.push(ThreadEntry::Coach {
          content: follow_up.clone(),
          step_title: None,
        });
      }
    } else {
      // Stop at first unanswered step
      break;
    }
  }

  thread
}

/// Planning coach (chat-style) with auto-scroll and focus management
#[component]
fn PlanningCoach(
  active_phase: String,
  answers: Vec<CoachAnswer>,
  on_answer: EventHandler<(String, String)>,
  on_phase_change: EventHandler<String>,
) -> Element {
  let draft = use_signal(String::new);
  // Track scroll position for scroll shadow indicator
  let is_scrolled_to_bottom = use_signal(|| true);
  let show_top_shadow = use_signal(|| false);

  let steps = get_steps_for_phase_string(&active_phase);
  let completed_ids: HashSet<&str> = answers.iter().map(|a| a.step_id.as_str()).collect();
  let current_step = steps
    .iter()
    .find(|s| !completed_ids.contains(s.id.as_str()))
    .cloned();

  // Build the conversation thread
  let thread = build_thread(&steps, &answers);
  let thread_len = thread.len();

  // Auto-scroll effect when thread changes
  use_effect({
    let mut is_scrolled_to_bottom = is_scrolled_to_bottom;
    move || {
      // When thread length changes, reset to scrolled to bottom
      let _ = thread_len;
      is_scrolled_to_bottom.set(true);
    }
  });

  // Check phase completion based on required steps
  let phase_complete = steps.iter().all(|s| completed_ids.contains(s.id.as_str()));
  let phases: Vec<_> = PHASES.iter().map(|(k, _)| *k).collect();
  let next_phase = phases
    .iter()
    .skip_while(|p| **p != active_phase.as_str())
    .nth(1)
    .map(|s| s.to_string());

  // Pre-compute strings for rsx!
  let completion_message = if next_phase.is_some() {
    "This phase is locked in. Ready to continue?".to_string()
  } else {
    "Your plan is fully specified. Review the tasks in the sidebar, then hand off to agents."
      .to_string()
  };

  // Check if draft is empty (for button state)
  let draft_empty = draft.read().trim().is_empty();

  // Get step id for handlers - use empty string as fallback for signal initialization
  let step_id_for_handlers = current_step
    .as_ref()
    .map_or_else(String::new, |s| s.id.clone());

  // Focus-visible styling shared across buttons
  let focus_visible_class = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[hsl(0,0%,4%)]";

  rsx! {
    div { class: "flex h-full flex-col relative",
      // Top scroll shadow indicator (shows when scrolled down)
      if *show_top_shadow.read() {
        div { class: "absolute top-0 left-0 right-0 h-6 bg-gradient-to-b from-[hsl(0,0%,4%)] to-transparent pointer-events-none z-10" }
      }

      // Conversation thread with scroll shadow
      div {
        class: "flex-1 overflow-y-auto px-6 py-6 scroll-smooth",
        onscroll: {
          let mut show_top_shadow = show_top_shadow;
          move |e| {
            // Show top shadow when scrolled more than 20px
            show_top_shadow.set(e.scroll_top() > 20.0);
          }
        },
        div { class: "mx-auto max-w-xl space-y-4",
          // Render thread entries
          for (i, entry) in thread.iter().enumerate() {
            match entry {
              ThreadEntry::Coach { content, step_title } => rsx! {
                CoachBubble {
                  key: "coach-{i}",
                  step_title: step_title.clone(),
                  content: content.clone(),
                }
              },
              ThreadEntry::User { content } => rsx! {
                UserBubble {
                  key: "user-{i}",
                  content: content.clone(),
                }
              },
              ThreadEntry::Terminal { commands } => rsx! {
                InlineTerminal {
                  key: "terminal-{i}",
                  commands: commands.clone(),
                }
              },
            }
          }

          // Current step hint (if not phase complete)
          if let Some(ref step) = current_step {
            if !phase_complete {
              // Show hint box with the step's hint as contextual guidance
              {
                let hint_text = step.hint.as_ref().map_or_else(String::new, |s| s.clone());
                rsx! {
                  // Tooltip-style hint with hover effect
                  div {
                    class: "ml-10 group relative rounded-md border border-dashed border-white/20 px-3 py-2 text-xs leading-relaxed text-white/50 animate-fade-up transition-colors hover:border-white/30 hover:text-white/60",
                    // Info icon indicator
                    span { class: "absolute -left-1 -top-1 flex h-4 w-4 items-center justify-center rounded-full bg-white/10 text-[8px] text-white/40 group-hover:bg-white/20 group-hover:text-white/60 transition-colors",
                      "?"
                    }
                    "{hint_text}"
                  }
                }
              }
            }
          }

          // Phase complete message
          if phase_complete {
            CoachBubble {
              step_title: None,
              content: completion_message,
            }
            if let Some(next) = &next_phase {
              {
                let label = capitalize_first(next);
                rsx! {
                  div { class: "ml-10",
                    button {
                      class: format!("rounded-md bg-blue-500 px-4 py-2 text-sm font-medium text-white hover:bg-blue-600 active:bg-blue-700 active:scale-[0.98] transition-all duration-150 {focus_visible_class}"),
                      onclick: {
                        let on_phase_change = on_phase_change;
                        let next = next.clone();
                        move |_| on_phase_change.call(next.clone())
                      },
                      "Continue to {label}"
                    }
                  }
                }
              }
            }
          }
        }
      }

      // Input area
      if current_step.is_some() && !phase_complete {
        {
          let placeholder_text = current_step.as_ref().map_or("Type your answer...", |s| s.title.as_str());
          rsx! {
            div { class: "shrink-0 border-t border-white/10 px-6 py-4 bg-[hsl(0,0%,4%)]",
              div { class: "mx-auto max-w-xl",
                // Enhanced focus-within styling with ring effect
                div { class: "overflow-hidden rounded-lg border border-white/20 bg-white/5 transition-all duration-200 focus-within:border-blue-500/50 focus-within:ring-2 focus-within:ring-blue-500/20",
                  textarea {
                    // Auto-focus when step changes (via autofocus attribute)
                    class: "w-full resize-none bg-transparent px-4 py-3 text-sm text-white placeholder:text-white/30 focus:outline-none",
                    placeholder: placeholder_text,
                    rows: 3,
                    value: "{draft}",
                    autofocus: true,
                    oninput: {
                      let mut draft = draft;
                      move |e| draft.set(e.value().to_string())
                    },
                    onkeydown: {
                      let on_answer = on_answer;
                      let step_id = step_id_for_handlers.clone();
                      let mut draft = draft;
                      move |e| {
                        // Cmd+Enter (Mac) or Ctrl+Enter (Windows/Linux) to submit
                        if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) {
                          e.prevent_default();
                          let value = draft.read().trim().to_string();
                          if !value.is_empty() {
                            on_answer.call((step_id.clone(), value));
                            draft.set(String::new());
                          }
                        }
                      }
                    },
                  }
                  div { class: "flex items-center justify-between px-4 py-2",
                    // Skip button with hover/focus states
                    button {
                      class: format!("text-xs text-white/50 hover:text-white/80 active:text-white transition-colors rounded-sm px-2 py-1 -ml-2 {focus_visible_class}"),
                      onclick: {
                        let on_answer = on_answer;
                        let step_id = step_id_for_handlers.clone();
                        let mut draft = draft;
                        move |_| {
                          on_answer.call((step_id.clone(), "(skipped)".to_string()));
                          draft.set(String::new());
                        }
                      },
                      "Skip"
                    }
                    // Right side: keyboard hint and send button
                    div { class: "flex items-center gap-2",
                      // Keyboard shortcut hint with tooltip styling
                      kbd { class: "hidden rounded bg-white/10 px-1.5 py-0.5 font-mono text-[10px] text-white/50 border border-white/10 sm:inline",
                        "\u{2318}\u{21B5}"
                      }
                      // Send button with enhanced states
                      button {
                        class: format!("rounded-md bg-blue-500 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-600 active:bg-blue-700 active:scale-[0.98] transition-all duration-150 disabled:opacity-30 disabled:pointer-events-none {focus_visible_class}"),
                        disabled: draft_empty,
                        onclick: {
                          let on_answer = on_answer;
                          let step_id = step_id_for_handlers.clone();
                          let mut draft = draft;
                          move |_| {
                            let value = draft.read().trim().to_string();
                            if !value.is_empty() {
                              on_answer.call((step_id.clone(), value));
                              draft.set(String::new());
                            }
                          }
                        },
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
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
  s.chars().next().map_or_else(String::new, |c| {
    let upper = c.to_uppercase().collect::<String>();
    format!("{}{}", upper, s.chars().skip(1).collect::<String>())
  })
}

/// Coach bubble
#[component]
fn CoachBubble(step_title: Option<String>, content: String) -> Element {
  rsx! {
    div { class: "flex gap-3 animate-fade-up",
      div { class: "flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-blue-500/20 text-xs font-bold text-blue-400",
        "B"
      }
      div { class: "max-w-lg",
        if let Some(title) = step_title {
          span { class: "ml-0 mb-1 block text-[10px] font-medium uppercase tracking-widest text-white/40",
            "{title}"
          }
        }
        p { class: "text-sm leading-relaxed text-white/90", "{content}" }
      }
    }
  }
}

/// User bubble
#[component]
fn UserBubble(content: String) -> Element {
  rsx! {
    div { class: "flex justify-end animate-fade-up",
      div { class: "max-w-lg rounded-lg bg-blue-500/10 px-4 py-2.5 text-sm leading-relaxed text-white",
        "{content}"
      }
    }
  }
}

// ============================================================================
// V2-Enhanced Inline Terminal Component
// ============================================================================

/// Connection status configuration for StatusIndicator
#[derive(Clone, Debug, PartialEq)]
struct StatusConfig {
  color: &'static str,
  text: &'static str,
}

/// Get status configuration based on connection status and demo mode
fn get_status_config(status: ConnectionStatus, is_demo_mode: bool) -> StatusConfig {
  if is_demo_mode {
    return StatusConfig {
      color: "bg-yellow-500/70",
      text: "Demo Mode",
    };
  }
  match status {
    ConnectionStatus::Connected => StatusConfig {
      color: "bg-chart-2",
      text: "Connected",
    },
    ConnectionStatus::Connecting => StatusConfig {
      color: "bg-yellow-500 animate-pulse",
      text: "Connecting...",
    },
    ConnectionStatus::Disconnected => StatusConfig {
      color: "bg-muted-foreground/50",
      text: "Disconnected",
    },
    ConnectionStatus::Error => StatusConfig {
      color: "bg-red-500",
      text: "Error",
    },
  }
}

/// Status indicator component showing connection status with colored dot
#[component]
fn StatusIndicator(status: ConnectionStatus, is_demo_mode: bool) -> Element {
  let config = get_status_config(status, is_demo_mode);
  rsx! {
    div { class: "flex items-center gap-1.5 px-2 py-1",
      span { class: "h-2 w-2 rounded-full {config.color}" }
      span { class: "text-[10px] font-medium text-white/50", "{config.text}" }
    }
  }
}

/// Terminal line type for the enhanced terminal
#[derive(Clone, Debug, PartialEq)]
enum V2TerminalLineType {
  Cmd,
  Output,
  Comment,
  Separator,
  Error,
}

/// A line in the enhanced terminal feed
#[derive(Clone, Debug)]
struct V2TerminalLine {
  line_type: V2TerminalLineType,
  text: String,
  agent: Option<String>,
  executed: bool,
}

impl V2TerminalLine {
  fn cmd(text: String) -> Self {
    Self {
      line_type: V2TerminalLineType::Cmd,
      text,
      agent: None,
      executed: false,
    }
  }

  fn output(text: String) -> Self {
    Self {
      line_type: V2TerminalLineType::Output,
      text,
      agent: None,
      executed: false,
    }
  }

  fn comment(text: String) -> Self {
    Self {
      line_type: V2TerminalLineType::Comment,
      text,
      agent: None,
      executed: false,
    }
  }

  fn separator() -> Self {
    Self {
      line_type: V2TerminalLineType::Separator,
      text: String::new(),
      agent: None,
      executed: false,
    }
  }

  fn error(text: String) -> Self {
    Self {
      line_type: V2TerminalLineType::Error,
      text,
      agent: None,
      executed: false,
    }
  }

  fn with_agent(self, agent: String) -> Self {
    Self {
      agent: Some(agent),
      ..self
    }
  }

  fn executed(self) -> Self {
    Self {
      executed: true,
      ..self
    }
  }
}

/// Convert the tuple format to V2TerminalLine format using functional iterator chain
fn commands_to_lines(commands: &[(String, String, String)]) -> Vec<V2TerminalLine> {
  commands
    .iter()
    .flat_map(|(agent, cmd, output)| {
      [
        V2TerminalLine::cmd(cmd.clone()).with_agent(agent.clone()),
        V2TerminalLine::output(output.clone()),
      ]
    })
    .collect()
}

/// Inline terminal block with animated staggered reveal (V2 Enhanced)
///
/// Features:
/// - StatusIndicator showing connection status
/// - TerminalLine types: cmd, output, comment, separator, error
/// - Live streaming simulation with staggered timing
/// - Auto-scroll behavior
/// - Better line styling with agent labels
#[component]
fn InlineTerminal(commands: Vec<(String, String, String)>) -> Element {
  // Convert to V2 line format
  let lines = commands_to_lines(&commands);

  // Track visible count for staggered animation
  let visible_count = use_signal(|| 0usize);
  let total_items = lines.len();

  // Track if streaming is in progress
  let is_streaming = use_signal(|| true);

  // Animation effect: increment visible count over time (staggered reveal)
  use_effect({
    let mut visible_count = visible_count;
    let mut is_streaming = is_streaming;
    let total_items = total_items;
    move || {
      let current = *visible_count.read();
      if current < total_items {
        // Staggered timing: 30ms per line (matching TypeScript v2)
        let delay = 30u64;
        spawn(async move {
          tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
          let next = visible_count.read().saturating_add(1);
          if next <= total_items {
            visible_count.set(next);
          }
          if next >= total_items {
            is_streaming.set(false);
          }
        });
      }
    }
  });

  let is_running = *is_streaming.read() || *visible_count.read() < total_items;
  let current_visible = *visible_count.read();

  rsx! {
    div { class: "mx-2 my-1.5 flex flex-col overflow-hidden rounded-lg border border-white/10 bg-[hsl(0,0%,3%)]",
      // Header bar with status indicator
      div { class: "flex shrink-0 items-center justify-between border-b border-white/5 px-3 py-1.5",
        div { class: "flex items-center gap-2",
          // Traffic lights (macOS style)
          div { class: "flex gap-1",
            span { class: "h-2 w-2 rounded-full bg-red-500/60" }
            span { class: "h-2 w-2 rounded-full bg-yellow-500/60" }
            span { class: "h-2 w-2 rounded-full bg-green-500/60" }
          }
          span { class: "font-mono text-[10px] text-white/30", "beads-cli" }
        }
        // Status indicator (Demo Mode for preview)
        StatusIndicator {
          status: ConnectionStatus::Connected,
          is_demo_mode: true,
        }
        // Running indicator
        if is_running {
          span { class: "flex items-center gap-1",
            span { class: "h-1.5 w-1.5 animate-pulse rounded-full bg-green-400" }
            span { class: "font-mono text-[10px] text-green-400/70", "running" }
          }
        }
      }

      // Terminal content with auto-scroll
      div {
        class: "flex-1 overflow-y-auto bg-[hsl(0,0%,3%)] p-3 font-mono text-xs leading-relaxed scroll-smooth",

        // Render lines based on type
        for (i, line) in lines.iter().enumerate() {
          {
            let line_visible = i < current_visible;
            let animation_delay = format!("animation-delay: {}ms", i * 30);

            // Only render if visible
            if line_visible {
              match line.line_type {
                V2TerminalLineType::Separator => rsx! {
                  div {
                    key: "sep-{i}",
                    class: "h-2",
                  }
                },
                V2TerminalLineType::Comment => rsx! {
                  div {
                    key: "comment-{i}",
                    class: "animate-fade-up text-white/30 italic",
                    style: "{animation_delay}",
                    "{line.text}"
                  }
                },
                V2TerminalLineType::Error => rsx! {
                  div {
                    key: "error-{i}",
                    class: "animate-fade-up text-red-500",
                    style: "{animation_delay}",
                    "{line.text}"
                  }
                },
                V2TerminalLineType::Cmd => rsx! {
                  div {
                    key: "cmd-{i}",
                    class: "animate-fade-up flex items-start gap-1.5",
                    style: "{animation_delay}",
                    // Agent label
                    if let Some(ref agent) = line.agent {
                      span {
                        class: if agent == "claude-code" {
                          "mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium bg-purple-500/15 text-purple-400"
                        } else {
                          "mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium bg-blue-500/15 text-blue-400"
                        },
                        "{agent}"
                      }
                    }
                    // Command with green $ prefix
                    span { class: "text-chart-2", "$" }
                    span { class: "text-white/90", "{line.text}" }
                  }
                },
                V2TerminalLineType::Output => rsx! {
                  div {
                    key: "output-{i}",
                    class: "animate-fade-up pl-4 text-white/40",
                    style: "{animation_delay}",
                    "{line.text}"
                  }
                },
              }
            } else {
              rsx! { div { key: "hidden-{i}" } }
            }
          }
        }

        // Blinking cursor at end
        div { class: "mt-1 flex items-center gap-1",
          span { class: "text-chart-2", "$" }
          span {
            class: if is_running {
              "inline-block h-3.5 w-1.5 bg-white/70"
            } else {
              "inline-block h-3.5 w-1.5 bg-white/70 animate-terminal-blink"
            }
          }
        }
      }
    }
  }
}

/// Get value for a step ID, returning None if skipped or not found
fn get_val(answers: &[CoachAnswer], id: &str) -> Option<String> {
  answers.iter().find(|a| a.step_id == id).and_then(|a| {
    if a.value == "(skipped)" {
      None
    } else {
      Some(a.value.clone())
    }
  })
}

/// Parse text into non-empty trimmed lines
fn parse_lines(text: Option<&str>) -> Vec<String> {
  text.map_or(Vec::new(), |t| {
    t.lines()
      .map(|l| l.trim().to_string())
      .filter(|l| !l.is_empty())
      .collect()
  })
}

/// Section header with label and optional count badge
#[component]
fn SectionHeader(label: String, count: Option<usize>) -> Element {
  rsx! {
    div { class: "flex items-center gap-2 pb-2 pt-5 first:pt-0",
      h4 { class: "text-xs font-semibold uppercase tracking-widest text-white/50",
        "{label}"
      }
      if let Some(c) = count {
        span { class: "rounded-full bg-white/10 px-1.5 py-0.5 text-xs tabular-nums text-white/50",
          "{c}"
        }
      }
    }
  }
}

/// Thesis card for Problem/Antithesis/Solution
#[component]
fn ThesisCard(label: String, value: String, accent: Option<String>) -> Element {
  if value.is_empty() {
    return rsx! {};
  }
  let accent_classes = accent.map_or_else(|| "border-white/10 bg-white/5".to_string(), |a| a);
  rsx! {
    div { class: "animate-fade-up rounded-lg border px-3 py-2.5 {accent_classes}",
      span { class: "mb-1 block text-xs font-medium uppercase tracking-wider text-white/50",
        "{label}"
      }
      p { class: "text-sm leading-relaxed text-white/90", "{value}" }
    }
  }
}

/// Parse use case text into "actor can action so that benefit" parts
/// Format: "X can Y so that Z" (case-insensitive)
fn parse_use_case(text: &str) -> Option<(String, String, String)> {
  let lower = text.to_lowercase();
  let can_marker = " can ";
  let so_that_marker = " so that ";

  let can_pos = lower.find(can_marker)?;
  let so_that_pos = lower.find(so_that_marker).filter(|&pos| pos > can_pos)?;

  let actor = text[..can_pos].trim().to_string();
  let action_start = can_pos + can_marker.len();
  let action = text[action_start..so_that_pos].trim().to_string();
  let benefit_start = so_that_pos + so_that_marker.len();
  let benefit = text[benefit_start..].trim().to_string();

  if actor.is_empty() || action.is_empty() || benefit.is_empty() {
    None
  } else {
    Some((actor, action, benefit))
  }
}

/// Use case row with parsed "can/so that" format highlighting
/// Enhanced with hover effects
#[component]
fn UseCaseRow(text: String, index: usize) -> Element {
  let parsed = parse_use_case(&text);
  rsx! {
    div { class: "animate-fade-up flex items-start gap-2.5 rounded-md px-2 py-2 transition-all duration-150 hover:bg-white/5 group",
      span { class: "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded bg-white/10 font-mono text-xs text-white/50 group-hover:bg-white/15 transition-colors",
        "{index}"
      }
      if let Some((actor, action, benefit)) = parsed {
        p { class: "min-w-0 text-sm leading-relaxed",
          span { class: "font-medium text-blue-400 group-hover:text-blue-300 transition-colors", "{actor}" }
          span { class: "text-white/50", " can " }
          span { class: "text-white/90", "{action}" }
          span { class: "text-white/50", " so that " }
          span { class: "text-white/70", "{benefit}" }
        }
      } else {
        p { class: "min-w-0 text-sm text-white/90 group-hover:text-white transition-colors", "{text}" }
      }
    }
  }
}

/// Parse task text into (module, title) parts
fn parse_task(text: &str) -> (Option<String>, String) {
  let parts: Vec<&str> = text.splitn(2, ':').collect();
  if parts.len() > 1 {
    (
      Some(parts[0].trim().to_string()),
      parts[1].trim().to_string(),
    )
  } else {
    (None, text.to_string())
  }
}

/// Task row with module label, selection state, and right arrow
/// Enhanced with focus-visible states and active press feedback
#[component]
fn TaskRow(text: String, index: usize, selected: bool, on_select: EventHandler<()>) -> Element {
  let (module, title) = parse_task(&text);
  let bg_class = if selected {
    "bg-blue-500/10 ring-1 ring-blue-500/30".to_string()
  } else {
    "hover:bg-white/5 active:bg-white/10".to_string()
  };
  let badge_class = if selected {
    "bg-blue-500 text-white".to_string()
  } else {
    "bg-white/10 text-white/50 group-hover:bg-white/15".to_string()
  };
  let focus_class = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[hsl(0,0%,4%)]";
  rsx! {
    button {
      class: format!("animate-fade-up flex w-full items-start gap-2.5 rounded-md px-2 py-2 text-left transition-all duration-150 {bg_class} group {focus_class}"),
      onclick: move |_| on_select.call(()),
      span { class: "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded font-mono text-xs transition-colors {badge_class}",
        "{index}"
      }
      div { class: "min-w-0 flex-1",
        div { class: "flex items-center gap-1.5",
          if let Some(ref mod_name) = module {
            span { class: "rounded bg-orange-500/15 px-1.5 py-0.5 font-mono text-xs text-orange-400",
              "{mod_name}"
            }
          }
        }
        p { class: "mt-0.5 text-sm text-white/90 group-hover:text-white transition-colors", "{title}" }
      }
      // Right arrow SVG with hover effect
      svg {
        width: "14",
        height: "14",
        view_box: "0 0 14 14",
        fill: "none",
        class: "mt-1 shrink-0 text-white/40 group-hover:text-white/60 transition-colors",
        path {
          d: "M5 3L9 7L5 11",
          stroke: "currentColor",
          stroke_width: "1.5",
          stroke_linecap: "round",
          stroke_linejoin: "round",
        }
      }
    }
  }
}

/// Expandable task detail panel with Acceptance Criteria and Edge Cases
/// Enhanced with focus-visible states for close button
#[component]
fn TaskDetail(task: String, index: usize, on_close: EventHandler<()>) -> Element {
  let (module, title) = parse_task(&task);
  let focus_class = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[hsl(0,0%,4%)]";
  rsx! {
    div { class: "animate-fade-up rounded-lg border border-blue-500/20 bg-blue-500/5",
      // Header
      div { class: "flex items-center justify-between border-b border-blue-500/10 px-3 py-2",
        div { class: "flex items-center gap-2",
          span { class: "flex h-5 w-5 items-center justify-center rounded bg-blue-500 font-mono text-xs text-white",
            "{index}"
          }
          if let Some(ref mod_name) = module {
            span { class: "rounded bg-orange-500/15 px-1.5 py-0.5 font-mono text-xs text-orange-400",
              "{mod_name}"
            }
          }
          span { class: "text-sm font-medium text-white/90", "{title}" }
        }
        button {
          class: format!("rounded p-0.5 text-white/50 hover:text-white hover:bg-white/10 active:bg-white/20 transition-all duration-150 {focus_class}"),
          onclick: move |_| on_close.call(()),
          // X icon SVG
          svg {
            width: "14",
            height: "14",
            view_box: "0 0 14 14",
            fill: "none",
            path {
              d: "M4 4L10 10M10 4L4 10",
              stroke: "currentColor",
              stroke_width: "1.5",
              stroke_linecap: "round",
            }
          }
        }
      }
      // Content
      div { class: "space-y-3 px-3 py-3",
        // Acceptance Criteria
        div {
          span { class: "mb-1 block text-xs font-medium uppercase tracking-wider text-white/50",
            "Acceptance Criteria"
          }
          div { class: "rounded border border-dashed border-white/10 px-3 py-3 text-center text-xs text-white/30",
            "Add criteria in the Develop phase"
          }
        }
        // Edge Cases
        div {
          span { class: "mb-1 block text-xs font-medium uppercase tracking-wider text-white/50",
            "Edge Cases"
          }
          div { class: "rounded border border-dashed border-white/10 px-3 py-3 text-center text-xs text-white/30",
            "The coach will prompt you for these"
          }
        }
      }
    }
  }
}

/// Artifact panel (Plan tab) - displays thesis, use cases, and tasks
/// Enhanced with scroll shadow indicators
#[component]
fn ArtifactPanel(
  answers: Vec<CoachAnswer>,
  active_phase: String,
  terminal_lines: Vec<TerminalLine>,
  connection_status: ConnectionStatus,
) -> Element {
  let selected_task = use_signal(|| None::<usize>);
  // Track scroll position for scroll shadow indicator
  let show_top_shadow = use_signal(|| false);

  // Extract values from answers
  let problem = get_val(&answers, "problem");
  let antithesis = get_val(&answers, "antithesis");
  let solution = get_val(&answers, "solution");
  let persona = get_val(&answers, "persona");
  let scenario = get_val(&answers, "scenario");
  let use_cases = parse_lines(get_val(&answers, "use-cases").as_deref());
  let constraints = get_val(&answers, "constraints");
  let tasks = parse_lines(get_val(&answers, "tasks").as_deref());

  // Calculate progress
  let total_req = total_required();
  let total_complete = total_done(&answers);
  let progress = if total_req > 0 {
    ((total_complete * 100) / total_req).min(100)
  } else {
    0
  };
  let progress_width = format!("width: {progress}%");

  // Check if we have any content to display
  let has_anything = !answers.is_empty();

  // Empty state message based on phase
  let empty_message = if active_phase == "discover" {
    "Answer the coach to build your thesis, persona, and north star scenario."
  } else {
    "Your plan will build up here as you answer."
  };

  rsx! {
    div { class: "flex h-full flex-col relative",
      // Progress bar at top (fixed)
      div { class: "shrink-0 px-4 pt-3 pb-1 bg-[hsl(0,0%,4%)] z-10",
        div { class: "flex items-center gap-2",
          div { class: "h-1 flex-1 rounded-full bg-white/10",
            div {
              class: "h-full rounded-full bg-blue-500 transition-all duration-700 ease-out",
              style: "{progress_width}",
            }
          }
          span { class: "font-mono text-xs text-white/50", "{progress}%" }
        }
      }

      // Top scroll shadow indicator (shows when scrolled down)
      if *show_top_shadow.read() {
        div { class: "absolute top-[44px] left-0 right-0 h-4 bg-gradient-to-b from-[hsl(0,0%,4%)] to-transparent pointer-events-none z-10" }
      }

      // Content area with scroll shadow tracking
      div {
        class: "flex-1 overflow-y-auto px-4 py-2 scroll-smooth",
        onscroll: {
          let mut show_top_shadow = show_top_shadow;
          move |e| {
            // Show top shadow when scrolled more than 10px
            show_top_shadow.set(e.scroll_top() > 10.0);
          }
        },
        if !has_anything {
          // Empty state
          div { class: "flex h-full items-center justify-center",
            p { class: "max-w-xs text-center text-sm leading-relaxed text-white/30",
              "{empty_message}"
            }
          }
        } else {
          // Content sections
          div { class: "space-y-1 pb-4",
            // Thesis section (Problem, Antithesis, Solution)
            if problem.is_some() || antithesis.is_some() || solution.is_some() {
              SectionHeader { label: "Thesis".to_string() }
              div { class: "space-y-2",
                if let Some(ref p) = problem {
                  ThesisCard {
                    label: "Problem".to_string(),
                    value: p.clone(),
                    accent: None,
                  }
                }
                if let Some(ref a) = antithesis {
                  ThesisCard {
                    label: "Antithesis".to_string(),
                    value: a.clone(),
                    accent: Some("border-purple-500/20 bg-purple-500/5".to_string()),
                  }
                }
                if let Some(ref s) = solution {
                  ThesisCard {
                    label: "Solution".to_string(),
                    value: s.clone(),
                    accent: None,
                  }
                }
              }
            }

            // User/Persona section
            if let Some(ref p) = persona {
              SectionHeader { label: "User".to_string() }
              div { class: "animate-fade-up rounded-lg border border-orange-500/20 bg-orange-500/5 px-3 py-2.5",
                p { class: "text-sm leading-relaxed text-white/90", "{p}" }
              }
            }

            // North Star/Scenario section
            if let Some(ref s) = scenario {
              SectionHeader { label: "North Star".to_string() }
              div { class: "animate-fade-up rounded-lg border border-green-500/20 bg-green-500/5 px-3 py-2.5",
                p { class: "text-sm leading-relaxed text-white/80", "{s}" }
              }
            }

            // Use Cases section
            if !use_cases.is_empty() {
              SectionHeader {
                label: "Use Cases".to_string(),
                count: Some(use_cases.len()),
              }
              div { class: "space-y-0.5",
                for (i, uc) in use_cases.iter().enumerate() {
                  UseCaseRow {
                    key: "{i}",
                    text: uc.clone(),
                    index: i + 1,
                  }
                }
              }
            }

            // Stack/Constraints section
            if let Some(ref c) = constraints {
              SectionHeader { label: "Stack".to_string() }
              div { class: "animate-fade-up rounded-lg border border-white/10 bg-white/5 px-3 py-2.5",
                p { class: "font-mono text-xs leading-relaxed text-white/80", "{c}" }
              }
            }

            // Tasks section
            if !tasks.is_empty() {
              SectionHeader {
                label: "Tasks".to_string(),
                count: Some(tasks.len()),
              }
              // Task detail panel (shown when a task is selected)
              if let Some(idx) = *selected_task.read() {
                if let Some(task) = tasks.get(idx) {
                  div { class: "mb-2",
                    TaskDetail {
                      task: task.clone(),
                      index: idx + 1,
                      on_close: {
                        let mut selected_task = selected_task;
                        move |_| selected_task.set(None)
                      },
                    }
                  }
                }
              }
              // Task rows
              div { class: "space-y-0.5",
                for (i, t) in tasks.iter().enumerate() {
                  TaskRow {
                    key: "{i}",
                    text: t.clone(),
                    index: i + 1,
                    selected: *selected_task.read() == Some(i),
                    on_select: {
                      let mut selected_task = selected_task;
                      let i = i;
                      move |_| {
                        let current = *selected_task.read();
                        selected_task.set(if current == Some(i) { None } else { Some(i) })
                      }
                    },
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

/// Terminal line view
#[component]
fn TerminalLineView(line: TerminalLine) -> Element {
  match line.line_type {
    TerminalLineType::Cmd => rsx! {
      div { class: "flex items-start gap-1.5 text-white",
        if let Some(ref agent) = line.agent {
          span { class: if agent == "claude-code" { "mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium bg-purple-500/20 text-purple-400" }
                        else { "mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium bg-blue-500/20 text-blue-400" },
            "{agent}"
          }
        }
        span { class: "text-green-400", "$" }
        span { class: "text-white", "{line.text}" }
      }
    },
    TerminalLineType::Output => rsx! {
      div { class: "pl-4 text-white/50", "{line.text}" }
    },
    TerminalLineType::Comment => rsx! {
      div { class: "text-white/30", "{line.text}" }
    },
    TerminalLineType::Error => rsx! {
      div { class: "text-red-400", "{line.text}" }
    },
    TerminalLineType::Separator => rsx! { div { class: "h-2" } },
  }
}

/// Graph node types
#[derive(Clone, Debug, PartialEq)]
enum NodeGroup {
  Thesis,
  Persona,
  Scenario,
  UseCase,
  Task,
}

impl NodeGroup {
  fn color(&self) -> &'static str {
    match self {
      NodeGroup::Thesis => "hsl(221, 83%, 53%)",
      NodeGroup::Persona => "hsl(262, 83%, 58%)",
      NodeGroup::Scenario => "hsl(142, 71%, 45%)",
      NodeGroup::UseCase => "hsl(38, 92%, 50%)",
      NodeGroup::Task => "hsl(0, 72%, 51%)",
    }
  }

  fn bg_color(&self) -> &'static str {
    match self {
      NodeGroup::Thesis => "hsla(221, 83%, 53%, 0.15)",
      NodeGroup::Persona => "hsla(262, 83%, 58%, 0.15)",
      NodeGroup::Scenario => "hsla(142, 71%, 45%, 0.15)",
      NodeGroup::UseCase => "hsla(38, 92%, 50%, 0.15)",
      NodeGroup::Task => "hsla(0, 72%, 51%, 0.15)",
    }
  }

  fn label(&self) -> &'static str {
    match self {
      NodeGroup::Thesis => "thesis",
      NodeGroup::Persona => "persona",
      NodeGroup::Scenario => "scenario",
      NodeGroup::UseCase => "usecase",
      NodeGroup::Task => "task",
    }
  }
}

/// Graph node for visualization
#[derive(Clone, Debug)]
struct GraphNode {
  id: String,
  label: String,
  group: NodeGroup,
  x: i32,
  y: i32,
}

/// Graph edge connecting nodes
#[derive(Clone, Debug)]
struct GraphEdge {
  from: String,
  to: String,
}

/// Build graph nodes and edges from answers
fn build_graph(
  answers: &[CoachAnswer],
  width: i32,
  height: i32,
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
  let mut nodes = Vec::new();
  let mut edges = Vec::new();
  let cx = width / 2;
  let cy = height / 2;

  // Thesis nodes in top center cluster
  let problem = get_val(answers, "problem");
  let antithesis = get_val(answers, "antithesis");
  let solution = get_val(answers, "solution");

  if problem.is_some() {
    nodes.push(GraphNode {
      id: "problem".into(),
      label: "Problem".into(),
      group: NodeGroup::Thesis,
      x: cx - 80,
      y: 60,
    });
  }

  if antithesis.is_some() {
    nodes.push(GraphNode {
      id: "antithesis".into(),
      label: "Antithesis".into(),
      group: NodeGroup::Thesis,
      x: cx + 80,
      y: 60,
    });
    if problem.is_some() {
      edges.push(GraphEdge {
        from: "problem".into(),
        to: "antithesis".into(),
      });
    }
  }

  if solution.is_some() {
    nodes.push(GraphNode {
      id: "solution".into(),
      label: "Solution".into(),
      group: NodeGroup::Thesis,
      x: cx,
      y: 130,
    });
    if problem.is_some() {
      edges.push(GraphEdge {
        from: "problem".into(),
        to: "solution".into(),
      });
    }
    if antithesis.is_some() {
      edges.push(GraphEdge {
        from: "antithesis".into(),
        to: "solution".into(),
      });
    }
  }

  // Persona
  let persona = get_val(answers, "persona");
  if persona.is_some() {
    nodes.push(GraphNode {
      id: "persona".into(),
      label: "User".into(),
      group: NodeGroup::Persona,
      x: cx - 160,
      y: 200,
    });
    if solution.is_some() {
      edges.push(GraphEdge {
        from: "solution".into(),
        to: "persona".into(),
      });
    }
  }

  // Scenario (North Star)
  let scenario = get_val(answers, "scenario");
  if scenario.is_some() {
    nodes.push(GraphNode {
      id: "scenario".into(),
      label: "North Star".into(),
      group: NodeGroup::Scenario,
      x: cx + 160,
      y: 200,
    });
    if persona.is_some() {
      edges.push(GraphEdge {
        from: "persona".into(),
        to: "scenario".into(),
      });
    }
    if solution.is_some() {
      edges.push(GraphEdge {
        from: "solution".into(),
        to: "scenario".into(),
      });
    }
  }

  // Use cases - fan out below
  let use_cases = parse_lines(get_val(answers, "use-cases").as_deref());
  let use_case_count = use_cases.len() as i32;
  let uc_start_x = cx - (((use_case_count.saturating_sub(1)) * 70) / 2);

  for (i, uc) in use_cases.iter().enumerate() {
    let id = format!("uc-{i}");
    let short = if uc.len() > 20 {
      format!("{}..", uc.chars().take(18).collect::<String>())
    } else {
      uc.clone()
    };
    nodes.push(GraphNode {
      id: id.clone(),
      label: short,
      group: NodeGroup::UseCase,
      x: uc_start_x + (i as i32) * 70,
      y: 300,
    });
    if scenario.is_some() {
      edges.push(GraphEdge {
        from: "scenario".into(),
        to: id,
      });
    }
  }

  // Tasks - fan out at bottom
  let tasks = parse_lines(get_val(answers, "tasks").as_deref());
  let task_count = tasks.len() as i32;
  let t_start_x = cx - (((task_count.saturating_sub(1)) * 60) / 2);

  for (i, t) in tasks.iter().enumerate() {
    let id = format!("task-{i}");
    let short = t
      .split(':')
      .next()
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .map_or_else(|| t.chars().take(14).collect(), |s| s);
    nodes.push(GraphNode {
      id: id.clone(),
      label: short,
      group: NodeGroup::Task,
      x: t_start_x + (i as i32) * 60,
      y: 400,
    });
    // Link to nearest use case or scenario
    if !use_cases.is_empty() {
      let uc_idx = i.min(use_cases.len() - 1);
      edges.push(GraphEdge {
        from: format!("uc-{uc_idx}"),
        to: id,
      });
    } else if scenario.is_some() {
      edges.push(GraphEdge {
        from: "scenario".into(),
        to: id,
      });
    }
  }

  (nodes, edges)
}

/// Find a node by ID
fn find_node<'a>(nodes: &'a [GraphNode], id: &str) -> Option<&'a GraphNode> {
  nodes.iter().find(|n| n.id == id)
}

/// Graph panel with node visualization
#[component]
fn GraphPanel(answers: Vec<CoachAnswer>) -> Element {
  let hovered_node = use_signal(|| None::<String>);

  // Graph dimensions for layout calculation
  let width = 440;
  let height = 500;

  let (nodes, edges) = build_graph(&answers, width, height);

  // Empty state
  if nodes.is_empty() {
    return rsx! {
      div { class: "flex h-full items-center justify-center",
        p { class: "text-sm text-white/40",
          "Answer questions to see your plan graph grow"
        }
      }
    };
  }

  // Build SVG viewBox
  let view_box = format!("0 0 {width} {height}");

  rsx! {
    div { class: "relative h-full w-full overflow-hidden",
      // SVG Graph
      svg {
        width: "100%",
        height: "100%",
        view_box: "{view_box}",
        class: "cursor-crosshair",

        // Edges
        for edge in &edges {
          if let (Some(from_node), Some(to_node)) = (find_node(&nodes, &edge.from), find_node(&nodes, &edge.to)) {
            line {
              x1: "{from_node.x}",
              y1: "{from_node.y}",
              x2: "{to_node.x}",
              y2: "{to_node.y}",
              stroke: "hsl(0, 0%, 20%)",
              stroke_width: "1",
            }
          }
        }

        // Nodes
        for node in &nodes {
          {
            let is_hovered = *hovered_node.read() == Some(node.id.clone());
            let radius = if is_hovered { 22i32 } else { 18i32 };
            let color = node.group.color();
            let bg_color = node.group.bg_color();
            let node_id = node.id.clone();
            let node_id_for_leave = node.id.clone();

            rsx! {
              g {
                key: "{node.id}",
                // Hover glow circle
                if is_hovered {
                  circle {
                    cx: "{node.x}",
                    cy: "{node.y}",
                    r: "30",
                    fill: "{bg_color}",
                  }
                }
                // Main circle
                circle {
                  cx: "{node.x}",
                  cy: "{node.y}",
                  r: "{radius}",
                  fill: "{bg_color}",
                  stroke: "{color}",
                  stroke_width: if is_hovered { "2" } else { "1.5" },
                  onmouseenter: {
                    let mut hovered_node = hovered_node;
                    let node_id = node_id.clone();
                    move |_| hovered_node.set(Some(node_id.clone()))
                  },
                  onmouseleave: {
                    let mut hovered_node = hovered_node;
                    move |_| hovered_node.set(None)
                  },
                  class: "transition-all duration-150",
                }
                // Label
                text {
                  x: "{node.x}",
                  y: "{node.y + radius + 14}",
                  text_anchor: "middle",
                  fill: "hsl(0, 0%, 80%)",
                  font_size: if is_hovered { "11" } else { "10" },
                  font_family: "system-ui, sans-serif",
                  onmouseenter: {
                    let mut hovered_node = hovered_node;
                    move |_| hovered_node.set(Some(node_id_for_leave.clone()))
                  },
                  onmouseleave: {
                    let mut hovered_node = hovered_node;
                    move |_| hovered_node.set(None)
                  },
                  "{node.label}"
                }
              }
            }
          }
        }
      }

      // Legend
      div { class: "absolute bottom-3 left-3 flex flex-wrap gap-3",
        for group in &[NodeGroup::Thesis, NodeGroup::Persona, NodeGroup::Scenario, NodeGroup::UseCase, NodeGroup::Task] {
          div { class: "flex items-center gap-1.5",
            span {
              class: "inline-block h-2.5 w-2.5 rounded-full",
              style: "background-color: {group.color()}",
            }
            span { class: "font-mono text-xs capitalize text-white/60",
              "{group.label()}"
            }
          }
        }
      }
    }
  }
}

/// Phase color configuration
struct PhaseColors {
  ring: &'static str,
  bg: &'static str,
  text: &'static str,
}

/// Get colors for a phase
fn get_phase_colors(phase: &str) -> PhaseColors {
  match phase {
    "discover" => PhaseColors {
      ring: "ring-blue-500/50",
      bg: "bg-blue-500/10",
      text: "text-blue-400",
    },
    "define" => PhaseColors {
      ring: "ring-purple-500/50",
      bg: "bg-purple-500/10",
      text: "text-purple-400",
    },
    "develop" => PhaseColors {
      ring: "ring-orange-500/50",
      bg: "bg-orange-500/10",
      text: "text-orange-400",
    },
    "deliver" => PhaseColors {
      ring: "ring-green-500/50",
      bg: "bg-green-500/10",
      text: "text-green-400",
    },
    _ => PhaseColors {
      ring: "ring-white/20",
      bg: "bg-white/5",
      text: "text-white/70",
    },
  }
}

/// All required steps across all phases (step_id, title, required)
fn get_all_required_steps() -> Vec<(&'static str, &'static str, bool)> {
  vec![
    ("problem", "Problem Statement", true),
    ("antithesis", "Antithesis", false),
    ("solution", "Solution", true),
    ("persona", "Target Persona", true),
    ("scenario", "North Star", false),
    ("use-cases", "Use Cases", true),
    ("constraints", "Constraints", false),
    ("tasks", "Tasks", true),
  ]
}

/// Phase state information for rendering
struct PhaseState {
  phase: &'static str,
  steps: Vec<CoachStep>,
  total: usize,
  done: usize,
  is_complete: bool,
  is_active: bool,
  current_step: Option<CoachStep>,
}

/// Calculate phase states from answers and active phase
fn calculate_phase_states(answers: &[CoachAnswer], active_phase: &str) -> Vec<PhaseState> {
  let phases: &[&str] = &["discover", "define", "develop", "deliver"];
  let completed_ids: Vec<&str> = answers.iter().map(|a| a.step_id.as_str()).collect();
  let all_steps = get_all_required_steps();

  phases
    .iter()
    .map(|&phase| {
      let steps = get_steps_for_phase_string(phase);
      let required: Vec<_> = steps
        .iter()
        .filter(|s| all_steps.iter().any(|(id, _, req)| *req && *id == s.id))
        .cloned()
        .collect();
      let done = required
        .iter()
        .filter(|s| completed_ids.contains(&s.id.as_str()))
        .count();
      let current_step = steps
        .iter()
        .find(|s| !completed_ids.contains(&s.id.as_str()))
        .cloned();
      let is_complete = !required.is_empty()
        && required
          .iter()
          .all(|s| completed_ids.contains(&s.id.as_str()));
      let is_active = active_phase == phase;

      PhaseState {
        phase,
        steps,
        total: required.len(),
        done,
        is_complete,
        is_active,
        current_step: if is_active { current_step } else { None },
      }
    })
    .collect()
}

/// State panel showing phase progress and step states
#[component]
fn StatePanel(answers: Vec<CoachAnswer>, active_phase: String) -> Element {
  let all_steps = get_all_required_steps();
  let completed_ids: Vec<&str> = answers.iter().map(|a| a.step_id.as_str()).collect();

  // Calculate global progress for required steps only
  let required_steps: Vec<_> = all_steps
    .iter()
    .filter(|(_, _, req)| *req)
    .copied()
    .collect();
  let total_steps = required_steps.len();
  let completed_steps = required_steps
    .iter()
    .filter(|(id, _, _)| completed_ids.contains(id))
    .count();
  let current_global_idx = required_steps
    .iter()
    .position(|(id, _, _)| !completed_ids.contains(id));

  let phase_states = calculate_phase_states(&answers, &active_phase);

  rsx! {
    div { class: "flex h-full flex-col gap-6 p-4",
      // Overall progress
      div { class: "space-y-2",
        div { class: "flex items-center justify-between",
          span { class: "text-xs font-medium uppercase tracking-widest text-white/50",
            "Progress"
          }
          span { class: "font-mono text-xs text-white/50",
            "{completed_steps}/{total_steps}"
          }
        }
        div { class: "flex gap-1",
          for (i, (step_id, _, _)) in required_steps.iter().enumerate() {
            {
              let is_completed = completed_ids.contains(step_id);
              let is_current = current_global_idx == Some(i);
              let class = if is_completed {
                "h-1.5 flex-1 rounded-full bg-blue-500 transition-all duration-500".to_string()
              } else if is_current {
                "h-1.5 flex-1 rounded-full bg-blue-500/40 animate-pulse-glow transition-all duration-500".to_string()
              } else {
                "h-1.5 flex-1 rounded-full bg-white/10 transition-all duration-500".to_string()
              };
              rsx! {
                div { class: "{class}" }
              }
            }
          }
        }
      }

      // Phase state cards
      div { class: "flex flex-1 flex-col gap-3",
        for (phase_idx, ps) in phase_states.iter().enumerate() {
          {
            let colors = get_phase_colors(ps.phase);
            let animation_delay = format!("animation-delay: {}ms", phase_idx * 80);

            // Determine card styling based on state
            let card_class = if ps.is_active {
              format!(
                "rounded-lg border p-3 transition-all duration-300 ring-2 border-transparent animate-state-active {} {}",
                colors.ring, colors.bg
              )
            } else if ps.is_complete {
              "rounded-lg border p-3 transition-all duration-300 border-white/10 bg-white/5".to_string()
            } else {
              "rounded-lg border p-3 transition-all duration-300 border-white/5 bg-transparent".to_string()
            };

            // Determine phase label text class
            let label_class = if ps.is_active {
              format!("text-sm font-medium capitalize {}", colors.text)
            } else if ps.is_complete {
              "text-sm font-medium capitalize text-white/70".to_string()
            } else {
              "text-sm font-medium capitalize text-white/40".to_string()
            };

            rsx! {
              div {
                class: "animate-fade-up",
                style: "{animation_delay}",
                div { class: "{card_class}",
                  // Phase header
                  div { class: "flex items-center justify-between",
                    div { class: "flex items-center gap-2",
                      if ps.is_complete {
                        // Checkmark icon
                        svg {
                          width: "16",
                          height: "16",
                          view_box: "0 0 16 16",
                          fill: "none",
                          class: "text-green-400",
                          circle {
                            cx: "8",
                            cy: "8",
                            r: "7",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                          }
                          path {
                            d: "M5 8L7 10L11 6",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                          }
                        }
                      } else if ps.is_active {
                        // Active indicator
                        span { class: "flex h-4 w-4 items-center justify-center rounded-full {colors.bg}",
                          span { class: "h-2 w-2 rounded-full bg-current {colors.text} animate-pulse" }
                        }
                      } else {
                        // Inactive indicator
                        span { class: "flex h-4 w-4 items-center justify-center rounded-full bg-white/10",
                          span { class: "h-1.5 w-1.5 rounded-full bg-white/20" }
                        }
                      }
                      span { class: "{label_class}",
                        "{ps.phase}"
                      }
                    }
                    span { class: "font-mono text-xs text-white/40",
                      "{ps.done}/{ps.total}"
                    }
                  }

                  // Step sub-states (only for active phase)
                  if ps.is_active && !ps.steps.is_empty() {
                    div { class: "mt-3 space-y-1.5 pl-6",
                      for step in &ps.steps {
                        {
                          let step_done = completed_ids.contains(&step.id.as_str());
                          let is_current_step = ps
                            .current_step
                            .as_ref()
                            .map_or(false, |cs| cs.id == step.id);

                          let step_text_class = if step_done {
                            "text-xs text-white/40 line-through"
                          } else if is_current_step {
                            "text-xs font-medium text-white"
                          } else {
                            "text-xs text-white/30"
                          };

                          rsx! {
                            div { class: "flex items-center gap-2",
                              if step_done {
                                // Done checkmark
                                svg {
                                  width: "12",
                                  height: "12",
                                  view_box: "0 0 12 12",
                                  fill: "none",
                                  class: "text-green-400 shrink-0",
                                  path {
                                    d: "M3 6L5 8L9 4",
                                    stroke: "currentColor",
                                    stroke_width: "1.5",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                  }
                                }
                              } else if is_current_step {
                                // Current step indicator (pinging)
                                span { class: "relative flex h-3 w-3 shrink-0",
                                  span { class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-blue-500/40" }
                                  span { class: "relative inline-flex h-3 w-3 rounded-full bg-blue-500" }
                                }
                              } else {
                                // Pending step
                                span { class: "h-3 w-3 shrink-0 rounded-full border border-white/20" }
                              }
                              span { class: "{step_text_class}",
                                "{step.title}"
                              }
                            }
                          }
                        }
                      }
                    }
                  }

                  // Transition arrow (not for last phase)
                  if phase_idx < 3 {
                    div { class: "mt-2 flex justify-center",
                      svg {
                        width: "12",
                        height: "16",
                        view_box: "0 0 12 16",
                        fill: "none",
                        class: "text-white/10",
                        path {
                          d: "M6 0V12M2 8L6 12L10 8",
                          stroke: "currentColor",
                          stroke_width: "1.5",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
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
  }
}

/// Get terminal commands for a step
fn get_commands_for_step(step_id: &str, value: &str) -> Vec<(String, String, String)> {
  let v = value.chars().take(50).collect::<String>();
  match step_id {
    "problem" => vec![
      (
        "planner".into(),
        "br init --project beads-plan".into(),
        "Initialized .beads/ in current directory".into(),
      ),
      (
        "planner".into(),
        format!("br create --type epic --title \"Problem: {v}...\""),
        format!("Created bd-a1f0  Problem: {v}"),
      ),
    ],
    "antithesis" => vec![(
      "planner".into(),
      format!("br update bd-a1f0 --label antithesis --note \"{v}...\""),
      "Updated bd-a1f0  +label:antithesis".into(),
    )],
    "solution" => vec![
      (
        "planner".into(),
        format!("br create --type epic --title \"Solution: {v}...\""),
        "Created bd-b2e1  Solution".into(),
      ),
      (
        "planner".into(),
        "br dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from".into(),
        "Linked bd-b2e1 -> bd-a1f0 (discovered-from)".into(),
      ),
    ],
    "persona" => vec![(
      "planner".into(),
      format!("br create --type task --parent bd-b2e1 --title \"Persona: {v}...\""),
      "Created bd-b2e1.1  Persona".into(),
    )],
    "use-cases" => value
      .lines()
      .filter(|l| !l.trim().is_empty())
      .enumerate()
      .map(|(i, uc)| {
        (
          "planner".into(),
          format!(
            "br create --type feature --title \"{}...\"",
            uc.chars().take(50).collect::<String>()
          ),
          format!("Created bd-c{i}  {uc}"),
        )
      })
      .collect(),
    "tasks" => {
      let mut cmds = Vec::new();
      for (i, t) in value.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let parts: Vec<_> = t.splitn(2, ':').collect();
        let (mod_name, title) = if parts.len() > 1 {
          (parts[0].trim(), parts[1].trim())
        } else {
          ("core", t.trim())
        };
        cmds.push((
          "claude-code".into(),
          format!("br create --type task --title \"{title}\" --label \"{mod_name}\""),
          format!("Created bd-d{i}  [{mod_name}] {title}"),
        ));
      }
      cmds.push((
        "claude-code".into(),
        "br ready --json".into(),
        format!("[{} task(s) ready]", cmds.len()),
      ));
      cmds
    }
    _ => vec![],
  }
}

// ============================================================================
// Micro-Interaction Components
// ============================================================================

/// Tooltip component for contextual help
/// Shows on hover with proper positioning
#[component]
fn Tooltip(content: String, children: Element) -> Element {
  let is_visible = use_signal(|| false);

  rsx! {
    div {
      class: "relative inline-flex",
      onmouseenter: {
        let mut is_visible = is_visible;
        move |_| is_visible.set(true)
      },
      onmouseleave: {
        let mut is_visible = is_visible;
        move |_| is_visible.set(false)
      },
      {children}

      // Tooltip popup
      if *is_visible.read() {
        div {
          class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 z-50 animate-fade-up",
          div { class: "rounded-md bg-white/10 backdrop-blur-sm border border-white/20 px-2 py-1 text-xs text-white/80 whitespace-nowrap shadow-lg",
            "{content}"
          }
          // Arrow
          div { class: "absolute top-full left-1/2 -translate-x-1/2 -mt-px",
            div { class: "border-4 border-transparent border-t-white/20" }
          }
        }
      }
    }
  }
}

/// Loading dots animation for typing indicators
#[component]
fn LoadingDots() -> Element {
  rsx! {
    span { class: "inline-flex items-center gap-0.5",
      span { class: "w-1 h-1 rounded-full bg-white/50 animate-bounce", style: "animation-delay: 0ms" }
      span { class: "w-1 h-1 rounded-full bg-white/50 animate-bounce", style: "animation-delay: 150ms" }
      span { class: "w-1 h-1 rounded-full bg-white/50 animate-bounce", style: "animation-delay: 300ms" }
    }
  }
}

/// Keyboard shortcut hint with tooltip
/// Shows keyboard shortcut with descriptive tooltip on hover
#[component]
fn KeyboardShortcut(shortcut: String, description: String) -> Element {
  rsx! {
    Tooltip {
      content: description,
      kbd { class: "hidden rounded bg-white/10 px-1.5 py-0.5 font-mono text-[10px] text-white/50 border border-white/10 sm:inline cursor-help",
        "{shortcut}"
      }
    }
  }
}

/// Focus ring wrapper for custom focus styling
/// Use this to wrap elements that need consistent focus-visible behavior
#[component]
fn FocusRing(children: Element) -> Element {
  rsx! {
    div { class: "focus-within:ring-2 focus-within:ring-blue-500/50 focus-within:ring-offset-2 focus-within:ring-offset-[hsl(0,0%,4%)] rounded-md transition-all duration-150",
      {children}
    }
  }
}
