---
work_package_id: WP21
title: Validation Rules
lane: planned
dependencies: []
subtasks: []
---

# WP21: Validation Rules

## Objective

Rule engine for spec validation.

## Context

- **Source**: `/tmp/intent-cli/src/intent/` (Gleam source)
- **Target**: `clarity-web/src/intent/`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition |
|----|--------------|
| P1 | Input validated |
| P2 | Dependencies satisfied |

### Postconditions

| ID | Postcondition |
|----|---------------|
| Q1 | Result<T, E> for all fallible ops |
| Q2 | No panics |

### Error Taxonomy

```rust
pub enum ParseError {
    InvalidJson { reason: String, position: usize },
    MissingField { field: String, context: String },
    InvalidFieldType { field: String, expected: String, actual: String },
}
```

---

## Definition of Done

- [ ] Core functionality implemented
- [ ] Contract tests pass
- [ ] No unwrap/expect
