#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Keyboard shortcut definitions for Clarity desktop app
//!
//! This module provides a pure, type-safe keyboard shortcut system.
//! Shortcuts are defined as immutable data structures with combinators
//! for composition and matching.

use std::collections::HashMap;

/// Represents a keyboard key
///
/// Pure enum for keyboard keys without any runtime state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
  /// Letter keys
  Character(char),
  /// Number keys
  Number(u8),
  /// Function keys
  F(u8),
  /// Special keys
  Escape,
  Enter,
  Tab,
  Backspace,
  Delete,
  Home,
  End,
  PageUp,
  PageDown,
  /// Arrow keys
  ArrowUp,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  /// Modifier-only representation (for parsing)
  Control,
  Alt,
  Meta,
  Shift,
  /// Question mark key
  Question,
}

/// Keyboard modifier keys
///
/// Represents modifier state in a pure, immutable way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modifiers {
  /// No modifiers
  None,
  /// Control key only
  Control,
  /// Alt/Option key only
  Alt,
  /// Meta/Command key only
  Meta,
  /// Shift key only
  Shift,
  /// Control + Shift
  ControlShift,
  /// Alt + Shift
  AltShift,
  /// Meta + Shift
  MetaShift,
  /// Control + Alt
  ControlAlt,
  /// Control + Meta
  ControlMeta,
  /// Alt + Meta
  AltMeta,
  /// Control + Alt + Shift
  ControlAltShift,
  /// Control + Meta + Shift
  ControlMetaShift,
  /// Alt + Meta + Shift
  AltMetaShift,
  /// All modifiers
  All,
}

impl Modifiers {
  /// Parse modifiers from a keyboard event state
  ///
  /// Pure function converting event state to modifier enum.
  #[must_use]
  pub const fn from_parts(ctrl: bool, alt: bool, meta: bool, shift: bool) -> Self {
    match (ctrl, alt, meta, shift) {
      (false, false, false, false) => Self::None,
      (true, false, false, false) => Self::Control,
      (false, true, false, false) => Self::Alt,
      (false, false, true, false) => Self::Meta,
      (false, false, false, true) => Self::Shift,
      (true, false, false, true) => Self::ControlShift,
      (false, true, false, true) => Self::AltShift,
      (false, false, true, true) => Self::MetaShift,
      (true, true, false, false) => Self::ControlAlt,
      (true, false, true, false) => Self::ControlMeta,
      (false, true, true, false) => Self::AltMeta,
      (true, true, false, true) => Self::ControlAltShift,
      (true, false, true, true) => Self::ControlMetaShift,
      (false, true, true, true) => Self::AltMetaShift,
      (true, true, true, _) => Self::All,
    }
  }

  /// Check if Control key is pressed
  #[must_use]
  pub const fn has_control(self) -> bool {
    matches!(
      self,
      Self::Control
        | Self::ControlShift
        | Self::ControlAlt
        | Self::ControlMeta
        | Self::ControlAltShift
        | Self::ControlMetaShift
        | Self::All
    )
  }

  /// Check if Alt key is pressed
  #[must_use]
  pub const fn has_alt(self) -> bool {
    matches!(
      self,
      Self::Alt
        | Self::AltShift
        | Self::ControlAlt
        | Self::AltMeta
        | Self::ControlAltShift
        | Self::All
    )
  }

  /// Check if Meta key is pressed
  #[must_use]
  pub const fn has_meta(self) -> bool {
    matches!(
      self,
      Self::Meta
        | Self::MetaShift
        | Self::ControlMeta
        | Self::AltMeta
        | Self::ControlMetaShift
        | Self::All
    )
  }

  /// Check if Shift key is pressed
  #[must_use]
  pub const fn has_shift(self) -> bool {
    matches!(
      self,
      Self::Shift
        | Self::ControlShift
        | Self::AltShift
        | Self::MetaShift
        | Self::ControlAltShift
        | Self::ControlMetaShift
        | Self::All
    )
  }
}

/// A keyboard shortcut combination
///
/// Immutable value representing a complete keyboard shortcut.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
  modifiers: Modifiers,
  key: Key,
}

impl Shortcut {
  /// Create a new shortcut
  #[must_use]
  pub const fn new(modifiers: Modifiers, key: Key) -> Self {
    Self { modifiers, key }
  }

