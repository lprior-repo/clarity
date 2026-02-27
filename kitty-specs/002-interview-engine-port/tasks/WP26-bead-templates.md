---
work_package_id: WP26
title: Bead Templates
lane: planned
dependencies: []
subtasks: []
---

# WP26: Bead Templates

## Objective

16-section CUE bead generation.

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

## Definition of Done

- [ ] Core functionality implemented
- [ ] Contract tests pass
- [ ] No unwrap/expect in production code
- [ ] Documentation complete
