# Clarity Rust Planning CLI Architecture Spec

Status: Draft architecture brief. Not ready for bead decomposition until `docs/architecture/architecture-spec-review-findings.md` blockers are resolved.

Date: 2026-06-20

Target repository: `/home/lewis/src/clarity`

Target language: Rust only

Reference repository: `/home/lewis/src/intent-cli`

Reference status: Source material only. Do not build new runtime dependency on Gleam/Erlang.

## 1. Mission

Clarity is a frictionful Rust planning and interrogation CLI for Rust application work. It forces developers and agents through a full Double Diamond interview and a global five-lattice adversarial review before a plan can be considered complete.

The tool exists to prevent shallow specifications. It behaves like a combined SRE, software architect, Simon Wardley strategy reviewer, Charlie Munger inversion reviewer, security reviewer, and test strategist.

The tool shall produce two artifact layers:

1. A session-level `KirkContract16` that records the complete interrogated plan.
2. Task-level enhanced beads validated against the planner enhanced-bead CUE schema.

The tool shall stop normal interview work at `SpecComplete`. Creating `bd` issues is optional downstream emission and is never part of core completion.

## 2. Non-Goals

The system shall not target non-Rust applications.

The system shall not depend on Gleam, Erlang, or intent-cli at runtime.

The system shall not design around `br`. Existing `br` beads are migration/reference material only.

The system shall not treat JSONL as canonical state.

The system shall not silently skip hard gates.

The system shall not permit same-session concurrent writers.

The system shall not create `bd` issues during `clarity interview`.

The system shall not treat reviewer output as trustworthy unless it validates against schema and cites evidence.

The system shall not proceed to `SpecComplete` from degraded recovery unless missing raw evidence has been re-entered or regenerated.

## 3. Ubiquitous Language

`Session`: One planning/interview run for a single Rust work scope.

`Normal question`: A question asked during the main Double Diamond interview before the 100-question cap.

`Reviewer repair question`: A question required by one of the six adversarial reviewers after normal questioning freezes.

`Gate`: A deterministic or evidence-backed condition that must pass before `SpecComplete`.

`Hard gate`: A gate that cannot be skipped or downgraded.

`Event`: A canonical append-only record in Fjall. Events are the source of truth.

`Snapshot`: A derived checkpoint from event log prefix `1..=source_seq`.

`Projection`: A derived non-canonical representation, such as sanitized JSONL.

`KirkContract16`: Session-level 16-section contract generated from the interview and review evidence.

`Enhanced bead`: Task-level implementation packet conforming to `schemas/enhanced-bead.cue`.

`PME`: Product/Planning/Mental-model Engine modules currently orphaned under `clarity-web/src/pme/`; these become first-class domain inputs.

`AI effect`: A provider call that may be retried, repaired, failed, or marked ambiguous.

`Reviewer`: One of the six adversarial reviewers: SRE, Architect, Wardley, Munger, Security, Test.

`bd emission`: Optional creation of bead issues in `bd` after local artifacts are complete.

## 4. EARS Requirements

### Ubiquitous Requirements

THE SYSTEM SHALL target Rust application planning only.

THE SYSTEM SHALL generate both a session-level `KirkContract16` and task-level enhanced beads.

THE SYSTEM SHALL use Fjall as the canonical append-only event store.

THE SYSTEM SHALL write sanitized JSONL only as a derived projection after Fjall batch commit.

THE SYSTEM SHALL use CUE question banks from `schemas/questions.cue` plus project-local additive overrides.

THE SYSTEM SHALL reject project-local question overrides that weaken canonical gates.

THE SYSTEM SHALL support Rust-specific profiles: `rust-cli`, `rust-library`, `rust-web-service`, `rust-async-service`, `rust-storage`, `rust-ui`, and `rust-refactor`.

THE SYSTEM SHALL use OpenCode as the first AI provider through a swappable provider boundary.

THE SYSTEM SHALL use both deterministic heuristics and AI inference.

THE SYSTEM SHALL enforce all hard gates before `SpecComplete`.

THE SYSTEM SHALL validate enhanced beads against `schemas/enhanced-bead.cue`.

THE SYSTEM SHALL treat `bd` emission as optional downstream output.

THE SYSTEM SHALL store raw transcript data in Fjall and sanitized data in JSONL projection by default.

THE SYSTEM SHALL require explicit consent for raw export.

THE SYSTEM SHALL make every implementation bead Rust-specific and include tests-first, zero-panic, Moon-only, and verification-lane requirements.

### Event-Driven Requirements

