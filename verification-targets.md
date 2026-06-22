# Verification Targets — Module-to-Lane Roadmap

**Date:** 2026-06-21
**Audience:** `proof-planner` (primary), `rust-contract` (input), `proof-writer` (consumer)
**Companion files:** `formal-verification-report.md`, `verification-ledger.jsonl`
**Scope:** `clarity-web/src/` — 238 files, 96,428 LOC, 25+ submodules

---

## 0. What this document is — and is not

This is a **targeting plan**. It classifies each module by the verification lane (or
absence of lane) that is appropriate, and explains why. It does **not**:

- Write or approve any proof obligation.
- Replace the contract work owned by `rust-contract`.
- Replace the plan-review work owned by `proof-plan-reviewer`.
- Replace the artifact-writing work owned by `proof-writer`.
- Replace the bridge work owned by `proof-to-implementation`.
- Replace the test-plan work owned by `test-planner` / `test-writer`.
- Replace the evidence-packaging work owned by `evidence-packaging`.

This is a roadmap, not a closure.

## 1. Why "every line has a proof" is the wrong target

A literal per-line refinement obligation set for 96K LOC of UI rendering, CLI plumbing,
serde round-trips, and integration test code is mathematically and operationally
infeasible. The right target is **behavior-affecting functions** selected by risk,
not by line count. The skill spec says exactly this:

> *"Default Rust behavior to Verus/Kani/Flux/proptest, add Loom/fuzz only by risk
> profile, and writes machine-readable obligations only."*

Concretely: a `serde_json::to_string` call is not behavior-affecting — it is exercised
by integration tests and is the library's contract, not ours. A `ServerFn` boundary
that turns user input into a stored entity IS behavior-affecting and warrants a refinement
spec. Targeting the former inflates the ledger with obligations that test nothing the
type system doesn't already check.

## 2. Lane legend

| Lane | Tool | When to pick it |
|------|------|-----------------|
| **V** | Verus | Pure functions, invariants on data, arithmetic-safe code, typestate-style contracts, no I/O. |
| **K** | Kani | Bounded model check of unsafe-adjacent code, fixed-width arithmetic, parser bounds, hash collisions, integer overflow in tight loops. |
| **F** | Flux | Refinement types layered onto existing pure functions without rewriting them; lighter-weight than Verus. |
| **P** | proptest / quickcheck | Round-trip properties, parser/serializer parity, idempotence, monoid laws. |
| **Z** | fuzz (cargo-fuzz) | Untrusted-input parsers, format decoders, regex hot paths. |
| **T** | TLA+ | Temporal workflows: bead lifecycles, retries, leases, batch ordering, lease renewal. |
| **L** | Loom | Concurrent code with `Send + Sync` interactions, channel ordering, spawn/join correctness. |
| **M** | Miri | `unsafe` blocks (none currently — `unsafe_code = "forbid"` in workspace). |
| **X** | exercise-only (no proof) | Glue code, serde boundaries, Dioxus component shells, CLI argument dispatch, config parsing. Test-covered, not proof-covered. |
| **—** | out of scope | Generated code, integration test fixtures, dev-only modules. |

A module may carry multiple lanes. The first letter is the *primary* lane; subscripts
mark secondary lanes applied to specific functions inside the module.

## 3. Tooling prerequisites

| Tool | Installed | Required before |
|------|:---------:|-----------------|
| `tlc` | ✅ `/home/lewis/.local/share/mise/installs/http-tla2tools/1.7.4/tlc` | Lane **T** |
| `verus` | ✅ `/home/lewis/.local/bin/verus` | Lane **V** |
| `cargo-flux` | ✅ `/home/lewis/.cargo/bin/cargo-flux` | Lane **F** |
| `kani` | ❌ not installed | Lane **K** — install before any Kani obligation is written |
| `apalache` | ❌ not installed | Lane **T** sym. model-check — install for Apalache cross-check on TLA+ specs |
| `cargo-fuzz` | ❌ not installed | Lane **Z** — install before any fuzz obligation is written |

