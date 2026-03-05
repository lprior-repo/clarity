# Contract Specification: Add Missing Behavior Fields

## Context
- Feature: Add notes, requires, tags to Behavior struct
- Domain terms: Behavior, dependency, tagging
- Assumptions: Backward compatible with #[serde(default)]
- Open questions: Should 'intent' field be renamed to 'description'?

## Preconditions
- [P1] Behavior struct exists in types/behavior.rs
- [P2] serde derive macros available

## Postconditions
- [Q1] Behavior has notes: String field with #[serde(default)]
- [Q2] Behavior has requires: Vec<String> field with #[serde(default)]
- [Q3] Behavior has tags: Vec<String> field with #[serde(default)]
- [Q4] Parsing JSON without these fields succeeds
- [Q5] Parsing JSON with these fields populates them
- [Q6] Default values are empty string/empty vec

## Invariants
- [I1] All existing behaviors parse without error
- [I2] Field order matches Gleam version

## Error Taxonomy
- No new errors - fields are optional with defaults

## Contract Signatures
```rust
pub struct Behavior {
    pub name: String,
    pub description: String,
    pub notes: String,
    pub requires: Vec<String>,
    pub tags: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub verification: Option<Verification>,
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Behavior exists | Compile-time | struct definition |
| P2: serde available | Compile-time | derive macros |

## Violation Examples (REQUIRED)
- VIOLATES [Q4]: Parse {"name":"foo"} without new fields fails -- WRONG, should succeed
- VIOLATES [Q6]: tags defaults to ["default"] -- WRONG, should default to []

## Ownership Contracts
- Behavior owns all fields
- Clone derived for convenience

## Non-goals
- [ ] Validation of tag format
- [ ] Dependency resolution (separate concern)
