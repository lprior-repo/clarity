#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_const_for_fn)]

//! Responsive design hook for Clarity desktop app
//!
//! This module provides hooks for detecting window dimensions and breakpoints
//! for responsive UI design. Uses Dioxus signals for reactive state management.
//!
//! ## Breakpoints
//! - Mobile: < 768px
//! - Tablet: 768px - 1023px
//! - Desktop: >= 1024px
//! - Large Desktop: >= 1280px
//!
//! ## Usage
//! ```rust,ignore
//! fn MyComponent() -> Element {
//!     let responsive = use_responsive();
//!
//!     rsx! {
//!         div {
//!             class: if responsive.is_mobile() { "flex-col" } else { "flex-row" },
//!             "Content adapts to screen size"
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;

// ===== Breakpoint Constants =====

/// Mobile breakpoint threshold (exclusive upper bound)
const MOBILE_MAX: u32 = 767;

/// Tablet breakpoint range: 768px - 1023px
const TABLET_MIN: u32 = 768;
const TABLET_MAX: u32 = 1023;

/// Desktop breakpoint: >= 1024px
const DESKTOP_MIN: u32 = 1024;

/// Large desktop breakpoint: >= 1280px
const LARGE_DESKTOP_MIN: u32 = 1280;

/// Default window width when dimension cannot be determined
const DEFAULT_WIDTH: u32 = 1024;

/// Default window height when dimension cannot be determined
const DEFAULT_HEIGHT: u32 = 768;

// ===== Breakpoint Enum =====

/// Responsive breakpoint enum for type-safe breakpoint matching
///
/// Represents the four primary breakpoints in the Clarity design system:
/// - Mobile: < 768px
/// - Tablet: 768px - 1023px
/// - Desktop: 1024px - 1279px
/// - `LargeDesktop`: >= 1280px
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResponsiveBreakpoint {
  /// Mobile devices (< 768px)
  Mobile,
  /// Tablet devices (768px - 1023px)
  Tablet,
  /// Desktop screens (1024px - 1279px)
  Desktop,
  /// Large desktop screens (>= 1280px)
  LargeDesktop,
}

impl ResponsiveBreakpoint {
  /// Determine breakpoint from width
  ///
  /// Pure function that maps a pixel width to the corresponding breakpoint.
  ///
  /// # Arguments
  /// * `width` - Window width in pixels
  ///
  /// # Returns
  /// The corresponding `ResponsiveBreakpoint`
  #[must_use]
  pub const fn from_width(width: u32) -> Self {
    if width <= MOBILE_MAX {
      Self::Mobile
    } else if width <= TABLET_MAX {
      Self::Tablet
    } else if width < LARGE_DESKTOP_MIN {
      Self::Desktop
    } else {
      Self::LargeDesktop
    }
  }

  /// Check if this breakpoint is mobile
  #[must_use]
  pub const fn is_mobile(self) -> bool {
    matches!(self, Self::Mobile)
  }

  /// Check if this breakpoint is tablet
  #[must_use]
  pub const fn is_tablet(self) -> bool {
    matches!(self, Self::Tablet)
  }

  /// Check if this breakpoint is desktop or larger
  #[must_use]
  pub const fn is_desktop_or_larger(self) -> bool {
    matches!(self, Self::Desktop | Self::LargeDesktop)
  }

  /// Check if this breakpoint is tablet or smaller
  #[must_use]
  pub const fn is_tablet_or_smaller(self) -> bool {
    matches!(self, Self::Mobile | Self::Tablet)
  }

  /// Get the minimum width for this breakpoint
  #[must_use]
  pub const fn min_width(self) -> u32 {
    match self {
      Self::Mobile => 0,
      Self::Tablet => TABLET_MIN,
      Self::Desktop => DESKTOP_MIN,
      Self::LargeDesktop => LARGE_DESKTOP_MIN,
    }
  }

  /// Get CSS class for this breakpoint
  #[must_use]
  pub const fn css_class(self) -> &'static str {
    match self {
      Self::Mobile => "breakpoint-mobile",
      Self::Tablet => "breakpoint-tablet",
      Self::Desktop => "breakpoint-desktop",
      Self::LargeDesktop => "breakpoint-large-desktop",
    }
  }
}

