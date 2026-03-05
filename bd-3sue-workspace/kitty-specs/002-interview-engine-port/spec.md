# Feature: Interview Engine Port

**Version**: 1.0.0
**Status**: Draft
**Mission**: software-dev
**Created**: 2026-02-27

---

## Overview

Port the intent-cli interview engine from Gleam to Rust, creating a new `clarity-web/src/intent/` module. This enables single-binary distribution by removing the Gleam/Erlang runtime dependency from Clarity Progressive Discover.

### Problem Statement

The current Clarity Progressive Discover workflow depends on intent-cli, which is written in Gleam and requires the Erlang runtime. This creates deployment complexity and prevents true single-binary distribution. Users must have both the Rust-based clarity-web binary AND the Gleam/Erlang toolchain installed.

### Proposed Solution

Port the core interview engine (~3,200 lines of Gleam logic) to idiomatic Rust, maintaining API compatibility while leveraging Rust's type system for additional safety. The ported code will live in the `clarity-web/src/intent/` module (named "intent" to match the original intent-cli naming and to encompass broader functionality including planning, beads, and quality analysis).

---

## Actors

### Primary Actors

- **Developer**: Using the interview engine to capture requirements through structured interrogation
- **System**: Running interview sessions, detecting gaps and conflicts, generating work items

### Secondary Actors

- **AI Assistant**: Consuming interview sessions to generate specifications and implementation plans
- **Quality Analyzer**: Evaluating spec completeness and AI readiness

---

## User Scenarios

### Scenario 1: Create and Run Interview Session

**Given** a developer wants to capture requirements for an API feature
**When** they create a new interview session with profile type "Api"
**Then** the system initializes a session in Discovery stage with empty answers, gaps, and conflicts

**Flow**:
1. Developer specifies profile type (Api, Cli, Event, Data, Workflow, or UI)
2. System creates `InterviewSession` with unique ID, timestamp, and initial state
3. System returns session ready for first question

### Scenario 2: Answer Questions and Detect Gaps

**Given** an active interview session
**When** the developer answers a critical question with insufficient detail (< 10 characters)
**Then** the system detects a blocking gap and prevents progression

**Flow**:
1. Developer receives question based on profile and round
2. Developer provides response
3. System extracts structured fields from response
4. System calculates confidence score
5. System detects blocking gaps for brief answers to critical questions
6. System returns updated session with any detected gaps

### Scenario 3: Detect Conflicts Between Answers

**Given** an interview session with multiple answers
**When** the developer provides answers that contain conflicting requirements (e.g., "fast" and "strongly consistent")
**Then** the system detects CAP theorem conflicts and presents resolution options

**Flow**:
1. Developer answers questions from different perspectives (Developer, Ops, Security)
2. System analyzes answer pairs for known conflict patterns
3. System creates Conflict records with resolution options
4. Developer chooses resolution option
5. System records chosen resolution

### Scenario 4: Complete Interview and Generate Beads

**Given** a completed interview session (stage = Complete)
**When** the developer requests work item generation
**Then** the system generates BeadRecord objects suitable for issue tracking

**Flow**:
1. System checks session is in Complete stage
2. System filters answers by profile-specific keywords
3. System generates BeadRecord for each relevant answer
4. System outputs beads as JSONL and/or enhanced CUE format

### Scenario 5: Persist and Resume Session

**Given** an in-progress interview session
**When** the developer saves and later resumes the session
**Then** the session state is preserved in JSONL format and can be diffed against previous versions

**Flow**:
1. System serializes session to JSONL line
2. System appends to `.interview/sessions.jsonl`
3. On resume, system deserializes session from JSONL
4. System provides session diff showing changes since last save

---

## Functional Requirements

### FR-1: Core Types Module

**Priority**: Critical

The system SHALL provide a `clarity-web/src/intent/types.rs` module containing:

