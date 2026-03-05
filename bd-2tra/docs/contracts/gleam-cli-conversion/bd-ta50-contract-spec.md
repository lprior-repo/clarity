# Contract Specification: Add Missing Spec Fields

## Context
- Feature: Add audience, version, success_criteria to Spec struct
- Domain terms: Spec, Feature, Behavior, serde
- Assumptions: Backward compatible with #[serde(default)]
- Open questions: None

## Preconditions
- [P1] Spec struct exists in types/spec.rs
- [P2] serde derive macros available

## Postconditions
- [Q1] Spec has audience: String field with #[serde(default)]
- [Q2] Spec has version: String field with #[serde(default)]
- [Q3] Spec has success_criteria: Vec<String> field with #[serde(default)]
- [Q4] Parsing JSON without these fields succeeds
- [Q5] Parsing JSON with these fields populates them
- [Q6] Default values are empty string/empty vec

## Invariants
- [I1] All existing specs parse without error
- [I2] Field order matches Gleam version

## Error Taxonomy
- No new errors - fields are optional with defaults

## Contract Signatures
```rust
pub struct Spec {
    pub name: String,
    pub description: String,
    pub audience: String,
    pub version: String,
    pub success_criteria: Vec<String>,
    pub features: Vec<Feature>,
    pub invariants: Vec<Invariant>,
    pub anti_patterns: Vec<AntiPattern>,
    pub ai_hints: Option<AIHints>,
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Spec exists | Compile-time | struct definition |
| P2: serde available | Compile-time | derive macros |

## Violation Examples (REQUIRED)
- VIOLATES [Q4]: Parse {} without fields fails -- WRONG, should succeed with defaults
- VIOLATES [Q6]: audience defaults to "unknown" -- WRONG, should default to ""

## Ownership Contracts
- Spec owns all fields
- Clone derived for convenience

## Non-goals
- [ ] Validation of field values (separate concern)
