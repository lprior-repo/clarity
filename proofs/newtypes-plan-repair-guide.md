# Proof Plan Repair Guide — `cl-0n6` / `domain/newtypes.rs`

**Bead:** `cl-0n6`
**Source review:** `proofs/newtypes-plan-review.md` (`STATUS: REJECTED`)
**Owner:** `proof-planner` (replan)
**Date:** 2026-06-21

This document is the smallest set of changes to bring the plan from
`STATUS: REJECTED` to `STATUS: APPROVED-WITH-CONDITIONS`. There are 9 changes
across 2 blockers and 7 non-blocking findings.

---

## Change 1 (BLOCKER) — Fix obligations JSONL schema drift

**File:** `proofs/newtypes-obligations.planned.jsonl`
**Finding:** F-BLOCK-1 (`E_SCHEMA_VERSION_MISSING` + `E_SCHEMA_MISSING_FIELD`)

**What to change:**

For every one of the 41 rows, do **all three**:

1. Rename `schema_version` from `"proof-obligations/v1"` to `"proof-obligation/v1"`
   (singular). This is the canonical schema per
   `go-skill/references/proof-schemas.md §Required Schema Versions`.

2. Add a `domain_claim` field — a one-sentence plain-English statement of the
   property being verified. Suggested values:

   | Row(s) | domain_claim |
   |---|---|
   | PO-NT-V-01 | `AnswerId::try_from is the validator that makes whitespace-only inputs unrepresentable inside AnswerId` |
   | PO-NT-V-02 | `StepId::try_from is the validator that makes whitespace-only inputs unrepresentable inside StepId` |
   | PO-NT-V-03 | `BeadId::try_from is the validator that makes whitespace-only inputs unrepresentable inside BeadId` |
   | PO-NT-V-04..V-06 | `FromStr is equivalent to try_from(s.to_string()) for the validating newtypes (delegation contract)` |
   | PO-NT-V-07 | `Timestamp::try_from enforces RFC 3339 compliance via chrono::DateTime::parse_from_rfc3339` |
   | PO-NT-V-08 | `Timestamp::from_str delegates to Timestamp::try_from (delegation contract)` |
   | PO-NT-V-09..V-12 | `From<X> for String extracts the inner String byte-for-byte (identity conversion)` |
   | PO-NT-V-13..V-16 | `as_str returns the inner String as a str slice (observer contract)` |
   | PO-NT-V-17..V-21 | `Display::fmt writes the inner String verbatim (Display delegation contract)` |
   | PO-NT-V-22 | `AnswerValue is an unconstrained newtype; constructors/observers preserve inner String verbatim` |
   | PO-NT-V-23 | `From<String>, From<&str>, From<AnswerValue> for String are identity conversions` |
   | PO-NT-V-24 | `Timestamp::now produces an RFC 3339-formatted timestamp via chrono wall-clock` |
   | PO-NT-V-25 | `Timestamp::default delegates to Timestamp::now (delegation contract)` |
   | PO-NT-P-01..P-03 | `Whitespace-only inputs are rejected; non-whitespace inputs are preserved verbatim (property)` |
   | PO-NT-P-04 | `FromStr and try_from(s.to_string()) agree on all inputs (property equivalence)` |
   | PO-NT-P-05 | `Timestamp::try_from rejects non-RFC-3339 inputs and accepts valid RFC 3339 with byte-preservation` |
   | PO-NT-P-06 | `Timestamp::default round-trips via chrono::DateTime::parse_from_rfc3339 (wall-clock property)` |
   | PO-NT-P-07 | `format!("{}", t) equals t.as_str() for all five newtypes (Display round-trip property)` |
   | PO-NT-P-08 | `AnswerValue round-trips through new/as_str/From/String (identity property)` |
   | PO-NT-P-09 | `serde_json round-trips identity for each newtype on well-formed input (boundary property)` |
   | PO-NT-K1, F1, L1, M1, T1, Z1, X1 | `(not_applicable)` |

3. **Optional but recommended:** collapse the redundant `id` / `obligation_id`
   duplication. Either keep `id` (canonical per schema) and remove `obligation_id`,
   OR keep `obligation_id` and remove `id`. The schema requires `id`; remove
   `obligation_id` to match.

**Verification:** Run a JSON-schema validator (e.g., `jsonschema -i
newtypes-obligations.planned.jsonl proof-obligation-v1.schema.json` if available;
otherwise grep for `proof-obligations/v1` and `domain_claim` and confirm zero
hits / 41 hits respectively).

---

## Change 2 (BLOCKER) — Resolve test-body lint conflict

**File:** `proofs/newtypes-proof-plan.md` + `proofs/newtypes-obligations.planned.jsonl`
**Finding:** F-BLOCK-2 (`E_BEHAVIOR_TEST_NOT_INDEPENDENT` variant)