Two gaps must close before the relevant lanes are honest. **Kani** install:
`cargo install --locked kani --version <current> --features all`. **Apalache**: download
the prebuilt jar from the official release; pin via `.mise.toml`.

## 4. Pre-flight gate

Before any proof work begins in this roadmap:

| Gate | Owner | Why |
|------|-------|-----|
| Close `cl-2q6` (clippy debt, see `formal-verification-report.md`) | `holzman-rust` + `landing-skill` | `cargo clippy` must be clean under `-D warnings` for any verifier invocation to be stable. |
| Re-run `cargo test --workspace --all-features` and capture PASS/FAIL into the ledger | `formal-verifier` | Baseline behavior coverage before adding new obligations on top. |
| Install missing tools (§3) | infra | Honest lane coverage requires honest tool availability. |
| Author `rust-contract` artifacts for the first module | `rust-contract` | Required input to `proof-planner`. |
| Author `proof-obligations.planned.jsonl` for the first module | `proof-planner` | Required for the verifier to know what to execute. |
| Review the plan | `proof-plan-reviewer` | Adversarial review before any proof is written. |

## 5. Module-by-module roadmap

### 5.1 `clarity-web/src/domain/` — Pure domain logic

**LOC:** ~3,000 across 7 files. Highest verification value in the crate.

| File | LOC | Primary lane | Secondary | Rationale |
|------|----:|--------------|-----------|-----------|
| `domain/newtypes.rs` | ~270 | **V** | P | Newtype validators (`NonEmptyString`, `SessionId`, etc.) are pure and security-critical. Verus specs pin validators; proptest covers the boundary. |
| `domain/types.rs` | ~30  | **V** | — | Tiny pure types; fast to spec. |
| `domain/error.rs` | ~20  | **V** | — | Railway error taxonomy; spec the `From` graph. |
| `domain/scenario.rs` | ~750 | **V** + **P** | F | Scenario state machine. Verus for transitions, proptest for round-trip on serialization. |
| `domain/quality.rs` | ~20  | **F** | — | Quality score wrapper, lightweight refinement. |
| `domain/straw_man.rs` | ~350 | **V** + **P** | — | Adversarial argument model; pure logic. |

**Recommended first obligation:** `domain/newtypes.rs` — smallest, highest signal,
cleanest typestate mapping.

### 5.2 `clarity-web/src/storage/` — Persistence

**LOC:** ~8,500 across 8 files. Persistence invariants are the highest-impact proof target.

| File | LOC | Primary lane | Secondary | Rationale |
|------|----:|--------------|-----------|-----------|
| `storage/types.rs` | ~440 | **V** | — | Storage value types; refine. |
| `storage/fjall_event_store.rs` | ~210 | **V** + **K** | T | Event store over Fjall LSM; Verus for write-batching invariants, Kani for key-space arithmetic, TLA+ for crash-recovery ordering (separate spec). |
| `storage/redb_store.rs` | ~510 | **V** + **K** | — | redb transactional store; Kani for transaction-bound arithmetic. |
| `storage/redb_transcript_store.rs` | ~530 | **V** + **K** | — | Transcript-specific redb store. |
| `storage/transcript_store.rs` | ~520 | **V** | P | Transcript trait surface; spec the trait object. |
| `storage/path_util.rs` | ~620 | **F** + **P** | Z | Path normalization; Flux refines "no traversal escape", proptest for fuzzer-style cases, fuzz for adversarial paths. |
| `storage/integration_test.rs` | ~370 | **X** | — | Integration tests, not a proof target. |
| `storage/mod.rs` | ~50  | **X** | — | Re-exports. |

**TLA+ spec candidate:** `storage/fjall_event_store.rs` — write-batch ack, recovery,
snapshot-consistency temporal property. Owned by `tla-plus` once contracts land.

### 5.3 `clarity-web/src/intent/` — DSL parsing & validation

**LOC:** ~30,000 across 14 submodules. Mixed: parser logic (P/Z), types (V), validation (V/P).