WHEN a user starts an interview, THE SYSTEM SHALL acquire a session lock and append `InterviewStarted` before asking questions.

WHEN a user answers a question, THE SYSTEM SHALL append `UserAnswerRecorded` before AI extraction or gate evaluation.

WHEN an AI call is about to run, THE SYSTEM SHALL append `AiCallRequested` before sending the provider request.

WHEN an AI call succeeds, THE SYSTEM SHALL append `AiExtractionSucceeded`, `AiQuestionRecorded`, `AiReviewSucceeded`, or `AiSummarySucceeded` depending on operation.

WHEN an AI call fails after retries, THE SYSTEM SHALL append `AiCallFailed` and return an explicit error.

WHEN a prior AI request has no terminal event on resume, THE SYSTEM SHALL append `AiEffectAmbiguous` and retry under the same logical `effect_id`.

WHEN normal interview questions reach 100, THE SYSTEM SHALL freeze normal questioning and run the six-reviewer panel.

WHEN a reviewer fails, THE SYSTEM SHALL append `ReviewerFailed` and block `SpecComplete`.

WHEN reviewer-required repair questions reach 30 and gates still fail, THE SYSTEM SHALL return `InterviewExhausted`.

WHEN `KirkContract16` is compiled, THE SYSTEM SHALL include all sections 0 through 15.

WHEN JSONL projection write fails after Fjall batch commit, THE SYSTEM SHALL keep Fjall canonical and mark projection out of sync.

WHEN a same-session writer cannot acquire the lock, THE SYSTEM SHALL reject mutation with `SessionLockAlreadyHeld`.

WHEN `clarity beads emit` creates some `bd` issues and then fails, THE SYSTEM SHALL record partial emission and retry only missing beads on the next run.

### State-Driven Requirements

WHILE in `New`, THE SYSTEM SHALL only allow session start, session import, or abort.

WHILE in `Interviewing`, THE SYSTEM SHALL allow stdin answers, escape commands, AI extraction, and gate checks.

WHILE in `NormalQuestioningFrozen`, THE SYSTEM SHALL reject normal AI questions and only run adversarial review.

WHILE in `Reviewing`, THE SYSTEM SHALL collect reviewer outputs and fail closed on invalid reviewer output.

WHILE in `RepairQuestioning`, THE SYSTEM SHALL only ask reviewer-required repair questions.

WHILE in `SpecComplete`, THE SYSTEM SHALL reject further interview mutation unless a new revision/session is started.

WHILE in `RecoveredDegraded`, THE SYSTEM SHALL prevent `SpecComplete` until missing raw evidence is restored.

WHILE a session lock is held, THE SYSTEM SHALL require the owner token for every mutating command.

### Optional Requirements

WHERE `bd` is installed, THE SYSTEM MAY emit enhanced beads as `bd` issues through explicit user command.

WHERE `bd` is not installed, THE SYSTEM SHALL still complete local artifacts and report `BdUnavailable` only for emission commands.

WHERE a project supplies local question CUE, THE SYSTEM MAY add or strengthen questions and gates.

WHERE future AI providers are added, THE SYSTEM SHALL preserve the same provider trait semantics.

### Unwanted Requirements

IF a user invokes `/skip` for a hard gate, THE SYSTEM SHALL NOT skip the gate.

IF an AI reviewer returns invalid JSON, THE SYSTEM SHALL NOT treat it as pass.

IF JSONL projection differs from Fjall canonical state, THE SYSTEM SHALL NOT use JSONL for state transitions.

IF local CUE overrides weaken canonical gates, THE SYSTEM SHALL NOT start the session with those overrides.

IF a `bd` issue with the same `Clarity-Bead-Key` but different content exists, THE SYSTEM SHALL NOT reuse it silently.

IF a lock expires and another process acquires it, THE SYSTEM SHALL NOT allow the original owner token to mutate state.

IF Fjall state is rebuilt from sanitized JSONL, THE SYSTEM SHALL NOT allow `SpecComplete` without restoring missing raw evidence.

## 5. Product Scope

The product is a local CLI-first Rust planning tool. It is not a hosted service. It is not a multi-user collaborative system. It does not require a remote workflow orchestrator.

The initial storage, locking, and recovery model assumes one local machine and many possible local processes. Fjall has an exclusive one-process-per-database lock. The MVP serializes all mutating commands for a given Fjall database root. Same-session concurrent writes are rejected by the domain lock, and same-database concurrent processes are rejected by Fjall's file lock. Concurrent sessions are supported only when they use different database roots or a future single-owner daemon.

## 6. CLI Command Surface

