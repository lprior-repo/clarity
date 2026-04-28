#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::types::{get_steps_for_phase, Answer};

/// Terminal command for visualization
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TermCmd {
  pub agent: String,
  pub cmd: String,
  pub out: String,
}

#[must_use]
pub fn truncate(value: &str, max: usize) -> &str {
  if value.len() > max {
    let end = value.floor_char_boundary(max);
    &value[..end]
  } else {
    value
  }
}

fn non_empty_lines(value: &str) -> Vec<&str> {
  value
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect()
}

fn problem_cmds(value: &str) -> Vec<TermCmd> {
  vec![
    TermCmd {
      agent: "planner".into(),
      cmd: "bd init --project beads-plan".into(),
      out: "Initialized .beads/ — SQLite + JSONL ready".into(),
    },
    TermCmd {
      agent: "planner".into(),
      cmd: format!("bd create -t epic --title \"Problem: {value}...\""),
      out: "Created bd-a1f0  [epic] Problem Statement".into(),
    },
  ]
}

fn antithesis_cmds(value: &str) -> Vec<TermCmd> {
  vec![TermCmd {
    agent: "planner".into(),
    cmd: format!("bd update bd-a1f0 --label antithesis --note \"{value}...\""),
    out: "Updated bd-a1f0  +label:antithesis".into(),
  }]
}

fn solution_cmds(value: &str) -> Vec<TermCmd> {
  vec![
    TermCmd {
      agent: "planner".into(),
      cmd: format!("bd create -t epic --title \"Solution: {value}...\""),
      out: "Created bd-b2e1  [epic] Solution".into(),
    },
    TermCmd {
      agent: "planner".into(),
      cmd: "bd dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from".into(),
      out: "Edge: bd-b2e1 -[discovered-from]-> bd-a1f0".into(),
    },
  ]
}

fn persona_cmds(value: &str) -> Vec<TermCmd> {
  vec![TermCmd {
    agent: "planner".into(),
    cmd: format!("bd create -t task --parent bd-b2e1 --title \"Persona: {value}...\""),
    out: "Created bd-b2e1.1  [task] Persona".into(),
  }]
}

fn scenario_cmds() -> Vec<TermCmd> {
  vec![
    TermCmd {
      agent: "planner".into(),
      cmd: "bd create -t task --parent bd-b2e1 --title \"North Star Scenario\"".into(),
      out: "Created bd-b2e1.2  [task] North Star".into(),
    },
    TermCmd {
      agent: "planner".into(),
      cmd: "bd dep add bd-b2e1.2 --related bd-b2e1.1".into(),
      out: "Edge: bd-b2e1.2 -[related]-> bd-b2e1.1".into(),
    },
    TermCmd {
      agent: "planner".into(),
      cmd: "bd show bd-b2e1 --graph".into(),
      out: "Graph: 2 nodes, 3 edges, 0 cycles  [valid]".into(),
    },
  ]
}

fn use_case_cmds(value: &str) -> Vec<TermCmd> {
  let lines = non_empty_lines(value);
  let count_cmd = TermCmd {
    agent: "planner".into(),
    cmd: "bd list --status open --json | jq length".into(),
    out: (lines.len() + 2).to_string(),
  };

  lines
    .iter()
    .enumerate()
    .map(|(i, use_case)| {
      let short = truncate(use_case, 48);
      let out_short = truncate(use_case, 28);
      TermCmd {
        agent: "planner".into(),
        cmd: format!("bd create -t feature --title \"{short}...\""),
        out: format!("Created bd-c{i}d{i}  [feature] {out_short}..."),
      }
    })
    .chain(std::iter::once(count_cmd))
    .collect()
}

fn constraints_cmds(value: &str) -> Vec<TermCmd> {
  vec![
    TermCmd {
      agent: "planner".into(),
      cmd: format!("bd update bd-b2e1 --label stack --note \"{value}...\""),
      out: "Updated bd-b2e1  +label:stack".into(),
    },
    TermCmd {
      agent: "claude-code".into(),
      cmd: "bd ready --assignee claude-code --json".into(),
      out: "[] — no tasks claimed yet".into(),
    },
  ]
}

