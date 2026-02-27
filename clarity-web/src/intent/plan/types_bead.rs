use super::types_error::PlanError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBead {
  pub id: String,
  pub title: String,
  #[serde(default)]
  pub description: String,
  pub phase: u32,
  #[serde(default)]
  pub priority: u32,
  #[serde(default)]
  pub dependencies: Vec<String>,
  #[serde(default)]
  pub ready: bool,
  #[serde(default)]
  pub completed: bool,
  #[serde(default)]
  pub effort: u32,
  #[serde(default)]
  pub tags: Vec<String>,
}

impl Default for PlanBead {
  fn default() -> Self {
    Self {
      id: String::new(),
      title: String::new(),
      description: String::new(),
      phase: 1,
      priority: 0,
      dependencies: Vec::new(),
      ready: false,
      completed: false,
      effort: 0,
      tags: Vec::new(),
    }
  }
}

impl PlanBead {
  pub fn new(id: String, title: String, phase: u32) -> Result<Self, PlanError> {
    if id.trim().is_empty() {
      return Err(PlanError::EmptyBeadId);
    }
    if title.trim().is_empty() {
      return Err(PlanError::EmptyBeadTitle);
    }
    Ok(Self {
      id,
      title,
      phase,
      ..Self::default()
    })
  }

  #[must_use]
  pub fn with_description(self, description: String) -> Self {
    Self {
      description,
      ..self
    }
  }

  #[must_use]
  pub fn with_priority(self, priority: u32) -> Self {
    Self { priority, ..self }
  }

  #[must_use]
  pub fn with_dependency(self, dependency: String) -> Self {
    let dependencies = self
      .dependencies
      .iter()
      .cloned()
      .chain((!self.dependencies.contains(&dependency)).then_some(dependency))
      .collect();
    Self {
      dependencies,
      ..self
    }
  }

  #[must_use]
  pub fn with_effort(self, effort: u32) -> Self {
    Self { effort, ..self }
  }

  #[must_use]
  pub fn with_tag(self, tag: String) -> Self {
    let tags = self
      .tags
      .iter()
      .cloned()
      .chain((!self.tags.contains(&tag)).then_some(tag))
      .collect();
    Self { tags, ..self }
  }

  #[must_use]
  pub fn dependencies_satisfied(&self, completed_ids: &[&str]) -> bool {
    self
      .dependencies
      .iter()
      .all(|dep| completed_ids.contains(&dep.as_str()))
  }
}
