#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Bead import functionality
//!
//! Pure functions for importing beads from JSON and CSV formats.
//! All validation and transformation is done without side effects.

use crate::db::models::{Bead, BeadPriority, BeadStatus, BeadType, NewBead};
use crate::export::{BeadExport, ExportError, ExportedBead};
use rpds::Vector;
use std::collections::HashSet;
use std::str::FromStr;
use thiserror::Error;

/// Import conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
  /// Skip beads that already exist (by ID)
  Skip,
  /// Replace existing beads with imported data
  Replace,
  /// Merge imported data with existing beads (imported fields take precedence)
  Merge,
}

/// Import preview showing what will change
#[derive(Debug, Clone)]
pub struct ImportPreview {
  /// Beads that will be added
  pub to_add: Vector<ExportedBead>,
  /// Beads that will be replaced
  pub to_replace: Vector<ExportedBead>,
  /// Beads that will be merged
  pub to_merge: Vector<ExportedBead>,
  /// Beads that will be skipped
  pub to_skip: Vector<ExportedBead>,
  /// Errors encountered during validation
  pub errors: Vector<ImportError>,
}

impl ImportPreview {
  /// Create a new empty preview
  #[must_use]
  pub fn new() -> Self {
    Self {
      to_add: Vector::new(),
      to_replace: Vector::new(),
      to_merge: Vector::new(),
      to_skip: Vector::new(),
      errors: Vector::new(),
    }
  }

  /// Total number of beads to process
  #[must_use]
  pub fn total_count(&self) -> usize {
    self.to_add.len() + self.to_replace.len() + self.to_merge.len() + self.to_skip.len()
  }

  /// Check if any errors occurred
  #[must_use]
  pub fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  /// Check if any changes will be made
  #[must_use]
  pub fn has_changes(&self) -> bool {
    !self.to_add.is_empty() || !self.to_replace.is_empty() || !self.to_merge.is_empty()
  }

  /// Add a bead to the appropriate list based on conflict resolution
  #[must_use]
  pub fn add_bead(&self, bead: ExportedBead, resolution: ConflictResolution) -> Self {
    match resolution {
      ConflictResolution::Skip => Self {
        to_skip: self.to_skip.push_back(bead),
        ..self.clone()
      },
      ConflictResolution::Replace => Self {
        to_replace: self.to_replace.push_back(bead),
        ..self.clone()
      },
      ConflictResolution::Merge => Self {
        to_merge: self.to_merge.push_back(bead),
        ..self.clone()
      },
    }
  }

  /// Add an error to the preview
  #[must_use]
  pub fn add_error(&self, error: ImportError) -> Self {
    Self {
      errors: self.errors.push_back(error),
      ..self.clone()
    }
  }
}

impl Default for ImportPreview {
  fn default() -> Self {
    Self::new()
  }
}

/// Import errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ImportError {
  #[error("JSON parsing failed: {0}")]
  JsonParse(String),

  #[error("Invalid JSON structure: {0}")]
  InvalidJson(String),

  #[error("CSV parsing failed: {0}")]
  CsvParse(String),

  #[error("Invalid bead ID: {0}")]
  InvalidId(String),

  #[error("Invalid user ID: {0}")]
  InvalidUserId(String),

  #[error("Invalid status: {0}")]
  InvalidStatus(String),

  #[error("Invalid type: {0}")]
  InvalidType(String),

  #[error("Invalid priority: {0}")]
  InvalidPriority(String),

  #[error("Invalid date format: {0}")]
  InvalidDateFormat(String),

  #[error("Missing required field: {0}")]
  MissingField(String),

  #[error("Empty title not allowed")]
  EmptyTitle,

  #[error("Data format error: {0}")]
  InvalidDataFormat(String),

  #[error("No valid beads found in import data")]
  NoValidBeads,
}

/// Import result type
pub type ImportResult<T> = Result<T, ImportError>;

/// Parsed import data with validation results
#[derive(Debug, Clone)]
pub struct ParsedImport {
  /// Successfully parsed beads
  pub valid_beads: Vector<ExportedBead>,
  /// Errors encountered during parsing
  pub errors: Vector<ImportError>,
}

impl ParsedImport {
  /// Create a new parsed import
  #[must_use]
  pub const fn new(valid_beads: Vector<ExportedBead>, errors: Vector<ImportError>) -> Self {
    Self {
      valid_beads,
      errors,
    }
  }

  /// Check if any beads were successfully parsed
  #[must_use]
  pub fn has_valid_beads(&self) -> bool {
    !self.valid_beads.is_empty()
  }

  /// Check if any errors occurred
  #[must_use]
  pub fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  /// Get the number of valid beads
  #[must_use]
  pub fn count(&self) -> usize {
    self.valid_beads.len()
  }
}