| File / subdir | LOC | Primary lane | Secondary | Rationale |
|---------------|----:|--------------|-----------|-----------|
| `intent/parser.rs` | ~630 | **Z** + **P** | V | The DSL parser. Fuzz the front door, proptest the round-trip, Verus the AST invariant (`typed != Err`). **Clippy hotspot** (3 `unneeded_wildcard_pattern` in tests). |
| `intent/formats.rs` | ~450 | **P** | F | Format round-trip properties. |
| `intent/errors.rs` | ~410 | **V** | — | Error taxonomy. |
| `intent/types/` (8 files) | ~2,500 | **V** | P | Behavior, feature, spec, invariant, names, type_error, anti_pattern, verification — the type algebra of the DSL. Verus the algebra; proptest the surface. |
| `intent/validation/` (5 files) | ~3,800 | **V** + **P** | — | Rule + semantic + spec validator. Pure enough to spec. |
| `intent/quality/` (8 files) | ~7,500 | **P** + **Z** | V | Linter, analyzer, effects, improver. Pure-ish. Proptest for property-based lint discovery; fuzz for parser-feeding. |
| `intent/templates/` (3 files) | ~3,200 | **P** | V | Template generation; proptest for substitution, Verus for the generator contract. |
| `intent/batch/` | ~? | **V** + **T** | — | Batch processor. TLA+ spec for ordering + retry. |
| `intent/beads/` | ~? | **V** + **T** | P | Bead lifecycle, templates, feedback service. TLA+ for state machine. |
| `intent/interview/` | ~4,000 | **P** | F | Answer extraction, answer file — proptest for extraction properties. |
| `intent/loader.rs`, `mod.rs`, `security.rs`, `types.rs` | small | **X** | — | Glue. |

### 5.4 `clarity-web/src/lattice/` — Cross-cutting analysis

| File | LOC | Primary lane | Secondary | Rationale |
|------|----:|--------------|-----------|-----------|
| `lattice/quality.rs` | ~620 | **V** + **P** | — | Quality dimensions; spec the scoring algebra. |
| `lattice/effects.rs` | ~450 | **V** | P | Effect tracking. **Clippy hotspot** (1 `uninlined_format_args`). |
| `lattice/gap_detection.rs` | ~360 | **V** + **P** | — | Gap detector. **Clippy hotspot**. |
| `lattice/conflict_detection.rs` | ~460 | **V** + **P** | — | Conflict detector. |
| `lattice/quality_dimensions.rs` | ~450 | **V** | P | Dimensions algebra. |

### 5.5 `clarity-web/src/kirk/`

| File | LOC | Primary lane | Secondary | Rationale |
|------|----:|--------------|-----------|-----------|
| `kirk/types.rs` | ~400 | **V** | — | Pure type definitions. |
| `kirk/terminal_integration.rs` | ~610 | **V** + **P** | — | Terminal integration; spec the interface. **Clippy hotspot** (1 `significant_drop_in_scrutinee` — the only correctness-tier clippy failure in the baseline). |

### 5.6 `clarity-web/src/pme/` — Project memory engine

| File | LOC | Primary lane | Secondary | Rationale |
|------|----:|--------------|-----------|-----------|
| `pme/define/brutal_truths.rs` | ~425 | **V** + **P** | — | Adversarial prompts — pure. |
| `pme/define/great_reindexing.rs` | ~480 | **P** | V | Round-trip + algebraic specs. |
| `pme/discover/cdi_logger.rs` | ~445 | **V** | P | Pure logger logic. |
| `pme/discover/north_star.rs` | ~480 | **P** | — | Discovery — property-based. |
| `pme/discover/persona_forge.rs` | ~490 | **P** | V | Persona construction. |
| `pme/discover/thesis_generator.rs` | ~440 | **P** | — | Generator. |
| `pme/infra/logging.rs` | ~445 | **X** | — | Infrastructure glue. |
| `pme/infra/metrics.rs` | ~535 | **F** | — | Metric dimensions; lightweight refinement. |
| `pme/infra/testing.rs` | ~510 | **X** | — | Test infrastructure. |
| `pme/infra/tracing.rs` | ~485 | **X** | — | Tracing setup. |

### 5.7 `clarity-web/src/config/`

