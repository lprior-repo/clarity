---
work_package_id: WP03
title: Invariant and AntiPattern Types
lane: planned
dependencies: []
subtasks: [T010, T011, T012, T013, T014]
---

# WP03: Invariant and AntiPattern Types

## Objective

Port Invariant, AntiPattern, AIHints types from `types.gleam` to extend the core spec type system.

## Context

- **Source**: `/tmp/intent-cli/src/intent/types.gleam` (lines 45-90)
- **Target**: `clarity-web/src/intent/types.rs`
- **Priority**: P0 (Critical)
- **Dependencies**: WP02 (Core Spec Types)

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement Level | Type/Pattern |
|----|--------------|-------------------|--------------|
| P1 | WP02 types exist | Compile-time | `Spec`, `Feature` available |
| P2 | Invariant name is non-empty | Runtime | `!invariant.name.is_empty()` |
| P3 | AntiPattern name is non-empty | Runtime | `!anti_pattern.name.is_empty()` |
| P4 | AIHints entities keys are non-empty | Runtime | `entities.keys().all(|k| !k.is_empty())` |

### Postconditions

| ID | Postcondition | Enforcement Level | Verification |
|----|---------------|-------------------|--------------|
| Q1 | Invariant serializes with criteria array | Runtime | JSON contains `"criteria": [...]` |
| Q2 | AntiPattern examples are valid JSON | Runtime | `serde_json::Value` parses |
| Q3 | AIHints entity map preserves all entries | Runtime | Round-trip preserves keys |
| Q4 | Spec updated to include new fields | Compile-time | `spec.invariants` exists |

### Invariants

| ID | Invariant | Scope |
|----|-----------|-------|
| I1 | Invariant.criteria is never null | Invariant |
| I2 | AntiPattern.bad_example is valid JSON | AntiPattern |
| I3 | AntiPattern.good_example is valid JSON | AntiPattern |
| I4 | AIHints.pitfalls is never null | AIHints |

### Error Taxonomy

```rust
pub enum InvariantError {
    /// Invariant name is empty
    EmptyInvariantName,
    /// AntiPattern name is empty
    EmptyAntiPatternName,
    /// Entity key is empty in AIHints
    EmptyEntityKey,
    /// Bad example JSON is invalid
    InvalidBadExampleJson { reason: String },
    /// Good example JSON is invalid
    InvalidGoodExampleJson { reason: String },
}
```

### Violation Examples (REQUIRED)

```
VIOLATES P2: Invariant name is empty
  -> InvariantError::EmptyInvariantName

VIOLATES P3: AntiPattern name is empty
  -> InvariantError::EmptyAntiPatternName

VIOLATES P4: Entity key is empty string
  -> InvariantError::EmptyEntityKey

VIOLATES I2: Bad example contains invalid JSON
  -> InvariantError::InvalidBadExampleJson { reason: "expected value at line 1" }
```

---

## Subtasks

### T010: Add Invariant struct

```rust
/// Invariant: global constraint that must always hold
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invariant {
    /// Invariant name (required, non-empty)
    pub name: String,
    /// What this invariant ensures
    #[serde(default)]
    pub description: String,
    /// Invariant criteria
    #[serde(default)]
    pub criteria: Vec<String>,
}

impl Invariant {
    pub fn new(name: String) -> Result<Self, InvariantError> {
        if name.is_empty() {
            return Err(InvariantError::EmptyInvariantName);
        }
        Ok(Self { name, description: String::new(), criteria: Vec::new() })
    }
}
```

### T011: Add AntiPattern struct

```rust
/// AntiPattern: pattern to avoid with examples
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AntiPattern {
    /// Anti-pattern name (required, non-empty)
    pub name: String,
    /// Why this pattern is problematic
    #[serde(default)]
    pub description: String,
    /// Example of the anti-pattern
    pub bad_example: serde_json::Value,
    /// Correct alternative
    pub good_example: serde_json::Value,
    /// Explanation of the difference
    #[serde(default)]
    pub why: String,
}

impl AntiPattern {
    pub fn new(name: String, bad: serde_json::Value, good: serde_json::Value)
        -> Result<Self, InvariantError>
    {
        if name.is_empty() {
            return Err(InvariantError::EmptyAntiPatternName);
        }
        Ok(Self {
            name,
            description: String::new(),
            bad_example: bad,
            good_example: good,
            why: String::new(),
        })
    }
}
```

### T012: Add AIHints struct

### T013: Add EntityHint, ImplementationHints, SecurityHints helper structs

### T014: Update Spec struct to include invariants, anti_patterns, ai_hints

---

## Test Strategy

### Contract Verification Tests

```rust
#[test]
fn test_p2_invariant_name_non_empty() {
    let result = Invariant::new("".to_string());
    assert!(matches!(result, Err(InvariantError::EmptyInvariantName)));
}

#[test]
fn test_q1_invariant_serializes_criteria_array() {
    let inv = Invariant::new("test".to_string()).unwrap()
        .with_criteria(vec!["c1".to_string()]);
    let json = serde_json::to_string(&inv).unwrap();
    assert!(json.contains(r#""criteria":["c1"]"#));
}

#[test]
fn test_q2_antipattern_examples_valid_json() {
    let ap = AntiPattern::new(
        "test".to_string(),
        serde_json::json!({"bad": true}),
        serde_json::json!({"good": true}),
    ).unwrap();
    let json = serde_json::to_string(&ap).unwrap();
    let _: AntiPattern = serde_json::from_str(&json).unwrap();
}
```

---

## Definition of Done

- [ ] Invariant, AntiPattern, AIHints structs defined
- [ ] Helper structs (EntityHint, etc.) defined
- [ ] Spec updated with new fields
- [ ] All contract tests pass