/// Parse beads from JSON string
///
/// # Errors
/// - Returns `ImportError::JsonParse` if JSON parsing fails
/// - Returns `ImportError::InvalidJson` if structure is invalid
pub fn parse_json(json: &str) -> ImportResult<ParsedImport> {
  // Parse JSON into BeadExport
  let export: BeadExport =
    serde_json::from_str(json).map_err(|e| ImportError::JsonParse(e.to_string()))?;

  // Validate version
  if export.version != "1.0" {
    return Err(ImportError::InvalidJson(format!(
      "Unsupported version: {}",
      export.version
    )));
  }

  // Validate and categorize beads
  let (valid_beads, errors) = export.beads.into_iter().map(validate_exported_bead).fold(
    (Vector::new(), Vector::new()),
    |(mut ok, mut err), item| match item {
      Ok(v) => {
        ok = ok.push_back(v);
        (ok, err)
      }
      Err(e) => {
        err = err.push_back(e);
        (ok, err)
      }
    },
  );

  Ok(ParsedImport::new(valid_beads, errors))
}

/// Parse beads from CSV string
///
/// # Errors
/// - Returns `ImportError::CsvParse` if CSV parsing fails
pub fn parse_csv(csv: &str) -> ImportResult<ParsedImport> {
  let mut lines = csv.lines();
  let header = lines
    .next()
    .ok_or(ImportError::CsvParse("Empty CSV".to_string()))?;

  // Validate header
  let expected_header =
    "id,title,description,status,priority,bead_type,created_by,created_at,updated_at";
  if header != expected_header {
    return Err(ImportError::CsvParse(format!(
      "Invalid header. Expected: {expected_header}"
    )));
  }

  // Parse each row
  let (valid_beads, errors) = lines
    .enumerate()
    .map(|(line_num, row)| {
      ExportedBead::from_csv_row(row).map_err(|e| match e {
        ExportError::CsvSerialization(msg) => ImportError::CsvParse(format!(
          "Line {}: {}",
          line_num + 2, // +2 because header is line 1
          msg
        )),
        _ => ImportError::CsvParse(format!("Line {}: Unknown error", line_num + 2)),
      })
    })
    .map(|result| result.and_then(validate_exported_bead))
    .fold(
      (Vector::new(), Vector::new()),
      |(mut ok, mut err), item| match item {
        Ok(v) => {
          ok = ok.push_back(v);
          (ok, err)
        }
        Err(e) => {
          err = err.push_back(e);
          (ok, err)
        }
      },
    );

  if valid_beads.is_empty() && errors.is_empty() {
    return Err(ImportError::NoValidBeads);
  }

  Ok(ParsedImport::new(valid_beads, errors))
}

/// Validate an exported bead
///
/// # Errors
/// Returns various `ImportError` types if validation fails
fn validate_exported_bead(bead: ExportedBead) -> ImportResult<ExportedBead> {
  // Validate title is not empty
  if bead.title.trim().is_empty() {
    return Err(ImportError::EmptyTitle);
  }

  // Validate status
  if BeadStatus::from_str(&bead.status).is_err() {
    return Err(ImportError::InvalidStatus(bead.status));
  }

  // Validate bead type
  if BeadType::from_str(&bead.bead_type).is_err() {
    return Err(ImportError::InvalidType(bead.bead_type));
  }

  // Validate priority
  if BeadPriority::new(bead.priority).is_err() {
    return Err(ImportError::InvalidPriority(bead.priority.to_string()));
  }

  // Validate ID format
  if uuid::Uuid::parse_str(&bead.id).is_err() {
    return Err(ImportError::InvalidId(bead.id));
  }

  // Validate user ID format if present
  if let Some(ref user_id) = bead.created_by {
    if uuid::Uuid::parse_str(user_id).is_err() {
      return Err(ImportError::InvalidUserId(user_id.clone()));
    }
  }

  // Validate date formats
  if chrono::DateTime::parse_from_rfc3339(&bead.created_at).is_err() {
    return Err(ImportError::InvalidDateFormat(bead.created_at));
  }

  if chrono::DateTime::parse_from_rfc3339(&bead.updated_at).is_err() {
    return Err(ImportError::InvalidDateFormat(bead.updated_at));
  }

  Ok(bead)
}

