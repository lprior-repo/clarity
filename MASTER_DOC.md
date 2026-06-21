# Master Doc: Clarity Rust Planning CLI PRD + Architecture Spec

Status: Normative source of truth for product requirements, architecture, domain contracts, safety gates, and bead-decomposition readiness. This document is the master doc. `architecture-spec.md` is only a compatibility pointer.

Product scope: full-scope AI-generated Rust codebase. Do not shrink this into a toy MVP. A 30k-line implementation is acceptable when the generated code remains typed, tested, bounded, reviewable, and split into molecular beads.

Decomposition readiness: `decomposition_ready: full-product-dag`. This means this master doc is ready to generate the full product implementation DAG. It does not mean the current repository already implements every behavior, passes every runtime gate, or may claim `SpecComplete` without evidence.

Implementation status: `implementation_complete: false`. Current source code is evidence for what exists, not a constraint that reduces the target product.

## Control Plane: Read This Before Any Decomposition

| Axis | Current value | Meaning |
|---|---|---|
| Canonical source | `MASTER_DOC.md` | This file wins over README, old `architecture-spec.md`, issue comments, generated summaries, and stale agent transcripts. |
| Product scope | Full product | Build the whole Clarity planning compiler described here; do not reduce scope because it is large. |
| Decomposition mode | `full-product-dag` | Generate beads for the entire dependency DAG in section 33. |
| Implementation status | incomplete | Existing code may be thin, legacy, or partial; generated beads shall close that gap. |
| Shipping gate | evidence-bound | No behavior ships without source, tests, schema/proof evidence, reviewer acceptance, and Moon command output. |
| Optional downstream | `bd` emission | `bd` issue creation remains downstream of local validated beads and never defines core `SpecComplete`. |

### Allowed Work From This Master Doc

Agents may generate and execute beads for the full product, including:

1. Schema/domain/reducer foundation.
2. Fjall event store, locks, snapshots, projections, and recovery.
3. Full CLI command behavior.
4. AI provider boundary and OpenCode implementation.
5. Reviewer orchestration and deterministic evidence validation.
6. KIRK16/CUE/enhanced-bead artifact generation.
7. Security redaction, prompt minimization, safe paths, and raw export consent.
8. Optional privacy-consented `bd` emission.

### Forbidden Misreadings

Agents shall not:

1. Treat `foundation`, `delivery wave`, or `dependency order` as scope reduction.
2. Implement only start/status and call the product complete.
3. Treat schema existence as semantic parity without valid/invalid examples and executable checks.
4. Treat Fjall keyspace creation as the full storage contract.
5. Treat reviewer JSON as trusted without deterministic evidence validation.
6. Treat current redb modules as the target canonical store.
7. Treat optional `bd` emission as required for `SpecComplete`.

Date: 2026-06-20

Target repository: `/home/lewis/src/clarity`

Target language: Rust only

Reference repository: `/home/lewis/src/intent-cli`

Reference status: Source material only. Do not build new runtime dependency on Gleam/Erlang.

## 0. Product Requirements / PRD

### 0.1 Product Name

The product is `clarity`: a local, CLI-first Rust planning and interrogation tool.

The canonical specification for the product is this file: `MASTER_DOC.md`.

Any agent, human, or downstream tool that uses `architecture-spec.md`, scattered design notes, issue text, or generated summaries as the source of truth is wrong unless those artifacts explicitly cite this master doc and preserve its normative requirements.

### 0.2 Product Thesis

Rust projects fail less often when planning artifacts are forced through adversarial product, architecture, reliability, security, domain-modeling, testing, and operational review before implementation tasks are generated.

Clarity exists to make shallow planning painful and incomplete plans impossible to accidentally treat as ready.

The tool is intentionally frictionful. It is not a brainstorming toy, requirements notepad, or generic chatbot wrapper. It is a gatekeeping planning compiler for Rust work.

### 0.3 Target Users

Primary users:

1. Solo Rust developers who want implementation agents to receive unambiguous, testable, bounded tasks.
2. Agent orchestrators that need a high-assurance PRD/spec before decomposing work into beads.
3. Technical leads who want to prevent vague feature requests from becoming garbage implementation tasks.
4. Maintainers of local Rust CLIs, libraries, services, storage systems, async systems, and Rust UI applications.

Secondary users:

1. Reviewers who need a deterministic evidence trail for why a plan was accepted or rejected.
2. Operators who need failure-mode, recovery, and telemetry requirements captured before implementation.

Explicit non-users:

1. Non-Rust product teams.
2. Hosted SaaS collaboration teams expecting multi-user editing.
3. Users who want AI to skip hard questions and generate tasks from vibes.

### 0.4 User Problems

The system solves these concrete problems:

1. Users provide broad feature ideas, but agents implement against ambiguous nouns, missing failure modes, and unstated invariants.
2. Existing planning flows reward happy-path prose and do not force preconditions, postconditions, or error taxonomy.
3. Task decomposition happens before the domain model is stable, producing oversized beads and hidden coupling.
4. AI-generated reviewer output is treated as trustworthy even when it lacks deterministic evidence.
5. Planning artifacts become scattered across markdown, JSONL, issue comments, and agent transcripts without one canonical state machine.
6. `bd` issue creation happens too early, leaking rough ideas and creating stale downstream work.

### 0.5 Success Metrics

The product is successful when all of these are true for a representative Rust feature planning session:

1. A user can start, resume, abort, export, and inspect a session entirely from the CLI.
2. The session cannot reach `SpecComplete` until every hard gate has deterministic evidence.
3. The generated `KirkContract16` validates against the canonical KIRK schema and includes sections 0 through 15.
4. Every generated enhanced bead validates against the Clarity enhanced-bead schema.
5. A crash at any documented side-effect boundary either commits a complete event batch or recovers to a named non-corrupt state.
6. Same-session concurrent mutation is rejected with a named error.
7. Sanitized export does not include detected secrets.
8. Provider calls are request-event-first, redacted, bounded, retry-limited, and replay-aware.
9. Optional `bd` emission is idempotent, privacy-consented, and recoverable after partial failure.
10. A black-hat reviewer cannot point to an unspecified reducer transition, external effect, schema, or hard gate and call it hand-wavy.

### 0.6 Product Principles

1. The event log is the truth. Derived files are not truth.
2. Hard gates are hard. The system explains them; it does not skip them.
3. AI output is untrusted input until schema and evidence validation pass.
4. Rust domain types must make illegal states unrepresentable where practical.
5. Every external side effect that matters has a request event before the effect and a terminal event after it.
6. Bead generation is downstream of spec completion; `bd` emission is downstream of local bead validation.
7. Human-readable output is never the only machine-readable contract.
8. Recovery is a product feature, not an implementation afterthought.
9. Security defaults must prevent accidental secret leakage to providers, exports, and `bd`.
10. Boring, explicit, typed workflows beat clever abstractions.

### 0.7 Initial Full Product Scope

The initial product build is intentionally full-scope. It is not a smaller MVP. The scope is large because Clarity is designed for AI-generated implementation, where rigorous decomposition and verification matter more than minimizing line count.

Initial full product includes:

1. Local CLI command surface defined in section 6.
2. Rust-only profiles defined in section 7.
3. Event-sourced session state in Fjall.
4. Session locking for local process safety.
5. Double Diamond interview flow.
6. Global five-lattice review.
7. Six mandatory adversarial reviewers.
8. Hard-gate evaluation.
9. KIRK16 artifact compilation.
10. CUE spec validation.
11. Local enhanced bead generation and validation.
12. Sanitized JSONL projection/export.
13. Explicit-consent raw export.
14. Optional privacy-consented `bd` emission.

Initial full product excludes:

1. Hosted service mode.
2. Multi-user collaborative editing.
3. Remote orchestration.
4. Web UI.
5. Encryption at rest unless the team chooses to promote it from deferred work into foundation scope.

### 0.7A Delivery Waves Are Dependency Order, Not Scope Reduction

Delivery waves exist only to prevent illegal dependency inversions. They do not reduce the product.

| Wave | Purpose | Ships alone? | Notes |
|---|---|---|---|
| Wave 1: contracts | Schemas, types, event envelope, reducer, gate models | No | Establishes the language of the system. |
| Wave 2: canonical storage | Fjall append/replay, locks, snapshots, projections, recovery | No | Implements the event truth layer. |
| Wave 3: CLI behaviors | start/resume/status/abort/export/gates/spec/beads/sessions | No | Exposes product workflows over the reducer and storage. |
| Wave 4: AI/reviewers | provider boundary, prompt minimization, six reviewers, evidence validation | No | Makes adversarial interrogation executable. |
| Wave 5: artifact pipeline | KIRK16, CUE spec, enhanced beads, schema validation | No | Produces validated implementation work. |
| Wave 6: downstream emission | privacy-consented `bd` idempotent emission | Optional | Downstream only; not core completion. |

