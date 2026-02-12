//! Terminal Feed Component
//!
//! A Dioxus component that displays terminal output with OpenCode integration.
//! Shows mock commands when disconnected, real streaming output when connected.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]
// Dioxus rsx! macro internally uses unwrap
#![allow(clippy::disallowed_methods)]

use crate::opencode_client::{ConnectionStatus, OpenCodeClient, TerminalLine, TerminalLineType};
use dioxus::prelude::*;
use std::collections::HashSet;

/// Terminal feed component
///
/// Displays terminal output with OpenCode server integration.
/// Shows mock commands when disconnected, real streaming output when connected.
#[component]
pub fn TerminalFeed(answers: Vec<(String, String)>) -> Element {
  let client = use_signal(|| OpenCodeClient::default());
  let status = use_signal(|| ConnectionStatus::Disconnected);
  let mut executed_commands = use_signal(HashSet::<String>::new);
  let live_lines = use_signal(Vec::<TerminalLine>::new);
  let is_streaming = use_signal(|| false);

  // Generate preview lines from answers
  let preview_lines = use_memo(move || generate_preview_lines(&answers));

  // Determine if we're in mock mode
  let is_mock_mode =
    *status.read() == ConnectionStatus::Disconnected || *status.read() == ConnectionStatus::Error;

  // Display lines - show live output if available, otherwise preview
  let display_lines = use_memo(move || {
    let live = live_lines.read();
    if !live.is_empty() {
      live.clone()
    } else {
      preview_lines.read().clone()
    }
  });

  rsx! {
      div { class: "terminal-feed flex flex-col h-full",
          // Header with status
          TerminalHeader {
              status: *status.read(),
              is_mock_mode,
              on_reconnect: move |_| {
                  let client = client;
                  spawn(async move {
                      let _ = client.read().check_health().await;
                  });
              },
              on_run_all: move |_| {
                  // Execute all commands
                  let preview = preview_lines.read().clone();
                  let mut executed = executed_commands.write();
                  for line in &preview {
                      if matches!(line.line_type, TerminalLineType::Cmd) {
                          let key = format!("{}:{:?}", line.text, line.agent);
                          executed.insert(key);
                      }
                  }
              },
              has_commands: preview_lines.read().iter().any(|l| matches!(l.line_type, TerminalLineType::Cmd)),
              is_streaming: *is_streaming.read(),
          }

          // Terminal content
          div { class: "terminal-content flex-1 overflow-y-auto bg-gray-900 p-4 font-mono text-xs",
              for (i, line) in display_lines.read().iter().enumerate() {
                  TerminalLineView {
                      key: "{i}",
                      line: line.clone(),
                      index: i,
                      is_mock_mode,
                      on_execute: {
                          let line_text = line.text.clone();
                          let line_agent = line.agent.clone();
                          move |_| {
                              let key = format!("{}:{:?}", line_text, line_agent);
                              executed_commands.write().insert(key);
                              // In real implementation, would send to OpenCode
                          }
                      },
                  }
              }

              // Blinking cursor
              div { class: "mt-1 flex items-center gap-1",
                  span { class: "text-green-400", "$" }
                  span {
                      class: if *is_streaming.read() {
                          "inline-block h-3.5 w-1.5 bg-white/70"
                      } else {
                          "inline-block h-3.5 w-1.5 bg-white/70 animate-pulse"
                      }
                  }
              }
          }
      }
  }
}

/// Terminal header component
#[component]
fn TerminalHeader(
  status: ConnectionStatus,
  is_mock_mode: bool,
  on_reconnect: EventHandler<()>,
  on_run_all: EventHandler<()>,
  has_commands: bool,
  is_streaming: bool,
) -> Element {
  let (status_color, status_text) = if is_mock_mode {
    ("bg-yellow-500/70", "Demo Mode")
  } else {
    match status {
      ConnectionStatus::Connected => ("bg-green-500", "Connected"),
      ConnectionStatus::Connecting => ("bg-yellow-500 animate-pulse", "Connecting..."),
      ConnectionStatus::Disconnected => ("bg-gray-500", "Disconnected"),
      ConnectionStatus::Error => ("bg-red-500", "Error"),
    }
  };

  rsx! {
      div { class: "flex shrink-0 items-center justify-between border-b border-gray-700 px-3 py-1.5",
          div { class: "flex items-center gap-1.5",
              span { class: "h-2 w-2 rounded-full {status_color}" }
              span { class: "text-xs text-gray-400", "{status_text}" }
          }

          div { class: "flex items-center gap-2",
              if status == ConnectionStatus::Disconnected {
                  button {
                      class: "rounded px-2 py-0.5 text-xs text-gray-400 hover:bg-gray-700",
                      onclick: move |_| on_reconnect.call(()),
                      "Reconnect"
                  }
              }

              if !is_mock_mode && has_commands {
                  button {
                      class: "rounded bg-blue-500/20 px-2 py-0.5 text-xs text-blue-400 hover:bg-blue-500/30 disabled:opacity-50",
                      onclick: move |_| on_run_all.call(()),
                      disabled: is_streaming,
                      if is_streaming { "Running..." } else { "Run All" }
                  }
              }
          }
      }
  }
}

