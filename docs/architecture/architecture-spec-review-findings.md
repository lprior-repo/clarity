# Architecture Spec Review Findings

Date: 2026-06-20

Scope: `architecture-spec.md`

Verdict: Not ready for `arch-spec-to-beads` until the blockers below are resolved.

## Critical Blockers

1. The current enhanced bead CUE schema does not evaluate. `schemas/enhanced-bead.cue` references `strings.MinRunes` without importing `strings`.
2. `schemas/enhanced-bead.cue` still requires bead IDs matching `intent-cli-*`, which conflicts with Clarity/Rust-only identity.
3. The spec defines Rust-specific profiles, but current CUE schemas define generic profiles: `api`, `cli`, `event`, `data`, `workflow`, `ui`, and `common`.
4. `schemas/kirk.cue` does not define `#KirkContract16`, even though the spec requires KIRK16 schema validation.
5. The current CLI binary exposes only the thin commands: `extract`, `quality`, `validate-straw-man`, `validate-vorp`, `validate-holes`, and `status`; the spec command surface is not implemented.
6. The current storage code uses redb tables, while the spec now requires Fjall keyspaces and event-sourced state.
7. The event schema lacks payload schemas for AI effect correlation, reviewer lifecycle, bd create requests, projection writes, and recovery.
8. The state machine lacks exact reducer semantics for stuck cases: reviewers pass but gates fail, reviewer fails with no repair questions, EOF/SIGINT, and crashes during artifact generation.
9. The reviewer trust model needs deterministic evidence validation. Schema-valid reviewer output with irrelevant evidence IDs must not pass.
10. bd idempotency needs a stable content hash and external-effect request event before `bd create`.

## Security Blockers

1. Raw transcript data is plaintext for MVP. The spec must explicitly accept this risk or move encryption-at-rest into the foundation.
2. AI provider prompts need a pre-provider secret scan and prompt-minimization policy.
3. `bd` emission can leak full enhanced bead content and needs explicit privacy consent plus redaction.
4. Local CUE schemas and project-local overrides are poisonable unless canonical schema hashes are pinned or trusted through a manifest.
5. Project-local question overrides can add secret-harvesting questions unless override validation includes forbidden-question policies.
6. All file paths, session IDs, artifact IDs, and export destinations need safe newtypes and no-follow/canonical-root validation.

## Decomposition Blockers

1. Add a decomposition contract before bead generation: max effort, max touched files, one behavior per bead, and required test evidence.
2. Add an explicit dependency DAG for schema reconciliation, domain types, event model, Fjall store, locks, snapshots, CLI, AI provider, reviewers, gates, artifacts, and bd emission.
3. Split PME integration into separate adapter contracts: VORP, failure categories, human limitations, CDI evidence, NFR tradeoffs.
4. Split AI work into pure provider traits and shell-specific OpenCode implementation.
5. Split reviewer work into schema validation, prompt construction, evidence validation, orchestration, and repair-question flow.
6. Keep bd emission downstream and out of core `SpecComplete` foundation beads.

## Required Spec Fixes

1. Define actual CUE schemas for Rust profiles, KIRK16, reviewer reports, event payloads, and Clarity enhanced beads.
2. Fix `schemas/enhanced-bead.cue` so it evaluates and uses Clarity IDs.
3. Add a command/state/event reducer table.
4. Add a crash-recovery matrix for AI calls, reviewer calls, projection writes, artifact writes, and bd emission.
5. Add exact CLI stdout/stderr/JSON/exit-code contracts.
6. Add exact sanitizer policy for JSONL, provider prompts, raw export, and bd emission.
7. Add Fjall keyspace definitions, key encodings, batch boundaries, persistence mode, and one-process-per-database behavior.