fn tasks_cmds(value: &str) -> Vec<TermCmd> {
  let lines = non_empty_lines(value);
  let task_count = lines.len();
  let final_cmds = [
    TermCmd {
      agent: "opencode".into(),
      cmd: "bd ready --json".into(),
      out: format!("[{task_count} task(s) on execution frontier]"),
    },
    TermCmd {
      agent: "opencode".into(),
      cmd: "bd list --status open --fmt table".into(),
      out: format!("{task_count} open  0 in-progress  0 done"),
    },
  ];

  lines
    .iter()
    .enumerate()
    .flat_map(|(i, task)| {
      let (module, title) = task
        .split_once(':')
        .map_or(("core", *task), |(m, rest)| (m.trim(), rest.trim()));

      let title_short = truncate(title, 44);
      let create_cmd = TermCmd {
        agent: "claude-code".into(),
        cmd: format!("bd create -t task --title \"{title_short}\" --label \"{module}\" -p P2"),
        out: format!("Created bd-d{i}e{i}  [{module}]"),
      };

      let dep_cmd = if i > 0 {
        Some(TermCmd {
          agent: "claude-code".into(),
          cmd: format!("bd dep add bd-d{i}e{i} --related bd-d{}e{}", i - 1, i - 1),
          out: format!("Edge: bd-d{i}e{i} -[related]-> bd-d{}e{}", i - 1, i - 1),
        })
      } else {
        None
      };

      std::iter::once(create_cmd).chain(dep_cmd)
    })
    .chain(final_cmds)
    .collect()
}

/// Generate terminal commands for a given step answer
fn cmds_for_step(id: &str, val: &str) -> Vec<TermCmd> {
  let value = truncate(val, 55);

  match id {
    "problem" => problem_cmds(value),
    "antithesis" => antithesis_cmds(value),
    "solution" => solution_cmds(value),
    "persona" => persona_cmds(value),
    "scenario" => scenario_cmds(),
    "use-cases" => use_case_cmds(val),
    "constraints" => constraints_cmds(value),
    "tasks" => tasks_cmds(val),
    _ => vec![],
  }
}

/// Get color class for an agent
fn agent_color(agent: &str) -> (&'static str, &'static str) {
  match agent {
    "planner" => (
      "bg-blue-500/20 text-blue-400 ring-1 ring-blue-500/30",
      "#3b82f6",
    ),
    "claude-code" => (
      "bg-amber-500/20 text-amber-400 ring-1 ring-amber-500/30",
      "#f59e0b",
    ),
    "opencode" => (
      "bg-emerald-500/20 text-emerald-400 ring-1 ring-emerald-500/30",
      "#10b981",
    ),
    _ => (
      "bg-gray-500/20 text-gray-400 ring-1 ring-gray-500/30",
      "#6b7280",
    ),
  }
}

/// Frame data for terminal rendering
#[derive(Clone, Debug)]
struct FrameData {
  line_idx: usize,
  kind: String,
  agent: Option<String>,
  text: String,
  timestamp: String,
}

/// Build frame data from commands
fn build_frames(cmds: &[TermCmd], visible_count: usize) -> Vec<FrameData> {
  let mut frames = Vec::new();
  let mut char_count = 0;

  for (i, cmd) in cmds.iter().enumerate() {
    let cmd_visible = if char_count + cmd.cmd.len() <= visible_count {
      cmd.cmd.clone()
    } else if char_count < visible_count {
      cmd.cmd[..visible_count - char_count].to_string()
    } else {
      String::new()
    };
    char_count += cmd.cmd.len();

    let ts = format!("{:02}:{:02}:{:02}", 10 + i / 3600, (i / 60) % 60, i % 60);
    frames.push(FrameData {
      line_idx: i * 2,
      kind: "cmd".to_string(),
      agent: Some(cmd.agent.clone()),
      text: cmd_visible,
      timestamp: ts,
    });

    let out_visible = if char_count + cmd.out.len() <= visible_count {
      cmd.out.clone()
    } else if char_count < visible_count {
      cmd.out[..visible_count - char_count].to_string()
    } else {
      String::new()
    };
    char_count += cmd.out.len();

    let ts_out = format!(
      "{:02}:{:02}:{:02}",
      10 + i / 3600 + 2,
      ((i + 2) / 60) % 60,
      (i + 2) % 60
    );
    frames.push(FrameData {
      line_idx: i * 2 + 1,
      kind: "out".to_string(),
      agent: None,
      text: out_visible,
      timestamp: ts_out,
    });
  }

  frames.into_iter().filter(|f| !f.text.is_empty()).collect()
}

