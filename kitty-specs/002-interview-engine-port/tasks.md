# Tasks: Intent-CLI Full Port (Gleam → Rust)

**Feature**: 002-interview-engine-port
**Generated**: 2026-02-27
**Total Work Packages**: 33
**Estimated Total Lines**: ~15,602

## Setup

### WP01: Project Foundation

**Goal**: Set up the intent module structure and dependencies
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~280 lines
**Implementation Command**: `spec-kitty implement WP01`

**Included Subtasks**:
- [ ] T001: Create `clarity-web/src/intent/` directory structure with mod.rs
- [ ] T002: Add serde, serde_json, chrono, regex, thiserror, anyhow, uuid dependencies
- [ ] T003: Create `clarity-web/tests/intent/` test directory
- [ ] T004: Copy CUE schemas from `/tmp/intent-cli/schema/` to `clarity-web/schemas/`

**Implementation Sketch**:
1. Create directory structure: `src/intent/{interview,plan,beads,quality,validation,batch,documents,templates,cli,util}/mod.rs`
2. Update `Cargo.toml` with new dependencies (use specific versions)
3. Create test directory mirroring source structure
4. Copy 8 CUE schema files verbatim

**Parallel Opportunities**: None (foundation must exist first)

**Dependencies**: None

**Risks**:
- Dependency version conflicts with existing clarity-web deps

**Definition of Done**:
- All directories exist with mod.rs files
- Dependencies compile successfully
- CUE schemas copied and accessible
- `cargo check` passes on empty module

---

## Foundation Tier (Core Types & Infrastructure)

### WP02: Core Spec Types

**Goal**: Port Spec, Feature, Behavior, Verification types from types.gleam (90 lines)
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~320 lines
**Implementation Command**: `spec-kitty implement WP02`

**Dependencies**: WP01

**Included Subtasks**:
- [ ] T005: Create `clarity-web/src/intent/types.rs` with Spec struct
- [ ] T006: Add Feature struct with behaviors field
- [ ] T007: Add Behavior struct with verifications, requires, tags fields
- [ ] T008: Add Verification struct with criteria and examples
- [ ] T009: Derive Debug, Clone, PartialEq, Serialize, Deserialize for all types

**Implementation Sketch**:
1. Define Spec struct with all fields from Gleam
2. Define Feature, Behavior, Verification structs
3. Use `HashMap<String, String>` for extracted fields
4. Use `serde_json::Value` for JSON fields
5. Add derive macros for serialization

**Parallel Opportunities**: None

**Risks**:
- JSON field type mapping (Gleam Dynamic → Rust serde_json::Value)

---

### WP03: Invariant and AntiPattern Types

**Goal**: Port Invariant, AntiPattern, AIHints types from types.gleam
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~300 lines
**Implementation Command**: `spec-kitty implement WP03`

**Dependencies**: WP02

**Included Subtasks**:
- [ ] T010: Add Invariant struct with name, description, criteria fields
- [ ] T011: Add AntiPattern struct with bad_example and good_example JSON fields
- [ ] T012: Add AIHints struct with implementation, entities, security, pitfalls fields
- [ ] T013: Add EntityHint, ImplementationHints, SecurityHints helper structs
- [ ] T014: Update Spec struct to include invariants, anti_patterns, ai_hints fields

**Implementation Sketch**:
1. Define helper structs first (EntityHint, ImplementationHints, SecurityHints)
2. Define Invariant with criteria as Vec<String>
3. Define AntiPattern with bad_example/good_example as serde_json::Value
4. Define AIHints with HashMap<String, EntityHint>
5. Update Spec struct fields

**Parallel Opportunities**: None (depends on WP02)

---

### WP04: Error Type System

**Goal**: Port errors.gleam (234 lines) with contextual error reporting
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~380 lines
**Implementation Command**: `spec-kitty implement WP04`

**Dependencies**: WP01

**Included Subtasks**:
- [ ] T015: Create `clarity-web/src/intent/errors.rs` with IntentError enum
- [ ] T016: Add ContextualError wrapper with field suggestions
- [ ] T017: Implement Levenshtein distance for field suggestions
- [ ] T018: Add ValidationError and FieldFailure types
- [ ] T019: Implement user-friendly error formatting

**Implementation Sketch**:
1. Define IntentError enum covering all Gleam error cases
2. Create ContextualError struct with error + suggestions
3. Implement Levenshtein distance function (std-algo)
4. Add Display impl for user-friendly formatting
5. Add thiserror derive for error source chains

**Parallel Opportunities**: Can run in parallel with WP02-WP03

**Risks**:
- Levenshtein algorithm correctness

---

### WP05: Security Validation