**The problem:** `clarity-web/src/domain/mod.rs:7-9` unconditionally denies
`clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` at module level.
These apply to `#[cfg(test)] mod tests` inside `domain/newtypes.rs`. PO-NT-P-09
currently implies test bodies that use `.expect(...)`.

**Choose ONE of the following three strategies (RECOMMENDED: strategy c):**

### Strategy (a) — Use `.unwrap_or_*` instead of `.expect(...)`

**File:** `proofs/newtypes-obligations.planned.jsonl`
**Row:** PO-NT-P-09
**Change:** In `expected_evidence`, replace
`from_str::<T>(&to_string(&t).expect(...))` with
`from_str::<T>(&to_string(&t).unwrap_or_default())` (or
`.unwrap_or_else(|_| String::new())`).

**Effect:** Same property assertion, no `.expect()` call, no compile error.

### Strategy (b) — Add `#[allow]` attributes to test functions

**File:** `clarity-web/src/domain/newtypes.rs` (the `#[cfg(test)] mod tests`
submodule that proof-writer adds)
**Change:** Add `#[allow(clippy::expect_used, clippy::unwrap_used)]` to each
proptest function that needs the calls.

**Effect:** Compiles; explicit acknowledgment of the exception. The allow
attribute is on the test function only, not the production code.

### Strategy (c) — Move proptest file outside `domain/` (RECOMMENDED)

**File:** New `clarity-web/tests/newtypes_proptest.rs`
**Change:** proof-writer creates the proptest file at this path instead of
inside `domain/newtypes.rs`. The `domain/` module-level denies do not apply
because the file is outside the module tree.

**Effect:** Cleanest fix — test isolation from production-module lints.

**Update plan & obligations to reflect choice:**

- `proofs/newtypes-proof-plan.md` §5 row PO-NT-P-09 `target_module` field:
  change from `clarity-web/src/domain/newtypes.rs` to
  `clarity-web/tests/newtypes_proptest.rs` (if strategy c).
- `proofs/newtypes-proof-plan.md` §11 pre-flight checklist: update
  `cargo test -p clarity-web --lib domain::newtypes -- ...` to
  `cargo test -p clarity-web --test newtypes_proptest -- ...` (if strategy c).
- `proofs/newtypes-obligations.planned.jsonl` row PO-NT-P-09: update
  `artifact`, `target_module`, `command`, `workdir`, and `expected_evidence_command`
  to reflect the chosen strategy.

---

## Change 3 (non-blocking) — Strengthen Flux non-applicability rationale

**File:** `proofs/newtypes-proof-plan.md`
**Finding:** F-NB-1 (`E_LANE_DECISION_WEAK`)

**What to change:** §4 row "F (Flux)". Replace the rationale with a
concrete module-property statement:

> **F (Flux) — NOT APPLICABLE.** No refinement predicate in
> `clarity-web/src/domain/newtypes.rs` is expressible in Flux but not Verus.
> All five newtypes have at most one refinement shape (`!s.trim().is_empty()`
> for AnswerId/StepId/BeadId; `parse_from_rfc3339(&s).is_ok()` for Timestamp;
> none for AnswerValue), and Verus already pins each via `ensures`. The
> `const fn` body of `AnswerValue::new` cannot be refined at the Flux level
> because Flux does not spec `const fn` returns. `cargo-flux` is installed but
> unused for this module.

---

## Change 4 (non-blocking) — Add compensating-evidence cross-reference for Display obligations

**File:** `proofs/newtypes-obligations.planned.jsonl`
**Finding:** F-NB-5 (`E_PROOF_PLAN_MISSING_NONVACUITY`)

**What to change:** For each of PO-NT-V-17, PO-NT-V-18, PO-NT-V-19, PO-NT-V-20,
PO-NT-V-21, add a field:

```json
"compensating_evidence_refs": ["PO-NT-P-07"]
```

**Effect:** Closes the vacuity loop — the 5 `external_body` Display obligations
now have an explicit pointer to their executable compensating evidence.

---

## Change 5 (non-blocking) — Fix PO-NT-V-25 `verus_mode`

**File:** `proofs/newtypes-obligations.planned.jsonl`
**Finding:** F-NB-4 (`E_VERIFIER_MODE_INCONSISTENCY`)

**What to change:** Row PO-NT-V-25 `verus_mode`: change from `"exec"` to
`"exec+extern_spec"` (matches PO-NT-V-24 and reflects the extern_spec on
chrono::Utc::now and chrono::DateTime::to_rfc3339).

---

## Change 6 (non-blocking) — Acknowledge `cl-2vr` in §9 Blockers

**File:** `proofs/newtypes-proof-plan.md`
**Finding:** F-NB-3 (`E_REVIEW_STATUS_MISSING`)

**What to change:** Append to §9 Blocker #4 (currently: "Clippy debt (cl-2q6)
is independent of this plan..."):

