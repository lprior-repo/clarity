#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The Four Brutal Truths that every plan must pass before compilation.
///
/// These truths act as a final quality gate in the Preview phase,
/// ensuring that the plan is robust enough to proceed to KIRK compilation.
/// Each truth represents a critical aspect of plan viability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrutalTruth {
  /// Scale: Will this survive 10,000 users?
  /// Tests whether the solution can handle real-world load and complexity.
  Scale,

  /// Back-loaded Value: Are checkbox features included?
  /// Ensures that essential features are not pushed to "later" indefinitely.
  BackLoadedValue,

  /// VORP: Is this better than the current workaround?
  /// Value Over Replacement Product - tests if the solution is meaningfully
  /// better than what users currently do.
  Vorp,

  /// Sustaining: Will this break in a month?
  /// Tests long-term viability and maintenance considerations.
  Sustaining,
}

impl BrutalTruth {
  /// Get all four brutal truths as a slice.
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[
      Self::Scale,
      Self::BackLoadedValue,
      Self::Vorp,
      Self::Sustaining,
    ]
  }

  /// Get the display label for this truth.
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::Scale => "Scale",
      Self::BackLoadedValue => "Back-loaded Value",
      Self::Vorp => "VORP",
      Self::Sustaining => "Sustaining",
    }
  }

  /// Get the checkbox label (question format) for this truth.
  #[must_use]
  pub const fn checkbox_label(&self) -> &'static str {
    match self {
      Self::Scale => "Survives 10,000 users",
      Self::BackLoadedValue => "Checkbox features included",
      Self::Vorp => "Better than workaround",
      Self::Sustaining => "Won't break in a month",
    }
  }

  /// Get a detailed description of what this truth validates.
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Scale => {
        "Tests whether the solution can handle real-world load, \
                 complexity, and edge cases at scale."
      }
      Self::BackLoadedValue => {
        "Ensures that essential features are not indefinitely \
                 postponed to a 'later' that never comes."
      }
      Self::Vorp => {
        "Value Over Replacement Product: Is this meaningfully better \
                 than what users currently do to solve the problem?"
      }
      Self::Sustaining => {
        "Tests long-term viability: Will this solution remain \
                 functional and maintainable over time?"
      }
    }
  }

  /// Get help text that explains what to consider for this truth.
  #[must_use]
  pub const fn help_text(&self) -> &'static str {
    match self {
      Self::Scale => {
        "Consider: What happens when 10,000 users hit this simultaneously? \
                 Where are the bottlenecks? What breaks first?"
      }
      Self::BackLoadedValue => {
        "Consider: Which features are 'nice to have' vs 'essential'? \
                 Are you pushing critical functionality to phase 2+?"
      }
      Self::Vorp => {
        "Consider: What is the current workaround? Why would users \
                 switch from it? Is the improvement significant enough?"
      }
      Self::Sustaining => {
        "Consider: What happens when dependencies update? Who maintains \
                 this? What could cause it to stop working?"
      }
    }
  }

  /// Get the ordinal position (0-3) of this truth.
  #[must_use]
  pub const fn ordinal(&self) -> usize {
    match self {
      Self::Scale => 0,
      Self::BackLoadedValue => 1,
      Self::Vorp => 2,
      Self::Sustaining => 3,
    }
  }

  /// Get the total count of brutal truths (always 4).
  #[must_use]
  pub const fn count() -> usize {
    4
  }
}

impl fmt::Display for BrutalTruth {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.label())
  }
}

/// State of validation for the Four Brutal Truths.
///
/// Tracks which truths have been validated (checked) by the user.
/// All four must be checked before proceeding to KIRK compilation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrutalTruthsState {
  /// Validation flags for each truth, indexed by `BrutalTruth::ordinal()`.
  pub checked: [bool; BrutalTruth::count()],
}

impl BrutalTruthsState {
  /// Create a new state with all truths unchecked.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Create a state with all truths checked.
  #[must_use]
  pub const fn all_checked() -> Self {
    Self {
      checked: [true; BrutalTruth::count()],
    }
  }

  /// Whether Scale truth has been validated.
  #[must_use]
  pub const fn scale(&self) -> bool {
    self.checked[BrutalTruth::Scale.ordinal()]
  }

  /// Whether Back-loaded Value truth has been validated.
  #[must_use]
  pub const fn back_loaded_value(&self) -> bool {
    self.checked[BrutalTruth::BackLoadedValue.ordinal()]
  }