/// Render a single terminal frame
fn render_frame(frame: &FrameData) -> Element {
  let FrameData {
    line_idx,
    kind,
    agent,
    text,
    timestamp,
  } = frame;

  let is_cmd = kind == "cmd";
  let text_class = if is_cmd {
    "text-white/90"
  } else {
    "text-white/35"
  };

  rsx! {
      div {
          key: "{line_idx}",
          class: "flex items-start gap-2 animate-term-line",
          style: "animation-delay: 0ms; animation-fill-mode: both;",

          // Timestamp
          span { class: "mt-px shrink-0 select-none font-mono text-[9px] text-white/15 tabular-nums", "{timestamp}" }

          // Agent badge (cmd lines only)
          if is_cmd {
              if let Some(ag) = agent {
                  {
                      let (badge, _) = agent_color(ag);
                      rsx! {
                          span {
                              class: format!("mt-px shrink-0 rounded px-1.5 py-px text-[9px] font-semibold leading-none tracking-wide {badge}"),
                              "{ag}"
                          }
                      }
                  }
              }
          }

          // Prompt char
          if is_cmd {
              span { class: "shrink-0 select-none text-emerald-500/80", "$" }
          } else {
              span { class: "shrink-0 select-none pl-[4.5rem] text-white/15", "→" }
          }

          // Text
          span { class: "{text_class}", "{text}" }
      }
  }
}

/// Inline terminal stream component - shows animated terminal commands
#[component]
pub fn InlineTerminalStream(cmds: Vec<TermCmd>, step_id: String) -> Element {
  let _ = step_id;
  let visible_chars = use_signal(|| 0usize);
  let total_chars: usize = cmds.iter().map(|c| c.cmd.len() + c.out.len()).sum();

  let visible_count = *visible_chars.read();
  let frames = build_frames(&cmds, visible_count);
  let all_done = visible_count >= total_chars;

  let last_timestamp = frames
    .last()
    .map_or_else(|| "00:00:00".to_string(), |f| f.timestamp.clone());

  let frame_elements: Vec<Element> = frames.iter().map(render_frame).collect();

  rsx! {
      div {
          class: "relative my-3 overflow-hidden rounded-lg border border-white/[0.08] bg-[hsl(0,0%,2%)] shadow-xl",

          // Scanline texture
          div {
              aria_hidden: "true",
              class: "pointer-events-none absolute inset-0 z-10 rounded-lg",
              style: "background-image: repeating-linear-gradient(0deg,transparent,transparent 3px,rgba(0,0,0,0.15) 3px,rgba(0,0,0,0.15) 4px);"
          }

          // CRT vignette
          div {
              aria_hidden: "true",
              class: "pointer-events-none absolute inset-0 z-10 rounded-lg",
              style: "background: radial-gradient(ellipse at 50% 50%,transparent 55%,rgba(0,0,0,0.5) 100%);"
          }

          // Title bar
          div {
              class: "relative z-20 flex items-center gap-2 border-b border-white/[0.06] px-3 py-1.5",
              div { class: "flex gap-1",
                  span { class: "h-2.5 w-2.5 rounded-full bg-red-500/60" }
                  span { class: "h-2.5 w-2.5 rounded-full bg-amber-500/60" }
                  span { class: "h-2.5 w-2.5 rounded-full bg-emerald-500/60" }
              }
              span { class: "flex-1 text-center font-mono text-[10px] text-white/20 select-none", "beads-cli — agent session" }
              if !all_done {
                  span { class: "flex items-center gap-1.5",
                      span { class: "h-1.5 w-1.5 rounded-full bg-emerald-400", style: "box-shadow: 0 0 6px #10b981; animation: pulse 1s ease-in-out infinite;" }
                      span { class: "font-mono text-[10px] text-emerald-400/80", "running" }
                  }
              } else {
                  span { class: "flex items-center gap-1.5",
                      span { class: "h-1.5 w-1.5 rounded-full bg-white/15" }
                      span { class: "font-mono text-[10px] text-white/25", "done" }
                  }
              }
          }

          // Body
          div { class: "relative z-20 space-y-0 px-3 pb-3 pt-2 font-mono text-xs leading-[1.7]",
              for frame in frame_elements.iter() {
                  {frame.clone()}
              }

              // Idle prompt after completion
              if all_done {
                  div { class: "flex items-center gap-2",
                      span {
                          class: "shrink-0 select-none font-mono text-[9px] text-white/15",
                          "{last_timestamp}"
                      }
                      span { class: "select-none text-emerald-500/80", "$" }
                      span { class: "ml-px inline-block h-[0.85em] w-[6px] translate-y-[1px] bg-white/60 align-text-bottom animate-terminal-blink" }
                  }
              }
          }
      }
  }
}