### Primary Commands

`clarity interview start --profile <profile>` starts a new session.

`clarity interview resume <session_id>` resumes an existing session.

`clarity interview status <session_id>` prints state, gates, question counts, lock status, and artifact status.

`clarity interview export <session_id>` exports sanitized JSONL and artifacts.

`clarity interview export <session_id> --raw` exports raw transcript only after explicit consent event.

`clarity interview abort <session_id>` transitions the session to `Aborted`.

`clarity gates run <session_id>` re-runs deterministic gates from canonical state.

`clarity spec compile <session_id>` compiles or re-compiles `KirkContract16` and CUE spec.

`clarity beads generate <session_id>` generates enhanced beads locally.

`clarity beads emit <session_id>` emits generated beads to `bd`.

`clarity sessions list` lists sessions.

### Escape Commands During Interview

`/done` requests immediate gate evaluation. If gates fail, the tool explains missing evidence and continues.

`/skip` records `SkipAttempted`. The tool explains why the hard gate cannot be skipped and asks a narrower replacement question. After three skip attempts for the same gate, it escalates to review summary.

`/why` explains why the current question is required and which gates it supports.

`/status` prints gate and checkmark state.

`/resume <session_id>` resumes another session after releasing the current lock if safe.

`/abort` records abort intent and transitions to `Aborted` after confirmation.

`/export` writes sanitized export from current committed Fjall state.

## 7. Rust Profiles

The canonical profiles are Rust-specific:

`rust-cli`: CLI commands, stdout/stderr behavior, exit codes, config, shell completion, offline behavior.

`rust-library`: public API, type contracts, semver, feature flags, docs, examples, property tests.

`rust-web-service`: routes, auth, persistence, request validation, response schema, observability, graceful shutdown.

`rust-async-service`: task ownership, cancellation, backpressure, queue bounds, timeouts, shutdown drain/finalize, Loom where appropriate.

`rust-storage`: schemas, migrations, durability, snapshots, compaction, backup/recovery, corruption handling, fsync expectations.

`rust-ui`: Rust UI clients such as Dioxus/Makepad/Tauri where applicable, accessibility, state transitions, offline/cache behavior.

`rust-refactor`: existing-code transformation, invariants preservation, compatibility constraints, migration plan, tests and proof obligations.

Profile-specific questions are loaded from CUE and may be extended locally.

## 8. Session State Machine

### States

`New`: Session allocated but interview not started.

`Interviewing`: Normal Double Diamond questioning is active.

`NormalQuestioningFrozen`: Normal question budget exhausted or user requested review.

`Reviewing`: Six-reviewer panel is evaluating artifacts and evidence.

`RepairQuestioning`: Reviewer-required repair questions are being asked.

`InterviewExhausted`: Question and repair budget exhausted while hard gates still fail.

`SpecComplete`: All hard gates pass and required artifacts are generated/validated.

`Aborted`: User intentionally aborted the session.

`RecoveredDegraded`: State was rebuilt from sanitized projection or partial evidence and lacks raw transcript fidelity.

### Transition Rules

`New -> Interviewing`: allowed only after lock acquisition and `InterviewStarted` event.

`Interviewing -> NormalQuestioningFrozen`: occurs when normal questions reach 100 or user requests review through `/done` with enough evidence to run review.

`NormalQuestioningFrozen -> Reviewing`: system starts six-reviewer panel.

`Reviewing -> RepairQuestioning`: at least one reviewer fails and emits required repair questions, with repair budget remaining.

`Reviewing -> SpecComplete`: all reviewers pass and all hard gates pass.

`RepairQuestioning -> Reviewing`: repair answers recorded and reviewer-required gate rechecks are run.

`RepairQuestioning -> InterviewExhausted`: 30 reviewer-repair questions used and gates still fail.

`Interviewing -> SpecComplete`: allowed only if `/done` runs all gates and all six reviewers pass before question budget is exhausted.

`Any non-terminal -> Aborted`: allowed after user confirmation.

`RecoveredDegraded -> Interviewing`: allowed only after user accepts degraded recovery and raw evidence gaps are listed.

`RecoveredDegraded -> SpecComplete`: forbidden until missing raw evidence is re-entered or regenerated.

Terminal states are `SpecComplete`, `InterviewExhausted`, and `Aborted`.

## 9. Question Budget

Normal interview question budget: 100 questions.

Reviewer-repair question budget: 30 questions.

Normal questions include initial seed questions, AI dynamic questions, deterministic follow-up questions, and replacement questions after `/skip` before normal freeze.

