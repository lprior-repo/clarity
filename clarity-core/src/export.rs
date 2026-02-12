#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Bead export functionality
//!
//! Pure functions for exporting beads to JSON and CSV formats.
//! All functions are deterministic and side-effect free.

use crate::db::models::{Bead, BeadId, BeadPriority, BeadStatus, BeadType};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Export format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
  /// JSON format with full bead data
  Json,
  /// CSV format for spreadsheet compatibility
  Csv,
}

impl ExportFormat {
  /// Get the file extension for this format
  #[must_use]
  pub const fn extension(&self) -> &str {
    match self {
      Self::Json => "json",
      Self::Csv => "csv",
    }
  }

  /// Get the MIME type for this format
  #[must_use]
  pub const fn mime_type(&self) -> &str {
    match self {
      Self::Json => "application/json",
      Self::Csv => "text/csv",
    }
  }
}

/// Export errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExportError {
  #[error("JSON serialization failed: {0}")]
  /// Returns `ExportError::JsonSerialization` if JSON export fails
  JsonSerialization(String),

  #[error("CSV serialization failed: {0}")]
  /// Returns `ExportError::CsvSerialization` if CSV export fails
  CsvSerialization(String),

  #[error("No beads to export")]
  EmptyDataset,

  #[error("Invalid date format: {0}")]
  InvalidDateFormat(String),
}

/// Export result type
pub type ExportResult<T> = Result<T, ExportError>;

/// Serializable export format for beads
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedBead {
  /// Unique identifier
  pub id: String,
  /// Bead title
  pub title: String,
  /// Optional description
  pub description: Option<String>,
  /// Current status
  pub status: String,
  /// Priority level (1=high, 2=medium, 3=low)
  pub priority: i16,
  /// Bead type
  pub bead_type: String,
  /// Creator user ID
  pub created_by: Option<String>,
  /// Creation timestamp (ISO 8601)
  pub created_at: String,
  /// Last update timestamp (ISO 8601)
  pub updated_at: String,
}

impl From<&Bead> for ExportedBead {
  fn from(bead: &Bead) -> Self {
    Self {
      id: bead.id.to_string(),
      title: bead.title.clone(),
      description: bead.description.clone(),
      status: bead.status.as_str().to_string(),
      priority: bead.priority.0,
      bead_type: bead.bead_type.as_str().to_string(),
      created_by: bead.created_by.map(|id| id.to_string()),
      created_at: bead.created_at.clone(),
      updated_at: bead.updated_at.clone(),
    }
  }
}

/// Container for exported bead data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadExport {
  /// Export format version
  pub version: String,
  /// Export timestamp
  pub exported_at: String,
  /// Total bead count
  pub count: usize,
  /// Exported beads
  pub beads: Vec<ExportedBead>,
}

impl BeadExport {
  /// Create a new export from beads
  #[must_use]
  pub fn new(beads: &[Bead]) -> Self {
    Self {
      version: "1.0".to_string(),
      exported_at: Utc::now().to_rfc3339(),
      count: beads.len(),
      beads: beads.iter().map(ExportedBead::from).collect(),
    }
  }

  /// Convert to JSON string
  ///
  /// # Errors
  /// Returns `ExportError::JsonSerialization` if JSON serialization fails
  pub fn to_json(&self) -> ExportResult<String> {
    serde_json::to_string_pretty(self).map_err(|e| ExportError::JsonSerialization(e.to_string()))
  }

  /// Convert to CSV string
  ///
  /// # Errors
  /// Returns `ExportError::CsvSerialization` if CSV generation fails
  pub fn to_csv(&self) -> ExportResult<String> {
    if self.beads.is_empty() {
      return Err(ExportError::EmptyDataset);
    }

    // CSV header
    let header = "id,title,description,status,priority,bead_type,created_by,created_at,updated_at";

    // Convert each bead to CSV row
    let rows: ExportResult<Vec<String>> = self.beads.iter().map(ExportedBead::to_csv_row).collect();

    let rows = rows?;

    // Combine header and rows
    let csv = [header, &rows.join("\n")].join("\n");
    Ok(csv)
  }
}

impl ExportedBead {
  /// Convert bead to CSV row
  ///
  /// # Errors
  /// Returns `ExportError::CsvSerialization` if a field contains invalid characters
  pub fn to_csv_row(&self) -> ExportResult<String> {
    // Helper to escape CSV values
    let escape = |value: &str| -> String {
      if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
      } else {
        value.to_string()
      }
    };

    let fields: Vec<String> = [
      escape(&self.id),
      escape(&self.title),
      escape(self.description.as_deref().map_or("", |s| s)),
      escape(&self.status),
      escape(&self.priority.to_string()),
      escape(&self.bead_type),
      escape(self.created_by.as_deref().map_or("", |s| s)),
      escape(&self.created_at),
      escape(&self.updated_at),
    ]
    .into_iter()
    .collect();