The product is not accepted until the full wave set required for `SpecComplete` is implemented and proven. Wave boundaries are planning constraints, not cuts to scope.

### 0.8 Product-Level Acceptance Test

Given a user starts a Rust CLI planning session, answers required prompts, survives adversarial review, compiles KIRK16, generates enhanced beads, and exports sanitized artifacts, when the user runs status, the system shall report `SpecComplete` only if all required gates, schemas, reviewers, artifacts, event-prefix hashes, and projection status checks satisfy this master doc.

Given any required gate, schema, reviewer evidence check, storage commit, provider redaction check, or artifact validation fails, when the user requests completion, the system shall refuse `SpecComplete`, name the exact error variant, name the failed gate or artifact, and provide the next valid recovery action.

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

The initial storage, locking, and recovery model assumes one local machine and many possible local processes. Fjall has an exclusive one-process-per-database lock. The initial full product serializes all mutating commands for a given Fjall database root. Same-session concurrent writes are rejected by the domain lock, and same-database concurrent processes are rejected by Fjall's file lock. Concurrent sessions are supported only when they use different database roots or a future single-owner daemon.

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

## 6A. CLI I/O, JSON, Stdin, Signal, and Exit-Code Contract

The CLI contract is stable product surface. Human output may change in wording only when the machine-readable JSON contract remains compatible.

### 6A.1 Global CLI Rules

Every command shall support `--json` unless the command is an interactive TTY-only command.

Every mutating command shall define:

1. Valid source states.
2. Required lock behavior.
3. Events appended before side effects.
4. Atomic Fjall batch boundary.
5. Resulting state.
6. Stable error variant on failure.
7. Human stdout contract.
8. Human stderr contract.
9. JSON response contract.
10. Stable exit code.

Human stdout is for successful user-facing summaries only.

Human stderr is for errors, warnings, recovery hints, and degraded-state notices.

Machine consumers shall use `--json` and shall not parse human prose.

Errors printed in either human or JSON form shall include:

1. `error_code`: stable enum-style string matching the error taxonomy.
2. `session_id`: present when known.
3. `state`: present when known.
4. `message`: short human-readable explanation.
5. `remediation`: exact next valid command or user action when one exists.
6. `evidence_event_ids`: events supporting the failure when applicable.

### 6A.2 Stable Exit Codes

| Exit code | Meaning | Examples |
|---:|---|---|
| 0 | Success | status printed, export written, gates pass |
| 1 | Generic unexpected command failure | uncategorized shell failure after typed mapping failed |
| 2 | CLI usage error | invalid flags, missing required args, invalid enum |
| 3 | Command not allowed in current state | `CommandNotAllowedInState` |
| 4 | Hard gate failure | `GateScoreBelowThreshold`, missing required evidence |
| 5 | Schema/artifact validation failure | CUE validation failed, KIRK16 invalid |
| 6 | Lock/concurrency failure | `SessionLockAlreadyHeld`, `SessionLockOwnerMismatch` |
| 7 | Storage/event failure | Fjall commit failure, event sequence conflict |
| 8 | AI/provider failure | provider unavailable, invalid AI response after retries |
| 9 | Security/privacy failure | secret detected, unsafe export path, raw consent missing |
| 10 | `bd` emission failure | `BdUnavailable`, partial emission, key collision |
| 11 | Recovery/degraded-state block | `RecoveredDegraded` cannot complete |
| 12 | Interview exhausted | terminal `InterviewExhausted` |
| 130 | SIGINT | user interrupt, state preserved at last committed event |

### 6A.3 JSON Envelope

Successful JSON output shall use this top-level envelope:

```json
{
  "ok": true,
  "command": "string",
  "session_id": "string|null",
  "state": "string|null",
  "last_seq": 0,
  "data": {},
  "warnings": []
}
```

Error JSON output shall use this top-level envelope:

```json
{
  "ok": false,
  "command": "string",
  "session_id": "string|null",
  "state": "string|null",
  "last_seq": 0,
  "error": {
    "error_code": "string",
    "message": "string",
    "remediation": "string|null",
    "evidence_event_ids": []
  },
  "warnings": []
}
```

Unknown fields may be added only in minor-compatible schema versions. Required fields may not be removed without a major CLI contract version change.

### 6A.4 Command Output Matrix

| Command | Success stdout | Success JSON `data` | Failure stderr | Exit-code family |
|---|---|---|---|---|
| `clarity interview start --profile <profile>` | Session id, profile, first question | `session_id`, `profile`, `state`, `question`, `lock_expires_at` | validation/lock/schema error + remediation | 0,2,5,6,7,9 |
| `clarity interview resume <session_id>` | Current question or next required action | `state`, `next_action`, `gate_summary`, `lock_expires_at` | state/lock/recovery error | 0,3,6,7,11 |
| `clarity interview status <session_id>` | State, gates, counts, artifacts, projection | `state`, `question_counts`, `gates`, `artifacts`, `projection_status`, `lock` | session/storage error | 0,7 |
| `clarity interview export <session_id>` | Export path and redaction summary | `export_path`, `raw`, `redaction_summary`, `source_seq` | privacy/path/projection error | 0,7,9,11 |
| `clarity interview abort <session_id>` | Abort confirmation and final seq | `state`, `aborted_at`, `last_seq` | state/lock/storage error | 0,3,6,7 |
| `clarity gates run <session_id>` | Gate pass/fail table | `gates`, `failed_gates`, `evidence_event_ids` | storage/schema error | 0,4,5,7 |
| `clarity spec compile <session_id>` | Artifact ids and validation result | `kirk_artifact_id`, `cue_artifact_id`, `schema_hashes` | gate/schema/artifact error | 0,4,5,7,11 |
| `clarity beads generate <session_id>` | Bead count and validation result | `bead_artifact_ids`, `count`, `schema_hash` | spec/schema/artifact error | 0,4,5,7,11 |
| `clarity beads emit <session_id>` | Created/skipped/failed counts | `created`, `skipped`, `failed`, `mappings`, `partial` | consent/bd/idempotency error | 0,9,10 |
| `clarity sessions list` | Session table | `sessions` | storage error | 0,7 |

### 6A.5 Stdin and Interactive Interview Rules

Interactive commands read UTF-8 from stdin. Invalid UTF-8 fails with `InputEncodingInvalid` and exit code 2.

Maximum single answer size is 64 KiB after UTF-8 decoding. Larger input fails with `InputTooLarge` before any provider call.

EOF while waiting for a user answer shall append no partial answer. The command returns `StdinClosed { session_id, state }` with exit code 3. Previously committed events remain authoritative.

SIGINT while no Fjall batch is open exits 130 after printing the last committed state. SIGINT during an external provider or `bd` effect does not assume cancellation succeeded; recovery rules in section 20A apply on resume.

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

## 8A. Command-State-Event Reducer Contract

The reducer is the executable heart of Clarity. Implementation beads shall treat this section as normative over informal transition prose.

### 8A.0 Reducer-First Architecture Rule

The reducer and event contract define the product semantics. Fjall, JSONL, OpenCode, CUE commands, stdout, stderr, and `bd` are shells around that semantics.

Implementation order shall be:

1. Define trusted domain types.
2. Define closed event envelope and payload types.
3. Define pure reducer decisions.
4. Prove reducer transitions and stuck cases with tests/properties.
5. Implement Fjall append/replay as persistence for reducer-approved events.
6. Implement CLI commands as shells that parse input, call the reducer, append events, run external effects, and print envelopes.

Storage shall never invent state transitions. Storage may reject corrupted, non-contiguous, invalid, or hash-mismatched events, but it shall not decide product semantics.

An implementation bead that starts by adding storage behavior without a prior reducer/event obligation shall be rejected unless it is a pure keyspace/bootstrap bead with no product-state transition.

The reducer shall be a pure function over trusted domain types:

```text
reduce(current_state, command, validated_input, current_time, committed_event_prefix) -> ReducerDecision
```

`ReducerDecision` shall contain:

