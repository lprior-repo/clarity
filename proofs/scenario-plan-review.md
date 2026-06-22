# Proof Plan Review — `clarity-web/src/domain/scenario.rs` (cl-zup)

| Field | Value |
|---|---|
| **Bead** | `cl-zup` |
| **Target** | `clarity-web/src/domain/scenario.rs` (634 LOC; production ~323, test ~309) |
| **Plan under review** | `proofs/scenario-proof-plan.md` (45.4K, 11 sections) |
| **Obligations under review** | `proofs/scenario-obligations.planned.jsonl` (67.0K, 40 rows: 25 V + 8 P + 7 NA) |
| **Reviewer skill** | `proof-plan-reviewer` |
| **Reviewer invocation ID** | `cl-zup-proof-plan-reviewer-2026-06-21T22:15:00Z-a1b2c3d4` |
| **Planner invocation ID** | `cl-zup-proof-planner-2026-06-21T20:49:32Z` |
| **Review state** | `in_review` |
| **Verdict** | `STATUS: REJECTED` |

---

## 1. Reviewed artifacts (with SHA-256)

| Artifact | Path | SHA-256 | Bytes |
|---|---|---|---|
| Proof plan | `proofs/scenario-proof-plan.md` | `803ffd7f79d2b4ecb8f768aa0d6cb7303d2a0c1a32c2a250bed635aa556793ba` | 46,442 |
| Obligations JSONL | `proofs/scenario-obligations.planned.jsonl` | `b0b2a62adc136c268c122e4bea1915084b71a8aa706548950df2083d3d46f602` | 68,570 |
| Source under proof | `clarity-web/src/domain/scenario.rs` | `03d8e0c96f20d0330cdd48d5bfa3ca51208ad82fa25fc869ffe876ad1721e63e` | 19,257 |
| Parent module | `clarity-web/src/domain/mod.rs` | `1fc1e0aaad894e1cb8a7e6326d751a20373362f3cc7be24d25315bfa0e93cb2d` | 765 |
| Lane roadmap | `verification-targets.md` | `bc9e62838fc6d35136ee32c6ba6ecc7828447771d0388422b9a9d925d30ab2ed` | (n/a) |
| Workspace state | `formal-verification-report.md` | `16ee260c0640a755fd9dcf68f8547bed8d0a4038491c4d73725e5f7c6d9c819e` | (n/a) |

**Provenance verified:** reviewer invocation ID `cl-zup-proof-plan-reviewer-2026-06-21T22:15:00Z-a1b2c3d4`
differs from planner invocation ID `cl-zup-proof-planner-2026-06-21T20:49:32Z`. No
self-stamped reviewer fields in planner artifacts.

---

## 2. Reviewer verdict and counts

| Disposition | Count |
|---|---:|
| blocker | 8 |
| owner_approved_debt | 1 |
| owner_approved_no_action | 4 |
| **Total findings** | **13** |

The 8 blocker findings are recorded with `disposition: blocker` (severity field is
mixed: 4 with `severity: blocker`, 4 with `severity: minor` but disposition
blocker because they each require artifact-level fixes before approval). The 1
approved debt is `owner_approved_debt`. The 4 observations are
`owner_approved_no_action` — no action required.

**Verdict: `STATUS: REJECTED`**

Eight blocker findings, four non-blocking observations, and one approved debt are recorded
in `proofs/scenario-plan-findings.jsonl` (13 rows, `finding/v1`). The repair guide is in
`proofs/scenario-plan-repair-guide.md`. Lane reviews are in
`proofs/scenario-verifier-lane-review.jsonl` (9 rows, `verifier-lane-review/v1`).

---

## 3. Adversarial answers (from the brief)

### Q1 — State machine analysis correctness (§1.1)

**Correct.** The two-axis decomposition is accurate to source:

- **Axis A (Bullets):** `is_trigger_empty` ∧ `is_value_moment_empty` ∧ `is_feeling_empty`
  (lines 308-322); `is_bullets_complete` is the negated conjunction (lines 300-304). The
  `EmptyBullets ⇔ any bullet empty trimmed` decomposition is sound.