impl Default for ResponsiveBreakpoint {
  fn default() -> Self {
    Self::Desktop
  }
}

impl std::fmt::Display for ResponsiveBreakpoint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Mobile => write!(f, "mobile (<{TABLET_MIN}px)"),
      Self::Tablet => write!(f, "tablet ({TABLET_MIN}-{TABLET_MAX}px)"),
      Self::Desktop => write!(f, "desktop ({DESKTOP_MIN}-{}px)", LARGE_DESKTOP_MIN - 1),
      Self::LargeDesktop => write!(f, "large desktop (>={LARGE_DESKTOP_MIN}px)"),
    }
  }
}

// ===== Responsive State =====

/// Immutable responsive state snapshot
///
/// Contains all information about the current responsive state including
/// dimensions and computed breakpoint flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResponsiveState {
  /// Current window width in pixels
  width: u32,
  /// Current window height in pixels
  height: u32,
  /// Current breakpoint
  breakpoint: ResponsiveBreakpoint,
}

impl ResponsiveState {
  /// Create a new responsive state from dimensions
  ///
  /// # Arguments
  /// * `width` - Window width in pixels
  /// * `height` - Window height in pixels
  #[must_use]
  pub const fn new(width: u32, height: u32) -> Self {
    Self {
      width,
      height,
      breakpoint: ResponsiveBreakpoint::from_width(width),
    }
  }

  /// Create responsive state with default dimensions
  #[must_use]
  pub const fn with_defaults() -> Self {
    Self::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)
  }

  /// Check if current breakpoint is mobile (< 768px)
  #[must_use]
  pub const fn is_mobile(self) -> bool {
    self.breakpoint.is_mobile()
  }

  /// Check if current breakpoint is tablet (768px - 1023px)
  #[must_use]
  pub const fn is_tablet(self) -> bool {
    self.breakpoint.is_tablet()
  }

  /// Check if current breakpoint is desktop (>= 1024px)
  #[must_use]
  pub const fn is_desktop(self) -> bool {
    self.breakpoint.is_desktop_or_larger()
  }

  /// Check if current breakpoint is large desktop (>= 1280px)
  #[must_use]
  pub const fn is_large_desktop(self) -> bool {
    matches!(self.breakpoint, ResponsiveBreakpoint::LargeDesktop)
  }

  /// Get current window width
  #[must_use]
  pub const fn width(self) -> u32 {
    self.width
  }

  /// Get current window height
  #[must_use]
  pub const fn height(self) -> u32 {
    self.height
  }

  /// Get current breakpoint
  #[must_use]
  pub const fn breakpoint(self) -> ResponsiveBreakpoint {
    self.breakpoint
  }

  /// Get aspect ratio (width / height)
  ///
  /// Returns `None` if height is 0 to avoid division by zero.
  #[must_use]
  pub fn aspect_ratio(self) -> Option<f64> {
    if self.height == 0 {
      None
    } else {
      Some(f64::from(self.width) / f64::from(self.height))
    }
  }

  /// Check if the viewport is in portrait orientation
  #[must_use]
  pub const fn is_portrait(self) -> bool {
    self.height > self.width
  }

  /// Check if the viewport is in landscape orientation
  #[must_use]
  pub const fn is_landscape(self) -> bool {
    self.width > self.height
  }

  /// Get recommended sidebar width based on breakpoint
  #[must_use]
  pub const fn sidebar_width(self) -> u32 {
    match self.breakpoint {
      ResponsiveBreakpoint::Mobile => 0, // Hidden
      ResponsiveBreakpoint::Tablet => 280,
      ResponsiveBreakpoint::Desktop => 320,
      ResponsiveBreakpoint::LargeDesktop => 380,
    }
  }

  /// Check if sidebar should be collapsed
  #[must_use]
  pub const fn should_collapse_sidebar(self) -> bool {
    self.breakpoint.is_mobile()
  }

  /// Get recommended font scale factor (1.0 = base)
  #[must_use]
  pub const fn font_scale(self) -> f32 {
    match self.breakpoint {
      ResponsiveBreakpoint::Mobile => 0.875,
      ResponsiveBreakpoint::Tablet => 0.9375,
      ResponsiveBreakpoint::Desktop => 1.0,
      ResponsiveBreakpoint::LargeDesktop => 1.0625,
    }
  }

  /// Get recommended button size (min 44px for touch)
  #[must_use]
  pub const fn button_size(self) -> u32 {
    if self.breakpoint.is_tablet_or_smaller() {
      44 // Touch-friendly size
    } else {
      36 // Desktop size
    }
  }
}

