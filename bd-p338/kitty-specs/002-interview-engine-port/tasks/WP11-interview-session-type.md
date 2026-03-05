---
work_package_id: WP11
title: InterviewSession Type
lane: planned
dependencies: []
subtasks: [T052, T053, T054, T055, T056]
---

# WP11: InterviewSession Type

## Objective

Define the central interview state machine type with all lifecycle methods.

## Context

- **Source**: `/tmp/intent-cli/src/intent/interview.gleam` (lines 200-400)
- **Target**: `clarity-web/src/intent/interview/types.rs`
- **Priority**: P0 (Critical)

## Contract Specification

### State Machine

```
Discovery → Refinement → Validation → Complete
    ↑            ↓            ↓
    └──────── Paused ←───────┘
```

### Preconditions

| ID | Precondition |
|----|--------------|
| P1 | Session ID is non-empty |
| P2 | Timestamps are valid ISO 8601 |
| P3 | No stage regression (Complete → Discovery) |

### Postconditions

| ID | Postcondition |
|----|---------------|
| Q1 | New session starts in Discovery stage |
| Q2 | updated_at is refreshed on every mutation |
| Q3 | completed_at is set when reaching Complete |

---

## Type Definition

### T052: InterviewSession struct

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterviewSession {
    /// Unique session identifier
    pub id: String,
    /// System profile type
    pub profile: Profile,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
    /// Completion timestamp (if complete)
    pub completed_at: Option<String>,
    /// Current interview stage
    pub stage: InterviewStage,
    /// Number of completed rounds
    pub rounds_completed: u32,
    /// Collected answers
    #[serde(default)]
    pub answers: Vec<Answer>,
    /// Detected gaps
    #[serde(default)]
    pub gaps: Vec<Gap>,
    /// Detected conflicts
    #[serde(default)]
    pub conflicts: Vec<Conflict>,
    /// Unstructured notes
    #[serde(default)]
    pub raw_notes: String,
    /// Current phase number
    pub current_phase: u32,
    /// Completed phase numbers
    #[serde(default)]
    pub completed_phases: Vec<u32>,
}
```

### T053: Implement InterviewSession::new()

```rust
impl InterviewSession {
    pub fn new(id: String, profile: Profile, timestamp: String) -> Result<Self, InterviewError> {
        if id.is_empty() {
            return Err(InterviewError::EmptySessionId);
        }
        Ok(Self {
            id,
            profile,
            created_at: timestamp.clone(),
            updated_at: timestamp,
            completed_at: None,
            stage: InterviewStage::Discovery,
            rounds_completed: 0,
            answers: Vec::new(),
            gaps: Vec::new(),
            conflicts: Vec::new(),
            raw_notes: String::new(),
            current_phase: 1,
            completed_phases: Vec::new(),
        })
    }
}
```

### T056: State transition validation

```rust
impl InterviewSession {
    /// Check if transition to new stage is valid
    pub fn can_transition_to(&self, new_stage: InterviewStage) -> Result<(), InterviewError> {
        use InterviewStage::*;
        match (&self.stage, &new_stage) {
            // Valid transitions
            (Discovery, Refinement) => Ok(()),
            (Refinement, Validation) => Ok(()),
            (Validation, Complete) => Ok(()),
            (Paused, Discovery) | (Paused, Refinement) | (Paused, Validation) => Ok(()),
            (s, Paused) => Ok(()), // Any stage can pause

            // Invalid transitions
            (Complete, _) => Err(InterviewError::StageTransitionInvalid {
                from: self.stage.clone(),
                to: new_stage,
                reason: "Cannot transition from Complete".to_string(),
            }),
            (from, to) if from > to => Err(InterviewError::StageTransitionInvalid {
                from: from.clone(),
                to: to.clone(),
                reason: "Cannot regress stages".to_string(),
            }),
            _ => Ok(()),
        }
    }
}
```

---

## Definition of Done

- [ ] InterviewSession struct complete
- [ ] new() constructor with validation
- [ ] State transition validation
- [ ] Round-trip serialization
