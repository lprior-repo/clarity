//! Plan Types - Core data structures for execution planning
//!
//! This module defines types for planning work items (beads) and execution:
//! - `PlanBead` - A work item with dependencies and phase assignment
//! - `ExecutionPlan` - Complete plan with phases and ordering
//! - `PlanError` - Errors that can occur during planning

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during planning operations
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlanError {
    /// Bead ID is empty
    #[error("bead ID cannot be empty")]
    EmptyBeadId,

    /// Bead title is empty
    #[error("bead title cannot be empty")]
    EmptyBeadTitle,

    /// Dependency refers to non-existent bead
    #[error("dependency '{dependency}' refers to non-existent bead '{bead_id}'")]
    InvalidDependency {
        /// The bead that has the invalid dependency
        bead_id: String,
        /// The dependency that doesn't exist
        dependency: String,
    },

    /// Circular dependency detected
    #[error("circular dependency detected in bead graph")]
    CircularDependency,

    /// Phase number is invalid
    #[error("invalid phase number: {phase_number}")]
    InvalidPhaseNumber {
        /// The invalid phase number
        phase_number: u32,
    },

    /// Session ID is empty
    #[error("session ID cannot be empty")]
    EmptySessionId,

    /// No beads to plan
    #[error("no beads available to plan")]
    NoBeads,

    /// Duplicate bead ID
    #[error("duplicate bead ID: {0}")]
    DuplicateBeadId(String),
}

/// A work item (bead) in the execution plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBead {
    /// Unique bead identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    #[serde(default)]
    pub description: String,
    /// Phase this bead belongs to (1-indexed)
    pub phase: u32,
    /// Priority within phase (lower = higher priority)
    #[serde(default)]
    pub priority: u32,
    /// IDs of beads this depends on
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Whether this bead is ready to execute
    #[serde(default)]
    pub ready: bool,
    /// Whether this bead is completed
    #[serde(default)]
    pub completed: bool,
    /// Estimated effort (story points or hours)
    #[serde(default)]
    pub effort: u32,
    /// Tags for categorization
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
    /// Create a new plan bead
    ///
    /// # Errors
    /// Returns `PlanError::EmptyBeadId` if id is empty
    /// Returns `PlanError::EmptyBeadTitle` if title is empty
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

    /// Builder method to add description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Builder method to set priority
    #[must_use]
    pub const fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Builder method to add dependency
    #[must_use]
    pub fn with_dependency(mut self, dependency: String) -> Self {
        if !self.dependencies.contains(&dependency) {
            self.dependencies.push(dependency);
        }
        self
    }

    /// Builder method to set effort
    #[must_use]
    pub const fn with_effort(mut self, effort: u32) -> Self {
        self.effort = effort;
        self
    }

    /// Builder method to add tag
    #[must_use]
    pub fn with_tag(mut self, tag: String) -> Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    /// Check if all dependencies are satisfied
    #[must_use]
    pub fn dependencies_satisfied(&self, completed_ids: &[&str]) -> bool {
        self.dependencies
            .iter()
            .all(|dep| completed_ids.contains(&dep.as_str()))
    }
}

/// Phase in the execution plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanPhase {
    /// Phase number (1-indexed)
    pub number: u32,
    /// Phase name
    #[serde(default)]
    pub name: String,
    /// Phase description
    #[serde(default)]
    pub description: String,
    /// Bead IDs in this phase (in execution order)
    #[serde(default)]
    pub beads: Vec<String>,
    /// Whether this phase is complete
    #[serde(default)]
    pub complete: bool,
}

impl Default for PlanPhase {
    fn default() -> Self {
        Self {
            number: 1,
            name: String::new(),
            description: String::new(),
            beads: Vec::new(),
            complete: false,
        }
    }
}

impl PlanPhase {
    /// Create a new phase
    #[must_use]
    pub fn new(number: u32, name: String) -> Self {
        Self {
            number,
            name,
            ..Self::default()
        }
    }

