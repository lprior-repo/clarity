#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::db::error::{DbError, DbResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===== Domain Types (Newtypes) =====

/// Macro to generate UUID-based ID types with consistent behavior
macro_rules! uuid_id {
  (
    $(#[$meta:meta])*
    $name:ident
  ) => {
    $(#[$meta])*
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct $name(pub Uuid);

    impl $name {
      /// Create a new random ID
      pub fn new() -> Self {
        Self(Uuid::new_v4())
      }

      /// Create from Uuid string
      ///
      /// # Errors
      /// Returns `DbError::InvalidUuid` if the string is not a valid UUID
      #[allow(clippy::should_implement_trait)]
      pub fn from_str(s: &str) -> DbResult<Self> {
        Uuid::parse_str(s)
          .map(Self)
          .map_err(|_| DbError::InvalidUuid(s.to_string()))
      }

      /// Get underlying Uuid
      pub const fn as_uuid(&self) -> Uuid {
        self.0
      }

      /// Get string representation of the ID
      #[must_use]
      pub fn as_str(&self) -> String {
        self.0.to_string()
      }
    }

    impl Default for $name {
      fn default() -> Self {
        Self::new()
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }

    impl From<$name> for Uuid {
      fn from(id: $name) -> Self {
        id.0
      }
    }

    impl From<Uuid> for $name {
      fn from(uuid: Uuid) -> Self {
        Self(uuid)
      }
    }

    impl PartialOrd for $name {
      fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
      }
    }

    impl Ord for $name {
      fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
      }
    }
  };
}

// Apply the macro to generate ID types
uuid_id!(
  /// Bead identifier
  BeadId
);

// ===== Enums =====

/// Bead status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bead_status", rename_all = "lowercase")]
pub enum BeadStatus {
  Open,
  InProgress,
  Blocked,
  Deferred,
  Closed,
}

impl BeadStatus {
  /// Parse a string into a `BeadStatus`
  ///
  /// # Errors
  /// - Returns `DbError::Validation` if the string is not a valid status
  ///
  /// Get the status as a lowercase string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Open => "open",
      Self::InProgress => "in_progress",
      Self::Blocked => "blocked",
      Self::Deferred => "deferred",
      Self::Closed => "closed",
    }
  }
}

impl std::fmt::Display for BeadStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl std::str::FromStr for BeadStatus {
  type Err = DbError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "open" => Ok(Self::Open),
      "in_progress" => Ok(Self::InProgress),
      "blocked" => Ok(Self::Blocked),
      "deferred" => Ok(Self::Deferred),
      "closed" => Ok(Self::Closed),
      _ => Err(DbError::validation(format!("Invalid bead status: {s}"))),
    }
  }
}

/// Bead type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bead_type", rename_all = "lowercase")]
pub enum BeadType {
  Feature,
  Bugfix,
  Refactor,
  Test,
  Docs,
}

impl BeadType {
  /// Parse a string into a `BeadType`
  ///
  /// # Errors
  /// - Returns `DbError::Validation` if the string is not a valid type
  ///
  /// Get the type as a lowercase string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Feature => "feature",
      Self::Bugfix => "bugfix",
      Self::Refactor => "refactor",
      Self::Test => "test",
      Self::Docs => "docs",
    }
  }
}

impl std::fmt::Display for BeadType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl std::str::FromStr for BeadType {
  type Err = DbError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "feature" => Ok(Self::Feature),
      "bugfix" => Ok(Self::Bugfix),
      "refactor" => Ok(Self::Refactor),
      "test" => Ok(Self::Test),
      "docs" => Ok(Self::Docs),
      _ => Err(DbError::validation(format!("Invalid bead type: {s}"))),
    }
  }
}

// Re-export from domain types
pub use crate::domain::types::BeadPriority;

// ===== Domain Models =====

/// Bead entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bead {
  pub id: BeadId,
  pub title: String,
  pub description: Option<String>,
  pub status: BeadStatus,
  pub priority: BeadPriority,
  pub bead_type: BeadType,
  pub created_by: Option<Uuid>,
  pub created_at: String,
  pub updated_at: String,
}

/// New bead (without id and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBead {
  pub title: String,
  pub description: Option<String>,
  pub status: BeadStatus,
  pub priority: BeadPriority,
  pub bead_type: BeadType,
  pub created_by: Option<Uuid>,
}

/// Interview entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interview {
  pub id: Uuid,
  pub spec_name: String,
  pub questions: serde_json::Value,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Spec entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
  pub id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub schema: serde_json::Value,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Bead filters for server-side filtering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadFilters {
  pub status: Option<String>,
  pub bead_type: Option<String>,
  pub priority: Option<i16>,
  pub created_by: Option<Uuid>,
  pub search: Option<String>,
  pub page: Option<u32>,
  pub page_size: Option<u32>,
}

impl BeadFilters {
  #[must_use]
  pub const fn new() -> Self {
    Self {
      status: None,
      bead_type: None,
      priority: None,
      created_by: None,
      search: None,
      page: None,
      page_size: None,
    }
  }
}

impl BeadFilters {
  /// Check if filters are active (has any filter applied)
  #[must_use]
  pub fn is_active(&self) -> bool {
    self.status.is_some()
      || self.bead_type.is_some()
      || self.priority.is_some()
      || self.created_by.is_some()
      || self.search.as_ref().is_some_and(|s| !s.is_empty())
      || self.page.is_some()
      || self.page_size.is_some()
  }

  /// Get the page number (default: 1)
  #[must_use]
  pub fn page(&self) -> u32 {
    self.page.map_or(1, |p| p)
  }

  /// Get the page size (default: 25)
  #[must_use]
  pub fn page_size(&self) -> u32 {
    self.page_size.map_or(25, |p| p)
  }

  /// Calculate the offset for pagination
  #[must_use]
  pub fn offset(&self) -> u32 {
    (self.page() - 1) * self.page_size()
  }
}

impl Default for BeadFilters {
  fn default() -> Self {
    Self::new()
  }
}

/// Paginated bead results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginatedBeads {
  pub beads: Vec<Bead>,
  pub total: u64,
  pub page: u32,
  pub page_size: u32,
  pub total_pages: u32,
}

impl PaginatedBeads {
  #[must_use]
  pub fn new(beads: Vec<Bead>, total: u64, page: u32, page_size: u32) -> Self {
    let total_pages = if page_size == 0 {
      0
    } else {
      let pages_u64 = total.div_ceil(u64::from(page_size));
      u32::try_from(pages_u64).unwrap_or(u32::MAX)
    };

    Self {
      beads,
      total,
      page,
      page_size,
      total_pages,
    }
  }

  #[must_use]
  pub const fn has_next(&self) -> bool {
    self.page < self.total_pages
  }

  #[must_use]
  pub const fn has_previous(&self) -> bool {
    self.page > 1
  }
}
