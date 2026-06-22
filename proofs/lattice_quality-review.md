# Proof Review — `proofs/lattice_quality-{verus,proptest}.rs` + writeup

| Field | Value |
|---|---|
| **Bead** | `cl-dv5` — Verification: `clarity-web/src/lattice/quality.rs` (Verus + proptest) |
| **Date** | 2026-06-21 |
| **Reviewer** | `proof-reviewer` (adversarial, lethal-finding posture) |
| **Source module** | `clarity-web/src/lattice/quality.rs` (1558 LOC: 565 production + 993 `#[cfg(test)] mod tests`) |
| **Artifacts reviewed** | `proofs/lattice_quality_verus.rs` (532 lines, 19.2 KB), `proofs/lattice_quality_proptest.rs` (519 lines, 19.2 KB), `proofs/lattice_quality-writeup.md` (286 lines, 25 KB) |
| **External evidence (read-only)** | `/tmp/opencode/verus-lattice_quality.txt` (raw Verus log), `verification-ledger.jsonl` rows 27, 29, 31 (cl-dv5 status), `proofs/straw_man_verus.rs` (sibling pattern) |
| **Tool posture** | **Did NOT run Verus.** Verified log is read-only evidence. **Did NOT run proptest.** Wiring is BLOCKED_TOOLING per ledger row 31. |
| **Review verdict** | **REJECTED.** Three blockers (one verifier-execution blocker, two structural/disconnect blockers) + four non-blocking observations. |

---

## 1. Adversarial posture

The proof-reviewer skill mandates the following:

> "Assume the proof writer was lazy and tried to pass toy artifacts. Find the lie."

I attack from five angles the user task explicitly enumerated:

1. **Verifier-rejection fact-check.** The Verus log `pub spec enum QualityDimensionSpec` at line 57 is the trigger. Confirm real, confirm the fix, confirm the fix is semantically equivalent.
2. **Anti-laundering audit.** Writeup §12 claims zero `exec fn`, zero `#[verifier::external_body]`, zero `#[verifier::external]`, zero `axiom`. Confirm by grep.
3. **Trusted-boundary honesty.** Writeup §6 lists the five `calculate_*` heuristic bodies (lines 254-565) as trusted. Are the specs honest about leaving the algorithm unverified, or do they disguise that?
4. **Lemma self-containment.** 14 lemmas (OB-LQ-V-01..V-14) each prove something stronger than the precondition. Any lemma that depends on an unproven lemma without declaring it?
5. **proptest production-coupling.** 27 proptest properties — does each invoke production code, or is there a local re-implementation?

Additional attack vectors I exercised:

- **Signature mismatch.** The spec `spec_calculate_quality` does not model the production `calculate_quality` — it takes an `overall` parameter that production does not accept. Disconnect.
- **Vacuous-spec check.** Several lemmas prove only the literal `seq!` macro from the spec itself, not the production array. Disconnect at the spec/production boundary.
- **No plan / no ratification.** The writeup §2 explicitly admits `rust-contract` did not run and the obligation IDs are provisional. Same gate-skip pattern as `cl-54n`. Repeat offender for procedural gaps.

The findings are ordered by severity, each using the canonical `finding/v1` schema: artifact, obligation, severity, required fix, disposition. The disposition is one of `fixed_with_evidence`, `owner_approved_debt`, `owner_approved_no_action`, `routed`, or (when blocker) a recommendation of re-review.

---

## 2. Verifier execution status (read-only inspection)

`proofs/lattice_quality_verus.rs` was previously run by the host verifier and rejected. The raw log at `/tmp/opencode/verus-lattice_quality.txt` is short and unambiguous:

```
error: expected `enum`
  --> /home/lewis/src/clarity/proofs/lattice_quality_verus.rs:57:5
   |
57 | pub spec enum QualityDimensionSpec {
   |     ^^^^

error: aborting due to 1 previous error
```