- **Axis B (Hole Punching):** three `is_addressed(ht)` predicates (lines 175-191), with
  `HolesComplete ⇔ ∀ ht, is_addressed(ht)` (lines 159-172). The `addressed_count ∈ 0..=3`
  decomposition is sound (lines 229-234).
- **Composed terminal:** `ScenarioField::is_complete ⇔ bullets_complete ∧ hole_punching.is_complete`
  (line 294-296). No rejected/failed state is correctly observed.

**L1-L9 laws verified accurate.** Spot-checked against source:

| Law | Source ref | Plan claim | Verdict |
|---|---|---|---|
| L1 (right-most-wins) | lines 199-203: `address` writes exactly one field per call | correct | ✓ |
| L2 (idempotent) | L1 + `e1 == e2` | correct | ✓ |
| L3 (addressed_count monotonic) | line 229-234: pure observer; `address` cannot decrement | correct | ✓ |
| L4 (unaddressed_holes idempotent) | lines 219-225: pure observer | correct | ✓ |
| L5 (is_complete conjunction) | line 294-296 | correct | ✓ |
| L6 (is_complete ⇔ addressed_count == 3 ∧ bullets_complete) | lines 159-172 vs 229-234 | correct | ✓ |
| L7 (is_bullets_complete ⇔ negations of is_*_empty) | lines 300-304 vs 308-322 | correct | ✓ |
| L8 (HoleType::all canonical 3-element slice) | lines 37-43 | correct | ✓ |
| L9 (with_severity clamp `[1, 5]`) | line 107 | correct | ✓ |

**Missing law candidates (not blocking):**

- **L10** — `normalize_explanation` is its own inverse only under self-application via
  Option, which is trivially true. Not worth adding.
- **L11** — `from_strings` is functionally equivalent to three `address` calls from
  `default()`. Captured by REQ-SC-20; not separately enumerated as a law. Acceptable.
- **L12** — `is_addressed(ht) ⇒ addressed_count() ≥ 1` per-axis. Not a separate law;
  follows from REQ-SC-19's decomposition. Acceptable.

**No law was found to be wrong against source.** The state-machine analysis is the
strongest aspect of the plan.

---

### Q2 — Contract gap (26 INFERRED clauses)

**Reject-amplifying.** The 26 `INFERRED` clauses are the planner's own guess, not
rust-contract output. The plan §2 admits this. Per the go-skill workflow, rust-contract
MUST precede proof-planner; the planner here ran in reverse.

This is **a workspace-wide condition** per `formal-verification-report.md §3` (cl-2vr
critical finding + adjacent rust-contract gap affecting all 8 module plans). For
`scenario.rs` specifically, the gap is **amplified** by:

- Q1 ('addressed' semantics — `Some(s) ∧ !s.trim().is_empty()` vs alternatives) drives
  5 clauses (REQ-SC-11, -12, -14, -15, -16).
- Q2 (whitespace semantics in bullets) drives 2 clauses (REQ-SC-21, -22).

A different decision on Q1 would invalidate the proof shape for 5 of 25 Verus specs and
3 of 8 proptest properties. **INFERRED status is not acceptable for this module
specifically** because the central predicate of the state machine is itself one of the
open questions.

**Recorded as finding F-BLOCK-001** with `disposition: blocker`. Resolution requires
`rust-contract` to author `clarity-web/src/domain/contract.md`.

---

### Q3 — Lane discipline (25 V + 8 P)

**Sound.** TLA+ rejection (§4 row T) is correctly argued: the state machine is
predicate-driven over immutable values, with no internal mutable state, no event log,
no scheduler, no retries, no leases, no batch ordering. A TLA+ spec would describe
client-side sequencing, not module behaviour, and would be vacuous for the
module-internal guarantees.

Flux downgrade (§4 row F) is correctly argued: `Option<String>` and unconstrained
`String` cannot directly carry refinement indices without type restructure. Verus
covers the same predicates at lower author cost.