impl Default for ResponsiveState {
  fn default() -> Self {
    Self::with_defaults()
  }
}

// ===== Hook Implementation =====

/// Hook for responsive design state
///
/// Returns a `ResponsiveState` snapshot based on default desktop dimensions.
/// This hook provides breakpoint detection for responsive UI design.
///
/// For dynamic window sizing, use `use_responsive_with_dimensions` instead.
///
/// # Returns
/// A `ResponsiveState` with current dimensions and breakpoint information
///
/// # Example
/// ```rust,ignore
/// fn MyComponent() -> Element {
///     let responsive = use_responsive();
///
///     rsx! {
///         div {
///             class: if responsive.is_mobile() {
///                 "flex flex-col"
///             } else {
///                 "flex flex-row"
///             },
///             // Content
///         }
///     }
/// }
/// ```
#[must_use]
pub const fn use_responsive() -> ResponsiveState {
  // Return default desktop state
  // In a real implementation, this would read from window dimensions
  ResponsiveState::with_defaults()
}

/// Hook for responsive state with custom dimensions
///
/// Use this when you need to specify window dimensions manually,
/// such as when reading from dioxus-desktop window API.
///
/// # Arguments
/// * `width` - Current window width in pixels
/// * `height` - Current window height in pixels
///
/// # Returns
/// A `ResponsiveState` for the given dimensions
#[must_use]
pub const fn use_responsive_with_dimensions(width: u32, height: u32) -> ResponsiveState {
  ResponsiveState::new(width, height)
}

/// Hook for responsive state from a signal
///
/// Use this when you have a signal that tracks window dimensions.
///
/// # Arguments
/// * `dimensions_signal` - Signal containing (width, height) tuple
///
/// # Returns
/// A `ResponsiveState` computed from the signal dimensions
#[must_use]
pub fn use_responsive_from_signal(dimensions_signal: Signal<(u32, u32)>) -> ResponsiveState {
  let (width, height) = *dimensions_signal.read();
  ResponsiveState::new(width, height)
}

/// Hook for responsive breakpoint only
///
/// Use this when you only need the breakpoint, not full dimensions.
///
/// # Returns
/// Current `ResponsiveBreakpoint`
#[must_use]
pub fn use_breakpoint() -> ResponsiveBreakpoint {
  use_responsive().breakpoint()
}

/// Hook for checking if viewport is mobile
///
/// Convenience hook for mobile-specific logic.
#[must_use]
pub fn use_is_mobile() -> bool {
  use_responsive().is_mobile()
}

/// Hook for checking if viewport is tablet
///
/// Convenience hook for tablet-specific logic.
#[must_use]
pub fn use_is_tablet() -> bool {
  use_responsive().is_tablet()
}

/// Hook for checking if viewport is desktop
///
/// Convenience hook for desktop-specific logic.
#[must_use]
pub fn use_is_desktop() -> bool {
  use_responsive().is_desktop()
}

// ===== Responsive Layout Helper =====

/// Get responsive CSS classes for common layout patterns
///
/// Returns a string of Tailwind-like CSS classes based on the current breakpoint.
///
/// # Arguments
/// * `mobile_class` - Classes for mobile breakpoint
/// * `tablet_class` - Classes for tablet breakpoint (optional)
/// * `desktop_class` - Classes for desktop breakpoint
#[must_use]
pub fn use_responsive_classes(
  mobile_class: &str,
  tablet_class: Option<&str>,
  desktop_class: &str,
) -> String {
  let state = use_responsive();

  match state.breakpoint() {
    ResponsiveBreakpoint::Mobile => mobile_class.to_string(),
    ResponsiveBreakpoint::Tablet => {
      tablet_class.map_or_else(|| desktop_class.to_string(), ToString::to_string)
    }
    ResponsiveBreakpoint::Desktop | ResponsiveBreakpoint::LargeDesktop => desktop_class.to_string(),
  }
}