The verifier aborts on the **first** syntax error and does not report the others. A full syntax scan of the file shows the same forbidden pattern at **7 sites** (verifier stops at line 57):

| Line | Token | Note |
|---|---|---|
| 53 | `pub spec const SPEC_MINIMUM_GATE: u8 = 70;` | `pub spec const` is not Verus syntax |
| 57 | `pub spec enum QualityDimensionSpec` | first error; verifier aborts here |
| 66 | `pub spec enum IssueSeveritySpec` | unreported (verifier exited at 57) |
| 76 | `pub spec enum QualityErrorSpec` | unreported |
| 86 | `pub spec struct DimensionScoreSpec` | unreported |
| 94 | `pub spec struct QualityIssueSpec` | unreported |
| 101 | `pub spec struct QualityScoreSpec` | unreported |

The writeup §7 acceptance criteria require "Exit code 0". The actual result is exit 1. `verification-ledger.jsonl` row 29 records this as `verifier_execution` for `cl-dv5` with `exit_status: 1`, `classification: FAIL_LOCAL`. The error message in that row restates the same finding.

`verification-ledger.jsonl` row 27 (cl-dv5 module_proof_artifacts) lists status as `PENDING_FORMAL_EXECUTION (Verus) + BLOCKED_TOOLING.wiring (proptest)`. This is wrong: the V lane was actually executed and failed. The mismatch is a separate documentation problem but does not change the underlying fact — the spec does not verify, period.

The sibling artifact `proofs/straw_man_verus.rs` (cl-vv2) uses `pub enum StrawManTrap { ... }` and `pub struct StrawManValidation { ... }` — **without** the `spec` keyword — and that pattern compiles past the parser (it then fails on a separate `slice::contains` vstd limitation, but it is at least syntactically valid). The author of `lattice_quality_verus.rs` confused the syntactic position of `spec`: `spec` modifies function signatures (`pub open spec fn`, `pub closed spec fn`) and proof-fn return types, not type declarations. The correct way to declare spec-only enums in Verus is to declare them as ordinary `pub enum` (which is then visible to both exec and spec contexts) or to keep the production type and reference it via `external_type_specification`.

**Fix semantics.** The user's prompt says "The fix: remove the `spec` keyword from each spec type declaration." That change is mechanically correct — it converts `pub spec enum X` to `pub enum X` etc., which parses. But the **semantic** equivalence is approximate, not exact:

- `pub spec enum X` does not exist in Verus, so the author's intent cannot be exactly recovered.
- `pub enum X` declares an exec enum (with `Eq`/`Hash`/`Serialize` etc. available). The author clearly wanted a spec-only mirror.
- The faithful fix is to either (a) drop the `*Spec` mirror types and import the production types via `external_type_specification` from `clarity_web::lattice::quality`, or (b) keep the mirror types as `pub enum` (accepting they are exec) and explicitly document the boundary in the trust ledger.

Option (a) is structurally correct; option (b) is what the user prompt proposes. Either way, the trust ledger needs a new row for the spec/production correspondence (it currently has none).

---

## 3. Findings (per `proof-reviewer` schema)

### F1 — BLOCKER (verifier-execution)

- **Artifact:** `proofs/lattice_quality_verus.rs:53, 57, 66, 76, 86, 94, 101` (all `pub spec <decl>` sites)
- **Obligation:** All 14 Verus obligations (OB-LQ-V-01..V-14) — none can verify while the file does not parse.
- **Severity:** blocker
- **Evidence:**
  - `/tmp/opencode/verus-lattice_quality.txt:1-8` (raw log).
  - `verification-ledger.jsonl` row 29 (`verifier_execution`, `cl-dv5`, `exit_status: 1`, `FAIL_LOCAL`).
  - Sibling pattern `proofs/straw_man_verus.rs:62-75` uses `pub enum`/`pub struct` without `spec` and parses.
