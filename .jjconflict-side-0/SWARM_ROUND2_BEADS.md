# SWARM ROUND 2 BEADS - Remaining Issues from QA Report

**Date**: 2026-02-08
**Agent**: Swarm Agent 11 (Planner)
**Source**: QA Gatekeeper 17 Report
**Status**: READY FOR IMPLEMENTATION

---

## EXECUTIVE SUMMARY

The QA report identified **3 critical issues** blocking the entire pipeline:

1. **P0 - CRITICAL**: Compilation errors in launcher.rs (const fn syntax errors)
2. **P0 - CRITICAL**: 40 remaining clippy warnings (down from 269, but not zero)
3. **P1 - HIGH**: Pipeline synchronization issues (triage vs bead database)

**Total Beads Created**: 3
**Total Estimated Effort**: 5hr 30min

---

## BEAD 1: Fix Launcher Compilation Errors

**Bead ID**: `fix-launcher-syntax`
**Title**: `launcher: fix const fn self parameter syntax errors`
**Type**: bug
**Priority**: 0 (P0 - CRITICAL)
**Effort**: 30min
**Dependencies**: None

### Clarifications

**Resolved Questions**:
- Q: What is the exact syntax error?
  A: The const functions at lines 559 and 566 use `&_self` which is invalid Rust syntax. Const functions cannot have `self` parameters unless explicitly typed.
- Q: Why were these marked const?
  A: These are platform-specific stub implementations marked for future optimization.

**Open Questions**:
- None

**Assumptions**:
- These functions can be non-const (they are stub implementations)
- The #[allow(clippy::unused_self)] attribute is intentional
- These functions should remain as methods (not associated functions)

### EARS Requirements

**Ubiquitous**:
- THE SYSTEM SHALL compile the entire workspace without errors
- THE SYSTEM SHALL maintain valid Rust syntax in all source files
- THE SYSTEM SHALL support platform-specific file association registration

**Event-Driven**:
- WHEN a developer builds the workspace, THE SYSTEM SHALL succeed without compilation errors
- WHEN cargo build is invoked, THE SYSTEM SHALL produce valid artifacts for all targets

**Unwanted**:
- IF const fn syntax is used incorrectly, THE SYSTEM SHALL NOT fail to compile
- IF platform-specific stubs are present, THE SYSTEM SHALL NOT prevent compilation of other platforms
- BECAUSE these are stub implementations, THE SYSTEM SHALL NOT require complex const logic

**State-Driven**:
- WHEN compilation is attempted, THE SYSTEM SHALL be in a valid syntax state
- WHEN the codebase HEAD is built, THE SYSTEM SHALL be free of compilation blockers

### KIRK Contracts

**Preconditions**:
- File `clarity-client/src/launcher.rs` exists
- Lines 559-572 contain the malformed const fn definitions
- Git repository is in clean state (except for target/)

**Postconditions**:
- `cargo build --workspace` succeeds without errors
- All platform-specific registration functions compile correctly
- No const fn syntax errors remain in launcher.rs
- Functions can be called as methods (not associated functions)

**Invariants**:
- All functions maintain their public API surface
- Platform-specific cfg attributes remain intact
- Function signatures remain compatible with existing call sites (lines 314, 346)
- No behavioral changes to file association or protocol handler registration logic

### Research Requirements

**Files to Read**:
- `/home/lewis/src/clarity/clarity-client/src/launcher.rs` (lines 310-360, 558-580)
- `/home/lewis/src/clarity/clarity-client/src/launcher.rs` (full file to understand context)

**Patterns to Find**:
- Other const fn definitions in the codebase for consistency
- Similar platform-specific stub implementations
- Usage patterns of register_file_associations and register_protocol_handlers

**Questions**:
- Why were these marked const? Is there a const context requirement?
- Are there tests that verify these functions can be called as methods?
- What is the expected behavior when these functions are called?

### Inversions

**Security**:
- No security implications (syntax-only fix)

**Usability**:
- Failure mode: Cannot compile project blocks all development
- Impact: HIGH - Prevents all builds and testing

**Data Integrity**:
- No data integrity implications

**Integration Failures**:
- What if changing const to non-const breaks const contexts?
  - Mitigation: Search for const contexts that call these functions
- What if downstream code relies on these being const?
  - Mitigation: Check all call sites (lines 314, 346) - they are in non-const functions

### ATDD Tests