  /// Create a Control+key shortcut
  #[must_use]
  pub const fn ctrl(key: Key) -> Self {
    Self {
      modifiers: Modifiers::Control,
      key,
    }
  }

  /// Create a plain key shortcut (no modifiers)
  #[must_use]
  pub const fn plain(key: Key) -> Self {
    Self {
      modifiers: Modifiers::None,
      key,
    }
  }

  /// Get the modifiers
  #[must_use]
  pub const fn modifiers(&self) -> Modifiers {
    self.modifiers
  }

  /// Get the key
  #[must_use]
  pub const fn key(&self) -> &Key {
    &self.key
  }

  /// Format shortcut for display
  ///
  /// Pure transformation to human-readable string.
  #[must_use]
  pub fn format(&self) -> String {
    let mut parts = Vec::new();

    if self.modifiers.has_control() {
      parts.push("Ctrl");
    }
    if self.modifiers.has_alt() {
      parts.push("Alt");
    }
    if self.modifiers.has_meta() {
      parts.push("Meta");
    }
    if self.modifiers.has_shift() && !matches!(self.key, Key::Character(_)) {
      parts.push("Shift");
    }

    let key_str = match self.key {
      Key::Character(c) => {
        if self.modifiers.has_shift() {
          c.to_uppercase().to_string()
        } else {
          c.to_string()
        }
      }
      Key::Number(n) => n.to_string(),
      Key::F(n) => format!("F{n}"),
      Key::Escape => "Esc".to_string(),
      Key::Enter => "Enter".to_string(),
      Key::Tab => "Tab".to_string(),
      Key::Backspace => "Backspace".to_string(),
      Key::Delete => "Delete".to_string(),
      Key::Home => "Home".to_string(),
      Key::End => "End".to_string(),
      Key::PageUp => "PageUp".to_string(),
      Key::PageDown => "PageDown".to_string(),
      Key::ArrowUp => "↑".to_string(),
      Key::ArrowDown => "↓".to_string(),
      Key::ArrowLeft => "←".to_string(),
      Key::ArrowRight => "→".to_string(),
      Key::Control => "Ctrl".to_string(),
      Key::Alt => "Alt".to_string(),
      Key::Meta => "Meta".to_string(),
      Key::Shift => "Shift".to_string(),
      Key::Question => "?".to_string(),
    };

    parts.push(&key_str);
    parts.join("+")
  }
}

/// Application action that can be triggered by a shortcut
///
/// Pure enum representing all keyboard-activated actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
  /// Create a new bead
  NewBead,
  /// Focus search input
  FocusSearch,
  /// Save current form
  SaveForm,
  /// Cancel current operation or clear input
  Cancel,
  /// Show keyboard shortcuts help
  ShowHelp,
  /// Delete selected bead
  DeleteBead,
  /// Undo last action
  Undo,
  /// Redo last undone action
  Redo,
}

/// Action description for help display
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionDescription {
  /// The action enum
  pub action: Action,
  /// Human-readable description
  pub description: &'static str,
  /// Keyboard shortcuts (can have multiple)
  pub shortcuts: Vec<Shortcut>,
}

impl ActionDescription {
  /// Create a new action description with single shortcut
  #[must_use]
  pub fn new(action: Action, description: &'static str, shortcut: Shortcut) -> Self {
    Self {
      action,
      description,
      shortcuts: vec![shortcut],
    }
  }

  /// Create a new action description with multiple shortcuts
  #[must_use]
  pub const fn new_multiple(
    action: Action,
    description: &'static str,
    shortcuts: Vec<Shortcut>,
  ) -> Self {
    Self {
      action,
      description,
      shortcuts,
    }
  }
}

/// Global keyboard shortcuts registry
///
/// Pure mapping of shortcuts to actions. This is immutable
/// and can be freely shared across the application.
#[derive(Clone, Debug)]
pub struct Shortcuts {
  /// Map from shortcut to action
  shortcuts: HashMap<Shortcut, Action>,
  /// Map from action to description
  descriptions: Vec<ActionDescription>,
}

