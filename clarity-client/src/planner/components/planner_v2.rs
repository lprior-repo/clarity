//! V2-Style Planner Page
//!
//! Clean split layout matching v2-typescript-sample:
//! - Top: Phase tabs with progress
//! - Left: Chat-style PlanningCoach
//! - Right: Tabbed panel (Plan/Graph/State)
//!
//! ## Responsive Design
//! - Mobile (< 768px): Single column, tabs for switching views
//! - Tablet (768px - 1023px): Split layout with narrower sidebar
//! - Desktop (>= 1024px): Full split layout with wider sidebar
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

use crate::hooks::{use_responsive, ResponsiveState};
use crate::opencode_client::{ConnectionStatus, OpenCodeClient, TerminalLine, TerminalLineType};
use crate::planner::bead_serializer::append_to_beads;
use crate::planner::coach_validation::{validate_step, CoachWarning};
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

/// Build the coaching prompt sent to GLM for a given step answer.
///
/// The prompt asks the model to validate/enrich the answer and surface any
/// gaps, keeping the response concise so it fits inline in the terminal.
fn build_coaching_prompt(step_id: &str, step_question: &str, value: &str) -> String {
  format!(
    "You are a rigorous product planning coach. \
A user just answered a planning question. \
Respond in 2-4 sentences: \
(1) confirm what's strong about their answer, \
(2) flag the most important gap or ambiguity, \
(3) give one concrete suggestion to strengthen it. \
Be direct and concise. \
\n\nQuestion: {step_question}\nAnswer: {value}\nStep: {step_id}"
  )
}