- **Required fix:** Remove the `spec` keyword from all seven declaration sites listed above. The mechanical change is:
  - `pub spec const SPEC_MINIMUM_GATE` → `pub const SPEC_MINIMUM_GATE`
  - `pub spec enum QualityDimensionSpec` → `pub enum QualityDimensionSpec` (and lines 66, 76)
  - `pub spec struct DimensionScoreSpec` → `pub struct DimensionScoreSpec` (and lines 94, 101)
  Then re-run `verus --crate-type lib --edition 2021 proofs/lattice_quality_verus.rs` and capture exit 0. Add a `verifier_execution` row to `verification-ledger.jsonl` with `exit_status: 0`, `classification: PASS`.
- **Disposition:** `fixed_with_evidence` (required before re-review)

### F2 — BLOCKER (spec/production disconnect on `calculate_quality`)

- **Artifact:** `proofs/lattice_quality_verus.rs:217-232` (`spec_calculate_quality`)
- **Obligation:** OB-LQ-V-11, OB-LQ-V-12, OB-LQ-V-14 (and indirectly the §3 contract REQ-LQ-10)
- **Severity:** blocker (semantic disconnect)
- **Evidence:** The spec signature is `spec_calculate_quality(answers_len: int, overall: int) -> Result<QualityScoreSpec, QualityErrorSpec>`. The production signature is `pub fn calculate_quality(answers: &[Answer], ears: &[EarsRequirementRef], _inversion: &InversionControl) -> Result<QualityScore, QualityError>` at `clarity-web/src/lattice/quality.rs:223-251`. Production **computes** `overall = floor(sum(d.score for d in dimensions) / 5)` from the heuristic bodies; production does **not** accept `overall` as a parameter.
  - OB-LQ-V-12 (line 450-466) "proves" `match spec_calculate_quality(answers_len, overall) { Ok(q) => q.overall == overall && 0 <= q.overall <= 100, ... }` for any `0 <= overall <= 100`. But this only proves a property of the spec function, which trivially passes through whatever `overall` the caller provides. It does **not** prove any property of production, because production's `overall` is computed, not supplied.
  - OB-LQ-V-13 (lemma_mean_of_five_in_unit_interval) is pure arithmetic on five `int`s in `[0,100]`. The lemma is mathematically correct but disconnected from the production pipeline — it does not reference `DimensionScore.score`, does not reference the production `dimensions.iter().map(|d| u32::from(d.score)).sum::<u32>() / dimensions.len() as u32` computation at `quality.rs:245`, and does not connect to the `u8::try_from` overflow guard at line 247-248. The writeup §3 contract REQ-LQ-10 says `overall = floor(sum(d.score for d in dimensions) / 5)`; the spec does not assert this; the lemma proves a weaker abstract statement.
  - OB-LQ-V-14 (idempotence, lines 521-530) is a literal tautology: `spec_calculate_quality(a, b) == spec_calculate_quality(a, b)`. Every pure function is equal to itself; no information content.
  - The dimensions in the Ok-arm of the spec are `Seq::empty()` (line 226-227) with the comment "TRUSTED: populated by heuristic body." The contract REQ-LQ-10 requires `dimensions.len() == 5`. The spec does not assert this; it returns `Seq::empty()` and trusts the body.
- **Required fix:** Rewrite `spec_calculate_quality` so its signature and postconditions match production:
  ```rust
  pub open spec fn spec_calculate_quality(
      answers_len: int,
      ears_len: int,
      // per-dimension scores computed by the (trusted) heuristic bodies
      d1: int, d2: int, d3: int, d4: int, d5: int,
  ) -> Result<QualityScoreSpec, QualityErrorSpec>
  ```
  with the Ok-arm carrying `dimensions: seq![d1, d2, d3, d4, d5].map(...)` and `overall: floor((d1+d2+d3+d4+d5)/5)`. Then re-prove OB-LQ-V-12 to assert `q.dimensions.len() == 5`, `q.overall == floor((d1+...+d5)/5)`, and `0 <= q.overall <= 100` — combining OB-LQ-V-12 with OB-LQ-V-13. Either this rewrite, or drop the four `calculate_quality` lemmas (V-11..V-14) as out-of-scope per writeup §10 (and acknowledge that REQ-LQ-10 / REQ-LQ-11 / REQ-LQ-13 are covered by proptest only, which is a substantive loss of coverage).