**Happy Paths**:
1. Remove `const` keyword from both functions (lines 560, 568)
2. Run `cargo build --workspace`
3. Verify: Build succeeds with 0 errors
4. Verify: No warnings about unused self (attribute handles this)
5. Verify: Call sites at lines 314 and 346 still work correctly

**Error Paths**:
1. If removing const introduces new errors, investigate const context requirements
2. If call sites fail, verify function signatures match expected pattern

**Edge Cases**:
- Platform-specific compilation: Ensure cfg attributes work correctly
- Cross-compilation: Verify syntax is valid for all target platforms

### E2E Tests

**Pipeline Test**:
```bash
# Test 1: Compilation succeeds
cd /home/lewis/src/clarity
cargo build --workspace
# Expected: exit code 0, no compilation errors

# Test 2: Platform-specific builds
cargo build --workspace --target x86_64-unknown-linux-gnu
# Expected: exit code 0

# Test 3: Check specific file compiles
cargo build -p clarity-client
# Expected: exit code 0
```

**Scenarios**:
1. Developer clones repo and runs cargo build - should succeed
2. CI pipeline runs full build - should pass compilation stage
3. Cross-platform build for Linux target - should succeed

### Verification Checkpoints

**Research Gate**:
- [ ] Read launcher.rs lines 558-580
- [ ] Understand why const was used
- [ ] Verify no const context requirements

**Test Gate**:
- [ ] cargo build --workspace succeeds
- [ ] No new warnings introduced
- [ ] All call sites still work

**Implementation Gate**:
- [ ] const keyword removed from both functions
- [ ] Function signatures remain compatible
- [ ] No other changes to logic

**Integration Gate**:
- [ ] Full workspace builds
- [ ] Platform-specific targets build
- [ ] No regressions in other modules

### Implementation Tasks

**Phase 0: Research (5min)**
- [ ] Read launcher.rs lines 558-580
- [ ] Search for other const fn in file
- [ ] Verify call sites at lines 314, 346
- [ ] Document why const can be safely removed

**Phase 1: Test (5min)**
- [ ] Verify current compilation fails
- [ ] Document exact error messages
- [ ] Identify minimal fix required

**Phase 2: Implementation (10min)**
- [ ] Remove `const` keyword from line 560
- [ ] Remove `const` keyword from line 568
- [ ] Verify function signatures: `fn register_linux_file_associations(&self)`
- [ ] Verify function signatures: `fn register_linux_protocol_handlers(&self)`

**Phase 3: Validation (5min)**
- [ ] Run `cargo build --workspace`
- [ ] Verify 0 compilation errors
- [ ] Verify 0 new warnings
- [ ] Check that call sites still work

**Phase 4: Documentation (5min)**
- [ ] Commit with message: "fix(launcher): remove invalid const from platform stubs"
- [ ] Reference: Resolves compilation errors in lines 559-572

**Parallelization**: None (single atomic fix)

### Failure Modes

**Symptom**: Build still fails after removing const
- **Cause**: Call sites require const context
- **Debug**: `cargo build --workspace 2>&1 | grep -A5 "register_linux"`
- **Fix**: Check if callers are in const contexts

**Symptom**: New warnings about unused_self
- **Cause**: #[allow] attribute not working
- **Debug**: `cargo clippy -p clarity-client 2>&1 | grep unused_self`
- **Fix**: Verify attribute syntax: `#[allow(clippy::unused_self)]`

**Symptom**: Different compilation errors appear
- **Cause**: Unrelated syntax issues in file
- **Debug**: Full cargo build output
- **Fix**: Address each error individually

### Anti-Hallucination

**Read-Before-Write Rules**:
- MUST read launcher.rs before editing
- MUST verify exact line numbers and context
- MUST check git status to understand current state

**API Existence Checks**:
- Verify DesktopLauncher type exists
- Verify LauncherError type exists
- Verify call sites exist and are correct

**Validation Steps**:
1. Read the file first
2. Understand the context
3. Make minimal change (remove const only)
4. Build immediately to verify
5. No speculative changes

### Context Survival

**Progress Files**:
- Edit in place: `/home/lewis/src/clarity/clarity-client/src/launcher.rs`

**Recovery Instructions**:
```bash
# If something goes wrong
git checkout clarity-client/src/launcher.rs
cargo build --workspace
# Should return to known failing state
```

**Checkpoint Markers**:
- After Phase 0: Document findings
- After Phase 2: Build must succeed
- After Phase 4: Commit must be made

### Completion Checklist