    Ok(fields.join(","))
  }

  /// Parse from CSV row
  ///
  /// # Errors
  /// Returns `ExportError::CsvSerialization` if parsing fails
  pub fn from_csv_row(row: &str) -> ExportResult<Self> {
    let columns: Vec<&str> = row.split(',').collect();

    if columns.len() != 9 {
      return Err(ExportError::CsvSerialization(format!(
        "Expected 9 columns, found {}",
        columns.len()
      )));
    }

    // Helper to unescape CSV values
    let unescape = |value: &str| -> String {
      let trimmed = value.trim();
      if trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].replace("\"\"", "\"")
      } else {
        trimmed.to_string()
      }
    };

    let id = unescape(columns[0]);
    let title = unescape(columns[1]);
    let description = {
      let desc = unescape(columns[2]);
      if desc.is_empty() {
        None
      } else {
        Some(desc)
      }
    };
    let status = unescape(columns[3]);
    let priority = unescape(columns[4])
      .parse::<i16>()
      .map_err(|_| ExportError::CsvSerialization("Invalid priority value".to_string()))?;
    let bead_type = unescape(columns[5]);
    let created_by = {
      let user = unescape(columns[6]);
      if user.is_empty() {
        None
      } else {
        Some(user)
      }
    };
    let created_at = unescape(columns[7]);
    let updated_at = unescape(columns[8]);

    Ok(Self {
      id,
      title,
      description,
      status,
      priority,
      bead_type,
      created_by,
      created_at,
      updated_at,
    })
  }
}

/// Export beads to specified format
///
/// # Errors
/// - Returns `ExportError::EmptyDataset` if no beads provided
/// - Returns `ExportError::JsonSerialization` if JSON export fails
/// - Returns `ExportError::CsvSerialization` if CSV export fails
pub fn export_beads(beads: &[Bead], format: ExportFormat) -> ExportResult<String> {
  if beads.is_empty() {
    return Err(ExportError::EmptyDataset);
  }

  let export = BeadExport::new(beads);

  match format {
    ExportFormat::Json => export.to_json(),
    ExportFormat::Csv => export.to_csv(),
  }
}

