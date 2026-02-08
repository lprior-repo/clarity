# Performance Benchmarking Beads for Clarity

**Session**: perf-benchmarks
**Created**: 2026-02-08
**Focus**: Database operations, serialization, and validation performance

---

## Bead 1: SQLite Connection Pool Benchmarks

**ID**: `perf-db-pool`
**Title**: `db: Add SQLite connection pool performance benchmarks`
**Type**: feature
**Priority**: 1
**Effort**: 2hr

### 0. Clarifications

**Resolved Questions**:
- Focus on `sqlite_pool.rs` operations (pool creation, connection acquisition, metrics)
- Benchmark both warm and cold pool states
- Include concurrent access patterns (10-50 concurrent tasks)
- Use Criterion.rs for statistical analysis

**Open Questions**:
- Should we benchmark connection retry logic? (Phase 2 research task)
- What are production pool sizes? (Assume 5-10 based on config defaults)

**Assumptions**:
- Criterion.rs will be added to dev-dependencies
- Benchmarks run in CI with `--bench` flag
- Baseline metrics stored in `benches/baseline/`
- Use in-memory SQLite (`sqlite::memory:`) for isolation

### 1. EARS Requirements

**Ubiquitous Requirements**:
- THE SYSTEM SHALL provide automated benchmarks for all pool operations
- THE SYSTEM SHALL measure throughput (ops/sec), latency (ns), and memory allocation
- THE SYSTEM SHALL support comparison against baseline metrics
- THE SYSTEM SHALL generate HTML reports with visual charts

**Event-Driven Requirements**:
- WHEN benchmark suite is executed, THE SYSTEM SHALL run all database pool benchmarks and generate Criterion reports
- WHEN pool benchmarks detect regression >10%, THE SYSTEM SHALL fail the CI pipeline with performance regression notice

**Unwanted Behaviors**:
- IF benchmark code modifies production database paths, THE SYSTEM SHALL NOT commit benchmark-only code to production paths, BECAUSE benchmarks should live in `benches/` directory
- IF benchmarks take longer than 5 minutes to run, THE SYSTEM SHALL NOT be included in standard CI suite, BECAUSE CI speed is critical for developer productivity

### 2. KIRK Contracts

**Preconditions**:
- Criterion.rs is added to dev-dependencies in `Cargo.toml`
- `benches/` directory exists in `clarity-core/`
- `sqlite_pool.rs` module is accessible from benchmark code
- Workspace has `[workspace.dependencies.criterion]` defined

**Postconditions**:
- `benches/db_pool_bench.rs` exists with 4+ benchmark functions
- `cargo bench --bench db_pool` runs successfully
- Criterion HTML report generates with baseline comparisons
- Benchmark results documented in this file

**Invariants**:
- Benchmarks MUST use in-memory SQLite databases
- Benchmarks MUST clean up resources after each iteration
- Benchmark names MUST follow pattern: `bench_<operation>_<scenario>`
- All benchmarks MUST compile with `#[forbid(unsafe_code)]`

### 2.5 Research Requirements

**Files to Read**:
- `/home/lewis/src/clarity/clarity-core/src/db/sqlite_pool.rs` - Pool implementation
- `/home/lewis/src/clarity/clarity-core/src/db/mod.rs` - Module exports
- `/home/lewis/src/clarity/Cargo.toml` - Workspace dependencies

**Patterns to Find**:
- Existing test patterns in `sqlite_pool.rs` tests module (lines 336-537)
- Pool configuration options and their defaults (lines 46-59)
- WAL mode PRAGMA settings (lines 164-189)

**Research Questions**:
- What are the typical pool sizes used in production? (Check SqliteDbConfig defaults)
- Are there existing benchmarks to follow patterns from? (Search for `benches/` directory)
- Should we benchmark connection retry logic? (Evaluate `acquire_sqlite_with_retry`)

### 3. Inversions

**Security Inversions**:
- Benchmarks MUST NOT use production database paths
- In-memory databases MUST be isolated per benchmark iteration

**Usability Inversions**:
- Benchmark output MUST be human-readable (Criterion HTML reports)
- Long-running benchmarks (>5 min) SHOULD be optional in CI

**Data Integrity Inversions**:
- Benchmark results MUST be reproducible across runs
- Random data generation MUST use seeded RNG

**Integration Failures**:
- If Criterion.rs conflicts with workspace dependencies, MUST use compatible version
- If async benchmarks don't compile, MUST research `criterion::async` support

### 4. ATDD Tests

**Happy Paths**:
- `cargo bench --bench db_pool` completes without errors
- Criterion generates HTML report with throughput/latency metrics
- Baseline comparison shows +/- 5% variance is acceptable
- Benchmarks measure: pool creation, connection acquisition, concurrent access