- `Profile` enum with variants: `Api`, `Cli`, `Event`, `Data`, `Workflow`, `Ui`
- `InterviewStage` enum with variants: `Discovery`, `Refinement`, `Validation`, `Complete`, `Paused`
- `Perspective` enum with variants: `User`, `Developer`, `Ops`, `Security`, `Business`
- `QuestionPriority` enum with variants: `Critical`, `Important`, `NiceToHave`
- `QuestionCategory` enum with variants: `HappyPath`, `ErrorCase`, `EdgeCase`, `Constraint`, `Dependency`, `NonFunctional`
- `Answer` struct with fields: `question_id`, `question_text`, `perspective`, `round`, `response`, `extracted`, `confidence`, `notes`, `timestamp`
- `Gap` struct with fields: `id`, `field`, `description`, `blocking`, `suggested_default`, `why_needed`, `round`, `resolved`, `resolution`
- `Conflict` struct with fields: `id`, `between`, `description`, `impact`, `options`, `chosen`
- `ConflictResolution` struct with fields: `option`, `description`, `tradeoffs`, `recommendation`
- `InterviewSession` struct with fields: `id`, `profile`, `created_at`, `updated_at`, `completed_at`, `stage`, `rounds_completed`, `answers`, `gaps`, `conflicts`, `raw_notes`, `current_phase`, `completed_phases`
- `Question` struct with fields: `id`, `round`, `perspective`, `category`, `priority`, `question`, `context`, `example`, `expected_type`, `extract_into`, `depends_on`, `blocks`

All types SHALL derive `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`.

### FR-2: Interview Engine Module

**Priority**: Critical

The system SHALL provide a `clarity-web/src/intent/engine.rs` module implementing:

- `InterviewSession::new(id: String, profile: Profile, timestamp: String) -> Self`
- `InterviewSession::add_answer(&mut self, answer: Answer) -> &mut Self`
- `InterviewSession::detect_gaps(&self) -> Vec<Gap>`
- `InterviewSession::detect_conflicts(&self) -> Vec<Conflict>`
- `InterviewSession::check_for_gaps(&mut self, question: &Question, answer: &Answer) -> Vec<Gap>`
- `InterviewSession::check_for_conflicts(&mut self, new_answer: &Answer) -> Vec<Conflict>`
- `InterviewSession::complete_round(&mut self) -> &mut Self`
- `InterviewSession::get_current_round(&self) -> u32`
- `InterviewSession::can_proceed(&self) -> Result<(), String>`
- `InterviewSession::resolve_conflict(&mut self, conflict_id: &str, chosen_option: i32) -> Result<(), String>`
- `InterviewSession::resolve_gap(&mut self, gap_id: &str, resolution: &str) -> &mut Self`
- `InterviewSession::get_blocking_gaps(&self) -> Vec<&Gap>`
- `InterviewSession::get_unresolved_conflicts(&self) -> Vec<&Conflict>`
- `InterviewSession::complete_phase(&mut self, phase_number: u32) -> &mut Self`
- `InterviewSession::can_execute_phase(&self, phase_number: u32) -> bool`
- `create_session(id: String, profile: Profile, timestamp: String) -> InterviewSession`
- `profile_to_string(profile: Profile) -> String`
- `string_to_profile(s: &str) -> Result<Profile, String>`
- `stage_to_string(stage: InterviewStage) -> String`
- `format_progress(session: &InterviewSession) -> String`

### FR-3: Field Extraction

**Priority**: High

The system SHALL provide field extraction from answer text:

- `extract_from_answer(question_id: &str, response: &str, extract_fields: &[String]) -> HashMap<String, String>`
- Support extraction patterns for: `auth_method`, `entities`, `audience`, and generic fields
- Pattern matching for auth methods: jwt, oauth, session, api_key, none
- Pattern matching for audience types: mobile, web, api, cli, internal
- Entity extraction via capitalized word detection

### FR-4: Gap Detection

**Priority**: Critical

The system SHALL detect gaps based on profile-specific required fields:

- Api profile: base_url, auth_method, happy_path, error_cases, response_format
- Cli profile: command_name, happy_path, help_text, exit_codes
- Event profile: event_type, payload_schema, trigger
- Data profile: data_model, access_patterns, retention
- Workflow profile: steps, happy_path, error_recovery
- UI profile: user_flows, happy_path, states

Critical questions answered with < 10 characters SHALL generate blocking gaps.

### FR-5: Conflict Detection

**Priority**: High

The system SHALL detect conflicts between answers:

- CAP theorem conflict: "fast"/"latency" + "consistent"/"accurate"
- Anonymous + audit conflict: "anonymous" + "audit"/"log"
- Developer + Ops perspective conflicts on consistency vs. latency

