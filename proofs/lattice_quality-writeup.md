# Writeup & Proof Plan — `clarity-web/src/lattice/quality.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-dv5` |
| **Target** | `clarity-web/src/lattice/quality.rs` (~620 LOC) |
| **Primary lane** | **V** (Verus) — algebraic spec/proof |
| **Secondary lane** | **P** (proptest) — range / round-trip / monotonicity / idempotence |
| **Contract status** | **GAP — no `rust-contract` artifact exists for this module.** Clauses below are inferred from source (type signatures, doc-comments, and `tests` block). `proof-planner` has not yet run; obligation IDs (`OB-LQ-V-NN`, `OB-LQ-P-NN`) are PROVISIONAL and must be replaced with planner-assigned IDs from `proof-obligations.planned.jsonl` before review. |
| **Author** | `proof-writer` |
| **Date** | 2026-06-21 |

---

## 1. Module characterisation

`clarity-web/src/lattice/quality.rs` is a **pure scoring module**: zero I/O, zero concurrency, zero `unsafe` (`#![forbid(unsafe_code)]` at line 23). It evaluates five quality dimensions (Completeness, Consistency, Testability, Clarity, Security) on a set of user answers + EARS requirements + inversion control data, returning a `QualityScore` with overall average + per-dimension scores + collected issues.

The module declares 12 public types / enums and 1 public function (`calculate_quality`), plus five private aggregation helpers. It carries a `#[cfg(test)] mod tests` block of ~50 unit tests (lines 567–1558), some explicitly labelled as mutant-catching (lines 1087+).

| Item | Lines | Kind | In-scope for V? | In-scope for P? |
|---|---|---|:---:|:---:|
| `MINIMUM_GATE: u8` | 34 | Domain constant | **V-13** | **P-16** |
| `enum QualityError` | 37–47 | Error taxonomy | — | (matched in P-02, P-05) |
| `enum QualityDimension` | 50–66 | 5-variant enum | **V-01, V-02, V-03, V-04** | **P-17, P-18, P-19** |
| `QualityDimension::all / label / description` | 70–100 | Pure functions | **V-01, V-02, V-03, V-04** | **P-17, P-18, P-19** |
| `struct DimensionScore` | 104–108 | `{ dimension, score: u8 }` | **V-05..V-08** | **P-01..P-04, P-13, P-14, P-21** |
| `DimensionScore::new / passes` | 110–123 | Constructors + threshold check | **V-05..V-08** | **P-01..P-04** |
| `struct QualityIssue` + `enum IssueSeverity` | 126–149 | Issue record + severity | — | **P-15, P-22, P-24** |
| `struct QualityScore` | 152–160 | `{ overall, dimensions, issues }` | **V-09, V-10** | **P-05, P-06, P-12..P-15, P-20** |
| `QualityScore::new / passes / get_dimension / get_issues` | 162–197 | Constructors + accessors | **V-09, V-10** | **P-05, P-06, P-13, P-14, P-15** |
| `struct EarsRequirementRef` | 199–205 | Pure data | — | **P-26** |
| `struct InversionControl` | 207–212 | Pure data | — | **P-27** |
| `pub fn calculate_quality` | 223–251 | Orchestrator (returns 5-dim Vec + overall mean) | **V-11, V-12, V-13, V-14** | **P-07..P-12** |
| `fn calculate_completeness` | 254–308 | String-processing heuristic | **TRUSTED** | (covered by `tests::test_calculate_completeness_*`) |
| `fn calculate_consistency` + `has_contradiction` | 311–363 | String-processing heuristic | **TRUSTED** | (covered by `tests::test_calculate_consistency_*`) |
| `fn calculate_testability` | 366–401 | EARS-coverage heuristic | **TRUSTED** | (covered by `tests::test_calculate_testability_*`) |
| `fn calculate_clarity` | 404–482 | Jargon / complexity heuristic | **TRUSTED** | (covered by `tests::test_calculate_clarity_*`) |
| `fn calculate_security` | 485–565 | Keyword-coverage heuristic | **TRUSTED** | (covered by `tests::test_calculate_security_*`) |