**Error Paths**:
- Benchmark fails if `sqlite_pool.rs` public API changes
- Criterion reports compilation error if `benches/` not in `Cargo.toml`
- Benchmark panics if in-memory database path is invalid

**Edge Cases**:
- Pool with `max_connections=1` (single connection bottleneck)
- Pool with `max_connections=100` (stress test)
- Concurrent acquisition from 50 tokio tasks
- Pool acquisition timeout scenarios
- Pool metrics under 100% utilization

**Contract Tests**:
- `SqliteDbConfig::in_memory()` always returns valid config
- `create_sqlite_pool()` succeeds with in-memory URL
- `get_sqlite_pool_metrics()` returns valid utilization (0-100%)

### 5. E2E Tests

**Pipeline Test**:
```bash
# Full benchmark suite execution
cargo bench --bench db_pool -- --save-baseline main

# Verify report generation
test -f target/criterion/db_pool/report/index.html

# Compare against baseline
cargo bench --bench db_pool -- --baseline main
```

**Scenarios**:
1. **Developer Workflow**: Run benchmarks locally before commit
2. **CI Workflow**: Run benchmarks on PR, compare against `main` branch
3. **Regression Detection**: CI fails if throughput degrades >10%

### 5.5 Verification Checkpoints

**Research Gate**:
- [ ] Read `sqlite_pool.rs` implementation
- [ ] Research Criterion.rs async benchmark patterns
- [ ] Check for existing `benches/` directory structure

**Test Gate**:
- [ ] Benchmarks compile without errors
- [ ] All benchmarks complete in <5 minutes
- [ ] Criterion HTML report generates successfully

**Implementation Gate**:
- [ ] At least 4 benchmark functions implemented
- [ ] Benchmarks cover: creation, acquisition, concurrency, metrics
- [ ] All benchmarks use in-memory databases

**Integration Gate**:
- [ ] `Cargo.toml` updated with Criterion dependency
- [ ] CI configuration includes benchmark job
- [ ] Baseline metrics documented in `docs/performance/`

### 6. Implementation Tasks

**Phase 0: Research (15min)**
- [ ] Read `clarity-core/src/db/sqlite_pool.rs` to understand public API
- [ ] Research Criterion.rs benchmark patterns for async code (tokio runtime)
- [ ] Check if `benches/` directory exists in workspace structure
- [ ] Review existing test patterns in `sqlite_pool.rs` tests module

**Phase 1: Setup (30min)**
- [ ] Create `clarity-core/benches/` directory if not present
- [ ] Add `criterion = { version = "0.5", features = ["html_reports"] }` to workspace dev-dependencies
- [ ] Create `benches/db_pool_bench.rs` with basic benchmark skeleton
- [ ] Configure Criterion for CI output (non-interactive mode)

**Phase 2: Core Benchmarks (PARALLEL - 4 tasks)**
- [ ] **Task A**: Implement `bench_pool_creation()` - measure pool initialization time (cold/warm)
- [ ] **Task B**: Implement `bench_connection_acquire()` - measure single connection acquisition latency
- [ ] **Task C**: Implement `bench_concurrent_acquire()` - measure 10/50 concurrent acquisitions
- [ ] **Task D**: Implement `bench_pool_metrics()` - measure metrics collection overhead

**Phase 3: Validation (30min)**
- [ ] Run `cargo bench --bench db_pool` to generate baseline
- [ ] Verify Criterion HTML report generates in `target/criterion/report/`
- [ ] Test benchmarks with different pool sizes (1, 5, 10, 50 connections)
- [ ] Profile memory allocation using Criterion's memory profiling

**Phase 4: Integration (15min)**
- [ ] Document baseline metrics in `docs/performance/db_pool_baseline.md`
- [ ] Add CI configuration to run benchmarks on merge to main
- [ ] Create performance regression threshold (10% degradation)
- [ ] Update `Cargo.toml` `[[bench]]` section if needed

### 7. Failure Modes

**Symptoms** | **Causes** | **Debugging Commands**
--- | --- | ---
Benchmark fails to compile | Criterion.rs version conflicts | `cargo tree -i criterion`
Benchmarks timeout (>5min) | Pool size too large or infinite loops | `cargo bench --bench db_pool -- --profile-time=10`
Benchmark results inconsistent | Random data or system noise | `RUSTFLAGS='--cfg=bench' cargo bench --bench db_pool`
Memory leak detected | Pool not closed after benchmark | `valgrind --tool=massif cargo bench`
Async runtime panic | Tokio runtime not configured | Check `#[tokio::test]` vs runtime setup