- **Disposition:** `fixed_with_evidence` (required before re-review)

### F3 — BLOCKER (no upstream plan ratification, same gate-skip as cl-54n / cl-vv2)

- **Artifact:** `proofs/lattice_quality-writeup.md §2` ("Contract gap (honest disclosure)")
- **Obligation:** All 14 V + 27 P = 41 obligations (writeup §5)
- **Severity:** blocker (procedural; routes out of `proof-writer`)
- **Evidence:** The writeup explicitly admits:
  - "**GAP — no `rust-contract` artifact exists for this module.**"
  - "All clauses used in this writeup are *inferred by direct reading of the source*, not authored."
  - "Obligation IDs (`OB-LQ-V-NN`, `OB-LQ-P-NN`) are PROVISIONAL and must be replaced with planner-assigned IDs."
  - "`rust-contract` must author `contract.md` ... and either ratify or correct the clauses."
  - "`proof-planner` must produce `proof-obligations.planned.jsonl` ... replacing the provisional IDs."
  - "`proof-plan-reviewer` must review this writeup + the obligations JSONL before any verifier invocation."
  - `verification-ledger.jsonl` row 27 confirms status `PENDING_FORMAL_EXECUTION` and lists `blocking_for: ["wiring for proptest", "cl-u04 for Kani/fuzz", "formal-verifier execution for Verus (after spec repair)"]`.
  - This is the **same gate-skip** as `cl-54n` (proof-writer dispatched without proof-plan-reviewer pass) and `cl-vv2` (writeup §1 explicitly notes "no approved proof plan exists yet"). The pattern is now established across the verification fleet and is a workspace-level process gap, but for this specific bead the blocker stands.
- **Required fix:** Out of scope for `proof-writer` / `proof-reviewer`. Routed:
  - `rust-contract` must author `clarity-web/src/lattice/contract.md` ratifying the 13 clauses in writeup §3.
  - `proof-planner` must produce `proofs/lattice-quality-obligations.planned.jsonl` with formal IDs.
  - `proof-plan-reviewer` must accept the plan before proof-writer is dispatched to write proofs.
  - Until all three close, the bridge to `proof-to-implementation` cannot begin.
- **Disposition:** routed (cannot be fixed inside this bundle; blocks advancement to bridge)

### F4 — OBSERVATION (vacuous lemmas V-01, V-02, V-03, V-04, V-05, V-06, V-09)

- **Artifact:** `proofs/lattice_quality_verus.rs:240-258, 262-276, 280-312, 386-403`
- **Obligation:** OB-LQ-V-01, V-02, V-03, V-04, V-05, V-06, V-09
- **Severity:** observation (non-blocking, but the spec/production correspondence is by manual review)
- **Evidence:**
  - V-01, V-02: prove properties of the literal `seq!` macro inside `spec_dimension_all()`. The proof is empty (`{ // The seq! literal has 5 entries ... }`). Whether `spec_dimension_all()` corresponds to `QualityDimension::all()` (production, `quality.rs:70-78`) is asserted by manual review only.
  - V-03, V-04: same pattern — the spec function returns a literal `seq!`; the proof is trivial. Correspondence to `QualityDimension::label` / `description` (`quality.rs:81-100`) is manual.
  - V-05, V-06: spec `spec_dimension_score_new(d, score)` returns `Ok` iff `0 <= score <= 100`. Production `DimensionScore::new` (`quality.rs:112-117`) does `match score { 0..=100 => Ok(...), invalid => Err(...) }`. The two are identical, but the spec is a **copy** of production — the proof is "the if-guard mirrors the production match". No mechanism binds them.
  - V-09: same as V-05/V-06 lifted to `QualityScore::new`.
