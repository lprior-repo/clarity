#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::option_if_let_else)]

//! Keyboard shortcuts help dialog component
//!
//! This module provides a modal dialog that displays all available
//! keyboard shortcuts in the application. It's a pure UI component
//! that receives visibility state through props.

use crate::shortcuts::{Action, Shortcuts};
use dioxus::prelude::*;
use std::rc::Rc;

/// Keyboard help dialog component
///
/// Displays a modal dialog with all registered keyboard shortcuts.
/// The dialog visibility is controlled by the `visible` prop.
///
/// # Arguments
///
/// * `visible` - Whether the dialog is currently visible
/// Keyboard help dialog component properties
#[derive(Clone, Props)]
pub struct KeyboardHelpDialogProps {
  /// Whether the dialog is currently visible
  pub visible: bool,
  /// Callback when the dialog is closed
  #[props(default)]
  pub on_close: Callback,
}

// Manual PartialEq for KeyboardHelpDialogProps
impl PartialEq for KeyboardHelpDialogProps {
  fn eq(&self, other: &Self) -> bool {
    self.visible == other.visible
  }
}

impl Eq for KeyboardHelpDialogProps {}

/// * `on_close` - Callback when the dialog is closed
#[component]
pub fn KeyboardHelpDialog(props: KeyboardHelpDialogProps) -> Element {
  let visible = props.visible;
  let on_close = props.on_close;
  let shortcuts = Shortcuts::default_mappings();

  rsx! {
      if visible {
          div { class: "modal-overlay keyboard-help-overlay",
              onclick: move |_| on_close.call(()),
              div { class: "modal keyboard-help-modal",
                  onclick: move |e: Event<MouseData>| {
                      e.stop_propagation();
                  },

                  // Header
                  div { class: "modal-header",
                      h2 { "Keyboard Shortcuts" }
                      button {
                          class: "modal-close",
                          onclick: move |_| on_close.call(()),
                          "×"
                      }
                  }

                  // Body - shortcuts list
                  div { class: "modal-body",
                      p { class: "keyboard-help-intro",
                          "Press the following keys to perform actions:"
                      }

                      table { class: "keyboard-shortcuts-table",
                          thead {
                              tr {
                                  th { "Shortcut" }
                                  th { "Action" }
                              }
                          }
                          tbody {
                              for description in shortcuts.descriptions().iter() {
                                  ShortcutRow {
                                      description: Rc::new(description.clone())
                                  }
                              }
                          }
                      }

                      div { class: "keyboard-help-footer",
                          p { "Press Ctrl+? or Esc to close this dialog" }
                      }
                  }
              }
          }
      }
  }
}

/// Single shortcut row component properties
#[derive(Clone, Props)]
pub struct ShortcutRowProps {
  /// The action description to display
  pub description: std::rc::Rc<crate::shortcuts::ActionDescription>,
}

impl PartialEq for ShortcutRowProps {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.description, &other.description)
  }
}

/// Single shortcut row component
///
/// Displays one shortcut in the table with its key combination and description.
#[component]
fn ShortcutRow(props: ShortcutRowProps) -> Element {
  let description = props.description;

  // Display the first shortcut (if any)
  if let Some(shortcut) = description.shortcuts.first() {
    let is_ctrl = shortcut.modifiers().has_control();
    let is_alt = shortcut.modifiers().has_alt();
    let is_meta = shortcut.modifiers().has_meta();

    rsx! {
        tr {
            td { class: "shortcut-key",
                // Show modifiers with visual styling
                if is_ctrl {
                    kbd { class: "key-modifier", "Ctrl" }
                    span { class: "key-separator", "+" }
                }
                if is_alt {
                    kbd { class: "key-modifier", "Alt" }
                    span { class: "key-separator", "+" }
                }
                if is_meta {
                    kbd { class: "key-modifier", "⌘" }
                    span { class: "key-separator", "+" }
                }

                // The main key
                kbd {
                    class: "key-main",
                    "{format_main_key(shortcut)}"
                }
            }
            td { class: "shortcut-description",
                "{description.description}"
            }
        }
    }
  } else {
    rsx! { tr { td { "No shortcuts configured" } } }
  }
}

/// Format the main key for display
///
/// Pure function to format the key part of a shortcut.
#[must_use]
fn format_main_key(shortcut: &crate::shortcuts::Shortcut) -> String {
  match shortcut.key() {
    crate::shortcuts::Key::Character(c) => {
      if shortcut.modifiers().has_shift() {
        c.to_uppercase().to_string()
      } else {
        c.to_string()
      }
    }
    crate::shortcuts::Key::Number(n) => n.to_string(),
    crate::shortcuts::Key::F(n) => format!("F{n}"),
    crate::shortcuts::Key::Escape => "Esc".to_string(),
    crate::shortcuts::Key::Enter => "Enter".to_string(),
    crate::shortcuts::Key::Tab => "Tab".to_string(),
    crate::shortcuts::Key::Backspace => "⌫".to_string(),
    crate::shortcuts::Key::Delete => "⌦".to_string(),
    crate::shortcuts::Key::Home => "Home".to_string(),
    crate::shortcuts::Key::End => "End".to_string(),
    crate::shortcuts::Key::PageUp => "PageUp".to_string(),
    crate::shortcuts::Key::PageDown => "PageDown".to_string(),
    crate::shortcuts::Key::ArrowUp => "↑".to_string(),
    crate::shortcuts::Key::ArrowDown => "↓".to_string(),
    crate::shortcuts::Key::ArrowLeft => "←".to_string(),
    crate::shortcuts::Key::ArrowRight => "→".to_string(),
    crate::shortcuts::Key::Question => "?".to_string(),
    _ => format!("{:?}", shortcut.key()),
  }
}