### 7.5 Anti-Hallucination

**Read-Before-Write Rules**:
- MUST read `sqlite_pool.rs` before writing benchmarks
- MUST check if `benches/` exists before creating directory
- MUST verify Criterion.rs workspace dependency before adding

**API Existence Checks**:
- `create_sqlite_pool()` - check function signature
- `SqliteDbConfig::in_memory()` - verify method exists
- `get_sqlite_pool_metrics()` - confirm return type

**Don't Invent APIs**:
- Don't assume `criterion::async` exists (it doesn't, use `tokio::runtime`)
- Don't create benchmark helpers without checking Criterion docs

### 7.6 Context Survival

**Progress Files**:
- `benches/db_pool_bench.rs` - Main benchmark implementation
- `docs/performance/db_pool_baseline.md` - Baseline metrics
- `target/criterion/db_pool/` - Criterion reports (gitignored)

**Recovery Instructions**:
```bash
# If benchmarks fail to compile
cargo clean
cargo bench --bench db_pool -- --nocapture

# If baseline is lost
cargo bench --bench db_pool -- --save-baseline main

# To compare changes
cargo bench --bench db_pool -- --baseline main
```

### 8. Completion Checklist

- [ ] All benchmarks compile without errors
- [ ] `cargo bench --bench db_pool` runs successfully
- [ ] Criterion HTML report generates with visualizations
- [ ] Baseline metrics documented in `docs/performance/`
- [ ] At least 4 benchmark functions implemented
- [ ] Benchmarks cover pool lifecycle operations
- [ ] CI configuration includes benchmark job
- [ ] Performance regression threshold configured
- [ ] Documentation updated in `PERFORMANCE_BEADS.md`

### 9. Context

**Related Files**:
- `/home/lewis/src/clarity/clarity-core/src/db/sqlite_pool.rs` (lines 156-197: pool creation)
- `/home/lewis/src/clarity/clarity-core/src/db/pool.rs` (pool traits)
- `/home/lewis/src/clarity/Cargo.toml` (workspace dependencies)

**Similar Implementations**:
- Test patterns in `sqlite_pool.rs` tests module (lines 336-537)
- `test_sqlite_pool_in_memory()` shows in-memory DB usage
- `test_wal_mode_enabled()` shows PRAGMA verification

### 10. AI Hints

**Do**:
- Use `criterion::BenchmarkId` to parameterize benchmarks by pool size
- Use `criterion::Throughput::Elements` for ops/sec reporting
- Use `black_box()` to prevent compiler optimization of test data
- Create helper functions to generate test configurations
- Benchmark warm-up iterations to stabilize JIT

**Don't**:
- Don't use production database paths
- Don't benchmark I/O operations (use in-memory)
- Don't forget to close pools after benchmarks
- Don't use `unwrap()` in benchmarks (handle errors properly)

**Code Patterns**:
```rust
// Correct async benchmark pattern
fn bench_async_operation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("bench_pool_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let config = SqliteDbConfig::in_memory();
            create_sqlite_pool(&config).await.unwrap()
        })
    });
}
```

**Constitution**:
- Zero-unwrap philosophy: Benchmarks can use `unwrap()` for test setup only
- All benchmark code must follow workspace lint rules
- Use `#[forbid(unsafe_code)]` in benchmark code

---

## Bead 2: JSON Serialization Benchmarks

**ID**: `perf-json-serialize`
**Title**: `formatter: Add JSON serialization/deserialization benchmarks`
**Type**: feature
**Priority**: 1
**Effort**: 2hr

### 0. Clarifications

**Resolved Questions**:
- Focus on `formatter.rs` `JsonFormatter` implementation (manual JSON building)
- Benchmark both compact and pretty-print modes
- Test with varying interview sizes: small (10), medium (100), large (1000 questions)
- Measure formatting throughput and memory allocation

**Open Questions**:
- Should we benchmark `serde_json::to_string` as comparison baseline? (Phase 2)

**Assumptions**:
- Manual JSON serialization in `formatter.rs` (lines 176-250) is the target
- `serde_json` is used for field escaping in current implementation
- `InterviewBuilder` can create test data of varying sizes
- Current implementation uses `write!` macro for string building

### 1. EARS Requirements

**Ubiquitous Requirements**:
- THE SYSTEM SHALL benchmark JSON formatting for all interview sizes
- THE SYSTEM SHALL measure serialization time and memory allocation
- THE SYSTEM SHALL compare compact vs pretty-print performance
- THE SYSTEM SHALL provide baseline metrics for regression detection

