#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::lattice::ears::{parse_requirements, EarsOutput, EarsRequirement};
use crate::types::{prompt_steps, Answer};

/// Section header component
#[component]
fn SectionHeader(label: String, count: Option<usize>) -> Element {
  rsx! {
      div { class: "flex items-center gap-2 pb-2 pt-5 first:pt-0",
          h4 {
              class: "text-xs font-semibold uppercase tracking-widest text-muted-foreground/70",
              "{label}"
          }
          if let Some(n) = count {
              span {
                  class: "rounded-full bg-secondary px-1.5 py-0.5 text-xs tabular-nums text-muted-foreground",
                  "{n}"
              }
          }
      }
  }
}

/// Thesis card component
#[component]
fn ThesisCard(label: String, value: Option<String>, accent_class: String) -> Element {
  let Some(content) = value else {
    return rsx! { "" };
  };

  rsx! {
      div {
          class: format!("animate-fade-up rounded-lg border px-3 py-2.5 {}", accent_class),
          span {
              class: "mb-1 block text-xs font-medium uppercase tracking-wider text-muted-foreground",
              "{label}"
          }
          p {
              class: "text-sm leading-relaxed text-foreground",
              "{content}"
          }
      }
  }
}

/// Use case row component
#[component]
fn UseCaseRow(text: String, index: usize) -> Element {
  rsx! {
      div {
          class: "animate-fade-up flex items-start gap-2.5 rounded-md px-2 py-2 transition-colors hover:bg-secondary/50",
          span {
              class: "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded bg-secondary font-mono text-xs text-muted-foreground",
              "{index + 1}"
          }
          p { class: "min-w-0 text-sm leading-relaxed text-foreground", "{text}" }
      }
  }
}

/// Task row component
#[component]
fn TaskRow(text: String, index: usize, selected: bool, on_click: EventHandler<()>) -> Element {
  let parts: Vec<&str> = text.splitn(2, ':').collect();
  let module = if parts.len() > 1 {
    Some(parts[0].trim().to_string())
  } else {
    None
  };
  let title = if parts.len() > 1 {
    parts[1].trim().to_string()
  } else {
    text.clone()
  };

  rsx! {
      button {
          "type": "button",
          onclick: move |_| on_click(()),
          class: format!(
              "animate-fade-up flex w-full items-start gap-2.5 rounded-md px-2 py-2 text-left transition-all {}",
              if selected { "bg-primary/10 ring-1 ring-primary/30" } else { "hover:bg-secondary/50" }
          ),
          span {
              class: format!(
                  "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded font-mono text-xs transition-colors {}",
                  if selected { "bg-primary text-primary-foreground" } else { "bg-secondary text-muted-foreground" }
              ),
              "{index + 1}"
          }
          div { class: "min-w-0 flex-1",
              div { class: "flex items-center gap-1.5",
                  if let Some(mod_name) = &module {
                      span {
                          class: "rounded bg-chart-5/15 px-1.5 py-0.5 font-mono text-xs text-chart-5",
                          "{mod_name}"
                      }
                  }
              }
              p { class: "mt-0.5 text-sm text-foreground", "{title}" }
          }
      }
  }
}

/// Task detail component
#[component]
fn TaskDetail(task: String, index: usize, on_close: EventHandler<()>) -> Element {
  let parts: Vec<&str> = task.splitn(2, ':').collect();
  let module = if parts.len() > 1 {
    Some(parts[0].trim().to_string())
  } else {
    None
  };
  let title = if parts.len() > 1 {
    parts[1].trim().to_string()
  } else {
    task.clone()
  };

  rsx! {
      div { class: "animate-fade-up rounded-lg border border-primary/20 bg-primary/5",
          div {
              class: "flex items-center justify-between border-b border-primary/10 px-3 py-2",
              div { class: "flex items-center gap-2",
                  span {
                      class: "flex h-5 w-5 items-center justify-center rounded bg-primary font-mono text-xs text-primary-foreground",
                      "{index + 1}"
                  }
                  if let Some(mod_name) = &module {
                      span {
                          class: "rounded bg-chart-5/15 px-1.5 py-0.5 font-mono text-xs text-chart-5",
                          "{mod_name}"
                      }
                  }
                  span { class: "text-sm font-medium text-foreground", "{title}" }
              }
              button {
                  "type": "button",
                  onclick: move |_| on_close(()),
                  class: "rounded p-0.5 text-muted-foreground hover:text-foreground",
                  "x"
              }
          }
          div { class: "space-y-3 px-3 py-3",
              div {
                  span { class: "mb-1 block text-xs font-medium uppercase tracking-wider text-muted-foreground", "Acceptance Criteria" }
                  div { class: "rounded border border-dashed border-border/50 px-3 py-3 text-center text-xs text-muted-foreground/40", "Add criteria in the Develop phase" }
              }
          }
      }
  }
}