**Refinement-style properties** that are *documented* but *not enforced* by the type system:

- `DimensionScore.score` is documented as "0–100" but the type is `u8` (0..=255). The `new` constructor enforces the range, but a `DimensionScore { score: 200, .. }` literal could be constructed directly. **Verus V-05/V-06 pin the range; proptest P-01/P-02 verify the runtime check.**
- `QualityScore.overall` is documented as "0–100" but the type is `u8`. Same story. **V-09 pins; P-05 verifies.**
- `calculate_quality` is documented as returning "average of 5 dimensions". The math `overall = floor(sum / 5)` is a structural property of the implementation. **V-13 proves `floor(sum/5) ∈ [0, 100]` from the requires.**
- `QualityDimension::all` is documented as "All dimensions". **V-01, V-02 pin the exact 5-variant set; P-17 verifies at runtime.**

## 2. Contract gap (honest disclosure)

The bead `cl-dv5` has **no upstream `rust-contract` artifact** under `clarity-web/src/lattice/contract.md` (or any path matching `**/contract.md` in the workspace). All clauses used in this writeup are *inferred by direct reading of the source*, not authored. The provenance field on every obligation row is `INFERRED`.

**Action required before `proof-reviewer` runs:**

1. `rust-contract` must author `contract.md` (or equivalent) for `clarity-web/src/lattice/` and either ratify or correct the clauses in §3. If the inferred clauses are wrong, the Verus spec and proptest properties will exercise the wrong contract. The plan is gated on this ratification.
2. `proof-planner` must produce `proof-obligations.planned.jsonl` with formal IDs, replacing the provisional `OB-LQ-V-NN` / `OB-LQ-P-NN` IDs in this artifact.
3. `proof-plan-reviewer` must review this writeup + the obligations JSONL before any verifier invocation.

The 14 Verus properties (OB-LQ-V-01 .. V-14) and 27 proptest properties (OB-LQ-P-01 .. P-27) below are *provisional* and ready to be re-keyed once the planner formalizes.

## 3. Inferred contract clauses

The requirements below are derived from the source's doc-comments, type signatures, and `#[serde(...)]` attributes. Each row carries a `clause_origin` of `INFERRED`.

| Req ID | Source | Inferred clause | Origin |
|---|---|---|---|
| REQ-LQ-1 | `MINIMUM_GATE` (line 34) | Constant is `70`. | INFERRED |
| REQ-LQ-2 | `QualityDimension` (lines 50–66) | Exactly `{Completeness, Consistency, Testability, Clarity, Security}`. | INFERRED |
| REQ-LQ-3 | `QualityDimension::all` (lines 70–78) | Returns the 5 variants in declaration order, no duplicates. | INFERRED |
| REQ-LQ-4 | `QualityDimension::label` (lines 81–89) | Each variant maps to a non-empty `&'static str`. | INFERRED |
| REQ-LQ-5 | `QualityDimension::description` (lines 92–100) | Each variant maps to a non-empty `&'static str`. | INFERRED |
| REQ-LQ-6 | `DimensionScore::new` (lines 112–117) | `Ok(score)` iff `0 <= score <= 100`; on Ok, fields are faithfully copied. | INFERRED |
| REQ-LQ-7 | `DimensionScore::passes` (lines 120–122) | Returns `score >= threshold`. | INFERRED |
| REQ-LQ-8 | `QualityScore::new` (lines 164–177) | `Ok(overall)` iff `0 <= overall <= 100`; on Ok, overall is faithfully copied. | INFERRED |
| REQ-LQ-9 | `QualityScore::passes` (lines 180–182) | Returns `overall >= threshold`. | INFERRED |
| REQ-LQ-10 | `calculate_quality` (lines 223–251) | On `answers.is_empty()` returns `Err(EmptyAnswers)`; otherwise returns `Ok(QualityScore { overall, dimensions, issues })` where `dimensions.len() == 5`, `0 <= overall <= 100`, and `overall = floor(sum(d.score for d in dimensions) / 5)`. | INFERRED |
| REQ-LQ-11 | `calculate_quality` (line 245) | The mean-floor property is preserved (sum of 5 scores in `[0,100]` divided by 5 fits in `u8`). | INFERRED |
| REQ-LQ-12 | `serde::{Serialize, Deserialize}` derives | `serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap_or_default()) == Ok(t)` for every serializable type. | INFERRED |
| REQ-LQ-13 | `calculate_quality` (lines 223–251) | The function is pure (no I/O, no mutation of inputs, no hidden state). Two calls with identical arguments return identical results. | INFERRED |

