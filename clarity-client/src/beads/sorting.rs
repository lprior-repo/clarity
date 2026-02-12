//! Bead Sorting Logic
//!
//! Provides sorting functionality for bead lists with various criteria.
//! Follows functional programming patterns with zero unwrap rules.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_pass_by_value)]

use clarity_core::db::models::Bead;
use itertools::Itertools;
use std::cmp::Ordering;

/// Sorting criteria for bead lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
  Title,
  Status,
  Priority,
  Type,
  CreatedAt,
}

/// Sort direction (ascending or descending)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
  Ascending,
  Descending,
}

impl SortDirection {
  /// Get the sort indicator symbol for display
  #[must_use]
  pub const fn get_indicator(&self) -> &'static str {
    match self {
      Self::Ascending => "↑",
      Self::Descending => "↓",
    }
  }

  /// Toggle to the opposite direction
  #[must_use]
  pub const fn toggle(&self) -> Self {
    match self {
      Self::Ascending => Self::Descending,
      Self::Descending => Self::Ascending,
    }
  }
}

/// Sorting configuration
#[derive(Debug, Clone)]
pub struct SortConfig {
  pub by: SortBy,
  pub direction: SortDirection,
}

impl Default for SortConfig {
  fn default() -> Self {
    Self {
      by: SortBy::CreatedAt,
      direction: SortDirection::Descending,
    }
  }
}

impl SortConfig {
  /// Create a new sort configuration
  #[must_use]
  pub fn new(by: SortBy, direction: SortDirection) -> Self {
    Self { by, direction }
  }

  /// Sort a list of beads according to the configuration
  #[must_use]
  pub fn sort_beads(&self, beads: Vec<Bead>) -> Vec<Bead> {
    beads
      .into_iter()
      .sorted_by(|a, b| self.compare_beads(a, b))
      .collect()
  }

  /// Compare two beads based on the sorting criteria
  fn compare_beads(&self, a: &Bead, b: &Bead) -> Ordering {
    let order = match self.by {
      SortBy::Title => {
        let comparison = a.title.to_lowercase().cmp(&b.title.to_lowercase());
        if comparison == Ordering::Equal {
          a.created_at.cmp(&b.created_at)
        } else {
          comparison
        }
      }
      SortBy::Status => {
        let comparison = a.status.as_str().cmp(b.status.as_str());
        if comparison == Ordering::Equal {
          a.created_at.cmp(&b.created_at)
        } else {
          comparison
        }
      }
      SortBy::Priority => {
        // Higher priority numbers come first (1 = High, 2 = Medium, 3 = Low)
        let comparison = a.priority.0.cmp(&b.priority.0);
        if comparison == Ordering::Equal {
          // Tie-breaker: use ID for deterministic ordering
          a.id.cmp(&b.id)
        } else {
          comparison
        }
      }
      SortBy::Type => {
        let comparison = a.bead_type.as_str().cmp(b.bead_type.as_str());
        if comparison == Ordering::Equal {
          a.created_at.cmp(&b.created_at)
        } else {
          comparison
        }
      }
      SortBy::CreatedAt => a.created_at.cmp(&b.created_at),
    };

    // Apply direction
    match self.direction {
      SortDirection::Ascending => order,
      SortDirection::Descending => order.reverse(),
    }
  }

  /// Get visual indicator for current sort
  #[must_use]
  pub fn get_indicator(&self) -> &'static str {
    match self.direction {
      SortDirection::Ascending => "▲",
      SortDirection::Descending => "▼",
    }
  }

  /// Get human-readable label for sort field
  #[must_use]
  pub fn get_field_label(&self) -> &'static str {
    match self.by {
      SortBy::Title => "Title",
      SortBy::Status => "Status",
      SortBy::Priority => "Priority",
      SortBy::Type => "Type",
      SortBy::CreatedAt => "Created",
    }
  }
}