**Event-Driven Requirements**:
- WHEN serialization benchmark runs, THE SYSTEM SHALL test with 10, 100, and 1000 question interviews
- WHEN formatting regression exceeds 15%, THE SYSTEM SHALL fail CI with performance notice

**Unwanted Behaviors**:
- IF benchmark uses hardcoded static test data, THE SYSTEM SHALL NOT have unreliable measurements, BECAUSE static data gets optimized away by compiler
- IF benchmark includes I/O operations, THE SYSTEM SHALL NOT measure disk/network time, BECAUSE only serialization CPU time should be measured

### 2. KIRK Contracts

**Preconditions**:
- Criterion.rs is added as dev-dependency
- `formatter.rs` `JsonFormatter` is accessible and public
- `InterviewBuilder` can create test interviews
- Benchmark harness can create test interviews of various sizes

**Postconditions**:
- `benches/json_formatter_bench.rs` exists with size-variant benchmarks
- Benchmarks measure small (10), medium (100), large (1000) interviews
- Both compact and pretty-print modes are benchmarked
- Baseline metrics documented in this file

**Invariants**:
- Benchmarks MUST use `black_box()` to prevent compiler optimization
- Test data MUST be recreated for each iteration (no static data)
- Benchmark names MUST indicate size: `bench_json_<size>_<mode>`

### 2.5 Research Requirements

**Files to Read**:
- `/home/lewis/src/clarity/clarity-core/src/formatter.rs` (lines 135-260: JsonFormatter)
- `/home/lewis/src/clarity/clarity-core/src/interview.rs` (Interview structure)
- `/home/lewis/src/clarity/clarity-core/src/types/question.rs` (Question types)

**Patterns to Find**:
- `InterviewBuilder` usage in formatter tests (lines 405-431)
- `JsonFormatter::format()` implementation (lines 176-250)
- Manual JSON string building with `write!` macro pattern

**Research Questions**:
- Should we benchmark `serde_json::to_string` as comparison? (Phase 2 decision)
- What is the maximum realistic interview size? (Assume 1000 for now)
- Should we benchmark Markdown and PlainText formatters too? (Not in this bead)

### 3. Inversions

**Security Inversions**:
- Test data MUST NOT contain sensitive information
- Special character escaping MUST be tested (quotes, backslashes, Unicode)

**Usability Inversions**:
- Benchmark output MUST clearly label size variants
- Large benchmark (1000 questions) SHOULD complete in reasonable time

**Data Integrity Inversions**:
- Test data generation MUST be deterministic
- Random content MUST use seeded RNG or fixed patterns

**Integration Failures**:
- If `InterviewBuilder` API changes, benchmarks MUST adapt
- If `JsonFormatter` becomes private, benchmarks fail

### 4. ATDD Tests

**Happy Paths**:
- `cargo bench --bench json_formatter` runs all size variants
- Criterion report shows throughput (ints/sec) for each size
- Pretty-print is slower than compact (expected behavior)
- Large interview (1000 questions) completes in <1 second

**Error Paths**:
- Benchmark fails if `InterviewBuilder` API changes
- Compilation error if `JsonFormatter` is not pub
- Benchmark times out if interview size is too large

**Edge Cases**:
- Empty interview (0 questions)
- Interview with 10,000 questions (stress test)
- Interview with Unicode/special characters in all fields
- Interview with deeply nested answer structures

**Contract Tests**:
- `JsonFormatter::new()` creates valid formatter
- `JsonFormatter::format()` returns `Result<String, FormatError>`
- Empty interview produces valid JSON `{"questions":[]}`

### 5. E2E Tests

**Pipeline Test**:
```bash
# Full benchmark suite
cargo bench --bench json_formatter -- --save-baseline main

# Verify report
test -f target/criterion/json_formatter/report/index.html

# Compare sizes
cargo bench --bench json_formatter -- --baseline main
```

**Scenarios**:
1. **Size Comparison**: 10 vs 100 vs 1000 questions
2. **Mode Comparison**: Compact vs Pretty-print for each size
3. **Regression Detection**: 15% threshold for CI failures

### 5.5 Verification Checkpoints

**Research Gate**:
- [ ] Read `formatter.rs` `JsonFormatter` implementation
- [ ] Study `InterviewBuilder` API for test data creation
- [ ] Review existing formatter tests for data patterns

**Test Gate**:
- [ ] Benchmarks compile without errors
- [ ] All benchmarks complete in <2 minutes
- [ ] `black_box()` prevents optimization (verify with miri)

**Implementation Gate**:
- [ ] At least 6 benchmark functions (3 sizes × 2 modes)
- [ ] Size variants: 10, 100, 1000 questions
- [ ] Both compact and pretty-print modes covered