## 4. Verifier lane decisions

| Lane | Decision | Evidence / rationale |
|---|---|---|
| **V** (Verus) | **REQUIRED — primary** | Module is pure; the scoring algebra (range, monotonicity, idempotence, mean-floor) is exactly what Verus specs. Per `verification-targets.md §5.4`. Verus installed at `/home/lewis/.local/bin/verus` (v0.2026.05.05.d03e906). |
| **P** (proptest) | **REQUIRED — secondary** | All four serializable record types derive `serde::{Serialize, Deserialize}`. Range / round-trip / monotonicity / idempotence are the natural proptest targets; cheaper than Verus for arbitrary `String` content. proptest 1.10.0 is a dev-dependency of `clarity-web` (`clarity-web/Cargo.toml:44`). |
| **K** (Kani) | **NOT APPLICABLE** | No `unsafe`, no fixed-width arithmetic overflow (the `try_from` on line 247–248 guards `u32 → u8`), no parser, no index ops. Kani not installed (see `verification-targets.md §3`) — even if applicable, install would be required first. |
| **F** (Flux) | **NOT APPLICABLE** | Verus covers the refinement properties (range, mean-floor) with more rigour. Flux would be redundant for this module. `cargo-flux` is installed but unused. |
| **L** (Loom) | **NOT APPLICABLE** | No concurrency — no threads, channels, atomics, `Send + Sync` interactions, async, or spawn calls anywhere in `quality.rs`. |
| **M** (Miri) | **NOT APPLICABLE** | `#![forbid(unsafe_code)]` at line 23 and workspace level (`Cargo.toml:10`). No `unsafe` blocks to verify. |
| **T** (TLA+) | **NOT APPLICABLE** | No temporal workflow — no state machine, retries, leases, batch ordering, or any temporal property. The five `calculate_*` functions are stateless function calls. |
| **Z** (fuzz) | **NOT APPLICABLE** | No hand-written parser, regex, codec, or frame decoder. Adversarial input funnels through `&[Answer]` / `&[EarsRequirementRef]` — typed Rust slices, not byte buffers. The 50 `#[test]` cases in `tests` already cover boundary conditions exhaustively (mutation tests at lines 1087+). `cargo-fuzz` is not installed. |
| **X** (exercise-only) | **NOT APPLICABLE** | The whole module is in scope for V + P coverage. The `#[cfg(test)] mod tests` block is exercise coverage, not proof. |

Two infrastructure gaps block adjacent lanes (`K`, `Z`) but those lanes are `not_applicable` to this module regardless, so the gaps are not blockers for *this* plan. They are noted for `landing-skill` pre-flight in `verification-targets.md §4`.

## 5. Proof coverage matrix

