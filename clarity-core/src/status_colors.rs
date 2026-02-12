//! Status color coding system
//!
//! Provides color mappings for different progress statuses across the application.
//! Uses consistent color tokens from the design system.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::progress::ProgressStatus;
use serde::{Deserialize, Serialize};

/// Color scheme for different statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusColor {
  /// Chart colors - used for data visualization and status indicators
  Chart1,
  Chart2,
  Chart3,
  Chart4,

  /// Primary brand colors
  Primary,
  PrimaryLight,
  PrimaryDark,

  /// Semantic colors
  Success,
  Warning,
  Error,
  Info,

  /// Neutral colors
  MutedForeground50,
  MutedForeground40,
  MutedForeground30,

  /// Background colors
  Background,
  BackgroundElevated,
  BackgroundSubtle,
}

/// Status color scheme configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusColorScheme {
  /// Color mapping for each status
  pub colors: std::collections::HashMap<ProgressStatus, StatusColor>,
  /// Pulse animation for active statuses
  pub pulse_statuses: Vec<ProgressStatus>,
  /// Secondary colors for hover/active states
  pub hover_colors: std::collections::HashMap<ProgressStatus, StatusColor>,
}

impl StatusColorScheme {
  /// Create a new default status color scheme
  #[must_use]
  pub fn new() -> Self {
    let mut colors = std::collections::HashMap::new();

    // Color mapping based on status
    colors.insert(ProgressStatus::Completed, StatusColor::Chart2);
    colors.insert(ProgressStatus::InProgress, StatusColor::Primary);
    colors.insert(ProgressStatus::NotStarted, StatusColor::MutedForeground50);
    colors.insert(ProgressStatus::Blocked, StatusColor::Chart4);
    colors.insert(ProgressStatus::Deferred, StatusColor::MutedForeground40);

    let mut hover_colors = std::collections::HashMap::new();

    // Hover colors - slightly lighter versions
    hover_colors.insert(ProgressStatus::Completed, StatusColor::Chart2);
    hover_colors.insert(ProgressStatus::InProgress, StatusColor::PrimaryLight);
    hover_colors.insert(ProgressStatus::NotStarted, StatusColor::MutedForeground40);
    hover_colors.insert(ProgressStatus::Blocked, StatusColor::Chart4);
    hover_colors.insert(ProgressStatus::Deferred, StatusColor::MutedForeground30);

    let pulse_statuses = vec![ProgressStatus::InProgress];

    Self {
      colors,
      pulse_statuses,
      hover_colors,
    }
  }

  /// Get the primary color for a status
  #[must_use]
  pub fn get_color(&self, status: ProgressStatus) -> StatusColor {
    self
      .colors
      .get(&status)
      .copied()
      .unwrap_or(StatusColor::MutedForeground50)
  }

  /// Get the hover color for a status
  #[must_use]
  pub fn get_hover_color(&self, status: ProgressStatus) -> StatusColor {
    self
      .hover_colors
      .get(&status)
      .copied()
      .unwrap_or(StatusColor::MutedForeground40)
  }

  /// Check if a status should have pulse animation
  #[must_use]
  pub fn should_pulse(&self, status: ProgressStatus) -> bool {
    self.pulse_statuses.contains(&status)
  }

  /// Get CSS color classes for a status
  #[must_use]
  pub fn get_css_classes(&self, status: ProgressStatus, include_pulse: bool) -> String {
    let mut classes = Vec::new();

    // Base color class
    match self.get_color(status) {
      StatusColor::Chart1 => classes.push("bg-chart-1".to_string()),
      StatusColor::Chart2 => classes.push("bg-chart-2".to_string()),
      StatusColor::Chart3 => classes.push("bg-chart-3".to_string()),
      StatusColor::Chart4 => classes.push("bg-chart-4".to_string()),
      StatusColor::Primary => classes.push("bg-primary".to_string()),
      StatusColor::PrimaryLight => classes.push("bg-primary/80".to_string()),
      StatusColor::PrimaryDark => classes.push("bg-primary/120".to_string()),
      StatusColor::Success => classes.push("bg-success".to_string()),
      StatusColor::Warning => classes.push("bg-warning".to_string()),
      StatusColor::Error => classes.push("bg-error".to_string()),
      StatusColor::Info => classes.push("bg-info".to_string()),
      StatusColor::MutedForeground50 => classes.push("bg-muted-foreground/50".to_string()),
      StatusColor::MutedForeground40 => classes.push("bg-muted-foreground/40".to_string()),
      StatusColor::MutedForeground30 => classes.push("bg-muted-foreground/30".to_string()),
      StatusColor::Background => classes.push("bg-background".to_string()),
      StatusColor::BackgroundElevated => classes.push("bg-background-elevated".to_string()),
      StatusColor::BackgroundSubtle => classes.push("bg-background-subtle".to_string()),
    }

    // Text color
    match self.get_color(status) {
      StatusColor::Chart1 | StatusColor::Chart2 | StatusColor::Chart3 | StatusColor::Chart4 => {
        classes.push("text-chart-foreground".to_string());
      }
      StatusColor::Primary | StatusColor::PrimaryLight | StatusColor::PrimaryDark => {
        classes.push("text-primary-foreground".to_string());
      }
      StatusColor::Success => classes.push("text-success-foreground".to_string()),
      StatusColor::Warning => classes.push("text-warning-foreground".to_string()),
      StatusColor::Error => classes.push("text-error-foreground".to_string()),
      StatusColor::Info => classes.push("text-info-foreground".to_string()),
      _ => classes.push("text-muted-foreground".to_string()),
    }

    // Pulse animation if applicable
    if include_pulse && self.should_pulse(status) {
      classes.push("animate-pulse".to_string());
    }

    classes.join(" ")
  }