Reviewer-repair questions include only questions emitted by one of the six reviewers after normal questioning freezes.

At 100 normal questions, normal AI questioning freezes. The system runs the reviewer panel.

If reviewer-required questions remain after 30 repair questions and hard gates still fail, the session transitions to `InterviewExhausted`.

## 10. Question Bank and CUE Overrides

Canonical question definitions live in `schemas/questions.cue`.

Project-local overrides may live in `.clarity/questions.cue`.

Merge semantics are additive and strengthening only:

1. New questions may be added.
2. Existing question wording may be extended only if the resulting question still covers the same canonical gate.
3. Existing question priority may be increased, not decreased.
4. Existing gates may be made stricter, not weaker.
5. Required security, reliability, concurrency, and verification questions may not be removed.
6. Reviewer categories may not be disabled.

If an override weakens canonical gates, session start fails with `QuestionOverrideWeakensGate`.

## 11. Double Diamond and Global Five Lattices

The tool runs the full Double Diamond first:

1. Discover: product thesis, problem, persona, non-persona, scenario, antithesis, VORP, straw-man traps, hole punching, CDI evidence.
2. Define: EARS extraction, gaps, constraints, NFR tradeoffs, initial KIRK contracts.
3. Develop: Rust-specific architecture, implementation slices, tests, verification lanes, enhanced bead generation.
4. Deliver: operational readiness, failure modes, bd emission readiness, handoff evidence.

After Double Diamond artifacts exist, the system runs the five global lattices:

1. EARS ambiguity elimination.
2. KIRK contract/domain modeling.
3. Inversion and exact error taxonomy.
4. Second-order consequence tracing.
5. Premortem and operational telemetry.

The five lattices evaluate the complete plan globally rather than repeating independently inside each Double Diamond phase.

## 12. PME Integration Contract

PME modules become first-class domain inputs. Current orphaned modules under `clarity-web/src/pme/` must be wired through stable domain adapters rather than exposed as ad hoc internals.

### Canonical PME Decisions

PME `VorpScore` replaces current `VorpValidation` as the canonical VORP score model.

PME `FailureCategory` enriches antithesis analysis. Antithesis remains exactly three user-visible points, but each point must map to one or more failure categories.

PME `HumanLimitation` composes with `StrawManTrap`. Straw-man validation rejects unrealistic personas; human-limitation modeling quantifies realistic frictions.

PME `CdiLogger` becomes standalone discovery evidence. CDI evidence supports but does not replace hard gates.

PME `NfrWizard` becomes the Define/Develop tradeoff gate for Rust reliability, latency, consistency, availability, scalability, maintainability, and security tradeoffs.

### PME Evidence Events

`PmeVorpScored`: records VORP dimensions, brutal-truth weights, threshold result, and evidence answer IDs.

`PmeFailureCategoriesMapped`: records three antithesis points and mapped failure categories.

`PmeHumanLimitationsModeled`: records persona limitations and severity.

`PmeCdiEvidenceRecorded`: records discovery signal type, signal strength, and source answer IDs.

`PmeNfrTradeoffsRecorded`: records selected tradeoffs, rejected tradeoffs, and rationale.

### PME Failure Behavior

Invalid PME outputs fail closed for the corresponding gate.

Missing PME evidence triggers reviewer-required questions before `SpecComplete`.

## 13. AI Provider Boundary

The provider trait must support these operations:

`ask_question(context) -> AiQuestion`

`extract_fields(input, schema) -> ExtractedFields`

`review_artifact(artifact, rubric) -> ReviewReport`

`repair_json(raw, schema) -> JsonRepairResult`

`summarize(context) -> Summary`

`health_check() -> ProviderStatus`

### AI Retry Rules

Before each provider call, append `AiCallRequested`.

Retry count: 3 attempts.

Backoff: exponential, bounded, and recorded.

On valid success, append operation-specific success event.

On invalid JSON, attempt `repair_json` within the same retry budget unless the operation has already exhausted attempts.

On final failure, append `AiCallFailed` and return explicit error.

On resume with request but no terminal result, append `AiEffectAmbiguous`, retry as a new attempt under the same logical `effect_id`, and preserve all attempts as evidence.

AI calls are at-least-once effects. The system must not assume provider determinism.

## 14. Reviewer Panel

The six reviewers are mandatory:

1. SRE reviewer: reliability, recovery, operational blast radius.
2. Architect reviewer: domain model, invariants, boundaries, type contracts.
3. Wardley reviewer: value chain, strategic position, build/buy/commodity assumptions.
4. Munger inversion reviewer: incentives, failure modes, second-order effects.
5. Security reviewer: abuse, secrets, data leakage, injection, malicious input.
6. Test reviewer: ATDD, E2E, property tests, mutation resistance, deterministic assertions.

