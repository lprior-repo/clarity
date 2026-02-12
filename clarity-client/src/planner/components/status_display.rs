//! Status display component
//!
//! Displays progress status with appropriate color coding and animations.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

use crate::planner::state::PlannerState;
use clarity_core::progress::ProgressStatus;
use clarity_core::status_colors;
use dioxus::prelude::*;

/// Status badge component
///
/// Displays a status with appropriate color coding and optional pulse animation.
#[component]
pub fn StatusBadge(status: ProgressStatus, size: StatusBadgeSize, show_pulse: bool) -> Element {
  let css_classes = status_colors::get_status_css_classes(status, show_pulse);
  let _hover_css_classes = status_colors::get_status_hover_css_classes(status);
  let border_css_classes = status_colors::get_status_border_css_classes(status);

  let size_classes = match size {
    StatusBadgeSize::Small => "text-xs px-2 py-1 rounded-full",
    StatusBadgeSize::Medium => "text-sm px-3 py-1.5 rounded-md",
    StatusBadgeSize::Large => "text-base px-4 py-2 rounded-lg",
  };

  rsx! {
    span {
      class: format!("inline-flex items-center gap-2 border {} {} {}", css_classes, border_css_classes, size_classes),
      onmouseenter: move |_| {
        // Hover effect handled by CSS classes
      },
      onclick: move |_| {
        // Click handler could trigger status change in the future
      },

      // Status icon
      span {
        class: "w-2 h-2 rounded-full",
        style: format!("background-color: var(--{});", get_color_from_status(status)),
      },

      // Status text
      span {
        class: "font-medium",
        "{format_status(status)}"
      }
    }
  }
}

/// Status indicator component
///
/// A simpler indicator that just shows a colored dot.
#[component]
pub fn StatusIndicator(status: ProgressStatus, show_pulse: bool) -> Element {
  let css_classes = status_colors::get_status_css_classes(status, show_pulse);

  rsx! {
    span {
      class: format!("inline-block w-3 h-3 rounded-full {}", css_classes),
      title: format_status(status),
    }
  }
}

/// Status progress bar component
///
/// Shows progress with a visual bar and status colors.
#[component]
pub fn StatusProgressBar(status: ProgressStatus, progress: f32) -> Element {
  let status_color = status_colors::get_status_display_color(status);
  let color_class = match status_color {
    status_colors::StatusColor::Chart1 => "bg-chart-1",
    status_colors::StatusColor::Chart2 => "bg-chart-2",
    status_colors::StatusColor::Chart3 => "bg-chart-3",
    status_colors::StatusColor::Chart4 => "bg-chart-4",
    status_colors::StatusColor::Primary => "bg-primary",
    status_colors::StatusColor::PrimaryLight => "bg-primary/80",
    status_colors::StatusColor::PrimaryDark => "bg-primary/120",
    status_colors::StatusColor::Success => "bg-success",
    status_colors::StatusColor::Warning => "bg-warning",
    status_colors::StatusColor::Error => "bg-error",
    status_colors::StatusColor::Info => "bg-info",
    status_colors::StatusColor::MutedForeground50 => "bg-muted-foreground/50",
    status_colors::StatusColor::MutedForeground40 => "bg-muted-foreground/40",
    status_colors::StatusColor::MutedForeground30 => "bg-muted-foreground/30",
    _ => "bg-muted-foreground/50",
  };

  let progress_text = format!("{:.0}%", progress * 100.0);

  rsx! {
    div {
      class: "w-full bg-background-subtle rounded-full h-2",

      // Progress fill
      div {
        class: format!("h-2 rounded-full transition-all duration-300 {}", color_class),
        style: format!("width: {}%", progress * 100.0),
      }

      // Status text
      div {
        class: "mt-1 text-xs text-muted-foreground flex justify-between",

        span {
          "{format_status(status)}"
        }

        span {
          "{progress_text}"
        }
      }
    }
  }
}

