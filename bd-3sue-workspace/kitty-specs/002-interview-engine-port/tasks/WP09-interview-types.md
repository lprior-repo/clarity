---
work_package_id: WP09
title: Interview Types
lane: planned
dependencies: []
subtasks: [T041, T042, T043, T044, T045, T046]
---

# WP09: Interview Types

## Objective

Port `question_types.gleam` (42 lines) - foundational enums for the interview system.

## Context

- **Source**: `/tmp/intent-cli/src/intent/question_types.gleam` (42 lines)
- **Target**: `clarity-web/src/intent/interview/types.rs`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition |
|----|--------------|
| P1 | All enum values are valid UTF-8 |
| P2 | Serde is available |

### Postconditions

| ID | Postcondition |
|----|---------------|
| Q1 | All enums serialize to lowercase JSON |
| Q2 | All enums deserialize from lowercase JSON |
| Q3 | Unknown variants return DeserializationError |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | Profile has exactly 6 variants |
| I2 | InterviewStage has exactly 5 variants |
| I3 | Perspective has exactly 5 variants |

---

## Type Definitions

### T041-T046: Core Interview Enums

```rust
use serde::{Deserialize, Serialize};

/// System profile type - determines which questions to ask
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// REST/GraphQL API
    Api,
    /// Command-line interface
    Cli,
    /// Event-driven system
    Event,
    /// Data processing system
    Data,
    /// Workflow automation
    Workflow,
    /// User interface
    Ui,
}

impl Profile {
    /// Get required fields for this profile
    pub fn required_fields(&self) -> &'static [&'static str] {
        match self {
            Profile::Api => &["base_url", "auth_method", "happy_path", "error_cases", "response_format"],
            Profile::Cli => &["command_name", "happy_path", "help_text", "exit_codes"],
            Profile::Event => &["event_type", "payload_schema", "trigger"],
            Profile::Data => &["data_model", "access_patterns", "retention"],
            Profile::Workflow => &["steps", "happy_path", "error_recovery"],
            Profile::Ui => &["user_flows", "happy_path", "states"],
        }
    }
}

/// Interview session lifecycle stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterviewStage {
    /// Initial information gathering
    Discovery,
    /// Detail refinement
    Refinement,
    /// Validation of collected info
    Validation,
    /// Interview complete
    Complete,
    /// Paused for later resumption
    Paused,
}

/// Answer perspective - who is answering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Perspective {
    User,
    Developer,
    Ops,
    Security,
    Business,
}

/// Question priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionPriority {
    Critical,
    Important,
    NiceToHave,
}

/// Question category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionCategory {
    HappyPath,
    ErrorCase,
    EdgeCase,
    Constraint,
    Dependency,
    NonFunctional,
}
```

---

## Test Strategy

```rust
#[test]
fn test_profile_serialization() {
    assert_eq!(serde_json::to_string(&Profile::Api).unwrap(), r#""api""#);
    assert_eq!(serde_json::from_str::<Profile>(r#""cli""#).unwrap(), Profile::Cli);
}

#[test]
fn test_stage_serialization() {
    assert_eq!(serde_json::to_string(&InterviewStage::Discovery).unwrap(), r#""discovery""#);
}

#[test]
fn test_profile_required_fields() {
    assert!(Profile::Api.required_fields().contains(&"auth_method"));
    assert!(Profile::Cli.required_fields().contains(&"exit_codes"));
}
```

---

## Definition of Done

- [ ] Profile enum with 6 variants
- [ ] InterviewStage enum with 5 variants
- [ ] Perspective enum with 5 variants
- [ ] QuestionPriority enum with 3 variants
- [ ] QuestionCategory enum with 6 variants
- [ ] All enums serialize to lowercase