impl Shortcuts {
  /// Create the default shortcuts registry
  ///
  /// Pure function returning the complete shortcut mappings.
  #[must_use]
  pub fn default_mappings() -> Self {
    let mut shortcuts = HashMap::new();
    let descriptions = vec![
      ActionDescription::new(
        Action::NewBead,
        "Create a new bead",
        Shortcut::ctrl(Key::Character('n')),
      ),
      ActionDescription::new(
        Action::FocusSearch,
        "Focus search input",
        Shortcut::ctrl(Key::Character('f')),
      ),
      ActionDescription::new_multiple(
        Action::SaveForm,
        "Save current form",
        vec![
          Shortcut::ctrl(Key::Character('s')),
          Shortcut::new(Modifiers::Meta, Key::Enter),
        ],
      ),
      ActionDescription::new(
        Action::Cancel,
        "Cancel or clear",
        Shortcut::plain(Key::Escape),
      ),
      ActionDescription::new(
        Action::ShowHelp,
        "Show keyboard shortcuts",
        Shortcut::ctrl(Key::Question),
      ),
      ActionDescription::new(
        Action::DeleteBead,
        "Delete selected bead",
        Shortcut::plain(Key::Delete),
      ),
      ActionDescription::new(
        Action::Undo,
        "Undo last action",
        Shortcut::ctrl(Key::Character('z')),
      ),
      ActionDescription::new(
        Action::Redo,
        "Redo last undone action",
        Shortcut::ctrl(Key::Character('y')),
      ),
    ];

    for desc in &descriptions {
      for shortcut in &desc.shortcuts {
        shortcuts.insert(shortcut.clone(), desc.action);
      }
    }

    Self {
      shortcuts,
      descriptions,
    }
  }

  /// Look up an action by shortcut
  ///
  /// Pure lookup returning None if no action is bound.
  #[must_use]
  pub fn get_action(&self, shortcut: &Shortcut) -> Option<Action> {
    self.shortcuts.get(shortcut).copied()
  }

  /// Get all action descriptions
  ///
  /// Pure accessor returning immutable descriptions.
  #[must_use]
  pub fn descriptions(&self) -> &[ActionDescription] {
    &self.descriptions
  }

  /// Check if a shortcut matches any registered action
  #[must_use]
  pub fn is_registered(&self, shortcut: &Shortcut) -> bool {
    self.shortcuts.contains_key(shortcut)
  }
}

impl Default for Shortcuts {
  fn default() -> Self {
    Self::default_mappings()
  }
}