/// Coach chat bubble
#[component]
fn CoachBubble(
  label: Option<String>,
  #[props(default)] step_number: Option<usize>,
  children: Element,
) -> Element {
  let badge_text = step_number.map_or_else(|| "?".to_string(), |n| n.to_string());
  rsx! {
      div { class: "flex gap-3 animate-fade-up",
          div { class: "flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-primary/20 text-xs font-bold text-primary ring-1 ring-primary/30", "{badge_text}" }
          div { class: "flex-1 space-y-1",
              if let Some(lbl) = label {
                  span { class: "block text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/50", "{lbl}" }
              }
              div { class: "max-w-lg text-sm leading-relaxed text-foreground", {children} }
          }
      }
  }
}

/// User chat bubble
#[component]
fn UserBubble(children: Element) -> Element {
  rsx! {
      div { class: "flex animate-fade-up justify-end",
          div { class: "max-w-lg rounded-lg bg-secondary px-4 py-2.5 text-sm leading-relaxed text-foreground ring-1 ring-white/[0.06]", {children} }
      }
  }
}

/// Agent call data for HUD
#[derive(Clone, Debug)]
struct AgentCallData {
  agent: String,
  count: usize,
  color: String,
}

/// Build agent call data from answers
fn build_agent_calls(answers: &[Answer]) -> Vec<AgentCallData> {
  let mut agents: HashMap<String, usize> = HashMap::new();

  for answer in answers {
    for cmd in cmds_for_step(&answer.step_id, &answer.value) {
      *agents.entry(cmd.agent).or_insert(0) += 1;
    }
  }

  agents
    .into_iter()
    .map(|(agent, count)| AgentCallData {
      color: agent_color(&agent).1.to_string(),
      agent,
      count,
    })
    .collect()
}

/// HUD strip showing API call counts
#[component]
fn HUDStrip(answers: Vec<Answer>) -> Element {
  let agent_calls = build_agent_calls(&answers);
  let total: usize = agent_calls.iter().map(|a| a.count).sum();

  rsx! {
      div { class: "flex shrink-0 items-center gap-3 border-b border-white/[0.04] bg-[hsl(0,0%,3%)] px-6 py-1",
          span { class: "font-mono text-[10px] text-white/20", "API CALLS" }
          span { class: "font-mono text-[10px] font-bold text-primary", "{total}" }
          for call in agent_calls.iter() {
              span { class: "font-mono text-[10px]", style: format!("color: {color}", color = call.color), "{call.agent}:{call.count}" }
          }
          span { class: "ml-auto h-1.5 w-1.5 animate-pulse rounded-full bg-[hsl(142,71%,45%)]" }
      }
  }
}

/// Thread entry types
#[derive(Clone, Debug)]
enum ThreadEntry {
  Coach {
    content: String,
    label: Option<String>,
    step_number: Option<usize>,
  },
  User {
    content: String,
  },
  Terminal {
    cmds: Vec<TermCmd>,
    step_id: String,
  },
}