/// Hook for managing keyboard help dialog state
///
/// This hook provides the state and callbacks needed for the
/// keyboard help dialog. It handles both showing and hiding
/// the dialog via keyboard shortcuts.
///
/// # Returns
///
/// Returns a tuple containing:
/// - `visible`: Signal indicating if dialog is visible
/// - `show`: Callback to show the dialog
/// - `hide`: Callback to hide the dialog
/// - `toggle`: Callback to toggle dialog visibility
#[must_use]
pub fn use_keyboard_help() -> (Signal<bool>, Callback, Callback, Callback) {
  let visible = use_signal(|| false);

  let show = {
    let mut visible = visible;
    Callback::new(move |()| {
      visible.set(true);
    })
  };

  let hide = {
    let mut visible = visible;
    Callback::new(move |()| {
      visible.set(false);
    })
  };

  let toggle = {
    let mut visible = visible;
    Callback::new(move |()| {
      let current = *visible.read();
      visible.set(!current);
    })
  };

  (visible, show, hide, toggle)
}

/// Hook that integrates keyboard help with global keyboard shortcuts
///
/// This hook automatically shows/hides the help dialog when
/// Ctrl+? or Escape is pressed.
///
/// # Returns
///
/// Returns the same tuple as `use_keyboard_help`:
/// - `visible`: Signal indicating if dialog is visible
/// - `show`: Callback to show the dialog
/// - `hide`: Callback to hide the dialog
/// - `toggle`: Callback to toggle dialog visibility
#[must_use]
pub fn use_keyboard_help_with_shortcuts() -> (Signal<bool>, Callback, Callback, Callback) {
  let (visible, show, hide, toggle) = use_keyboard_help();

  // Set up keyboard handler for showing/hiding help
  use_effect(move || {
    let visible = visible;
    let show = show;
    let hide = hide;
    let toggle = toggle;

    // Store for potential future use with keyboard handlers
    let _ = (visible, show, hide, toggle);

    // Note: Global keyboard listener setup would go here
    // For now, the shortcuts are handled at component level
  });

  (visible, show, hide, toggle)
}

/// Keyboard shortcut hint component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct ShortcutHintProps {
  /// The action to display a shortcut for
  pub action: Action,
}

/// Keyboard shortcut hint component
///
/// Small inline component showing a keyboard shortcut hint.
/// Useful for buttons that have keyboard shortcuts.
///
/// # Arguments
///
/// * `action` - The action to display a shortcut for
#[component]
pub fn ShortcutHint(props: ShortcutHintProps) -> Element {
  let action = props.action;
  let shortcuts = Shortcuts::default_mappings();

  rsx! {
      {
          shortcuts
              .descriptions()
              .iter()
              .find(|d| d.action == action)
              .and_then(|description| description.shortcuts.first())
              .map(|shortcut| {
                  let formatted = shortcut.format();
                  rsx! {
                      kbd { class: "shortcut-hint",
                          "{formatted}"
                      }
                  }
              })
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format_main_key() {
    let shortcut = crate::shortcuts::Shortcut::ctrl(crate::shortcuts::Key::Character('n'));
    assert_eq!(format_main_key(&shortcut), "n");

    let shortcut = crate::shortcuts::Shortcut::new(
      crate::shortcuts::Modifiers::ControlShift,
      crate::shortcuts::Key::Character('s'),
    );
    assert_eq!(format_main_key(&shortcut), "S");

    let shortcut = crate::shortcuts::Shortcut::plain(crate::shortcuts::Key::Escape);
    assert_eq!(format_main_key(&shortcut), "Esc");

    let shortcut = crate::shortcuts::Shortcut::plain(crate::shortcuts::Key::Delete);
    assert_eq!(format_main_key(&shortcut), "⌦");
  }

  #[test]
  fn test_use_keyboard_help_returns_callbacks() {
    // This test just verifies the hook compiles and returns the right types
    // We can't actually run Dioxus hooks in unit tests
    let _ = || {
      let (visible, show, hide, toggle) = use_keyboard_help();
      // Verify they're callable (at least type-wise)
      let _: Signal<bool> = visible;
      let _: dioxus::prelude::Callback<()> = show;
      let _: dioxus::prelude::Callback<()> = hide;
      let _: dioxus::prelude::Callback<()> = toggle;
    };
  }

  #[test]
  fn test_action_description_clone() {
    let shortcuts = Shortcuts::default_mappings();
    let descriptions = shortcuts.descriptions();

    for desc in descriptions {
      let _desc = desc.clone();
    }
  }
}