**Code**:
- [ ] const keyword removed from line 560
- [ ] const keyword removed from line 568
- [ ] No other changes to function signatures
- [ ] All cfg attributes preserved

**Tests**:
- [ ] cargo build --workspace succeeds (exit 0)
- [ ] cargo build -p clarity-client succeeds (exit 0)
- [ ] No compilation errors in output
- [ ] No new warnings introduced

**CI**:
- [ ] Will pass compilation stage
- [ ] Will pass clippy stage (no new warnings)

**Documentation**:
- [ ] Commit message clear and descriptive
- [ ] References issue and resolution

### Context

**Related Files**:
- `/home/lewis/src/clarity/clarity-client/src/launcher.rs` (lines 310-360, 558-580)
- Call sites: lines 314, 346

**Similar Implementations**:
- Windows registration functions (lines 400-500) - check for similar patterns
- macOS registration functions (lines 500-550) - check for similar patterns

**Patterns**:
- Platform-specific stubs are common in this file
- None of the other platform functions use const

### AI Hints

**Do**:
- Remove ONLY the const keyword
- Keep all other attributes including #[allow(clippy::unused_self)]
- Keep the &self parameter
- Keep the cfg attributes
- Keep the function bodies unchanged
- Test immediately after edit

**Don't**:
- Don't change function signatures
- Don't change the self parameter
- Don't modify the function bodies
- Don't add new attributes
- Don't refactor related code
- Don't "fix" other parts of the file

**Code Patterns**:
```rust
// BEFORE (wrong):
const fn register_linux_file_associations(&self) -> Result<(), LauncherError>

// AFTER (correct):
fn register_linux_file_associations(&self) -> Result<(), LauncherError>
```

**Constitution**:
- Minimal change principle: Only fix the syntax error
- Test-driven: Build immediately to verify
- Document: Explain why const was removed

---

## BEAD 2: Resolve Remaining Clippy Warnings

**Bead ID**: `fix-clippy-warnings`
**Title**: `clippy: resolve remaining 40 warnings to zero-panic compliance`
**Type**: bug
**Priority**: 0 (P0 - CRITICAL)
**Effort**: 4hr
**Dependencies**: None (can run parallel with Bead 1)

### Clarifications

**Resolved Questions**:
- Q: How many warnings remain?
  A: 40 clippy warnings (down from 269, significant progress made)
- Q: What types of warnings?
  A: Need to run clippy to categorize (cast precision loss, format strings, unwrap calls, etc.)
- Q: Can test code have different policies?
  A: Recent commit "docs(lint): standardize test code lint policies" addressed this
- Q: Have we disabled disallowed-methods?
  A: Recent commit "fix(clippy): allow unwrap in tests and disable disallowed-methods" did this

**Open Questions**:
- What are the specific remaining 40 warnings?
- Are they in production code or test code?
- Can they be properly allowed or must they be fixed?

**Assumptions**:
- Remaining warnings are non-trivial (previous attempts to fix them failed)
- Some warnings may require #[allow] attributes with justification
- Test code warnings may need different handling than production code

### EARS Requirements

**Ubiquitous**:
- THE SYSTEM SHALL have zero clippy warnings in production code
- THE SYSTEM SHALL compile with strict clippy lints enabled
- THE SYSTEM SHALL document any allowed lint exceptions with justification

**Event-Driven**:
- WHEN clippy is run, THE SYSTEM SHALL produce 0 warnings
- WHEN new code is added, THE SYSTEM SHALL pass clippy checks
- WHEN CI pipeline runs, THE SYSTEM SHALL complete clippy stage successfully

**Unwanted**:
- IF clippy warnings remain, THE SYSTEM SHALL NOT allow beads to land
- IF warnings are suppressed without justification, THE SYSTEM SHALL NOT accept the suppression
- BECAUSE zero-panic is a quality gate, THE SYSTEM SHALL NOT compromise on lint strictness

**State-Driven**:
- WHEN code is at HEAD, THE SYSTEM SHALL be clippy-clean
- WHEN production code is compiled, THE SYSTEM SHALL use strict lints

### KIRK Contracts

**Preconditions**:
- Git repository at latest commit
- Rust toolchain installed (1.80.0)
- clippy available via rustup

**Postconditions**:
- `cargo clippy --workspace --all-targets` produces 0 warnings
- All allowed lints have documented justification
- Production code has no unwrap/expect without proper error handling
- Test code unwrap calls are explicitly allowed

**Invariants**:
- No behavioral changes to production logic
- Test coverage remains the same
- API surfaces unchanged
- Performance characteristics unchanged