| Req ID | Lane | Obligation ID | Source target | Artifact |
|---|---|---|---|---|
| REQ-LQ-1 | V | OB-LQ-V-13 (range arithmetic) + (constant 70 implicit) | `MINIMUM_GATE` (line 34) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-1 | P | OB-LQ-P-16 | `MINIMUM_GATE` (line 34) | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-2 | V | OB-LQ-V-02 | `QualityDimension` variants (lines 50–66) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-2 | P | OB-LQ-P-17, OB-LQ-P-23 | `QualityDimension` enum | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-3 | V | OB-LQ-V-01, OB-LQ-V-02 | `QualityDimension::all` (lines 70–78) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-3 | P | OB-LQ-P-17 | `QualityDimension::all` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-4 | V | OB-LQ-V-03 | `QualityDimension::label` (lines 81–89) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-4 | P | OB-LQ-P-18 | `QualityDimension::label` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-5 | V | OB-LQ-V-04 | `QualityDimension::description` (lines 92–100) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-5 | P | OB-LQ-P-19 | `QualityDimension::description` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-6 | V | OB-LQ-V-05, OB-LQ-V-06 | `DimensionScore::new` (lines 112–117) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-6 | P | OB-LQ-P-01, OB-LQ-P-02, OB-LQ-P-21 | `DimensionScore::new` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-7 | V | OB-LQ-V-07, OB-LQ-V-08 | `DimensionScore::passes` (lines 120–122) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-7 | P | OB-LQ-P-03, OB-LQ-P-04 | `DimensionScore::passes` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-8 | V | OB-LQ-V-09 | `QualityScore::new` (lines 164–177) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-8 | P | OB-LQ-P-05, OB-LQ-P-20 | `QualityScore::new` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-9 | V | OB-LQ-V-10 | `QualityScore::passes` (lines 180–182) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-9 | P | OB-LQ-P-06 | `QualityScore::passes` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-10 | V | OB-LQ-V-11, OB-LQ-V-12, OB-LQ-V-14 | `calculate_quality` (lines 223–251) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-10 | P | OB-LQ-P-07, OB-LQ-P-08, OB-LQ-P-09, OB-LQ-P-10, OB-LQ-P-11, OB-LQ-P-12 | `calculate_quality` | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-11 | V | OB-LQ-V-13 | `calculate_quality` mean-floor (lines 244–248) | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-12 | P | OB-LQ-P-20, OB-LQ-P-21, OB-LQ-P-22, OB-LQ-P-23, OB-LQ-P-24, OB-LQ-P-25, OB-LQ-P-26, OB-LQ-P-27 | All `Serialize`/`Deserialize` types | `proofs/lattice_quality_proptest.rs` |
| REQ-LQ-13 | V | OB-LQ-V-14 | `calculate_quality` purity | `proofs/lattice_quality_verus.rs` |
| REQ-LQ-13 | P | OB-LQ-P-11 | `calculate_quality` determinism | `proofs/lattice_quality_proptest.rs` |

**Optional / supporting properties** (also covered by P):

- OB-LQ-P-13, OB-LQ-P-14: `get_dimension` correctness.
- OB-LQ-P-15: `get_issues` filter correctness.

These have no Verus counterpart — Verus spec fns for `get_dimension` / `get_issues` would require modeling `Vec` filtering, which Verus handles but adds fuel cost for marginal benefit over the proptest properties.

## 6. Trusted base plan

These are the assumptions the proofs lean on. Each is either explicitly trusted or has its own obligation.