1. Accepted or rejected command.
2. Required lock action.
3. Events to append before any shell side effect.
4. Shell effects to execute after request events commit.
5. Events to append after shell effect completion.
6. Derived indexes to update in the same Fjall batch as appended events.
7. Resulting session state.
8. Error variant when rejected.

### 8A.1 Reducer Invariants

The reducer shall never perform I/O.

The reducer shall never inspect sanitized JSONL projection.

The reducer shall never trust AI/reviewer prose directly.

The reducer shall reject unknown states, unknown commands, unknown event types, unknown schema versions, and non-contiguous event sequences.

The reducer shall not transition to `SpecComplete` unless all SpecComplete acceptance criteria in section 26 are true for the same event prefix.

### 8A.2 Core Command-State Table

| Command | Valid states | Request/pre-side-effect events | Side effects | Terminal events | Result state | Rejection error |
|---|---|---|---|---|---|---|
| `interview start` | `New` or absent session | `SessionLockAcquired`, `InterviewStarted` | none before first question; optional provider call only after `AiCallRequested` | `AiQuestionRecorded` when provider used | `Interviewing` | `CommandNotAllowedInState`, `SessionLockAlreadyHeld`, `SchemaValidationFailed` |
| `interview resume` | any non-terminal except stolen lock cases | `SessionLockAcquired` or `SessionLockRefreshed` | detect ambiguous effects, rebuild projection if requested | `AiEffectAmbiguous` when needed, `RecoveredDegraded` when evidence missing | current valid state or `RecoveredDegraded` | `SessionLockOwnerMismatch`, `ProjectionOutOfSync` |
| user answer | `Interviewing`, `RepairQuestioning` | `UserAnswerRecorded` | gate calculations, optional provider request after commit | `GatePassed`/`GateFailed`, `AiQuestionRecorded` when needed | same state, `NormalQuestioningFrozen`, `Reviewing`, or `InterviewExhausted` | `InputTooLarge`, `CommandNotAllowedInState` |
| `/done` | `Interviewing`, `RepairQuestioning` | `PhaseCompleted` or `NormalQuestioningFrozen` | gate calculations, reviewer panel, artifact compilation only after request events | reviewer/gate/artifact events | `Reviewing`, `RepairQuestioning`, `SpecComplete`, or `InterviewExhausted` | `GateScoreBelowThreshold`, `ReviewerFailed` |
| `/skip` | `Interviewing`, `RepairQuestioning` | `SkipAttempted` | none required | replacement question or reviewer escalation event | same state or `RepairQuestioning` | `SkipLimitExceeded` |
| `gates run` | all non-aborted states | none unless a recheck is requested | deterministic gate calculation | `GatePassed`, `GateFailed`, `GateRecheckRequested` | unchanged unless exhaustion/completion criteria met | `SchemaValidationFailed` |
| `spec compile` | `Reviewing`, `RepairQuestioning`, `Interviewing` after `/done`, not degraded | `ArtifactCompileRequested` | compile KIRK16 and CUE artifacts | `KirkCompiled`, `CueSpecCompiled`, `CueSpecValidated` | unchanged unless bead criteria also complete | `ArtifactGenerationIncomplete`, `SchemaValidationFailed` |
| `beads generate` | artifact-ready states, not degraded | `BeadGenerationRequested` | generate local enhanced bead artifacts | `EnhancedBeadsGenerated`, `EnhancedBeadsValidated` | `SpecComplete` only if all gates/reviewers/artifacts pass | `SchemaValidationFailed`, `ArtifactGenerationIncomplete` |
| `beads emit` | `SpecComplete` | `BdEmitStarted`, `BdCreateRequested` per missing bead before each create | `bd create` shell command | `BdBeadCreated`, `BdBeadSkippedExisting`, `BdEmitPartial`, `BdEmitCompleted`, `BdEmitFailed` | `SpecComplete` | `RawExportRequiresExplicitConsent`, `BdKeyCollision`, `BdUnavailable` |
| `interview abort` | any non-terminal | `InterviewAborted`, `SessionLockReleased` | none | none | `Aborted` | `SessionLockOwnerMismatch` |
| `interview export` | any state with committed events | `RawExportRequested` for raw export; none for sanitized unless export audit enabled | write export file | `RawExportWritten` or `SanitizedExportWritten` | unchanged | `RawExportRequiresExplicitConsent`, `UnsafePathRejected` |

### 8A.3 Stuck-Case Rules

If all six reviewers pass but any hard gate fails, the session shall not enter `SpecComplete`. The system shall append `GateFailed` events for the failed gates and transition to `RepairQuestioning` if reviewer-repair budget remains. If no repair budget remains, it shall append `InterviewExhausted` and enter `InterviewExhausted`.

If a reviewer fails and emits no `required_questions`, the output is schema-invalid unless it marks each blocking finding as `unrecoverable: true` with deterministic evidence. Unrecoverable blocking findings transition to `InterviewExhausted` after recording `ReviewerFailed`.

If an artifact compiles but validation fails, the artifact may be retained as failed evidence, but it shall not satisfy the artifact gate. The session remains in the previous non-terminal state and reports `SchemaValidationFailed`.

If JSONL projection fails after a successful canonical commit, the reducer shall keep the canonical transition and mark projection out of sync. Projection status shall never block `SpecComplete` when it is explicitly marked out of sync and canonical state is intact.

If a lock expires during a command before commit, the command shall fail with `SessionLockExpiredDuringCommand` and append no mutation events unless the failed lock refresh itself was committed as a diagnostic event.

### 8A.4 Artifact Readiness Before SpecComplete

`SpecComplete` shall be reachable only when the same event prefix contains all of these validated terminal events:

1. `KirkCompiled`.
2. `CueSpecCompiled`.
3. `CueSpecValidated`.
4. `EnhancedBeadsGenerated`.
5. `EnhancedBeadsValidated`.
6. Six reviewer pass events.
7. All required gate pass events.