### Research Requirements

**Files to Read**:
- Need to run clippy first to identify affected files
- Recent commits for context: 1ef99d9, 5bd130b, 95a8655
- Any clippy configuration files (.clippy.toml, rustfmt.toml)

**Patterns to Find**:
- What are the 40 remaining warnings?
- Which files have the most warnings?
- Are there clusters of similar warnings?

**Questions**:
- Can these warnings be easily fixed or must they be allowed?
- What is the justification for any allowed warnings?
- Are there false positives from clippy?

### Inversions

**Security**:
- Some clippy warnings relate to security (e.g., unwrap on user input)
- Must ensure no security regressions when fixing warnings

**Usability**:
- Zero warnings improves code quality and maintainability
- Easier for new contributors to understand codebase

**Data Integrity**:
- Clippy warnings often indicate potential data loss or corruption
- Fixing them improves reliability

**Integration Failures**:
- What if fixing one warning introduces another?
  - Mitigation: Fix incrementally, rebuild after each change
- What if clippy suggestions are wrong?
  - Mitigation: Review each suggestion, allow with justification if needed

### ATDD Tests

**Happy Paths**:
1. Run `cargo clippy --workspace --all-targets`
2. Capture all 40 warnings
3. Categorize by type and file
4. Fix or allow each warning with justification
5. Re-run clippy
6. Verify: 0 warnings

**Error Paths**:
1. If fixing a warning breaks tests, revert and investigate
2. If clippy suggestion is incorrect, allow with justification comment
3. If warning is in test code, verify test lint policy applies

**Edge Cases**:
- Warnings in generated code
- Warnings in third-party dependencies
- False positives from clippy

### E2E Tests

**Pipeline Test**:
```bash
# Test 1: Run full clippy check
cargo clippy --workspace --all-targets -- -D warnings
# Expected: exit code 0, no warnings

# Test 2: Run moon CI task
moon run :ci --force
# Expected: All stages pass, including clippy

# Test 3: Count warnings before and after
cargo clippy --workspace --all-targets 2>&1 | grep -c "warning:"
# Expected: 0
```

**Scenarios**:
1. Developer runs clippy locally - should see 0 warnings
2. CI pipeline runs quality checks - should pass
3. New code is added - must not introduce warnings

### Verification Checkpoints

**Research Gate**:
- [ ] Run cargo clippy to capture all warnings
- [ ] Categorize warnings by type
- [ ] Identify files with most warnings
- [ ] Document which can be fixed vs allowed

**Test Gate**:
- [ ] Baseline warning count: 40
- [ ] After fixes: 0 warnings
- [ ] No new compilation errors
- [ ] All tests still pass

**Implementation Gate**:
- [ ] Each warning addressed (fixed or allowed)
- [ ] Justification comments for allowed warnings
- [ ] No behavioral changes
- [ ] Code review ready

**Integration Gate**:
- [ ] Full clippy check passes
- [ ] moon run :ci --force passes
- [ ] No regressions in functionality
- [ ] Documentation updated

### Implementation Tasks

**Phase 0: Research (30min)**
- [ ] Run `cargo clippy --workspace --all-targets 2>&1 | tee /tmp/clippy_warnings.txt`
- [ ] Count and categorize warnings: `grep -c "warning:" /tmp/clippy_warnings.txt`
- [ ] Group by file: `grep "warning:" /tmp/clippy_warnings.txt | cut -d: -f1 | sort | uniq -c`
- [ ] Group by type: `grep "warning:" /tmp/clippy_warnings.txt | awk '{print $NF}' | sort | uniq -c`
- [ ] Document findings in implementation notes

**Phase 1: Categorization (30min)**
- [ ] For each warning, determine: fixable vs allow needed
- [ ] Identify high-priority warnings (security, correctness)
- [ ] Identify low-priority warnings (style, pedantic)
- [ ] Create spreadsheet or tracking document

**Phase 2: Implementation (2.5hr) - CAN PARALLELIZE**
- [ ] Batch 1: Fix all simple warnings (casts, formatting, etc.) - 30min
- [ ] Batch 2: Fix medium warnings (unwrap, expect) - 1hr
- [ ] Batch 3: Allow complex warnings with justification - 1hr

**Phase 3: Validation (30min)**
- [ ] Run `cargo clippy --workspace --all-targets`
- [ ] Verify 0 warnings
- [ ] Run `cargo test --workspace`
- [ ] Verify all tests pass
- [ ] Run `moon run :ci --force`
- [ ] Verify all CI stages pass

