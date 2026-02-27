---
work_package_id: "WP01"
title: "Project Foundation"
lane: "planned"
dependencies: []
subtasks: ["T001", "T002", "T003", "T004"]
---

# WP01: Project Foundation

## Objective

Set up the `intent` module structure and dependencies in clarity-web. This is the foundational work package that all others depend on.

## Context

- **Source**: New module creation (no Gleam port for structure)
- **Target**: `clarity-web/src/intent/` directory structure
- **Priority**: P0 (Critical) - Must complete before any other WP

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement Level | Type/Pattern |
|----|--------------|-------------------|--------------|
| P1 | `clarity-web` crate exists | Compile-time | Module path check |
| P2 | `Cargo.toml` is valid TOML | Compile-time | `cargo check` |
| P3 | No existing `intent` module | Runtime | `!Path::exists("src/intent")` |

### Postconditions

| ID | Postcondition | Enforcement Level | Verification |
|----|---------------|-------------------|--------------|
| Q1 | All 10 submodule directories exist | Runtime | `for dir in SUBMODULES: assert!(dir.exists())` |
| Q2 | Each submodule has `mod.rs` | Runtime | `for dir in SUBMODULES: assert!(dir.join("mod.rs").exists())` |
| Q3 | `cargo check` passes with empty modules | Compile-time | Build succeeds |
| Q4 | Dependencies compile without version conflicts | Compile-time | `cargo build` succeeds |
| Q5 | CUE schemas are byte-for-byte identical to source | Runtime | `diff -q source/ target/` |

### Invariants

| ID | Invariant | Scope |
|----|-----------|-------|
| I1 | All module paths use `snake_case` | Global |
| I2 | `mod.rs` re-exports public API | Per-module |
| I3 | Dependencies use exact versions (no `^`) | Cargo.toml |

### Error Taxonomy

```rust
pub enum FoundationError {
    /// Cargo.toml parse failed
    InvalidCargoToml { path: PathBuf, reason: String },
    /// Module already exists
    ModuleExists { path: PathBuf },
    /// Dependency version conflict
    DependencyConflict { dep: String, existing: String, requested: String },
    /// CUE schema copy failed
    SchemaCopyFailed { source: PathBuf, target: PathBuf, io_error: std::io::Error },
    /// Directory creation failed
    DirectoryCreationFailed { path: PathBuf, io_error: std::io::Error },
}
```

### Violation Examples (REQUIRED)

```
VIOLATES P3: Intent module already exists at src/intent/
  -> FoundationError::ModuleExists { path: "src/intent" }

VIOLATES Q1: Submodule directory missing after creation
  -> FoundationError::DirectoryCreationFailed { path: "src/intent/interview", io_error: ... }

VIOLATES Q4: Dependency version conflict (serde 1.0 vs 2.0)
  -> FoundationError::DependencyConflict { dep: "serde", existing: "2.0", requested: "1.0" }

VIOLATES Q5: CUE schema corrupted during copy
  -> FoundationError::SchemaCopyFailed { source: "/tmp/intent-cli/schema/intent.cue", ... }
```

---

## Subtasks

### T001: Create `clarity-web/src/intent/` directory structure with mod.rs

**Purpose**: Establish the module hierarchy for all intent functionality.

**Implementation Steps**:
1. Create base directory: `src/intent/`
2. Create submodules: `interview/`, `plan/`, `beads/`, `quality/`, `validation/`, `batch/`, `documents/`, `templates/`, `cli/`, `util/`
3. Create `mod.rs` in each with module doc comment
4. Create root `src/intent/mod.rs` re-exporting submodules

**Files to Create**:
- `clarity-web/src/intent/mod.rs` (~50 lines)
- `clarity-web/src/intent/interview/mod.rs` (~10 lines)
- `clarity-web/src/intent/plan/mod.rs` (~10 lines)
- `clarity-web/src/intent/beads/mod.rs` (~10 lines)
- `clarity-web/src/intent/quality/mod.rs` (~10 lines)
- `clarity-web/src/intent/validation/mod.rs` (~10 lines)
- `clarity-web/src/intent/batch/mod.rs` (~10 lines)
- `clarity-web/src/intent/documents/mod.rs` (~10 lines)
- `clarity-web/src/intent/templates/mod.rs` (~10 lines)
- `clarity-web/src/intent/cli/mod.rs` (~10 lines)
- `clarity-web/src/intent/util/mod.rs` (~10 lines)

