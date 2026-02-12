#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

//! Domain repository interfaces
//!
//! This module defines the repository interfaces for the domain layer,
//! following the Repository Pattern to abstract persistence concerns.

use crate::domain::models::NewBead;
use crate::domain::models::{Bead, ModelError};
use crate::domain::types::{BeadId, BeadPriority, BeadStatus, BeadType, UserId};
use std::collections::HashMap;

/// Repository interface for bead operations
pub trait BeadRepository {
  /// Create a new bead
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Validation fails
  /// - Business rules are violated
  async fn create_bead(&self, new_bead: NewBead) -> Result<Bead, ModelError>;

  /// Get a bead by ID
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Bead is not found
  async fn get_bead_by_id(&self, bead_id: BeadId) -> Result<Option<Bead>, ModelError>;

  /// Update bead status
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Bead is not found
  /// - Status transition is invalid
  async fn update_bead_status(
    &self,
    bead_id: BeadId,
    new_status: BeadStatus,
  ) -> Result<Bead, ModelError>;

  /// Update bead priority
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Bead is not found
  async fn update_bead_priority(
    &self,
    bead_id: BeadId,
    new_priority: BeadPriority,
  ) -> Result<Bead, ModelError>;

  /// Update bead title and description
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Bead is not found
  /// - Validation fails
  async fn update_bead(
    &self,
    bead_id: BeadId,
    title: Option<String>,
    description: Option<String>,
  ) -> Result<Bead, ModelError>;

  /// Delete a bead
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Bead is not found
  /// - Business rules prevent deletion
  async fn delete_bead(&self, bead_id: BeadId) -> Result<(), ModelError>;

  /// Get all beads
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Database operation fails
  async fn get_all_beads(&self) -> Result<Vec<Bead>, ModelError>;

  /// Get beads by status
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Database operation fails
  async fn get_beads_by_status(&self, status: BeadStatus) -> Result<Vec<Bead>, ModelError>;

  /// Get beads by priority
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Database operation fails
  async fn get_beads_by_priority(&self, priority: BeadPriority) -> Result<Vec<Bead>, ModelError>;

  /// Get beads by creator
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Database operation fails
  async fn get_beads_by_creator(&self, creator_id: UserId) -> Result<Vec<Bead>, ModelError>;

  /// Get bead statistics
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Database operation fails
  async fn get_bead_statistics(&self) -> Result<BeadStatistics, ModelError>;

  /// Search beads with filters and pagination
  ///
  /// # Errors
  /// Returns `ModelError` if:
  /// - Database operation fails
  async fn search_beads(&self, filters: BeadSearchFilters) -> Result<BeadSearchResult, ModelError>;
}

/// Statistics for beads
#[derive(Debug, Clone, PartialEq)]
pub struct BeadStatistics {
  pub total: usize,
  pub status_counts: HashMap<BeadStatus, usize>,
  pub priority_counts: HashMap<BeadPriority, usize>,
  pub type_counts: HashMap<BeadType, usize>,
}

impl BeadStatistics {
  /// Get the count of beads with a specific status
  #[must_use]
  pub fn count_by_status(&self, status: BeadStatus) -> usize {
    self.status_counts.get(&status).copied().unwrap_or(0)
  }

  /// Get the count of beads with a specific type
  #[must_use]
  pub fn count_by_type(&self, bead_type: BeadType) -> usize {
    self.type_counts.get(&bead_type).copied().unwrap_or(0)
  }

  /// Get the count of beads with a specific priority
  #[must_use]
  pub fn count_by_priority(&self, priority: BeadPriority) -> usize {
    self.priority_counts.get(&priority).copied().unwrap_or(0)
  }

  /// Calculate the percentage of beads with a specific status
  #[must_use]
  pub fn percentage_by_status(&self, status: BeadStatus) -> f64 {
    if self.total == 0 {
      0.0
    } else {
      (self.count_by_status(status) as f64 / self.total as f64) * 100.0
    }
  }
}

/// Search filters for beads
#[derive(Debug, Clone, PartialEq)]
pub struct BeadSearchFilters {
  pub status: Option<BeadStatus>,
  pub priority: Option<BeadPriority>,
  pub bead_type: Option<BeadType>,
  pub creator_id: Option<UserId>,
  pub search_term: Option<String>,
  pub page: Option<u32>,
  pub page_size: Option<u32>,
}

impl BeadSearchFilters {
  /// Create new empty filters
  #[must_use]
  pub fn new() -> Self {
    Self {
      status: None,
      priority: None,
      bead_type: None,
      creator_id: None,
      search_term: None,
      page: None,
      page_size: None,
    }
  }

  /// Create filters with pagination defaults
  #[must_use]
  pub fn with_pagination(mut self, page: u32, page_size: u32) -> Self {
    self.page = Some(page);
    self.page_size = Some(page_size);
    self
  }

  /// Get the page number (default: 1)
  #[must_use]
  pub fn page(&self) -> u32 {
    self.page.unwrap_or(1)
  }

  /// Get the page size (default: 25)
  #[must_use]
  pub fn page_size(&self) -> u32 {
    self.page_size.unwrap_or(25)
  }

  /// Calculate the offset for pagination
  #[must_use]
  pub fn offset(&self) -> u32 {
    (self.page() - 1) * self.page_size()
  }
}

impl Default for BeadSearchFilters {
  fn default() -> Self {
    Self::new()
  }
}

/// Search result for beads
#[derive(Debug, Clone, PartialEq)]
pub struct BeadSearchResult {
  pub beads: Vec<Bead>,
  pub total: u64,
  pub page: u32,
  pub page_size: u32,
  pub total_pages: u32,
}

impl BeadSearchResult {
  /// Create a new search result
  #[must_use]
  pub fn new(beads: Vec<Bead>, total: u64, page: u32, page_size: u32) -> Self {
    let total_pages = if page_size == 0 {
      0
    } else {
      ((total + page_size as u64 - 1) / page_size as u64) as u32
    };

    Self {
      beads,
      total,
      page,
      page_size,
      total_pages,
    }
  }

  /// Check if there are more pages
  #[must_use]
  pub fn has_more_pages(&self) -> bool {
    self.page < self.total_pages
  }

  /// Check if this is the first page
  #[must_use]
  pub fn is_first_page(&self) -> bool {
    self.page == 1
  }
}