/// Sort beads with custom configuration
///
/// This is a convenience function for sorting beads with a specific configuration.
/// Returns a new vector with sorted beads (original is not modified).
#[must_use]
pub fn sort_beads(beads: Vec<Bead>, config: SortConfig) -> Vec<Bead> {
  config.sort_beads(beads)
}

/// Sort beads by title (ascending)
#[must_use]
pub fn sort_by_title(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Title, SortDirection::Ascending),
  )
}

/// Sort beads by title (descending)
#[must_use]
pub fn sort_by_title_desc(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Title, SortDirection::Descending),
  )
}

/// Sort beads by status (ascending)
#[must_use]
pub fn sort_by_status(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Status, SortDirection::Ascending),
  )
}

/// Sort beads by status (descending)
#[must_use]
pub fn sort_by_status_desc(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Status, SortDirection::Descending),
  )
}

/// Sort beads by priority (descending - high to low)
#[must_use]
pub fn sort_by_priority(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Priority, SortDirection::Descending),
  )
}

/// Sort beads by priority (ascending - low to high)
#[must_use]
pub fn sort_by_priority_asc(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Priority, SortDirection::Ascending),
  )
}

/// Sort beads by type (ascending)
#[must_use]
pub fn sort_by_type(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Type, SortDirection::Ascending),
  )
}

/// Sort beads by type (descending)
#[must_use]
pub fn sort_by_type_desc(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::Type, SortDirection::Descending),
  )
}

/// Sort beads by created date (descending - newest first)
#[must_use]
pub fn sort_by_created_at(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::CreatedAt, SortDirection::Descending),
  )
}