**Density check:** 25 V + 8 P = 33 obligations against ~323 production LOC = 1 obligation
per ~10 LOC. For a pure-data state-machine module where every function is a transition,
constructor, or predicate, this is **below the typical ratio** (compare 41 obligations
on `domain/newtypes.rs` at ~270 LOC ≈ 1 per 6.6 LOC, cl-0n6). The 25 V + 8 P
allocation is appropriate.

**Kani, Loom, Miri, Z (fuzz), X (exercise-only)** — all `not_applicable` decisions are
sound. Each cites concrete evidence (no unsafe, no concurrency, no parser, no fuzzable
boundary). All 9 lane decisions are accepted in `verifier-lane-review.jsonl`.

---

### Q4 — Five open refinement questions (plan §3)

**Two are behavior-affecting and BLOCKING.** The plan §9 item 2 labels all five as
"NON-BLOCKING but recommended". This is wrong.

| Q | Affects | Behavior-affecting? | Verdict |
|---|---|---|---|
| Q1 | REQ-SC-11, -12, -14, -15, -16 | **YES** — central predicate of L5, L6, L13, L14, L15, L16 | **BLOCKING** |
| Q2 | REQ-SC-21, -22 | **YES** — predicate definition changes for bullets_complete | **BLOCKING** |
| Q3 | REQ-SC-7 | No — additive (named constants vs literals) | non-blocking |
| Q4 | REQ-SC-8 | No — boolean comparison, proof shape unchanged | non-blocking |
| Q5 | REQ-SC-14 | No — L1/L2 hold under all three signatures | non-blocking |

**Recorded as finding F-BLOCK-002** with `disposition: blocker`.

---

### Q5 — L8 / L9 verification

**Both hold.** Source inspection confirms:

- **L8 (HoleType::all canonical):** Source lines 37-43 return `&[Self::DiscoveryHole,
  Self::EdgeCaseHole, Self::MotivationDropOff]`. Length 3. Variants are exactly the
  three declared variants. **Plan claim is sound.**
- **L9 (with_severity clamp [1, 5]):** Source line 107: `severity: severity.clamp(1, 5)`.
  `u8::clamp` is the Rust stdlib contract: `self.clamp(min, max)` returns `min` if
  `self < min`, `max` if `self > max`, else `self`. For `min=1, max=5` and `self ∈ 0..=255`:
  - `self == 0 → 1` (min)
  - `self == 1 → 1`
  - `self == 5 → 5`
  - `self == 6 → 5` (max)
  - `self == 255 → 5` (max)

All four boundary cases asserted by REQ-SC-7 and PO-SC-V-07 hold. **Plan claim is sound.**

The plan does not assert L9 for `s > 5` exhaustively — it asserts the boundary set
{0, 1, 5, 6, 254, 255} and the postcondition `1 ≤ result ≤ 5`. The postcondition is
stronger than any single boundary; the boundary asserts are belt-and-braces. **Sound.**

---

### Q6 — Clippy / lint impact

**Plan claim is accurate for production code, with one exposure acknowledged for proptest additions.**

- **`scenario.rs` production code (lines 1-323) is panic-free.** Full-text grep
  confirmed zero `.unwrap()`, `.expect(`, `panic!`, `todo!`, `unimplemented!`
  invocations in production lines. Only matches are the module-level
  `#![warn(clippy::unwrap_used)]` etc. attributes (lines 1-5) and the line 101 doc
  comment "This function does not panic."
- **Test module (lines 325-633)** has two `.unwrap_or_default()` calls (lines 535, 628)
  in the existing tests. These are clippy-exempt via the module-level
  `#![allow(clippy::unwrap_used, clippy::expect_used, ...)]` at lines 326-340.
- **`scenario.rs` lint configuration:** `#![warn(clippy::unwrap_used)]`,
  `#![warn(clippy::expect_used)]`, `#![warn(clippy::panic)]` (lines 1-3) **override** the
  parent `domain/mod.rs` deny rules (lines 7-9). Production code is at warn level, not
  deny. Test module opens the lints entirely.
- **`pedantic` and `nursery`** are at warn level (lines 4-5). New proptest bodies using
  common proptest idiom (e.g. `vec![]`, `String::new()`, named field struct literals)
  may trip pedantic-friction warnings.
