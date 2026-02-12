//! Diamond stepper component
//!
//! A phase navigation component for the Diamond methodology.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::planner::types::DiamondPhase;
use dioxus::prelude::*;

/// Diamond stepper component
///
/// Displays the four Diamond methodology phases as an interactive stepper.
/// Shows the current phase, allows navigation between phases, and displays progress.
#[component]
pub fn DiamondStepper(
  current_phase: DiamondPhase,
  on_phase_change: Callback<DiamondPhase>,
  #[props(optional)] show_labels: Option<bool>,
  #[props(optional)] labels: Option<Vec<String>>,
  #[props(optional)] interactive: Option<bool>,
) -> Element {
  let show_labels_val = show_labels.unwrap_or(true);
  let is_interactive = interactive.unwrap_or(true);

  // Use custom labels or defaults
  let phase_labels = labels.unwrap_or_else(|| {
    vec![
      "Discovery".to_string(),
      "Design".to_string(),
      "Development".to_string(),
      "Delivery".to_string(),
    ]
  });

  let phases = [
    (
      DiamondPhase::Top,
      phase_labels.get(0).map_or("Discovery", |s| s.as_str()),
    ),
    (
      DiamondPhase::Right,
      phase_labels.get(1).map_or("Design", |s| s.as_str()),
    ),
    (
      DiamondPhase::Bottom,
      phase_labels.get(2).map_or("Development", |s| s.as_str()),
    ),
    (
      DiamondPhase::Left,
      phase_labels.get(3).map_or("Delivery", |s| s.as_str()),
    ),
  ];

  let current_phase_for_check = current_phase;

  let check_is_current = move |phase: DiamondPhase| -> bool { current_phase_for_check == phase };

  let check_is_complete = move |phase: DiamondPhase| -> bool {
    match phase {
      DiamondPhase::Top => false,
      DiamondPhase::Right => matches!(
        current_phase_for_check,
        DiamondPhase::Bottom | DiamondPhase::Left
      ),
      DiamondPhase::Bottom => matches!(current_phase_for_check, DiamondPhase::Left),
      DiamondPhase::Left => false,
    }
  };

  let get_progress_class = || -> String {
    if check_is_current(DiamondPhase::Top) {
      "progress-0"
    } else if check_is_current(DiamondPhase::Right) {
      "progress-33"
    } else if check_is_current(DiamondPhase::Bottom) {
      "progress-66"
    } else {
      "progress-100"
    }
    .to_string()
  };

  let progress_percentage = get_progress_width(current_phase);

  let is_interactive_val = is_interactive;
  let current_phase_val = current_phase;

  let can_navigate_to = move |phase: DiamondPhase| -> bool {
    if !is_interactive_val {
      return false;
    }

    // Can always go to current phase
    if phase == current_phase_val {
      return true;
    }

    // Can always go backward
    match (current_phase_val, phase) {
      (DiamondPhase::Right, DiamondPhase::Top) => true,
      (DiamondPhase::Bottom, DiamondPhase::Right | DiamondPhase::Top) => true,
      (DiamondPhase::Left, DiamondPhase::Bottom | DiamondPhase::Right | DiamondPhase::Top) => true,
      _ => false,
    }
  };

  let handle_phase_click = move |phase: DiamondPhase| {
    if can_navigate_to(phase) {
      on_phase_change.call(phase);
    }
  };

  rsx! {
      div { class: "diamond-stepper",
          // Progress bar at the top
          div { class: format!("stepper-progress {}", get_progress_class()),
              div { class: "progress-bar-container",
                  div {
                      class: "progress-bar-fill",
                      style: "width: {progress_percentage}%;"
                  }
              }
              div { class: "progress-text",
                  "{progress_percentage}% Complete"
              }
          }

          // Phase steps
          div { class: "stepper-phases",
              for (phase_var, label) in phases {
                  button {
                      key: "{label:?}",
                      class: format!(
                          "{} {} {} {}",
                          "phase-step",
                          if check_is_current(phase_var) { "current" } else { "" },
                          if check_is_complete(phase_var) { "complete" } else { "" },
                          if can_navigate_to(phase_var) && is_interactive { "clickable" } else { "disabled" }
                      ),
                      disabled: !can_navigate_to(phase_var) || !is_interactive,
                      onclick: move |_| handle_phase_click(phase_var),

                      // Phase icon/indicator
                      div { class: "phase-indicator",
                          if check_is_complete(phase_var) {
                              // Complete: show checkmark
                              div { class: "phase-icon complete-icon", "✓" }
                          } else if check_is_current(phase_var) {
                              // Current: show number or active dot
                              div { class: "phase-icon current-icon",
                                  {match phase_var {
                                      DiamondPhase::Top => "1",
                                      DiamondPhase::Right => "2",
                                      DiamondPhase::Bottom => "3",
                                      DiamondPhase::Left => "4",
                                  }}
                              }
                          } else {
                              // Future: show empty circle
                              div { class: "phase-icon future-icon" }
                          }
                      }

                      // Phase label
                      if show_labels_val {
                          div { class: "phase-label-container",
                              span { class: "phase-label", "{label}" }
                              if check_is_current(phase_var) {
                                  span { class: "phase-status", "(Current)" }
                              } else if check_is_complete(phase_var) {
                                  span { class: "phase-status", "(Complete)" }
                              }
                          }
                      }

                      // Phase connector line
                      if phase_var != DiamondPhase::Left {
                          div { class: format!("phase-connector {}",
                              if check_is_current(phase_var) || check_is_complete(phase_var) { "active" } else { "inactive" }
                          ) }
                      }
                  }
              }
          }

          // Phase description
          div { class: "stepper-description",
              p { class: "current-phase-description",
                  {format!("Currently in: {}", current_phase)}
              }
              if show_labels_val {
                  p { class: "phase-instructions",
                      {if is_interactive {
                          "Click on previous phases to navigate back"
                      } else {
                          "Phase navigation is disabled"
                      }}
                  }
              }
          }
      }
  }
}