Each conflict SHALL include at least 2 resolution options with tradeoffs.

### FR-6: Bead Templates Module

**Priority**: High

The system SHALL provide a `clarity-web/src/intent/bead_templates.rs` module implementing:

- `BeadRecord` struct with fields: `title`, `description`, `profile_type`, `priority`, `issue_type`, `labels`, `ai_hints`, `acceptance_criteria`, `dependencies`
- `BeadStats` struct with fields: `total`, `by_type`, `by_priority`
- `generate_beads_from_session(session: &InterviewSession) -> Vec<BeadRecord>`
- Profile-specific bead generators: `generate_api_beads`, `generate_cli_beads`, `generate_event_beads`, `generate_data_beads`, `generate_workflow_beads`, `generate_ui_beads`
- `bead_to_jsonl_line(bead: &BeadRecord) -> String`
- `beads_to_jsonl(beads: &[BeadRecord]) -> String`
- `beads_to_enhanced_cue(beads: &[BeadRecord]) -> String`
- `filter_beads_by_type(beads: &[BeadRecord], issue_type: &str) -> Vec<&BeadRecord>`
- `sort_beads_by_priority(beads: &mut [BeadRecord])`
- `add_dependency(beads: &mut [BeadRecord], from_title: &str, to_title: &str)`
- `bead_stats(beads: &[BeadRecord]) -> BeadStats`

### FR-7: Quality Analyzer Module

**Priority**: High

The system SHALL provide a `clarity-web/src/intent/quality_analyzer.rs` module implementing:

- `QualityReport` struct with fields: `coverage_score`, `clarity_score`, `testability_score`, `ai_readiness_score`, `overall_score`, `issues`, `suggestions`
- `QualityIssue` enum with variants: `MissingErrorTests`, `MissingAuthenticationTest`, `MissingEdgeCases`, `VagueRules`, `NoExamples`, `MissingExplanations`, `UntestedInvariants`, `MissingAiHints`, `MissingPreconditions`, `MissingPostconditions`
- `analyze_spec(spec: &Spec) -> QualityReport`
- Score calculations (0-100 scale):
  - Coverage: base 50 + error behavior bonus + auth test bonus + edge case bonus + invariant bonus
  - Clarity: base 60 + intent ratio + notes ratio - vague verification penalty
  - Testability: base 70 + dependencies bonus + preconditions bonus + postconditions bonus + examples bonus
  - AI Readiness: base 50 + AI hints bonus + verification ratio + examples bonus
- `format_report(report: &QualityReport) -> String`

### FR-8: Storage Module

**Priority**: High

The system SHALL provide a `clarity-web/src/intent/storage.rs` module implementing:

- `SessionRecord` struct for simplified storage representation
- `AnswerVersion` struct for historical answer tracking
- `AnswerWithHistory` struct combining current answer with versions
- `SessionSnapshot` struct for diff comparison
- `SessionDiff` struct for comparing sessions
- `AnswerDiff` struct for single answer changes
- `AnswerChangeType` enum: `Added`, `Modified`, `Removed`
- `session_to_jsonl_line(session: &InterviewSession) -> String`
- `append_session_to_jsonl(session: &InterviewSession, jsonl_path: &Path) -> Result<(), String>`
- `list_sessions_from_jsonl(jsonl_path: &Path) -> Result<Vec<InterviewSession>, String>`
- `get_session_from_jsonl(jsonl_path: &Path, session_id: &str) -> Result<InterviewSession, String>`
- `create_snapshot(session: &InterviewSession, description: &str) -> SessionSnapshot`
- `diff_sessions(from: &InterviewSession, to: &InterviewSession) -> SessionDiff`
- `format_diff(diff: &SessionDiff) -> String`
- `append_to_history(session: &InterviewSession, description: &str, history_path: &Path) -> Result<(), String>`
- `list_session_history(history_path: &Path, session_id: &str) -> Result<Vec<SessionSnapshot>, String>`

### FR-9: Module Organization

**Priority**: Critical

The system SHALL organize code in `clarity-web/src/intent/` with:

- `mod.rs` - Public API exports and module documentation
- `types.rs` - Core type definitions
- `engine.rs` - Interview session logic
- `bead_templates.rs` - Bead generation
- `quality_analyzer.rs` - Quality scoring
- `storage.rs` - JSONL persistence

