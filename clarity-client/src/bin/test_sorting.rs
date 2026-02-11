//! Test sorting functionality

use itertools::Itertools;
use std::cmp::Ordering;

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

  fn sort_beads(&self, beads: Vec<Bead>) -> Vec<Bead> {
    beads
      .into_iter()
      .sorted_by(|a, b| self.compare_beads(a, b))
      .collect()
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
        // Higher priority numbers come first (1 = High, 2 = Medium, 3 = Low)
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

fn main() {
  println!("Testing sorting functionality...");

  let beads = vec![
    Bead::new("Zebra", "open", 1, "feature", 100),
    Bead::new("Apple", "open", 1, "feature", 200),
    Bead::new("Banana", "open", 1, "feature", 300),
  ];

  // Test sorting by title ascending
  let config = SortConfig::new(SortBy::Title, SortDirection::Ascending);
  let sorted = config.sort_beads(beads.clone());
  println!("Sorted by title (ascending):");
  for bead in &sorted {
    println!("- {}: {}", bead.title, bead.created_at);
  }
  assert_eq!(sorted[0].title, "Apple");
  assert_eq!(sorted[1].title, "Banana");
  assert_eq!(sorted[2].title, "Zebra");
  println!("✓ Title sorting works");

  // Test sorting by title descending
  let config = SortConfig::new(SortBy::Title, SortDirection::Descending);
  let sorted = config.sort_beads(beads.clone());
  println!("\nSorted by title (descending):");
  for bead in &sorted {
    println!("- {}: {}", bead.title, bead.created_at);
  }
  assert_eq!(sorted[0].title, "Zebra");
  assert_eq!(sorted[1].title, "Banana");
  assert_eq!(sorted[2].title, "Apple");
  println!("✓ Title descending sorting works");

  // Test sorting by priority
  let priority_beads = vec![
    Bead::new("A", "open", 3, "feature", 100),
    Bead::new("B", "open", 1, "feature", 200),
    Bead::new("C", "open", 2, "feature", 300),
  ];
  let config = SortConfig::new(SortBy::Priority, SortDirection::Ascending);
  let sorted = config.sort_beads(priority_beads);
  println!("\nSorted by priority (ascending - 1 to 3):");
  for bead in &sorted {
    println!(
      "- {}: {} (priority: {})",
      bead.title, bead.created_at, bead.priority
    );
  }
  assert_eq!(sorted[0].priority, 1);
  assert_eq!(sorted[1].priority, 2);
  assert_eq!(sorted[2].priority, 3);
  println!("✓ Priority sorting works");

  // Test sort indicators
  let config = SortConfig::new(SortBy::Priority, SortDirection::Descending);
  assert_eq!(config.get_indicator(), "▼");
  assert_eq!(config.get_field_label(), "Priority");
  println!("✓ Sort indicators work");

  // Test tie-breaking
  let tie_beads = vec![
    Bead::new("Same", "open", 1, "feature", 200),
    Bead::new("Same", "open", 1, "feature", 100),
  ];
  let config = SortConfig::new(SortBy::Title, SortDirection::Ascending);
  let sorted = config.sort_beads(tie_beads);
  println!("\nTie-breaking test:");
  for bead in &sorted {
    println!(
      "- {}: {} (created_at: {})",
      bead.title, bead.bead_type, bead.created_at
    );
  }
  assert_eq!(sorted[0].created_at, 100); // older first
  assert_eq!(sorted[1].created_at, 200); // newer later
  println!("✓ Tie-breaking works");

  println!("\n✅ All sorting tests passed!");
  println!("\nImplementation summary:");
  println!("- ✅ Created SortBy enum with 5 fields: Title, Status, Priority, Type, CreatedAt");
  println!("- ✅ Created SortDirection enum with Ascending/Descending");
  println!("- ✅ Implemented SortConfig with sorting logic");
  println!("- ✅ Added visual indicators (▲/▼)");
  println!("- ✅ Implemented tie-breaking with created_at");
  println!("- ✅ All sorting uses functional patterns (no unwrap/expect)");
}