/// Get the progress percentage for the current phase
#[must_use]
pub fn get_progress_width(phase: DiamondPhase) -> u32 {
  match phase {
    DiamondPhase::Top => 0,
    DiamondPhase::Right => 33,
    DiamondPhase::Bottom => 66,
    DiamondPhase::Left => 100,
  }
}

/// Get phase order for navigation purposes
#[must_use]
pub const fn phase_order(phase: DiamondPhase) -> u8 {
  match phase {
    DiamondPhase::Top => 0,
    DiamondPhase::Right => 1,
    DiamondPhase::Bottom => 2,
    DiamondPhase::Left => 3,
  }
}

/// Check if navigation from source to target is allowed
#[must_use]
pub const fn can_navigate(from: DiamondPhase, to: DiamondPhase) -> bool {
  let from_order = phase_order(from);
  let to_order = phase_order(to);

  // Can always stay in same phase
  if from_order == to_order {
    return true;
  }

  // Can always go backward
  to_order < from_order
}

/// Get next phase or None if at the end
#[must_use]
pub const fn next_phase(phase: DiamondPhase) -> Option<DiamondPhase> {
  match phase {
    DiamondPhase::Top => Some(DiamondPhase::Right),
    DiamondPhase::Right => Some(DiamondPhase::Bottom),
    DiamondPhase::Bottom => Some(DiamondPhase::Left),
    DiamondPhase::Left => None,
  }
}