/// V2-style planner page with responsive layout
#[component]
pub fn PlannerV2() -> Element {
  let mut active_phase = use_signal(|| "discover".to_string());
  let mut answers = use_signal(Vec::<CoachAnswer>::new);
  let mut right_tab = use_signal(|| "plan".to_string());
  let client = use_signal(OpenCodeClient::default);
  let mut connection_status = use_signal(|| ConnectionStatus::Disconnected);
  let mut terminal_lines = use_signal(Vec::<TerminalLine>::new);
  let mut is_thinking = use_signal(|| false);
  /// Maps step_id → AI coaching response text
  let mut ai_feedback: Signal<Vec<(String, String)>> = use_signal(Vec::new);
  /// step_id currently being processed by AI (None = idle)
  let mut thinking_step_id: Signal<Option<String>> = use_signal(|| None);
  /// Maps step_id → PME validation warnings (set synchronously on submit)
  let mut validation_warnings: Signal<Vec<(String, Vec<CoachWarning>)>> = use_signal(Vec::new);

  // Responsive state
  let responsive = use_responsive();
  let is_mobile = responsive.is_mobile();
  let is_tablet = responsive.is_tablet();

  // Mobile view toggle (0 = coach, 1 = panel)
  let mut mobile_view = use_signal(|| 0u8);

  // On mount: check health and create a planning session
  use_effect({
    let client = client;
    let mut connection_status = connection_status;
    move || {
      spawn(async move {
        let c = client.read().clone();
        if c.check_health().await {
          connection_status.set(ConnectionStatus::Connected);
          // Create a new session for this planning run (ignore errors — we'll
          // fall back gracefully if it fails)
          let _ = c.create_session("Beads Planning Session").await;
        } else {
          connection_status.set(ConnectionStatus::Disconnected);
        }
      });
    }
  });

  // Step lookup helper — fetches the question text for a given step_id
  let get_step_question = |step_id: &str| -> String {
    use crate::planner::prompts::get_steps_for_phase_string;
    for phase in &["discover", "define", "develop", "deliver"] {
      for step in get_steps_for_phase_string(phase) {
        if step.id == step_id {
          return step.question.clone();
        }
      }
    }
    step_id.to_string()
  };

  let handle_answer = move |(step_id, value): (String, String)| {
    // 1. Store the answer immediately so the UI advances
    let mut current = answers.write().clone();
    current.retain(|a| a.step_id != step_id);
    current.push(CoachAnswer {
      step_id: step_id.clone(),
      value: value.clone(),
    });
    answers.set(current.clone());

    // 2. Run PME validation synchronously using prior answers for cross-step checks
    {
      let priors: Vec<(String, String)> = current
        .iter()
        .filter(|a| a.step_id != step_id)
        .map(|a| (a.step_id.clone(), a.value.clone()))
        .collect();
      let warnings = validate_step(&step_id, &value, &priors);
      let mut wv = validation_warnings.write().clone();
      wv.retain(|(id, _)| id != &step_id);
      wv.push((step_id.clone(), warnings));
      validation_warnings.set(wv);
    }

    // 3. Show a "thinking" entry in the terminal while we wait for the AI
    {
      let mut lines = terminal_lines.write().clone();
      lines.push(TerminalLine::comment(format!(
        "# coach reviewing: {step_id}"
      )));
      terminal_lines.set(lines);
    }
    is_thinking.set(true);
    thinking_step_id.set(Some(step_id.clone()));

    // 4. Spawn async AI call
    let step_question = get_step_question(&step_id);
    let prompt = build_coaching_prompt(&step_id, &step_question, &value);
    let c = client.read().clone();
    let sid = step_id.clone();
    spawn(async move {
      match c.send_message(&prompt).await {
        Ok(ai_lines) => {
          // Collect text parts into a single feedback string
          let text = ai_lines
            .into_iter()
            .filter(|l| l.line_type == crate::opencode_client::TerminalLineType::Output)
            .map(|l| l.text)
            .collect::<Vec<_>>()
            .join("\n");
          // Store AI feedback keyed by step_id
          let mut fb = ai_feedback.write().clone();
          fb.retain(|(id, _)| id != &sid);
          fb.push((sid.clone(), text));
          ai_feedback.set(fb);
        }
        Err(e) => {
          // Degrade gracefully — show the error as the feedback text
          let mut fb = ai_feedback.write().clone();
          fb.retain(|(id, _)| id != &sid);
          fb.push((sid.clone(), format!("[AI offline: {e}]")));
          ai_feedback.set(fb);
        }
      }
      is_thinking.set(false);
      thinking_step_id.set(None);
    });
  };

  let total_req = total_required();
  let total_complete = total_done(&answers.read());

  // Layout classes based on responsive state - use static Tailwind classes
  let main_layout_class = if is_mobile {
    "flex flex-col"
  } else {
    "flex flex-row"
  };

  // Right panel width - use static Tailwind classes that JIT can detect
  let right_panel_class = if is_mobile {
    "w-full"
  } else {
    // Match v2 TypeScript: w-[440px] lg:w-[500px]
    "w-[440px] lg:w-[500px]"
  };

  rsx! {
    div { class: "flex h-screen flex-col overflow-hidden bg-[hsl(0,0%,4%)] text-white font-sans",
      // Top header - responsive padding
      header { class: if is_mobile {
        "flex shrink-0 items-center justify-between border-b border-white/10 px-3 py-2"
      } else {
        "flex shrink-0 items-center justify-between border-b border-white/10 px-5 py-2"
      },
        div { class: "flex items-center gap-4 md:gap-6",
          // Logo - smaller on mobile
          div { class: "flex items-center gap-2",
            div { class: if is_mobile {
              "flex h-5 w-5 items-center justify-center rounded-md bg-blue-500"
            } else {
              "flex h-6 w-6 items-center justify-center rounded-md bg-blue-500"
            },
              svg {
                width: if is_mobile { "12" } else { "14" },
                height: if is_mobile { "12" } else { "14" },
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
            span { class: if is_mobile {
              "text-xs font-bold tracking-tight"
            } else {
              "text-sm font-bold tracking-tight"
            }, "Beads Planner" }
          }

          // Phase tabs - scrollable on mobile
          nav { class: "flex items-center overflow-x-auto scrollbar-hide",
            for (phase_key, label) in PHASES.iter() {
              PhaseTab {
                key: "{phase_key}",
                phase_key: *phase_key,
                label: *label,
                active: *active_phase.read() == *phase_key,
                done: phase_done(*phase_key, &answers.read()),
                is_mobile: is_mobile,
                on_click: {
                  let mut active_phase = active_phase;
                  let phase_key = phase_key.to_string();
                  move |_| active_phase.set(phase_key.clone())
                },
              }
            }
          }
        }

        // Progress counter - hidden on very small screens
        if !is_mobile {
          span { class: "font-mono text-xs text-white/50", "{total_complete}/{total_req}" }
        }
      }

      // Main content - responsive layout
      div { class: "{main_layout_class} flex-1 overflow-hidden min-w-0",
        // Mobile view switcher tabs
        if is_mobile {
          div { class: "flex shrink-0 border-b border-white/10",
            button {
              class: if *mobile_view.read() == 0 {
                "flex-1 px-4 py-2 text-sm font-medium text-white border-b-2 border-blue-500"
              } else {
                "flex-1 px-4 py-2 text-sm font-medium text-white/60 hover:text-white/80"
              },
              onclick: move |_| mobile_view.set(0),
              "Coach"
            }
            button {
              class: if *mobile_view.read() == 1 {
                "flex-1 px-4 py-2 text-sm font-medium text-white border-b-2 border-blue-500"
              } else {
                "flex-1 px-4 py-2 text-sm font-medium text-white/60 hover:text-white/80"
              },
              onclick: move |_| mobile_view.set(1),
              "Plan"
            }
          }
        }

        // Left: Planning Coach (hidden on mobile when viewing panel)
        if !is_mobile || *mobile_view.read() == 0 {
          main { class: if is_mobile {
            "flex-1 overflow-hidden"
          } else {
            "flex-1 overflow-hidden border-r border-white/10"
          },
            PlanningCoach {
              active_phase: active_phase.read().clone(),
              answers: answers.read().clone(),
              validation_warnings: validation_warnings.read().clone(),
              ai_feedback: ai_feedback.read().clone(),
              thinking_step_id: thinking_step_id.read().clone(),
              is_mobile: is_mobile,
              on_answer: handle_answer,
              on_phase_change: {
                let mut active_phase = active_phase;
                move |phase| active_phase.set(phase)
              },
            }
          }
        }

        // Right: Tabbed panel (hidden on mobile when viewing coach)
        if !is_mobile || *mobile_view.read() == 1 {
          div { class: "flex {right_panel_class} shrink-0 flex-col",
            // Tabs - hidden on mobile (using view switcher instead)
            if !is_mobile {
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
            }

            // Panel content
            div { class: "flex-1 overflow-hidden",
              if *right_tab.read() == "plan" || is_mobile {
                ArtifactPanel {
                  answers: answers.read().clone(),
                  active_phase: active_phase.read().clone(),
                  terminal_lines: terminal_lines.read().clone(),
                  connection_status: *connection_status.read(),
                  is_thinking: *is_thinking.read(),
                  is_mobile: is_mobile,
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
  is_mobile: bool,
  on_click: EventHandler<()>,
) -> Element {
  let index_str = phase_index_str(&phase_key);
  // Focus-visible styling for keyboard navigation accessibility
  let focus_class = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[hsl(0,0%,4%)]";
  // Active state for press feedback
  let active_press_class = if active { "" } else { "active:scale-[0.98]" };

  // Responsive sizing
  let padding_class = if is_mobile {
    "px-2 py-1.5"
  } else {
    "px-3 py-2"
  };
  let text_class = if is_mobile { "text-xs" } else { "text-sm" };
  let badge_class = if is_mobile {
    "h-3.5 w-3.5 text-[10px]"
  } else {
    "h-4 w-4 text-xs"
  };

  rsx! {
    button {
      class: if active {
        format!("relative flex items-center gap-1 {padding_class} {text_class} text-white rounded-sm whitespace-nowrap {focus_class}")
      } else {
        format!("relative flex items-center gap-1 {padding_class} {text_class} text-white/60 hover:text-white/80 transition-all duration-150 rounded-sm whitespace-nowrap {focus_class} {active_press_class}")
      },
      onclick: move |_| on_click.call(()),
      if done {
        span { class: if is_mobile { "text-green-400 text-xs" } else { "text-green-400" }, "✓" }
      } else {
        span { class: if active {
          format!("flex {badge_class} items-center justify-center rounded-full bg-blue-500/20 text-blue-400")
        } else {
          format!("flex {badge_class} items-center justify-center rounded-full bg-white/10 text-white/50")
        },
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
  /// PME lattice validation warnings — shown synchronously before AI feedback
  Validation {
    warnings: Vec<CoachWarning>,
  },
  /// AI coaching feedback (GLM response) shown inline after the user's answer
  AiFeedback {
    text: String,
    thinking: bool,
  },
}

/// Build the conversation thread from steps, answers, validation warnings, and AI feedback.
///
/// - `validation_warnings` maps step_id → warnings (shown synchronously after user answer)
/// - `ai_feedback` maps step_id → AI response text (empty = still thinking)
fn build_thread(
  steps: &[CoachStep],
  answers: &[CoachAnswer],
  validation_warnings: &[(String, Vec<CoachWarning>)],
  ai_feedback: &[(String, String)],
  is_thinking_for: Option<&str>,
) -> Vec<ThreadEntry> {
  let mut thread = Vec::new();

  for step in steps {
    let answer_opt = answers.iter().find(|a| a.step_id == step.id);

    thread.push(ThreadEntry::Coach {
      content: step.question.clone(),
      step_title: Some(step.title.clone()),
    });

    if let Some(answer) = answer_opt {
      thread.push(ThreadEntry::User {
        content: answer.value.clone(),
      });

      // Show PME validation warnings synchronously (if any)
      if let Some((_, warnings)) = validation_warnings.iter().find(|(id, _)| id == &step.id) {
        if !warnings.is_empty() {
          thread.push(ThreadEntry::Validation {
            warnings: warnings.clone(),
          });
        }
      }

      // Show AI feedback if available, or a spinner if still thinking
      if let Some(fb) = ai_feedback.iter().find(|(id, _)| id == &step.id) {
        thread.push(ThreadEntry::AiFeedback {
          text: fb.1.clone(),
          thinking: false,
        });
      } else if is_thinking_for == Some(step.id.as_str()) {
        thread.push(ThreadEntry::AiFeedback {
          text: String::new(),
          thinking: true,
        });
      }

      // Add follow-up if present
      if let Some(ref follow_up) = step.follow_up {
        thread.push(ThreadEntry::Coach {
          content: follow_up.clone(),
          step_title: None,
        });
      }
    } else {
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
  validation_warnings: Vec<(String, Vec<CoachWarning>)>,
  ai_feedback: Vec<(String, String)>,
  thinking_step_id: Option<String>,
  is_mobile: bool,
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
  let thread = build_thread(
    &steps,
    &answers,
    &validation_warnings,
    &ai_feedback,
    thinking_step_id.as_deref(),
  );
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

  // Responsive classes
  let padding_class = if is_mobile { "px-4 py-4" } else { "px-6 py-6" };
  let max_width_class = if is_mobile { "max-w-full" } else { "max-w-xl" };
  let button_min_height = if is_mobile { "min-h-[44px]" } else { "" };

  rsx! {
    div { class: "flex h-full flex-col relative",
      // Top scroll shadow indicator (shows when scrolled down)
      if *show_top_shadow.read() {
        div { class: "absolute top-0 left-0 right-0 h-6 bg-gradient-to-b from-[hsl(0,0%,4%)] to-transparent pointer-events-none z-10" }
      }

      // Conversation thread with scroll shadow
      div {
        class: format!("flex-1 overflow-y-auto {padding_class} scroll-smooth"),
        onscroll: {
          let mut show_top_shadow = show_top_shadow;
          move |e| {
            // Show top shadow when scrolled more than 20px
            show_top_shadow.set(e.scroll_top() > 20.0);
          }
        },
        div { class: "mx-auto {max_width_class} space-y-4",
          // Render thread entries
          for (i, entry) in thread.iter().enumerate() {
            match entry {
              ThreadEntry::Coach { content, step_title } => rsx! {
                CoachBubble {
                  key: "coach-{i}",
                  step_title: step_title.clone(),
                  content: content.clone(),
                  is_mobile: is_mobile,
                }
              },
              ThreadEntry::User { content } => rsx! {
                UserBubble {
                  key: "user-{i}",
                  content: content.clone(),
                  is_mobile: is_mobile,
                }
              },
              ThreadEntry::AiFeedback { text, thinking } => rsx! {
                AiFeedbackBubble {
                  key: "ai-{i}",
                  text: text.clone(),
                  thinking: *thinking,
                  is_mobile: is_mobile,
                }
              },
              ThreadEntry::Validation { warnings } => rsx! {
                ValidationBubble {
                  key: "val-{i}",
                  warnings: warnings.clone(),
                  is_mobile: is_mobile,
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
                    class: if is_mobile {
                      "ml-2 group relative rounded-md border border-dashed border-white/20 px-2 py-1.5 text-xs leading-relaxed text-white/50 animate-fade-up transition-colors hover:border-white/30 hover:text-white/60"
                    } else {
                      "ml-10 group relative rounded-md border border-dashed border-white/20 px-3 py-2 text-xs leading-relaxed text-white/50 animate-fade-up transition-colors hover:border-white/30 hover:text-white/60"
                    },
                    // Info icon indicator
                    span { class: "absolute -left-1 -top-1 flex h-4 w-4 items-center justify-center rounded-full bg-white/10 text-[8px] text-white/40 group-hover:bg-white/20 group-hover:text-white/60 transition-colors",
                      "?"
                    },
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
              is_mobile: is_mobile,
            }
            if let Some(next) = &next_phase {
              {
                let label = capitalize_first(next);
                rsx! {
                  div { class: if is_mobile { "ml-2" } else { "ml-10" },
                    button {
                      class: format!("rounded-md bg-blue-500 px-4 py-2 text-sm font-medium text-white hover:bg-blue-600 active:bg-blue-700 active:scale-[0.98] transition-all duration-150 {focus_visible_class} {button_min_height}"),
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
            div { class: if is_mobile {
              "shrink-0 border-t border-white/10 px-4 py-3 bg-[hsl(0,0%,4%)]"
            } else {
              "shrink-0 border-t border-white/10 px-6 py-4 bg-[hsl(0,0%,4%)]"
            },
              div { class: "mx-auto {max_width_class}",
                // Enhanced focus-within styling with ring effect
                div { class: "overflow-hidden rounded-lg border border-white/20 bg-white/5 transition-all duration-200 focus-within:border-blue-500/50 focus-within:ring-2 focus-within:ring-blue-500/20",
                  textarea {
                    // Auto-focus when step changes (via autofocus attribute)
                    class: if is_mobile {
                      "w-full resize-none bg-transparent px-3 py-2.5 text-sm text-white placeholder:text-white/30 focus:outline-none"
                    } else {
                      "w-full resize-none bg-transparent px-4 py-3 text-sm text-white placeholder:text-white/30 focus:outline-none"
                    },
                    placeholder: placeholder_text,
                    rows: if is_mobile { 2 } else { 3 },
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
                  div { class: if is_mobile {
                    "flex items-center justify-between px-3 py-2"
                  } else {
                    "flex items-center justify-between px-4 py-2"
                  },
                    // Skip button with hover/focus states
                    button {
                      class: format!("text-xs text-white/50 hover:text-white/80 active:text-white transition-colors rounded-sm px-2 py-1 -ml-2 {focus_visible_class} {button_min_height}"),
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
                      // Keyboard shortcut hint - hidden on mobile
                      if !is_mobile {
                        kbd { class: "hidden rounded bg-white/10 px-1.5 py-0.5 font-mono text-[10px] text-white/50 border border-white/10 sm:inline",
                          "\u{2318}\u{21B5}"
                        }
                      }
                      // Send button with enhanced states - larger on mobile
                      button {
                        class: if is_mobile {
                          format!("rounded-md bg-blue-500 px-4 py-2 text-sm font-medium text-white hover:bg-blue-600 active:bg-blue-700 active:scale-[0.98] transition-all duration-150 disabled:opacity-30 disabled:pointer-events-none {focus_visible_class} min-h-[44px]")
                        } else {
                          format!("rounded-md bg-blue-500 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-600 active:bg-blue-700 active:scale-[0.98] transition-all duration-150 disabled:opacity-30 disabled:pointer-events-none {focus_visible_class}")
                        },
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
fn CoachBubble(step_title: Option<String>, content: String, is_mobile: bool) -> Element {
  let avatar_class = if is_mobile {
    "h-6 w-6 text-[10px]"
  } else {
    "h-7 w-7 text-xs"
  };
  let content_padding = if is_mobile {
    "px-3 py-2"
  } else {
    "px-4 py-2.5"
  };

  rsx! {
    div { class: "flex gap-2 md:gap-3 animate-fade-up",
      div { class: "flex {avatar_class} shrink-0 items-center justify-center rounded-full bg-blue-500/20 font-bold text-blue-400",
        "B"
      }
      div { class: "max-w-lg",
        if let Some(title) = step_title {
          span { class: "ml-0 mb-1 block text-[10px] font-medium uppercase tracking-widest text-white/40",
            "{title}"
          }
        }
        p { class: "text-sm leading-relaxed text-white/90 {content_padding}", "{content}" }
      }
    }
  }
}

/// User bubble
#[component]
fn UserBubble(content: String, is_mobile: bool) -> Element {
  let padding_class = if is_mobile {
    "px-3 py-2"
  } else {
    "px-4 py-2.5"
  };

  rsx! {
    div { class: "flex justify-end animate-fade-up",
      div { class: "max-w-lg rounded-lg bg-blue-500/10 {padding_class} text-sm leading-relaxed text-white",
        "{content}"
      }
    }
  }
}

/// AI coaching feedback bubble — shown after the user's answer.
///
/// When `thinking` is true renders a pulsing "…" indicator.
/// When `thinking` is false renders the AI text as a dimmer coach-style bubble.
#[component]
fn AiFeedbackBubble(text: String, thinking: bool, is_mobile: bool) -> Element {
  let avatar_class = if is_mobile {
    "h-6 w-6 text-[10px]"
  } else {
    "h-7 w-7 text-xs"
  };
  let content_padding = if is_mobile {
    "px-3 py-2"
  } else {
    "px-4 py-2.5"
  };

  rsx! {
    div { class: "flex gap-2 md:gap-3 animate-fade-up",
      // Distinct avatar for AI feedback — slightly different shade
      div { class: "flex {avatar_class} shrink-0 items-center justify-center rounded-full bg-purple-500/20 font-bold text-purple-400",
        "AI"
      }
      div { class: "max-w-lg",
        span { class: "ml-0 mb-1 block text-[10px] font-medium uppercase tracking-widest text-white/30",
          "Coach Analysis"
        }
        if thinking {
          // Thinking indicator — three animated dots
          div { class: "{content_padding} flex items-center gap-1",
            span { class: "h-1.5 w-1.5 rounded-full bg-white/40 animate-bounce",
              style: "animation-delay: 0ms"
            }
            span { class: "h-1.5 w-1.5 rounded-full bg-white/40 animate-bounce",
              style: "animation-delay: 150ms"
            }
            span { class: "h-1.5 w-1.5 rounded-full bg-white/40 animate-bounce",
              style: "animation-delay: 300ms"
            }
          }
        } else {
          p { class: "text-sm leading-relaxed text-white/70 {content_padding}", "{text}" }
        }
      }
    }
  }
}

/// PME lattice validation warnings bubble — shown synchronously after the user's answer.
///
/// Uses amber/yellow styling to distinguish from AI coach feedback (purple).
/// Only rendered when there are actual warnings to show.
#[component]
fn ValidationBubble(warnings: Vec<CoachWarning>, is_mobile: bool) -> Element {
  if warnings.is_empty() {
    return rsx! {};
  }

  let avatar_class = if is_mobile {
    "h-6 w-6 text-[10px]"
  } else {
    "h-7 w-7 text-xs"
  };
  let content_padding = if is_mobile {
    "px-3 py-2"
  } else {
    "px-4 py-2.5"
  };

  rsx! {
    div { class: "flex gap-2 md:gap-3 animate-fade-up",
      // Amber avatar to signal PME/structural concern
      div { class: "flex {avatar_class} shrink-0 items-center justify-center rounded-full bg-amber-500/20 font-bold text-amber-400",
        "!"
      }
      div { class: "max-w-lg",
        span { class: "ml-0 mb-1 block text-[10px] font-medium uppercase tracking-widest text-amber-400/60",
          "Structure Check"
        }
        div { class: "space-y-1.5 {content_padding}",
          for warning in &warnings {
            div { class: "rounded-md border border-amber-500/20 bg-amber-500/5 px-3 py-2",
              p { class: "text-xs font-semibold text-amber-400 mb-0.5", "{warning.label}" }
              p { class: "text-xs leading-relaxed text-white/70", "{warning.message}" }
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
fn SectionHeader(label: String, count: Option<usize>, is_mobile: bool) -> Element {
  let padding_class = if is_mobile {
    "pb-1.5 pt-4"
  } else {
    "pb-2 pt-5"
  };

  rsx! {
    div { class: "flex items-center gap-2 {padding_class} first:pt-0",
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
fn ThesisCard(label: String, value: String, accent: Option<String>, is_mobile: bool) -> Element {
  if value.is_empty() {
    return rsx! {};
  }
  let accent_classes = accent.map_or_else(|| "border-white/10 bg-white/5".to_string(), |a| a);
  let padding_class = if is_mobile {
    "px-2.5 py-2"
  } else {
    "px-3 py-2.5"
  };

  rsx! {
    div { class: "animate-fade-up rounded-lg border {padding_class} {accent_classes}",
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
/// Enhanced with hover effects and responsive design
#[component]
fn UseCaseRow(text: String, index: usize, is_mobile: bool) -> Element {
  let parsed = parse_use_case(&text);
  let padding_class = if is_mobile {
    "px-1.5 py-1.5"
  } else {
    "px-2 py-2"
  };
  let badge_size = if is_mobile {
    "h-4 w-4 text-[10px]"
  } else {
    "h-5 w-5 text-xs"
  };

  rsx! {
    div { class: "animate-fade-up flex items-start gap-2 md:gap-2.5 rounded-md {padding_class} transition-all duration-150 hover:bg-white/5 group",
      span { class: "mt-0.5 flex {badge_size} shrink-0 items-center justify-center rounded bg-white/10 font-mono text-white/50 group-hover:bg-white/15 transition-colors",
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
fn TaskRow(
  text: String,
  index: usize,
  selected: bool,
  is_mobile: bool,
  on_select: EventHandler<()>,
) -> Element {
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

  // Responsive sizing
  let padding_class = if is_mobile {
    "px-1.5 py-1.5"
  } else {
    "px-2 py-2"
  };
  let gap_class = if is_mobile { "gap-2" } else { "gap-2.5" };
  let badge_size = if is_mobile {
    "h-4 w-4 text-[10px]"
  } else {
    "h-5 w-5 text-xs"
  };
  let min_height = if is_mobile { "min-h-[44px]" } else { "" };

  rsx! {
    button {
      class: format!("animate-fade-up flex w-full items-start {gap_class} rounded-md {padding_class} text-left transition-all duration-150 {bg_class} group {focus_class} {min_height}"),
      onclick: move |_| on_select.call(()),
      span { class: "mt-0.5 flex {badge_size} shrink-0 items-center justify-center rounded font-mono transition-colors {badge_class}",
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
      // Right arrow SVG with hover effect - hidden on very small screens
      if !is_mobile {
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
}

/// Expandable task detail panel with Acceptance Criteria and Edge Cases
/// Enhanced with focus-visible states for close button and responsive design
#[component]
fn TaskDetail(task: String, index: usize, is_mobile: bool, on_close: EventHandler<()>) -> Element {
  let (module, title) = parse_task(&task);
  let focus_class = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[hsl(0,0%,4%)]";

  // Responsive sizing
  let padding_class = if is_mobile {
    "px-2.5 py-1.5"
  } else {
    "px-3 py-2"
  };
  let content_padding = if is_mobile {
    "px-2.5 py-2.5"
  } else {
    "px-3 py-3"
  };
  let badge_size = if is_mobile {
    "h-4 w-4 text-[10px]"
  } else {
    "h-5 w-5 text-xs"
  };
  let close_button_size = if is_mobile {
    "min-w-[44px] min-h-[44px]"
  } else {
    ""
  };

  rsx! {
    div { class: "animate-fade-up rounded-lg border border-blue-500/20 bg-blue-500/5",
      // Header
      div { class: "flex items-center justify-between border-b border-blue-500/10 {padding_class}",
        div { class: "flex items-center gap-2",
          span { class: "flex {badge_size} items-center justify-center rounded bg-blue-500 font-mono text-white",
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
          class: format!("rounded p-1.5 text-white/50 hover:text-white hover:bg-white/10 active:bg-white/20 transition-all duration-150 {focus_class} {close_button_size}"),
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
      div { class: "space-y-3 {content_padding}",
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
/// Enhanced with scroll shadow indicators and responsive design
#[component]
fn ArtifactPanel(
  answers: Vec<CoachAnswer>,
  active_phase: String,
  terminal_lines: Vec<TerminalLine>,
  connection_status: ConnectionStatus,
  is_thinking: bool,
  is_mobile: bool,
) -> Element {
  let selected_task = use_signal(|| None::<usize>);
  // Track scroll position for scroll shadow indicator
  let show_top_shadow = use_signal(|| false);
  // Export state: None = idle, Some(msg) = success/error message
  let mut export_status: Signal<Option<String>> = use_signal(|| None);

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

  // Responsive classes
  let padding_class = if is_mobile {
    "px-3 pt-2 pb-1"
  } else {
    "px-4 pt-3 pb-1"
  };
  let content_padding = if is_mobile { "px-3 py-2" } else { "px-4 py-2" };

  rsx! {
    div { class: "flex h-full flex-col relative",
      // Progress bar at top (fixed)
      div { class: "shrink-0 {padding_class} bg-[hsl(0,0%,4%)] z-10",
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
        div { class: "absolute top-[36px] md:top-[44px] left-0 right-0 h-4 bg-gradient-to-b from-[hsl(0,0%,4%)] to-transparent pointer-events-none z-10" }
      }

      // Content area with scroll shadow tracking
      div {
        class: format!("flex-1 overflow-y-auto {content_padding} scroll-smooth"),
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
              SectionHeader { label: "Thesis".to_string(), is_mobile: is_mobile }
              div { class: "space-y-2",
                if let Some(ref p) = problem {
                  ThesisCard {
                    label: "Problem".to_string(),
                    value: p.clone(),
                    accent: None,
                    is_mobile: is_mobile,
                  }
                }
                if let Some(ref a) = antithesis {
                  ThesisCard {
                    label: "Antithesis".to_string(),
                    value: a.clone(),
                    accent: Some("border-purple-500/20 bg-purple-500/5".to_string()),
                    is_mobile: is_mobile,
                  }
                }
                if let Some(ref s) = solution {
                  ThesisCard {
                    label: "Solution".to_string(),
                    value: s.clone(),
                    accent: None,
                    is_mobile: is_mobile,
                  }
                }
              }
            }

            // User/Persona section
            if let Some(ref p) = persona {
              SectionHeader { label: "User".to_string(), is_mobile: is_mobile }
              div { class: "animate-fade-up rounded-lg border border-orange-500/20 bg-orange-500/5 px-3 py-2.5",
                p { class: "text-sm leading-relaxed text-white/90", "{p}" }
              }
            }

            // North Star/Scenario section
            if let Some(ref s) = scenario {
              SectionHeader { label: "North Star".to_string(), is_mobile: is_mobile }
              div { class: "animate-fade-up rounded-lg border border-green-500/20 bg-green-500/5 px-3 py-2.5",
                p { class: "text-sm leading-relaxed text-white/80", "{s}" }
              }
            }

            // Use Cases section
            if !use_cases.is_empty() {
              SectionHeader {
                label: "Use Cases".to_string(),
                count: Some(use_cases.len()),
                is_mobile: is_mobile,
              }
              div { class: "space-y-0.5",
                for (i, uc) in use_cases.iter().enumerate() {
                  UseCaseRow {
                    key: "{i}",
                    text: uc.clone(),
                    index: i + 1,
                    is_mobile: is_mobile,
                  }
                }
              }
            }

            // Stack/Constraints section
            if let Some(ref c) = constraints {
              SectionHeader { label: "Stack".to_string(), is_mobile: is_mobile }
              div { class: "animate-fade-up rounded-lg border border-white/10 bg-white/5 px-3 py-2.5",
                p { class: "font-mono text-xs leading-relaxed text-white/80", "{c}" }
              }
            }

            // Tasks section
            if !tasks.is_empty() {
              SectionHeader {
                label: "Tasks".to_string(),
                count: Some(tasks.len()),
                is_mobile: is_mobile,
              }
              // Task detail panel (shown when a task is selected)
              if let Some(idx) = *selected_task.read() {
                if let Some(task) = tasks.get(idx) {
                  div { class: "mb-2",
                    TaskDetail {
                      task: task.clone(),
                      index: idx + 1,
                      is_mobile: is_mobile,
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
                    is_mobile: is_mobile,
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

            // Export bead button — only shown when plan is 100% complete
            if progress >= 100 {
              div { class: "pt-4",
                // Status message (success or error)
                if let Some(ref msg) = *export_status.read() {
                  div {
                    class: if msg.starts_with("Saved") {
                      "mb-2 rounded-md border border-green-500/30 bg-green-500/10 px-3 py-2 text-xs text-green-400"
                    } else {
                      "mb-2 rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-400"
                    },
                    "{msg}"
                  }
                }
                button {
                  class: "w-full rounded-lg border border-blue-500/40 bg-blue-500/10 px-4 py-2.5 text-sm font-medium text-blue-300 transition-colors hover:bg-blue-500/20 hover:text-blue-200 active:scale-[0.98]",
                  onclick: {
                    let answers = answers.clone();
                    move |_| {
                      match append_to_beads(&answers, ".beads/issues.jsonl") {
                        Ok(id) => export_status.set(Some(format!("Saved bead {id} to .beads/issues.jsonl"))),
                        Err(e) => export_status.set(Some(format!("Export failed: {e}"))),
                      }
                    }
                  },
                  "Export Bead"
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