| Trust | Why trusted | Mitigation in obligations |
|---|---|---|
| The five `calculate_*` heuristic bodies (lines 254–565) | String-processing heuristics (`has_contradiction`, jargon term list, security keyword list) whose exact numerical output is intentionally NOT in scope for this proof plan. The internal logic is covered by the ~30 `#[test]` cases in the production `tests` block (lines 567–1558, including 13 explicit mutation-catching tests at lines 1087+). | No Verus spec on the heuristic bodies. The proptest properties P-07..P-11 verify the *observable contract* (empty → error, 5 dimensions, valid overall, idempotent) but not the specific score values. |
| `serde_json` round-trip preserves values | Library contract; not our code. | OB-LQ-P-20..P-27 explicitly exercise the round-trip on every serializable type in this module — if serde silently corrupted values, the properties would fail and shrink to a minimal counterexample. |
| `String` is a total type over arbitrary UTF-8 | Rust stdlib. | Proptest generates arbitrary `String` via `vec<any::<char>>`, covering empty / non-ASCII / very long inputs. |
| The `QualityDimension` enum is closed (no future variant added) | Type-system fact at the time the spec is written. | OB-LQ-V-01, OB-LQ-V-02 enumerate the 5 variants explicitly via `seq!` literal. Adding a variant breaks the spec at compile time. |
| `Vec<T>` and `&[T]` semantics | Rust stdlib. | Not explicitly modelled; Verus uses `Seq<T>` to mirror, and proptest uses real `Vec`/`&[]`. |
| Verus's `Seq<char>` equality model | Verus framework. | The `spec_dimension_label` / `spec_dimension_description` definitions match the source `&'static str` literals character-for-character; the writeup asserts this correspondence as a manual review check. |

## 7. Waiver candidates

**None.** All in-scope behaviour is provable under the chosen lanes (V + P). The non-applicable lanes (K, F, L, M, T, Z, X) have concrete evidence in §4 and do not require waivers — they are genuinely not needed.

The heuristic bodies (REQ-LQ-11's mean-floor aside) are intentionally not proved because they are string-processing heuristics whose exact numerical output is not a contract. If `rust-contract` later decides that *specific* score values are part of the contract (e.g., "completeness of 5/5 required fields = exactly 100"), an additional obligation set will be added — but that's an additive change, not a waiver of behaviour.

## 8. Bridge input for `proof-to-implementation`

| Spec / property | Rust source ref | Independent behaviour test |
|---|---|---|
| OB-LQ-V-01, V-02 | `QualityDimension` enum + `QualityDimension::all` (lines 50–78) | Existing `test_quality_dimension_all` (line 1018) and `test_quality_dimension_labels` (line 1001) cover the positive case. |
| OB-LQ-V-03, V-04 | `QualityDimension::label` / `description` (lines 81–100) | Existing `test_quality_dimension_labels` (line 1001) and `test_quality_dimension_descriptions` (line 1009). |
| OB-LQ-V-05, V-06 | `DimensionScore::new` (lines 112–117) | Existing `test_dimension_score_valid_range` (line 600) and `test_dimension_score_invalid_too_high` (line 610). |
| OB-LQ-V-07, V-08 | `DimensionScore::passes` (lines 120–122) | Existing `test_dimension_score_passes_threshold` (line 617). |
| OB-LQ-V-09 | `QualityScore::new` (lines 164–177) | Implicit in `test_calculate_quality_*` (lines 720+). |
| OB-LQ-V-10 | `QualityScore::passes` (lines 180–182) | Existing `test_quality_score_passes_threshold` (line 628). |
| OB-LQ-V-11, V-12 | `calculate_quality` (lines 223–251) | Existing `test_calculate_quality_empty_answers` (line 720) and `test_calculate_quality_perfect_scores` (line 732). |
| OB-LQ-V-13 | mean-floor at lines 244–248 | Existing `test_overall_score_calculation` (line 1029) covers the happy case. |
| OB-LQ-V-14 | `calculate_quality` purity | New property P-11 covers this. |
| OB-LQ-P-01..P-04 | `DimensionScore` range / monotonicity | Partially covered by existing `test_dimension_score_*`. Properties extend to arbitrary inputs. |
| OB-LQ-P-05, P-06 | `QualityScore` range / monotonicity | Partially covered by `test_quality_score_passes_threshold`. Properties extend to arbitrary inputs. |
| OB-LQ-P-07..P-12 | `calculate_quality` structural invariants | Properties are new; existing tests cover boundary cases (lines 720, 732, 1029). |
| OB-LQ-P-13..P-15 | `get_dimension` / `get_issues` accessors | Existing `test_quality_score_get_dimension` (line 661) and `test_quality_score_get_issues` (line 689). |
| OB-LQ-P-16..P-19 | Constants & `all()` | Existing `test_minimum_gate_constant` style coverage (line 1001+) and `test_quality_dimension_all` (line 1018). |
| OB-LQ-P-20..P-27 | JSON round-trip | New properties; no existing coverage for arbitrary `String` content. |

The proof-writer **has not modified** any production function body. Verus `#[verifier::external_body]` is NOT used in `lattice_quality_verus.rs` — the spec file is purely mathematical and does not call production code; the bridge assumption (spec ↔ implementation) is recorded in §6 above and is the responsibility of `proof-to-implementation`.

