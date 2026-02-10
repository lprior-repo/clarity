#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Undo/redo functionality for bead operations
//!
//! This module implements the Command pattern for reversible bead operations.
//! All commands are pure and return Results for error handling.

use clarity_core::db::models::{Bead, BeadId, NewBead};
use clarity_core::db::DbResult;
use rpds::Vector;
use std::rc::Rc;

// ===== Command Trait =====

/// Command trait for undoable operations
///
/// All commands must implement execute and undo methods.
/// Each method returns a Result with a description of the action.
pub trait Command {
  /// Execute the command
  ///
  /// # Errors
  /// Returns a description if execution fails
  fn execute(&self) -> DbResult<String>;

  /// Undo the command
  ///
  /// # Errors
  /// Returns a description if undo fails
  fn undo(&self) -> DbResult<String>;

  /// Get a description of this command
  #[must_use]
  fn describe(&self) -> String;
}

// ===== Bead Commands =====

/// Command to create a new bead
#[derive(Clone)]
pub struct CreateBeadCommand {
  /// Database accessor (via Rc for shared ownership)
  db: Rc<dyn DatabaseAccess>,
  /// The bead to create
  bead: NewBead,
  /// The created bead (set after execution)
  created_bead: Option<Rc<Bead>>,
}

impl PartialEq for CreateBeadCommand {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.db, &other.db) &&
    self.bead.title == other.bead.title &&
    self.bead.description == other.bead.description &&
    self.bead.status == other.bead.status &&
    self.bead.priority == other.bead.priority &&
    self.bead.bead_type == other.bead.bead_type &&
    match (&self.created_bead, &other.created_bead) {
      (None, None) => true,
      (Some(a), Some(b)) => Rc::ptr_eq(a, b),
      _ => false,
    }
  }
}

impl CreateBeadCommand {
  /// Create a new command for bead creation
  #[must_use]
  pub const fn new(db: Rc<dyn DatabaseAccess>, bead: NewBead) -> Self {
    Self {
      db,
      bead,
      created_bead: None,
    }
  }
}

impl Command for CreateBeadCommand {
  fn execute(&self) -> DbResult<String> {
    let bead = self.db.create_bead(self.bead.clone())?;
    Ok(format!("Created bead: {}", bead.title))
  }

  fn undo(&self) -> DbResult<String> {
    // We need the created bead ID - this is a limitation of the pure approach
    // In practice, we'd need to store the created bead
    Err(clarity_core::db::error::DbError::Validation(
      "Cannot undo create: bead ID not stored".to_string(),
    ))
  }

  fn describe(&self) -> String {
    format!("Create bead: {}", self.bead.title)
  }
}

/// Command to update an existing bead
#[derive(Clone)]
pub struct UpdateBeadCommand {
  /// Database accessor
  db: Rc<dyn DatabaseAccess>,
  /// Bead ID to update
  id: BeadId,
  /// Previous bead state (for undo)
  old: Rc<Bead>,
  /// New bead data
  new: NewBead,
}

impl PartialEq for UpdateBeadCommand {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id &&
    Rc::ptr_eq(&self.db, &other.db) &&
    Rc::ptr_eq(&self.old, &other.old) &&
    self.new.title == other.new.title &&
    self.new.description == other.new.description &&
    self.new.status == other.new.status &&
    self.new.priority == other.new.priority &&
    self.new.bead_type == other.new.bead_type
  }
}

impl UpdateBeadCommand {
  /// Create a new command for bead update
  #[must_use]
  pub const fn new(db: Rc<dyn DatabaseAccess>, id: BeadId, old: Rc<Bead>, new: NewBead) -> Self {
    Self { db, id, old, new }
  }
}

impl Command for UpdateBeadCommand {
  fn execute(&self) -> DbResult<String> {
    let updated = self.db.update_bead(self.id, self.new.clone())?;
    Ok(format!("Updated bead: {}", updated.title))
  }

  fn undo(&self) -> DbResult<String> {
    let restored = self
      .db
      .update_bead(self.id, bead_to_new_bead(self.old.as_ref()))?;
    Ok(format!("Restored bead: {}", restored.title))
  }

  fn describe(&self) -> String {
    format!("Update bead: {}", self.old.title)
  }
}

/// Command to delete a bead
#[derive(Clone)]
pub struct DeleteBeadCommand {
  /// Database accessor
  db: Rc<dyn DatabaseAccess>,
  /// The bead being deleted (stored for undo)
  bead: Rc<Bead>,
}

