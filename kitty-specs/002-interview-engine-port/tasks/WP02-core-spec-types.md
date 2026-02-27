---
work_package_id: "WP02"
title: "Core Spec Types"
lane: "planned"
dependencies: ["WP01"]
subtasks: ["T005", "T006", "T007", "T008", "T009"]
---

# WP02: Core Spec Types

## Objective

Port Spec, Feature, Behavior, Verification types from `types.gleam` (90 lines) to Rust with full Design by Contract specification.

## Context

- **Source**: `/tmp/intent-cli/src/intent/types.gleam` (90 lines)
- **Target**: `clarity-web/src/intent/types.rs`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement Level | Type/Pattern |
|----|--------------|-------------------|--------------|
| P1 | WP01 completed (module structure exists) | Runtime | `Path::exists("src/intent/mod.rs")` |
| P2 | serde dependency available | Compile-time | `use serde::{Serialize, Deserialize}` |
| P3 | Spec name is non-empty when constructing | Runtime | `!name.is_empty()` |
| P4 | Feature name is non-empty when constructing | Runtime | `!feature.name.is_empty()` |

### Postconditions

| ID | Postcondition | Enforcement Level | Verification |
|----|---------------|-------------------|--------------|
| Q1 | Spec serializes to valid JSON | Runtime | `serde_json::to_string(&spec).is_ok()` |
| Q2 | Spec deserializes from valid JSON | Runtime | `serde_json::from_str::<Spec>(json).is_ok()` |
| Q3 | Round-trip serialization preserves data | Runtime | `assert_eq!(spec, deserialize(serialize(spec)))` |
| Q4 | Behavior name is valid identifier format | Runtime | `VALID_IDENT_REGEX.is_match(&behavior.name)` |
| Q5 | All types derive required traits | Compile-time | `impl Debug + Clone + PartialEq + Serialize + Deserialize` |

### Invariants

| ID | Invariant | Scope |
|----|-----------|-------|
| I1 | Spec.features is never null (empty vec ok) | Spec |
| I2 | Feature.behaviors is never null | Feature |
| I3 | Behavior.verifications is never null | Behavior |
| I4 | All string fields are UTF-8 | All types |

### Error Taxonomy

```rust
pub enum TypeError {
    /// Spec name is empty
    EmptySpecName,
    /// Feature name is empty
    EmptyFeatureName { feature_index: usize },
    /// Behavior name is invalid identifier
    InvalidBehaviorName { feature_index: usize, behavior_index: usize, name: String },
    /// JSON serialization failed
    SerializationFailed { reason: String },
    /// JSON deserialization failed
    DeserializationFailed { reason: String, source: String },
    /// Round-trip failed
    RoundTripFailed { original: Spec, recovered: Spec },
}
```

### Violation Examples (REQUIRED)

```
VIOLATES P3: Spec name is empty string
  -> TypeError::EmptySpecName

VIOLATES P4: Feature name is empty string
  -> TypeError::EmptyFeatureName { feature_index: 2 }

VIOLATES Q4: Behavior name contains spaces "my behavior"
  -> TypeError::InvalidBehaviorName { feature_index: 0, behavior_index: 3, name: "my behavior" }

VIOLATES Q3: Round-trip loses data (features dropped)
  -> TypeError::RoundTripFailed { original: Spec { features: [...] }, recovered: Spec { features: [] } }
```

---

## Subtasks

### T005: Create `clarity-web/src/intent/types.rs` with Spec struct

**Purpose**: Define the root Spec type that contains all specification data.

**Contract for Spec**:

```rust
/// Root specification type
///
/// # Invariants
/// - `name` is non-empty
/// - `features` is never null (use empty vec)
///
/// # Example
/// ```
/// let spec = Spec::new("My API".to_string())?;
/// assert!(!spec.name.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    /// Spec name (required, non-empty)
    pub name: String,
    /// Detailed description
    #[serde(default)]
    pub description: String,
    /// Target audience
    #[serde(default)]
    pub audience: String,
    /// Spec version
    #[serde(default = "default_version")]
    pub version: String,
    /// Success criteria
    #[serde(default)]
    pub success_criteria: Vec<String>,
    /// Feature definitions
    #[serde(default)]
    pub features: Vec<Feature>,
}

impl Spec {
    /// Create new spec with name
    ///
    /// # Preconditions
    /// - `name` is non-empty
    ///
    /// # Errors
    /// Returns `TypeError::EmptySpecName` if name is empty
    pub fn new(name: String) -> Result<Self, TypeError> {
        if name.is_empty() {
            return Err(TypeError::EmptySpecName);
        }
        Ok(Self {
            name,
            description: String::new(),
            audience: String::new(),
            version: default_version(),
            success_criteria: Vec::new(),
            features: Vec::new(),
        })
    }
}
```

**Validation**:
- [ ] Spec::new rejects empty name
- [ ] Spec serializes to JSON correctly
- [ ] Spec deserializes from JSON correctly

---

### T006: Add Feature struct with behaviors field

**Purpose**: Define Feature as a logical grouping of related behaviors.

**Contract for Feature**:

```rust
/// Feature: logical grouping of related behaviors
///
/// # Invariants
/// - `name` is non-empty
/// - `behaviors` is never null
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    /// Feature name (required, non-empty)
    pub name: String,
    /// Feature description
    #[serde(default)]
    pub description: String,
    /// Behavior definitions
    #[serde(default)]
    pub behaviors: Vec<Behavior>,
}