- **cl-2vr exposure (clarity-web/Cargo.toml no `[lints]` opt-in)** is **bounded to
  zero** for `scenario.rs`. Plan §9 item 4 correctly identifies this.

The plan's claim "scenario.rs production function bodies contribute zero to the cl-2q6
baseline" is **verified accurate**. The plan §9 item 4 acknowledgement that the
proptest submodule needs `#[allow(...)]` is honest but does not enumerate the specific
lints. **Recorded as finding F-BLOCK-008** with `disposition: blocker`.

---

## 4. Lane review summary

9 lanes reviewed, all accepted. See `proofs/scenario-verifier-lane-review.jsonl`:

| Lane | Decision | Disposition | Notes |
|---|---|---|---|
| V (Verus) | required | accepted | 25 obligations; correct primary lane per §5.1 |
| P (proptest) | required | accepted | 8 obligations; serde + algebraic laws |
| F (Flux) | not_applicable (downgrade) | accepted | type-representation mismatch is principled |
| K (Kani) | not_applicable | accepted | no unsafe, no fixed-width, no parser |
| L (Loom) | not_applicable | accepted | no concurrency surface |
| M (Miri) | not_applicable | accepted | forbid(unsafe_code) at module + workspace |
| T (TLA+) | not_applicable | accepted | predicate-driven over immutable values; reserved for intent/* + storage/fjall |
| Z (fuzz) | not_applicable | accepted | structured serde_json boundary; fuzz is upstream in intent/parser.rs |
| X (exercise-only) | not_applicable | accepted | whole module in scope for V+P |

Each row carries `planner_invocation_id: cl-zup-proof-planner-2026-06-21T20:49:32Z` and
`reviewer_invocation_id: cl-zup-proof-plan-reviewer-2026-06-21T22:15:00Z-a1b2c3d4`.
No self-stamped fields.

---

## 5. Findings summary

| Code | Severity | Disposition | Artifact | Summary |
|---|---|---|---|---|
| `E_CONTRACT_INFERRED` | blocker | blocker | plan §3 | 26 INFERRED clauses; no rust-contract artifact |
| `E_OPEN_QUESTION_BEHAVIOR_AFFECTING` | blocker | blocker | plan §3 + §9 | Q1/Q2 mislabeled non-blocking |
| `E_TRUSTED_BASE_NOT_LEDGERED` | blocker | blocker | plan §6 | trusted-base prose, no JSONL ledger |
| `E_BRIDGE_NOT_PRESTAGED` | blocker | blocker | plan §8 | no rust-refinement-obligation JSONL |
| `E_TRUSTED_BASE_INTERNAL_CONTRADICTION` | blocker | blocker | plan §6 row 8 | `#[must_use]]` line citation wrong + self-contradictory |
| `E_OBLIGATION_BUNDLING` | blocker | blocker | plan §5 row PO-SC-V-23 | 3 functions in 1 obligation |
| `E_SCHEMA_DRIFT` | blocker | blocker | obligations JSONL | `proof-obligations/v1` should be `proof-obligation/v1` |
| `E_CLIPPY_PROPTEST_EXPOSURE` | blocker | blocker | plan §9 item 4 | specific lint allow list not enumerated |
| `E_TRIVIAL_PROOF_CLAIM` | minor | owner_approved_debt | plan §5 row PO-SC-V-18 | idempotence sub-claim trivially true; cost-benefit acceptable |
| `E_PLAN_LOC_CLAIM_CORRECTED` | observation | owner_approved_no_action | plan header | LOC ~634/~323 verified accurate |
| `E_STATE_MACHINE_ANALYSIS_VERIFIED` | observation | owner_approved_no_action | plan §1.1 | L1-L9 laws verified accurate |
| `E_CL2VR_EXPOSURE_BOUNDED` | observation | owner_approved_no_action | plan §9 | scenario.rs panic-free in production |
| `E_LANE_DISCIPLINE_SOUND` | observation | owner_approved_no_action | plan §4 | TLA+ rejection principled; density appropriate |

**13 findings total: 8 blockers + 1 approved debt + 4 observations.**

---

## 6. Re-submission path

The repair guide `proofs/scenario-plan-repair-guide.md` enumerates exact repair steps.
The smallest state to rerun is:

1. **rust-contract** authors `clarity-web/src/domain/contract.md` with explicit decisions
   on Q1 and Q2 (resolves F-BLOCK-001 and F-BLOCK-002 in one step).
2. **proof-planner** updates `proofs/scenario-proof-plan.md`:
   - §3: clause_origin AUTHORED for all 26
   - §9 item 2: reclassify Q1/Q2 as BLOCKING
   - §6 row 8: line citation 198, remove self-contradiction
   - §5: split PO-SC-V-23 into V-23a/b/c
   - §9 item 4: enumerate specific lint allow list
3. **proof-planner** updates `proofs/scenario-obligations.planned.jsonl`:
   - schema_version: `proof-obligation/v1` (singular)
   - clause_origin: AUTHORED; requires_contract: false on rows 1-33
   - 2 new rows (V-23b, V-23c) replacing V-23
4. **proof-planner** produces `proofs/scenario-trusted-base.jsonl` (10 rows).
5. **proof-planner** produces `proofs/scenario-refinement-obligations.planned.jsonl`
   (27 rows: V-01..V-25 plus V-23b/c).
6. Re-submit to `proof-plan-reviewer`.

---

## 7. Workspace context

Per `formal-verification-report.md §3` and `verification-targets.md §10`:

- The contract gap (F-BLOCK-001) is workspace-wide; no rust-contract artifact exists for
  any of the 8 modules in flight (cl-0n6, cl-zup, cl-5dp, cl-ooz, cl-vv2, cl-kse,
  cl-54n, cl-dv5).
- `cl-2vr` (P0 critical): clarity-web/Cargo.toml has no `[lints]` opt-in; workspace deny
  rules are inert; 901 panic-prone sites exist. **scenario.rs production code is
  panic-free (verified)**, so this does not block this plan specifically but blocks the
  workspace.
- `cl-2q6`: clippy debt baseline. **scenario.rs production code contributes zero to
  the baseline (verified)**.

The 8 blocker findings + 1 approved debt fall into three ownership buckets:

| Bucket | Findings | Owner |
|---|---|---|
| Contract-source blocking | F-BLOCK-001, F-BLOCK-002 | rust-contract |
| Machine-readable artifact gap | F-BLOCK-003, F-BLOCK-004, F-BLOCK-007 | proof-planner |
| Plan defect / rigor gap | F-BLOCK-005, F-BLOCK-006, F-BLOCK-008, F-DEBT-001 | proof-planner |

The first bucket cannot be resolved within the cl-zup lane alone. The bead should be
flagged as `blocked` on rust-contract output rather than re-cycled through
proof-plan-reviewer.

---

## 8. Final verdict

The plan is well-structured, honest about its gaps, and strong on the state-machine
analysis (L1-L9 laws verified accurate). However, **8 blocker findings** prevent
advancement to `proof-writer`:

1. Contract clauses are inferred rather than authored (F-BLOCK-001, `E_CONTRACT_INFERRED`).
2. Two open questions are behavior-affecting and mislabeled non-blocking (F-BLOCK-002, `E_OPEN_QUESTION_BEHAVIOR_AFFECTING`).
3. Trusted-base ledger is prose, not JSONL (F-BLOCK-003, `E_TRUSTED_BASE_NOT_LEDGERED`).
4. Bridge rows not pre-staged (F-BLOCK-004, `E_BRIDGE_NOT_PRESTAGED`).
5. Internal contradiction in trusted-base §6 row 8 (F-BLOCK-005, `E_TRUSTED_BASE_INTERNAL_CONTRADICTION`).
6. Three functions bundled into one obligation (F-BLOCK-006, `E_OBLIGATION_BUNDLING`).
7. Schema version drift (F-BLOCK-007, `E_SCHEMA_DRIFT`).
8. Specific proptest lint allow list not enumerated (F-BLOCK-008, `E_CLIPPY_PROPTEST_EXPOSURE`).

`STATUS: REJECTED`

*End of review.*