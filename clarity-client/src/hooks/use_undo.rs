#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Undo/redo hook for Dioxus components
//!
//! This hook provides undo/redo functionality with reactive state management.

use dioxus::prelude::*;

/// Hook for undo/redo functionality
///
/// Returns:
/// - `can_undo`: Whether undo is available
/// - `can_redo`: Whether redo is available
/// - `undo`: Callback to perform undo
/// - `redo`: Callback to perform redo
/// - `clear`: Callback to clear history
/// - `peek_undo`: Description of next undo action
/// - `peek_redo`: Description of next redo action
/// - `undo_count`: Number of undoable actions
/// - `redo_count`: Number of redoable actions
#[must_use]
pub fn use_undo() -> (
  bool,
  bool,
  Callback,
  Callback,
  Callback,
  Option<String>,
  Option<String>,
  usize,
  usize,
) {
  let mut undo_stack = use_signal(crate::undo::UndoStack::new);
  let mut error = use_signal(|| Option::<String>::None);

  let can_undo = undo_stack.read().can_undo();
  let can_redo = undo_stack.read().can_redo();
  let undo_count = undo_stack.read().undo_count();
  let redo_count = undo_stack.read().redo_count();
  let peek_undo = undo_stack.read().peek_undo();
  let peek_redo = undo_stack.read().peek_redo();

  // Undo callback
  let undo = Callback::new(move |()| {
    error.set(None);

    let result = undo_stack
      .read()
      .undo()
      .map_err(|e| format!("Undo failed: {e}"));

    match &result {
      Ok((new_stack, _)) => {
        *undo_stack.write() = new_stack.clone();
      }
      Err(e) => {
        error.set(Some(e.clone()));
      }
    }
  });

  // Redo callback
  let redo = Callback::new(move |()| {
    error.set(None);

    let result = undo_stack
      .read()
      .redo()
      .map_err(|e| format!("Redo failed: {e}"));

    match &result {
      Ok((new_stack, _)) => {
        *undo_stack.write() = new_stack.clone();
      }
      Err(e) => {
        error.set(Some(e.clone()));
      }
    }
  });

  // Clear callback
  let clear = Callback::new(move |()| {
    let cleared = undo_stack.read().clear();
    *undo_stack.write() = cleared;
    error.set(None);
  });

  (
    can_undo, can_redo, undo, redo, clear, peek_undo, peek_redo, undo_count, redo_count,
  )
}

/// Hook to access the global undo stack
///
/// This provides access to the undo stack for pushing commands.
#[must_use]
pub fn use_undo_stack() -> Signal<crate::undo::UndoStack> {
  use_context::<UndoStackProvider>()
    .stack
    .unwrap_or_else(|| Signal::new(crate::undo::UndoStack::new()))
}

/// Provider for global undo stack
#[derive(Clone)]
pub struct UndoStackProvider {
  stack: Option<Signal<crate::undo::UndoStack>>,
}

impl UndoStackProvider {
  /// Create a new undo stack provider
  #[must_use]
  pub fn new() -> Self {
    Self {
      stack: Some(Signal::new(crate::undo::UndoStack::new())),
    }
  }
}

impl Default for UndoStackProvider {
  fn default() -> Self {
    Self::new()
  }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
  use crate::undo::UndoStack;

  #[test]
  fn test_undo_stack_default() {
    let stack = UndoStack::new();
    assert!(!stack.can_undo());
    assert!(!stack.can_redo());
    assert_eq!(stack.undo_count(), 0);
    assert_eq!(stack.redo_count(), 0);
  }
}