    /// Add a bead to this phase
    pub fn add_bead(&mut self, bead_id: String) {
        if !self.beads.contains(&bead_id) {
            self.beads.push(bead_id);
        }
    }
}

/// Complete execution plan with phases and ordering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Session ID this plan belongs to
    #[serde(default)]
    pub session_id: String,
    /// All beads in the plan
    #[serde(default)]
    pub beads: Vec<PlanBead>,
    /// Phases in the plan
    #[serde(default)]
    pub phases: Vec<PlanPhase>,
    /// Current phase number
    #[serde(default)]
    pub current_phase: u32,
    /// Bead IDs in execution order (topologically sorted)
    #[serde(default)]
    pub execution_order: Vec<String>,
    /// Whether the plan has been validated
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
    /// Create a new execution plan
    #[must_use]
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            ..Self::default()
        }
    }

    /// Add a bead to the plan
    ///
    /// # Errors
    /// Returns `PlanError::DuplicateBeadId` if a bead with the same ID exists
    pub fn add_bead(&mut self, bead: PlanBead) -> Result<(), PlanError> {
        if self.beads.iter().any(|b| b.id == bead.id) {
            return Err(PlanError::DuplicateBeadId(bead.id));
        }
        self.beads.push(bead);
        self.validated = false;
        Ok(())
    }

    /// Get a bead by ID
    #[must_use]
    pub fn get_bead(&self, id: &str) -> Option<&PlanBead> {
        self.beads.iter().find(|b| b.id == id)
    }

    /// Get a mutable bead by ID
    pub fn get_bead_mut(&mut self, id: &str) -> Option<&mut PlanBead> {
        self.beads.iter_mut().find(|b| b.id == id)
    }

    /// Get beads for a specific phase
    #[must_use]
    pub fn get_phase_beads(&self, phase: u32) -> Vec<&PlanBead> {
        self.beads.iter().filter(|b| b.phase == phase).collect()
    }

    /// Get completed bead IDs
    #[must_use]
    pub fn get_completed_ids(&self) -> Vec<&str> {
        self.beads
            .iter()
            .filter(|b| b.completed)
            .map(|b| b.id.as_str())
            .collect()
    }

    /// Get actionable beads (ready and not completed)
    #[must_use]
    pub fn get_actionable_beads(&self) -> Vec<&PlanBead> {
        let completed_ids = self.get_completed_ids();
        self.beads
            .iter()
            .filter(|b| !b.completed && b.dependencies_satisfied(&completed_ids))
            .collect()
    }

    /// Mark a bead as completed
    ///
    /// # Errors
    /// Returns `PlanError::EmptyBeadId` if id is empty
    /// Returns error if bead not found (via `get_bead_mut`)
    pub fn complete_bead(&mut self, id: &str) -> Result<(), PlanError> {
        if id.trim().is_empty() {
            return Err(PlanError::EmptyBeadId);
        }

        let bead = self
            .get_bead_mut(id)
            .ok_or_else(|| PlanError::InvalidDependency {
                bead_id: id.to_string(),
                dependency: "self".to_string(),
            })?;
        bead.completed = true;
        bead.ready = false;
        Ok(())
    }

    /// Get total effort estimate
    #[must_use]
    pub fn total_effort(&self) -> u32 {
        self.beads.iter().map(|b| b.effort).sum()
    }

    /// Get completed effort
    #[must_use]
    pub fn completed_effort(&self) -> u32 {
        self.beads
            .iter()
            .filter(|b| b.completed)
            .map(|b| b.effort)
            .sum()
    }

    /// Get progress percentage (0-100)
    #[must_use]
    pub fn progress_percentage(&self) -> f64 {
        if self.beads.is_empty() {
            return 0.0;
        }
        let completed = self.beads.iter().filter(|b| b.completed).count();
        (completed as f64 / self.beads.len() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_bead_new_valid() {
        let bead = PlanBead::new("bead-1".to_string(), "First bead".to_string(), 1);
        assert!(bead.is_ok());
        let bead = bead.expect("valid bead");
        assert_eq!(bead.id, "bead-1");
        assert_eq!(bead.title, "First bead");
        assert_eq!(bead.phase, 1);
    }

    #[test]
    fn test_plan_bead_new_empty_id() {
        let result = PlanBead::new(String::new(), "Title".to_string(), 1);
        assert!(matches!(result, Err(PlanError::EmptyBeadId)));
    }

    #[test]
    fn test_plan_bead_new_empty_title() {
        let result = PlanBead::new("id".to_string(), String::new(), 1);
        assert!(matches!(result, Err(PlanError::EmptyBeadTitle)));
    }

    #[test]
    fn test_plan_bead_dependencies_satisfied() {
        let bead = PlanBead::new("bead-2".to_string(), "Second".to_string(), 1)
            .expect("valid")
            .with_dependency("bead-1".to_string());

        assert!(!bead.dependencies_satisfied(&[]));
        assert!(!bead.dependencies_satisfied(&["other"]));
        assert!(bead.dependencies_satisfied(&["bead-1"]));
        assert!(bead.dependencies_satisfied(&["bead-1", "other"]));
    }

    #[test]
    fn test_plan_bead_builder() {
        let bead = PlanBead::new("bead-1".to_string(), "Test".to_string(), 2)
            .expect("valid")
            .with_description("Description".to_string())
            .with_priority(5)
            .with_effort(3)
            .with_tag("core".to_string())
            .with_tag("api".to_string());

        assert_eq!(bead.description, "Description");
        assert_eq!(bead.priority, 5);
        assert_eq!(bead.effort, 3);
        assert_eq!(bead.tags, vec!["core", "api"]);
    }

    #[test]
    fn test_execution_plan_add_bead() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        let bead = PlanBead::new("bead-1".to_string(), "First".to_string(), 1).expect("valid");

        let result = plan.add_bead(bead);
        assert!(result.is_ok());
        assert_eq!(plan.beads.len(), 1);
        assert!(!plan.validated);
    }

    #[test]
    fn test_execution_plan_duplicate_bead() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        let bead1 = PlanBead::new("bead-1".to_string(), "First".to_string(), 1).expect("valid");
        let bead2 = PlanBead::new("bead-1".to_string(), "Second".to_string(), 1).expect("valid");

        plan.add_bead(bead1).expect("should add");
        let result = plan.add_bead(bead2);
        assert!(matches!(result, Err(PlanError::DuplicateBeadId(_))));
    }

    #[test]
    fn test_execution_plan_get_bead() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        let bead = PlanBead::new("bead-1".to_string(), "First".to_string(), 1).expect("valid");
        plan.add_bead(bead).expect("should add");

        let found = plan.get_bead("bead-1");
        assert!(found.is_some());
        assert_eq!(found.map(|b| b.title.as_str()), Some("First"));

        let not_found = plan.get_bead("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_execution_plan_get_phase_beads() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        plan.add_bead(
            PlanBead::new("b1".to_string(), "Phase 1".to_string(), 1).expect("valid"),
        )
        .expect("should add");
        plan.add_bead(
            PlanBead::new("b2".to_string(), "Phase 2a".to_string(), 2).expect("valid"),
        )
        .expect("should add");
        plan.add_bead(
            PlanBead::new("b3".to_string(), "Phase 2b".to_string(), 2).expect("valid"),
        )
        .expect("should add");

        let phase1 = plan.get_phase_beads(1);
        assert_eq!(phase1.len(), 1);

        let phase2 = plan.get_phase_beads(2);
        assert_eq!(phase2.len(), 2);

        let phase3 = plan.get_phase_beads(3);
        assert!(phase3.is_empty());
    }

    #[test]
    fn test_execution_plan_complete_bead() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        plan.add_bead(
            PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid"),
        )
        .expect("should add");

        let result = plan.complete_bead("b1");
        assert!(result.is_ok());
        assert!(plan.get_bead("b1").map_or(false, |b| b.completed));
    }

    #[test]
    fn test_execution_plan_complete_bead_empty() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        let result = plan.complete_bead("");
        assert!(matches!(result, Err(PlanError::EmptyBeadId)));
    }

    #[test]
    fn test_execution_plan_progress() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        assert_eq!(plan.progress_percentage(), 0.0);

        plan.add_bead(
            PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid"),
        )
        .expect("should add");
        plan.add_bead(
            PlanBead::new("b2".to_string(), "Second".to_string(), 1).expect("valid"),
        )
        .expect("should add");

        assert_eq!(plan.progress_percentage(), 0.0);

        plan.complete_bead("b1").expect("should complete");
        assert_eq!(plan.progress_percentage(), 50.0);

        plan.complete_bead("b2").expect("should complete");
        assert_eq!(plan.progress_percentage(), 100.0);
    }

    #[test]
    fn test_execution_plan_effort() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        plan.add_bead(
            PlanBead::new("b1".to_string(), "First".to_string(), 1)
                .expect("valid")
                .with_effort(3),
        )
        .expect("should add");
        plan.add_bead(
            PlanBead::new("b2".to_string(), "Second".to_string(), 1)
                .expect("valid")
                .with_effort(5),
        )
        .expect("should add");

        assert_eq!(plan.total_effort(), 8);
        assert_eq!(plan.completed_effort(), 0);

        plan.complete_bead("b1").expect("should complete");
        assert_eq!(plan.completed_effort(), 3);
    }

    #[test]
    fn test_execution_plan_get_actionable_beads() {
        let mut plan = ExecutionPlan::new("session-1".to_string());

        // b1 has no dependencies
        plan.add_bead(
            PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid"),
        )
        .expect("should add");

        // b2 depends on b1
        plan.add_bead(
            PlanBead::new("b2".to_string(), "Second".to_string(), 1)
                .expect("valid")
                .with_dependency("b1".to_string()),
        )
        .expect("should add");

        // b3 depends on b2
        plan.add_bead(
            PlanBead::new("b3".to_string(), "Third".to_string(), 1)
                .expect("valid")
                .with_dependency("b2".to_string()),
        )
        .expect("should add");

        // Initially only b1 is actionable
        let actionable = plan.get_actionable_beads();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].id, "b1");

        // Complete b1
        plan.complete_bead("b1").expect("should complete");

        // Now b2 is actionable
        let actionable = plan.get_actionable_beads();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].id, "b2");

        // Complete b2
        plan.complete_bead("b2").expect("should complete");

        // Now b3 is actionable
        let actionable = plan.get_actionable_beads();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].id, "b3");
    }

    #[test]
    fn test_plan_phase() {
        let mut phase = PlanPhase::new(1, "Discovery".to_string());
        assert_eq!(phase.number, 1);
        assert_eq!(phase.name, "Discovery");

        phase.add_bead("b1".to_string());
        phase.add_bead("b2".to_string());
        phase.add_bead("b1".to_string()); // Duplicate, should not add

        assert_eq!(phase.beads, vec!["b1", "b2"]);
    }

    #[test]
    fn test_serde_roundtrip_plan_bead() {
        let bead = PlanBead::new("bead-1".to_string(), "Test Bead".to_string(), 2)
            .expect("valid")
            .with_description("A test bead".to_string())
            .with_priority(3)
            .with_dependency("bead-0".to_string())
            .with_effort(5)
            .with_tag("core".to_string());

        let json = serde_json::to_string(&bead).expect("should serialize");
        let parsed: PlanBead = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(bead, parsed);
    }

    #[test]
    fn test_serde_roundtrip_execution_plan() {
        let mut plan = ExecutionPlan::new("session-1".to_string());
        plan.add_bead(
            PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid"),
        )
        .expect("should add");
        plan.execution_order = vec!["b1".to_string()];

        let json = serde_json::to_string(&plan).expect("should serialize");
        let parsed: ExecutionPlan = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(plan, parsed);
    }
}