  /// Whether VORP truth has been validated.
  #[must_use]
  pub const fn vorp(&self) -> bool {
    self.checked[BrutalTruth::Vorp.ordinal()]
  }

  /// Whether Sustaining truth has been validated.
  #[must_use]
  pub const fn sustaining(&self) -> bool {
    self.checked[BrutalTruth::Sustaining.ordinal()]
  }

  /// Check if a specific truth has been validated.
  #[must_use]
  pub const fn is_checked(&self, truth: BrutalTruth) -> bool {
    self.checked[truth.ordinal()]
  }

  /// Set the validation state for a specific truth.
  #[must_use]
  pub const fn set_checked(self, truth: BrutalTruth, checked: bool) -> Self {
    let mut next = self;
    next.checked[truth.ordinal()] = checked;
    next
  }

  /// Toggle the validation state for a specific truth.
  #[must_use]
  pub const fn toggle(self, truth: BrutalTruth) -> Self {
    let checked = !self.is_checked(truth);
    self.set_checked(truth, checked)
  }

  /// Check if all truths have been validated.
  /// This is the gate condition for proceeding to KIRK compilation.
  #[must_use]
  pub const fn is_complete(&self) -> bool {
    self.checked[0] && self.checked[1] && self.checked[2] && self.checked[3]
  }

  /// Get the count of validated truths (0-4).
  #[must_use]
  pub fn checked_count(&self) -> usize {
    self.checked.iter().map(|&flag| usize::from(flag)).sum()
  }

  /// Get the count of unvalidated truths (0-4).
  #[must_use]
  pub fn unchecked_count(&self) -> usize {
    BrutalTruth::count() - self.checked_count()
  }

  /// Get a list of truths that have not been validated yet.
  #[must_use]
  pub fn unchecked_truths(&self) -> Vec<BrutalTruth> {
    BrutalTruth::all()
      .iter()
      .filter(|&&truth| !self.is_checked(truth))
      .copied()
      .collect()
  }

  /// Get completion percentage (0-100).
  #[must_use]
  pub fn completion_percentage(&self) -> u8 {
    let count = self.checked_count();
    match count {
      0 => 0,
      1 => 25,
      2 => 50,
      3 => 75,
      _ => 100,
    }
  }

  /// Convert to an array of bool values in order.
  #[must_use]
  pub const fn to_array(&self) -> [bool; 4] {
    self.checked
  }

  /// Create from an array of bool values.
  #[must_use]
  pub const fn from_array(arr: [bool; 4]) -> Self {
    Self { checked: arr }
  }
}

/// Props for `BrutalTruthsChecklist` component.
#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct BrutalTruthsChecklistProps {
  /// Signal containing the validation state for each truth.
  pub checked: Signal<BrutalTruthsState>,
  /// Whether the checkboxes are interactive.
  #[props(default = true)]
  pub enabled: bool,
  /// Whether to show help text for each truth.
  #[props(default = false)]
  pub show_help: bool,
  /// Whether to show the completion status.
  #[props(default = true)]
  pub show_status: bool,
}