Reviewer output schema:

```json
{
  "reviewer": "sre|architect|wardley|munger|security|test",
  "verdict": "Pass|Fail",
  "blocking_findings": [],
  "warnings": [],
  "required_questions": [],
  "required_gate_rechecks": [],
  "confidence": 0.0,
  "evidence_event_ids": []
}
```

Reviewer confidence threshold: 0.75.

If `confidence < 0.75`, the reviewer fails.

If reviewer output is invalid JSON or schema-invalid, the reviewer fails.

If reviewer output lacks evidence event IDs, the reviewer fails.

All six reviewers must pass before `SpecComplete`.

Reviewers may use the same OpenCode provider for MVP, but prompt isolation, schema isolation, and evidence citation are mandatory.

## 15. Hard Gates

All hard gates are required for `SpecComplete`.

### Required Fields Gate

All profile-required fields must be present and non-empty after normalization.

Profile-required fields are defined by canonical question CUE.

### VORP Gate

Each VORP dimension must be at least 0.70.

Average VORP score must be at least 0.75.

Canonical dimensions: Value, Obvious, Real, Possible.

PME `VorpScore` is the canonical scoring type.

### Straw-Man Gate

Detected traps must be zero.

Canonical traps: IrrationalActor, ManicPixieDreamUser, StoicMonk, YourClone.

Warnings may be recorded, but any detected trap blocks `SpecComplete`.

### Antithesis Gate

Exactly three antithesis points are required.

Each point specificity must be at least 0.30.

Average specificity must be at least 0.60.

Each point must map to at least one PME failure category.

### Hole Punching Gate

All three holes must be present and non-empty:

1. Discovery hole.
2. Edge-case hole.
3. Motivation drop-off hole.

### EARS Gate

At least one ubiquitous requirement is required.

At least one event-driven requirement is required.

At least one unwanted requirement is required.

State-driven, optional, and complex requirements are encouraged but not hard-required for MVP unless the selected profile CUE makes them required.

### KIRK16 Gate

KIRK sections 0 through 15 must be valid.

Sections 0 through 14 are content/evidence sections.

Section 15 is required derived metadata.

### CUE Spec Gate

The generated CUE spec must validate against `schemas/intent.cue` and `schemas/kirk.cue`.

### Enhanced Bead Gate

Every generated enhanced bead must validate against `schemas/enhanced-bead.cue`.

## 16. KirkContract16 Schema

`KirkContract16` is the session-level artifact. It contains exactly 16 sections.

0. Original Prompt: raw initial user prompt and profile.
1. Problem Statement: clarified Rust work problem.
2. Antithesis Points: exactly three pressure-test points plus PME failure categories.
3. Target Persona: primary user/developer/operator persona.
4. Straw Man Validation: trap results, zero-trap proof, human limitations.
5. Solution Description: proposed Rust solution and bounded scope.
6. VORP Justification: PME VORP scores, brutal-truth weights, evidence.
7. Non-Persona: explicitly excluded users/use cases.
8. Scenario Trigger: when the target scenario starts.
9. Scenario Value Moment: when value is realized.
10. Scenario Feeling: user/operator/developer state and friction.
11. Discovery Hole: why discovery may fail.
12. Edge Case Hole: boundary and adversarial use cases.
13. Motivation Drop-off: why user/developer/operator stops using or maintaining it.
14. EARS Requirements: extracted and validated EARS requirements.
15. Compilation Metadata: schema version/hash, session id, event log last seq/hash, gate results, AI provider/model, reviewer panel results, generated_at.

Section 15 is derived by the system. It is not user-authored, but it is required.

## 17. CUE Artifacts

`schemas/intent.cue` is the canonical base spec schema.

`schemas/kirk.cue` is the canonical KIRK extension schema.

`schemas/questions.cue` is the canonical question-bank schema.

`schemas/enhanced-bead.cue` is the canonical enhanced bead schema.

Every artifact must record schema version and schema hash used for validation.

Schema hash canonicalization: SHA-256 over canonical UTF-8 schema bytes after normal line-ending normalization to LF.

Artifact payload hash canonicalization: SHA-256 over canonical JSON serialization with sorted object keys.

## 18. Storage Architecture

### Canonical Store

Fjall is canonical.

The Fjall database stores these keyspaces:

`events`: append-only event records keyed by `[session_id_hash_16 | seq_be_u64]`.

