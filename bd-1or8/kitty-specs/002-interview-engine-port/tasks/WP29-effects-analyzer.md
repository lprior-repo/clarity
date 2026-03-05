---
work_package_id: WP29
title: Effects Analyzer
lane: planned
dependencies: []
subtasks: []
---

# WP29: Effects Analyzer

## Objective

Second-order effect detection.

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

### WP29 Violation Examples:

| Contract | Violation Call | Expected Error |
|----------|----------------|----------------|
| P1: Input validated | `analyze_effects(EffectsInput { effect_type: "".to_string(), ... })` | `Err(EffectsError::EmptyEffectType)` |
| P1: Input validated | `analyze_effects(EffectsInput { effect_type: "invalid_type".to_string(), ... })` | `Err(EffectsError::InvalidEffectType)` |
| P1: Input validated | `analyze_effects(EffectsInput { behavior: None, ... })` | `Err(EffectsError::MissingBehavior)` |
| Q1: Result<T, E> | Any function that could fail returns Result, never panics | Compile-time check |
| Q2: No panics | Code contains `.unwrap()` or `.expect()` | Fails code review |
| I1: Deterministic | Same input produces different output on repeated calls | Test failure |

### Test Parity:
- `test_empty_effect_type_returns_error` covers P1 violation
- `test_invalid_effect_type_returns_error` covers P1 violation
- `test_missing_behavior_returns_error` covers P1 violation
- `test_no_unwrap_in_effects_module` covers Q2
- `test_deterministic_output` covers I1

---

## Definition of Done

- [ ] Core functionality implemented
- [ ] Contract tests pass
- [ ] No unwrap/expect in production code
- [ ] Documentation complete