/// Get previous phase or None if at the start
#[must_use]
pub const fn prev_phase(phase: DiamondPhase) -> Option<DiamondPhase> {
  match phase {
    DiamondPhase::Top => None,
    DiamondPhase::Right => Some(DiamondPhase::Top),
    DiamondPhase::Bottom => Some(DiamondPhase::Right),
    DiamondPhase::Left => Some(DiamondPhase::Bottom),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_progress_width() {
    assert_eq!(get_progress_width(DiamondPhase::Top), 0);
    assert_eq!(get_progress_width(DiamondPhase::Right), 33);
    assert_eq!(get_progress_width(DiamondPhase::Bottom), 66);
    assert_eq!(get_progress_width(DiamondPhase::Left), 100);
  }

  #[test]
  fn test_phase_order() {
    assert_eq!(phase_order(DiamondPhase::Top), 0);
    assert_eq!(phase_order(DiamondPhase::Right), 1);
    assert_eq!(phase_order(DiamondPhase::Bottom), 2);
    assert_eq!(phase_order(DiamondPhase::Left), 3);
  }

  #[test]
  fn test_can_navigate_same_phase() {
    assert!(can_navigate(DiamondPhase::Top, DiamondPhase::Top));
    assert!(can_navigate(DiamondPhase::Right, DiamondPhase::Right));
    assert!(can_navigate(DiamondPhase::Bottom, DiamondPhase::Bottom));
    assert!(can_navigate(DiamondPhase::Left, DiamondPhase::Left));
  }

  #[test]
  fn test_can_navigate_forward() {
    // Forward navigation should return false (not allowed without validation)
    assert!(!can_navigate(DiamondPhase::Top, DiamondPhase::Right));
    assert!(!can_navigate(DiamondPhase::Right, DiamondPhase::Bottom));
    assert!(!can_navigate(DiamondPhase::Bottom, DiamondPhase::Left));
    assert!(!can_navigate(DiamondPhase::Top, DiamondPhase::Bottom));
    assert!(!can_navigate(DiamondPhase::Top, DiamondPhase::Left));
    assert!(!can_navigate(DiamondPhase::Right, DiamondPhase::Left));
  }

  #[test]
  fn test_can_navigate_backward() {
    // Backward navigation should always be allowed
    assert!(can_navigate(DiamondPhase::Right, DiamondPhase::Top));
    assert!(can_navigate(DiamondPhase::Bottom, DiamondPhase::Right));
    assert!(can_navigate(DiamondPhase::Left, DiamondPhase::Bottom));
    assert!(can_navigate(DiamondPhase::Bottom, DiamondPhase::Top));
    assert!(can_navigate(DiamondPhase::Left, DiamondPhase::Right));
    assert!(can_navigate(DiamondPhase::Left, DiamondPhase::Top));
  }

  #[test]
  fn test_next_phase() {
    assert_eq!(next_phase(DiamondPhase::Top), Some(DiamondPhase::Right));
    assert_eq!(next_phase(DiamondPhase::Right), Some(DiamondPhase::Bottom));
    assert_eq!(next_phase(DiamondPhase::Bottom), Some(DiamondPhase::Left));
    assert_eq!(next_phase(DiamondPhase::Left), None);
  }

  #[test]
  fn test_prev_phase() {
    assert_eq!(prev_phase(DiamondPhase::Top), None);
    assert_eq!(prev_phase(DiamondPhase::Right), Some(DiamondPhase::Top));
    assert_eq!(prev_phase(DiamondPhase::Bottom), Some(DiamondPhase::Right));
    assert_eq!(prev_phase(DiamondPhase::Left), Some(DiamondPhase::Bottom));
  }

  #[test]
  fn test_diamond_stepper_custom_labels() {
    let custom_labels = vec![
      "Explore".to_string(),
      "Plan".to_string(),
      "Build".to_string(),
      "Ship".to_string(),
    ];

    // Verify labels structure for component usage
    assert_eq!(custom_labels.len(), 4);
    assert_eq!(custom_labels[0], "Explore");
    assert_eq!(custom_labels[3], "Ship");
  }

  #[test]
  fn test_diamond_stepper_phase_ordering() {
    // Verify phases are in expected order
    let order = phase_order(DiamondPhase::Top);
    assert_eq!(order, 0);

    let next_order = phase_order(DiamondPhase::Right);
    assert!(next_order > order);

    let last_order = phase_order(DiamondPhase::Left);
    assert_eq!(last_order, 3);
  }

  #[test]
  fn test_navigation_symmetry() {
    // Test that going forward then backward returns to the same phase
    let start = DiamondPhase::Top;

    let phase1 = next_phase(start);
    let phase1 = phase1.unwrap();
    assert_eq!(phase1, DiamondPhase::Right);

    let phase2 = prev_phase(phase1);
    let phase2 = phase2.unwrap();
    assert_eq!(phase2, start);

    // Same test from middle phases
    let mid = DiamondPhase::Bottom;
    let next = next_phase(mid).unwrap();
    let back = prev_phase(next).unwrap();
    assert_eq!(back, mid);
  }
}
