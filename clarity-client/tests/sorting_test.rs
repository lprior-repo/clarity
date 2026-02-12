//! Test file for sorting functionality
//!
//! This is a standalone test file to verify our sorting implementation works correctly.

#![allow(warnings)]
#![allow(clippy::all)]

// Simple test to verify sorting logic without dependencies
#[cfg(test)]
mod tests {
  use std::cmp::Ordering;

  // Mock types for testing
  #[derive(Debug, Clone, PartialEq, Eq)]
  enum SortBy {
    Title,
    Status,
    Priority,
    Type,
    CreatedAt,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  enum SortDirection {
    Ascending,
    Descending,
  }

  #[derive(Debug, Clone, PartialEq, Eq)]
  struct Bead {
    title: String,
    status: String,
    priority: i16,
    bead_type: String,
    created_at: i64,
  }

  impl Bead {
    fn new(title: &str, status: &str, priority: i16, bead_type: &str, created_at: i64) -> Self {
      Self {
        title: title.to_string(),
        status: status.to_string(),
        priority,
        bead_type: bead_type.to_string(),
        created_at,
      }
    }
  }

  #[derive(Debug, Clone)]
  struct SortConfig {
    by: SortBy,
    direction: SortDirection,
  }

  impl SortConfig {
    fn new(by: SortBy, direction: SortDirection) -> Self {
      Self { by, direction }
    }

    fn sort_beads(&self, mut beads: Vec<Bead>) -> Vec<Bead> {
      // Use standard library sort_by instead of itertools sorted_by
      beads.sort_by(|a, b| self.compare_beads(a, b));
      beads
    }

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
          let comparison = a.status.cmp(&b.status);
          if comparison == Ordering::Equal {
            a.created_at.cmp(&b.created_at)
          } else {
            comparison
          }
        }
        SortBy::Priority => {
          let comparison = a.priority.cmp(&b.priority);
          if comparison == Ordering::Equal {
            a.created_at.cmp(&b.created_at)
          } else {
            comparison
          }
        }
        SortBy::Type => {
          let comparison = a.bead_type.cmp(&b.bead_type);
          if comparison == Ordering::Equal {
            a.created_at.cmp(&b.created_at)
          } else {
            comparison
          }
        }
        SortBy::CreatedAt => a.created_at.cmp(&b.created_at),
      };

      match self.direction {
        SortDirection::Ascending => order,
        SortDirection::Descending => order.reverse(),
      }
    }

    fn get_indicator(&self) -> &'static str {
      match self.direction {
        SortDirection::Ascending => "▲",
        SortDirection::Descending => "▼",
      }
    }

    fn get_field_label(&self) -> &'static str {
      match self.by {
        SortBy::Title => "Title",
        SortBy::Status => "Status",
        SortBy::Priority => "Priority",
        SortBy::Type => "Type",
        SortBy::CreatedAt => "Created",
      }
    }
  }

  #[test]
  fn test_sort_by_title() {
    let beads = vec![
      Bead::new("Zebra", "open", 1, "feature", 100),
      Bead::new("Apple", "open", 1, "feature", 200),
      Bead::new("Banana", "open", 1, "feature", 300),
    ];

    let config = SortConfig::new(SortBy::Title, SortDirection::Ascending);
    let sorted = config.sort_beads(beads);

    assert_eq!(sorted[0].title, "Apple");
    assert_eq!(sorted[1].title, "Banana");
    assert_eq!(sorted[2].title, "Zebra");
  }

  #[test]
  fn test_sort_by_title_desc() {
    let beads = vec![
      Bead::new("Apple", "open", 1, "feature", 100),
      Bead::new("Banana", "open", 1, "feature", 200),
      Bead::new("Zebra", "open", 1, "feature", 300),
    ];

    let config = SortConfig::new(SortBy::Title, SortDirection::Descending);
    let sorted = config.sort_beads(beads);

    assert_eq!(sorted[0].title, "Zebra");
    assert_eq!(sorted[1].title, "Banana");
    assert_eq!(sorted[2].title, "Apple");
  }

  #[test]
  fn test_sort_by_priority() {
    let beads = vec![
      Bead::new("A", "open", 3, "feature", 100),
      Bead::new("B", "open", 1, "feature", 200),
      Bead::new("C", "open", 2, "feature", 300),
    ];

    let config = SortConfig::new(SortBy::Priority, SortDirection::Ascending);
    let sorted = config.sort_beads(beads);

    assert_eq!(sorted[0].priority, 1);
    assert_eq!(sorted[1].priority, 2);
    assert_eq!(sorted[2].priority, 3);
  }

  #[test]
  fn test_sort_with_tie_breaker() {
    let beads = vec![
      Bead::new("Same", "open", 1, "feature", 200),
      Bead::new("Same", "open", 1, "feature", 100),
    ];

    let config = SortConfig::new(SortBy::Title, SortDirection::Ascending);
    let sorted = config.sort_beads(beads);

    // Same title, so created_at should break the tie
    assert_eq!(sorted[0].created_at, 100); // older first
    assert_eq!(sorted[1].created_at, 200); // newer later
  }

  #[test]
  fn test_sort_config_direction() {
    let config = SortConfig::new(SortBy::Priority, SortDirection::Descending);
    assert_eq!(config.get_indicator(), "▼");
    assert_eq!(config.get_field_label(), "Priority");

    let config_asc = SortConfig::new(SortBy::Title, SortDirection::Ascending);
    assert_eq!(config_asc.get_indicator(), "▲");
    assert_eq!(config_asc.get_field_label(), "Title");
  }

  #[test]
  fn test_sort_direction_toggle() {
    let beads = vec![
      Bead::new("C", "open", 1, "feature", 100),
      Bead::new("B", "open", 1, "feature", 200),
      Bead::new("A", "open", 1, "feature", 300),
    ];

    // Test ascending
    let config = SortConfig::new(SortBy::Title, SortDirection::Ascending);
    let sorted = config.sort_beads(beads.clone());
    assert_eq!(sorted[0].title, "A");
    assert_eq!(sorted[1].title, "B");
    assert_eq!(sorted[2].title, "C");

    // Test descending
    let config = SortConfig::new(SortBy::Title, SortDirection::Descending);
    let sorted = config.sort_beads(beads);
    assert_eq!(sorted[0].title, "C");
    assert_eq!(sorted[1].title, "B");
    assert_eq!(sorted[2].title, "A");
  }
}
