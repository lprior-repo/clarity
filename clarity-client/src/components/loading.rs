#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Loading spinner component
//!
//! Provides reusable loading indicators for async operations.

use dioxus::prelude::*;

/// Loading size variants
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LoadingSize {
  #[default]
  Small,
  Medium,
  Large,
}

impl LoadingSize {
  #[must_use]
  pub const fn as_str(&self) -> &str {
    match self {
      Self::Small => "small",
      Self::Medium => "medium",
      Self::Large => "large",
    }
  }
}

/// Loading variant types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LoadingVariant {
  #[default]
  Spinner,
  Dots,
  Pulse,
  Skeleton,
}

impl LoadingVariant {
  #[must_use]
  pub const fn as_str(&self) -> &str {
    match self {
      Self::Spinner => "spinner",
      Self::Dots => "dots",
      Self::Pulse => "pulse",
      Self::Skeleton => "skeleton",
    }
  }
}

/// Loading component properties
#[derive(Clone, Debug, Props, PartialEq, Eq)]
pub struct LoadingProps {
  /// Size of the loading indicator
  #[props(default)]
  pub size: LoadingSize,
  /// Variant of loading indicator
  #[props(default)]
  pub variant: LoadingVariant,
  /// Optional message to display
  #[props(default)]
  pub message: Option<String>,
  /// Whether to show a full-screen overlay
  #[props(default)]
  pub fullscreen: bool,
}

/// Loading spinner component
///
/// Displays a loading indicator with optional message.
/// Supports multiple sizes and variants for different contexts.
///
/// # Examples
///
/// Basic spinner:
/// ```rsx
/// Loading { }
/// ```
///
/// With message:
/// ```rsx
/// Loading {
///     message: "Loading beads...".to_string()
/// }
/// ```
///
/// Custom size and variant:
/// ```rsx
/// Loading {
///     size: LoadingSize::Large,
///     variant: LoadingVariant::Dots,
///     message: "Please wait...".to_string()
/// }
/// ```
#[component]
pub fn Loading(props: LoadingProps) -> Element {
  let size_class = format!("loading-{}", props.size.as_str());
  let variant_class = format!("loading-{}", props.variant.as_str());
  let base_classes = if props.fullscreen {
    "loading-overlay".to_string()
  } else {
    "loading-container".to_string()
  };

  rsx! {
      div { class: "{base_classes}",
          div { class: "{variant_class} {size_class}",
              match props.variant {
                  LoadingVariant::Spinner => rsx! {
                      svg {
                          class: "spinner",
                          view_box: "0 0 50 50",
                          circle {
                              class: "spinner-circle",
                              cx: "25",
                              cy: "25",
                              r: "20",
                              fill: "none",
                              stroke: "currentColor",
                              "stroke-width": "4"
                          }
                      }
                  },
                  LoadingVariant::Dots => rsx! {
                      div { class: "dots-wrapper",
                          span { class: "dot" }
                          span { class: "dot" }
                          span { class: "dot" }
                      }
                  },
                  LoadingVariant::Pulse => rsx! {
                      div { class: "pulse-circle" }
                  },
                  LoadingVariant::Skeleton => rsx! {
                      div { class: "skeleton-block" }
                  },
              }
          }

          {props.message.map(|msg| rsx! {
                  p { class: "loading-message", "{msg}" }
              })}
      }
  }
}

/// Inline loading component for small spaces
///
/// A compact loading indicator for use within buttons, cards, etc.
#[component]
pub fn LoadingInline(#[props(default)] message: Option<String>) -> Element {
  rsx! {
      span { class: "loading-inline",
          span { class: "spinner-inline" }
          {message.map(|msg| rsx! {
                  span { class: "loading-message-inline", "{msg}" }
              })}
      }
  }
}

/// Page-level loading component
///
/// A full-page loading state for route transitions or initial load.
#[component]
pub fn LoadingPage(message: String) -> Element {
  rsx! {
      div { class: "loading-page",
          div { class: "loading-page-content",
              Loading {
                  size: LoadingSize::Large,
                  variant: LoadingVariant::Spinner,
                  message: Some(message)
              }
          }
      }
  }
}

/// Card skeleton loader
///
/// Displays placeholder content while data loads.
#[component]
pub fn CardSkeleton(#[props(default)] count: usize) -> Element {
  let count = if count == 0 { 3 } else { count };

  rsx! {
      div { class: "skeleton-grid",
          for i in 0..count {
              div { key: "skeleton-{i}", class: "skeleton-card",
                  div { class: "skeleton-header" }
                  div { class: "skeleton-title" }
                  div { class: "skeleton-text" }
                  div { class: "skeleton-text short" }
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_loading_size_display() {
    assert_eq!(LoadingSize::Small.as_str(), "small");
    assert_eq!(LoadingSize::Medium.as_str(), "medium");
    assert_eq!(LoadingSize::Large.as_str(), "large");
  }

  #[test]
  fn test_loading_variant_display() {
    assert_eq!(LoadingVariant::Spinner.as_str(), "spinner");
    assert_eq!(LoadingVariant::Dots.as_str(), "dots");
    assert_eq!(LoadingVariant::Pulse.as_str(), "pulse");
    assert_eq!(LoadingVariant::Skeleton.as_str(), "skeleton");
  }

  #[test]
  fn test_loading_props_default() {
    // Note: LoadingProps uses Dioxus Props derive with #[props(default)]
    // We need to construct it manually for testing
    let props = LoadingProps {
      size: LoadingSize::Small,
      variant: LoadingVariant::Spinner,
      message: None,
      fullscreen: false,
    };
    assert_eq!(props.size, LoadingSize::Small);
    assert_eq!(props.variant, LoadingVariant::Spinner);
    assert!(props.message.is_none());
    assert!(!props.fullscreen);
  }

  #[test]
  fn test_loading_size_default() {
    assert_eq!(LoadingSize::default(), LoadingSize::Small);
  }

  #[test]
  fn test_loading_variant_default() {
    assert_eq!(LoadingVariant::default(), LoadingVariant::Spinner);
  }
}