**Integration Gate**:
- [ ] Baseline metrics documented
- [ ] CI configuration includes JSON formatter benchmarks
- [ ] Performance regression threshold set to 15%

### 6. Implementation Tasks

**Phase 0: Research (15min)**
- [ ] Read `formatter.rs` `JsonFormatter` implementation
- [ ] Study `InterviewBuilder` API for creating test data
- [ ] Review existing formatter tests for data patterns (lines 405-431)
- [ ] Check if `serde_json::to_string` should be comparison baseline

**Phase 1: Setup (30min)**
- [ ] Create `benches/json_formatter_bench.rs`
- [ ] Add helper functions to create test interviews of various sizes
- [ ] Set up Criterion benchmark groups for size variants
- [ ] Configure Criterion for parameterized benchmarks (by size)

**Phase 2: Core Benchmarks (PARALLEL - 6 tasks)**
- [ ] **Task A**: `bench_json_small_compact()` - 10 questions, compact mode
- [ ] **Task B**: `bench_json_small_pretty()` - 10 questions, pretty mode
- [ ] **Task C**: `bench_json_medium_compact()` - 100 questions, compact mode
- [ ] **Task D**: `bench_json_medium_pretty()` - 100 questions, pretty mode
- [ ] **Task E**: `bench_json_large_compact()` - 1000 questions, compact mode
- [ ] **Task F**: `bench_json_large_pretty()` - 1000 questions, pretty mode

**Phase 3: Validation (30min)**
- [ ] Run `cargo bench --bench json_formatter` to establish baselines
- [ ] Verify Criterion generates comparison charts
- [ ] Test that `black_box()` prevents optimization
- [ ] Measure memory allocation with Criterion's memory profiling

**Phase 4: Integration (15min)**
- [ ] Document baseline times in `docs/performance/json_baseline.md`
- [ ] Add performance regression check to CI (15% threshold)
- [ ] Create benchmark data generation script for reproducibility
- [ ] Update `PERFORMANCE_BEADS.md` with findings

### 7. Failure Modes

**Symptoms** | **Causes** | **Debugging Commands**
--- | --- | ---
Benchmark optimizes away | Static test data, no `black_box` | `cargo +nightly miri run` to check optimization
Inconsistent results | JIT warmup issues | Add warm-up iterations in Criterion config
Compilation error | `InterviewBuilder` API changed | Check interview.rs for new API
Pretty-print too slow | JSON parse/re-parse overhead | Profile with `flamegraph`
Memory leak | Formatter retains references | Use `valgrind` to check

### 7.5 Anti-Hallucination

**Read-Before-Write Rules**:
- MUST read `formatter.rs` before writing benchmarks
- MUST check `InterviewBuilder` API before creating test data
- MUST verify `JsonFormatter` is public

**API Existence Checks**:
- `JsonFormatter::new()` - check constructor
- `JsonFormatter::format()` - check method signature
- `InterviewBuilder::new()` - verify builder pattern

**Don't Invent APIs**:
- Don't assume `Question` has specific fields without checking
- Don't create mock data without following `Question` struct

### 7.6 Context Survival

**Progress Files**:
- `benches/json_formatter_bench.rs` - Benchmark implementation
- `docs/performance/json_baseline.md` - Baseline metrics
- `target/criterion/json_formatter/` - Criterion reports

**Recovery Instructions**:
```bash
# Regenerate baseline
cargo bench --bench json_formatter -- --save-baseline main

# Compare against baseline
cargo bench --bench json_formatter -- --baseline main

# Profile memory
cargo bench --bench json_formatter -- --profile-time=10
```

### 8. Completion Checklist

- [ ] All 6 benchmarks compile and run
- [ ] Size variants (10, 100, 1000) implemented
- [ ] Both compact and pretty-print modes covered
- [ ] `black_box()` prevents optimization
- [ ] Baseline metrics documented
- [ ] CI configuration includes regression check
- [ ] Large benchmark completes in <1 second
- [ ] Documentation updated

### 9. Context

**Related Files**:
- `/home/lewis/src/clarity/clarity-core/src/formatter.rs` (lines 135-260)
- `/home/lewis/src/clarity/clarity-core/src/interview.rs`
- `/home/lewis/src/clarity/clarity-core/src/types/question.rs`

**Similar Implementations**:
- `test_format_large_interview_completes_in_reasonable_time()` (lines 647-683)
- `create_test_interview()` helper (lines 405-431)

### 10. AI Hints

