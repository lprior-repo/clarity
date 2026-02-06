# Implementation Plan: Zero-Unwrap Philosophy Setup

## Phase 2: PLAN

### Task Breakdown

#### Step 1: Create Clippy Configuration
**File**: `.clippy.toml`
**Content**: 
```toml
# Zero-unwrap philosophy enforcement
warn-on-all-warnings = true

[clippy]
# Prevent unwrap and expect
warn-unwrap-used = true
warn-expect-used = true
warn-panic = true
warn-todo = true
warn-unimplemented = true

# Additional strict lints
warn-missing-errors-doc = true
warn-missing-docs = true
warn-needless-return = true
warn-unreadable-literal = true
warn-uninlined-format-args = true
warn-doc-markdown = true
warn-must-use-candidate = true
warn-missing-const-for-fn = true
warn-return-self-not-must-use = true
warn-should-implement-trait = true
warn-new-without-default = true
```

#### Step 2: Add Clippy Attributes to Each Crate
**Files**: 
- `clarity-core/src/lib.rs`
- `clarity-server/src/lib.rs` or `main.rs`
- `clarity-client/src/lib.rs` or `main.rs`

**Content**:
```rust
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
```

#### Step 3: Create Rustfmt Configuration
**File**: `rustfmt.toml`
**Content**:
```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 2
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
```

#### Step 4: Verify Configuration
**Command**: `moon run :quick`
**Expected**: All crates should pass formatting and lint checks

#### Step 5: Write Zero-Unwrap Tests
**Files**: 
- `clarity-core/tests/unwrap_prevention.rs`
- `clarity-server/tests/unwrap_prevention.rs`
- `clarity-client/tests/unwrap_prevention.rs`

**Test Content**:
```rust
#[test]
fn test_unwrap_prevents_unwrap_calls() {
    // This should fail when clippy is configured
    let value = Some(42);
    let _ = value.unwrap(); // Should fail with clippy::unwrap_used
}

#[test]
fn test_unwrap_prevents_expect_calls() {
    // This should fail when clippy is configured
    let value = Some(42);
    let _ = value.expect("This should not compile"); // Should fail with clippy::expect_used
}
```

#### Step 6: Address Existing Clippy Warnings
**Process**:
- Run `moon run :quick` and capture all warnings
- Fix each warning systematically
- Ensure zero warnings before proceeding

### Success Criteria
- ✅ `.clippy.toml` created with strict lints
- ✅ All crates have clippy deny attributes
- ✅ `rustfmt.toml` created with consistent formatting
- ✅ `moon run :quick` passes with zero warnings
- ✅ Zero unwrap/expect calls in codebase
- ✅ All crates use `Result<T, Error>` for error handling
- ✅ Tests verify zero-unwrap philosophy enforcement

### Risk Assessment
**Low Risk**: Configuration files are non-code changes
**Medium Risk**: Existing code may have unwrap/expect calls that need fixing
**Mitigation**: Start with configuration, then fix warnings systematically

### Dependencies
- Moon build system (already configured)
- Rust toolchain (already installed)
- Existing codebase structure (already set up)