The system may automatically run artifact compilation and bead generation as part of `/done` only if each shell effect is represented by request and terminal events. Otherwise the user must explicitly invoke `spec compile` and `beads generate`.

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
  "pass_claims": [],
  "required_questions": [],
  "required_gate_rechecks": [],
  "confidence": 0.0,
  "reviewed_event_prefix_hash": "sha256"
}
```

Reviewer confidence threshold: 0.75.

If `confidence < 0.75`, the reviewer fails.

If reviewer output is invalid JSON or schema-invalid, the reviewer fails.

If reviewer output lacks evidence event IDs, the reviewer fails.

All six reviewers must pass before `SpecComplete`.

Reviewers may use the same OpenCode provider for the initial full product, but prompt isolation, schema isolation, and evidence citation are mandatory.

## 14A. Reviewer Evidence Validation Contract

Reviewer output is not trusted because it is JSON-shaped. It is trusted only when deterministic validation proves that every claim is tied to relevant evidence from the same committed event prefix.

### 14A.1 Evidence Reference Rules

Every `evidence_event_id` cited by a reviewer shall:

1. Exist in the canonical Fjall event log.
2. Belong to the same `session_id`.
3. Have `seq` less than the reviewer output event sequence.
4. Be within the event prefix the reviewer was asked to inspect.
5. Be an allowed evidence type for the claimed gate or finding.
6. Not be generated by the same reviewer output being validated.
7. Not be a duplicate-only citation used to inflate evidence count.

Evidence from sanitized JSONL, stdout text, provider prose, issue comments, or stale artifacts is not valid unless it maps back to canonical event IDs.

### 14A.2 Claim-to-Evidence Mapping

Reviewer reports shall map evidence IDs to specific claims, not just include a flat evidence list.

The normalized reviewer report schema shall include:

```json
{
  "reviewer": "sre|architect|wardley|munger|security|test",
  "verdict": "Pass|Fail",
  "blocking_findings": [
    {
      "finding_id": "string",
      "claim": "string",
      "gate_ids": [],
      "evidence_event_ids": [],
      "required_questions": [],
      "unrecoverable": false
    }
  ],
  "warnings": [
    {
      "warning_id": "string",
      "claim": "string",
      "evidence_event_ids": []
    }
  ],
  "pass_claims": [
    {
      "gate_id": "string",
      "claim": "string",
      "evidence_event_ids": []
    }
  ],
  "required_questions": [],
  "required_gate_rechecks": [],
  "confidence": 0.0,
  "reviewed_event_prefix_hash": "sha256"
}
```

### 14A.3 Reviewer Pass Rules

A reviewer pass is valid only when all are true:

1. The report validates against the reviewer CUE schema.
2. `confidence >= 0.75`.
3. The report cites the exact event-prefix hash it reviewed.
4. Every pass claim has at least one relevant evidence event.
5. Every required reviewer category has at least one pass claim.
6. No blocking findings are present.
7. Deterministic evidence validation passes.

Confidence shall never compensate for missing, stale, irrelevant, future, duplicate-only, or self-generated evidence.

### 14A.4 Reviewer Failure Rules

A reviewer failure with recoverable findings shall include reviewer-required repair questions. Those questions consume the reviewer-repair budget only after being asked to the user.

A reviewer failure with no repair questions shall either be schema-invalid or explicitly unrecoverable. Unrecoverable failures shall include deterministic evidence and shall block `SpecComplete` permanently for that session revision.

Reviewer output that cites irrelevant evidence shall fail with `ReviewerEvidenceInvalid`, not with generic `ReviewerFailed`.

### 14A.5 Prompt Isolation Rules

Each reviewer prompt shall receive only:

1. Sanitized canonical evidence needed for that reviewer.
2. Schema/rubric for that reviewer.
3. Event IDs and artifact IDs needed for citation.

Reviewer prompts shall not include provider credentials, raw secrets, unrelated sessions, or instructions from user answers as executable prompt instructions.

### 14A.6 Reviewer Evidence Matrix

The deterministic evidence validator shall reject reviewer claims that cite evidence outside the allowed event/artifact families below.

| Reviewer | Required pass-claim categories | Allowed evidence event/artifact families |
|---|---|---|
| SRE | recovery, durability, concurrency, observability, blast radius | `RecoveredDegraded`, `AiEffectAmbiguous`, `JsonlProjectionFailed`, `SessionLock*`, `GatePassed`, `GateFailed`, storage/recovery artifacts, crash-recovery test evidence |
| Architect | domain model, reducer purity, invariants, type boundaries, dependency direction | reducer transition artifacts, `KirkCompiled`, `CueSpecValidated`, `EnhancedBeadsValidated`, type-contract evidence, event schema validation evidence |
| Wardley | user value, build/buy assumptions, commodity boundaries, strategic sequencing | KIRK sections 1, 3, 5, 6, 7; VORP evidence; decomposition DAG evidence; explicit non-goal evidence |
| Munger | inversion, incentives, second-order effects, pre-mortem failures | `PmeFailureCategoriesMapped`, `PmeHumanLimitationsModeled`, `PmeNfrTradeoffsRecorded`, hole-punching gate evidence, pre-mortem answers |
| Security | secrets, prompt injection, path safety, raw export, bd privacy, override poisoning | `PrivacyConsentRecorded`, redaction evidence, provider prompt scan evidence, `RawExportRequested`, safe-path validation evidence, schema manifest evidence, forbidden-question override evidence |
| Test | ATDD, behavior coverage, property tests, deterministic assertions, mutation resistance | test-plan artifacts, Moon command evidence, `EnhancedBeadsValidated`, acceptance-test evidence, property/fuzz/proof lane evidence |

### 14A.7 Gate Evidence Matrix

Hard-gate pass events shall cite only evidence capable of proving that gate.

| Gate | Required evidence |
|---|---|
| `required-fields` | User answer events and parsed field extraction evidence for problem, persona, solution, constraints, and success criteria. |
| `vorp` | VORP score artifact plus cited answers for value, obviousness, realism, and possibility. |
| `straw-man` | Straw-man validation artifact and antithesis/objection answers. |
| `antithesis` | Antithesis answer events and challenge-quality evidence. |
| `hole-punching` | Discovery hole, edge-case hole, and motivation drop-off evidence. |
| `ears` | Parsed EARS requirements with at least ubiquitous, event-driven, and unwanted requirement evidence. |
| `kirk16` | `KirkCompiled` artifact with sections `0..15`, non-empty evidence IDs, and source prefix hash. |
| `cue-spec` | `CueSpecCompiled` and `CueSpecValidated` events against `schemas/clarity-spec.cue`. |
| `enhanced-bead` | `EnhancedBeadsGenerated` and `EnhancedBeadsValidated` events against `schemas/enhanced-bead.cue`. |
| `security` | Redaction policy, prompt minimization, safe path, raw export consent, override trust, and bd privacy evidence as applicable. |
| `nfr` | Explicit latency, durability, recovery, resource, observability, and operational tradeoff evidence. |
| `reviewer` | Six valid reviewer pass reports, each with evidence mapped by claim and reviewed prefix hash. |

Evidence mismatch is a hard failure named `ReviewerEvidenceInvalid` or `GateEvidenceInvalid`; confidence, prose quality, or reviewer authority shall not override this matrix.

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

State-driven, optional, and complex requirements are encouraged but not hard-required for the initial full product unless the selected profile CUE makes them required.

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

## 17A. Normative CUE Schema Reconciliation Contract

This master doc is normative over stale schema names, stale profile enums, and `intent-cli` leftovers. Existing CUE files shall be repaired to match this section before decomposition is allowed.

### 17A.1 Required Schema Files

The repository shall contain these normative schema files:

| File | Required definitions | Purpose |
|---|---|---|
| `schemas/questions.cue` | `#QuestionBank`, `#Question`, `#GateRef`, `#RustProfile` | canonical questions and strengthening-only overrides |
| `schemas/kirk.cue` | `#KirkContract16`, `#KirkSection`, `#KirkMetadata` | session-level KIRK artifact |
| `schemas/enhanced-bead.cue` | `#ClarityEnhancedBead`, `#BeadEvidence`, `#BeadVerificationLane` | local enhanced bead artifacts |
| `schemas/reviewer-report.cue` | `#ReviewerReport`, `#ReviewerFinding`, `#ReviewerPassClaim` | six-reviewer reports |
| `schemas/events.cue` | `#EventEnvelope`, `#EventPayload`, payload definitions for every event type | event log validation and replay |
| `schemas/clarity-spec.cue` | `#ClaritySpec`, `#RustRequirement`, `#GateResult` | generated CUE spec artifact |

If a legacy schema path remains for compatibility, it shall import or alias the Clarity definition. It shall not define a competing contract.

### 17A.2 Rust Profile Enum

The canonical profile enum shall be exactly:

1. `rust-cli`
2. `rust-library`
3. `rust-web-service`
4. `rust-async-service`
5. `rust-storage`
6. `rust-ui`
7. `rust-refactor`

Generic profiles such as `api`, `cli`, `event`, `data`, `workflow`, `ui`, and `common` are legacy inputs only. They shall not validate as Clarity master-doc profile values unless explicitly migrated to one of the Rust profiles above.

### 17A.3 Clarity Enhanced Bead Identity

Enhanced bead IDs shall use Clarity identity, not `intent-cli-*` identity.

Accepted local bead ID forms:

```text
clarity-<session_slug>-<canonical_task_slug>-<content_hash_12>
```

or the deterministic internal ID:

```text
sha256(session_id || canonical_task_id || enhanced_bead_schema_hash || canonical_payload_hash)
```

Human-visible IDs shall match:

```text
^clarity-[a-z0-9][a-z0-9-]{2,80}-[a-f0-9]{12}$
```

The schema shall reject `intent-cli-*` IDs for new Clarity beads.

### 17A.4 KIRK16 Schema Contract

`schemas/kirk.cue` shall define `#KirkContract16` with exactly sections `0..15` matching section 16 of this master doc.

The schema shall reject:

1. Missing sections.
2. Duplicate section indexes.
3. Extra section indexes.
4. Empty required evidence lists.
5. Section 15 supplied as user-authored text rather than derived metadata.
6. Schema hash mismatch against the manifest.

### 17A.5 Reviewer, Event, and Bead Schemas

Reviewer reports shall validate against `schemas/reviewer-report.cue` and the deterministic evidence validation contract in section 14A.

Events shall validate against `schemas/events.cue`. Event payloads shall be a closed tagged union. Unknown event types or unknown required fields fail replay.

Enhanced beads shall validate against `schemas/enhanced-bead.cue`. The schema shall require Rust-first testing, zero-panic production constraints, Moon-only command evidence, typed error taxonomy, and allowed verification lanes.

### 17A.6 Schema Hash Manifest

The repository shall contain a trusted schema manifest at `schemas/manifest.json` or `schemas/manifest.cue` recording:

1. Schema file path.
2. Schema semantic version.
3. Canonical SHA-256 hash.
4. Required definitions exported by the file.
5. Compatibility aliases, if any.
6. Validation command.