/// `BrutalTruthsChecklist` component
///
/// Displays the Four Brutal Truths as interactive checkboxes.
/// Each truth must be validated before proceeding to KIRK compilation.
///
/// # Example
///
/// ```rust,ignore
/// let checked = use_signal(|| BrutalTruthsState::new());
///
/// rsx! {
///     BrutalTruthsChecklist {
///         checked: checked,
///         show_help: true,
///     }
/// }
/// ```
#[component]
pub fn BrutalTruthsChecklist(props: BrutalTruthsChecklistProps) -> Element {
  let mut expanded_help = use_signal(|| None::<BrutalTruth>);

  let toggle_truth = {
    let mut checked = props.checked;
    move |truth: BrutalTruth| {
      if props.enabled {
        let current = checked.read().clone();
        *checked.write() = current.toggle(truth);
      }
    }
  };

  let toggle_help = {
    move |truth: BrutalTruth| {
      let current = *expanded_help.read();
      *expanded_help.write() = if current == Some(truth) {
        None
      } else {
        Some(truth)
      };
    }
  };

  let state = props.checked.read();
  let is_complete = state.is_complete();
  let checked_count = state.checked_count();
  let completion_pct = state.completion_percentage();

  rsx! {
      div {
          class: "space-y-4",

          // Header
          div {
              class: "flex items-center justify-between",
              h3 {
                  class: "text-sm font-semibold text-foreground",
                  "Four Brutal Truths Check"
              }
              if props.show_status {
                  span {
                      class: if is_complete {
                          "text-xs font-medium text-green-600 dark:text-green-400"
                      } else {
                          "text-xs font-medium text-muted-foreground"
                      },
                      "{checked_count}/4 validated"
                  }
              }
          }

          // Progress bar
          if props.show_status {
              div {
                  class: "h-1.5 w-full rounded-full bg-muted",
                  div {
                      class: if is_complete {
                          "h-full rounded-full bg-green-500 transition-all duration-300"
                      } else {
                          "h-full rounded-full bg-primary transition-all duration-300"
                      },
                      style: "width: {completion_pct}%",
                  }
              }
          }

          // Checklist items
          div {
              class: "space-y-2",

              for truth in BrutalTruth::all() {
                  BrutalTruthItem {
                      truth: *truth,
                      checked: props.checked,
                      enabled: props.enabled,
                      show_help: props.show_help,
                      expanded_help: expanded_help,
                      on_toggle: {
                          let mut toggle_truth = toggle_truth;
                          move |t| toggle_truth(t)
                      },
                      on_toggle_help: {
                          let mut toggle_help = toggle_help;
                          move |t| toggle_help(t)
                      },
                  }
              }
          }

          // Completion status message
          if props.show_status {
              div {
                  class: "mt-3 flex items-center gap-2",

                  if is_complete {
                      svg {
                          xmlns: "http://www.w3.org/2000/svg",
                          width: "16",
                          height: "16",
                          view_box: "0 0 24 24",
                          fill: "none",
                          stroke: "currentColor",
                          stroke_width: "2",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          class: "text-green-500",
                          path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
                          path { d: "M20 6 9 17l-5-5" }
                      }
                      span {
                          class: "text-xs font-medium text-green-600 dark:text-green-400",
                          "All truths validated - ready to proceed"
                      }
                  } else {
                      svg {
                          xmlns: "http://www.w3.org/2000/svg",
                          width: "16",
                          height: "16",
                          view_box: "0 0 24 24",
                          fill: "none",
                          stroke: "currentColor",
                          stroke_width: "2",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          class: "text-amber-500",
                          circle { cx: "12", cy: "12", r: "10" }
                          path { d: "M12 16v-4" }
                          path { d: "M12 8h.01" }
                      }
                      span {
                          class: "text-xs text-muted-foreground",
                          "Validate all truths before proceeding"
                      }
                  }
              }
          }
      }
  }
}

/// Props for `BrutalTruthItem` component (internal).
#[derive(Clone, Debug, PartialEq, Props)]
pub struct BrutalTruthItemProps {
  /// The truth to display.
  pub truth: BrutalTruth,
  /// Signal containing the validation state.
  pub checked: Signal<BrutalTruthsState>,
  /// Whether the checkbox is interactive.
  pub enabled: bool,
  /// Whether to show help button.
  pub show_help: bool,
  /// Signal for which help is expanded.
  pub expanded_help: Signal<Option<BrutalTruth>>,
  /// Callback when toggling the truth.
  pub on_toggle: EventHandler<BrutalTruth>,
  /// Callback when toggling help.
  pub on_toggle_help: EventHandler<BrutalTruth>,
}