`snapshots`: derived snapshots keyed by `[session_id_hash_16 | source_seq_be_u64]`.

`locks`: current session locks keyed by `[session_id_hash_16]`.

`artifacts`: generated artifacts keyed by `[session_id_hash_16 | artifact_kind | artifact_id]`.

`gate_results`: latest gate results keyed by `[session_id_hash_16 | gate_id]`.

`projection_status`: JSONL projection status keyed by `[session_id_hash_16]`.

`bd_mappings`: local bead to bd issue mappings keyed by `[session_id_hash_16 | local_bead_id]`.

All numeric key components use big-endian byte order so lexical key order matches numeric order.

Every mutating command uses one Fjall `OwnedWriteBatch` for event append plus all same-transaction index updates. Critical state-changing batches call `PersistMode::SyncAll` until durability tuning proves a weaker mode is acceptable.

Read-only status and export commands use Fjall point-in-time snapshots where possible.

### Event Schema

Every event has:

```json
{
  "session_id": "string",
  "seq": 1,
  "event_id": "uuid",
  "event_type": "string",
  "payload": {},
  "created_at": "rfc3339",
  "idempotency_key": "string",
  "schema_version": "string",
  "actor": "User|AiProvider|Reviewer|System|Bd"
}
```

### Core Invariants

For a given `session_id`, committed event sequence numbers are contiguous from 1.

`event_id` is globally unique.

`(session_id, idempotency_key)` maps to exactly one semantic effect outcome.

Snapshots are derived from event prefix `1..=source_seq` and store event-prefix hash.

JSONL projections are rebuildable from Fjall and never canonical.

## 19. Event Taxonomy

### Session Events

`InterviewStarted`

`UserAnswerRecorded`

`SkipAttempted`

`PhaseCompleted`

`NormalQuestioningFrozen`

`InterviewExhausted`

`SpecCompleted`

`InterviewAborted`

`RecoveredDegraded`

### Lock Events

`SessionLockAcquired`

`SessionLockRefreshed`

`SessionLockReleased`

`SessionLockStolenAfterExpiry`

`SessionLockRejected`

### AI Events

`AiCallRequested`

`AiQuestionRecorded`

`AiExtractionSucceeded`

`AiReviewSucceeded`

`AiSummarySucceeded`

`AiJsonRepairSucceeded`

`AiJsonRepairFailed`

`AiCallFailed`

`AiEffectAmbiguous`

### Gate Events

`GatePassed`

`GateFailed`

`GateRecheckRequested`

### PME Events

`PmeVorpScored`

`PmeFailureCategoriesMapped`

`PmeHumanLimitationsModeled`

`PmeCdiEvidenceRecorded`

`PmeNfrTradeoffsRecorded`

### Reviewer Events

`ReviewerPanelStarted`

`ReviewerOutputRecorded`

`ReviewerPassed`

`ReviewerFailed`

`ReviewerOutputInvalid`

`ReviewerRepairQuestionRecorded`

### Artifact Events

`KirkCompiled`

`CueSpecCompiled`

`CueSpecValidated`

`EnhancedBeadsGenerated`

`EnhancedBeadsValidated`

### Projection and Export Events

`JsonlProjectionWritten`

`JsonlProjectionFailed`

`RawExportRequested`

`RawExportWritten`

`SanitizedExportWritten`

### bd Events

`BdEmitStarted`

`BdBeadCreated`

`BdBeadSkippedExisting`

`BdEmitPartial`

`BdEmitCompleted`

`BdEmitFailed`

## 20. Snapshot and Projection Rules

Snapshots are performance optimizations, not authority.

Each snapshot stores `source_seq`, `event_prefix_hash`, `schema_version`, and serialized state.

On load, if snapshot hash mismatches event log prefix, discard snapshot and replay events.

JSONL projection is written after Fjall batch commit. Projection failure never rolls back canonical events.

If projection is out of sync, `clarity interview status` reports it and offers projection rebuild.

Disaster recovery from sanitized JSONL creates `RecoveredDegraded` state with `raw_transcript_missing: true`.

## 21. Lock Protocol

Lock fields:

`session_id`

`owner_token: Uuid`

`owner_pid`

`hostname`

`acquired_at`

`expires_at`

Default TTL: 15 minutes.

Mutating commands must acquire or refresh the domain lock in the same Fjall write batch that appends events.

PID and hostname are diagnostic only. Authority is `owner_token`.

If the original owner resumes after expiry and another process holds a valid lock, the original owner receives `SessionLockOwnerMismatch` or `SessionLockAlreadyHeld` and cannot mutate state.