impl PartialEq for DeleteBeadCommand {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.db, &other.db) &&
    Rc::ptr_eq(&self.bead, &other.bead)
  }
}

impl DeleteBeadCommand {
  /// Create a new command for bead deletion
  #[must_use]
  pub const fn new(db: Rc<dyn DatabaseAccess>, bead: Rc<Bead>) -> Self {
    Self { db, bead }
  }
}

impl Command for DeleteBeadCommand {
  fn execute(&self) -> DbResult<String> {
    self.db.delete_bead(self.bead.id)?;
    Ok(format!("Deleted bead: {}", self.bead.title))
  }

  fn undo(&self) -> DbResult<String> {
    let restored = self.db.create_bead(bead_to_new_bead(self.bead.as_ref()))?;
    Ok(format!("Restored deleted bead: {}", restored.title))
  }

  fn describe(&self) -> String {
    format!("Delete bead: {}", self.bead.title)
  }
}

// ===== Database Access Abstraction =====

/// Trait for database operations used by commands
///
/// This abstraction allows commands to work with any database implementation.
pub trait DatabaseAccess {
  /// Create a new bead
  ///
  /// # Errors
  /// Returns database error if creation fails
  fn create_bead(&self, bead: NewBead) -> DbResult<Bead>;

  /// Update an existing bead
  ///
  /// # Errors
  /// Returns database error if bead not found or update fails
  fn update_bead(&self, id: BeadId, bead: NewBead) -> DbResult<Bead>;

  /// Delete a bead
  ///
  /// # Errors
  /// Returns database error if bead not found or deletion fails
  fn delete_bead(&self, id: BeadId) -> DbResult<()>;
}

// ===== Undo Stack Manager =====

/// Manager for undo/redo command history
///
/// Uses persistent vectors for structural sharing of command history.
#[derive(Clone)]
pub struct UndoStack {
  /// Stack of executed commands (newest at end)
  undo_stack: Vector<Rc<dyn Command>>,
  /// Stack of undone commands (newest at end)
  redo_stack: Vector<Rc<dyn Command>>,
  /// Maximum stack size (0 = unlimited)
  max_size: usize,
}

impl PartialEq for UndoStack {
  fn eq(&self, other: &Self) -> bool {
    self.max_size == other.max_size &&
    self.undo_stack.len() == other.undo_stack.len() &&
    self.redo_stack.len() == other.redo_stack.len() &&
    self.undo_stack.iter().zip(other.undo_stack.iter()).all(|(a, b)| Rc::ptr_eq(a, b)) &&
    self.redo_stack.iter().zip(other.redo_stack.iter()).all(|(a, b)| Rc::ptr_eq(a, b))
  }
}

impl UndoStack {
  /// Create a new undo stack
  #[must_use]
  pub fn new() -> Self {
    Self {
      undo_stack: Vector::new(),
      redo_stack: Vector::new(),
      max_size: 100,
    }
  }

  /// Create an undo stack with custom max size
  #[must_use]
  pub fn with_max_size(max_size: usize) -> Self {
    Self {
      undo_stack: Vector::new(),
      redo_stack: Vector::new(),
      max_size,
    }
  }

  /// Push a new command onto the undo stack
  ///
  /// Clears the redo stack as new actions invalidate redo history.
  #[must_use]
  pub fn push_command(&self, command: Rc<dyn Command>) -> Self {
    let undo_stack = self.undo_stack.push_back(command);

    // Limit stack size if needed
    let undo_stack = if self.max_size > 0 && undo_stack.len() > self.max_size {
      // Remove oldest command from front
      undo_stack.iter().skip(1).cloned().collect::<Vector<_>>()
    } else {
      undo_stack
    };

    Self {
      undo_stack,
      redo_stack: Vector::new(), // Clear redo on new action
      max_size: self.max_size,
    }
  }

  /// Undo the most recent command
  ///
  /// # Errors
  /// Returns error if undo stack is empty or command fails
  pub fn undo(&self) -> DbResult<(Self, Option<String>)> {
    if self.undo_stack.is_empty() {
      return Ok((self.clone(), None));
    }

    // Get the most recent command
    let len = self.undo_stack.len();
    let command = self
      .undo_stack
      .get(len - 1)
      .ok_or_else(|| {
        clarity_core::db::error::DbError::Validation("Undo stack corrupted".to_string())
      })?
      .clone();

    // Execute undo
    let message = command.undo()?;

    // Move command from undo to redo stack
    let undo_stack = self.undo_stack.iter().take(self.undo_stack.len() - 1).cloned().collect::<Vector<_>>();
    let redo_stack = self.redo_stack.push_back(command);

    Ok((
      Self {
        undo_stack,
        redo_stack,
        max_size: self.max_size,
      },
      Some(message),
    ))
  }