/// Status card component
///
/// Shows detailed status information in a card format.
#[component]
pub fn StatusCard(
  status: ProgressStatus,
  title: String,
  description: Option<String>,
  count: Option<usize>,
) -> Element {
  let _css_classes = status_colors::get_status_css_classes(status, false);
  let border_css_classes = status_colors::get_status_border_css_classes(status);

  rsx! {
    div {
      class: format!("p-4 border rounded-lg transition-all hover:shadow-md {}", border_css_classes),

      // Header
      div {
        class: "flex items-center justify-between mb-2",

        // Status indicator
        StatusIndicator {
          status,
          show_pulse: status_colors::STATUS_COLOR_SCHEME.should_pulse(status),
        }

        // Count badge
        if let Some(count) = count {
          span {
            class: "text-sm font-medium text-muted-foreground bg-background-subtle px-2 py-1 rounded",
            "{count}"
          }
        }
      }

      // Title
      h3 {
        class: "text-lg font-semibold mb-1",
        "{title}"
      }

      // Description
      if let Some(desc) = description {
        p {
          class: "text-sm text-muted-foreground",
          "{desc}"
        }
      }
    }
  }
}

/// Status summary component
///
/// Shows a summary of status counts across all tasks.
#[component]
pub fn StatusSummary(
  state: Signal<PlannerState>,
  on_status_click: Option<Callback<ProgressStatus>>,
) -> Element {
  let state_read = state.read();
  let _statuses = state_read.get_all_task_statuses();
  let metrics = state_read.calculate_status_metrics();

  rsx! {
    div {
      class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4",

      // Completed
      StatusCard {
        status: ProgressStatus::Completed,
        title: "Completed".to_string(),
        description: Some("Tasks finished".to_string()),
        count: Some(metrics.completed),
      }

      // In Progress
      StatusCard {
        status: ProgressStatus::InProgress,
        title: "Active".to_string(),
        description: Some("Work in progress".to_string()),
        count: Some(metrics.in_progress),
      }

      // Not Started
      StatusCard {
        status: ProgressStatus::NotStarted,
        title: "Pending".to_string(),
        description: Some("Not started yet".to_string()),
        count: Some(metrics.not_started),
      }

      // Blocked
      StatusCard {
        status: ProgressStatus::Blocked,
        title: "Blocked".to_string(),
        description: Some("Needs unblocking".to_string()),
        count: Some(metrics.blocked),
      }

      // Deferred
      StatusCard {
        status: ProgressStatus::Deferred,
        title: "Deferred".to_string(),
        description: Some("On hold".to_string()),
        count: Some(metrics.deferred),
      }
    }
  }
}

/// Size variants for status badges
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusBadgeSize {
  Small,
  Medium,
  Large,
}

/// Format status for display
#[must_use]
pub fn format_status(status: ProgressStatus) -> String {
  match status {
    ProgressStatus::Completed => "Complete",
    ProgressStatus::InProgress => "Active",
    ProgressStatus::NotStarted => "Pending",
    ProgressStatus::Blocked => "Blocked",
    ProgressStatus::Deferred => "Deferred",
  }
  .to_string()
}

/// Get color from status for styling
#[must_use]
pub fn get_color_from_status(status: ProgressStatus) -> String {
  match status {
    ProgressStatus::Completed => "chart-2",
    ProgressStatus::InProgress => "primary",
    ProgressStatus::NotStarted => "muted-foreground/50",
    ProgressStatus::Blocked => "chart-4",
    ProgressStatus::Deferred => "muted-foreground/40",
  }
  .to_string()
}

#[cfg(test)]
mod tests {
  use super::*;
  use clarity_core::progress::ProgressStatus;

  #[test]
  fn test_format_status() {
    assert_eq!(format_status(ProgressStatus::Completed), "Complete");
    assert_eq!(format_status(ProgressStatus::InProgress), "Active");
    assert_eq!(format_status(ProgressStatus::NotStarted), "Pending");
    assert_eq!(format_status(ProgressStatus::Blocked), "Blocked");
    assert_eq!(format_status(ProgressStatus::Deferred), "Deferred");
  }

  #[test]
  fn test_get_color_from_status() {
    assert_eq!(get_color_from_status(ProgressStatus::Completed), "chart-2");
    assert_eq!(get_color_from_status(ProgressStatus::InProgress), "primary");
    assert_eq!(
      get_color_from_status(ProgressStatus::NotStarted),
      "muted-foreground/50"
    );
    assert_eq!(get_color_from_status(ProgressStatus::Blocked), "chart-4");
    assert_eq!(
      get_color_from_status(ProgressStatus::Deferred),
      "muted-foreground/40"
    );
  }

  // Note: Component rendering tests (StatusIndicator, StatusBadge, StatusProgressBar)
  // should be done via integration tests with a proper Dioxus test harness.
  // Dioxus components cannot be directly instantiated in unit tests without
  // the Dioxus runtime context.
}