/// Build thread entries from phase steps and answers
fn build_thread_entries(
  phase_steps: &[&crate::types::PromptStep],
  answers: &[Answer],
) -> Vec<ThreadEntry> {
  phase_steps
    .iter()
    .enumerate()
    .flat_map(|(idx, step)| {
      let step_number = Some(idx + 1);
      let answer = answers.iter().find(|a| a.step_id == step.id);

      let mut entries = vec![ThreadEntry::Coach {
        content: step.question.clone(),
        label: Some(step.title.clone()),
        step_number,
      }];

      if let Some(ans) = answer {
        entries.push(ThreadEntry::User {
          content: ans.value.clone(),
        });

        let cmds = cmds_for_step(&step.id, &ans.value);
        if !cmds.is_empty() {
          entries.push(ThreadEntry::Terminal {
            cmds,
            step_id: step.id.clone(),
          });
        }

        if let Some(ref follow_up) = step.follow_up {
          entries.push(ThreadEntry::Coach {
            content: follow_up.clone(),
            label: None,
            step_number: None, // Follow-ups don't get a number
          });
        }
      }

      entries
    })
    .collect()
}

/// Render a thread entry
fn render_thread_entry(entry: &ThreadEntry) -> Element {
  match entry {
    ThreadEntry::Coach {
      content,
      label,
      step_number,
    } => {
      let content = content.clone();
      let label = label.clone();
      let step_number = *step_number;
      rsx! {
          CoachBubble { label, step_number, "{content}" }
      }
    }
    ThreadEntry::User { content } => {
      let content = content.clone();
      rsx! {
          UserBubble { "{content}" }
      }
    }
    ThreadEntry::Terminal { cmds, step_id } => {
      let cmds = cmds.clone();
      let step_id = step_id.clone();
      rsx! {
          InlineTerminalStream { cmds, step_id }
      }
    }
  }
}

