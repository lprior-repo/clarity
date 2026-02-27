---
work_package_id: WP32
title: Spec Templates
lane: planned
dependencies: []
subtasks: []
---

# WP32: Spec Templates

## Objective

Profile-specific spec templates.

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

### WP32 Violation Examples:

| Contract | Violation Call | Expected Error |
|----------|----------------|----------------|
| P1: Input validated | `generate_spec_template(Profile::Api, Session { id: "".to_string(), ... })` | `Err(TemplateError::EmptySessionId)` |
| P1: Input validated | `fill_template(template_with_missing_fields, session)` | `Err(TemplateError::MissingRequiredField)` |
| I1: Deterministic ops | Same session produces different templates | Test failure |
| Q1: Result<T, E> | `generate_spec_template(invalid_profile, session)` | `Err(TemplateError::InvalidProfile)` |
| Q2: No panics | Code contains `.unwrap()` on template rendering | Fails code review |

### Test Parity:
- `test_empty_session_id_returns_error` covers P1 violation #1
- `test_missing_required_field_returns_error` covers P1 violation #2
- `test_template_generation_is_deterministic` covers I1
- `test_no_unwrap_in_templates_module` covers Q2

## Definition of Done

- [ ] Core functionality implemented
- [ ] Contract tests pass
- [ ] No unwrap/expect in production code
- [ ] Documentation complete