**Goal**: Port security.gleam (296 lines) with path/shell/ReDoS validation
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~400 lines
**Implementation Command**: `spec-kitty implement WP05`

**Dependencies**: WP01, WP04

**Included Subtasks**:
- [ ] T020: Create `clarity-web/src/intent/security.rs` module
- [ ] T021: Implement check_literal_traversal() for path validation
- [ ] T022: Implement check_shell_metacharacters() function
- [ ] T023: Implement check_url_encoded() for encoded path traversal
- [ ] T024: Implement check_regex_redos() for catastrophic backtracking detection
- [ ] T025: Implement validate_session_id() format check

**Implementation Sketch**:
1. Define SecurityError enum variants
2. Implement path traversal checks (../, ..\, %2e%2e patterns)
3. Implement shell metacharacter detection (;, &, |, $, `, \n, \r)
4. Implement URL encoding detection (%XX patterns)
5. Implement ReDoS detection (nested quantifiers, alternation depth)

**Parallel Opportunities**: None

**Risks**:
- ReDoS detection may have false positives
- Path traversal edge cases (Windows vs Unix)

---

### WP06: Format Validators

**Goal**: Port formats.gleam (569 lines) with RFC-compliant validators
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~450 lines
**Implementation Command**: `spec-kitty implement WP06`

**Dependencies**: WP01, WP04

**Included Subtasks**:
- [ ] T026: Create `clarity-web/src/intent/formats.rs` module
- [ ] T027: Implement validate_email() with RFC 5322 compliant parsing
- [ ] T028: Implement validate_uuid() with version/variant checking
- [ ] T029: Implement validate_uri() with RFC 3986 scheme validation
- [ ] T030: Implement validate_iso8601() with calendar validation
- [ ] T031: Add helper functions: is_valid_hex, is_leap_year, get_days_in_month

**Implementation Sketch**:
1. Define FormatError enum
2. Implement email parsing (@ split, local/domain validation)
3. Implement UUID format (8-4-4-4-12) with version/variant bit checks
4. Implement URI parsing (scheme://authority validation)
5. Implement ISO8601 with leap year and days-in-month validation

**Parallel Opportunities**: Can run in parallel with WP05

**Risks**:
- Calendar edge cases (leap years, month lengths)
- URI scheme validation complexity

---

### WP07: Case-Insensitive Utilities

**Goal**: Port case_insensitive.gleam (85 lines) for string matching
**Priority**: P1 (High)
**Estimated Prompt Size**: ~220 lines
**Implementation Command**: `spec-kitty implement WP07`

**Dependencies**: WP01

**Included Subtasks**:
- [ ] T032: Create `clarity-web/src/intent/util/case_insensitive.rs`
- [ ] T033: Implement contains_any_ignore_case() function
- [ ] T034: Implement equals_ignore_case() function
- [ ] T035: Add unit tests for case-insensitive operations

**Implementation Sketch**:
1. Use Rust's to_lowercase() for comparison
2. Handle Unicode edge cases
3. Implement contains_any with iterator pattern

**Parallel Opportunities**: Can run in parallel with WP05, WP06

---

### WP08: Array Indexing Utilities

**Goal**: Port array_indexing.gleam (287 lines) for JSON navigation
**Priority**: P1 (High)
**Estimated Prompt Size**: ~350 lines
**Implementation Command**: `spec-kitty implement WP08`

**Dependencies**: WP01

**Included Subtasks**:
- [ ] T036: Create `clarity-web/src/intent/util/array_indexing.rs`
- [ ] T037: Implement ArrayIndexing enum (NoArray, Index, LastN, All)
- [ ] T038: Implement parse_path_component() for [index] syntax
- [ ] T039: Implement split_path() for dot/bracket navigation
- [ ] T040: Implement navigate_path() for JSON traversal

**Implementation Sketch**:
1. Define ArrayIndexing enum for index types
2. Parse integer indices from bracket notation
3. Handle negative indices (counting from end)
4. Implement path splitting on dots and brackets
5. Navigate serde_json::Value using indices

**Parallel Opportunities**: Can run in parallel with WP05-WP07

**Risks**:
- JSON navigation complexity
- Negative index edge cases

---

### WP09: Interview Types

**Goal**: Port question_types.gleam (42 lines) for interview system
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~280 lines
**Implementation Command**: `spec-kitty implement WP09`

**Dependencies**: WP01

**Included Subtasks**:
- [ ] T041: Create `clarity-web/src/intent/interview/types.rs`
- [ ] T042: Add Profile enum (Api, Cli, Event, Data, Workflow, Ui)
- [ ] T043: Add InterviewStage enum (Discovery, Refinement, Validation, Complete, Paused)
- [ ] T044: Add Perspective enum (User, Developer, Ops, Security, Business)
- [ ] T045: Add QuestionPriority enum (Critical, Important, NiceToHave)
- [ ] T046: Add QuestionCategory enum (HappyPath, ErrorCase, EdgeCase, Constraint, Dependency, NonFunctional)

**Implementation Sketch**:
1. Create interview/types.rs module
2. Define all enums with serde derives
3. Add #[serde(rename_all = "lowercase")] for JSON compatibility
4. Re-export at intent/interview/mod.rs level

**Parallel Opportunities**: Can run in parallel with WP05-WP08

**Risks**:
- JSON naming convention compatibility

---

### WP10: Answer, Gap, Conflict Types

**Goal**: Port core interview data structures
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~380 lines
**Implementation Command**: `spec-kitty implement WP10`

**Dependencies**: WP09

**Included Subtasks**:
- [ ] T047: Add Answer struct with all fields (question_id, perspective, round, response, extracted, confidence, timestamp)
- [ ] T048: Add Gap struct (id, field, description, blocking, suggested_default, why_needed, round, resolved, resolution)
- [ ] T049: Add Conflict struct (id, between, description, impact, options, chosen)
- [ ] T050: Add ConflictResolution struct (option, description, tradeoffs, recommendation)
- [ ] T051: Add Question struct for interview questions

**Implementation Sketch**:
1. Define structs matching Gleam record structure
2. Use HashMap<String, String> for extracted fields
3. Use Option<T> for nullable fields (chosen, completed_at)
4. Add Default impls for convenient construction

**Parallel Opportunities**: None

**Risks**:
- HashMap serialization ordering differences

---

### WP11: InterviewSession Type

**Goal**: Define the central interview state machine type
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~320 lines
**Implementation Command**: `spec-kitty implement WP11`

**Dependencies**: WP10

**Included Subtasks**:
- [ ] T052: Add InterviewSession struct with all fields
- [ ] T053: Implement InterviewSession::new() constructor
- [ ] T054: Implement InterviewSession::with_stage() helper
- [ ] T055: Add Default impl for InterviewSession
- [ ] T056: Implement state transition validation methods

**Implementation Sketch**:
1. Define InterviewSession struct (id, profile, created_at, updated_at, completed_at, stage, rounds_completed, answers, gaps, conflicts, raw_notes, current_phase, completed_phases)
2. Implement new() with default Discovery stage
3. Implement with_stage() for manual stage setting
4. Validate stage transitions (no backwards transitions)

**Parallel Opportunities**: None

**Risks**:
- State machine transition logic correctness

---

### WP12: Interview Engine Core

**Goal**: Port interview.gleam session management (851 lines)
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~480 lines
**Implementation Command**: `spec-kitty implement WP12`

**Dependencies**: WP11

**Included Subtasks**:
- [ ] T057: Implement InterviewSession::add_answer() method
- [ ] T058: Implement InterviewSession::complete_round() method
- [ ] T059: Implement InterviewSession::get_current_round() method
- [ ] T060: Implement InterviewSession::can_proceed() method
- [ ] T061: Implement InterviewSession::complete_phase() method
- [ ] T062: Implement InterviewSession::can_execute_phase() method

**Implementation Sketch**:
1. add_answer: push to answers vec, update updated_at, detect gaps
2. complete_round: increment rounds_completed, check stage transition rules
3. get_current_round: return rounds_completed + 1
4. can_proceed: check for blocking gaps and unresolved conflicts
5. complete_phase: add phase to completed_phases
6. can_execute_phase: check if phase dependencies satisfied

**Parallel Opportunities**: None

**Risks**:
- Stage transition logic complexity

---

### WP13: Gap Detection

**Goal**: Implement profile-specific gap detection from interview.gleam
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~420 lines
**Implementation Command**: `spec-kitty implement WP13`

**Dependencies**: WP12

**Included Subtasks**:
- [ ] T063: Implement get_required_fields() for each Profile variant
- [ ] T064: Implement InterviewSession::detect_gaps() method
- [ ] T065: Implement InterviewSession::check_for_gaps() method
- [ ] T066: Implement InterviewSession::get_blocking_gaps() method
- [ ] T067: Implement InterviewSession::resolve_gap() method

**Implementation Sketch**:
1. Define REQUIRED_FIELDS constant per profile
2. detect_gaps: check answered fields against required fields
3. check_for_gaps: detect gaps for specific question/answer
4. get_blocking_gaps: filter gaps by blocking field
5. resolve_gap: find gap by id, mark resolved, set resolution

**Parallel Opportunities**: Can run in parallel with WP14

**Risks**:
- Profile-specific field mapping accuracy

---

### WP14: Conflict Detection

**Goal**: Implement conflict detection (CAP theorem, anonymous+audit, perspective conflicts)
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~400 lines
**Implementation Command**: `spec-kitty implement WP14`

**Dependencies**: WP12

**Included Subtasks**:
- [ ] T068: Implement detect_cap_conflict() helper
- [ ] T069: Implement detect_anonymous_audit_conflict() helper
- [ ] T070: Implement detect_perspective_conflicts() helper
- [ ] T071: Implement InterviewSession::detect_conflicts() method
- [ ] T072: Implement InterviewSession::check_for_conflicts() method
- [ ] T073: Implement InterviewSession::resolve_conflict() method

**Implementation Sketch**:
1. detect_cap_conflict: check for "fast"/"latency" + "consistent"/"accurate"
2. detect_anonymous_audit_conflict: check for "anonymous" + "audit"/"log"
3. detect_perspective_conflicts: Developer vs Ops on consistency/latency
4. detect_conflicts: run all conflict detectors
5. check_for_conflicts: check new answer against existing
6. resolve_conflict: find conflict by id, set chosen option

**Parallel Opportunities**: Can run in parallel with WP13

**Risks**:
- False positive conflict detection
- String matching edge cases

---

### WP15: Interview Storage

**Goal**: Port interview_storage.gleam (1162 lines) for JSONL persistence
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~520 lines
**Implementation Command**: `spec-kitty implement WP15`

**Dependencies**: WP11

**Included Subtasks**:
- [ ] T074: Create `clarity-web/src/intent/interview/storage.rs`
- [ ] T075: Implement session_to_jsonl_line() function
- [ ] T076: Implement append_session_to_jsonl() function
- [ ] T077: Implement list_sessions_from_jsonl() function
- [ ] T078: Implement get_session_from_jsonl() function
- [ ] T079: Implement create_snapshot() and append_to_history() functions
- [ ] T080: Implement list_session_history() function

**Implementation Sketch**:
1. Use BufWriter for efficient file appending
2. Use BufReader for streaming JSONL parsing
3. Handle file-not-found gracefully for empty directories
4. Create parent directories automatically
5. Validate JSON format before parsing

**Parallel Opportunities**: None

**Risks**:
- File I/O error handling
- Large file performance

---

### WP16: Session Diffing

**Goal**: Implement session diffing and history tracking
**Priority**: P1 (High)
**Estimated Prompt Size**: ~380 lines
**Implementation Command**: `spec-kitty implement WP16`

**Dependencies**: WP15

**Included Subtasks**:
- [ ] T081: Implement diff_sessions() function
- [ ] T082: Implement AnswerDiff detection logic
- [ ] T083: Implement SessionDiff struct and formatting
- [ ] T084: Implement format_diff() function
- [ ] T085: Add SessionSnapshot and AnswerVersion types

**Implementation Sketch**:
1. Compare answers between two sessions by question_id
2. Detect Added, Modified, Removed changes
3. Compare stage, gaps, conflicts
4. Generate human-readable diff format
5. Track answer versions for history

**Parallel Opportunities**: None

**Risks**:
- Diff algorithm correctness
- Edge case handling (empty sessions)

---

## Parsing & Loading Tier

### WP17: JSON Parser

**Goal**: Port parser.gleam (320 lines) for JSON to Spec parsing
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~360 lines
**Implementation Command**: `spec-kitty implement WP17`

**Dependencies**: WP02, WP03

**Included Subtasks**:
- [ ] T086: Create `clarity-web/src/intent/parser.rs`
- [ ] T087: Implement parse_spec() function
- [ ] T088: Implement parse_feature() helper
- [ ] T089: Implement parse_behavior() helper
- [ ] T090: Implement dynamic_to_json() conversion
- [ ] T091: Implement sanitize_field_name() helper

**Implementation Sketch**:
1. Use serde_json::from_str for initial parse
2. Convert serde_json::Value to Spec struct
3. Handle missing fields with defaults
4. Sanitize field names (remove invalid characters)
5. Return detailed errors for invalid JSON

**Parallel Opportunities**: Can run in parallel with WP18

**Risks**:
- Dynamic type mapping
- Error message quality

---

### WP18: CUE Loader

**Goal**: Port loader.gleam (240 lines) for CUE file loading
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~320 lines
**Implementation Command**: `spec-kitty implement WP18`

**Dependencies**: WP01

**Included Subtasks**:
- [ ] T092: Create `clarity-web/src/intent/loader.rs`
- [ ] T093: Implement load_cue_file() function
- [ ] T094: Implement validate_cue() function using `cue` CLI
- [ ] T095: Implement load_spec_from_cue() function
- [ ] T096: Handle CUE command execution errors

**Implementation Sketch**:
1. Use std::process::Command to run `cue vet`
2. Read CUE file contents
3. Parse CUE output for validation errors
4. Convert validated CUE to JSON via `cue export`
5. Parse JSON into Spec

**Parallel Opportunities**: Can run in parallel with WP17

**Risks**:
- `cue` CLI availability
- Command execution error handling

---

### WP19: Spec Validator

**Goal**: Port spec_validator.gleam (578 lines) with circular dependency detection
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~460 lines
**Implementation Command**: `spec-kitty implement WP19`

**Dependencies**: WP17, WP18

**Included Subtasks**:
- [ ] T097: Create `clarity-web/src/intent/validation/spec_validator.rs`
- [ ] T098: Implement validate_spec_structure() function
- [ ] T099: Implement detect_circular_dependencies() using DFS
- [ ] T100: Implement detect_duplicate_behaviors() function
- [ ] T101: Implement validate_references() function
- [ ] T102: Implement ValidationResult and ValidationError types

**Implementation Sketch**:
1. Validate Spec has required fields (name, version, features)
2. Build dependency graph from behavior.requires
3. Run DFS to detect cycles
4. Check for duplicate behavior names within features
5. Validate all references (requires fields point to existing behaviors)
6. Return detailed validation report

**Parallel Opportunities**: None

**Risks**:
- DFS algorithm correctness
- Graph construction complexity

---

### WP20: Interpolation System

**Goal**: Port interpolate.gleam (301 lines) for variable interpolation
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~400 lines
**Implementation Command**: `spec-kitty implement WP20`

**Dependencies**: WP08

**Included Subtasks**:
- [ ] T103: Create `clarity-web/src/intent/validation/interpolation.rs`
- [ ] T104: Implement Context type with variables HashMap
- [ ] T105: Implement interpolate_string() for ${var} replacement
- [ ] T106: Implement resolve_path() for dot/bracket navigation
- [ ] T107: Implement navigate_json() for array indexing
- [ ] T108: Implement extract_capture() helper

**Implementation Sketch**:
1. Define Context struct with variables, request_body, response_body
2. Use regex to find ${var} patterns
3. Resolve variables by path (response.body.user.id)
4. Support array indexing: items[0], items[-1]
5. Handle missing variables gracefully

**Parallel Opportunities**: Can run in parallel with WP21

**Risks**:
- Regex performance
- Array indexing edge cases

---

### WP21: Validation Rules

**Goal**: Port rule.gleam (501 lines) for validation rule engine
**Priority**: P1 (High)
**Estimated Prompt Size**: ~420 lines
**Implementation Command**: `spec-kitty implement WP21`

**Dependencies**: WP20

**Included Subtasks**:
- [ ] T109: Create `clarity-web/src/intent/validation/rule.rs`
- [ ] T110: Implement Rule type definition
- [ ] T111: Implement apply_rule() function
- [ ] T112: Implement validate_with_rules() function
- [ ] T113: Implement RuleError and RuleResult types

**Implementation Sketch**:
1. Define Rule enum (Required, Pattern, Range, Custom)
2. Implement rule application logic
3. Collect rule violations
4. Return detailed rule results

**Parallel Opportunities**: Can run in parallel with WP20

**Risks**:
- Rule engine complexity
- Error aggregation

---

## Planning Tier

### WP22: Plan Mode

**Goal**: Port plan_mode.gleam (923 lines) for execution planning
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~500 lines
**Implementation Command**: `spec-kitty implement WP22`

**Dependencies**: WP12, WP13

**Included Subtasks**:
- [ ] T114: Create `clarity-web/src/intent/plan/mod.rs`
- [ ] T115: Create `clarity-web/src/intent/plan/plan_mode.rs`
- [ ] T116: Implement ExecutionPlan type
- [ ] T117: Implement Phase type with beads and status
- [ ] T118: Implement PlanBead type
- [ ] T119: Implement compute_plan() function
- [ ] T120: Implement apply_phase_gating() function

**Implementation Sketch**:
1. Define ExecutionPlan, Phase, PlanBead structs
2. compute_plan: analyze session, generate beads, organize into phases
3. apply_phase_gating: check phase completion before execution
4. Handle blockers and dependencies

**Parallel Opportunities**: Can run in parallel with WP23

**Risks**:
- Dependency resolution complexity
- Phase ordering logic

---

### WP23: Plan Next

**Goal**: Port plan_next.gleam (102 lines) for next action determination
**Priority**: P1 (High)
**Estimated Prompt Size**: ~280 lines
**Implementation Command**: `spec-kitty implement WP23`

**Dependencies**: WP22

**Included Subtasks**:
- [ ] T121: Create `clarity-web/src/intent/plan/plan_next.rs`
- [ ] T122: Implement get_next_action() function
- [ ] T123: Implement determine_next_phase() helper
- [ ] T124: Implement get_actionable_beads() function

**Implementation Sketch**:
1. Analyze current session state
2. Return next recommended action
3. Determine which phase to work on next
4. Filter beads by actionable status

**Parallel Opportunities**: Can run in parallel with WP22

---

### WP24: Plan Emit Beads

**Goal**: Port plan_emit_beads.gleam (277 lines) with idempotency
**Priority**: P1 (High)
**Estimated Prompt Size**: ~360 lines
**Implementation Command**: `spec-kitty implement WP24`

**Dependencies**: WP22

**Included Subtasks**:
- [ ] T125: Create `clarity-web/src/intent/plan/plan_emit_beads.rs`
- [ ] T126: Implement emit_beads() function
- [ ] T127: Implement check_existing_beads() for idempotency
- [ ] T128: Implement EmissionResult type
- [ ] T129: Implement dry-run mode support

**Implementation Sketch**:
1. Check existing bead titles in tracker
2. Filter out already-created beads
3. Generate commands for new beads
4. Support dry-run mode for preview
5. Return emission summary

**Parallel Opportunities**: Can run in parallel with WP23

**Risks**:
- Bead tracker integration
- Idempotency correctness

---

### WP25: Resolver

**Goal**: Port resolver.gleam (258 lines) for dependency resolution
**Priority**: P1 (High)
**Estimated Prompt Size**: ~340 lines
**Implementation Command**: `spec-kitty implement WP25`

**Dependencies**: WP22

**Included Subtasks**:
- [ ] T130: Create `clarity-web/src/intent/plan/resolver.rs`
- [ ] T131: Implement resolve_dependencies() function
- [ ] T132: Implement detect_cycles() function
- [ ] T133: Implement topological_sort() function
- [ ] T134: Implement ResolutionResult type

**Implementation Sketch**:
1. Build dependency graph from bead.requires
2. Detect circular dependencies
3. Perform topological sort for execution order
4. Return resolution results

**Parallel Opportunities**: Can run in parallel with WP24

**Risks**:
- Graph algorithm correctness
- Cycle detection edge cases

---

## Bead Generation Tier

### WP26: Bead Templates

**Goal**: Port bead_templates.gleam (778 lines) for work item generation
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~480 lines
**Implementation Command**: `spec-kitty implement WP26`

**Dependencies**: WP12

**Included Subtasks**:
- [ ] T135: Create `clarity-web/src/intent/beads/mod.rs`
- [ ] T136: Create `clarity-web/src/intent/beads/templates.rs`
- [ ] T137: Implement BeadRecord type
- [ ] T138: Implement BeadStats type
- [ ] T139: Implement generate_beads_from_session() function
- [ ] T140: Implement profile-specific generators (generate_api_beads, etc.)
- [ ] T141: Implement beads_to_jsonl() and beads_to_enhanced_cue() functions

**Implementation Sketch**:
1. Define BeadRecord with all required fields
2. Generate beads based on session answers and profile
3. Profile-specific filtering and prioritization
4. Output as JSONL and 16-section CUE format
5. Generate acceptance criteria from answers

**Parallel Opportunities**: Can run in parallel with WP27

**Risks**:
- CUE template correctness
- Profile-specific logic accuracy

---

### WP27: Bead Feedback

**Goal**: Port bead_feedback.gleam (410 lines) for feedback collection
**Priority**: P1 (High)
**Estimated Prompt Size**: ~320 lines
**Implementation Command**: `spec-kitty implement WP27`

**Dependencies**: WP26

**Included Subtasks**:
- [ ] T142: Create `clarity-web/src/intent/beads/feedback.rs`
- [ ] T143: Implement BeadFeedback type
- [ ] T144: Implement collect_feedback() function
- [ ] T145: Implement update_bead_status() function
- [ ] T146: Implement FeedbackStatus type

**Implementation Sketch**:
1. Define feedback collection types
2. Track bead status through lifecycle
3. Store feedback with bead references
4. Support status updates

**Parallel Opportunities**: Can run in parallel with WP26

---

## Quality Tier

### WP28: Quality Analyzer

**Goal**: Port quality_analyzer.gleam (470 lines) for quality scoring
**Priority**: P0 (Critical)
**Estimated Prompt Size**: ~420 lines
**Implementation Command**: `spec-kitty implement WP28`

**Dependencies**: WP02

**Included Subtasks**:
- [ ] T147: Create `clarity-web/src/intent/quality/mod.rs`
- [ ] T148: Create `clarity-web/src/intent/quality/analyzer.rs`
- [ ] T149: Implement QualityReport type
- [ ] T150: Implement QualityIssue enum
- [ ] T151: Implement analyze_spec() function
- [ ] T152: Implement score calculation functions (coverage, clarity, testability, ai_readiness)
- [ ] T153: Implement format_report() function

**Implementation Sketch**:
1. Define QualityReport with scores (0-100) and issues
2. Define QualityIssue enum variants
3. Calculate coverage: error tests + auth tests + edge cases + invariants
4. Calculate clarity: intent ratio + notes ratio - vague penalties
5. Calculate testability: dependencies + preconditions + postconditions + examples
6. Calculate AI readiness: AI hints + verification + examples
7. Return overall score as weighted average

**Parallel Opportunities**: Can run in parallel with WP29, WP33

**Risks**:
- Scoring algorithm correctness
- Weight calibration

---

### WP33: Spec Linter

**Goal**: Port spec_linter.gleam (383 lines) for spec linting and style checking
**Priority**: P1 (High)
**Estimated Prompt Size**: ~360 lines
**Implementation Command**: `spec-kitty implement WP33`

**Dependencies**: WP02

**Included Subtasks**:
- [ ] T174: Create `clarity-web/src/intent/quality/linter.rs`
- [ ] T175: Implement LintRule enum (NamingConvention, RequiredFields, DeprecatedPattern, etc.)
- [ ] T176: Implement LintResult type with severity levels
- [ ] T177: Implement lint_spec() function
- [ ] T178: Implement lint_feature() and lint_behavior() helpers
- [ ] T179: Implement format_lint_report() function

**Implementation Sketch**:
1. Define LintRule enum covering all lint rule types
2. Define LintResult with file, line, rule, message, severity
3. lint_spec: run all lint rules on spec
4. Check naming conventions (snake_case for behaviors, etc.)
5. Check required fields presence
6. Flag deprecated patterns
7. Return formatted lint report

**Parallel Opportunities**: Can run in parallel with WP28

**Risks**:
- Lint rule completeness
- False positive rate

**Rust Contract Specification**:

### Preconditions

| ID | Precondition |
|----|--------------|
| P1 | Spec structure validated (run spec_validator first) |
| P2 | Lint rules loaded from configuration |

### Postconditions

| ID | Postcondition |
|----|---------------|
| Q1 | Result<LintReport, LintError> for all fallible ops |
| Q2 | No panics on malformed specs |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | Linting is deterministic (same input = same output) |
| I2 | Lint rules are configurable but have sensible defaults |

## Violation Examples (Rust Contract Requirement)

### WP33 Violation Examples:

| Contract | Violation Call | Expected Error |
|----------|----------------|----------------|
| P1 | `lint_spec(spec_that_failed_validation)` | `Err(LintError::InvalidSpecStructure)` |
| P2 | `lint_spec_with_rules(empty_rules_config)` | `Err(LintError::NoRulesConfigured)` |
| I1 | Same spec produces different lint results | Test failure |
| Q1 | `lint_spec(malformed_json_spec)` | `Err(LintError::ParseError)` not panic |
| Q2 | Code contains `.unwrap()` on rule application | Fails code review |

### Test Parity:
- `test_invalid_spec_structure_returns_error` covers P1
- `test_no_rules_configured_returns_error` covers P2
- `test_linting_is_deterministic` covers I1
- `test_no_unwrap_in_linter_module` covers Q2

---

## Definition of Done

- [ ] Core functionality implemented
- [ ] Contract tests pass
- [ ] No unwrap/expect in production code
- [ ] Documentation complete

---

### WP29: Effects Analyzer

**Goal**: Port effects_analyzer.gleam (286 lines) for second-order effect detection
**Priority**: P1 (High)
**Estimated Prompt Size**: ~360 lines
**Implementation Command**: `spec-kitty implement WP29`

**Dependencies**: WP02

**Included Subtasks**:
- [ ] T154: Create `clarity-web/src/intent/quality/effects.rs`
- [ ] T155: Implement EffectType enum
- [ ] T156: Implement Effect type
- [ ] T157: Implement analyze_behavior() function
- [ ] T158: Implement analyze_spec() function
- [ ] T159: Implement format_effects_json() and format_effects_cli() functions

**Implementation Sketch**:
1. Define EffectType (StateChange, Notification, Cascade, RaceCondition, RollbackRequired)
2. Define Effect with type, description, severity, suggestion
3. Analyze behavior intent for effects (create/update/delete operations)
4. Detect cascade effects (related records affected)
5. Detect race conditions (concurrent modifications)
6. Detect rollback requirements (reversibility)

**Parallel Opportunities**: Can run in parallel with WP28

**Risks**:
- Effect detection accuracy
- False positive/negative rate

---

### WP30: Quality Improver

**Goal**: Port improver.gleam (403 lines) for spec improvement suggestions
**Priority**: P1 (High)
**Estimated Prompt Size**: ~340 lines
**Implementation Command**: `spec-kitty implement WP30`

**Dependencies**: WP28

**Included Subtasks**:
- [ ] T160: Create `clarity-web/src/intent/quality/improver.rs`
- [ ] T161: Implement suggest_improvements() function
- [ ] T162: Implement add_missing_tests() suggestion
- [ ] T163: Implement improve_vague_rules() suggestion
- [ ] T164: Implement add_examples() suggestion

**Implementation Sketch**:
1. Analyze quality report issues
2. Generate specific improvement suggestions
3. Suggest missing test cases
4. Suggest clarifications for vague rules
5. Suggest examples for behaviors

**Parallel Opportunities**: Can run in parallel with WP29

---

## Remaining Tier (lower priority)

### WP31: Semantic Validator

**Goal**: Port semantic_validator.gleam (332 lines)
**Priority**: P1 (High)
**Estimated Prompt Size**: ~360 lines
**Implementation Command**: `spec-kitty implement WP31`

**Dependencies**: WP19

**Included Subtasks**:
- [ ] T165: Create `clarity-web/src/intent/validation/semantic_validator.rs`
- [ ] T166: Implement validate_semantics() function
- [ ] T167: Implement cross_reference_validation() function
- [ ] T168: Implement consistency_checks() function

**Implementation Sketch**:
1. Validate cross-references between behaviors
2. Check consistency of terminology
3. Validate semantic constraints
4. Return semantic validation report

**Parallel Opportunities**: Can run in parallel with WP32

---

### WP32: Spec Templates

**Goal**: Port spec_templates.gleam (1206 lines) for template generation
**Priority**: P1 (High)
**Estimated Prompt Size**: ~500 lines
**Implementation Command**: `spec-kitty implement WP32`

**Dependencies**: WP02

**Included Subtasks**:
- [ ] T169: Create `clarity-web/src/intent/templates/mod.rs`
- [ ] T170: Create `clarity-web/src/intent/templates/spec_templates.rs`
- [ ] T171: Implement generate_spec_template() function
- [ ] T172: Implement profile-specific templates
- [ ] T173: Implement fill_template() function

**Implementation Sketch**:
1. Define template structure for each profile
2. Fill in template with session data
3. Generate boilerplate content
4. Return complete spec template

**Parallel Opportunities**: Can run in parallel with WP31

---

## Summary

| Phase | WPs | Subtasks | Est. Lines |
|-------|-----|----------|------------|
| Setup | 1 | 4 | 280 |
| Foundation | 11 | 52 | ~4,740 |
| Parsing | 5 | 17 | ~1,860 |
| Planning | 4 | 17 | ~1,480 |
| Beads | 2 | 12 | ~800 |
| Quality | 4 | 23 | ~1,480 |
| Remaining | 2 | 9 | ~860 |
| **Total** | **33** | **179** | **~15,602** |

### Size Distribution

- Smallest WP: WP07 (~220 lines, 4 subtasks)
- Largest WP: WP15 (~520 lines, 7 subtasks)
- Average WP size: ~360 lines
- All WPs within ideal range (200-500 lines)

✓ **All work packages are properly sized** - no WP exceeds 700 lines

### Parallelization Opportunities

**Foundation Phase** (after WP01-WP02-WP04):
- WP05, WP06, WP07, WP08, WP09 can all run in parallel (5 concurrent)
- WP10, WP11 form a sequential chain
- WP13, WP14 can run in parallel after WP12

**Parsing Phase** (after WP02-WP03):
- WP17, WP18 can run in parallel
- WP20, WP21 can run in parallel

**Planning Phase**:
- WP22, WP23, WP24, WP25 have some parallel opportunities

### MVP Scope

**Minimum Viable Product**: WP01 through WP16
- Foundation + Interview Engine + Storage
- Enables basic interview workflow
- ~8,000 lines ported

### Next Steps

1. Begin with WP01 (Project Foundation)
2. Follow dependency chain for sequential WPs
3. Exploit parallelization where possible
4. Run `/spec-kitty.analyze` after WP16 for mid-course correction