  /// Get CSS classes for hover state
  #[must_use]
  pub fn get_hover_css_classes(&self, status: ProgressStatus) -> String {
    let mut classes = Vec::new();

    // Hover color
    match self.get_hover_color(status) {
      StatusColor::Chart1 => classes.push("hover:bg-chart-1".to_string()),
      StatusColor::Chart2 => classes.push("hover:bg-chart-2".to_string()),
      StatusColor::Chart3 => classes.push("hover:bg-chart-3".to_string()),
      StatusColor::Chart4 => classes.push("hover:bg-chart-4".to_string()),
      StatusColor::Primary => classes.push("hover:bg-primary".to_string()),
      StatusColor::PrimaryLight => classes.push("hover:bg-primary/80".to_string()),
      StatusColor::PrimaryDark => classes.push("hover:bg-primary/120".to_string()),
      StatusColor::Success => classes.push("hover:bg-success".to_string()),
      StatusColor::Warning => classes.push("hover:bg-warning".to_string()),
      StatusColor::Error => classes.push("hover:bg-error".to_string()),
      StatusColor::Info => classes.push("hover:bg-info".to_string()),
      StatusColor::MutedForeground50 => classes.push("hover:bg-muted-foreground/50".to_string()),
      StatusColor::MutedForeground40 => classes.push("hover:bg-muted-foreground/40".to_string()),
      StatusColor::MutedForeground30 => classes.push("hover:bg-muted-foreground/30".to_string()),
      StatusColor::Background => classes.push("hover:bg-background".to_string()),
      StatusColor::BackgroundElevated => classes.push("hover:bg-background-elevated".to_string()),
      StatusColor::BackgroundSubtle => classes.push("hover:bg-background-subtle".to_string()),
    }

    classes.join(" ")
  }

  /// Get border color classes for a status
  #[must_use]
  pub fn get_border_css_classes(&self, status: ProgressStatus) -> String {
    let mut classes = Vec::new();

    // Border color matches background color but with opacity
    match self.get_color(status) {
      StatusColor::Chart1 => classes.push("border-chart-1/30".to_string()),
      StatusColor::Chart2 => classes.push("border-chart-2/30".to_string()),
      StatusColor::Chart3 => classes.push("border-chart-3/30".to_string()),
      StatusColor::Chart4 => classes.push("border-chart-4/30".to_string()),
      StatusColor::Primary => classes.push("border-primary/30".to_string()),
      StatusColor::PrimaryLight => classes.push("border-primary/20".to_string()),
      StatusColor::PrimaryDark => classes.push("border-primary/40".to_string()),
      StatusColor::Success => classes.push("border-success/30".to_string()),
      StatusColor::Warning => classes.push("border-warning/30".to_string()),
      StatusColor::Error => classes.push("border-error/30".to_string()),
      StatusColor::Info => classes.push("border-info/30".to_string()),
      StatusColor::MutedForeground50 => classes.push("border-muted-foreground/20".to_string()),
      StatusColor::MutedForeground40 => classes.push("border-muted-foreground/15".to_string()),
      StatusColor::MutedForeground30 => classes.push("border-muted-foreground/10".to_string()),
      StatusColor::Background => classes.push("border-background/20".to_string()),
      StatusColor::BackgroundElevated => classes.push("border-background-elevated/20".to_string()),
      StatusColor::BackgroundSubtle => classes.push("border-background-subtle/20".to_string()),
    }

    classes.join(" ")
  }
}

impl Default for StatusColorScheme {
  fn default() -> Self {
    Self::new()
  }
}