/// Internal component for a single brutal truth item.
#[component]
pub fn BrutalTruthItem(props: BrutalTruthItemProps) -> Element {
  let truth = props.truth;
  let is_checked = props.checked.read().is_checked(truth);
  let is_expanded = *props.expanded_help.read() == Some(truth);

  rsx! {
      div {
          class: "rounded-md border border-border/50 bg-background/50 p-3 transition-colors hover:bg-muted/30",

          // Checkbox row
          div {
              class: "flex items-start gap-3",

              // Checkbox
              button {
                  class: if is_checked {
                      "flex h-5 w-5 shrink-0 items-center justify-center rounded border-2 border-green-500 bg-green-500 text-white transition-colors"
                  } else if props.enabled {
                      "flex h-5 w-5 shrink-0 items-center justify-center rounded border-2 border-border bg-background transition-colors hover:border-primary"
                  } else {
                      "flex h-5 w-5 shrink-0 items-center justify-center rounded border-2 border-border bg-background opacity-50"
                  },
                  disabled: !props.enabled,
                  onclick: {
                      let on_toggle = props.on_toggle;
                      move |_| on_toggle.call(truth)
                  },

                  if is_checked {
                      svg {
                          xmlns: "http://www.w3.org/2000/svg",
                          width: "12",
                          height: "12",
                          view_box: "0 0 24 24",
                          fill: "none",
                          stroke: "currentColor",
                          stroke_width: "3",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          path { d: "M20 6 9 17l-5-5" }
                      }
                  }
              }

              // Label and description
              div {
                  class: "flex-1",

                  div {
                      class: "flex items-center gap-2",
                      label {
                          class: if is_checked {
                              "text-sm font-medium text-foreground"
                          } else {
                              "text-sm font-medium text-foreground/80"
                          },
                          {truth.label()}
                      }
                      span {
                          class: "text-xs text-muted-foreground",
                          {truth.checkbox_label()}
                      }
                  }

                  // Description
                  p {
                      class: "mt-0.5 text-xs text-muted-foreground",
                      {truth.description()}
                  }
              }

              // Help toggle button
              if props.show_help {
                  button {
                      class: if is_expanded {
                          "rounded p-1 text-primary transition-colors hover:bg-muted"
                      } else {
                          "rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                      },
                      onclick: {
                          let on_toggle_help = props.on_toggle_help;
                          move |_| on_toggle_help.call(truth)
                      },
                      svg {
                          xmlns: "http://www.w3.org/2000/svg",
                          width: "14",
                          height: "14",
                          view_box: "0 0 24 24",
                          fill: "none",
                          stroke: "currentColor",
                          stroke_width: "2",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          circle { cx: "12", cy: "12", r: "10" }
                          path { d: "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" }
                          path { d: "M12 17h.01" }
                      }
                  }
              }
          }

          // Expanded help text
          if props.show_help && is_expanded {
              div {
                  class: "mt-2 rounded bg-muted/50 p-2",
                  p {
                      class: "text-xs text-muted-foreground italic",
                      {truth.help_text()}
                  }
              }
          }
      }
  }
}

/// Props for `BrutalTruthsSummary` component.
#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct BrutalTruthsSummaryProps {
  /// The validation state to summarize.
  pub state: BrutalTruthsState,
  /// Whether to show in compact mode.
  #[props(default = false)]
  pub compact: bool,
}

