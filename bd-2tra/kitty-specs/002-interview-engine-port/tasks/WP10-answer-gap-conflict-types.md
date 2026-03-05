---
work_package_id: WP10
title: Answer, Gap, Conflict Types
lane: planned
dependencies: []
subtasks: [T047, T048, T049, T050, T051]
---

# WP10: Answer, Gap, Conflict Types

## Objective

Port core interview data structures: Answer, Gap, Conflict, ConflictResolution.

## Context

- **Source**: `/tmp/intent-cli/src/intent/interview.gleam` (lines 50-200)
- **Target**: `clarity-web/src/intent/interview/types.rs`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition |
|----|--------------|
| P1 | Answer question_id is non-empty |
| P2 | Gap id is non-empty |
| P3 | Conflict id is non-empty |
| P4 | Conflict has at least 2 resolution options |

### Postconditions

| ID | Postcondition |
|----|---------------|
| Q1 | Answer round > 0 |
| Q2 | Answer confidence in [0.0, 1.0] |
| Q3 | Gap round matches answer round |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | Blocking gap prevents stage advancement |
| I2 | Chosen option index < options.len() |

---

## Type Definitions

### T047: Answer struct

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    /// Question identifier (required)
    pub question_id: String,
    /// Question text for context
    pub question_text: String,
    /// Perspective of the answer
    pub perspective: Perspective,
    /// Round number (1-indexed)
    pub round: u32,
    /// User's response
    pub response: String,
    /// Extracted fields from response
    #[serde(default)]
    pub extracted: HashMap<String, String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Additional notes
    #[serde(default)]
    pub notes: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

impl Answer {
    pub fn new(question_id: String, response: String, round: u32) -> Result<Self, InterviewError> {
        if question_id.is_empty() {
            return Err(InterviewError::EmptyQuestionId);
        }
        Ok(Self {
            question_id,
            question_text: String::new(),
            perspective: Perspective::Developer,
            round,
            response,
            extracted: HashMap::new(),
            confidence: 1.0,
            notes: String::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}
```

### T048: Gap struct

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gap {
    /// Unique gap identifier
    pub id: String,
    /// Missing field name
    pub field: String,
    /// Why this is needed
    pub description: String,
    /// Whether this blocks progression
    pub blocking: bool,
    /// Suggested default value
    #[serde(default)]
    pub suggested_default: String,
    /// Explanation of why needed
    pub why_needed: String,
    /// Round when detected
    pub round: u32,
    /// Resolution status
    pub resolved: bool,
    /// Resolution value if resolved
    #[serde(default)]
    pub resolution: String,
}
```

### T049-T050: Conflict and ConflictResolution

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    /// Unique conflict identifier
    pub id: String,
    /// Pair of conflicting answer IDs
    pub between: (String, String),
    /// What the conflict is
    pub description: String,
    /// Impact if unresolved
    pub impact: String,
    /// Resolution options
    pub options: Vec<ConflictResolution>,
    /// Chosen option index (if resolved)
    pub chosen: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// Option name
    pub option: String,
    /// What this means
    pub description: String,
    /// Trade-offs of this choice
    pub tradeoffs: String,
    /// Whether recommended
    pub recommendation: String,
}
```

---

## Definition of Done

- [ ] Answer, Gap, Conflict, ConflictResolution structs
- [ ] All derive required traits
- [ ] Builder methods for construction
- [ ] Unit tests for validation