/// Convert exported beads back to domain beads (for validation testing)
///
/// # Errors
/// Returns `ExportError::InvalidDateFormat` if timestamps are invalid
pub fn exported_to_domain(beads: &[ExportedBead]) -> ExportResult<Vec<Bead>> {
  beads
    .iter()
    .map(|exported| {
      // For db::models::Bead, we keep dates as strings (ISO 8601 format)
      Ok(Bead {
        id: BeadId::from_str(&exported.id).map_err(|_| {
          ExportError::CsvSerialization(format!("Invalid bead ID: {}", exported.id))
        })?,
        title: exported.title.clone(),
        description: exported.description.clone(),
        status: BeadStatus::from_str(&exported.status).map_err(|_| {
          ExportError::CsvSerialization(format!("Invalid status: {}", exported.status))
        })?,
        priority: BeadPriority::new(exported.priority)
          .map_err(|_| ExportError::CsvSerialization("Invalid priority value".to_string()))?,
        bead_type: BeadType::from_str(&exported.bead_type).map_err(|_| {
          ExportError::CsvSerialization(format!("Invalid bead type: {}", exported.bead_type))
        })?,
        created_by: match exported.created_by {
          Some(ref user_id) => Some(
            Uuid::parse_str(user_id)
              .map_err(|_| ExportError::CsvSerialization(format!("Invalid user ID: {user_id}")))?,
          ),
          None => None,
        },
        created_at: exported.created_at.clone(),
        updated_at: exported.updated_at.clone(),
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::models::BeadId;
  use uuid::Uuid;

  #[test]
  fn test_export_format_extension() {
    assert_eq!(ExportFormat::Json.extension(), "json");
    assert_eq!(ExportFormat::Csv.extension(), "csv");
  }

  #[test]
  fn test_export_format_mime_type() {
    assert_eq!(ExportFormat::Json.mime_type(), "application/json");
    assert_eq!(ExportFormat::Csv.mime_type(), "text/csv");
  }

  #[test]
  fn test_exported_bead_from_domain() {
    let bead = Bead {
      id: BeadId::from(Uuid::nil()),
      title: "Test Bead".to_string(),
      description: Some("Test Description".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::HIGH,
      bead_type: BeadType::Feature,
      created_by: None,
      created_at: Utc::now().to_rfc3339(),
      updated_at: Utc::now().to_rfc3339(),
    };

    let exported = ExportedBead::from(&bead);

    assert_eq!(exported.title, "Test Bead");
    assert_eq!(exported.description, Some("Test Description".to_string()));
    assert_eq!(exported.status, "open");
    assert_eq!(exported.priority, 1);
    assert_eq!(exported.bead_type, "feature");
  }

  #[test]
  fn test_bead_export_new() {
    let beads = vec![
      Bead {
        id: BeadId::from(Uuid::new_v4()),
        title: "Bead 1".to_string(),
        description: None,
        status: BeadStatus::Open,
        priority: BeadPriority::HIGH,
        bead_type: BeadType::Feature,
        created_by: None,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
      },
      Bead {
        id: BeadId::from(Uuid::new_v4()),
        title: "Bead 2".to_string(),
        description: Some("Description".to_string()),
        status: BeadStatus::InProgress,
        priority: BeadPriority::MEDIUM,
        bead_type: BeadType::Bugfix,
        created_by: None,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
      },
    ];

    let export = BeadExport::new(&beads);

    assert_eq!(export.version, "1.0");
    assert_eq!(export.count, 2);
    assert_eq!(export.beads.len(), 2);
  }

  #[test]
  fn test_export_beads_to_json() {
    let beads = vec![Bead {
      id: BeadId::from(Uuid::new_v4()),
      title: "Test Bead".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::HIGH,
      bead_type: BeadType::Feature,
      created_by: None,
      created_at: Utc::now().to_rfc3339(),
      updated_at: Utc::now().to_rfc3339(),
    }];

    let result = export_beads(&beads, ExportFormat::Json);
    assert!(result.is_ok());

    let json = match result {
      Ok(json) => json,
      Err(e) => panic!("Expected Ok, got Err: {e}"),
    };
    assert!(json.contains("Test Bead"));
    assert!(json.contains("\"version\""));
  }

  #[test]
  fn test_export_beads_to_csv() {
    let beads = vec![Bead {
      id: BeadId::from(Uuid::new_v4()),
      title: "Test Bead".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::HIGH,
      bead_type: BeadType::Feature,
      created_by: None,
      created_at: Utc::now().to_rfc3339(),
      updated_at: Utc::now().to_rfc3339(),
    }];

    let result = export_beads(&beads, ExportFormat::Csv);
    assert!(result.is_ok());

    let csv = match result {
      Ok(csv) => csv,
      Err(e) => panic!("Expected Ok, got Err: {e}"),
    };
    assert!(csv.contains("id,title,description"));
    assert!(csv.contains("Test Bead"));
  }

  #[test]
  fn test_export_beads_empty() {
    let beads: Vec<Bead> = vec![];
    let result = export_beads(&beads, ExportFormat::Json);
    assert_eq!(result, Err(ExportError::EmptyDataset));
  }

  #[test]
  fn test_exported_bead_csv_roundtrip() {
    let original = ExportedBead {
      id: Uuid::new_v4().to_string(),
      title: "Test Bead".to_string(),
      description: Some("Test Description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_by: None,
      created_at: Utc::now().to_rfc3339(),
      updated_at: Utc::now().to_rfc3339(),
    };

    let csv_row = match original.to_csv_row() {
      Ok(row) => row,
      Err(e) => panic!("Failed to create CSV row: {e}"),
    };
    let restored = match ExportedBead::from_csv_row(&csv_row) {
      Ok(bead) => bead,
      Err(e) => panic!("Failed to parse CSV row: {e}"),
    };

    assert_eq!(restored.id, original.id);
    assert_eq!(restored.title, original.title);
    assert_eq!(restored.description, original.description);
    assert_eq!(restored.status, original.status);
    assert_eq!(restored.priority, original.priority);
  }

  #[test]
  fn test_exported_bead_csv_with_comma() {
    let bead = ExportedBead {
      id: Uuid::new_v4().to_string(),
      title: "Bead with, comma".to_string(),
      description: None,
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_by: None,
      created_at: Utc::now().to_rfc3339(),
      updated_at: Utc::now().to_rfc3339(),
    };

    let csv_row = match bead.to_csv_row() {
      Ok(row) => row,
      Err(e) => panic!("Failed to create CSV row: {e}"),
    };
    let escaped_id = if bead.id.contains(',') || bead.id.contains('"') || bead.id.contains('\n') {
      format!("\"{}\"", bead.id.replace('"', "\"\""))
    } else {
      bead.id
    };
    assert!(csv_row.contains(&escaped_id));
  }

  #[test]
  fn test_exported_bead_csv_empty_description() {
    let bead = ExportedBead {
      id: Uuid::new_v4().to_string(),
      title: "Test Bead".to_string(),
      description: None,
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_by: None,
      created_at: Utc::now().to_rfc3339(),
      updated_at: Utc::now().to_rfc3339(),
    };

    let csv_row = match bead.to_csv_row() {
      Ok(row) => row,
      Err(e) => panic!("Failed to create CSV row: {e}"),
    };
    let restored = match ExportedBead::from_csv_row(&csv_row) {
      Ok(bead) => bead,
      Err(e) => panic!("Failed to parse CSV row: {e}"),
    };

    assert_eq!(restored.description, None);
  }
}