/// `BrutalTruthsSummary` component
///
/// Displays a compact summary of the Four Brutal Truths validation status.
/// Used in the Preview phase summary view.
#[component]
pub fn BrutalTruthsSummary(props: BrutalTruthsSummaryProps) -> Element {
  let state = &props.state;
  let is_complete = state.is_complete();

  if props.compact {
    rsx! {
        div {
            class: "flex items-center gap-2",
            for truth in BrutalTruth::all() {
                div {
                    class: if state.is_checked(*truth) {
                        "flex h-6 w-6 items-center justify-center rounded-full bg-green-100 text-green-600 dark:bg-green-900/30 dark:text-green-400"
                    } else {
                        "flex h-6 w-6 items-center justify-center rounded-full bg-muted text-muted-foreground"
                    },
                    if state.is_checked(*truth) {
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "12",
                            height: "12",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "3",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M20 6 9 17l-5-5" }
                        }
                    } else {
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "12",
                            height: "12",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M18 6 6 18" }
                            path { d: "m6 6 12 12" }
                        }
                    }
                }
            }
        }
    }
  } else {
    rsx! {
        div {
            class: "rounded-md border border-border/50 bg-card p-4",
            div {
                class: "flex items-center justify-between",
                h4 {
                    class: "text-sm font-medium text-foreground",
                    "Four Brutal Truths"
                }
                span {
                    class: if is_complete {
                        "rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700 dark:bg-green-900/30 dark:text-green-400"
                    } else {
                        "rounded-full bg-amber-100 px-2 py-0.5 text-xs font-medium text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
                    },
                    if is_complete { "Complete" } else { "Incomplete" }
                }
            }
            div {
                class: "mt-3 grid grid-cols-2 gap-2",
                for truth in BrutalTruth::all() {
                    div {
                        class: "flex items-center gap-2",
                        div {
                            class: if state.is_checked(*truth) {
                                "h-2 w-2 rounded-full bg-green-500"
                            } else {
                                "h-2 w-2 rounded-full bg-muted-foreground/30"
                            },
                        }
                        span {
                            class: "text-xs text-muted-foreground",
                            {truth.label()}
                        }
                    }
                }
            }
        }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // ========== BrutalTruth Tests ==========

  #[test]
  fn test_brutal_truth_all_returns_four() {
    let all = BrutalTruth::all();
    assert_eq!(all.len(), 4);
    assert!(all.contains(&BrutalTruth::Scale));
    assert!(all.contains(&BrutalTruth::BackLoadedValue));
    assert!(all.contains(&BrutalTruth::Vorp));
    assert!(all.contains(&BrutalTruth::Sustaining));
  }

  #[test]
  fn test_brutal_truth_labels() {
    assert_eq!(BrutalTruth::Scale.label(), "Scale");
    assert_eq!(BrutalTruth::BackLoadedValue.label(), "Back-loaded Value");
    assert_eq!(BrutalTruth::Vorp.label(), "VORP");
    assert_eq!(BrutalTruth::Sustaining.label(), "Sustaining");
  }

  #[test]
  fn test_brutal_truth_checkbox_labels() {
    assert_eq!(BrutalTruth::Scale.checkbox_label(), "Survives 10,000 users");
    assert_eq!(
      BrutalTruth::BackLoadedValue.checkbox_label(),
      "Checkbox features included"
    );
    assert_eq!(BrutalTruth::Vorp.checkbox_label(), "Better than workaround");
    assert_eq!(
      BrutalTruth::Sustaining.checkbox_label(),
      "Won't break in a month"
    );
  }

  #[test]
  fn test_brutal_truth_descriptions_not_empty() {
    for truth in BrutalTruth::all() {
      assert!(
        !truth.description().is_empty(),
        "Description should not be empty for {truth:?}"
      );
    }
  }

  #[test]
  fn test_brutal_truth_help_text_not_empty() {
    for truth in BrutalTruth::all() {
      assert!(
        !truth.help_text().is_empty(),
        "Help text should not be empty for {truth:?}"
      );
    }
  }

  #[test]
  fn test_brutal_truth_ordinal() {
    assert_eq!(BrutalTruth::Scale.ordinal(), 0);
    assert_eq!(BrutalTruth::BackLoadedValue.ordinal(), 1);
    assert_eq!(BrutalTruth::Vorp.ordinal(), 2);
    assert_eq!(BrutalTruth::Sustaining.ordinal(), 3);
  }

  #[test]
  fn test_brutal_truth_count() {
    assert_eq!(BrutalTruth::count(), 4);
  }

  #[test]
  fn test_brutal_truth_display() {
    assert_eq!(format!("{}", BrutalTruth::Scale), "Scale");
    assert_eq!(format!("{}", BrutalTruth::Vorp), "VORP");
  }

  #[test]
  fn test_brutal_truth_serialization() {
    let truth = BrutalTruth::Vorp;
    let json_result = serde_json::to_string(&truth);
    assert!(json_result.is_ok());

    let parsed = json_result
      .ok()
      .and_then(|json| serde_json::from_str::<BrutalTruth>(&json).ok());
    assert_eq!(parsed, Some(BrutalTruth::Vorp));
  }

  // ========== BrutalTruthsState Tests ==========

  #[test]
  fn test_state_new_all_unchecked() {
    let state = BrutalTruthsState::new();
    assert!(!state.scale());
    assert!(!state.back_loaded_value());
    assert!(!state.vorp());
    assert!(!state.sustaining());
    assert!(!state.is_complete());
    assert_eq!(state.checked_count(), 0);
  }

  #[test]
  fn test_state_all_checked() {
    let state = BrutalTruthsState::all_checked();
    assert!(state.scale());
    assert!(state.back_loaded_value());
    assert!(state.vorp());
    assert!(state.sustaining());
    assert!(state.is_complete());
    assert_eq!(state.checked_count(), 4);
  }

  #[test]
  fn test_state_is_checked() {
    let state = BrutalTruthsState::from_array([true, false, true, false]);

    assert!(state.is_checked(BrutalTruth::Scale));
    assert!(!state.is_checked(BrutalTruth::BackLoadedValue));
    assert!(state.is_checked(BrutalTruth::Vorp));
    assert!(!state.is_checked(BrutalTruth::Sustaining));
  }

  #[test]
  fn test_state_set_checked() {
    let state = BrutalTruthsState::new();

    let state = state.set_checked(BrutalTruth::Scale, true);
    assert!(state.scale());
    assert!(!state.back_loaded_value());

    let state = state.set_checked(BrutalTruth::Vorp, true);
    assert!(state.vorp());
  }

  #[test]
  fn test_state_toggle() {
    let state = BrutalTruthsState::new();
    assert!(!state.scale());

    let state = state.toggle(BrutalTruth::Scale);
    assert!(state.scale());

    let state = state.toggle(BrutalTruth::Scale);
    assert!(!state.scale());
  }

  #[test]
  fn test_state_is_complete() {
    let mut state = BrutalTruthsState::new();
    assert!(!state.is_complete());

    state = state.set_checked(BrutalTruth::Scale, true);
    assert!(!state.is_complete());

    state = state.set_checked(BrutalTruth::BackLoadedValue, true);
    assert!(!state.is_complete());

    state = state.set_checked(BrutalTruth::Vorp, true);
    assert!(!state.is_complete());

    state = state.set_checked(BrutalTruth::Sustaining, true);
    assert!(state.is_complete());
  }

  #[test]
  fn test_state_checked_count() {
    let state = BrutalTruthsState::new();
    assert_eq!(state.checked_count(), 0);

    let state = state.set_checked(BrutalTruth::Scale, true);
    assert_eq!(state.checked_count(), 1);

    let state = state.set_checked(BrutalTruth::Vorp, true);
    assert_eq!(state.checked_count(), 2);
  }

  #[test]
  fn test_state_unchecked_count() {
    let state = BrutalTruthsState::all_checked();
    assert_eq!(state.unchecked_count(), 0);

    let state = BrutalTruthsState::new();
    assert_eq!(state.unchecked_count(), 4);

    let state = state.set_checked(BrutalTruth::Scale, true);
    assert_eq!(state.unchecked_count(), 3);
  }

  #[test]
  fn test_state_unchecked_truths() {
    let state = BrutalTruthsState::new();
    let unchecked = state.unchecked_truths();
    assert_eq!(unchecked.len(), 4);

    let state = state.set_checked(BrutalTruth::Scale, true);
    let unchecked = state.unchecked_truths();
    assert_eq!(unchecked.len(), 3);
    assert!(!unchecked.contains(&BrutalTruth::Scale));
  }

  #[test]
  fn test_state_completion_percentage() {
    let state = BrutalTruthsState::new();
    assert_eq!(state.completion_percentage(), 0);

    let state = state.set_checked(BrutalTruth::Scale, true);
    assert_eq!(state.completion_percentage(), 25);

    let state = state.set_checked(BrutalTruth::BackLoadedValue, true);
    assert_eq!(state.completion_percentage(), 50);

    let state = state.set_checked(BrutalTruth::Vorp, true);
    assert_eq!(state.completion_percentage(), 75);

    let state = state.set_checked(BrutalTruth::Sustaining, true);
    assert_eq!(state.completion_percentage(), 100);
  }

  #[test]
  fn test_state_to_array() {
    let state = BrutalTruthsState::from_array([true, false, true, false]);

    let arr = state.to_array();
    assert_eq!(arr, [true, false, true, false]);
  }

  #[test]
  fn test_state_from_array() {
    let arr = [false, true, false, true];
    let state = BrutalTruthsState::from_array(arr);

    assert!(!state.scale());
    assert!(state.back_loaded_value());
    assert!(!state.vorp());
    assert!(state.sustaining());
  }

  #[test]
  fn test_state_roundtrip_array() {
    let original = BrutalTruthsState::from_array([true, false, true, true]);

    let arr = original.to_array();
    let restored = BrutalTruthsState::from_array(arr);

    assert_eq!(original, restored);
  }

  #[test]
  fn test_state_serialization() {
    let state = BrutalTruthsState::from_array([true, false, true, false]);

    let json_result = serde_json::to_string(&state);
    assert!(json_result.is_ok());

    let parsed = json_result
      .ok()
      .and_then(|json| serde_json::from_str::<BrutalTruthsState>(&json).ok());
    assert_eq!(parsed, Some(state));
  }

  #[test]
  fn test_state_default() {
    let state = BrutalTruthsState::default();
    assert!(!state.scale());
    assert!(!state.back_loaded_value());
    assert!(!state.vorp());
    assert!(!state.sustaining());
  }

  #[test]
  fn test_state_clone() {
    let state = BrutalTruthsState::all_checked();
    let cloned = state.clone();
    assert_eq!(state, cloned);
  }

  #[test]
  fn test_state_equality() {
    let a = BrutalTruthsState::all_checked();
    let b = BrutalTruthsState::all_checked();
    let c = BrutalTruthsState::new();

    assert_eq!(a, b);
    assert_ne!(a, c);
  }
}