Project-local CUE overrides shall declare the canonical schema hash they extend. Overrides targeting unknown or mismatched hashes fail with `SchemaTrustManifestMismatch`.

### 17A.7 Minimum Schema Validation Commands

Before `decomposition_ready` can become `full-product-dag`, the repository shall have commands equivalent to:

```text
cue vet schemas/questions.cue
cue vet schemas/kirk.cue
cue vet schemas/enhanced-bead.cue
cue vet schemas/reviewer-report.cue
cue vet schemas/events.cue
cue vet schemas/clarity-spec.cue
```

Project policy may wrap these commands in Moon tasks, but the evidence shall name the exact commands run and their output.

### 17A.8 Fail-Closed Rule

If any normative schema fails evaluation, has a missing required definition, has a stale `intent-cli` identity requirement, or conflicts with this master doc, then `SpecComplete`, bead generation, and `arch-spec-to-beads` decomposition shall fail closed.

### 17A.9 Canonical Schema Examples

Schemas shall include or be tested against representative valid and invalid examples. These examples are semantic acceptance tests, not documentation garnish.

#### 17A.9.1 Event Envelope Examples

Valid `AiCallRequested` event shape:

```json
{
  "session_id": "clarity-session-1",
  "seq": 7,
  "event_id": "018f0000-0000-7000-8000-000000000007",
  "event_type": "AiCallRequested",
  "payload": {
    "kind": "ai-call-requested",
    "effect_id": "ask-q-7",
    "operation": "ask_question",
    "provider": "opencode",
    "attempt_no": 1,
    "max_attempts": 3,
    "schema_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "prompt_hash": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "redaction_policy_version": "1.0.0",
    "secret_scan_result": "pass",
    "input_event_ids": ["018f0000-0000-7000-8000-000000000006"],
    "timeout_ms": 30000
  },
  "created_at": "2026-06-21T00:00:00Z",
  "idempotency_key": "ai:clarity-session-1:ask-q-7:1",
  "schema_version": "1.0.0",
  "actor": "System",
  "prev_event_hash": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
}
```

Invalid event shape that shall be rejected:

```json
{
  "event_type": "AiCallRequested",
  "payload": { "kind": "generic", "description": "trust me" }
}
```

Reason: payload kind does not match the event type and would launder untyped provider effects.

#### 17A.9.2 Reviewer Report Examples

Valid passing reviewer report requires pass claims with evidence:

```json
{
  "reviewer": "security",
  "verdict": "Pass",
  "blocking_findings": [],
  "warnings": [],
  "pass_claims": [
    {
      "gate_id": "security",
      "claim": "Provider prompt redaction policy is specified and evidence-cited.",
      "evidence_event_ids": ["018f0000-0000-7000-8000-000000000012"]
    }
  ],
  "required_questions": [],
  "required_gate_rechecks": [],
  "confidence": 0.92,
  "reviewed_event_prefix_hash": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
}
```

Invalid reviewer report that shall be rejected:

```json
{
  "reviewer": "security",
  "verdict": "Pass",
  "blocking_findings": [],
  "pass_claims": [],
  "confidence": 0.99
}
```

Reason: high confidence without pass claims and evidence is meaningless.

#### 17A.9.3 KIRK16 Examples

Valid KIRK16 artifact shall have exactly 16 ordered sections indexed `0..15`, and every section shall cite at least one event ID. Section 15 shall contain derived compilation metadata, not user prose.

Invalid KIRK16 artifact examples:

1. Missing section 14 EARS requirements.
2. Duplicate section index 7.
3. Section 15 with no `event_log_hash`.
4. Any section with empty `evidence_event_ids`.

#### 17A.9.4 Enhanced Bead Examples

Valid enhanced bead identity:

```text
clarity-session-reducer-transition-a1b2c3d4e5f6
```

Invalid enhanced bead identity:

```text
intent-cli-reducer-transition
```

Reason: Clarity shall not emit stale `intent-cli` identities for new work.

Every valid enhanced bead shall include master-doc references, preconditions, postconditions, type-level invariants, touched error variants, touched event types, tests to write first, Moon evidence commands, allowed files, and completion evidence.

#### 17A.9.5 Question Override Examples

Valid local override: adds a stricter storage corruption question for `rust-storage` without removing canonical security/recovery questions.

Invalid local override: removes the provider secret-scan question or adds “paste your API key so the reviewer can inspect it.”

Reason: overrides may strengthen gates, but shall not weaken or harvest secrets.

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

## 18A. Fjall Keyspace, Value, Batch, and Migration Contract

### 18A.1 Keyspace Value Contracts

| Keyspace | Key bytes | Value encoding | Max value size | Corruption behavior |
|---|---|---|---:|---|
| `events` | `[session_id_hash_16 | seq_be_u64]` | canonical JSON `EventEnvelope` validated by `schemas/events.cue` | 1 MiB | stop replay with `EventPayloadSchemaInvalid` or `StorageCorruptionDetected` |
| `snapshots` | `[session_id_hash_16 | source_seq_be_u64]` | canonical JSON snapshot with event-prefix hash | 8 MiB | discard snapshot and replay events; report `SnapshotHashMismatch` |
| `locks` | `[session_id_hash_16]` | canonical JSON `SessionLock` | 16 KiB | fail mutating command with `StorageCorruptionDetected` |
| `artifacts` | `[session_id_hash_16 | artifact_kind_u8 | artifact_id_hash_16]` | canonical JSON artifact envelope plus payload hash | 4 MiB per artifact | artifact invalid; canonical events remain |
| `gate_results` | `[session_id_hash_16 | gate_id_u16]` | canonical JSON latest `GateResult` plus source seq | 256 KiB | recompute from events; if recompute fails, fail gate run |
| `projection_status` | `[session_id_hash_16]` | canonical JSON projection status | 64 KiB | mark projection unknown; do not use projection for transitions |
| `bd_mappings` | `[session_id_hash_16 | local_bead_id_hash_16]` | canonical JSON bd mapping with content hash | 64 KiB | block bd emission with `LocalBeadMappingConflict` |

All stored values shall include:

1. `schema_version`.
2. `schema_hash` where a CUE schema applies.
3. `created_at` or `updated_at`.
4. `source_seq` or event range when derived.
5. Payload hash for artifact and derived values.

### 18A.2 Batch Boundaries

Each mutating command shall use exactly one Fjall write batch for a single reducer decision unless an external side effect requires a request-event batch followed by a terminal-event batch.

Request-event batch pattern:

1. Acquire/refresh lock.
2. Append request event.
3. Update derived indexes needed to mark the effect in-flight.
4. Persist batch.
5. Execute external side effect.
6. Append terminal event in a second batch.

No external side effect may be executed before its request event commits.

### 18A.3 Persist Modes

The initial full product shall use `PersistMode::SyncAll` for:

1. Session start.
2. User answers.
3. Lock mutation.
4. Provider request events.
5. Provider terminal events.
6. Artifact validation events.
7. bd request and terminal events.
8. Abort and terminal state events.

Performance beads may weaken persist mode only after benchmarks, crash tests, and an explicit durability tradeoff are accepted.

### 18A.4 One-Process-Per-Database Behavior

Fjall database open failure caused by another live process shall map to `DatabaseAlreadyOpen { path, pid_hint }` and exit code 6 for mutating commands.

Read-only commands may either fail with the same error or use a future read-only daemon/API. The initial full product shall fail closed rather than bypass Fjall locking with ad hoc file reads.

### 18A.5 redb Migration/Discard Policy

Existing redb state in the current repository is not canonical for the new Clarity CLI.

Initial full-product policy: no automatic redb migration is required. The new Fjall store starts from imported/exported artifacts or a new session.

If a future migration is added, it shall be an explicit command that:

1. Reads redb as untrusted legacy input.
2. Parses every value into Clarity domain types.
3. Appends `LegacyRedbImportStarted` and terminal import events.
4. Marks imported raw-evidence gaps.
5. Prevents `SpecComplete` until missing raw evidence is restored or explicitly proven unnecessary.

Silent redb reuse is forbidden.

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

`ArtifactCompileRequested`

`BeadGenerationRequested`

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

`PrivacyConsentRecorded`

`RawExportWritten`

`SanitizedExportWritten`

### bd Events

`BdEmitStarted`

`BdCreateRequested`

`BdBeadCreated`

`BdBeadSkippedExisting`

`BdEmitPartial`

`BdEmitCompleted`

`BdEmitFailed`

## 19A. Closed Event Payload Schema Contract