  /// Redo the most recently undone command
  ///
  /// # Errors
  /// Returns error if redo stack is empty or command fails
  pub fn redo(&self) -> DbResult<(Self, Option<String>)> {
    if self.redo_stack.is_empty() {
      return Ok((self.clone(), None));
    }

    // Get the most recent undone command
    let len = self.redo_stack.len();
    let command = self
      .redo_stack
      .get(len - 1)
      .ok_or_else(|| {
        clarity_core::db::error::DbError::Validation("Redo stack corrupted".to_string())
      })?
      .clone();

    // Execute redo
    let message = command.execute()?;

    // Move command from redo to undo stack
    let redo_stack = self.redo_stack.iter().take(self.redo_stack.len() - 1).cloned().collect::<Vector<_>>();
    let undo_stack = self.undo_stack.push_back(command);

    Ok((
      Self {
        undo_stack,
        redo_stack,
        max_size: self.max_size,
      },
      Some(message),
    ))
  }

  /// Clear all undo/redo history
  #[must_use]
  pub fn clear(&self) -> Self {
    Self {
      undo_stack: Vector::new(),
      redo_stack: Vector::new(),
      max_size: self.max_size,
    }
  }

  /// Check if undo is available
  #[must_use]
  pub fn can_undo(&self) -> bool {
    !self.undo_stack.is_empty()
  }

  /// Check if redo is available
  #[must_use]
  pub fn can_redo(&self) -> bool {
    !self.redo_stack.is_empty()
  }

  /// Get the number of undoable commands
  #[must_use]
  pub fn undo_count(&self) -> usize {
    self.undo_stack.len()
  }

  /// Get the number of redoable commands
  #[must_use]
  pub fn redo_count(&self) -> usize {
    self.redo_stack.len()
  }

  /// Get description of next undo action
  #[must_use]
  pub fn peek_undo(&self) -> Option<String> {
    self
      .undo_stack
      .get(self.undo_stack.len().saturating_sub(1))
      .map(|cmd| cmd.describe())
  }

  /// Get description of next redo action
  #[must_use]
  pub fn peek_redo(&self) -> Option<String> {
    self
      .redo_stack
      .get(self.redo_stack.len().saturating_sub(1))
      .map(|cmd| cmd.describe())
  }
}

impl Default for UndoStack {
  fn default() -> Self {
    Self::new()
  }
}

// ===== Helper: Convert Bead to NewBead =====

/// Convert a Bead reference to `NewBead`
#[must_use]
pub fn bead_to_new_bead(bead: &Bead) -> NewBead {
  NewBead {
    title: bead.title.clone(),
    description: bead.description.clone(),
    status: bead.status,
    priority: bead.priority,
    bead_type: bead.bead_type,
    created_by: bead.created_by,
  }
}

// ===== Database Access Implementation for DesktopDb =====

impl DatabaseAccess for crate::db::DesktopDb {
  fn create_bead(&self, bead: NewBead) -> DbResult<Bead> {
    // Use blocking wrapper since Command trait is sync
    self.create_bead_sync(bead)
  }

  fn update_bead(&self, id: BeadId, bead: NewBead) -> DbResult<Bead> {
    self.update_bead_sync(id, bead)
  }

  fn delete_bead(&self, id: BeadId) -> DbResult<()> {
    self.delete_bead_sync(id)
  }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  use super::*;
  use clarity_core::db::models::{BeadPriority, BeadStatus, BeadType};

  // Mock database for testing
  #[derive(Clone)]
  struct MockDatabase {
    beads: std::sync::Arc<std::sync::Mutex<Vec<Bead>>>,
  }

  impl MockDatabase {
    fn new() -> Self {
      Self {
        beads: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
      }
    }
  }

  impl DatabaseAccess for MockDatabase {
    fn create_bead(&self, bead: NewBead) -> DbResult<Bead> {
      let mut beads = self.beads.lock().map_err(|e| {
        clarity_core::db::error::DbError::Validation(format!("Lock poisoned: {e}"))
      })?;
      let id = BeadId::new();
      let now = chrono::Utc::now();
      let new_bead = Bead {
        id,
        title: bead.title,
        description: bead.description,
        status: bead.status,
        priority: bead.priority,
        bead_type: bead.bead_type,
        created_by: bead.created_by,
        created_at: now,
        updated_at: now,
      };
      beads.push(new_bead.clone());
      Ok(new_bead)
    }