/// Parse a key from a string
///
/// Pure function for parsing key names.
#[must_use]
pub fn parse_key(key_str: &str) -> Option<Key> {
  let key_str = key_str.to_lowercase();

  match key_str.as_str() {
    "escape" | "esc" => Some(Key::Escape),
    "enter" | "return" => Some(Key::Enter),
    "tab" => Some(Key::Tab),
    "backspace" => Some(Key::Backspace),
    "delete" | "del" => Some(Key::Delete),
    "home" => Some(Key::Home),
    "end" => Some(Key::End),
    "pageup" => Some(Key::PageUp),
    "pagedown" => Some(Key::PageDown),
    "arrowup" | "up" => Some(Key::ArrowUp),
    "arrowdown" | "down" => Some(Key::ArrowDown),
    "arrowleft" | "left" => Some(Key::ArrowLeft),
    "arrowright" | "right" => Some(Key::ArrowRight),
    "?" | "question" => Some(Key::Question),
    "control" | "ctrl" => Some(Key::Control),
    "alt" | "option" => Some(Key::Alt),
    "meta" | "command" | "cmd" | "super" | "win" => Some(Key::Meta),
    "shift" => Some(Key::Shift),
    s if s.len() == 1 && s.chars().next().is_some_and(char::is_alphabetic) => {
      s.chars().next().map(Key::Character)
    }
    s if s.len() == 1 && s.chars().next().is_some_and(|c| c.is_ascii_digit()) => s
      .chars()
      .next()
      .and_then(|c| c.to_digit(10))
      .map(|d| Key::Number(d as u8)),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_modifiers_from_parts() {
    assert_eq!(
      Modifiers::from_parts(false, false, false, false),
      Modifiers::None
    );
    assert_eq!(
      Modifiers::from_parts(true, false, false, false),
      Modifiers::Control
    );
    assert_eq!(
      Modifiers::from_parts(true, false, false, true),
      Modifiers::ControlShift
    );
  }

  #[test]
  fn test_modifiers_has_control() {
    assert!(!Modifiers::None.has_control());
    assert!(Modifiers::Control.has_control());
    assert!(Modifiers::ControlShift.has_control());
    assert!(!Modifiers::Shift.has_control());
  }

  #[test]
  fn test_shortcut_format() {
    let shortcut = Shortcut::ctrl(Key::Character('n'));
    assert_eq!(shortcut.format(), "Ctrl+n");

    // For Character keys with Shift, the Shift is applied by uppercasing the character
    let shortcut = Shortcut::new(Modifiers::ControlShift, Key::Character('s'));
    assert_eq!(shortcut.format(), "Ctrl+S");

    let shortcut = Shortcut::plain(Key::Escape);
    assert_eq!(shortcut.format(), "Esc");

    let shortcut = Shortcut::plain(Key::Delete);
    assert_eq!(shortcut.format(), "Delete");
  }

  #[test]
  fn test_shortcut_equality() {
    let s1 = Shortcut::ctrl(Key::Character('n'));
    let s2 = Shortcut::ctrl(Key::Character('n'));
    assert_eq!(s1, s2);

    let s3 = Shortcut::ctrl(Key::Character('s'));
    assert_ne!(s1, s3);
  }

  #[test]
  fn test_shortcuts_default_mappings() {
    let shortcuts = Shortcuts::default_mappings();

    // Test all defined shortcuts
    assert_eq!(
      shortcuts.get_action(&Shortcut::ctrl(Key::Character('n'))),
      Some(Action::NewBead)
    );
    assert_eq!(
      shortcuts.get_action(&Shortcut::ctrl(Key::Character('f'))),
      Some(Action::FocusSearch)
    );
    assert_eq!(
      shortcuts.get_action(&Shortcut::ctrl(Key::Character('s'))),
      Some(Action::SaveForm)
    );
    assert_eq!(
      shortcuts.get_action(&Shortcut::plain(Key::Escape)),
      Some(Action::Cancel)
    );
    assert_eq!(
      shortcuts.get_action(&Shortcut::ctrl(Key::Question)),
      Some(Action::ShowHelp)
    );
    assert_eq!(
      shortcuts.get_action(&Shortcut::plain(Key::Delete)),
      Some(Action::DeleteBead)
    );
    assert_eq!(
      shortcuts.get_action(&Shortcut::ctrl(Key::Character('z'))),
      Some(Action::Undo)
    );
    assert_eq!(
      shortcuts.get_action(&Shortcut::ctrl(Key::Character('y'))),
      Some(Action::Redo)
    );

    // Test undefined shortcut
    assert_eq!(
      shortcuts.get_action(&Shortcut::ctrl(Key::Character('x'))),
      None
    );
  }

  #[test]
  fn test_shortcuts_descriptions() {
    let shortcuts = Shortcuts::default_mappings();
    let descriptions = shortcuts.descriptions();

    assert!(!descriptions.is_empty());
    assert!(descriptions.iter().any(|d| d.action == Action::NewBead));
    assert!(descriptions.iter().any(|d| d.action == Action::ShowHelp));
  }

  #[test]
  fn test_parse_key() {
    assert_eq!(parse_key("escape"), Some(Key::Escape));
    assert_eq!(parse_key("esc"), Some(Key::Escape));
    assert_eq!(parse_key("Enter"), Some(Key::Enter));
    assert_eq!(parse_key("DELETE"), Some(Key::Delete));
    assert_eq!(parse_key("a"), Some(Key::Character('a')));
    assert_eq!(parse_key("Z"), Some(Key::Character('z')));
    assert_eq!(parse_key("5"), Some(Key::Number(5)));
    assert_eq!(parse_key("arrowup"), Some(Key::ArrowUp));
    assert_eq!(parse_key("?"), Some(Key::Question));
    assert_eq!(parse_key("invalid"), None);
  }

  #[test]
  fn test_action_description() {
    let desc = ActionDescription::new(
      Action::NewBead,
      "Create a new bead",
      Shortcut::ctrl(Key::Character('n')),
    );

    assert_eq!(desc.action, Action::NewBead);
    assert_eq!(desc.description, "Create a new bead");
    assert_eq!(desc.shortcut, Shortcut::ctrl(Key::Character('n')));
  }

  #[test]
  fn test_action_copy() {
    let action = Action::NewBead;
    let action_copy = action;
    assert_eq!(action, action_copy);
  }
}