impl Feature {
    /// Create new feature
    ///
    /// # Preconditions
    /// - `name` is non-empty
    ///
    /// # Errors
    /// Returns `TypeError::EmptyFeatureName` if name is empty
    pub fn new(name: String) -> Result<Self, TypeError> {
        if name.is_empty() {
            return Err(TypeError::EmptyFeatureName { feature_index: 0 });
        }
        Ok(Self {
            name,
            description: String::new(),
            behaviors: Vec::new(),
        })
    }
}
```

**Validation**:
- [ ] Feature::new rejects empty name
- [ ] Feature with empty behaviors serializes correctly

---

### T007: Add Behavior struct with verifications, requires, tags fields

**Purpose**: Define Behavior as a single testable behavior.

**Contract for Behavior**:

```rust
/// Behavior: single testable behavior
///
/// # Invariants
/// - `name` is valid identifier (snake_case, no spaces)
/// - `verifications` is never null
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Behavior {
    /// Behavior name (identifier format: snake_case)
    pub name: String,
    /// What the behavior accomplishes
    #[serde(default)]
    pub intent: String,
    /// Additional notes
    #[serde(default)]
    pub notes: String,
    /// Dependencies on other behaviors
    #[serde(default)]
    pub requires: Vec<String>,
    /// Categorization tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Preconditions
    #[serde(default)]
    pub preconditions: Vec<String>,
    /// Postconditions
    #[serde(default)]
    pub postconditions: Vec<String>,
    /// Verification methods
    #[serde(default)]
    pub verifications: Vec<Verification>,
}

impl Behavior {
    /// Create new behavior
    ///
    /// # Preconditions
    /// - `name` matches `^[a-z][a-z0-9_]*$`
    ///
    /// # Errors
    /// Returns `TypeError::InvalidBehaviorName` if name is invalid
    pub fn new(name: String) -> Result<Self, TypeError> {
        lazy_static! {
            static ref VALID_NAME: Regex = Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();
        }
        if !VALID_NAME.is_match(&name) {
            return Err(TypeError::InvalidBehaviorName {
                feature_index: 0,
                behavior_index: 0,
                name,
            });
        }
        Ok(Self {
            name,
            intent: String::new(),
            notes: String::new(),
            requires: Vec::new(),
            tags: Vec::new(),
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            verifications: Vec::new(),
        })
    }
}
```

**Validation**:
- [ ] Behavior::new rejects names with spaces
- [ ] Behavior::new rejects names starting with number
- [ ] Behavior::new accepts valid snake_case names

---

### T008: Add Verification struct with criteria and examples

**Purpose**: Define how to verify a behavior works correctly.

**Contract for Verification**:

```rust
/// Verification: how to verify a behavior works
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Verification {
    /// Verification description
    #[serde(default)]
    pub description: String,
    /// Success criteria
    #[serde(default)]
    pub criteria: Vec<String>,
    /// Example data
    #[serde(default)]
    pub examples: Vec<serde_json::Value>,
}

impl Verification {
    /// Create new verification
    pub fn new(description: String) -> Self {
        Self {
            description,
            criteria: Vec::new(),
            examples: Vec::new(),
        }
    }
}
```

**Validation**:
- [ ] Verification serializes examples as JSON array
- [ ] Verification accepts any JSON value as example

---

### T009: Derive Debug, Clone, PartialEq, Serialize, Deserialize for all types

**Purpose**: Ensure all types have consistent trait implementations.

**Implementation Steps**:
1. Verify all structs have `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
2. Add `#[serde(default)]` to optional fields
3. Add `#[serde(skip_serializing_if = "Vec::is_empty")]` for cleaner JSON

**Validation**:
- [ ] All types can be cloned
- [ ] All types can be compared for equality
- [ ] All types serialize to JSON without errors

---

## Test Strategy

### Contract Verification Tests

```rust
#[test]
fn test_p3_spec_name_non_empty() {
    let result = Spec::new("".to_string());
    assert!(matches!(result, Err(TypeError::EmptySpecName)));
}

#[test]
fn test_p4_feature_name_non_empty() {
    let result = Feature::new("".to_string());
    assert!(matches!(result, Err(TypeError::EmptyFeatureName { .. })));
}

#[test]
fn test_q4_behavior_name_valid_identifier() {
    let valid_names = vec!["get_user", "create_order", "a", "test_123"];
    for name in valid_names {
        assert!(Behavior::new(name.to_string()).is_ok());
    }

    let invalid_names = vec!["my behavior", "123test", "Test", "test-name"];
    for name in invalid_names {
        assert!(Behavior::new(name.to_string()).is_err());
    }
}

#[test]
fn test_q3_round_trip_preserves_data() {
    let spec = Spec::new("Test API".to_string()).unwrap()
        .with_feature(Feature::new("auth".to_string()).unwrap()
            .with_behavior(Behavior::new("login".to_string()).unwrap()));

    let json = serde_json::to_string(&spec).unwrap();
    let recovered: Spec = serde_json::from_str(&json).unwrap();

    assert_eq!(spec, recovered);
}
```

### Contract Violation Tests

```rust
#[test]
fn test_violation_p3_empty_spec_name() {
    let result = Spec::new("".to_string());
    assert!(matches!(result, Err(TypeError::EmptySpecName)));
}

#[test]
fn test_violation_q4_invalid_behavior_name_spaces() {
    let result = Behavior::new("my behavior".to_string());
    assert!(matches!(result, Err(TypeError::InvalidBehaviorName { name, .. }) if name == "my behavior"));
}
```

---

## Definition of Done

- [ ] Spec, Feature, Behavior, Verification structs defined
- [ ] All preconditions enforced with Result returns
- [ ] All types derive required traits
- [ ] Round-trip serialization tests pass
- [ ] Contract verification tests pass
- [ ] Contract violation tests pass

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Serde derive macro issues | Low | Already in dependencies |
| JSON null vs empty vec | Medium | Use `#[serde(default)]` |
