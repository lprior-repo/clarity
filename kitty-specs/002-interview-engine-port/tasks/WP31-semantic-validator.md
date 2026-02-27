---
work_package_id: WP31
title: Semantic Validator
lane: planned
dependencies: []
subtasks: []
---

# WP31: Semantic Validator

## Objective

Cross-reference and consistency validation.

## Context

- **Source**: `/tmp/intent-cli/src/intent/` (Gleam source)
- **Target**: `clarity-web/src/intent/`
- **Priority**: P0/P1 (see plan.md for tier details)

## Contract Specification

### Preconditions

| ID | Precondition |
|----|--------------|
| P1 | Input validated |

### Postconditions

| ID | Postcondition |
|----|---------------|
| Q1 | Result<T, E> for all fallible ops |
| Q2 | No panics |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | All operations are deterministic |

---

## Violation Examples (Rust Contract Requirement)

### WP31 Violation Examples:

| Contract | Violation Call | Expected Error |
|----------|----------------|----------------|
| P1: Input validated | `validate_semantics(Spec { name: "".to_string(), ... })` | `Err(ValidationError::EmptySpecName)` |
| P1: Input validated | `cross_reference_validation(spec_with_broken_refs)` | `Err(ValidationError::BrokenReference)` |
| I1: Deterministic ops | Same spec produces different validation results | Test failure |
| Q1: Result<T, E> | `consistency_checks(invalid_spec_structure)` | `Err(ValidationError::InvalidStructure)` |
| Q2: No panics | Code contains `.unwrap()` on cross-reference lookup | Fails code review |

### Test Parity:
- `test_empty_spec_name_returns_validation_error` covers P1 violation #1
- `test_broken_reference_returns_error` covers P1 violation #2
- `test_validation_is_deterministic` covers I1
- `test_no_unwrap_in_semantic_validator` covers Q2

## Definition of Done

- [ ] Core functionality implemented
- [ ] Contract tests pass
- [ ] No unwrap/expect in production code
- [ ] Documentation complete