**Phase 4: Documentation (30min)**
- [ ] Commit each batch separately
- [ ] Document allowed warnings in code comments
- [ ] Update CONTRIBUTING.md if needed
- [ ] Create summary of changes

**Parallelization**:
- Batch 1, 2, 3 can be done by different agents
- Each batch should be committed separately

### Failure Modes

**Symptom**: Fixing one warning introduces 10 more
- **Cause**: Cascading type inference issues
- **Debug**: Fix warnings one at a time, rebuild after each
- **Fix**: Revert to last known good state, fix incrementally

**Symptom**: Tests start failing after clippy fixes
- **Cause**: Clippy suggestion changed behavior
- **Debug**: Identify which test failed, what changed
- **Fix**: Revert problematic change, allow warning with justification

**Symptom**: Cannot fix warning without breaking API
- **Cause**: Warning indicates fundamental design issue
- **Debug**: Review API design, consider deprecation
- **Fix**: Allow warning for now, create bead to refactor API

**Symptom**: Clippy itself crashes or hangs
- **Cause**: Known clippy bugs with complex macros
- **Debug**: Run with --verbose to see where it hangs
- **Fix**: Allow warning for specific problematic code, file upstream bug

### Anti-Hallucination

**Read-Before-Write Rules**:
- MUST run clippy first to see actual warnings
- MUST read each file before editing
- MUST understand warning before applying fix
- MUST test after each fix

**API Existence Checks**:
- Verify clippy suggestions are valid Rust
- Check if suggested methods exist on types
- Verify suggested imports are available

**Validation Steps**:
1. Run clippy, capture ALL warnings
2. Read each affected file
3. Understand each warning
4. Apply fix or document why allowing
5. Test immediately
6. No batch edits without testing

### Context Survival

**Progress Files**:
- `/tmp/clippy_warnings.txt` - Full clippy output
- `/tmp/clippy_fixes.md` - Tracking document for fixes

**Recovery Instructions**:
```bash
# If something goes wrong
git stash
cargo clippy --workspace --all-targets
# Should return to known 40-warning state
```

**Checkpoint Markers**:
- After Phase 0: Warning count and categorization documented
- After Phase 2: Each batch committed separately
- After Phase 3: 0 warnings achieved

### Completion Checklist

**Code**:
- [ ] All 40 warnings addressed
- [ ] Fixed warnings have proper fixes
- [ ] Allowed warnings have justification comments
- [ ] No behavioral changes to production code

**Tests**:
- [ ] cargo clippy --workspace --all-targets passes (exit 0)
- [ ] cargo test --workspace passes
- [ ] moon run :ci --force passes
- [ ] 0 warnings in clippy output

**CI**:
- [ ] Clippy stage will pass
- [ ] No new CI failures
- [ ] All quality gates pass

**Documentation**:
- [ ] Commits are clear and descriptive
- [ ] Allowed warnings have inline comments
- [ ] Summary document created

### Context

**Related Files**:
- To be determined after running clippy
- Likely candidates based on previous issues:
  - clarity-core/src/db/pool.rs
  - clarity-core/src/types.rs
  - clarity-core/src/validation.rs
  - clarity-client/src/app.rs

**Similar Implementations**:
- Recent fixes in commits 1ef99d9, 5bd130b, 95a8655
- Previous attempts to fix 269 warnings (failed but informative)

**Patterns**:
- Cast precision loss: Use as_*() or try_into() with error handling
- Format strings: Use format!() macros properly
- Unwrap in production: Replace with ? or expect with message
- Unwrap in tests: Already allowed by test lint policy

### AI Hints

**Do**:
- Run clippy first, get actual warnings
- Fix warnings incrementally, test after each
- Commit in batches by type/severity
- Add justification comments for allowed warnings
- Use #[expect(clippy::lint_name)] instead of #[allow] when appropriate
- Consider #[allow] at module level for pervasive warnings

**Don't**:
- Don't fix warnings without seeing them first
- Don't batch-edit multiple files without testing
- Don't suppress warnings without justification
- Don't change behavior while fixing warnings
- Don't refactor code unless necessary