**Do**:
- Use `criterion::BenchmarkId::new("size", size)` for parameterization
- Use `criterion::Throughput::Elements(q.len())` for questions/sec
- Generate test questions dynamically in loop
- Use `black_box(interview)` before formatting
- Compare against `serde_json::to_string` if feasible

**Don't**:
- Don't use static test data (gets optimized away)
- Don't forget to test special characters (quotes, Unicode)
- Don't benchmark I/O (only CPU time)

**Code Patterns**:
```rust
fn bench_json_compact(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_formatter");
    for size in [10, 100, 1000].iter() {
        let interview = create_test_interview(*size);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let interview = black_box(create_test_interview(size));
                JsonFormatter::compact().format(&interview).unwrap()
            })
        });
    }
    group.finish();
}
```

**Constitution**:
- Zero-unwrap in benchmarks is relaxed for test setup
- Follow workspace lint rules in benchmark code
- Use `#[forbid(unsafe_code)]`

---

## Bead 3: Validation Performance Benchmarks

**ID**: `perf-validation`
**Title**: `quality: Add validation and quality score benchmarks`
**Type**: feature
**Priority**: 2
**Effort**: 2hr

### 0. Clarifications

**Resolved Questions**:
- Focus on `quality.rs` validation operations
- Benchmark `QualityScore::new()`, `ValidationReport::aggregate()`, `Validator::validate()`
- Test with varying message counts (10, 100, 1000)
- Measure validation logic performance, not data structures

**Open Questions**:
- Should we benchmark custom validators? (Yes, include in Phase 2)

**Assumptions**:
- `ValidationReport` aggregation is O(n) where n = message count
- Custom validators may have varying complexity
- JSON serialization in `to_json()` is part of the cost

### 1. EARS Requirements

**Ubiquitous Requirements**:
- THE SYSTEM SHALL benchmark validation report operations
- THE SYSTEM SHALL measure quality score calculation performance
- THE SYSTEM SHALL test validator composition (AND/OR logic)
- THE SYSTEM SHALL provide baseline metrics for regression detection

**Event-Driven Requirements**:
- WHEN validation benchmark runs, THE SYSTEM SHALL test with 10, 100, 1000 messages
- WHEN validation regression exceeds 20%, THE SYSTEM SHALL fail CI with notice

**Unwanted Behaviors**:
- IF benchmark tests trivial validators, THE SYSTEM SHALL NOT have unrealistic results, BECAUSE production validators are more complex
- IF benchmark includes I/O, THE SYSTEM SHALL NOT measure file write time

### 2. KIRK Contracts

**Preconditions**:
- Criterion.rs is available as dev-dependency
- `quality.rs` validation types are accessible
- Test data can be created with `ValidationMessage`

**Postconditions**:
- `benches/validation_bench.rs` exists
- Benchmarks cover: score creation, report aggregation, validator composition
- Baseline metrics documented

**Invariants**:
- Validators MUST do real work (not just return `Ok`)
- Test data MUST use `black_box()`
- Benchmark names MUST indicate operation and size

### 2.5 Research Requirements

**Files to Read**:
- `/home/lewis/src/clarity/clarity-core/src/quality.rs` (lines 1-1018)
- Focus on: `QualityScore` (lines 52-104), `ValidationReport` (lines 182-314), `Validator` (lines 477-535)

**Patterns to Find**:
- `ValidationReport::aggregate()` implementation (lines 209-214)
- `Validator::and()` and `Validator::or()` composition
- Custom validator patterns (lines 437-474)

**Research Questions**:
- What is the complexity of `quality_score()` calculation? (Lines 382-411)
- How many custom validators are typical? (Assume 5-10)

### 3. Inversions

**Security Inversions**:
- Test data MUST NOT reflect real validation errors
- Error messages MUST be generic

**Usability Inversions**:
- Benchmark output MUST clearly label operations
- Complex validators (OR/AND) SHOULD be tested

**Data Integrity Inversions**:
- Validation logic MUST be deterministic
- Random failure rates MUST use seeded RNG

### 4. ATDD Tests

**Happy Paths**:
- `cargo bench --bench validation` runs successfully
- Quality score calculation is O(1) (constant time)
- Report aggregation scales linearly with message count
- Validator composition has reasonable overhead

**Error Paths**:
- Benchmark fails if `ValidationMessage` API changes
- Custom validator benchmark fails if closure captures

**Edge Cases**:
- Empty validation report (0 messages)
- Report with 10,000 messages (stress test)
- 10-level deep AND/OR composition
- Custom validators with expensive operations

### 5. E2E Tests

**Pipeline Test**:
```bash
cargo bench --bench validation -- --save-baseline main
test -f target/criterion/validation/report/index.html
```

### 5.5 Verification Checkpoints