> **Note on cl-2vr (CRITICAL P0, per formal-verification-report.md §3).**
> The workspace `[lints]` opt-in is missing for `clarity-web`, so workspace deny
> lints are inert and 901 panic-prone sites exist. For `domain/newtypes.rs`
> specifically, the module-level panic-freedom assumption is independently
> defensible: the source has zero `.unwrap()`, `.expect()`, `panic!`, `todo!`,
> `unimplemented!`, or `unsafe` sites across its 307 lines. The panic-freedom
> basis is the module inventory, not the (inert) workspace lint config. Closure
> of cl-2vr itself is owned by `holzman-rust` and tracked as a separate P0 bead.

---

## Change 7 (non-blocking) — Cross-reference PO-NT-V-25 from PO-NT-V-24

**File:** `proofs/newtypes-obligations.planned.jsonl`
**Finding:** F-NB-6 (`E_PROOF_PLAN_MISSING_NONVACUITY` mild)

**What to change:** Append to PO-NT-V-25 `notes` field:

> Structurally duplicates PO-NT-V-24 (Timestamp::default delegates to
> Timestamp::now at line 305). Same property; separate row for coverage of
> the Default trait surface.

---

## Change 8 (non-blocking) — Clean up PO-NT-P-09 evidence

**File:** `proofs/newtypes-obligations.planned.jsonl`
**Finding:** F-NB-7 (`E_COMMAND_EVIDENCE_MISSING`)

**What to change:** PO-NT-P-09 `expected_evidence` field. Replace with a
single concise statement:

> `proptest reports 256+ cases passing: for each newtype T and each
> well-formed input t, serde_json::from_str::<T>(&serde_json::to_string(&t))
> returns Ok(t.clone()).`

And move the original Rust snippet to a new field:

```json
"evidence_snippet": "from_str::<T>(&to_string(&t).expect(\"serde_json::to_string is total\")) == Ok(t.clone())"
```

---

## Change 9 (informational) — Add planner_invocation_id

**File:** host control plane (out of band)
**Finding:** F-NB-8 (`E_REVIEW_PROVENANCE_MISSING`)

**What to change:** Host control plane should record
`planner_invocation_id` for each proof-planner session. The current workspace
Markdown header lacks this field. This is a host-control-plane concern, not
a proof-planner artefact change.

**If the host control plane is not yet wired**, the proof-planner can add a
JSONL sidecar `proofs/newtypes-planner-invocation.jsonl` with one row per
planner session. Not blocking.

---

## After all 9 changes — smallest re-review state

After applying changes 1 and 2 (the two blockers), the plan should be
re-submitted for plan-review. The reviewer will re-run validation:

1. Schema validator passes on all 41 obligation rows (change 1).
2. The proptest strategy choice (a/b/c from change 2) is documented in
   both the plan and the obligations, and the test command reflects it.
3. Plan §9 Blocker #4 cites `cl-2vr` (change 6).
4. Other non-blocking changes are present.

Expected verdict: `STATUS: APPROVED-WITH-CONDITIONS` (the condition being
`rust-contract` ratifying the 16 INFERRED clauses in §3 of the plan before
proof-writer runs — this is a parallel workstream, not a plan-replan dependency).

---

## Validation checklist

Before re-submitting, run:

```bash
# 1. Schema drift check
rg -c '"schema_version":"proof-obligations/v1"' proofs/newtypes-obligations.planned.jsonl
# Expected: 0 (was 41)

rg -c '"schema_version":"proof-obligation/v1"' proofs/newtypes-obligations.planned.jsonl
# Expected: 41

rg -c '"domain_claim"' proofs/newtypes-obligations.planned.jsonl
# Expected: 41 (was 0)

# 2. Lint-conflict check (after strategy c — recommended)
rg -n '\.expect\(' proofs/newtypes-proof-plan.md
# Should appear only in §3 mention of design choice (if any) and Change 2 of the repair guide, not in PO-NT-P-09 expected_evidence.

# 3. cl-2vr acknowledgement
rg -n 'cl-2vr' proofs/newtypes-proof-plan.md
# Expected: ≥1

# 4. Display cross-reference
rg -c '"compensating_evidence_refs":\["PO-NT-P-07"\]' proofs/newtypes-obligations.planned.jsonl
# Expected: 5

# 5. V-25 mode fix
rg '"verus_mode":"exec",' proofs/newtypes-obligations.planned.jsonl
# Should return ONLY PO-NT-V-22, V-23, V-13..V-16 (the truly exec-only obligations) — NOT PO-NT-V-25
```

If any check fails, do not re-submit. Fix and re-validate.

---

## Re-review timing

Re-submit to `proof-plan-reviewer` with a note in the plan header:

> *Re-submission. Addresses F-BLOCK-1 (schema drift) and F-BLOCK-2 (test-body
> lint conflict) from the previous review (`STATUS: REJECTED`). Changes 3-9
> also applied. See `proofs/newtypes-plan-repair-guide.md` for diffs.*

The reviewer will re-run schema validation, lane-decision acceptance, and
vacuity checks. Expected outcome: `STATUS: APPROVED-WITH-CONDITIONS`.

---

*End of repair guide.*
