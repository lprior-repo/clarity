#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Keyboard event handling hook for Dioxus
//!
//! This module provides a pure keyboard event handling system that maps
//! browser keyboard events to application actions. It uses the shortcuts
//! module for pure shortcut definitions and provides reactive callbacks.

use crate::shortcuts::{Action, Key, Modifiers, Shortcut, Shortcuts};
use dioxus::prelude::*;

/// Result of a keyboard event match
///
/// Pure enum representing whether a keyboard event matched a shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchResult {
  /// Event matched a registered shortcut
  Matched(Action),
  /// Event did not match any shortcut
  NoMatch,
  /// Event was consumed (should not propagate)
  Consumed,
}

impl MatchResult {
  /// Check if the result was a match
  #[must_use]
  pub const fn is_matched(&self) -> bool {
    matches!(self, Self::Matched(_))
  }

  /// Check if the result was consumed
  #[must_use]
  pub const fn is_consumed(&self) -> bool {
    matches!(self, Self::Consumed)
  }

  /// Get the matched action if any
  #[must_use]
  pub const fn action(&self) -> Option<Action> {
    match self {
      Self::Matched(action) => Some(*action),
      _ => None,
    }
  }
}

/// Keyboard event data
///
/// Pure value extracted from keyboard events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
  /// The key that was pressed
  pub key: Key,
  /// Modifier state
  pub modifiers: Modifiers,
  /// Whether the event is repeat (key held down)
  pub is_repeat: bool,
}

impl KeyEvent {
  /// Create a new keyboard event
  #[must_use]
  pub const fn new(key: Key, modifiers: Modifiers, is_repeat: bool) -> Self {
    Self {
      key,
      modifiers,
      is_repeat,
    }
  }

  /// Convert to a shortcut
  ///
  /// Pure transformation to shortcut for matching.
  #[must_use]
  pub const fn as_shortcut(&self) -> Shortcut {
    Shortcut::new(self.modifiers, self.key)
  }

  /// Check if this is a repeat event
  #[must_use]
  pub const fn is_repeat_event(&self) -> bool {
    self.is_repeat
  }
}

/// Parse a key from a keyboard event code string
///
/// Pure function converting browser event data to Key enum.
#[must_use]
pub fn parse_key_from_event(key: &str, code: &str) -> Option<Key> {
  let key_lower = key.to_lowercase();

  // First try to match by key value (for character keys)
  match key_lower.as_str() {
    "escape" => return Some(Key::Escape),
    "enter" => return Some(Key::Enter),
    "tab" => return Some(Key::Tab),
    "backspace" => return Some(Key::Backspace),
    "delete" => return Some(Key::Delete),
    "home" => return Some(Key::Home),
    "end" => return Some(Key::End),
    "pageup" => return Some(Key::PageUp),
    "pagedown" => return Some(Key::PageDown),
    "arrowup" => return Some(Key::ArrowUp),
    "arrowdown" => return Some(Key::ArrowDown),
    "arrowleft" => return Some(Key::ArrowLeft),
    "arrowright" => return Some(Key::ArrowRight),
    "?" => return Some(Key::Question),
    _ => {}
  }

  // Handle character and number keys
  if key.len() == 1 {
    if let Some(ch) = key.chars().next() {
      if ch.is_alphabetic() {
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        return Some(Key::Character(lower));
      }
      if ch.is_ascii_digit() {
        return ch.to_digit(10).map(|d| Key::Number(d as u8));
      }
    }
  }

  // Otherwise, try to parse from code
  crate::shortcuts::parse_key(code)
}

/// Extract keyboard event data from web event
///
/// Pure function extracting `KeyEvent` from browser keyboard event data.
#[must_use]
pub fn extract_key_event(
  key: &str,
  code: &str,
  ctrl_key: bool,
  alt_key: bool,
  meta_key: bool,
  shift_key: bool,
  repeat: bool,
) -> Option<KeyEvent> {
  parse_key_from_event(key, code).map(|key| KeyEvent {
    key,
    modifiers: Modifiers::from_parts(ctrl_key, alt_key, meta_key, shift_key),
    is_repeat: repeat,
  })
}

/// Match a keyboard event against shortcuts
///
/// Pure function matching events to actions.
#[must_use]
pub fn match_shortcut(event: &KeyEvent, shortcuts: &Shortcuts) -> MatchResult {
  // Ignore repeat events for shortcuts
  if event.is_repeat_event() {
    return MatchResult::NoMatch;
  }

  let shortcut = event.as_shortcut();
  shortcuts
    .get_action(&shortcut)
    .map_or(MatchResult::NoMatch, MatchResult::Matched)
}

/// Action handler callback type
pub type ActionHandler = std::rc::Rc<dyn Fn(Action) + Send + Sync>;

