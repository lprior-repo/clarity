use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BeadError {
  #[error("empty bead title")]
  EmptyTitle,
  #[error("empty bead description")]
  EmptyDescription,
  #[error("invalid priority: {0}")]
  InvalidPriority(u8),
  #[error("missing profile type")]
  MissingProfileType,
  #[error("JSON serialization failed: {0}")]
  JsonError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadTemplate {
  pub title: String,
  pub description: String,
  pub profile_type: String,
  pub priority: u8,
  pub issue_type: String,
  pub labels: Vec<String>,
  pub ai_hints: String,
  pub acceptance_criteria: Vec<String>,
  pub dependencies: Vec<String>,
}

impl Default for BeadTemplate {
  fn default() -> Self {
    Self {
      title: String::new(),
      description: String::new(),
      profile_type: String::new(),
      priority: 3,
      issue_type: "task".to_string(),
      labels: Vec::new(),
      ai_hints: String::new(),
      acceptance_criteria: Vec::new(),
      dependencies: Vec::new(),
    }
  }
}

impl BeadTemplate {
  /// Creates a validated bead template.
  ///
  /// # Errors
  /// Returns `BeadError` when title, description, profile type, or priority are invalid.
  pub fn new(
    title: String,
    description: String,
    profile_type: String,
    priority: u8,
  ) -> Result<Self, BeadError> {
    if title.trim().is_empty() {
      return Err(BeadError::EmptyTitle);
    }
    if description.trim().is_empty() {
      return Err(BeadError::EmptyDescription);
    }
    if !(1..=5).contains(&priority) {
      return Err(BeadError::InvalidPriority(priority));
    }
    if profile_type.trim().is_empty() {
      return Err(BeadError::MissingProfileType);
    }
    Ok(Self {
      title,
      description,
      profile_type,
      priority,
      ..Self::default()
    })
  }

  #[must_use]
  pub fn with_label(self, label: String) -> Self {
    let labels = self
      .labels
      .iter()
      .cloned()
      .chain(std::iter::once(label))
      .unique()
      .collect();
    Self { labels, ..self }
  }

  #[must_use]
  pub fn with_issue_type(self, issue_type: String) -> Self {
    Self { issue_type, ..self }
  }

  #[must_use]
  pub fn with_ai_hints(self, hints: String) -> Self {
    Self {
      ai_hints: hints,
      ..self
    }
  }

  #[must_use]
  pub fn with_acceptance_criterion(self, criterion: String) -> Self {
    let acceptance_criteria = if criterion.trim().is_empty() {
      self.acceptance_criteria.clone()
    } else {
      self
        .acceptance_criteria
        .iter()
        .cloned()
        .chain(std::iter::once(criterion))
        .collect()
    };
    Self {
      acceptance_criteria,
      ..self
    }
  }

  #[must_use]
  pub fn with_dependency(self, dependency: String) -> Self {
    let dependencies = if dependency.trim().is_empty() {
      self.dependencies.clone()
    } else {
      self
        .dependencies
        .iter()
        .cloned()
        .chain(std::iter::once(dependency))
        .unique()
        .collect()
    };
    Self {
      dependencies,
      ..self
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BeadTemplateStats {
  pub total_beads: usize,
  pub by_priority: HashMap<u8, usize>,
  pub by_type: HashMap<String, usize>,
  pub by_profile: HashMap<String, usize>,
}

impl BeadTemplateStats {
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  #[must_use]
  pub fn from_beads(beads: &[BeadTemplate]) -> Self {
    Self {
      total_beads: beads.len(),
      by_priority: beads.iter().map(|bead| bead.priority).counts(),
      by_type: beads.iter().map(|bead| bead.issue_type.clone()).counts(),
      by_profile: beads.iter().map(|bead| bead.profile_type.clone()).counts(),
    }
  }
}

#[must_use]
pub fn bead_id_for_index(index: usize) -> String {
  format!("bd-{:04x}", index + 1)
}

#[must_use]
pub fn enhanced_bead_entry(bead: &BeadTemplate, index: usize) -> String {
  let id = bead_id_for_index(index);
  format!(
    r#"{{ id: "{}", title: "{}", description: "{}", type: "{}", priority: {} }}"#,
    id, bead.title, bead.description, bead.issue_type, bead.priority
  )
}

#[must_use]
pub fn with_validation_header(beads: &[BeadTemplate]) -> String {
  let mut output = String::from("# Validation Header\n");
  let _ = writeln!(output, "Total beads: {}", beads.len());

  let stats = BeadTemplateStats::from_beads(beads);
  let _ = writeln!(output, "By priority: {:?}", stats.by_priority);
  let _ = writeln!(output, "By type: {:?}", stats.by_type);

  output.push_str("\n# Beads\n");
  output
}

#[must_use]
pub fn filter_beads_by_type<'a>(
  beads: &'a [BeadTemplate],
  issue_type: &str,
) -> Vec<&'a BeadTemplate> {
  beads
    .iter()
    .filter(|b| b.issue_type == issue_type)
    .collect()
}

pub fn sort_beads_by_priority(beads: &mut [BeadTemplate]) {
  beads.sort_by_key(|b| b.priority);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bead_id_for_index() {
    assert_eq!(bead_id_for_index(0), "bd-0001");
    assert_eq!(bead_id_for_index(15), "bd-0010");
    assert_eq!(bead_id_for_index(255), "bd-0100");
  }

  #[test]
  fn test_enhanced_bead_entry() {
    let bead = BeadTemplate {
      title: "Test Bead".to_string(),
      description: "Test description".to_string(),
      issue_type: "feature".to_string(),
      priority: 2,
      ..Default::default()
    };

    let entry = enhanced_bead_entry(&bead, 0);
    assert!(entry.contains("bd-0001"));
    assert!(entry.contains("Test Bead"));
  }

  #[test]
  fn test_filter_beads_by_type() {
    let beads = vec![
      BeadTemplate {
        issue_type: "feature".to_string(),
        ..Default::default()
      },
      BeadTemplate {
        issue_type: "bug".to_string(),
        ..Default::default()
      },
      BeadTemplate {
        issue_type: "feature".to_string(),
        ..Default::default()
      },
    ];

    let features = filter_beads_by_type(&beads, "feature");
    assert_eq!(features.len(), 2);

    let bugs = filter_beads_by_type(&beads, "bug");
    assert_eq!(bugs.len(), 1);
  }

  #[test]
  fn test_sort_beads_by_priority() {
    let mut beads = vec![
      BeadTemplate {
        priority: 3,
        ..Default::default()
      },
      BeadTemplate {
        priority: 1,
        ..Default::default()
      },
      BeadTemplate {
        priority: 2,
        ..Default::default()
      },
    ];

    sort_beads_by_priority(&mut beads);
    assert_eq!(beads[0].priority, 1);
    assert_eq!(beads[1].priority, 2);
    assert_eq!(beads[2].priority, 3);
  }

  #[test]
  fn test_with_validation_header() {
    let beads = vec![
      BeadTemplate {
        priority: 1,
        issue_type: "feature".to_string(),
        ..Default::default()
      },
      BeadTemplate {
        priority: 2,
        issue_type: "bug".to_string(),
        ..Default::default()
      },
    ];

    let header = with_validation_header(&beads);
    assert!(header.contains("Total beads: 2"));
    assert!(header.contains("By priority:"));
  }
}