/// Parse lines from text
fn parse_lines(text: Option<String>) -> Vec<String> {
  text
    .map(|t| {
      t.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
    })
    .unwrap_or_default()
}

/// Get answer value by step ID
fn get_val(answers: &[Answer], id: &str) -> Option<String> {
  answers
    .iter()
    .find(|a| a.step_id == id && a.value != "(skipped)")
    .map(|a| a.value.clone())
}

/// Artifact panel data for rendering
struct ArtifactData {
  problem: Option<String>,
  antithesis: Option<String>,
  solution: Option<String>,
  persona: Option<String>,
  scenario: Option<String>,
  use_cases: Vec<String>,
  constraints: Option<String>,
  tasks: Vec<String>,
  progress: usize,
  has_anything: bool,
  ears_output: EarsOutput,
}

/// EARS requirement display data
#[derive(Clone, Debug, PartialEq)]
struct EarsDisplayData {
  requirement_type: String,
  text: String,
  color_class: String,
}

/// Build EARS display data from parsed requirements
fn build_ears_display(output: &EarsOutput) -> Vec<EarsDisplayData> {
  output
    .requirements
    .iter()
    .map(|req| {
      let (requirement_type, text, color_class) = match req {
        EarsRequirement::Ubiquitous { actor, action } => (
          "Ubiquitous".to_string(),
          format!("{actor} shall {action}"),
          "border-chart-1/20 bg-chart-1/5 text-chart-1".to_string(),
        ),
        EarsRequirement::StateDriven {
          actor,
          trigger,
          action,
        } => (
          "State-Driven".to_string(),
          format!("When {trigger}, {actor} shall {action}"),
          "border-chart-2/20 bg-chart-2/5 text-chart-2".to_string(),
        ),
        EarsRequirement::EventDriven {
          actor,
          trigger,
          action,
        } => (
          "Event-Driven".to_string(),
          format!("During {trigger}, {actor} shall {action}"),
          "border-chart-3/20 bg-chart-3/5 text-chart-3".to_string(),
        ),
        EarsRequirement::Unwanted {
          actor,
          condition,
          action,
        } => (
          "Unwanted".to_string(),
          format!("If {condition}, {actor} shall NOT {action}"),
          "border-chart-4/20 bg-chart-4/5 text-chart-4".to_string(),
        ),
        EarsRequirement::Optional {
          actor,
          condition,
          action,
        } => (
          "Optional".to_string(),
          format!("Where {condition}, {actor} shall {action}"),
          "border-chart-5/20 bg-chart-5/5 text-chart-5".to_string(),
        ),
      };
      EarsDisplayData {
        requirement_type,
        text,
        color_class,
      }
    })
    .collect()
}