**Research Gate**:
- [ ] Read `quality.rs` validation implementation
- [ ] Study `Validator` composition patterns

**Test Gate**:
- [ ] Benchmarks compile and run
- [ ] Performance is acceptable (<1ms for 100 messages)

**Implementation Gate**:
- [ ] Quality score benchmarks implemented
- [ ] Report aggregation benchmarks implemented
- [ ] Validator composition benchmarks implemented

**Integration Gate**:
- [ ] Baseline metrics documented
- [ ] CI includes validation benchmarks

### 6. Implementation Tasks

**Phase 0: Research (15min)**
- [ ] Read `quality.rs` (lines 52-535)
- [ ] Study validation test patterns (lines 538-1018)

**Phase 1: Setup (30min)**
- [ ] Create `benches/validation_bench.rs`
- [ ] Add helpers to create validation messages
- [ ] Set up benchmark groups

**Phase 2: Core Benchmarks (PARALLEL - 4 tasks)**
- [ ] **Task A**: `bench_quality_score_creation()` - score validation overhead
- [ ] **Task B**: `bench_report_aggregate()` - aggregate 10/100/1000 messages
- [ ] **Task C**: `bench_validator_and()` - AND composition overhead
- [ ] **Task D**: `bench_validator_or()` - OR composition overhead

**Phase 3: Validation (30min)**
- [ ] Run benchmarks and establish baselines
- [ ] Verify performance characteristics
- [ ] Test with edge cases (0 messages, 10,000 messages)

**Phase 4: Integration (15min)**
- [ ] Document baselines
- [ ] Add CI configuration
- [ ] Update documentation

### 7. Failure Modes

**Symptoms** | **Causes** | **Debugging**
--- | --- | ---
Score validation too slow | Range checks on every creation | Check if validation can be deferred
Aggregation O(n²) | Inefficient vector concatenation | Profile with flamegraph
Validator composition slow | Excessive closure allocations | Use `Arc` for shared validators

### 7.5 Anti-Hallucination

**Read-Before-Write Rules**:
- MUST read `quality.rs` before benchmarking
- MUST check `ValidationMessage` field types

**API Existence Checks**:
- `QualityScore::new()` - check signature
- `ValidationReport::aggregate()` - verify accepts `Vec<Self>`
- `Validator::and()` / `Validator::or()` - check return types

### 7.6 Context Survival

**Progress Files**:
- `benches/validation_bench.rs`
- `docs/performance/validation_baseline.md`

### 8. Completion Checklist

- [ ] All benchmarks compile and run
- [ ] Quality score, aggregation, composition covered
- [ ] Baseline metrics documented
- [ ] CI configuration updated
- [ ] Documentation updated

### 9. Context

**Related Files**:
- `/home/lewis/src/clarity/clarity-core/src/quality.rs`

**Similar Implementations**:
- Validation tests in `quality.rs` (lines 538-1018)

### 10. AI Hints

**Do**:
- Use `black_box()` on validation inputs
- Test realistic validator complexity
- Measure both success and failure paths
- Benchmark `to_json()` serialization

**Don't**:
- Don't use trivial validators (always returns `Ok`)
- Don't forget error path validation

**Code Patterns**:
```rust
fn bench_validator_and(c: &mut Criterion) {
    let v1 = Validator::single(|s| Ok(s.to_string()));
    let v2 = Validator::single(|s| Ok(s.to_string()));
    let combined = v1.and(v2);

    c.bench_function("validator_and", |b| {
        b.iter(|| {
            let input = black_box("test input".to_string());
            combined.validate(&input).unwrap()
        })
    });
}
```

---

## Bead 4: Benchmark Infrastructure Setup

**ID**: `perf-infrastructure`
**Title**: `infra: Add Criterion.rs benchmark infrastructure and CI integration`
**Type**: chore
**Priority**: 0
**Effort**: 1hr

### 0. Clarifications

**Resolved Questions**:
- Criterion.rs version 0.5 with html_reports feature
- Benchmarks live in `clarity-core/benches/`
- CI runs benchmarks on merge to main branch
- Baseline stored in `benches/baseline/main`

### 1. EARS Requirements

**Ubiquitous Requirements**:
- THE SYSTEM SHALL provide Criterion.rs infrastructure for all benchmarks
- THE SYSTEM SHALL generate HTML reports for local development
- THE SYSTEM SHALL integrate benchmarks with CI pipeline
- THE SYSTEM SHALL store baseline metrics for regression detection

**Event-Driven Requirements**:
- WHEN developer runs `cargo bench`, THE SYSTEM SHALL execute all benchmarks
- WHEN PR is created, THE SYSTEM SHALL run benchmarks and compare against main

