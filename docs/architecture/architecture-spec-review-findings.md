# Architecture Spec Review Findings

Date: 2026-06-20

Scope: Historical review of `architecture-spec.md`. The current source of truth is now top-level `MASTER_DOC.md`; `architecture-spec.md` is only a compatibility pointer.

Current status: the master doc now contains normative contracts addressing the doc-level gaps identified below. The first blocker-repair pass added canonical CUE schemas, a schema manifest, the target CLI command hierarchy, and a Fjall event-store foundation. `MASTER_DOC.md` is now full-scope and ready for full-product DAG decomposition; implementation completion still requires the full reducer/gate/artifact/bd behavior to be built and proven.

## 2026-06-21 Repair Evidence

Resolved or materially advanced:

1. `schemas/enhanced-bead.cue` now evaluates and uses Clarity bead IDs.
2. `schemas/questions.cue` now exposes Rust profiles: `rust-cli`, `rust-library`, `rust-web-service`, `rust-async-service`, `rust-storage`, `rust-ui`, `rust-refactor`.
3. `schemas/kirk.cue` now defines `#KirkContract16`, `#KirkSection`, and `#KirkMetadata`.
4. Added `schemas/reviewer-report.cue`, `schemas/events.cue`, `schemas/clarity-spec.cue`, and `schemas/manifest.json`.
5. Added Moon schema validation task: `moon run :schema-vet`.
6. Replaced the old top-level thin CLI command-only surface with the target `interview`, `gates`, `spec`, `beads`, and `sessions` hierarchy.
7. Added canonical Fjall event-store foundation in `clarity-web/src/storage/fjall_event_store.rs`; redb remains legacy UI/transcript storage.

Validation evidence:

1. `moon run :schema-vet` passes.
2. `moon run :fmt` passes after formatting.
3. `moon run :check` passes.
4. `moon run :test-unit` passes.
5. CLI smoke: `clarity-web --json interview start --profile rust-cli` writes a Fjall event and `interview status` reads it back.

Known remaining validation debt:

1. `moon run :clippy` still fails on pre-existing legacy server/test lint debt; tracked as bead `cl-2q6`.

Verdict: `MASTER_DOC.md` is approved as the source-of-truth PRD/spec. The review loop is closed for doc-level readiness and approved for `decomposition_ready: full-product-dag`. Do not claim `implementation_complete: true` until the implementation-depth blockers below are completed.

## Original Findings Disposition

| Original finding | Current disposition |
|---|---|
| Enhanced bead CUE schema did not evaluate because `strings.MinRunes` lacked import | **Closed.** `schemas/enhanced-bead.cue` imports `strings`; `moon run :schema-vet` passes. |
| Enhanced bead IDs required `intent-cli-*` | **Closed.** `schemas/enhanced-bead.cue` now requires Clarity IDs. |
| Question profiles were generic (`api`, `cli`, `event`, `data`, `workflow`, `ui`, `common`) | **Closed for canonical schema.** `schemas/questions.cue` exposes the seven Rust profiles from `MASTER_DOC.md`. |
| `schemas/kirk.cue` lacked `#KirkContract16` | **Closed.** `#KirkContract16`, `#KirkSection`, and `#KirkMetadata` exist. |
| CLI exposed only thin legacy commands | **Foundation-only closed.** The target command hierarchy exists and `interview start/status` use Fjall. Full reducer/gate/spec/bead/export/bd behavior remains a full-readiness implementation blocker. |
| Storage used redb instead of Fjall | **Foundation-only closed.** `FjallEventStore` exists and the CLI uses it for canonical interview events. Full event hash chain, sequence enforcement, lock protocol, snapshots, derived indexes, schema validation, and recovery semantics remain full-readiness blockers. |
| Event schema lacked AI/reviewer/bd/projection/recovery payloads | **Foundation-only closed.** `schemas/events.cue` exists, vets, and discriminates payload families by event type. Full semantic parity and production replay enforcement remain implementation blockers. |
| State machine lacked exact reducer semantics | **Closed at MASTER_DOC level.** See `MASTER_DOC.md` reducer/state sections. Implementation remains pending. |
| Reviewer trust model needed deterministic evidence validation | **Closed at MASTER_DOC/schema level.** `schemas/reviewer-report.cue` exists and `MASTER_DOC.md` defines deterministic evidence validation. Implementation remains pending. |
| bd idempotency needed stable content hash and request event | **Closed at MASTER_DOC/events-schema level.** Implementation remains pending. |
| Plaintext raw transcript risk was ambiguous | **Closed at MASTER_DOC level.** Initial full-product plaintext risk is explicitly accepted with constraints. |
| Provider prompts needed minimization and secret scan | **Closed at MASTER_DOC level.** Implementation remains pending. |
| bd emission needed privacy consent and redaction | **Closed at MASTER_DOC level.** Implementation remains pending. |
| Local schema/question overrides needed trust/forbidden-question policy | **Closed at MASTER_DOC level and schema manifest exists.** Implementation remains pending. |
| Safe newtypes/path validation needed specification | **Closed at MASTER_DOC level.** Implementation remains pending. |
| Decomposition contract and dependency DAG missing | **Closed.** `MASTER_DOC.md` defines bead decomposition contract and implementation DAG. |
| PME/AI/reviewer work needed adapter/decomposition boundaries | **Closed at MASTER_DOC level.** Implementation remains pending. |

## Remaining Full-Readiness Blockers

These do not block full-product DAG decomposition, but they do block claiming `implementation_complete: true`:

1. Implement real reducer-backed behavior for `gates run`, `spec compile`, `beads generate`, `beads emit`, `sessions list`, and export.
2. Complete Fjall event-store contract: hash chain, schema validation on append/replay, contiguous sequence enforcement, session lock protocol, snapshots, derived indexes, projection status, and recovery matrix behavior.
3. Tighten schemas to complete semantic parity with every `MASTER_DOC.md` required field, including full enhanced bead decomposition fields.
4. Implement reviewer evidence validation, prompt minimization, secret scan, redaction, local override trust checks, and bd idempotency behavior.
5. Resolve repo-wide clippy debt tracked as bead `cl-2q6`.