**Code Patterns**:
```rust
// Fixing unwrap:
// BEFORE:
let value = some_option.unwrap();

// AFTER:
let value = some_option.expect("context: why this should never be None");

// OR:
let value = some_option.ok_or_else(|| MyError::MissingValue)?;

// Fixing cast precision loss:
// BEFORE:
let value = some_i64 as i32;

// AFTER:
let value = i32::try_from(some_i64)
    .expect("value should fit in i32");

// OR (if overflow is possible):
let value = i32::try_from(some_i64)
    .map_err(|_| MyError::OutOfRange)?;
```

**Constitution**:
- Incremental fixes: Test after each change
- Document decisions: Comment why warnings are allowed
- Quality first: Zero warnings is the goal

---

## BEAD 3: Fix Pipeline Synchronization

**Bead ID**: `fix-pipeline-sync`
**Title**: `pipeline: synchronize triage output with bead database state`
**Type**: bug
**Priority**: 1 (P1 - HIGH)
**Effort**: 1hr
**Dependencies**: None

### Clarifications

**Resolved Questions**:
- Q: What is the synchronization issue?
  A: Triage showed bd-3p6 as `stage:ready-gatekeeper` but actual status is `IN_PROGRESS`/`stage:building`
- Q: What is the source of truth?
  A: Bead database (verified via `br show`)
- Q: Why does this matter?
  A: Agents may make decisions based on stale triage data

**Open Questions**:
- How does triage fetch bead data?
- Is there a cache that needs invalidation?
- How often should triage refresh?

**Assumptions**:
- Bead database (br commands) is source of truth
- Triage output may be cached or stale
- This is a data flow issue, not a storage issue

### EARS Requirements

**Ubiquitous**:
- THE SYSTEM SHALL maintain single source of truth for bead state
- THE SYSTEM SHALL ensure triage output matches bead database
- THE SYSTEM SHALL provide accurate bead status to all agents

**Event-Driven**:
- WHEN bead state changes, THE SYSTEM SHALL update all cached representations
- WHEN triage is run, THE SYSTEM SHALL fetch fresh data from bead database
- WHEN an agent queries bead status, THE SYSTEM SHALL return current state

**Unwanted**:
- IF triage shows stale data, THE SYSTEM SHALL NOT allow agents to act on it
- IF bead database and triage disagree, THE SYSTEM SHALL NOT proceed without reconciliation
- BECAUSE agents rely on accurate data, THE SYSTEM SHALL NOT serve stale bead state

**State-Driven**:
- WHEN a bead transition occurs, THE SYSTEM SHALL invalidate all caches
- WHEN database is updated, THE SYSTEM SHALL propagate changes immediately

### KIRK Contracts

**Preconditions**:
- Bead database exists and is accessible
- Triage command exists and can query bead database
- Bead tracking system is operational

**Postconditions**:
- `br show <bead-id>` matches `bv --robot-triage` output for stage
- All agents see consistent bead state
- No stale cache issues remain
- Source of truth is documented

**Invariants**:
- Bead database is always source of truth
- Triage is a read-only view of bead database
- No writes to bead state outside of bead database

### Research Requirements

**Files to Read**:
- Triage implementation (likely in bead-visualization or similar tool)
- Bead database schema/format
- Any caching layers in the pipeline

**Patterns to Find**:
- How does triage query bead database?
- Are there any cache files?
- What refresh mechanism exists?

**Questions**:
- Is there a cache timeout?
- Can we force refresh triage data?
- How do other agents get bead state?

### Inversions

**Security**:
- No security implications (data synchronization only)

**Usability**:
- Failure mode: Agents make wrong decisions based on stale data
- Impact: MEDIUM - Causes confusion but doesn't break builds

**Data Integrity**:
- Critical: Must ensure data consistency across system
- Single source of truth prevents conflicts

**Integration Failures**:
- What if triage cannot connect to bead database?
  - Mitigation: Fail fast, don't serve stale data
- What if bead database is locked or unavailable?
  - Mitigation: Queue requests, retry with backoff

### ATDD Tests

**Happy Paths**:
1. Change bead state using br command
2. Immediately run triage
3. Verify: Triage shows updated state
4. Verify: No caching delay

**Error Paths**:
1. If triage shows old state, identify cache issue
2. If bead database is unreachable, triage should fail
3. If state is inconsistent, trigger reconciliation

**Edge Cases**:
- Multiple agents updating beads simultaneously
- Triage run during bead state transition
- Network latency between database and triage

### E2E Tests