The event envelope in section 18 is not sufficient by itself. `payload` shall be a closed tagged union defined in `schemas/events.cue`.

### 19A.1 Event Envelope Rules

Every event shall include:

1. `session_id`: typed session ID string.
2. `seq`: contiguous positive u64 for that session.
3. `event_id`: UUIDv7 or equivalent sortable unique ID.
4. `event_type`: one of the taxonomy values in section 19.
5. `payload`: closed schema matching `event_type`.
6. `created_at`: RFC3339 timestamp from a monotonic-clock-checked time source.
7. `idempotency_key`: stable per semantic effect.
8. `schema_version`: event schema semantic version.
9. `actor`: `User|AiProvider|Reviewer|System|Bd`.
10. `prev_event_hash`: hash of previous event for the session, or null for seq 1.
11. `event_hash`: hash of canonical event envelope with `event_hash` omitted.

Replay shall fail closed on unknown event type, unknown required payload field, missing required payload field, invalid schema version, hash mismatch, or sequence gap.

### 19A.2 Common Payload Limits

Unless a stricter limit is specified:

1. Event envelope plus payload max serialized size is 1 MiB.
2. User-authored text field max size is 64 KiB.
3. Provider raw response field max size is 256 KiB after redaction.
4. Artifact payloads larger than 4 MiB shall live in the `artifacts` keyspace and be referenced by hash from events.

### 19A.3 Required Payload Fields by Event Family

Session events shall include source state, destination state when a transition occurs, command name, and actor.

Answer events shall include `question_id`, `question_kind`, `normalized_answer_hash`, `raw_answer_ref` or raw answer payload according to privacy policy, and supported gate IDs.

Gate events shall include `gate_id`, threshold, score or boolean result, deterministic evaluator version, input event IDs, and failure reasons.

Artifact events shall include `artifact_kind`, `artifact_id`, `schema_hash`, `payload_hash`, `source_event_prefix_hash`, and validation result.

Projection/export events shall include destination path hash, redaction policy version, source seq, source event-prefix hash, and raw/sanitized mode.

Lock events shall include owner token hash, acquired/refreshed/released timestamps, expiry, hostname hash, pid hint, and compare-and-set result.

### 19A.4 AI Effect Payloads

`AiCallRequested` payload shall include:

1. `effect_id`.
2. `operation`: `ask_question|extract_fields|review_artifact|repair_json|summarize|health_check`.
3. `provider`.
4. `model` when applicable.
5. `attempt_no`.
6. `max_attempts`.
7. `schema_hash` for expected response.
8. `prompt_hash` over minimized/redacted prompt.
9. `redaction_policy_version`.
10. `secret_scan_result`.
11. `input_event_ids`.
12. `timeout_ms`.

Every AI terminal event shall reference:

1. `effect_id`.
2. Original `AiCallRequested.event_id`.
3. `attempt_no`.
4. `response_hash` or failure category.
5. Validation result.
6. Repair attempt IDs when JSON repair occurred.

Provider calls are at-least-once effects. Idempotency is logical, not provider-enforced.

### 19A.5 Reviewer Payloads

`ReviewerPanelStarted` shall include reviewer set, reviewed event-prefix hash, rubric schema hashes, and artifact IDs reviewed.

`ReviewerOutputRecorded` shall include reviewer name, raw output artifact hash, normalized reviewer report hash, reviewed event-prefix hash, and schema validation status.

`ReviewerPassed` and `ReviewerFailed` shall include deterministic evidence validation result. They shall not be emitted from reviewer prose alone.

### 19A.6 bd Effect Payloads

`BdCreateRequested` shall be appended before invoking `bd create` and shall include:

1. `local_bead_id`.
2. `canonical_task_id`.
3. `content_hash`.
4. `schema_hash`.
5. `bd_command_args_hash`.
6. `idempotency_key`.
7. `privacy_consent_event_id`.
8. `redaction_policy_version`.

`content_hash` shall be SHA-256 over canonical JSON for the complete enhanced bead payload plus the enhanced-bead schema hash.

`BdBeadCreated` shall include `BdCreateRequested.event_id`, `bd_id`, observed issue URL or ID, and created content hash.

`BdBeadSkippedExisting` shall include matching `bd_id`, matching content hash, and scan evidence.

`BdEmitPartial` shall include succeeded, skipped, failed, and pending local bead IDs.

### 19A.7 Stable Idempotency Keys

Idempotency keys shall be derived from semantic intent, not wall-clock time.

Examples:

1. User answer: `answer:<session_id>:<question_id>:<answer_revision>`.
2. AI attempt: `ai:<session_id>:<effect_id>:<attempt_no>`.
3. Artifact compile: `artifact:<session_id>:<artifact_kind>:<source_event_prefix_hash>`.
4. bd create: `bd-create:<session_id>:<local_bead_id>:<content_hash>`.

If the same `(session_id, idempotency_key)` is seen with different semantic payload, the system shall fail with `IdempotencyKeyCollision`.

## 20. Snapshot and Projection Rules

Snapshots are performance optimizations, not authority.

Each snapshot stores `source_seq`, `event_prefix_hash`, `schema_version`, and serialized state.

On load, if snapshot hash mismatches event log prefix, discard snapshot and replay events.

JSONL projection is written after Fjall batch commit. Projection failure never rolls back canonical events.

If projection is out of sync, `clarity interview status` reports it and offers projection rebuild.

Disaster recovery from sanitized JSONL creates `RecoveredDegraded` state with `raw_transcript_missing: true`.

## 20A. Crash Recovery Matrix

Recovery shall be deterministic from Fjall canonical state. The system shall never infer success from a missing terminal event.

| Crash point | Observed on resume | Required recovery behavior | State impact |
|---|---|---|---|
| Before request event commit | no request event | treat effect as never started; user may retry command | unchanged |
| After request event commit, before external effect starts | request event without terminal event | append ambiguity/failed diagnostic as appropriate; retry same logical effect if safe | unchanged or explicit in-flight recovery |
| During AI provider call | `AiCallRequested` without terminal AI event | append `AiEffectAmbiguous`; retry under same `effect_id` with next attempt if budget remains | unchanged until terminal AI event |
| After AI success before terminal event commit | request event but no terminal event | treat provider outcome unknown; append `AiEffectAmbiguous`; retry; do not trust local temp files | unchanged |
| After terminal AI event commit before projection write | terminal event exists, projection stale | canonical state wins; mark projection out of sync or rebuild | canonical transition preserved |
| During reviewer call | reviewer request/AI request without valid reviewer terminal event | append `AiEffectAmbiguous` or `ReviewerOutputInvalid`; rerun reviewer if retry budget remains | `Reviewing` |
| After reviewer output stored but before deterministic evidence validation | `ReviewerOutputRecorded` without `ReviewerPassed`/`ReviewerFailed` | run deterministic validation; append pass/fail event | `Reviewing` or repair/exhaustion transition |
| During KIRK/CUE artifact write | `ArtifactCompileRequested` without terminal artifact events | discard temp artifact; recompile from event prefix | unchanged |
| After artifact value written but before validation event | artifact value exists without terminal event | validate hash/schema; append terminal validation or failure event | unchanged until validated |
| After `EnhancedBeadsGenerated` before `EnhancedBeadsValidated` | generated artifacts without validation | run schema validation; append pass/fail | no `SpecComplete` until validated |
| After Fjall canonical commit before JSONL projection | canonical seq ahead of projection | mark projection out of sync; offer rebuild | canonical state preserved |
| During sanitized export file write | export temp file may exist | delete or ignore temp file; retry export from canonical state | unchanged |
| During raw export after consent | consent event exists, no raw export terminal event | require re-confirmation unless export destination is identical and temp file is absent | unchanged |
| After `BdCreateRequested` before `bd create` starts | request exists, no terminal event | scan bd by `Clarity-Bead-Key`; create if absent; terminal event records outcome | `SpecComplete` preserved |
| During `bd create` | request exists, no terminal event | scan bd before retry; never blindly create duplicate | `SpecComplete` preserved; emission may be partial |
| After `bd create` succeeds before terminal event commit | bd issue may exist, no mapping | scan by key/content hash; append `BdBeadCreated` or collision error | `SpecComplete` preserved |
| Fjall snapshot corruption | snapshot hash mismatch | discard snapshot, replay events | state from replay |
| Event payload corruption | event hash/schema mismatch | fail closed with storage corruption; no projection fallback to complete | blocked |

### 20A.1 Temporary Files