The `mod.rs` SHALL re-export all public types and functions.

### FR-10: Error Handling

**Priority**: High

The system SHALL use `Result<T, E>` throughout with zero `unwrap()` or `expect()` calls:

- Define `InterviewError` enum covering: `JsonError`, `IoError`, `ParseError`, `ValidationError`, `NotFoundError`, `ConflictError`
- All fallible operations return `Result<T, InterviewError>`
- Error messages SHALL be user-friendly and actionable

---

## Non-Functional Requirements

### NFR-1: Type Safety

All types SHALL leverage Rust's type system for compile-time safety:
- Use `Option<T>` for nullable fields
- Use `Result<T, E>` for fallible operations
- Use newtypes for IDs (e.g., `SessionId(String)`) to prevent mixing

### NFR-2: Performance

- JSONL serialization/deserialization SHALL complete in < 10ms for sessions with < 100 answers
- Gap detection SHALL complete in < 5ms for sessions with < 50 answers
- Conflict detection SHALL complete in < 10ms for sessions with < 50 answers

### NFR-3: Compatibility

- JSONL format SHALL be compatible with existing intent-cli JSONL files
- Migration tooling SHALL NOT be required for existing `.interview/` directories

---

## Key Entities

### InterviewSession

The central entity representing a structured interview:

| Field | Type | Description |
|-------|------|-------------|
| id | String | Unique session identifier |
| profile | Profile | Type of system being specified |
| stage | InterviewStage | Current state machine state |
| answers | Vec<Answer> | Collected answers |
| gaps | Vec<Gap> | Missing information |
| conflicts | Vec<Conflict> | Detected contradictions |

### BeadRecord

A work item generated from interview answers:

| Field | Type | Description |
|-------|------|-------------|
| title | String | Work item title |
| description | String | Detailed description |
| profile_type | String | Source profile |
| priority | u8 | 0-4 priority level |
| issue_type | String | Type classification |
| labels | Vec<String> | Tags for categorization |
| acceptance_criteria | Vec<String> | Definition of done |

### QualityReport

Quality metrics for a specification:

| Field | Type | Description |
|-------|------|-------------|
| coverage_score | u8 | 0-100 coverage metric |
| clarity_score | u8 | 0-100 clarity metric |
| testability_score | u8 | 0-100 testability metric |
| ai_readiness_score | u8 | 0-100 AI readiness metric |
| issues | Vec<QualityIssue> | Detected problems |
| suggestions | Vec<String> | Improvement recommendations |

---

## Assumptions

1. **Serde JSON** will be used for serialization (already in clarity-web dependencies)
2. **HashMap** will be used for dictionary types (standard library)
3. **Chrono** will be used for timestamps if needed (already in clarity-web dependencies)
4. Session IDs are generated externally (by caller)
5. Questions are loaded from external source (not part of this port)
6. Phase gating integration with `plan_mode` types will be handled separately

---

## Out of Scope

- Question loading from YAML/JSON files (separate concern)
- SQLite database operations (JSONL-only for this port)
- AI-driven field extraction enhancement (future work)
- UI components for interview flow (separate feature)
- OpenCode server integration (separate feature)

---

## Success Criteria

1. **API Parity**: All public functions from intent-cli interview module have Rust equivalents
2. **Test Coverage**: > 80% line coverage on all interview modules
3. **Compatibility**: Existing `.interview/sessions.jsonl` files load correctly
4. **No Panics**: Zero `unwrap()`, `expect()`, or panic paths in production code
5. **Documentation**: All public types and functions have rustdoc comments
6. **Binary Size**: Ported code adds < 500KB to release binary size

---

## Dependencies

### Internal Dependencies

- `clarity-web` crate (for integration)
- `serde`, `serde_json` (for serialization)
- `chrono` (for timestamps, optional)

### External Dependencies

- None (removes Gleam/Erlang runtime dependency)

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| JSONL format incompatibility | High | Port decoder tests first, validate against existing files |
| Missing edge cases in port | Medium | Port unit tests alongside code, run both implementations in parallel during transition |
| Performance regression | Low | Benchmark critical paths, use Rust idioms (iterators, zero-copy) |
| Type mapping errors | Medium | Use property-based testing to validate round-trip serialization |