/// Parse requirements from answers
fn parse_requirements_from_answers(answers: &[Answer]) -> EarsOutput {
  // Collect all answer values that might contain requirements
  let requirement_text: String = answers
    .iter()
    .filter(|a| {
      // Only include answers that look like requirements
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
    EarsOutput::new()
  } else {
    parse_requirements(&requirement_text)
  }
}

/// Build artifact data from answers
fn build_artifact_data(answers: &[Answer]) -> ArtifactData {
  let problem = get_val(answers, "problem");
  let antithesis = get_val(answers, "antithesis");
  let solution = get_val(answers, "solution");
  let persona = get_val(answers, "persona");
  let scenario = get_val(answers, "scenario");
  let use_cases = parse_lines(get_val(answers, "use-cases"));
  let constraints = get_val(answers, "constraints");
  let tasks = parse_lines(get_val(answers, "tasks"));

  let required = prompt_steps().iter().filter(|s| s.required).count();
  let done = answers
    .iter()
    .filter(|a| {
      prompt_steps()
        .iter()
        .any(|s| s.id == a.step_id && s.required && a.value != "(skipped)")
    })
    .count();

  let progress = done
    .saturating_mul(100)
    .checked_div(required)
    .map_or(0, |v| v)
    .min(100);

  let ears_output = parse_requirements_from_answers(answers);

  ArtifactData {
    problem,
    antithesis,
    solution,
    persona,
    scenario,
    use_cases,
    constraints,
    tasks,
    progress,
    has_anything: !answers.is_empty(),
    ears_output,
  }
}

/// Render use case rows
fn render_use_case_rows(use_cases: &[String]) -> Vec<Element> {
  use_cases
    .iter()
    .enumerate()
    .map(|(i, uc)| {
      let uc = uc.clone();
      rsx! { UseCaseRow { text: uc, index: i } }
    })
    .collect()
}

/// EARS requirement card component
#[component]
fn EarsRequirementCard(data: EarsDisplayData, index: usize) -> Element {
  rsx! {
      div {
          class: format!("animate-fade-up rounded-lg border px-3 py-2.5 {}", data.color_class),
          div { class: "flex items-start justify-between gap-2",
              div { class: "min-w-0 flex-1",
                  span {
                      class: "mb-1 block text-xs font-medium uppercase tracking-wider text-muted-foreground/70",
                      "{data.requirement_type}"
                  }
                  p {
                      class: "text-sm leading-relaxed text-foreground",
                      "{data.text}"
                  }
              }
              span {
                  class: "shrink-0 flex h-5 w-5 items-center justify-center rounded bg-secondary/50 font-mono text-xs text-muted-foreground",
                  "{index + 1}"
              }
          }
      }
  }
}

/// Render EARS requirements section
fn render_ears_section(ears_output: &EarsOutput) -> Option<Element> {
  if ears_output.requirements.is_empty() {
    return None;
  }

  let display_data = build_ears_display(ears_output);
  let elements: Vec<Element> = display_data
    .iter()
    .enumerate()
    .map(|(i, data)| rsx! { EarsRequirementCard { data: data.clone(), index: i } })
    .collect();

  let error_count = ears_output.errors.len();
  let error_element = if error_count > 0 {
    Some(rsx! {
        div {
            class: "mt-2 rounded-md border border-chart-4/30 bg-chart-4/5 px-3 py-2",
            div { class: "flex items-center gap-2",
                svg {
                    width: "14",
                    height: "14",
                    view_box: "0 0 14 14",
                    fill: "none",
                    class: "text-chart-4 shrink-0",
                    path {
                        d: "M7 1C3.7 1 1 3.7 1 7C1 10.3 3.7 13 7 13C10.3 13 13 10.3 13 7C13 3.7 10.3 1 7 1ZM7 10C6.4 10 6 9.6 6 9C6 8.4 6.4 8 7 8C7.6 8 8 8.4 8 9C8 9.6 7.6 10 7 10ZM7 6.5C6.4 6.5 6 6.1 6 5.5V4C6 3.4 6.4 3 7 3C7.6 3 8 3.4 8 4V5.5C8 6.1 7.6 6.5 7 6.5Z",
                        fill: "currentColor"
                    }
                }
                span { class: "text-xs text-chart-4", "{error_count} requirement(s) could not be parsed" }
            }
        }
    })
  } else {
    None
  };

  Some(rsx! {
      div {
          SectionHeader { label: "Requirements (EARS)".to_string(), count: Some(ears_output.requirements.len()) }
          div { class: "space-y-2",
              for element in elements.iter() {
                  {element.clone()}
              }
              if let Some(err) = error_element {
                  {err}
              }
          }
      }
  })
}

/// `ArtifactPanel` component - displays accumulated planning artifacts
#[component]
pub fn ArtifactPanel(answers: Signal<Vec<Answer>>, active_phase: Signal<String>) -> Element {
  let mut selected_task = use_signal(|| None as Option<usize>);

  let data = build_artifact_data(&answers.read());
  let ArtifactData {
    problem,
    antithesis,
    solution,
    persona,
    scenario,
    use_cases,
    constraints,
    tasks,
    progress,
    has_anything,
    ears_output,
  } = data;

  let has_thesis = problem.is_some() || antithesis.is_some() || solution.is_some();
  let active_phase_str = active_phase.read();
  let empty_message = if *active_phase_str == "discover" {
    "Answer the coach to build your thesis, persona, and north star scenario."
  } else {
    "Your plan will build up here as you answer."
  };
  drop(active_phase_str);

  let use_case_elements = render_use_case_rows(&use_cases);

  let current_selected = *selected_task.read();
  let tasks_clone = tasks.clone();
  let task_elements: Vec<Element> = tasks
    .iter()
    .enumerate()
    .map(|(i, t)| {
      let t = t.clone();
      let selected = current_selected == Some(i);
      let mut signal = selected_task;
      rsx! {
          TaskRow {
              text: t,
              index: i,
              selected,
              on_click: move |()| {
                  let current = *signal.read();
                  signal.set(if current == Some(i) { None } else { Some(i) });
              }
          }
      }
    })
    .collect();

  rsx! {
      div { class: "flex h-full flex-col",
          // Progress bar
          div { class: "shrink-0 px-4 pt-3 pb-1",
              div { class: "flex items-center gap-2",
                  div { class: "h-1 flex-1 rounded-full bg-secondary",
                      div {
                          class: "h-full rounded-full bg-primary transition-all duration-700 ease-out",
                          style: format!("width: {}%", progress)
                      }
                  }
                  span { class: "font-mono text-xs text-muted-foreground", "{progress}%" }
              }
          }

          // Content
          div { class: "flex-1 overflow-y-auto px-4 py-2",
              if !has_anything {
                  div { class: "flex h-full items-center justify-center",
                      p {
                          class: "max-w-xs text-center text-sm leading-relaxed text-muted-foreground/40",
                          "{empty_message}"
                      }
                  }
              } else {
                  div { class: "space-y-1 pb-4",
                      // Thesis section
                      if has_thesis {
                          SectionHeader { label: "Thesis".to_string(), count: None }
                          div { class: "space-y-2",
                              ThesisCard { label: "Problem".to_string(), value: problem, accent_class: "border-border bg-card".to_string() }
                              ThesisCard { label: "Antithesis".to_string(), value: antithesis, accent_class: "border-chart-4/20 bg-chart-4/5".to_string() }
                              ThesisCard { label: "Solution".to_string(), value: solution, accent_class: "border-border bg-card".to_string() }
                          }
                      }

                      // Persona section
                      if let Some(p) = persona {
                          SectionHeader { label: "User".to_string(), count: None }
                          div { class: "animate-fade-up rounded-lg border border-chart-5/20 bg-chart-5/5 px-3 py-2.5",
                              p { class: "text-sm leading-relaxed text-foreground", "{p}" }
                          }
                      }

                      // Scenario section
                      if let Some(s) = scenario {
                          SectionHeader { label: "North Star".to_string(), count: None }
                          div { class: "animate-fade-up rounded-lg border border-chart-2/20 bg-chart-2/5 px-3 py-2.5",
                              p { class: "text-sm leading-relaxed text-foreground/80", "{s}" }
                          }
                      }

                      // Use cases section
                      if !use_cases.is_empty() {
                          SectionHeader { label: "Use Cases".to_string(), count: Some(use_cases.len()) }
                          div { class: "space-y-0.5",
                              for row in use_case_elements.iter() {
                                  {row.clone()}
                              }
                          }
                      }

                      // Constraints section
                      if let Some(c) = constraints {
                          SectionHeader { label: "Stack".to_string(), count: None }
                          div { class: "animate-fade-up rounded-lg border border-border bg-card px-3 py-2.5",
                              p { class: "font-mono text-xs leading-relaxed text-foreground/80", "{c}" }
                          }
                      }

                      // EARS Requirements section
                      if let Some(ears_section) = render_ears_section(&ears_output) {
                          {ears_section}
                      }

                      // Tasks section
                      if !tasks_clone.is_empty() {
                          SectionHeader { label: "Tasks".to_string(), count: Some(tasks_clone.len()) }
                          if let Some(selected) = *selected_task.read() {
                              if selected < tasks_clone.len() {
                                  div { class: "mb-2",
                                      TaskDetail {
                                          task: tasks_clone[selected].clone(),
                                          index: selected,
                                          on_close: move |()| selected_task.set(None)
                                      }
                                  }
                              }
                          }
                          div { class: "space-y-0.5",
                              for row in task_elements.iter() {
                                  {row.clone()}
                              }
                          }
                      }
                  }
              }
          }
      }
  }
}
