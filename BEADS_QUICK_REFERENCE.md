# Code Quality Beads - Quick Reference

## Priority 0 - CRITICAL (Must Complete First)

### bd-1aa: Fix test compilation errors
```bash
# Start here
br show bd-1aa          # Read full specification
br ready | grep bd-1aa  # Verify it's ready
# Implementation:
# 1. Remove duplicate import in allocator_test.rs:18
# 2. Add Deserialize derive to health.rs HealthResponse
# 3. Verify: cargo test --workspace
```

**Impact**: Unblocks entire test suite and CI/CD  
**Time**: 30 minutes

---

## Priority 1 - High (Can Run in Parallel)

### bd-1e6: Fix cast precision loss
```bash
br show bd-1e6
# 8 instances in pool.rs, sqlite_pool.rs, quality.rs
# Add #[allow(clippy::cast_precision_loss)] with comments
```

### bd-28d: Remove unnecessary clones
```bash
br show bd-28d
# 15+ instances
# Pattern: .clone().iter() → .iter()
```

### bd-17s: Add Eq derives
```bash
br show bd-17s
# Add Eq to PartialEq types
# Pattern: #[derive(PartialEq, Eq)]
```

### bd-btk: Fix format strings
```bash
br show bd-btk
# 8+ instances
# Pattern: format!("{}", x) → format!("{x}")
```

**Total Time**: ~3 hours (can parallelize)

---

## Verification Commands

```bash
# After each bead
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --check

# Quality gate (all beads complete)
cargo clean
cargo build --workspace --release
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Progress Tracking

| Bead | Status | Verified |
|------|--------|----------|
| bd-1aa | ⏳ Todo | [ ] |
| bd-1e6 | ⏳ Todo | [ ] |
| bd-28d | ⏳ Todo | [ ] |
| bd-17s | ⏳ Todo | [ ] |
| bd-btk | ⏳ Todo | [ ] |

---

## Workflow

1. **Start**: `bd-1aa` (blocks everything)
2. **Parallel**: `bd-1e6`, `bd-28d`, `bd-17s`, `bd-btk`
3. **Verify**: Run quality gate after each
4. **Commit**: One commit per bead
5. **Report**: Update this file with checkboxes

---

## Dependencies

```
bd-1aa (P0)
  ↓
Everything else unblocked
  ↓
bd-1e6 ─┐
bd-28d ─┼─ Can run in parallel
bd-17s ─┤
bd-btk ─┘
```

---

**Documentation**: See `CONTINUOUS_IMPROVEMENT_BEADS.md` for full details
**Report**: See `AGENT11_EXECUTION_REPORT.md` for planning process

---

## SWARM ROUND 2 BEADS (2026-02-08)

**New beads from QA Gatekeeper 17 Report analysis**

### Bead 1: Fix Launcher Syntax (P0 - CRITICAL)
```bash
# File: clarity-client/src/launcher.rs
# Lines 560, 568: Remove "const" keyword

# BEFORE:
const fn register_linux_file_associations(&self) -> Result<(), LauncherError>

# AFTER:
fn register_linux_file_associations(&self) -> Result<(), LauncherError>
```
**Time**: 30min | **Impact**: Unblocks all builds

### Bead 2: Fix Clippy Warnings (P0 - CRITICAL)
```bash
# 40 remaining warnings (down from 269)
# Run clippy and categorize:
cargo clippy --workspace --all-targets 2>&1 | tee /tmp/warnings.txt

# Fix in batches:
# Batch 1: Simple (casts, formatting) - 30min
# Batch 2: Medium (unwrap, expect) - 1hr
# Batch 3: Complex (allow with justification) - 1hr
```
**Time**: 4hr | **Impact**: Unblocks all landings

### Bead 3: Fix Pipeline Sync (P1 - HIGH)
```bash
# Investigate triage vs bead database mismatch
br show bd-3p6 | grep "Stage"
bv --robot-triage | grep "bd-3p6"

# Fix caching or document workaround
```
**Time**: 1hr | **Impact**: Improves coordination

**Execution**: Bead 1 first, then Beads 2+3 in parallel
**Total Time**: ~4.5hr with parallel agents
**Full Specs**: See `SWARM_ROUND2_BEADS.md`
**Report**: See `SWARM_AGENT11_REPORT.md`
