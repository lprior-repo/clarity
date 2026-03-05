---
work_package_id: WP15
title: Interview Storage
lane: planned
dependencies: []
subtasks: []
---

# WP15: Interview Storage

## Objective

JSONL persistence with streaming I/O.

## Context

- **Source**: `/tmp/intent-cli/src/intent/` (Gleam source)
- **Target**: `clarity-web/src/intent/interview/`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition |
|----|--------------|
| P1 | Session ID is non-empty |
| P2 | All inputs validated |

### Postconditions

| ID | Postcondition |
|----|---------------|
| Q1 | No panic paths |
| Q2 | Result<T, E> for all fallible ops |
| Q3 | Idempotent operations |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | State machine transitions are valid |
| I2 | Collections never null (empty ok) |

### Error Taxonomy

```rust
pub enum InterviewError {
    EmptySessionId,
    InvalidStageTransition { from: InterviewStage, to: InterviewStage, reason: String },
    BlockingGaps(Vec<Gap>),
    UnresolvedConflicts(Vec<Conflict>),
    SerializationError(String),
    IoError(String),
}
```

---

## Definition of Done

- [ ] Core types implemented
- [ ] Contract tests pass
- [ ] No unwrap/expect in production code
- [ ] Documentation complete