| File | LOC | Primary lane | Secondary | Rationale |
|------|----:|--------------|-----------|-----------|
| `config/ai.rs` | ~570 | **V** + **P** | — | AI config schema; pure. |

### 5.8 `clarity-web/src/server.rs` and `clarity-web/src/intent/cli/`

| File | LOC | Primary lane | Secondary | Rationale |
|------|----:|--------------|-----------|-----------|
| `server.rs` | 2,682 | **L** + **T** | — | The Dioxus server boundary. **Clippy hotspot** (53 errors, mostly inside `#[cfg(test)] mod tests`). Real verification target is the `ServerFn` boundary contract, not the test body. Loom for concurrent handler dispatch; TLA+ for request lifecycle. |
| `intent/cli/*.rs` | ~600 | **X** | P | CLI argument parsing — exercise-only with proptest on flag validation. |

### 5.9 `clarity-web/src/providers/`

Treat as **X** (exercise-only) unless it contains a verified-internal algorithm. Verify on inspection.

### 5.10 `e2e-tests/`

**Out of scope** for proof lanes — these are integration tests against a running server.

## 6. Out-of-scope modules

| Path | Reason |
|------|--------|
| `target/` | build artifacts. |
| `e2e-tests/` | integration tests; covered by `hands-on-qa`, not proofs. |
| Generated files (e.g. SQLx `.sqlx/` offline data) | not source. |
| `node_modules/`, `playwright-report/`, `playwright-tests/` | non-Rust. |

## 7. Recommended phasing

A real, shippable verification program runs one module at a time through the
go-skill lifecycle:

```
Phase 0 — preflight
   - Close cl-2q6 (clippy)
   - Install kani, apalache, cargo-fuzz
   - Establish cargo test PASS baseline

Phase 1 — domain seed (smallest, highest signal)
   1.1 rust-contract for domain/newtypes.rs
   1.2 proof-planner → proof-obligations.planned.jsonl
   1.3 proof-plan-reviewer
   1.4 proof-writer (Verus + proptest)
   1.5 proof-reviewer
   1.6 proof-to-implementation (bridge)
   1.7 test-planner / test-writer / test-reviewer
   1.8 formal-verifier (closes obligations in ledger)
   1.9 evidence-packaging + landing-skill

Phase 2 — domain expansion
   domain/scenario.rs, domain/straw_man.rs, domain/types.rs

Phase 3 — storage
   storage/newtypes, storage/fjall_event_store.rs (+ TLA+ spec),
   storage/redb_store.rs

Phase 4 — intent/types + intent/parser
   intent/types/* (Verus), intent/parser.rs (fuzz + proptest)

Phase 5 — intent/quality/* + intent/validation/*
   proptest-heavy; covers the largest LOC volume.

Phase 6 — server.rs ServerFn boundary
   Loom for concurrent handler dispatch; TLA+ for request lifecycle.

Phase 7 — remaining intent submodules + pme + lattice + kirk
   iterate as time permits; each module follows the same lifecycle.
```

Each phase ships with its own `verification-ledger.jsonl` rows and an
`evidence-packaging` bundle. The ledger is append-only; earlier phases are
not re-classified.

## 8. Honest non-targets

The following are **not** in this verification program:

- **Line-by-line proofs.** Refused. The cost-benefit ratio is wrong and the
  proof obligations would be vacuous for glue code.
- **UI rendering proofs.** Dioxus components are exercised by Playwright E2E
  tests, not refinement proofs.
- **Pure serde round-trips.** Covered by proptest properties, not Verus.
- **The test bodies themselves** (e.g. the 53 clippy errors in `server.rs`'s
  `#[cfg(test)]` module). Tests get exercise coverage; they don't get
  refinement specs. (They do, however, need to pass `cargo clippy`.)

## 9. Cross-references

- `formal-verification-report.md` — clippy baseline + toolchain state + non-targets.
- `verification-ledger.jsonl` — current ledger (clippy baseline only, no closure rows).
- `Cargo.toml` — workspace lint configuration (`unwrap_used`, `expect_used`, `panic`,
  `todo`, `unimplemented` all `deny`).