/// Get responsive value based on breakpoint
///
/// Returns different values based on the current breakpoint.
///
/// # Arguments
/// * `mobile_value` - Value for mobile breakpoint
/// * `tablet_value` - Value for tablet breakpoint
/// * `desktop_value` - Value for desktop breakpoint
#[must_use]
pub fn use_responsive_value<T: Clone>(mobile_value: T, tablet_value: T, desktop_value: T) -> T {
  let state = use_responsive();

  match state.breakpoint() {
    ResponsiveBreakpoint::Mobile => mobile_value,
    ResponsiveBreakpoint::Tablet => tablet_value,
    ResponsiveBreakpoint::Desktop | ResponsiveBreakpoint::LargeDesktop => desktop_value,
  }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]

  use super::*;

  #[test]
  fn test_breakpoint_from_width() {
    assert_eq!(
      ResponsiveBreakpoint::from_width(0),
      ResponsiveBreakpoint::Mobile
    );
    assert_eq!(
      ResponsiveBreakpoint::from_width(320),
      ResponsiveBreakpoint::Mobile
    );
    assert_eq!(
      ResponsiveBreakpoint::from_width(767),
      ResponsiveBreakpoint::Mobile
    );

    assert_eq!(
      ResponsiveBreakpoint::from_width(768),
      ResponsiveBreakpoint::Tablet
    );
    assert_eq!(
      ResponsiveBreakpoint::from_width(1023),
      ResponsiveBreakpoint::Tablet
    );

    assert_eq!(
      ResponsiveBreakpoint::from_width(1024),
      ResponsiveBreakpoint::Desktop
    );
    assert_eq!(
      ResponsiveBreakpoint::from_width(1279),
      ResponsiveBreakpoint::Desktop
    );

    assert_eq!(
      ResponsiveBreakpoint::from_width(1280),
      ResponsiveBreakpoint::LargeDesktop
    );
    assert_eq!(
      ResponsiveBreakpoint::from_width(1920),
      ResponsiveBreakpoint::LargeDesktop
    );
  }

  #[test]
  fn test_breakpoint_checks() {
    assert!(ResponsiveBreakpoint::Mobile.is_mobile());
    assert!(!ResponsiveBreakpoint::Mobile.is_tablet());
    assert!(!ResponsiveBreakpoint::Mobile.is_desktop_or_larger());

    assert!(!ResponsiveBreakpoint::Tablet.is_mobile());
    assert!(ResponsiveBreakpoint::Tablet.is_tablet());
    assert!(!ResponsiveBreakpoint::Tablet.is_desktop_or_larger());
    assert!(ResponsiveBreakpoint::Tablet.is_tablet_or_smaller());

    assert!(ResponsiveBreakpoint::Desktop.is_desktop_or_larger());
    assert!(!ResponsiveBreakpoint::Desktop.is_tablet_or_smaller());

    assert!(ResponsiveBreakpoint::LargeDesktop.is_desktop_or_larger());
  }

  #[test]
  fn test_breakpoint_min_width() {
    assert_eq!(ResponsiveBreakpoint::Mobile.min_width(), 0);
    assert_eq!(ResponsiveBreakpoint::Tablet.min_width(), 768);
    assert_eq!(ResponsiveBreakpoint::Desktop.min_width(), 1024);
    assert_eq!(ResponsiveBreakpoint::LargeDesktop.min_width(), 1280);
  }

  #[test]
  fn test_breakpoint_css_class() {
    assert_eq!(
      ResponsiveBreakpoint::Mobile.css_class(),
      "breakpoint-mobile"
    );
    assert_eq!(
      ResponsiveBreakpoint::Tablet.css_class(),
      "breakpoint-tablet"
    );
    assert_eq!(
      ResponsiveBreakpoint::Desktop.css_class(),
      "breakpoint-desktop"
    );
    assert_eq!(
      ResponsiveBreakpoint::LargeDesktop.css_class(),
      "breakpoint-large-desktop"
    );
  }

  #[test]
  fn test_responsive_state_new() {
    let state = ResponsiveState::new(800, 600);
    assert_eq!(state.width(), 800);
    assert_eq!(state.height(), 600);
    assert_eq!(state.breakpoint(), ResponsiveBreakpoint::Tablet);
    assert!(state.is_tablet());
    assert!(!state.is_mobile());
    assert!(!state.is_desktop());
  }

  #[test]
  fn test_responsive_state_defaults() {
    let state = ResponsiveState::with_defaults();
    assert_eq!(state.width(), DEFAULT_WIDTH);
    assert_eq!(state.height(), DEFAULT_HEIGHT);
    assert!(state.is_desktop());
  }

  #[test]
  fn test_responsive_state_orientation() {
    let portrait = ResponsiveState::new(600, 800);
    assert!(portrait.is_portrait());
    assert!(!portrait.is_landscape());

    let landscape = ResponsiveState::new(800, 600);
    assert!(landscape.is_landscape());
    assert!(!landscape.is_portrait());

    let square = ResponsiveState::new(500, 500);
    assert!(!square.is_portrait());
    assert!(!square.is_landscape());
  }

  #[test]
  fn test_responsive_state_aspect_ratio() {
    let state = ResponsiveState::new(800, 600);
    let ratio = state.aspect_ratio();
    assert!(ratio.is_some());
    let ratio = ratio.unwrap();
    assert!((ratio - 1.333_333_333_333_333_3).abs() < f64::EPSILON);

    let zero_height = ResponsiveState::new(800, 0);
    assert!(zero_height.aspect_ratio().is_none());
  }

  #[test]
  fn test_responsive_state_sidebar_width() {
    let mobile = ResponsiveState::new(320, 568);
    assert_eq!(mobile.sidebar_width(), 0);
    assert!(mobile.should_collapse_sidebar());

    let tablet = ResponsiveState::new(768, 1024);
    assert_eq!(tablet.sidebar_width(), 280);
    assert!(!tablet.should_collapse_sidebar());

    let desktop = ResponsiveState::new(1024, 768);
    assert_eq!(desktop.sidebar_width(), 320);

    let large = ResponsiveState::new(1920, 1080);
    assert_eq!(large.sidebar_width(), 380);
  }

  #[test]
  fn test_responsive_state_font_scale() {
    let mobile = ResponsiveState::new(320, 568);
    assert!((mobile.font_scale() - 0.875).abs() < f32::EPSILON);

    let tablet = ResponsiveState::new(768, 1024);
    assert!((tablet.font_scale() - 0.9375).abs() < f32::EPSILON);

    let desktop = ResponsiveState::new(1024, 768);
    assert!((desktop.font_scale() - 1.0).abs() < f32::EPSILON);

    let large = ResponsiveState::new(1920, 1080);
    assert!((large.font_scale() - 1.0625).abs() < f32::EPSILON);
  }

  #[test]
  fn test_responsive_state_button_size() {
    let mobile = ResponsiveState::new(320, 568);
    assert_eq!(mobile.button_size(), 44); // Touch-friendly

    let tablet = ResponsiveState::new(768, 1024);
    assert_eq!(tablet.button_size(), 44); // Touch-friendly

    let desktop = ResponsiveState::new(1024, 768);
    assert_eq!(desktop.button_size(), 36); // Desktop size
  }

  #[test]
  fn test_breakpoint_display() {
    let mobile = format!("{}", ResponsiveBreakpoint::Mobile);
    assert!(mobile.contains("mobile"));
    assert!(mobile.contains("768"));

    let tablet = format!("{}", ResponsiveBreakpoint::Tablet);
    assert!(tablet.contains("tablet"));
    assert!(tablet.contains("768"));
    assert!(tablet.contains("1023"));

    let desktop = format!("{}", ResponsiveBreakpoint::Desktop);
    assert!(desktop.contains("desktop"));
    assert!(desktop.contains("1024"));

    let large = format!("{}", ResponsiveBreakpoint::LargeDesktop);
    assert!(large.contains("large desktop"));
    assert!(large.contains("1280"));
  }

  #[test]
  fn test_responsive_state_equality() {
    let state1 = ResponsiveState::new(800, 600);
    let state2 = ResponsiveState::new(800, 600);
    let state3 = ResponsiveState::new(800, 700);

    assert_eq!(state1, state2);
    assert_ne!(state1, state3);
  }

  #[test]
  fn test_breakpoint_default() {
    let breakpoint = ResponsiveBreakpoint::default();
    assert_eq!(breakpoint, ResponsiveBreakpoint::Desktop);
  }
}