**Pipeline Test**:
```bash
# Test 1: Update bead and verify triage
br edit bd-3p6 --stage ready-gatekeeper
bv --robot-triage | grep "bd-3p6.*ready-gatekeeper"
# Expected: triage shows updated state

# Test 2: Compare outputs
br show bd-3p6 | grep "Stage"
# Compare with triage output
# Expected: States match

# Test 3: Force refresh if available
bv --robot-triage --refresh
# Expected: Fresh data from database
```

**Scenarios**:
1. Agent updates bead, another agent queries triage - should see new state
2. CI pipeline checks bead state - should get current data
3. Multiple agents query simultaneously - should get consistent data

### Verification Checkpoints

**Research Gate**:
- [ ] Locate triage implementation
- [ ] Understand data flow from bead database to triage
- [ ] Identify any caching mechanisms
- [ ] Document how to force refresh

**Test Gate**:
- [ ] Test current synchronization issue
- [ ] Document the delay/staleness
- [ ] Identify root cause

**Implementation Gate**:
- [ ] Fix identified root cause
- [ ] Add cache invalidation if needed
- [ ] Add force refresh option if needed

**Integration Gate**:
- [ ] Verify triage matches bead database
- [ ] Test multiple rapid state changes
- [ ] Document expected behavior

### Implementation Tasks

**Phase 0: Research (15min)**
- [ ] Find triage implementation: `which bv` or `which bead-visualize`
- [ ] Read source if available
- [ ] Check for cache files: `find /tmp /var -name "*bead*" -o -name "*triage*"`
- [ ] Check environment variables for cache/config
- [ ] Document findings

**Phase 1: Reproduction (10min)**
- [ ] Query current state of bd-3p6: `br show bd-3p6`
- [ ] Run triage: `bv --robot-triage | grep bd-3p6`
- [ ] Compare outputs, document discrepancy
- [ ] Test if issue is consistent or intermittent

**Phase 2: Root Cause (15min)**
- [ ] If cache file found, check modification time
- [ ] If caching in code, find cache duration
- [ ] Check if there's a refresh option
- [ ] Identify minimal fix

**Phase 3: Implementation (15min)**
- [ ] Implement fix (could be various approaches):
  - Option A: Disable caching entirely
  - Option B: Reduce cache timeout
  - Option C: Add --refresh flag
  - Option D: Invalidate cache on bead update
- [ ] Test fix with reproduction steps
- [ ] Verify triage now matches database

**Phase 4: Documentation (5min)**
- [ ] Document source of truth (bead database)
- [ ] Document how to get fresh triage data
- [ ] Update any relevant agent instructions
- [ ] Commit changes

**Parallelization**: None (investigation task)

### Failure Modes

**Symptom**: Cannot find triage implementation
- **Cause**: Binary-only distribution, closed source
- **Debug**: Check if it's a compiled binary or script
- **Fix**: Document workaround, use br commands as source of truth

**Symptom**: No cache files found
- **Cause**: Caching is in-memory or in database
- **Debug**: Check process memory, database tables
- **Fix**: May need to restart triage process, or accept limitation

**Symptom**: Fix requires changes to external tool
- **Cause**: Issue is in bead-visualization tool
- **Debug**: Verify if tool is open source
- **Fix**: File issue upstream, document workaround

**Symptom**: Issue is intermittent
- **Cause**: Race condition, network timing
- **Debug**: Run multiple rapid tests
- **Fix**: Add retry logic, document timing expectations

### Anti-Hallucination

**Read-Before-Write Rules**:
- MUST investigate actual triage tool before proposing fixes
- MUST verify if source is available
- MUST test any assumptions about caching

**API Existence Checks**:
- Verify bv/bead-visualize commands exist
- Check if br commands work reliably
- Verify bead database is accessible

**Validation Steps**:
1. Investigate first
2. Reproduce the issue
3. Identify root cause
4. Propose minimal fix
5. Test fix
6. No changes without investigation

### Context Survival

**Progress Files**:
- `/tmp/triage_investigation.md` - Investigation notes

**Recovery Instructions**:
```bash
# If something goes wrong
# No state changes to revert, just documentation
git checkout README.md  # if we updated docs
```

**Checkpoint Markers**:
- After Phase 0: Investigation findings documented
- After Phase 1: Issue reproduced and confirmed
- After Phase 2: Root cause identified
- After Phase 3: Fix implemented and tested

### Completion Checklist

**Code**:
- [ ] Triage synchronization fix implemented
- [ ] Cache invalidation working
- [ ] Or: Documentation of workaround

**Tests**:
- [ ] Triage output matches bead database
- [ ] Multiple rapid updates handled correctly
- [ ] No stale data served