/// Preview import with conflict resolution
///
/// # Errors
/// Returns errors from parsing
pub fn preview_import(
  data: &str,
  format: crate::export::ExportFormat,
  existing_ids: &HashSet<String>,
  resolution: ConflictResolution,
) -> ImportResult<ImportPreview> {
  let parsed = match format {
    crate::export::ExportFormat::Json => parse_json(data)?,
    crate::export::ExportFormat::Csv => parse_csv(data)?,
  };

  let preview = parsed
    .valid_beads
    .iter()
    .fold(ImportPreview::new(), |acc, bead| {
      if existing_ids.contains(&bead.id) {
        match resolution {
          ConflictResolution::Skip => acc.add_bead(bead.clone(), ConflictResolution::Skip),
          ConflictResolution::Replace => acc.add_bead(bead.clone(), ConflictResolution::Replace),
          ConflictResolution::Merge => acc.add_bead(bead.clone(), ConflictResolution::Merge),
        }
      } else {
        // New bead - always add
        ImportPreview {
          to_add: acc.to_add.push_back(bead.clone()),
          ..acc
        }
      }
    });

  // Add parsing errors to preview
  let preview_with_errors = parsed
    .errors
    .iter()
    .fold(preview, |acc, error| acc.add_error(error.clone()));

  Ok(preview_with_errors)
}

/// Convert imported beads to domain beads
///
/// # Errors
/// Returns `ImportError::InvalidDateFormat` if date parsing fails
pub fn imported_to_domain(beads: Vector<ExportedBead>) -> ImportResult<Vector<Bead>> {
  beads.iter().map(exported_to_domain_bead).collect()
}

/// Convert a single exported bead to domain bead
///
/// # Errors
/// Returns `ImportError::InvalidDateFormat` if date parsing fails
fn exported_to_domain_bead(exported: &ExportedBead) -> ImportResult<Bead> {
  let created_at = chrono::DateTime::parse_from_rfc3339(&exported.created_at)
    .map_err(|_| ImportError::InvalidDateFormat(exported.created_at.clone()))?
    .with_timezone(&chrono::Utc);

  let updated_at = chrono::DateTime::parse_from_rfc3339(&exported.updated_at)
    .map_err(|_| ImportError::InvalidDateFormat(exported.updated_at.clone()))?
    .with_timezone(&chrono::Utc);

  Ok(Bead {
    id: crate::db::models::BeadId::from_str(&exported.id)
      .map_err(|_| ImportError::InvalidId(exported.id.clone()))?,
    title: exported.title.clone(),
    description: exported.description.clone(),
    status: BeadStatus::from_str(&exported.status)
      .map_err(|_| ImportError::InvalidStatus(exported.status.clone()))?,
    priority: BeadPriority::new(exported.priority)
      .map_err(|_| ImportError::InvalidPriority(exported.priority.to_string()))?,
    bead_type: BeadType::from_str(&exported.bead_type)
      .map_err(|_| ImportError::InvalidType(exported.bead_type.clone()))?,
    created_by: match &exported.created_by {
      Some(user_id) => Some(
        crate::db::models::UserId::from_str(user_id)
          .map_err(|_| ImportError::InvalidUserId(user_id.clone()))?,
      ),
      None => None,
    },
    created_at,
    updated_at,
  })
}