/// Keyboard hook state
///
/// Immutable state for keyboard event handling.
#[derive(Clone)]
pub struct KeyboardState {
  /// Global shortcuts registry
  shortcuts: Shortcuts,
  /// Whether keyboard handling is enabled
  enabled: bool,
}

impl KeyboardState {
  /// Create a new keyboard state with default shortcuts
  #[must_use]
  pub fn new() -> Self {
    Self {
      shortcuts: Shortcuts::default_mappings(),
      enabled: true,
    }
  }

  /// Create keyboard state with custom shortcuts
  #[must_use]
  pub const fn with_shortcuts(shortcuts: Shortcuts) -> Self {
    Self {
      shortcuts,
      enabled: true,
    }
  }

  /// Disable keyboard handling
  #[must_use]
  pub const fn disabled(mut self) -> Self {
    self.enabled = false;
    self
  }

  /// Check if keyboard handling is enabled
  #[must_use]
  pub const fn is_enabled(&self) -> bool {
    self.enabled
  }

  /// Get the shortcuts registry
  #[must_use]
  pub const fn shortcuts(&self) -> &Shortcuts {
    &self.shortcuts
  }
}

impl Default for KeyboardState {
  fn default() -> Self {
    Self::new()
  }
}

/// Hook for handling keyboard shortcuts
///
/// This hook provides a callback for handling keyboard events
/// and dispatching to the appropriate action handler.
///
/// # Returns
///
/// Returns a tuple of:
/// - `on_key_down`: Event handler for keyboard events
/// - `action_handler`: Callback to register action handlers
#[must_use]
pub fn use_keyboard(
) -> std::rc::Rc<dyn Fn(String, String, bool, bool, bool, bool, bool) -> MatchResult + Send + Sync>
{
  let shortcuts = Shortcuts::default_mappings();

  (std::rc::Rc::new(
    move |key: String,
          code: String,
          ctrl_key: bool,
          alt_key: bool,
          meta_key: bool,
          shift_key: bool,
          repeat: bool| {
      match extract_key_event(&key, &code, ctrl_key, alt_key, meta_key, shift_key, repeat) {
        Some(event) => match_shortcut(&event, &shortcuts),
        None => MatchResult::NoMatch,
      }
    },
  )) as _
}

/// Hook for keyboard shortcuts with action handler
///
/// This hook provides both keyboard event handling and action dispatch.
/// When a shortcut is matched, the action handler is called.
///
/// # Arguments
///
/// * `action_handler` - Callback to invoke when an action is triggered
///
/// # Returns
///
/// Returns an event handler that can be attached to DOM elements
#[must_use]
pub fn use_keyboard_with_handler(
  action_handler: impl Fn(Action) + 'static,
) -> std::rc::Rc<dyn Fn(String, String, bool, bool, bool, bool, bool) -> MatchResult> {
  let shortcuts = Shortcuts::default_mappings();
  let handler = std::rc::Rc::new(action_handler);

  (std::rc::Rc::new(
    move |key: String,
          code: String,
          ctrl_key: bool,
          alt_key: bool,
          meta_key: bool,
          shift_key: bool,
          repeat: bool| {
      match extract_key_event(&key, &code, ctrl_key, alt_key, meta_key, shift_key, repeat) {
        Some(event) => {
          let result = match_shortcut(&event, &shortcuts);
          if let MatchResult::Matched(action) = result {
            handler(action);
            MatchResult::Matched(action)
          } else {
            result
          }
        }
        None => MatchResult::NoMatch,
      }
    },
  )) as _
}

/// Hook for global keyboard shortcuts
///
/// This hook sets up a global keyboard listener that captures
/// keyboard events at the window level. The handler is called
/// for every keyboard event.
///
/// # Arguments
///
/// * `action_handler` - Callback to invoke when an action is triggered
pub fn use_global_keyboard(action_handler: impl Fn(Action) + 'static) {
  let handler = use_keyboard_with_handler(action_handler);

  use_effect(move || {
    let _handler = handler.clone();
    // Set up window-level keyboard event listener
    // This would be integrated with Dioxus's event system
    // For now, this is a placeholder
  });
}