Stale lock stealing requires atomic compare-and-set: old lock must still be expired at commit time.

## 22. Security and Privacy

Fjall raw transcript data lives under XDG private data directory with `0700` directory permissions and `0600` file permissions for MVP.

Encryption at rest is deferred to a hardening bead using OS keyring.

Provider API keys are never written to events, artifacts, projections, or logs.

Before sanitized JSONL export, the system redacts obvious secrets:

1. API keys and tokens with common prefixes.
2. AWS access key patterns.
3. GitHub tokens.
4. Bearer tokens.
5. Private key blocks.
6. Connection strings with embedded credentials.
7. Environment assignments matching secret-like names.

Sanitized JSONL may retain non-secret product content.

Raw export requires `--raw` and records `RawExportRequested` plus explicit user confirmation.

Prompt-injection content from user answers is always treated as data, never instructions for reviewer/system prompts.

## 23. bd Emission

`clarity interview` never emits bd issues.

`clarity beads emit <session_id>` emits generated, locally validated enhanced beads.

Local bead identity:

`local_bead_id = sha256(session_id + canonical_task_id)`

The bd description includes:

`Clarity-Bead-Key: <local_bead_id>`

Emission behavior:

1. Scan existing bd issues for the key.
2. If exactly one matching issue has matching content hash, record mapping and skip create.
3. If matching key has different content, fail with `BdKeyCollision`.
4. If multiple matches exist, fail with `BdExistingIssueAmbiguous`.
5. If no match exists, call `bd create` with title, type, priority, and full enhanced bead description.
6. Record `local_bead_id -> bd_id` after successful create.
7. On partial failure, record `BdEmitPartial`; retry emits only missing beads.

No rollback is attempted for successfully created bd issues.

## 24. Error Taxonomy

The Rust error enum should include at least these variants.

### Session and State Errors

`CommandNotAllowedInState { command, state }`

`InterviewExhausted { failed_gates, missing_answers, reviewer_findings }`

`QuestionBudgetExceeded { session_id, max_questions, attempted_question_kind }`

`SkipLimitExceeded { session_id, gate, attempts }`

`StdinClosed { session_id, state }`

`InputTooLarge { bytes, max_bytes }`

### Lock Errors

`SessionLockAlreadyHeld { session_id, holder, expires_at }`

`SessionLockOwnerMismatch { session_id, attempted_owner_token }`

`SessionLockExpiredDuringCommand { session_id, owner_token, expired_at }`

`SessionLockRefreshFailed { session_id, owner_token, reason }`

`ClockMovedBackwards { previous_observed_at, observed_at }`

### Event and Storage Errors

`EventSeqConflict { session_id, expected_next_seq, actual_next_seq }`

`EventIdCollision { event_id }`

`IdempotencyKeyCollision { session_id, idempotency_key, existing_event_id, attempted_event_type }`

`FjallCommitFailed { session_id, attempted_seq, source }`

`SnapshotHashMismatch { session_id, snapshot_seq, expected_hash, actual_hash }`

`ProjectionWriteFailed { session_id, committed_through_seq, projection_path, source }`

`ProjectionOutOfSync { session_id, fjall_last_seq, jsonl_last_seq }`

### AI Errors

`AiProviderUnavailable { provider, operation }`

`AiCallFailed { provider, operation, effect_id, attempts, last_error }`

`AiEffectAmbiguous { effect_id, request_event_id }`

`AiResponseTooLarge { effect_id, bytes, max_bytes }`

`AiResponseSchemaInvalid { effect_id, schema_hash, validation_errors }`

`AiRepairFailed { effect_id, repair_attempts, last_error }`

### Gate and Artifact Errors

`SchemaValidationFailed { artifact_kind, schema_version, schema_hash, errors }`

`UnsupportedSchemaVersion { artifact_kind, found, supported }`

`ArtifactGenerationIncomplete { artifact_kind, missing_sections }`

`GateScoreBelowThreshold { gate, score, threshold, evidence_event_ids }`

`QuestionOverrideWeakensGate { question_id, gate_id, reason }`

### Reviewer Errors

`ReviewerOutputInvalid { reviewer, validation_errors, raw_output_event_id }`

`ReviewerFailed { reviewer, blocking_findings }`

`ReviewerConfidenceTooLow { reviewer, confidence, threshold }`

### Security and Export Errors

`RawExportRequiresExplicitConsent { session_id }`

`RedactionPolicyViolation { export_event_id, detector, match_count }`

`SecretDetectedInProviderRequest { detector, action }`

### bd Errors