### 2. KIRK Contracts

**Preconditions**:
- Workspace `Cargo.toml` exists
- `clarity-core/` directory exists

**Postconditions**:
- Criterion.rs in workspace dev-dependencies
- `benches/` directory created
- CI configuration includes benchmark job
- Documentation explains how to run benchmarks

### 2.5 Research Requirements

**Files to Read**:
- `/home/lewis/src/clarity/Cargo.toml` - Workspace configuration
- Check for existing CI configuration (`.github/workflows/`, `.moon/tasks.yml`)

**Patterns to Find**:
- Existing workspace dependency patterns
- CI benchmark job examples

### 3. Inversions

**Integration Failures**:
- If Criterion conflicts with other deps, MUST resolve version
- If CI fails on benchmarks, MUST make them optional for PRs

### 4. ATDD Tests

**Happy Paths**:
- `cargo bench` runs all benchmarks
- `cargo bench --bench db_pool` runs specific benchmark
- Criterion HTML report generates
- CI benchmark job succeeds

### 5. E2E Tests

**Pipeline Test**:
```bash
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

### 6. Implementation Tasks

**Phase 0: Research (15min)**
- [ ] Read workspace `Cargo.toml`
- [ ] Check existing CI configuration

**Phase 1: Setup (30min)**
- [ ] Add `criterion` to workspace dev-dependencies
- [ ] Create `clarity-core/benches/` directory
- [ ] Create `benches/mod.rs` if needed

**Phase 2: Configuration (15min)**
- [ ] Create `benches/criterion.toml` configuration
- [ ] Configure output format for CI (non-interactive)

**Phase 3: CI Integration (30min)**
- [ ] Add benchmark job to CI configuration
- [ ] Configure baseline storage and comparison
- [ ] Set performance regression thresholds

**Phase 4: Documentation (30min)**
- [ ] Create `docs/performance/README.md`
- [ ] Document how to run benchmarks
- [ ] Document how to add new benchmarks
- [ ] Document baseline management

### 7. Failure Modes

**Symptoms** | **Causes** | **Debugging**
--- | --- | ---
Criterion not found | Not in dev-dependencies | `cargo tree -i criterion`
Benchmarks not found | `benches/` not in `Cargo.toml` | Add `[[bench]]` sections
CI timeout | Benchmarks too long | Make benchmarks optional in PRs

### 7.5 Anti-Hallucination

**Read-Before-Write Rules**:
- MUST read `Cargo.toml` before adding dependencies
- MUST check existing CI config before modifying

### 7.6 Context Survival

**Progress Files**:
- `Cargo.toml` - Criterion dependency
- `benches/criterion.toml` - Configuration
- `docs/performance/README.md` - Documentation

### 8. Completion Checklist

- [ ] Criterion.rs added to workspace
- [ ] `benches/` directory created
- [ ] CI configuration includes benchmarks
- [ ] Documentation created
- [ ] `cargo bench` runs successfully

### 9. Context

**Related Files**:
- `/home/lewis/src/clarity/Cargo.toml`
- CI configuration files

### 10. AI Hints

**Do**:
- Use `criterion = "0.5"` for latest stable
- Enable `html_reports` feature for local dev
- Configure `sample_size = 100` for faster CI runs
- Use `--output-format bencher` for CI output

**Don't**:
- Don't add Criterion as regular dependency (dev-dependencies only)
- Don't commit `target/criterion/` directory

**Code Patterns**:
```toml
# In Cargo.toml
[workspace.dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

# In benches/criterion.toml
[ci]
bench = true
verbose = true

[[bench]]
name = "db_pool"
harness = false
```

---

## Summary

This performance benchmarking suite provides comprehensive coverage of Clarity's critical performance paths:

1. **Database Operations** (`perf-db-pool`): SQLite pool creation, connection acquisition, concurrent access
2. **Serialization** (`perf-json-serialize`): JSON formatting for various interview sizes
3. **Validation** (`perf-validation`): Quality scores, report aggregation, validator composition
4. **Infrastructure** (`perf-infrastructure`): Criterion.rs setup, CI integration, documentation

**Total Estimated Effort**: 7 hours (2hr + 2hr + 2hr + 1hr)

**Priority Order**: Infrastructure first (enables others), then db-pool and json-serialize (priority 1), then validation (priority 2)

**Performance Thresholds**:
- Database operations: 10% regression tolerance
- JSON serialization: 15% regression tolerance
- Validation: 20% regression tolerance

**Baseline Storage**: `benches/baseline/main/`

**CI Strategy**: Run on merge to main, compare against main branch baseline