- **Required fix (optional):** Either (a) add a `trusted-base-ledger.jsonl` row per `*Spec` mirror type declaring the spec/production correspondence as a manual-review boundary (consistent with writeup §6), or (b) rewrite using `external_type_specification` so the spec functions reference the production types directly. Option (b) requires the spec file to be inside the `clarity-web` crate build (currently the artifact is meant to compile standalone).
- **Disposition:** `owner_approved_debt` (writeup §6 already declares the boundary; ledger row would make it explicit)

### F5 — OBSERVATION (`spec_calculate_quality` parameters do not exist in production)

- **Artifact:** `proofs/lattice_quality_verus.rs:217-220`
- **Severity:** observation (already covered by F2's required fix)
- **Evidence:** Same evidence as F2. Reported separately because the spec signature mismatch is the load-bearing issue; F2 names the required fix. This row is a marker so the ledger can link F2 to a specific line range.
- **Disposition:** rolled into F2

### F6 — OBSERVATION (`arb_quality_score` uses `.expect("...")`)

- **Artifact:** `proofs/lattice_quality_proptest.rs:140-144`
- **Obligation:** OB-LQ-P-20 (and downstream P-12, P-15)
- **Severity:** observation (non-blocking under `#![allow(clippy::unwrap_used, clippy::expect_used)]` at line 48)
- **Evidence:** The generator constructs `QualityScore::new(overall, dimensions, issues).expect("valid overall must be Ok")` for `overall in 0u8..=100`. The `expect` is safe (the `0..=100` range is the validated range by `QualityScore::new` at `quality.rs:170`). It is acceptable under the file-level `#![allow]`. The pattern matches the sibling `proofs/straw_man_proptest.rs:46-53` convention.
- **Required fix (optional):** None. The allow-list is correct and the runtime assertion is safe.
- **Disposition:** `owner_approved_no_action`

### F7 — OBSERVATION (`prop_get_issues_filters_by_dimension` is partially tautological)

- **Artifact:** `proofs/lattice_quality_proptest.rs:386-407` (`prop_get_issues_filters_by_dimension`)
- **Obligation:** OB-LQ-P-15
- **Severity:** observation
- **Evidence:** The property asserts `filtered.len() == issues.iter().filter(|i| i.dimension == target).count()` where `filtered = q.get_issues(target)` and `q.issues == issues`. The two counts are the same set filtered by the same predicate, so the assertion is structurally true by construction. A stronger property would assert (a) every element in `filtered` has `dimension == target`, AND (b) `filtered.len() == target_issues.len()` (the count of `target_issues`, not the post-hoc filter count). The current form tests that `get_issues` returns *some* count, not that it returns *the correct* count.
- **Required fix (optional):** Strengthen OB-LQ-P-15:
  ```rust
  prop_assert_eq!(filtered.len(), target_issues.len(),
      "get_issues(target) must return exactly the issues created with target");
  for issue in &filtered {
      prop_assert_eq!(issue.dimension, target,
          "get_issues(target) must only return issues with dimension == target");
  }
  ```
- **Disposition:** `owner_approved_debt`

### F8 — OBSERVATION (writeup §5 source-mapping is mostly correct, with two minor imprecisions)

- **Artifact:** `proofs/lattice_quality-writeup.md §5` (proof coverage matrix)
- **Severity:** observation (cosmetic)
- **Evidence (positive):** Spot-check of 5 random mappings:
  - OB-LQ-V-05, V-06 → `DimensionScore::new (lines 112-117)`: source line 112 is `pub fn new(dimension: QualityDimension, score: u8) -> Result<Self, QualityError> {`, line 117 closes the function. ✓
  - OB-LQ-V-11, V-12 → `calculate_quality (lines 223-251)`: source line 223 is `pub fn calculate_quality(`, line 251 closes. ✓
  - OB-LQ-V-13 → mean-floor at lines 244-248: source lines 244-245 hold the sum/divide, lines 247-248 hold the `u8::try_from`. ✓ (Approximately matches; the writeup is precise.)
  - OB-LQ-V-14 → `calculate_quality` purity: source line 223-251 is the function; proptest P-11 (lines 337-345) exercises it. ✓
  - OB-LQ-P-20..P-27 → all 8 `Serialize`/`Deserialize` types: each type derives Serialize/Deserialize at the cited source line (`quality.rs:50, 104, 126, 144, 152, 200, 208`, plus `clarity-web/src/types.rs:68` for `Answer`). ✓
- **Evidence (imprecisions):**
  - Writeup §3 REQ-LQ-10 says "`overall = floor(sum(d.score for d in dimensions) / 5)`". The spec at V-11..V-14 does not enforce this — see F2.
  - Writeup §1 line 19 says "12 public types / enums". Actual count: 11 (re-export `pub use crate::types::Answer;` at line 31 plus 10 own items). Minor.
- **Required fix (optional):** Update §1 "12 public types" to "11 public types (re-export + 10 own)". Substantive issue tracked as F2.
- **Disposition:** `owner_approved_no_action` (cosmetic)

---

## 4. Anti-laundering audit (per `proof-reviewer` skill)

The proof-reviewer skill mandates scan of `assume(`, `#[verifier::external_body]`, `#[verifier::external]`, `axiom`, `exec fn`. Audit results:

| Marker | Count in `lattice_quality_verus.rs` | Honesty |
|---|---|---|
| `assume(` | 0 | n/a |
| `#[verifier::external_body]` | 0 | n/a (no Vacuum-Proof shortcut on production) |
| `#[verifier::external]` | 0 | n/a |
| `axiom` | 0 | n/a |
| `exec fn` | 0 | n/a (file is spec-only; no verbatim copy of production bodies) |
| `pub spec enum` (invalid syntax) | 3 | dishonest — verifier rejects; see F1 |
| `pub spec struct` (invalid syntax) | 3 | dishonest — verifier rejects; see F1 |
| `pub spec const` (invalid syntax) | 1 | dishonest — verifier rejects; see F1 |

**No `exec fn`** is correct because the file does not call into production. That is consistent with writeup §8 ("the spec file is purely mathematical and does not call production code"). The trade-off is that the bridge is purely by **manual review** of the literal text in spec functions vs. production bodies — there is no machine-checked link. This is acknowledged in writeup §6 but is not implemented as a `trusted-base-ledger.jsonl` row.

**Verbatim body copies:** not applicable (no `exec fn`).

**proptest production-coupling:** all 27 properties import `clarity_web::lattice::quality::*` and invoke the production API. No local re-implementation. No shadow functions. Confirmed by side-by-side comparison of the `use clarity_web::...` import at proptest line 50-53 vs. the call sites.

**Anti-laundering verdict:** **3 dishonest markers** (the 7 `pub spec` declarations), all collated under F1. No `assume`/`external_body`/`axiom` abuse. proptest is honest throughout.

---

## 5. Non-vacuity checks (per `proof-reviewer` skill)

Per the skill: "Demand evidence that the verifier could fail." Audit:

| Obligation | Non-vacuity status | Evidence |
|---|---|---|
| V-01 (`all().len() == 5`) | non-vacuous IF spec/production correspondence holds | empty proof body; vacuous as written but bridges to production via manual review of the literal seq! |
| V-02 (`all()` contains 5 variants) | non-vacuous IF spec/production correspondence holds | same; pure spec-level tautology without manual review |
| V-03 (`label` non-empty) | non-vacuous IF spec/production correspondence holds | match arms return non-empty literals; trivial |
| V-04 (`description` non-empty) | non-vacuous IF spec/production correspondence holds | match arms return non-empty literals; trivial |
| V-05 (`new` Ok range) | non-vacuous IF spec/production correspondence holds | spec is `if 0 <= score && score <= 100 { Ok } else { Err }`; production is identical match |
| V-06 (`new` Err range) | same as V-05 | same |
| V-07 (`passes` monotone in score) | non-vacuous | the proof body has actual `assert(...)` steps using transitivity of `>=` |
| V-08 (`passes` antitone in threshold) | non-vacuous | same as V-07 |
| V-09 (`QualityScore::new` validates) | non-vacuous IF spec/production correspondence holds | same as V-05 |
| V-10 (`QualityScore::passes` monotone) | non-vacuous | real `assert(...)` chain |
| V-11 (`calculate_quality` empty → Err) | partially vacuous (covered by F2) | the proof is "the if-guard `0 == 0` lands in the Err arm"; trivial for spec but maps to production behavior at `quality.rs:228-230` |
| V-12 (`calculate_quality` non-empty Ok) | **vacuous** (covered by F2) | spec trivially passes through caller-supplied `overall`; no constraint on `dimensions` |
| V-13 (mean floor in `[0,100]`) | mathematically correct but **disconnected from production** (covered by F2) | real arithmetic proof; does not reference `DimensionScore.score` or the production computation |
| V-14 (idempotence) | tautology | `f(a,b) == f(a,b)` is a literal tautology for any pure function |
| P-01 (`DimensionScore::new` accepts) | non-vacuous | proptest calls production constructor with boundary inputs |
| P-02 (`DimensionScore::new` rejects) | non-vacuous | proptest calls production constructor with `101..=255` |
| P-03, P-04 (monotonicity) | non-vacuous | proptest exercises boundary and random inputs |
| P-05, P-06 (`QualityScore::new`, `passes`) | non-vacuous | proptest calls production constructor and predicate |
| P-07..P-12 (calculate_quality structural) | non-vacuous | proptest calls production `calculate_quality` with structured inputs |
| P-13, P-14 (`get_dimension`) | non-vacuous | proptest exercises production accessor |
| P-15 (`get_issues` filter) | partially vacuous (covered by F7) | the assertion is internally consistent but not a strong contract |
| P-16 (`MINIMUM_GATE == 70`) | non-vacuous (trivially true) | proptest asserts production constant value |
| P-17 (`QualityDimension::all()` cardinality and distinctness) | non-vacuous | proptest calls production `all()` and asserts length + uniqueness |
| P-18, P-19 (label/description non-empty) | non-vacuous | proptest calls production methods |
| P-20..P-27 (JSON round-trip on 8 types) | non-vacuous | proptest serializes + deserializes production values |

**Verdict:** **1 vacuous obligation** (V-14 tautology), **1 partially vacuous** (V-12, covered by F2), **1 partially vacuous** (P-15, covered by F7). The rest are non-vacuous. No fatal non-vacuity.

---

## 6. Trust-base ledger audit (per `proof-reviewer` skill)

The skill mandates that every trust marker have a corresponding `trusted-base-ledger/v1` row. The writeup §6 functions as the trust ledger for this artifact. Audit:

| # | Trust entry | Honest? | Notes |
|---|---|---|---|
| 1 | The five `calculate_*` heuristic bodies (lines 254-565) | yes | honest: author acknowledges algorithm is unverified; structural contract is verified separately |
| 2 | `serde_json` round-trip preserves values | yes | library contract; P-20..P-27 exercise the round-trip |
| 3 | `String` is a total type over arbitrary UTF-8 | yes | Rust stdlib |
| 4 | `QualityDimension` enum is closed (no future variant) | yes | type-system fact; adding a variant breaks the spec at compile time |
| 5 | `Vec<T>` and `&[T]` semantics | yes | Rust stdlib |
| 6 | Verus's `Seq<char>` equality model | yes | Verus framework; spec `spec_dimension_label` matches the source `&'static str` literals character-for-character (writeup §6 manual check) |
| 7 (MISSING) | The 7 `*Spec` mirror types correspond to production types | **no** | no `trusted-base-ledger.jsonl` row exists; bridge is by writeup §6 prose, not by ledger |

**5 of 7 trust entries are honest.** Entry #7 is implicit but unledgered. Entry #1 is honest but the spec for `calculate_quality` is too weak (F2). The trust base is otherwise complete.

**No `proofs/lattice_quality_trusted_base.jsonl` exists.** This is a documented gap but does not rise to blocker severity given that the writeup §6 prose is comprehensive and the entries #1-6 are honest. Combined with F2 (spec signature disconnect), the artifact needs a `trusted-base-ledger.jsonl` row to document that `spec_calculate_quality(answers_len, overall)` is a *deliberate simplification* of production's `calculate_quality(answers, ears, inversion)` and not a faithful mirror — until then, a reader will be misled into thinking the V lemmas prove production behavior.

---

## 7. Bridge readiness

The proof-to-implementation bridge requires:

1. **Verus spec verifies with exit 0**: **NOT MET** (F1)
2. **Spec faithfully mirrors production behavior**: **NOT MET** for `calculate_quality` (F2)
3. **Upstream plan ratified**: **NOT MET** (F3, routed)
4. **proptest properties exercised by independent behavior tests**: **NOT YET** (this is the bridge's job; cannot start until 1-3)
5. **Refinement harness refs**: not applicable to this pure-data module (no refinement state to maintain)
6. **Trust ledger honest**: **PARTIALLY MET** (entry #7 missing; see §6)
7. **No `assume`/`external_body`/`axiom`**: **MET** (verified by audit §4)

**Recommendation: do not advance to bridge until F1, F2, F3 are resolved.**

---

## 8. Disposition summary

| Finding | Severity | Disposition |
|---|---|---|
| F1 — Verus does not verify (7× `pub spec` syntax errors) | blocker | `fixed_with_evidence` (required) |
| F2 — `spec_calculate_quality` does not model production (V-12 vacuous, V-13 disconnected, V-14 tautology) | blocker | `fixed_with_evidence` (required) |
| F3 — No upstream plan ratification (no `rust-contract`, no `proof-planner` JSONL, no `proof-plan-reviewer` pass) | blocker (procedural) | routed |
| F4 — V-01..V-06, V-09 lemmas prove only the spec literal; spec/production correspondence unledgered | observation | `owner_approved_debt` |
| F5 — `spec_calculate_quality` parameters do not exist in production | observation (rolled into F2) | rolled |
| F6 — `arb_quality_score` uses `.expect("...")` under file-level `#![allow]` | observation | `owner_approved_no_action` |
| F7 — `prop_get_issues_filters_by_dimension` is partially tautological | observation | `owner_approved_debt` |
| F8 — Writeup §5 source-mapping is mostly correct; §1 "12 public types" should be "11" | observation | `owner_approved_no_action` |

**Blockers: 3** (F1 verifier-execution, F2 spec/production disconnect, F3 procedural). **Non-blocking observations: 5** (F4, F5, F6, F7, F8 — all with explicit disposition).

---

## 9. Final status

```
STATUS: REJECTED
```

Advancement to `proof-to-implementation` bridge is **blocked** until F1 (remove `spec` keyword from 7 declaration sites; re-run verus; capture exit-0 evidence in `verification-ledger.jsonl`), F2 (rewrite `spec_calculate_quality` to mirror production signature or drop V-12/V-14 and downgrade REQ-LQ-10/11/13 to proptest-only), and F3 (`rust-contract` + `proof-planner` + `proof-plan-reviewer` ratify the 14 V + 27 P obligations) are resolved.

**Recommendation to proof-writer / next agent:** fix-and-reverify. The mechanical fix for F1 is one global text substitution (`pub spec ` → `pub ` at 7 sites) plus a verifier re-run. The semantic fix for F2 requires rewriting `spec_calculate_quality` to accept the 5 dimension scores as inputs and asserting the floor-mean relationship in the Ok-arm — this is more invasive but tractable. F3 is out of scope for the proof-writer but blocks downstream work. Do **not** advance to bridge until exit 0 is captured in `verification-ledger.jsonl`.

---

*End of proof review.*