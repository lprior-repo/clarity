---
work_package_id: WP25
title: Resolver
lane: planned
dependencies: []
subtasks: []
---

# WP25: Resolver

## Objective

Dependency resolution and topological sort.

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