    fn update_bead(&self, id: BeadId, bead: NewBead) -> DbResult<Bead> {
      let mut beads = self.beads.lock().map_err(|e| {
        clarity_core::db::error::DbError::Validation(format!("Lock poisoned: {e}"))
      })?;
      let pos = beads
        .iter()
        .position(|b| b.id == id)
        .ok_or_else(|| clarity_core::db::error::DbError::not_found("Bead", id.to_string()))?;

      let now = chrono::Utc::now();
      let updated = Bead {
        id,
        title: bead.title,
        description: bead.description,
        status: bead.status,
        priority: bead.priority,
        bead_type: bead.bead_type,
        created_by: bead.created_by,
        created_at: beads[pos].created_at,
        updated_at: now,
      };

      beads[pos] = updated.clone();
      Ok(updated)
    }

    fn delete_bead(&self, id: BeadId) -> DbResult<()> {
      let mut beads = self.beads.lock().map_err(|e| {
        clarity_core::db::error::DbError::Validation(format!("Lock poisoned: {e}"))
      })?;
      let pos = beads
        .iter()
        .position(|b| b.id == id)
        .ok_or_else(|| clarity_core::db::error::DbError::not_found("Bead", id.to_string()))?;

      beads.remove(pos);
      Ok(())
    }
  }

  #[test]
  fn test_undo_stack_new() {
    let stack = UndoStack::new();
    assert!(!stack.can_undo());
    assert!(!stack.can_redo());
    assert_eq!(stack.undo_count(), 0);
    assert_eq!(stack.redo_count(), 0);
  }
  
  #[test]
  fn test_undo_stack_with_max_size() {
    let stack = UndoStack::with_max_size(50);
    assert_eq!(stack.max_size, 50);
  }
  
  #[test]
  fn test_undo_stack_clear() {
    let db = Rc::new(MockDatabase::new()) as Rc<dyn DatabaseAccess>;
    let bead = NewBead {
      title: "Test".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };
  
    // Create a test bead first
    let created = db.create_bead(bead.clone()).unwrap();
    let command = Rc::new(DeleteBeadCommand::new(db.clone(), Rc::new(created)));
  
    let stack = UndoStack::new().push_command(command);
    assert!(stack.can_undo());
  
    let cleared = stack.clear();
    assert!(!cleared.can_undo());
    assert!(!cleared.can_redo());
  }
  
  #[test]
  fn test_undo_peek() {
    let db = Rc::new(MockDatabase::new()) as Rc<dyn DatabaseAccess>;
    let bead = NewBead {
      title: "Test Bead".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };
  
    let created = db.create_bead(bead).unwrap();
    let command = Rc::new(DeleteBeadCommand::new(db, Rc::new(created)));
  
    let stack = UndoStack::new().push_command(command);
  
    assert_eq!(
      stack.peek_undo(),
      Some("Delete bead: Test Bead".to_string())
    );
    assert_eq!(stack.peek_redo(), None);
  }
  
  #[test]
  fn test_delete_command_describe() {
    let db = Rc::new(MockDatabase::new()) as Rc<dyn DatabaseAccess>;
    let bead = NewBead {
      title: "My Bead".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
        created_by: None,
      };
  
      let created = db.create_bead(bead).unwrap();
      let command = DeleteBeadCommand::new(db, Rc::new(created));
  
      assert_eq!(command.describe(), "Delete bead: My Bead");
    }
  
    #[test]
    fn test_update_command_describe() {
      let db = Rc::new(MockDatabase::new()) as Rc<dyn DatabaseAccess>;
      let bead = NewBead {
        title: "Original".to_string(),
        description: None,
        status: BeadStatus::Open,
        priority: BeadPriority::MEDIUM,
        bead_type: BeadType::Feature,
        created_by: None,
      };
  
      let created = db.create_bead(bead).unwrap();
      let updated = NewBead {
        title: "Updated".to_string(),
        description: Some("New desc".to_string()),
        status: BeadStatus::Closed,
        priority: BeadPriority::HIGH,
        bead_type: BeadType::Bugfix,
        created_by: None,
      };
  
      let command = UpdateBeadCommand::new(db, created.id, Rc::new(created), updated);
  
      assert_eq!(command.describe(), "Update bead: Original");
    }
}