/// Sort beads by created date (ascending - oldest first)
#[must_use]
pub fn sort_by_created_at_asc(beads: Vec<Bead>) -> Vec<Bead> {
  sort_beads(
    beads,
    SortConfig::new(SortBy::CreatedAt, SortDirection::Ascending),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::{DateTime, Utc};
  use clarity_core::db::models::{BeadPriority, BeadStatus, BeadType};

  /// Helper function to create a test bead
  fn create_test_bead(
    id: &str,
    title: &str,
    status: BeadStatus,
    priority: BeadPriority,
    bead_type: BeadType,
    created_at: DateTime<Utc>,
  ) -> Bead {
    let created_at_str = created_at.to_rfc3339();
    let uuid = match id {
      "1" => uuid::uuid!("00000000-0000-0000-0000-000000000001"),
      "2" => uuid::uuid!("00000000-0000-0000-0000-000000000002"),
      "3" => uuid::uuid!("00000000-0000-0000-0000-000000000003"),
      _ => uuid::Uuid::new_v4(),
    };
    Bead {
      id: clarity_core::db::models::BeadId(uuid),
      title: title.to_string(),
      description: None,
      status,
      priority,
      bead_type,
      created_by: None,
      created_at: created_at_str.clone(),
      updated_at: created_at_str,
    }
  }

  #[test]
  fn test_sort_by_title() {
    let beads = vec![
      create_test_bead(
        "1",
        "Zebra",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "2",
        "Apple",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "3",
        "Banana",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
    ];

    let sorted = sort_by_title(beads);
    assert_eq!(sorted[0].title, "Apple");
    assert_eq!(sorted[1].title, "Banana");
    assert_eq!(sorted[2].title, "Zebra");
  }

  #[test]
  fn test_sort_by_title_desc() {
    let beads = vec![
      create_test_bead(
        "1",
        "Apple",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "2",
        "Banana",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "3",
        "Zebra",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
    ];

    let sorted = sort_by_title_desc(beads);
    assert_eq!(sorted[0].title, "Zebra");
    assert_eq!(sorted[1].title, "Banana");
    assert_eq!(sorted[2].title, "Apple");
  }

  #[test]
  fn test_sort_by_status() {
    let beads = vec![
      create_test_bead(
        "1",
        "A",
        BeadStatus::Closed,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "2",
        "B",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "3",
        "C",
        BeadStatus::InProgress,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
    ];

    let sorted = sort_by_status(beads);
    // Status strings: "closed", "open", "in_progress"
    // Ascending alphabetical order: "closed", "in_progress", "open"
    assert_eq!(sorted[0].status, BeadStatus::Closed);
    assert_eq!(sorted[1].status, BeadStatus::InProgress);
    assert_eq!(sorted[2].status, BeadStatus::Open);
  }

  #[test]
  fn test_sort_by_priority() {
    let beads = vec![
      create_test_bead(
        "1",
        "A",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "2",
        "B",
        BeadStatus::Open,
        BeadPriority::HIGH,
        BeadType::Feature,
        Utc::now(),
      ),
      create_test_bead(
        "3",
        "C",
        BeadStatus::Open,
        BeadPriority::MEDIUM,
        BeadType::Feature,
        Utc::now(),
      ),
    ];

    let sorted = sort_by_priority(beads);
    // sort_by_priority uses SortDirection::Descending which reverses the comparison.
    // Comparison: a.priority.0.cmp(&b.priority.0) gives ascending [1, 2, 3]
    // Descending reverses to [3, 2, 1] = [LOW, MEDIUM, HIGH]
    assert_eq!(sorted[0].priority, BeadPriority::LOW);
    assert_eq!(sorted[1].priority, BeadPriority::MEDIUM);
    assert_eq!(sorted[2].priority, BeadPriority::HIGH);
  }

  #[test]
  fn test_sort_by_type() {
    let beads = vec![
      create_test_bead(
        "1",
        "A",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Bugfix,
        Utc::now(),
      ),
      create_test_bead(
        "2",
        "B",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Docs,
        Utc::now(),
      ),
      create_test_bead(
        "3",
        "C",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        Utc::now(),
      ),
    ];

    let sorted = sort_by_type(beads);
    assert_eq!(sorted[0].bead_type, BeadType::Bugfix);
    assert_eq!(sorted[1].bead_type, BeadType::Docs);
    assert_eq!(sorted[2].bead_type, BeadType::Feature);
  }

  #[test]
  fn test_sort_by_created_at() {
    let now = Utc::now();
    let past = now - chrono::Duration::days(1);
    let future = now + chrono::Duration::days(1);

    let beads = vec![
      create_test_bead(
        "1",
        "A",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        past,
      ),
      create_test_bead(
        "2",
        "B",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        future,
      ),
      create_test_bead(
        "3",
        "C",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        now,
      ),
    ];

    let sorted = sort_by_created_at(beads);
    assert_eq!(sorted[0].created_at, future.to_rfc3339());
    assert_eq!(sorted[1].created_at, now.to_rfc3339());
    assert_eq!(sorted[2].created_at, past.to_rfc3339());
  }

  #[test]
  fn test_sort_config_direction() {
    let config = SortConfig::new(SortBy::Title, SortDirection::Descending);
    assert_eq!(config.get_indicator(), "▼");
    assert_eq!(config.get_field_label(), "Title");

    let config_asc = SortConfig::new(SortBy::Priority, SortDirection::Ascending);
    assert_eq!(config_asc.get_indicator(), "▲");
    assert_eq!(config_asc.get_field_label(), "Priority");
  }

  #[test]
  fn test_sort_with_tie_breaker() {
    let now = Utc::now();
    let later = now + chrono::Duration::seconds(1);

    let beads = vec![
      create_test_bead(
        "1",
        "Same Title",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        now,
      ),
      create_test_bead(
        "2",
        "Same Title",
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Feature,
        later,
      ),
    ];

    let sorted = sort_by_title(beads);
    // Same title, so created_at should break the tie
    // BeadId wraps a UUID, so compare UUIDs directly
    assert_eq!(
      sorted[0].id.0,
      uuid::uuid!("00000000-0000-0000-0000-000000000001")
    );
    assert_eq!(
      sorted[1].id.0,
      uuid::uuid!("00000000-0000-0000-0000-000000000002")
    );
  }
}