/// Global status color scheme instance
pub static STATUS_COLOR_SCHEME: std::sync::LazyLock<StatusColorScheme> =
  std::sync::LazyLock::new(StatusColorScheme::new);

/// Convenience function to get CSS classes for a status
#[must_use]
pub fn get_status_css_classes(status: ProgressStatus, include_pulse: bool) -> String {
  STATUS_COLOR_SCHEME.get_css_classes(status, include_pulse)
}

/// Convenience function to get hover CSS classes for a status
#[must_use]
pub fn get_status_hover_css_classes(status: ProgressStatus) -> String {
  STATUS_COLOR_SCHEME.get_hover_css_classes(status)
}

/// Convenience function to get border CSS classes for a status
#[must_use]
pub fn get_status_border_css_classes(status: ProgressStatus) -> String {
  STATUS_COLOR_SCHEME.get_border_css_classes(status)
}

/// Color mapping for display purposes
#[must_use]
pub fn get_status_display_color(status: ProgressStatus) -> StatusColor {
  STATUS_COLOR_SCHEME.get_color(status)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::progress::ProgressStatus;

  #[test]
  fn test_status_color_scheme_creation() {
    let scheme = StatusColorScheme::new();

    // Test all statuses have colors
    assert_eq!(
      scheme.get_color(ProgressStatus::Completed),
      StatusColor::Chart2
    );
    assert_eq!(
      scheme.get_color(ProgressStatus::InProgress),
      StatusColor::Primary
    );
    assert_eq!(
      scheme.get_color(ProgressStatus::NotStarted),
      StatusColor::MutedForeground50
    );
    assert_eq!(
      scheme.get_color(ProgressStatus::Blocked),
      StatusColor::Chart4
    );
    assert_eq!(
      scheme.get_color(ProgressStatus::Deferred),
      StatusColor::MutedForeground40
    );
  }

  #[test]
  fn test_status_pulse_statuses() {
    let scheme = StatusColorScheme::new();

    // Only InProgress should pulse by default
    assert!(scheme.should_pulse(ProgressStatus::InProgress));
    assert!(!scheme.should_pulse(ProgressStatus::Completed));
    assert!(!scheme.should_pulse(ProgressStatus::NotStarted));
    assert!(!scheme.should_pulse(ProgressStatus::Blocked));
    assert!(!scheme.should_pulse(ProgressStatus::Deferred));
  }

  #[test]
  fn test_status_css_classes() {
    let scheme = StatusColorScheme::new();

    // Test Completed status
    let classes = scheme.get_css_classes(ProgressStatus::Completed, false);
    assert!(classes.contains("bg-chart-2"));
    assert!(classes.contains("text-chart-foreground"));

    // Test InProgress status with pulse
    let classes = scheme.get_css_classes(ProgressStatus::InProgress, true);
    assert!(classes.contains("bg-primary"));
    assert!(classes.contains("text-primary-foreground"));
    assert!(classes.contains("animate-pulse"));

    // Test NotStarted status
    let classes = scheme.get_css_classes(ProgressStatus::NotStarted, false);
    assert!(classes.contains("bg-muted-foreground/50"));
    assert!(classes.contains("text-muted-foreground"));
  }

  #[test]
  fn test_status_hover_classes() {
    let scheme = StatusColorScheme::new();

    let classes = scheme.get_hover_css_classes(ProgressStatus::Completed);
    assert!(classes.contains("hover:bg-chart-2"));

    let classes = scheme.get_hover_css_classes(ProgressStatus::InProgress);
    assert!(classes.contains("hover:bg-primary/80"));
  }

  #[test]
  fn test_status_border_classes() {
    let scheme = StatusColorScheme::new();

    let classes = scheme.get_border_css_classes(ProgressStatus::Completed);
    assert!(classes.contains("border-chart-2/30"));

    let classes = scheme.get_border_css_classes(ProgressStatus::InProgress);
    assert!(classes.contains("border-primary/30"));
  }

  #[test]
  fn test_convenience_functions() {
    // Test convenience functions
    let classes = get_status_css_classes(ProgressStatus::Completed, false);
    assert!(classes.contains("bg-chart-2"));

    let hover_classes = get_status_hover_css_classes(ProgressStatus::Completed);
    assert!(hover_classes.contains("hover:bg-chart-2"));

    let border_classes = get_status_border_css_classes(ProgressStatus::Completed);
    assert!(border_classes.contains("border-chart-2/30"));

    let color = get_status_display_color(ProgressStatus::Completed);
    assert_eq!(color, StatusColor::Chart2);
  }

  #[test]
  fn test_default_scheme() {
    let scheme = StatusColorScheme::default();
    assert_eq!(
      scheme.get_color(ProgressStatus::Completed),
      StatusColor::Chart2
    );
  }
}