Artifact/export temporary files shall be written under a session-scoped temp directory. They shall include the target artifact ID and source event-prefix hash in the filename. Rename to final path shall occur only after content hash verification.

### 20A.2 Ambiguous External Effects

Ambiguous external effects shall not be hidden from the user. `status` shall report them until a terminal recovery event resolves the ambiguity.

### 20A.3 Recovery From Sanitized JSONL

Sanitized JSONL lacks raw transcript fidelity by design. Rebuilding Fjall from sanitized JSONL shall create `RecoveredDegraded` and shall set evidence-gap records for every missing raw field. `SpecComplete` is forbidden until those gaps are filled or regenerated from acceptable raw evidence.

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

Fjall raw transcript data lives under XDG private data directory with `0700` directory permissions and `0600` file permissions for the initial full product.

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

## 22A. Security Acceptance and Trust Contract

### 22A.1 Plaintext Raw Transcript Risk Decision

The initial full product explicitly accepts plaintext local raw transcript storage only under all of these constraints:

1. Data directory permissions are `0700`.
2. Data files are `0600`.
3. Raw export requires explicit consent.
4. Provider prompts use minimized/redacted context by default.
5. Sanitized exports and `bd` emission run the redaction policy.
6. Status output warns that encryption at rest is not enabled.

If any of those constraints cannot be implemented, encryption at rest moves from deferred work into foundation scope.

### 22A.2 Provider Prompt Minimization and Secret Scan

Before every provider request, the system shall:

1. Select only the minimum event/artifact fields required for the operation.
2. Treat all user-authored content as quoted data, not instructions.
3. Run deterministic secret detectors.
4. Redact detected non-essential secrets.
5. Fail with `SecretDetectedInProviderRequest` when an essential field contains an unredactable secret.
6. Record redaction policy version and prompt hash in `AiCallRequested`.

No provider request may be sent before the corresponding `AiCallRequested` event commits with `secret_scan_result: pass`.

### 22A.3 Redaction Policy

The redaction policy shall cover at least:

1. API keys and tokens with common prefixes.
2. AWS access keys and secret keys.
3. GitHub tokens.
4. Bearer tokens.
5. Private key blocks.
6. Connection strings with embedded credentials.
7. Environment assignments matching secret-like names.
8. SSH private keys.
9. npm, cargo, docker, cloud, and database tokens where recognizable.

Redaction shall replace secret values with deterministic placeholders such as `[REDACTED_SECRET_3_HASH_abcd1234]` so repeated occurrences can be correlated without leaking the value.

### 22A.4 bd Privacy Consent

`clarity beads emit` shall require explicit privacy consent distinct from raw export consent.

Before emission, the system shall show or return in JSON:

1. Number of beads to emit.
2. Redaction policy version.
3. Whether raw excerpts are included. The initial full product shall default to no raw excerpts.
4. Destination command and bd database context.
5. Content hashes for emitted beads.

The system shall append `PrivacyConsentRecorded` before `BdEmitStarted`.

Full enhanced bead content may be emitted only after redaction and consent. If redaction fails, emission fails with `RedactionPolicyViolation`.

### 22A.5 Schema and Question Override Trust

Canonical schema hashes shall be pinned in the trusted schema manifest described in section 17A.

Project-local question and CUE overrides shall be loaded only from canonicalized paths under the project root. Symlinks that escape the project root are forbidden.

Overrides shall be rejected if they:

1. Weaken canonical gates.
2. Remove required security/reliability/concurrency/verification questions.
3. Add questions whose primary purpose is secret harvesting.
4. Request provider credentials, private keys, tokens, or unrelated personal data.
5. Change canonical schema hashes without manifest approval.

Forbidden-question violations fail with `QuestionOverrideForbiddenSecurityQuestion`.

### 22A.6 Safe Newtypes and Path Validation

The implementation shall parse these into safe newtypes at the boundary:

1. `SessionId`.
2. `EventId`.
3. `ArtifactId`.
4. `GateId`.
5. `QuestionId`.
6. `LocalBeadId`.
7. `SchemaHash`.
8. `ContentHash`.
9. `SafeProjectPath`.
10. `SafeExportPath`.
11. `ProviderName`.
12. `RustProfile`.

Path newtypes shall enforce:

1. Canonical root containment.
2. No `..` traversal escape.
3. No symlink escape.
4. No FIFO/socket/device special files for export targets.
5. Parent directory exists and has safe permissions before writing.

Unsafe paths fail with `UnsafePathRejected`.

### 22A.7 Audit Logging Without Secret Leakage

Logs may include event IDs, hashes, schema versions, counts, durations, and error codes. Logs shall not include raw answers, provider prompts, provider responses, API keys, raw bead descriptions before redaction, or raw export payloads.

## 23. bd Emission

`clarity interview` never emits bd issues.

`clarity beads emit <session_id>` emits generated, locally validated, redacted enhanced beads only after explicit bd privacy consent.

Local bead identity:

`local_bead_id = sha256(session_id || canonical_task_id || enhanced_bead_schema_hash || canonical_payload_hash)`

The bd description includes:

`Clarity-Bead-Key: <local_bead_id>`

Emission behavior:

1. Scan existing bd issues for the key.
2. If exactly one matching issue has matching content hash, record mapping and skip create.
3. If matching key has different content, fail with `BdKeyCollision`.
4. If multiple matches exist, fail with `BdExistingIssueAmbiguous`.
5. If no match exists, append `BdCreateRequested` before the external command.
6. Call `bd create` with title, type, priority, and redacted enhanced bead description.
7. Record `local_bead_id -> bd_id` after successful create.
8. On partial failure, record `BdEmitPartial`; retry emits only missing beads after scanning by key/content hash.

No rollback is attempted for successfully created bd issues.

`bd` emission is never required for `SpecComplete`.

## 24. Error Taxonomy

The Rust error enum should include at least these variants.

### Session and State Errors

`CommandNotAllowedInState { command, state }`

`InterviewExhausted { failed_gates, missing_answers, reviewer_findings }`

`QuestionBudgetExceeded { session_id, max_questions, attempted_question_kind }`

`SkipLimitExceeded { session_id, gate, attempts }`

`StdinClosed { session_id, state }`

`InputTooLarge { bytes, max_bytes }`

`InputEncodingInvalid { encoding, command }`

### Lock Errors

`SessionLockAlreadyHeld { session_id, holder, expires_at }`

`SessionLockOwnerMismatch { session_id, attempted_owner_token }`

`SessionLockExpiredDuringCommand { session_id, owner_token, expired_at }`

`SessionLockRefreshFailed { session_id, owner_token, reason }`

`ClockMovedBackwards { previous_observed_at, observed_at }`

`DatabaseAlreadyOpen { path, pid_hint }`

### Event and Storage Errors

`EventSeqConflict { session_id, expected_next_seq, actual_next_seq }`

`EventIdCollision { event_id }`

`IdempotencyKeyCollision { session_id, idempotency_key, existing_event_id, attempted_event_type }`

`FjallCommitFailed { session_id, attempted_seq, source }`

`SnapshotHashMismatch { session_id, snapshot_seq, expected_hash, actual_hash }`

`ProjectionWriteFailed { session_id, committed_through_seq, projection_path, source }`

`ProjectionOutOfSync { session_id, fjall_last_seq, jsonl_last_seq }`

`EventPayloadSchemaInvalid { event_id, event_type, schema_hash, validation_errors }`

`StorageCorruptionDetected { keyspace, key_hash, reason }`

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

`SchemaTrustManifestMismatch { schema_path, expected_hash, actual_hash }`

### Reviewer Errors

`ReviewerOutputInvalid { reviewer, validation_errors, raw_output_event_id }`

`ReviewerFailed { reviewer, blocking_findings }`

`ReviewerConfidenceTooLow { reviewer, confidence, threshold }`

`ReviewerEvidenceInvalid { reviewer, evidence_event_id, reason }`

### Security and Export Errors

`RawExportRequiresExplicitConsent { session_id }`

`RedactionPolicyViolation { export_event_id, detector, match_count }`

`SecretDetectedInProviderRequest { detector, action }`

`UnsafePathRejected { path, reason }`

`QuestionOverrideForbiddenSecurityQuestion { question_id, reason }`

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
9. CUE spec validates against `schemas/clarity-spec.cue` and `schemas/kirk.cue`.
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

This master doc is ready for full-product DAG decomposition when `decomposition_ready: full-product-dag` is set in the header. That value means the document is complete enough to generate the complete implementation bead graph. It does not mean implementation is complete, tests pass, or runtime behavior is accepted.