**Validation**:
- [ ] `cargo check --package clarity-web` passes
- [ ] All module paths resolve
- [ ] No circular module dependencies

---

### T002: Add serde, serde_json, chrono, regex, thiserror, anyhow, uuid dependencies

**Purpose**: Add required dependencies with exact versions.

**Implementation Steps**:
1. Add to `[dependencies]` in `clarity-web/Cargo.toml`
2. Use exact versions (no `^` prefix)
3. Enable required features

**Dependency Specifications**:
```toml
[dependencies]
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"
chrono = { version = "0.4.34", features = ["serde"] }
regex = "1.10.3"
thiserror = "1.0.57"
anyhow = "1.0.80"
uuid = { version = "1.7.0", features = ["v4", "serde"] }

[dev-dependencies]
proptest = "1.4.0"
tempfile = "3.10.1"
```

**Validation**:
- [ ] `cargo build --package clarity-web` succeeds
- [ ] No duplicate dependency versions
- [ ] Features are correctly enabled

---

### T003: Create `clarity-web/tests/intent/` test directory

**Purpose**: Establish parallel test structure for intent module.

**Implementation Steps**:
1. Create `tests/intent/` directory
2. Create placeholder test files mirroring source structure
3. Add integration test helper module

**Files to Create**:
- `clarity-web/tests/intent/mod.rs`
- `clarity-web/tests/intent/types_test.rs`
- `clarity-web/tests/intent/interview/` directory

**Validation**:
- [ ] Test directory structure mirrors source
- [ ] `cargo test --package clarity-web` runs (even if tests are empty)

---

### T004: Copy CUE schemas from `/tmp/intent-cli/schema/` to `clarity-web/schemas/`

**Purpose**: Preserve CUE schema files for validation.

**Implementation Steps**:
1. Create `clarity-web/schemas/` directory
2. Copy all `.cue` files from source
3. Verify byte-for-byte copy

**Files to Copy**:
- `intent.cue` (113 lines)
- `interview.cue` (252 lines)
- `kirk.cue` (237 lines)
- `questions.cue` (480 lines)
- `enhanced-bead.cue` (570 lines)
- `ai_protocol.cue` (143 lines)
- `ai_interview.cue` (165 lines)
- `custom-questions.cue` (178 lines)

**Validation**:
- [ ] All 8 CUE files exist in target
- [ ] File contents match source (use `diff -q`)
- [ ] `cue vet schema/*.cue` passes (if cue CLI available)

---

## Test Strategy

### Contract Verification Tests

```rust
#[test]
fn test_p3_module_not_exists_before_creation() {
    assert!(!Path::new("src/intent").exists());
}

#[test]
fn test_q1_all_submodules_exist() {
    let submodules = ["interview", "plan", "beads", "quality",
                      "validation", "batch", "documents", "templates",
                      "cli", "util"];
    for submodule in submodules {
        assert!(Path::new(format!("src/intent/{}/mod.rs", submodule)).exists());
    }
}

#[test]
fn test_q5_schemas_byte_identical() {
    for schema in &SCHEMA_FILES {
        let source = PathBuf::from("/tmp/intent-cli/schema").join(schema);
        let target = PathBuf::from("schemas").join(schema);
        assert_eq!(fs::read(source).unwrap(), fs::read(target).unwrap());
    }
}
```

### Contract Violation Tests

```rust
#[test]
fn test_p3_violation_module_exists_error() {
    // Given: Module already exists
    fs::create_dir_all("src/intent").unwrap();

    // When: Attempting to create again
    let result = create_intent_module();

    // Then: Returns ModuleExists error
    assert!(matches!(result, Err(FoundationError::ModuleExists { .. })));
}
```

---

## Definition of Done

- [ ] All 10 submodule directories exist with `mod.rs`
- [ ] Root `intent/mod.rs` re-exports submodules
- [ ] All dependencies added with exact versions
- [ ] `cargo check --package clarity-web` passes
- [ ] Test directory structure created
- [ ] All 8 CUE schemas copied byte-for-byte
- [ ] All contract tests pass

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Dependency version conflict | High | Check existing deps first, use exact versions |
| CUE CLI not available | Low | Skip CUE validation if not installed |
| Circular module deps | Medium | Use `mod name;` not inline modules |

## Reviewer Guidance

1. Verify directory structure matches plan exactly
2. Check dependency versions don't conflict with existing
3. Confirm CUE schemas are verbatim copies
4. Ensure `cargo check` passes cleanly
