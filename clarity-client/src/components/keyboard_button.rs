#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Keyboard-enhanced button components
//!
//! This module provides button components that display keyboard shortcut hints
//! and tooltips for power users.

use crate::shortcuts::{Action, Shortcuts};
use dioxus::prelude::*;

/// Keyboard-enhanced button component properties
#[derive(Clone, Props)]
pub struct KeyboardButtonProps {
  /// The action this button performs
  pub action: Action,
  /// Button content (text, icon, etc.)
  pub children: Element,
  /// Additional CSS classes
  #[props(default)]
  pub class: String,
  /// Tooltip text
  #[props(default)]
  pub tooltip: String,
  /// Disabled state
  #[props(default)]
  pub disabled: bool,
  /// Click handler
  pub onclick: Callback<MouseEvent>,
}

impl PartialEq for KeyboardButtonProps {
  fn eq(&self, other: &Self) -> bool {
    self.action == other.action
      && self.class == other.class
      && self.tooltip == other.tooltip
      && self.disabled == other.disabled
  }
}

/// Keyboard-enhanced button component
///
/// A button that automatically displays a keyboard shortcut hint
/// and provides tooltip functionality for better UX.
#[component]
pub fn KeyboardButton(props: KeyboardButtonProps) -> Element {
  let shortcuts = Shortcuts::default_mappings();
  let action = props.action;
  let tooltip = props.tooltip;

  // Find the shortcuts for this action
  let shortcut_hint = shortcuts
    .descriptions()
    .iter()
    .find(|desc| desc.action == action)
    .map(|desc| {
      // Format all shortcuts for display
      let formatted_shortcuts = desc
        .shortcuts
        .iter()
        .map(|s| s.format())
        .collect::<Vec<_>>()
        .join(" / ");

      rsx! {
          kbd {
              class: "shortcut-hint",
              title: "Keyboard shortcuts",
              "{formatted_shortcuts}"
          }
      }
    });

  rsx! {
      div {
          class: "keyboard-button-container",
          title: tooltip,

          button {
              class: format!("btn {} {}", props.class,
                  if props.disabled { "disabled" } else { "" }),
              disabled: props.disabled,
              onclick: props.onclick,

              // Main content
              span { class: "button-content", {props.children} }

              // Shortcut hint (if available)
              {shortcut_hint}
          }

          // Tooltip (if provided)
          if !tooltip.is_empty() {
              span {
                  class: "button-tooltip",
                  "{tooltip}"
              }
          }
      }
  }
}

/// Standard save button with keyboard hints
#[component]
pub fn SaveButton(onclick: Callback<MouseEvent>, disabled: bool) -> Element {
  let content = match disabled {
    true => "Saving...".to_string(),
    false => "Save Plan".to_string(),
  };

  rsx! {
      KeyboardButton {
          action: crate::shortcuts::Action::SaveForm,
          class: "btn btn-primary".to_string(),
          tooltip: "Save your planning progress".to_string(),
          disabled,
          onclick,
          children: rsx! { {content} },
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_keyboard_button_props_default_values() {
    // Test that props with default values work
    let props = KeyboardButtonProps {
      action: crate::shortcuts::Action::SaveForm,
      children: rsx! { "Save" },
      class: String::new(),
      tooltip: String::new(),
      disabled: false,
      onclick: Callback::new(|_| {}),
    };
    assert_eq!(props.action, crate::shortcuts::Action::SaveForm);
    assert_eq!(props.disabled, false);
    assert!(props.tooltip.is_empty());
  }

  #[test]
  fn test_shortcut_hint_formatting() {
    // Test that shortcut hints are properly formatted
    let shortcuts = Shortcuts::default_mappings();
    let save_desc = shortcuts
      .descriptions()
      .iter()
      .find(|desc| desc.action == crate::shortcuts::Action::SaveForm);

    assert!(save_desc.is_some(), "SaveForm action should exist");
    if let Some(desc) = save_desc {
      assert_eq!(desc.shortcuts[0].format(), "Ctrl+s");
    }
  }
}