`BdUnavailable { command }`

`BdEmissionPartialFailure { session_id, succeeded_count, failed_count }`

`BdKeyCollision { bead_key, existing_bd_id, local_bead_id }`

`BdExistingIssueAmbiguous { bead_key, matching_issue_ids }`

`LocalBeadMappingConflict { local_bead_id, existing_bd_id, attempted_bd_id }`

`BdRateLimited { retry_after }`

## 25. Rust-Specific Enhanced Bead Defaults

Every generated bead must include Rust-specific instructions:

1. Write tests first.
2. Use Moon commands only for build/test/lint.
3. Do not use `unwrap`, `expect`, or `panic` in production code.
4. Do not introduce unsafe code unless the bead explicitly approves it and proof/review lanes cover it.
5. Prefer typed domain models and make illegal states unrepresentable.
6. Define preconditions, postconditions, and invariants.
7. Include happy-path, error-path, and edge-case ATDD tests.
8. Consider Verus, Kani, Flux, Loom, Miri, proptest, and fuzzing where risk profile justifies them.
9. Include anti-hallucination read-before-write rules.
10. Include context survival and recovery instructions.

## 26. SpecComplete Acceptance Criteria

A session reaches `SpecComplete` only when all are true:

1. State is not `RecoveredDegraded` unless raw evidence has been restored.
2. Required fields gate passes.
3. VORP gate passes.
4. Straw-man gate passes.
5. Antithesis gate passes.
6. Hole punching gate passes.
7. EARS gate passes.
8. KIRK16 gate passes for sections 0 through 15.
9. CUE spec validates against `intent.cue` and `kirk.cue`.
10. Enhanced beads are generated and validate against `enhanced-bead.cue`.
11. All six reviewers pass with confidence at least 0.75.
12. Latest snapshot hash matches event log prefix.
13. JSONL projection is either current or explicitly marked out-of-sync without affecting canonical state.
14. No unresolved hard-gate skip attempts remain.

## 27. InterviewExhausted Acceptance Criteria

A session reaches `InterviewExhausted` when all are true:

1. 100 normal questions have been used.
2. Six-reviewer panel has run.
3. 30 reviewer-repair questions have been used or no further useful reviewer-required questions remain.
4. At least one hard gate still fails.
5. The session records failed gates, missing answers, reviewer findings, and recommended next actions.

`InterviewExhausted` is terminal for that session revision. A future session/revision may be started using exported evidence.

## 28. Implementation Architecture Boundaries

Use functional-core, imperative-shell boundaries.

Data:

Session IDs, event IDs, lock owner tokens, gate IDs, artifact IDs, local bead IDs, schema versions, and hashes should be typed newtypes.

Calculations:

Gate evaluation, event reduction, snapshot hash calculation, CUE override validation, PME scoring adaptation, and reviewer output validation should be pure functions.

Actions:

stdin reads, Fjall write batches, JSONL writes, OpenCode calls, CUE command execution, and bd command execution are shell actions.

Actions must append intent/request events before external side effects where replay ambiguity matters.

## 29. Porting Decisions From intent-cli

Use intent-cli as source material for:

1. Richer `Spec`, `Feature`, `Behavior`, `Verification`, `Invariant`, `AntiPattern`, `AIHints`, `ImplementationHints`, `EntityHint`, and `SecurityHints` model.
2. CUE question bank and custom override model.
3. Enhanced bead template concepts where not already present in `schemas/enhanced-bead.cue`.
4. Session diffing concepts where useful.
5. Shell completion concepts, later through clap completion generation.

Do not port runtime dependencies on Gleam/Erlang.

Do not preserve `br` as a core integration.

## 30. Deferred Work

Encryption at rest is deferred.

Heterogeneous reviewer providers are deferred.

Remote/distributed orchestration is deferred.

Web UI is deferred.

Multi-user collaborative editing is deferred.

Automatic rollback of bd issue creation is not supported.

## 31. Readiness For Bead Decomposition

This spec is ready for `arch-spec-to-beads` only after a reviewer confirms the following are present:

1. Exact state machine.
2. Exact event schema and taxonomy.
3. Exact gate thresholds.
4. Exact artifact schemas and section lists.
5. Exact lock protocol.
6. Exact storage authority and projection rules.
7. Exact reviewer contract.
8. Exact bd idempotency and partial failure behavior.
9. Exact Rust-only scope and profiles.
10. Exact security/export policy.

This document currently satisfies those requirements at the architecture level. Implementation beads must still be decomposed into atomic Rust tasks with CUE-validated enhanced bead templates.