/// `PlanningCoach` component - main coaching interface
#[component]
pub fn PlanningCoach(
  active_phase: Signal<String>,
  answers: Signal<Vec<Answer>>,
  mut_answers: Signal<Vec<Answer>>,
  mut_active_phase: Signal<String>,
) -> Element {
  let mut draft = use_signal(String::new);

  let completed_ids: Vec<String> = answers.read().iter().map(|a| a.step_id.clone()).collect();

  let active_phase_val = active_phase.read();
  let phase_steps = get_steps_for_phase(&active_phase_val);
  drop(active_phase_val);

  let current_step = phase_steps.iter().find(|s| !completed_ids.contains(&s.id));

  let thread_entries = build_thread_entries(&phase_steps, &answers.read());

  let phase_complete = phase_steps
    .iter()
    .all(|s| !s.required || completed_ids.contains(&s.id));

  let phases = ["discover", "define", "develop", "deliver"];
  let active_phase_str = active_phase.read();
  let current_idx = phases.iter().position(|&p| *active_phase_str == p);
  drop(active_phase_str);
  let next_phase = current_idx
    .and_then(|i| phases.get(i + 1).copied())
    .map(String::from);

  let thread_elements: Vec<Element> = thread_entries.iter().map(render_thread_entry).collect();

  rsx! {
      div { class: "flex h-full flex-col",
          HUDStrip { answers: answers.read().clone() }

          // Conversation scroll area
          div { class: "flex-1 overflow-y-auto px-6 py-6 scroll-smooth",
              div { class: "mx-auto max-w-xl space-y-4",
                  for entry in thread_elements.iter() {
                      {entry.clone()}
                  }

                  // Hint card
                  if let Some(step) = current_step {
                      if !phase_complete {
                          div { class: "ml-10 animate-fade-up rounded-md border border-dashed border-white/[0.08] px-3 py-2 text-xs leading-relaxed text-muted-foreground/50", "{step.hint}" }
                      }
                  }

                  // Phase complete CTA
                  if phase_complete {
                      div { class: "space-y-3 pt-1",
                          CoachBubble {
                              label: None::<String>,
                              if next_phase.is_some() {
                                  "This phase is locked in. Ready to move forward?"
                              } else {
                                  "Plan fully specified. Review tasks in the sidebar, then hand off to agents."
                              }
                          }
                          if let Some(np) = &next_phase {
                              div { class: "ml-10",
                                  {
                                      let np = np.clone();
                                      let first_char = np.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
                                      let rest = if np.len() > 1 { &np[1..] } else { "" };
                                      let label = format!("Continue to {first_char}{rest}");
                                      rsx! {
                                          button {
                                              "type": "button",
                                              onclick: move |_| mut_active_phase.set(np.clone()),
                                              class: "rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground ring-1 ring-primary/40 transition-all hover:bg-primary/85 hover:ring-primary/70",
                                              "{label}"
                                          }
                                      }
                                  }
                              }
                          }
                      }
                  }
              }
          }

          // Input bar
          if let Some(step) = current_step {
              if !phase_complete {
                  {
                      let step_title = &step.title;
                      let placeholder = format!("{step_title}...");
                      let step_id = step.id.clone();
                      let step_id_for_skip = step_id.clone();
                      let step_required = step.required;
                      let draft_len = draft.read().len();
                      let draft_val = draft.read();
                      let is_draft_empty = draft_val.trim().is_empty();
                      drop(draft_val);
                      let char_count_text = if draft_len > 0 {
                          format!("{draft_len} chars")
                      } else {
                          String::new()
                      };

                      rsx! {
                          div { class: "shrink-0 border-t border-white/[0.06] px-6 py-4",
                              div { class: "mx-auto max-w-xl",
                                  div { class: "overflow-hidden rounded-lg border border-white/[0.08] bg-card ring-0 transition-all focus-within:border-primary/40 focus-within:ring-1 focus-within:ring-primary/20",
                                      textarea {
                                          value: "{draft}",
                                          oninput: move |e: Event<FormData>| {
                                              *draft.write() = e.value();
                                          },
                                          placeholder: "{placeholder}",
                                          rows: "3",
                                          class: "w-full resize-none bg-transparent px-4 py-3 text-sm text-foreground placeholder:text-white/20 focus:outline-none",
                                      }
                                      div { class: "flex items-center justify-between px-4 py-2 border-t border-white/[0.05]",
                                          div { class: "flex items-center gap-3",
                                              if !step_required {
                                                  button {
                                                      "type": "button",
                                                      onclick: move |_| {
                                                          let mut ans = mut_answers.write();
                                                          ans.retain(|a| a.step_id != step_id_for_skip);
                                                          ans.push(Answer {
                                                              step_id: step_id_for_skip.clone(),
                                                              value: "(skipped)".into(),
                                                              timestamp: chrono::Utc::now().to_rfc3339(),
                                                          });
                                                          *draft.write() = String::new();
                                                      },
                                                      class: "text-xs text-muted-foreground/50 hover:text-foreground",
                                                      "Skip"
                                                  }
                                              }
                                              span { class: "font-mono text-[10px] text-white/15", "{char_count_text}" }
                                          }
                                          div { class: "flex items-center gap-2",
                                              kbd { class: "hidden rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground/50 sm:inline", "Cmd+Enter" }
                                              button {
                                                  "type": "button",
                                                  onclick: move |_| {
                                                      let text = draft.read().clone();
                                                      if !text.trim().is_empty() {
                                                          let mut ans = mut_answers.write();
                                                          ans.retain(|a| a.step_id != step_id);
                                                          ans.push(Answer {
                                                              step_id: step_id.clone(),
                                                              value: text.trim().to_string(),
                                                              timestamp: chrono::Utc::now().to_rfc3339(),
                                                          });
                                                          *draft.write() = String::new();
                                                      }
                                                  },
                                                  disabled: is_draft_empty,
                                                  class: "rounded-md bg-primary px-3 py-1.5 text-xs font-semibold text-primary-foreground ring-1 ring-primary/40 transition-all disabled:opacity-25 hover:bg-primary/85",
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
}