### 31.1 Doc-Level Readiness Checklist

The master doc shall define:

1. Product requirements and initial full-product/non-goal scope.
2. Exact state machine and reducer stuck-case behavior.
3. Exact CLI I/O, JSON, stdin, signal, and exit-code contracts.
4. Exact event schema, closed payload taxonomy, idempotency, and hash rules.
5. Exact gate thresholds.
6. Exact artifact schemas and section lists.
7. Exact lock protocol.
8. Exact Fjall keyspaces, value encodings, batch boundaries, and recovery rules.
9. Exact reviewer contract and deterministic reviewer-evidence validation.
10. Exact bd idempotency, privacy consent, and partial failure behavior.
11. Exact Rust-only scope and profiles.
12. Exact security/export/provider-prompt/local-override policy.
13. Exact bead decomposition contract and dependency DAG.

This file now defines those doc-level contracts. That does not mean implementation is complete.

### 31.2 Source/Schema Evidence Checklist

Before `arch-spec-to-beads` may create full-product implementation beads, a reviewer shall verify evidence for:

1. `schemas/enhanced-bead.cue` evaluates successfully.
2. `schemas/enhanced-bead.cue` rejects stale `intent-cli-*` IDs and accepts Clarity IDs.
3. `schemas/questions.cue` exposes exactly the Rust profiles in section 7.
4. `schemas/kirk.cue` defines `#KirkContract16` with sections 0 through 15.
5. `schemas/reviewer-report.cue` exists and validates reviewer reports.
6. `schemas/events.cue` exists and validates event payload families by event type.
7. `schemas/clarity-spec.cue` exists and is the generated spec schema. Legacy `schemas/intent.cue` may exist only as compatibility material, not as the canonical Clarity spec.
8. The current CLI implementation gap is acknowledged and decomposed; existing thin commands are not mistaken for the target command surface.
9. The current redb implementation gap is acknowledged and decomposed; redb is not mistaken for the target Fjall store.
10. Moon or project-approved validation tasks exist for schema vetting.

If any item lacks evidence, decomposition shall create prerequisite repair beads before dependent implementation beads. It shall not reduce product scope.

### 31.3 Readiness Verdict Values

Allowed decomposition readiness values:

1. `decomposition_ready: false` — doc/source/schema blockers prevent honest decomposition.
2. `decomposition_ready: repair-prereqs` — generate only prerequisite repair beads needed to restore the master-doc contract.
3. `decomposition_ready: full-product-dag` — generate the full implementation DAG including AI, reviewers, artifacts, bead generation, storage, CLI, security, recovery, and optional bd emission.

Separate implementation readiness values:

1. `implementation_complete: false` — implementation is incomplete or unproven.
2. `implementation_complete: partial` — some waves are implemented and proven, but the full `SpecComplete` contract is not accepted.
3. `implementation_complete: true` — all product behavior is implemented, tested, reviewed, and accepted with evidence.

The header may say `decomposition_ready: full-product-dag` while `implementation_complete: false`. That is the normal state for a master doc before full AI implementation work begins.

## 32. Bead Decomposition Contract

Generated beads shall be molecular. Oversized beads are a spec violation.

### 32.1 One-Behavior Rule

Each bead shall implement exactly one externally observable behavior or one pure foundation contract.

Forbidden bead combinations:

1. Schema reconciliation plus Fjall storage implementation.
2. CLI command surface plus AI provider implementation.
3. Reviewer orchestration plus artifact generation.
4. bd emission plus core `SpecComplete` foundation.
5. Security redaction plus unrelated command behavior.
6. Multiple Rust profiles in one behavior bead unless the bead is only defining the shared profile enum.

### 32.2 Size Limits

Each bead shall declare:

1. Maximum touched production files.
2. Maximum touched test files.
3. Maximum estimated effort.
4. Allowed modules/crates.
5. Explicit non-goals.

Default limits:

| Bead class | Max production files | Max test files | Max effort |
|---|---:|---:|---|
| schema-only | 2 | 2 | S |
| pure domain type/reducer | 3 | 3 | S/M |
| storage shell behavior | 4 | 4 | M |
| CLI command behavior | 4 | 4 | M |
| AI/reviewer shell behavior | 4 | 4 | M |
| artifact generation behavior | 4 | 4 | M |
| bd emission behavior | 4 | 4 | M |

Any bead exceeding these limits requires decomposition into more molecular beads before implementation. This decomposes delivery; it does not reduce product scope.

### 32.3 Required Bead Sections

Every enhanced bead shall include:

1. `canonical_task_id`.
2. `local_bead_id`.
3. Source master-doc section references.
4. Preconditions.
5. Postconditions.
6. Type-level invariants.
7. Error variants touched.
8. Event types touched.
9. Valid source states and resulting states.
10. Acceptance tests to write first.
11. Verification lanes required or explicitly waived with rationale.
12. Moon commands required for evidence.
13. Files/modules allowed to touch.
14. Security/privacy considerations.
15. Recovery/crash cases when side effects are involved.
16. Completion evidence required.

### 32.4 Tests and Verification Defaults

Every implementation bead shall require tests first.

Default Rust verification expectations:

1. Pure reducers and domain types: unit tests plus property tests where state space is meaningful.
2. Event replay/idempotency/hash code: property tests and corruption tests.
3. Storage shell: integration tests with temp database and crash/reopen scenarios.
4. CLI shell: integration tests for stdout/stderr/JSON/exit codes.
5. Async/provider behavior: deterministic fake provider tests; Loom only when shared concurrency or cancellation semantics justify it.
6. Security/redaction/path code: adversarial tests for traversal, symlink escape, and representative secret formats.
7. bd emission: fake `bd` command tests for existing/missing/collision/partial failure cases.

All beads shall use Moon commands only for build/test/lint evidence.

### 32.5 SpecComplete Boundary

No bead may make `bd` emission a prerequisite for `SpecComplete`.

No bead may treat sanitized JSONL as canonical.

No bead may allow AI/reviewer output to pass without schema and deterministic evidence validation.

No bead may silently weaken a hard gate to make tests easier.

## 33. Implementation Dependency DAG

The planner shall preserve this dependency order unless a reviewer explicitly approves a narrower independent slice.

### 33.1 Foundation Layer

1. Schema manifest and schema reconciliation.
2. Rust profile enum and schema parity.
3. Safe newtypes: session IDs, event IDs, hashes, paths, profiles, gates, artifacts, local bead IDs.
4. Error taxonomy enum and exit-code mapping.
5. Event envelope, closed payload schemas, canonical hashing, and idempotency keys.
6. Pure state reducer and transition tests.
7. Hard-gate pure evaluators and gate result model.
8. Reviewer report model and deterministic evidence validator.

### 33.2 Persistence Layer

9. Fjall keyspace setup.
10. Event append/replay.
11. Derived indexes.
12. Lock protocol.
13. Snapshot creation/validation/discard.
14. Projection status and sanitized JSONL projection.
15. Crash recovery and ambiguous effect detection.

### 33.3 Product Logic Layer

16. Question bank loading and strengthening-only local overrides.
17. Double Diamond question flow.
18. PME adapters: VORP, failure categories, human limitations, CDI evidence, NFR tradeoffs.
19. KIRK16 compiler.
20. CUE spec compiler/validator.
21. Enhanced bead generator/validator.

### 33.4 Shell Layer

22. CLI parser and global JSON/error envelope.
23. `interview start/resume/status/abort/export` commands.
24. `gates run` command.
25. `spec compile` command.
26. `beads generate` command.
27. Provider trait and deterministic fake provider.
28. OpenCode provider shell.
29. Six-reviewer orchestration.
30. Provider prompt minimization and secret scan.
31. Safe export writer.

### 33.5 Downstream Optional Layer

32. bd privacy consent flow.
33. bd scan/idempotency mapping.
34. bd create request/terminal event workflow.
35. bd partial failure recovery.

### 33.6 Prohibited Dependency Inversions

1. Domain reducer shall not depend on Fjall, OpenCode, CUE command execution, bd, stdout, or filesystem paths as raw strings.
2. Gate evaluators shall not call AI providers.
3. Reviewer evidence validation shall not parse human stdout.
4. CLI commands shall not mutate state except through reducer decisions and event append APIs.
5. bd emission shall not import core interview internals except through validated artifact APIs.
6. JSONL projection shall not be read by the reducer.
