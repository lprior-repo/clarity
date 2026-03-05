use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
