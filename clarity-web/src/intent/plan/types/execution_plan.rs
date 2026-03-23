use super::bead::{BeadState, PlanBead};
use super::error::PlanError;
use super::phase::PlanPhase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
  #[serde(default)]
  pub session_id: String,
  #[serde(default)]
  pub beads: Vec<PlanBead>,
  #[serde(default)]
  pub phases: Vec<PlanPhase>,
  #[serde(default)]
  pub current_phase: u32,
  #[serde(default)]
  pub execution_order: Vec<String>,
  #[serde(default)]
  pub validated: bool,
}

impl Default for ExecutionPlan {
  fn default() -> Self {
    Self {
      session_id: String::new(),
      beads: Vec::new(),
      phases: Vec::new(),
      current_phase: 1,
      execution_order: Vec::new(),
      validated: false,
    }
  }
}

impl ExecutionPlan {
  #[must_use]
  pub fn new(session_id: String) -> Self {
    Self {
      session_id,
      ..Self::default()
    }
  }

  /// Adds a bead to the plan if its id is unique.
  ///
  /// # Errors
  /// Returns `PlanError::DuplicateBeadId` when another bead already has the same id.
  pub fn add_bead(&mut self, bead: PlanBead) -> Result<(), PlanError> {
    if self.beads.iter().any(|existing| existing.id == bead.id) {
      return Err(PlanError::DuplicateBeadId(bead.id));
    }
    self.beads.push(bead);
    self.validated = false;
    Ok(())
  }

  #[must_use]
  pub fn get_bead(&self, id: &str) -> Option<&PlanBead> {
    self.beads.iter().find(|bead| bead.id == id)
  }

  pub fn get_bead_mut(&mut self, id: &str) -> Option<&mut PlanBead> {
    self.beads.iter_mut().find(|bead| bead.id == id)
  }

  #[must_use]
  pub fn get_phase_beads(&self, phase: u32) -> Vec<&PlanBead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.phase == phase)
      .collect()
  }

  #[must_use]
  pub fn get_completed_ids(&self) -> Vec<&str> {
    self
      .beads
      .iter()
      .filter(|bead| bead.is_completed())
      .map(|bead| bead.id.as_str())
      .collect()
  }

  #[must_use]
  pub fn get_actionable_beads(&self) -> Vec<&PlanBead> {
    let completed_ids = self.get_completed_ids();
    self
      .beads
      .iter()
      .filter(|bead| !bead.is_completed() && bead.dependencies_satisfied(&completed_ids))
      .collect()
  }

  /// Marks an existing bead as completed.
  ///
  /// # Errors
  /// Returns `PlanError::EmptyBeadId` for blank ids, or `PlanError::InvalidDependency` when the bead id is not found.
  pub fn complete_bead(&mut self, id: &str) -> Result<(), PlanError> {
    if id.trim().is_empty() {
      return Err(PlanError::EmptyBeadId);
    }
    match self.get_bead_mut(id) {
      Some(bead) => {
        bead.state = BeadState::Completed;
        Ok(())
      }
      None => Err(PlanError::InvalidDependency {
        bead_id: id.to_string(),
        dependency: "self".to_string(),
      }),
    }
  }

  #[must_use]
  pub fn total_effort(&self) -> u32 {
    self.beads.iter().map(|bead| bead.effort).sum()
  }

  #[must_use]
  pub fn completed_effort(&self) -> u32 {
    self
      .beads
      .iter()
      .filter(|bead| bead.is_completed())
      .map(|bead| bead.effort)
      .sum()
  }

  #[must_use]
  pub fn progress_percentage(&self) -> f64 {
    if self.beads.is_empty() {
      return 0.0;
    }
    let completed = self.beads.iter().filter(|bead| bead.is_completed()).count();
    let completed_u32 = u32::try_from(completed).map_or(u32::MAX, |v| v);
    let total_u32 = u32::try_from(self.beads.len()).map_or(u32::MAX, |v| v);
    (f64::from(completed_u32) / f64::from(total_u32)) * 100.0
  }
}