- `.moon/tasks/rust.yml` — `clippy` task definition.
- `cl-2q6` — current blocker bead.
- `cl-dtd`, `cl-06d`, `cl-i6u`, `cl-7np` — adjacent open beads (drift, QA,
  blackhat); verification pipeline must coexist with them but does not block
  them once clippy is closed.

---

## 10. Update 2026-06-21 — verification program in flight

**CRITICAL FINDING (cl-2vr, P0):** `clarity-web/Cargo.toml` has no `[lints]` opt-in.
The workspace `[workspace.lints.clippy]` deny rules for `unwrap_used`,
`expect_used`, `panic`, `todo`, `unimplemented` are **inert** for `clarity-web`.
Empirical inventory:

| Family | Sites |
|--------|------:|
| `.unwrap()` | 316 |
| `.expect(` | 497 |
| `panic!` | 86 |
| `todo!` | 1 |
| `unimplemented!` | 1 |
| **Total panic-prone** | **901** |

This invalidates any Verus/proptest obligation that claims "no panic" on a
production function. The 8 module plans (cl-0n6 / cl-zup / cl-5dp / cl-ooz /
cl-vv2 / cl-kse / cl-54n / cl-dv5) implicitly assume panic-free source paths in
their INFERRED clauses. Re-validate those assumptions after `cl-2vr` closes.

### 10.1 Artifacts produced in this session (proofs/)

| Bead | Module | Plan | Proof artifacts |
|------|--------|------|-----------------|
| `cl-0n6` | `domain/newtypes.rs` | `proofs/newtypes-proof-plan.md` (41 obligations) | — |
| `cl-zup` | `domain/scenario.rs` | `proofs/scenario-proof-plan.md` (40 obligations) | — |
| `cl-5dp` | `storage/types.rs` | `proofs/storage-types-proof-plan.md` (22 obligations) | — |
| `cl-ooz` | `intent/types/behavior.rs` | `proofs/behavior-proof-plan.md` (21 obligations) | — |
| `cl-vv2` | `domain/straw_man.rs` | — | `proofs/straw_man_{verus,proptest,writeup}.{rs,md}` — 9 specs, 21 props |
| `cl-kse` | `storage/fjall_event_store.rs` | — | `proofs/fjall_event_store_{verus,kani,writeup}.{rs,md}` + trusted-base ledger — 16 specs, 10 proofs, 9 harnesses |
| `cl-54n` | `intent/parser.rs` | — | `proofs/parser_{proptest,fuzz,writeup}.{rs,md}` — 15 proptest cases, 1 fuzz target |
| `cl-dv5` | `lattice/quality.rs` | — | `proofs/lattice_quality_{verus,proptest,writeup}.{rs,md}` — 13 spec fns, 14 lemmas, 27 props |

### 10.2 Verifier runs captured

| Bead | Spec | Command | Exit | Verdict |
|------|------|---------|-----:|---------|
| `cl-vv2` | `straw_man_verus.rs` | `verus proofs/straw_man_verus.rs` | 1 | **FAIL_LOCAL** — `slice::contains` not in vstd |
| `cl-dv5` | `lattice_quality_verus.rs` | `verus proofs/lattice_quality_verus.rs` | 1 | **FAIL_LOCAL** — `pub spec enum` invalid syntax |
| `cl-kse` | `fjall_event_store_verus.rs` | `verus proofs/fjall_event_store_verus.rs` | 1 | **FAIL_LOCAL** — `Seq<char>` vs `Seq<u8>` mismatch |

All three have repair beads (`cl-ev9`, `cl-28r`, `cl-55u`).

### 10.3 Outstanding execution blocked on tooling/wiring

| Bead | Lane | Blocked on |
|------|------|-----------|
| `cl-vv2` (proptest) | P | wiring into `clarity-web/tests/` |
| `cl-54n` (proptest + fuzz) | P, Z | wiring + `cl-u04` (cargo-fuzz) |
| `cl-dv5` (proptest) | P | wiring |
| `cl-kse` (Kani) | K | `cl-u04` (kani) |