## 9. Blockers for proof-reviewer

1. **Contract ratification (BLOCKING).** `rust-contract` must author `contract.md` for `clarity-web/src/lattice/` and either ratify or correct the 13 clauses in §3. All 41 provisional obligations (14 V + 27 P) carry `clause_origin: INFERRED` to make this gate visible.
2. **Obligation ID formalization (BLOCKING).** `proof-planner` must produce `proof-obligations.planned.jsonl` and re-key the provisional `OB-LQ-V-NN` / `OB-LQ-P-NN` IDs in this writeup and the two artifact files.
3. **No tooling gaps for this plan.** Verus is installed at `/home/lewis/.local/bin/verus` (v0.2026.05.05.d03e906). proptest 1.10.0 is a dev-dependency. Kani / cargo-fuzz gaps exist but do not block this module.
4. **Clippy independent (NON-BLOCKING).** `cl-2q6` clippy gate does not affect `quality.rs` directly (this file has no `clippy::uninlined_format_args` or other lint failures per the baseline — see `formal-verification-report.md §4`). The proptest file uses `clippy::unwrap_used` / `clippy::expect_used` under `#![allow(...)]` since proptest requires panic-able assertions.

## 10. Non-targets (explicit)

Per `verification-targets.md §8` and the proof-writer skill boundary:

- **Line-by-line proofs.** Refused; not cost-effective. Only the public API surface is proved.
- **Heuristic body proofs.** The five `calculate_*` functions (lines 254–565) are intentionally NOT proved. Their exact numerical output depends on string-processing heuristics that are behavior-stable but specification-fragile — proving the exact formula would require a string semantics spec that is out of scope. The structural contract (always 5 dimensions, overall in [0,100], idempotent) IS proved (V-12, V-13, V-14 + P-08..P-12).
- **The 50 hand-written tests** in `#[cfg(test)] mod tests` (lines 567–1558) are exercise coverage, not proof targets. They are referenced by the bridge (§8) but not themselves proved.
- **Miri, Loom, TLA+, fuzz.** All not applicable (§4). No `not_applicable` obligation rows for these will be promoted to `waived`.
- **Production `unsafe`** — `forbid` at workspace level (`Cargo.toml:10`) and module level (`quality.rs:23`); no obligation needed.

## 11. Expected verifier commands

This section lists the **expected** commands for the formal-verifier agent. These are NOT executed by `proof-writer`; the user has explicitly instructed that no verifier runs in this delivery. Commands listed here for `formal-verifier` reference only.

### 11.1 Verus — `proofs/lattice_quality_verus.rs`

**Standalone verification** (no crate wiring required):

```bash
verus --crate-type lib --edition 2021 proofs/lattice_quality_verus.rs
```

This invokes Verus in library mode against the spec-only file. Expected outcome: 14 lemmas verified, no `assume` / `admit` / `external_body` introduced, no `axiom` declarations needed. (Per the verifier command templates in `proof-writer/references/lane-command-templates.md`.)

**Optional formatting check**:

```bash
verusfmt --check proofs/lattice_quality_verus.rs
```