**CI**:
- [ ] Agents can rely on triage data
- [ ] No confusion about bead state

**Documentation**:
- [ ] Source of truth documented
- [ ] Workaround documented if fix not possible
- [ ] Agent instructions updated if needed

### Context

**Related Files**:
- To be determined: bead-visualization or bv tool
- Bead database location (likely ~/.local/share/bead or similar)

**Similar Implementations**:
- Other pipeline tools that query bead database
- Consistency patterns in distributed systems

**Patterns**:
- Source of truth pattern
- Cache invalidation strategies
- Read-through vs write-through caching

### AI Hints

**Do**:
- Investigate actual tool before proposing fixes
- Verify source code availability
- Test minimal reproducible case
- Document workarounds if fix not possible
- Prioritize: Document source of truth

**Don't**:
- Don't assume caching without evidence
- Don't propose code changes without seeing source
- Don't spend more than 1hr on investigation
- Don't create complex workarounds

**Code Patterns**:
```
# If source is available:
# Look for cache configuration
# Look for database query code
# Add cache invalidation or reduce TTL

# If source is NOT available:
# Document bead database as source of truth
# Document how to query directly: br show <bead>
# Document expected delay/staleness
# Recommend agents use br commands for accuracy
```

**Constitution**:
- Source of truth: Bead database
- Investigation first, fixes second
- Document workarounds if needed
- 1hr timebox on investigation

---

## IMPLEMENTATION DEPENDENCIES

```
Bead 1 (fix-launcher-syntax) [30min]
    ↓
Bead 2 (fix-clippy-warnings) [4hr] - CAN RUN IN PARALLEL
    ↓
Bead 3 (fix-pipeline-sync) [1hr] - CAN RUN IN PARALLEL
```

**Critical Path**:
1. Bead 1 MUST complete first (blocks all compilation)
2. Bead 2 can run in parallel after Bead 1 (but also blocks pipeline)
3. Bead 3 is independent (blocks agents, not builds)

**Recommended Order**:
1. **First**: Bead 1 (fix-launcher-syntax) - 30min
2. **Second**: Bead 2 (fix-clippy-warnings) - 4hr (can parallelize within)
3. **Third**: Bead 3 (fix-pipeline-sync) - 1hr (or anytime)

**Total Time**: ~5.5hr (can be reduced with parallel agents)

---

## SUCCESS METRICS

### Overall Swarm Success
- [ ] All 3 beads created and validated
- [ ] Bead 1: Compilation succeeds
- [ ] Bead 2: 0 clippy warnings
- [ ] Bead 3: Triage matches database

### Pipeline Unblocked
- [ ] cargo build --workspace succeeds (0 errors)
- [ ] cargo clippy --workspace --all-targets passes (0 warnings)
- [ ] moon run :ci --force passes all stages
- [ ] Beads can progress to ready-gatekeeper stage
- [ ] Gatekeeper agents can verify and land beads

### Quality Improvements
- [ ] Zero compilation errors (down from 3+)
- [ ] Zero clippy warnings (down from 40)
- [ ] Improved data consistency (triage sync)

---

## VERIFICATION COMMANDS

After all beads are complete, run:

```bash
# Verify compilation
cargo build --workspace 2>&1 | tee /tmp/build_output.txt
# Expected: exit code 0, 0 errors

# Verify clippy
cargo clippy --workspace --all-targets 2>&1 | tee /tmp/clippy_output.txt
# Expected: exit code 0, 0 warnings

# Verify CI pipeline
moon run :ci --force 2>&1 | tee /tmp/ci_output.txt
# Expected: All stages pass

# Verify triage sync
br show bd-3p6 | grep "Stage"
bv --robot-triage | grep "bd-3p6"
# Expected: States match

# Verify bead database
br list --filter "stage:ready-gatekeeper"
# Expected: Beads can be promoted to this stage
```

---

## NEXT STEPS

1. **Implement Bead 1** (fix-launcher-syntax) - CRITICAL, blocks all builds
2. **Implement Bead 2** (fix-clippy-warnings) - CRITICAL, blocks all landings
3. **Implement Bead 3** (fix-pipeline-sync) - HIGH, improves agent coordination

After these beads are complete, the pipeline should be unblocked and normal swarm operations can resume.

---

**Report Generated**: 2026-02-08
**Agent**: Swarm Agent 11 (Planner)
**Mission**: Create atomic beads for remaining QA issues
**Status**: COMPLETE - 3 beads created, ready for implementation