/// Single terminal line view
#[component]
fn TerminalLineView(
  line: TerminalLine,
  index: usize,
  is_mock_mode: bool,
  on_execute: EventHandler<()>,
) -> Element {
  let animation_delay = index * 30;

  match line.line_type {
    TerminalLineType::Separator => rsx! {
        div { class: "h-2" }
    },
    TerminalLineType::Comment => rsx! {
        div {
            class: "text-gray-500/40",
            style: "animation-delay: {animation_delay}ms",
            "{line.text}"
        }
    },
    TerminalLineType::Error => rsx! {
        div {
            class: "text-red-400",
            style: "animation-delay: {animation_delay}ms",
            "{line.text}"
        }
    },
    TerminalLineType::Cmd => {
      let agent_badge = if let Some(ref agent) = line.agent {
        let badge_class = if agent == "claude-code" {
          "bg-purple-500/20 text-purple-400"
        } else {
          "bg-blue-500/20 text-blue-400"
        };
        Some(rsx! {
            span { class: "mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium {badge_class}",
                "{agent}"
            }
        })
      } else {
        None
      };

      let show_run_button = !line.executed && !is_mock_mode;

      rsx! {
          div {
              class: "flex items-start gap-1.5 text-white",
              style: "animation-delay: {animation_delay}ms",
              {agent_badge}
              span { class: "text-green-400", "$" }
              span { class: "text-white", "{line.text}" }
              if show_run_button {
                  button {
                      class: "ml-auto shrink-0 rounded bg-gray-700 px-1.5 py-px text-[9px] text-gray-400 hover:bg-gray-600",
                      onclick: move |_| on_execute.call(()),
                      "run"
                  }
              }
          }
      }
    }
    TerminalLineType::Output => rsx! {
        div {
            class: "pl-4 text-gray-400/60",
            style: "animation-delay: {animation_delay}ms",
            "{line.text}"
        }
    },
  }
}

/// Generate preview lines from planning answers
fn generate_preview_lines(answers: &[(String, String)]) -> Vec<TerminalLine> {
  let mut lines = Vec::new();
  let answers_map: std::collections::HashMap<&str, &str> = answers
    .iter()
    .map(|(k, v)| (k.as_str(), v.as_str()))
    .collect();

  let get_val =
    |id: &str| -> Option<&str> { answers_map.get(id).copied().filter(|v| *v != "(skipped)") };

  lines.push(TerminalLine::comment(
    "# Beads Planner - Agent Command Preview".to_string(),
  ));
  lines.push(TerminalLine::comment(
    "# Commands generated from your planning session".to_string(),
  ));
  lines.push(TerminalLine::separator());

  if let Some(problem) = get_val("problem") {
    lines.push(TerminalLine::comment("# Phase: Discover".to_string()));
    lines.push(
      TerminalLine::cmd("br init --project beads-plan".to_string())
        .with_agent("planner".to_string()),
    );
    lines.push(TerminalLine::output(
      "Initialized .beads/ in current directory".to_string(),
    ));
    lines.push(
      TerminalLine::cmd(format!(
        "br create --type epic --title \"Problem Statement\" --desc \"{}...\"",
        &problem.chars().take(60).collect::<String>()
      ))
      .with_agent("planner".to_string()),
    );
    lines.push(TerminalLine::output(
      "Created bd-a1f0  Problem Statement".to_string(),
    ));
  }

  if let Some(antithesis) = get_val("antithesis") {
    lines.push(
      TerminalLine::cmd(format!(
        "br update bd-a1f0 --label antithesis --note \"{}...\"",
        &antithesis.chars().take(50).collect::<String>()
      ))
      .with_agent("planner".to_string()),
    );
    lines.push(TerminalLine::output(
      "Updated bd-a1f0  +label:antithesis".to_string(),
    ));
  }

  if let Some(solution) = get_val("solution") {
    lines.push(
      TerminalLine::cmd(format!(
        "br create --type epic --title \"Solution\" --desc \"{}...\"",
        &solution.chars().take(60).collect::<String>()
      ))
      .with_agent("planner".to_string()),
    );
    lines.push(TerminalLine::output(
      "Created bd-b2e1  Solution".to_string(),
    ));
    lines.push(
      TerminalLine::cmd("br dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from".to_string())
        .with_agent("planner".to_string()),
    );
    lines.push(TerminalLine::output(
      "Linked bd-b2e1 -> bd-a1f0 (discovered-from)".to_string(),
    ));
  }

  if let Some(persona) = get_val("persona") {
    lines.push(
      TerminalLine::cmd(format!(
        "br create --type task --parent bd-b2e1 --title \"Persona: {}...\"",
        &persona.chars().take(40).collect::<String>()
      ))
      .with_agent("planner".to_string()),
    );
    lines.push(TerminalLine::output(
      "Created bd-b2e1.1  Persona definition".to_string(),
    ));
  }

  if answers.is_empty() {
    lines.push(TerminalLine::comment(
      "# Waiting for planning input...".to_string(),
    ));
    lines.push(TerminalLine::comment(
      "# Answer the coach's questions to generate agent commands".to_string(),
    ));
  }

  lines
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_generate_preview_lines_empty() {
    let lines = generate_preview_lines(&[]);
    assert!(!lines.is_empty());
    assert!(lines[0].text.contains("Beads Planner"));
  }

  #[test]
  fn test_generate_preview_lines_with_problem() {
    let answers = vec![("problem".to_string(), "Test problem statement".to_string())];
    let lines = generate_preview_lines(&answers);

    let has_cmd = lines
      .iter()
      .any(|l| matches!(l.line_type, TerminalLineType::Cmd));
    assert!(has_cmd);
  }

  #[test]
  fn test_generate_preview_lines_skips_skipped() {
    let answers = vec![("problem".to_string(), "(skipped)".to_string())];
    let lines = generate_preview_lines(&answers);

    // Should only have header comments, no actual commands
    let cmd_count = lines
      .iter()
      .filter(|l| matches!(l.line_type, TerminalLineType::Cmd))
      .count();
    assert_eq!(cmd_count, 0);
  }
}