/// Hook to check if a keyboard event matches a specific action
///
/// This is useful for showing shortcut hints in UI elements.
#[must_use]
pub fn use_shortcut_for_action(action: Action) -> Option<Shortcut> {
  let shortcuts = Shortcuts::default_mappings();
  shortcuts
    .descriptions()
    .iter()
    .find(|d| d.action == action)
    .map(|d| d.shortcut.clone())
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  use super::*;

  #[test]
  fn test_parse_key_from_event() {
    assert_eq!(parse_key_from_event("n", "KeyN"), Some(Key::Character('n')));
    assert_eq!(parse_key_from_event("N", "KeyN"), Some(Key::Character('n')));
    assert_eq!(parse_key_from_event("Escape", "Escape"), Some(Key::Escape));
    assert_eq!(parse_key_from_event("Enter", "Enter"), Some(Key::Enter));
    assert_eq!(parse_key_from_event("Delete", "Delete"), Some(Key::Delete));
    assert_eq!(parse_key_from_event("?", "Slash"), Some(Key::Question));
    assert_eq!(parse_key_from_event("5", "Digit5"), Some(Key::Number(5)));
  }

  #[test]
  fn test_extract_key_event() {
    let event = extract_key_event("n", "KeyN", true, false, false, false, false);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.key, Key::Character('n'));
    assert_eq!(event.modifiers, Modifiers::Control);
    assert!(!event.is_repeat);
  }

  #[test]
  fn test_key_event_as_shortcut() {
    let event = KeyEvent::new(Key::Character('n'), Modifiers::Control, false);
    let shortcut = event.as_shortcut();
    assert_eq!(shortcut, Shortcut::ctrl(Key::Character('n')));
  }

  #[test]
  fn test_match_shortcut() {
    let shortcuts = Shortcuts::default_mappings();

    // Match Ctrl+N
    let event = KeyEvent::new(Key::Character('n'), Modifiers::Control, false);
    assert_eq!(
      match_shortcut(&event, &shortcuts),
      MatchResult::Matched(Action::NewBead)
    );

    // No match for Ctrl+X (undefined)
    let event = KeyEvent::new(Key::Character('x'), Modifiers::Control, false);
    assert_eq!(match_shortcut(&event, &shortcuts), MatchResult::NoMatch);

    // Ignore repeat events
    let event = KeyEvent::new(Key::Character('n'), Modifiers::Control, true);
    assert_eq!(match_shortcut(&event, &shortcuts), MatchResult::NoMatch);
  }

  #[test]
  fn test_match_result() {
    let matched = MatchResult::Matched(Action::NewBead);
    assert!(matched.is_matched());
    assert!(!matched.is_consumed());
    assert_eq!(matched.action(), Some(Action::NewBead));

    let no_match = MatchResult::NoMatch;
    assert!(!no_match.is_matched());
    assert!(!no_match.is_consumed());
    assert_eq!(no_match.action(), None);

    let consumed = MatchResult::Consumed;
    assert!(!consumed.is_matched());
    assert!(consumed.is_consumed());
    assert_eq!(consumed.action(), None);
  }

  #[test]
  fn test_keyboard_state() {
    let state = KeyboardState::new();
    assert!(state.is_enabled());

    let state = state.disabled();
    assert!(!state.is_enabled());
  }

  #[test]
  fn test_use_shortcut_for_action() {
    let shortcut = use_shortcut_for_action(Action::NewBead);
    assert!(shortcut.is_some());
    assert_eq!(shortcut.unwrap(), Shortcut::ctrl(Key::Character('n')));

    let shortcut = use_shortcut_for_action(Action::FocusSearch);
    assert!(shortcut.is_some());
    assert_eq!(shortcut.unwrap(), Shortcut::ctrl(Key::Character('f')));
  }

  #[test]
  fn test_all_defined_shortcuts() {
    let shortcuts = Shortcuts::default_mappings();

    // Test that all actions have shortcuts
    let actions = vec![
      Action::NewBead,
      Action::FocusSearch,
      Action::SaveForm,
      Action::Cancel,
      Action::ShowHelp,
      Action::DeleteBead,
      Action::Undo,
      Action::Redo,
    ];

    for action in actions {
      let found = shortcuts.descriptions().iter().any(|d| d.action == action);
      assert!(found, "Action {:?} should have a shortcut defined", action);
    }
  }

  #[test]
  fn test_key_event_repeat_detection() {
    let event = KeyEvent::new(Key::Escape, Modifiers::None, true);
    assert!(event.is_repeat_event());

    let event = KeyEvent::new(Key::Escape, Modifiers::None, false);
    assert!(!event.is_repeat_event());
  }

  #[test]
  fn test_modifiers_combinations() {
    // Test all modifier combinations are correctly formed
    assert_eq!(
      Modifiers::from_parts(true, false, false, false),
      Modifiers::Control
    );
    assert_eq!(
      Modifiers::from_parts(false, true, false, false),
      Modifiers::Alt
    );
    assert_eq!(
      Modifiers::from_parts(false, false, true, false),
      Modifiers::Meta
    );
    assert_eq!(
      Modifiers::from_parts(false, false, false, true),
      Modifiers::Shift
    );
    assert_eq!(
      Modifiers::from_parts(true, false, false, true),
      Modifiers::ControlShift
    );
  }
}