/// Convert imported beads to `NewBead` for database insertion
///
/// # Errors
/// Returns `ImportError::InvalidDateFormat` if date parsing fails
pub fn imported_to_new_beads(beads: Vector<ExportedBead>) -> ImportResult<Vector<NewBead>> {
  beads
    .iter()
    .map(|exported| {
      Ok(NewBead {
        title: exported.title.clone(),
        description: exported.description.clone(),
        status: BeadStatus::from_str(&exported.status)
          .map_err(|_| ImportError::InvalidStatus(exported.status.clone()))?,
        priority: BeadPriority::new(exported.priority)
          .map_err(|_| ImportError::InvalidPriority(exported.priority.to_string()))?,
        bead_type: BeadType::from_str(&exported.bead_type)
          .map_err(|_| ImportError::InvalidType(exported.bead_type.clone()))?,
        created_by: match &exported.created_by {
          Some(user_id) => Some(
            crate::db::models::UserId::from_str(user_id)
              .map_err(|_| ImportError::InvalidUserId(user_id.clone()))?,
          ),
          None => None,
        },
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::export::{ExportFormat, ExportedBead};
  use uuid::Uuid;

  #[test]
  fn test_parse_json_valid() {
    let json = r#"{
            "version": "1.0",
            "exported_at": "2024-01-01T00:00:00Z",
            "count": 1,
            "beads": [
                {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "title": "Test Bead",
                    "description": "Test Description",
                    "status": "open",
                    "priority": 1,
                    "bead_type": "feature",
                    "created_by": null,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ]
        }"#;

    let result = parse_json(json);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(parsed.has_valid_beads());
    assert_eq!(parsed.count(), 1);
    assert!(!parsed.has_errors());
  }

  #[test]
  fn test_parse_json_invalid_version() {
    let json = r#"{
            "version": "2.0",
            "exported_at": "2024-01-01T00:00:00Z",
            "count": 0,
            "beads": []
        }"#;

    let result = parse_json(json);
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().to_string(),
      "Invalid JSON structure: Unsupported version: 2.0"
    );
  }

  #[test]
  fn test_parse_json_invalid_status() {
    let json = r#"{
            "version": "1.0",
            "exported_at": "2024-01-01T00:00:00Z",
            "count": 1,
            "beads": [
                {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "title": "Test Bead",
                    "description": null,
                    "status": "invalid_status",
                    "priority": 1,
                    "bead_type": "feature",
                    "created_by": null,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ]
        }"#;

    let result = parse_json(json);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(!parsed.has_valid_beads());
    assert!(parsed.has_errors());
  }

  #[test]
  fn test_parse_csv_valid() {
    let csv = "id,title,description,status,priority,bead_type,created_by,created_at,updated_at\n\
            00000000-0000-0000-0000-000000000000,Test Bead,Test Description,open,1,feature,,2024-01-01T00:00:00Z,2024-01-01T00:00:00Z";

    let result = parse_csv(csv);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(parsed.has_valid_beads());
    assert_eq!(parsed.count(), 1);
  }

  #[test]
  fn test_parse_csv_invalid_header() {
    let csv = "invalid,header,here\n\
            00000000-0000-0000-0000-000000000000,Test Bead,Test Description,open,1,feature";

    let result = parse_csv(csv);
    assert!(result.is_err());
  }

  #[test]
  fn test_validate_exported_bead_empty_title() {
    let bead = ExportedBead {
      id: Uuid::new_v4().to_string(),
      title: "   ".to_string(),
      description: None,
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_by: None,
      created_at: chrono::Utc::now().to_rfc3339(),
      updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let result = validate_exported_bead(bead);
    assert!(matches!(result, Err(ImportError::EmptyTitle)));
  }

  #[test]
  fn test_import_preview_new() {
    let preview = ImportPreview::new();
    assert!(!preview.has_errors());
    assert!(!preview.has_changes());
    assert_eq!(preview.total_count(), 0);
  }

  #[test]
  fn test_preview_import_with_conflicts() {
    let csv = "id,title,description,status,priority,bead_type,created_by,created_at,updated_at\n\
            00000000-0000-0000-0000-000000000000,Bead 1,Description,open,1,feature,,2024-01-01T00:00:00Z,2024-01-01T00:00:00Z\n\
            00000000-0000-0000-0000-000000000001,Bead 2,Description,in_progress,2,bugfix,,2024-01-01T00:00:00Z,2024-01-01T00:00:00Z";

    let mut existing_ids = HashSet::new();
    existing_ids.insert("00000000-0000-0000-0000-000000000000".to_string());

    let preview = preview_import(
      csv,
      ExportFormat::Csv,
      &existing_ids,
      ConflictResolution::Skip,
    )
    .unwrap();

    assert_eq!(preview.to_add.len(), 1);
    assert_eq!(preview.to_skip.len(), 1);
    assert!(preview.has_changes());
  }

  #[test]
  fn test_imported_to_domain() {
    let beads = Vector::from_iter(vec![ExportedBead {
      id: "00000000-0000-0000-0000-000000000000".to_string(),
      title: "Test Bead".to_string(),
      description: Some("Description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_by: None,
      created_at: "2024-01-01T00:00:00Z".to_string(),
      updated_at: "2024-01-01T00:00:00Z".to_string(),
    }]);

    let result = imported_to_domain(beads);
    assert!(result.is_ok());

    let domain_beads = result.unwrap();
    assert_eq!(domain_beads.len(), 1);
    assert_eq!(domain_beads.first().unwrap().title, "Test Bead");
  }

  #[test]
  fn test_imported_to_new_beads() {
    let beads = Vector::from_iter(vec![ExportedBead {
      id: "00000000-0000-0000-0000-000000000000".to_string(),
      title: "Test Bead".to_string(),
      description: None,
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_by: None,
      created_at: "2024-01-01T00:00:00Z".to_string(),
      updated_at: "2024-01-01T00:00:00Z".to_string(),
    }]);

    let result = imported_to_new_beads(beads);
    assert!(result.is_ok());

    let new_beads = result.unwrap();
    assert_eq!(new_beads.len(), 1);
    assert_eq!(new_beads.first().unwrap().title, "Test Bead");
  }

  #[test]
  fn test_conflict_resolution_skip() {
    let preview = ImportPreview::new();
    let bead = ExportedBead {
      id: Uuid::new_v4().to_string(),
      title: "Test".to_string(),
      description: None,
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_by: None,
      created_at: chrono::Utc::now().to_rfc3339(),
      updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let result = preview.add_bead(bead, ConflictResolution::Skip);
    assert_eq!(result.to_skip.len(), 1);
    assert_eq!(result.to_replace.len(), 0);
    assert_eq!(result.to_merge.len(), 0);
  }
}