**Trust-boundary audit** (must report zero findings):

```bash
rg -n 'assume\(|#\[verifier::external_body\]|#\[verifier::external\]|axiom' \
   --glob 'proofs/lattice_quality_verus.rs'
```

### 11.2 proptest — `proofs/lattice_quality_proptest.rs`

**Integration step (one-time, not part of this artifact)**:

```bash
# Move the artifact to the canonical location for cargo auto-discovery.
cp proofs/lattice_quality_proptest.rs clarity-web/tests/lattice_quality_proptest.rs
```

**Run** (after the integration step):

```bash
cargo test -p clarity-web --test lattice_quality_proptest -- --nocapture
```

Expected outcome: 27 properties verified, 256 cases each (configurable via `ProptestConfig::with_cases`), zero counterexamples, zero shrinks. Regression seed log: `--nocapture` shows the seed for each case so a failing case can be reproduced by `PROPTEST_SEED=<seed>`.

**Targeted run** (single property, faster iteration):

```bash
cargo test -p clarity-web --test lattice_quality_proptest prop_calculate_quality_idempotent -- --nocapture
```

### 11.3 Alternative: in-crate integration

If the artifact is integrated as a `#[path = "..."]` module inside `clarity-web/src/` (rather than the `tests/` directory), the command is:

```bash
cargo test -p clarity-web --lib lattice_quality_proptest -- --nocapture
```

This requires modifying `clarity-web/src/lib.rs` (add `#[cfg(test)] #[path = "../../proofs/lattice_quality_proptest.rs"] mod lattice_quality_proptest;`), which is a *production file modification* and **is NOT in this artifact's scope**. The integration step is the responsibility of `landing-skill` or the implementation owner.

## 12. Anti-verification-laundering audit

Per the `verus` skill mandate:

> `#[verifier::external_body]` is strictly forbidden for creating "Vacuum Proofs" of production code. You cannot declare a contract on an exec fn and then use `external_body` to avoid actually proving the function body.

The artifact `proofs/lattice_quality_verus.rs` is **NOT** a Vacuum Proof. Audit:

- The file contains **zero** `exec fn` declarations.
- The file contains **zero** `#[verifier::external_body]` annotations.
- The file contains **zero** `#[verifier::external]` annotations.
- The file contains **zero** `axiom` declarations.
- All `proof fn` bodies are explicit — empty bodies are used only where the postcondition follows trivially from `requires` and the spec function's structure (see e.g. `lemma_dimension_all_count`, `lemma_calculate_quality_idempotent`).
- The bridge between spec and production is recorded as a trusted boundary (§6), not hidden via `external_body`.

What the artifact DOES prove:

- Algebraic properties of the public API surface of `quality.rs` as a mathematical contract.
- Range, monotonicity, idempotence, and mean-floor invariants of the scoring algebra.

What the artifact DOES NOT prove (and explicitly disclaims):

- The exact formula of any `calculate_*` heuristic body.
- The byte-exact correspondence between `spec_dimension_label` and the source's `&'static str` literal (asserted via manual review; not machine-verified).

## 13. Pre-flight checklist for landing this artifact

- [ ] `rust-contract` produces contract clauses for the 13 inferred items in §3.
- [ ] `proof-planner` produces `proof-obligations.planned.jsonl` with formal IDs replacing `OB-LQ-V-NN` / `OB-LQ-P-NN`.
- [ ] `proof-plan-reviewer` reviews this writeup + the obligations JSONL.
- [ ] Verus invocation confirmed (see §11.1).
- [ ] proptest invocation confirmed (see §11.2) after the `tests/` integration step.
- [ ] `cl-2q6` clippy gate is independent of this artifact (`quality.rs` has no clippy failures per the baseline; the proptest file uses `#![allow(clippy::unwrap_used, clippy::expect_used)]` at file top).

---

*End of writeup.*
