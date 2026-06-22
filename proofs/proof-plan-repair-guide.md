# Proof Plan Repair Guide — `proofs/behavior-proof-plan.md` (cl-ooz)

**Verdict:** REJECTED. Algebraically correct, structurally incomplete.

This document names the **smallest state to rerun** the review successfully. It assumes no work has been done yet on `cl-ooz`'s proof artifacts (no Verus specs, no proptest, no trusted-base ledger).

---

## 1. The two blockers (must clear before proof-writer)

### Blocker F1 — `rust-contract` artifact missing

**What:** No `contract.md` (or equivalent) exists for `clarity-web/src/intent/types/`. All 14 INFERRED clauses (REQ-BH-1..REQ-BH-14) need ratification.

**Owner:** `rust-contract` skill.

**Smallest fix:**
1. `rust-contract` reads `clarity-web/src/intent/types/behavior.rs` (236 LOC).
2. `rust-contract` reads `clarity-web/src/intent/types/{type_error,feature,spec,verification}.rs` for context.
3. `rust-contract` authors `clarity-web/src/intent/types/contract.md` with one clause per REQ-BH-* ID, each tagged `clause_origin: AUTHORED` (vs the plan's INFERRED).
4. `rust-contract` answers the 3 open algebraic decisions in plan §3:
   - **Q1**: `MAX_PRECONDITIONS = MAX_POSTCONDITIONS = 20` — contract or impl detail? (Recommended: **contract**; source evidence at type_error.rs:58-63 supports this.)
   - **Q2**: `with_verification` — replacement vs accumulate? (Source makes replacement unambiguous; ratify as **replacement**.)
   - **Q3**: `add_precondition(&mut self) -> &mut Self` — part of public contract? (Ratify as **yes** — fluent builder is the public API; tests must chain.)
5. `rust-contract` updates or corrects each clause as needed. The 14 requirement IDs and their L1–L7 law tags should remain stable; the clause text may change.

**Acceptance evidence:** `clarity-web/src/intent/types/contract.md` exists; each clause has `clause_origin: AUTHORED`; all 14 REQ-BH-* IDs are mapped.

### Blocker F2 — JSONL schema non-compliance

**What:** 8 schema-required fields are missing on every one of 21 obligation rows.

**Owner:** `proof-planner` (or whoever regenerates the obligations JSONL).

**Smallest fix:** regenerate `proofs/behavior-obligations.planned.jsonl` with the following additions per row:

| Field | Example value |
|---|---|
| `schema_version` | `"proof-obligation/v1"` |
| `domain_claim` | A one-line claim distinct from `contract_clause` (e.g., for PO-V1: `"The snake_case name predicate is total over &str and equivalent to first-ascii-lowercase ∧ all-in-{lowercase,digit,_}."`) |
| `risk_tags` | Array of tags (e.g., `["rust_local_invariant","boundary"]` for PO-V8) |
| `workdir` | `"/home/lewis/src/clarity"` for all rows |
| `model_bounds` | Object — `{}` for unbounded Verus proofs; `{ "Vec.len": "usize, <= MAX+1" }` for PO-P1; `{ "Vec.len": "usize, <= 21" }` for PO-V13 |
| `tool_metadata` | Object — `{ "verus_version": "0.2026.05.05", "verus_path": "/home/lewis/.local/bin/verus" }` for V rows; `{ "proptest_version": "default workspace" }` for P rows; `{ "kani_version": "not installed" }` for K rows; etc. |
| `trusted_base_refs` | Array of `trusted-base-ledger/v1` row IDs (empty for now; populated when trusted-base-ledger.jsonl is authored in F5) |
| `behavior_affecting` | `true` for PO-V1..V13 (they constrain production behavior); `true` for PO-P1 (round-trip affects observable behavior); `false` for PO-K1/F1/L1/M1/T1/Z1/X1 (NA rows do not produce behavior) |

**Acceptance evidence:** Every one of the 21 rows validates against `proof-obligation/v1` schema. Spot-check: `jq '. | select(.schema_version == null)' proofs/behavior-obligations.planned.jsonl` returns zero rows.

**Schema validation note:** the sibling plans (`storage-types`, `newtypes`, `scenario`) have the same omissions. If the project intends this as a project-wide convention, the schema should be relaxed to make these fields optional at planning time and required at execution time. Either is acceptable; the JSONL must conform to the published schema at the stage it claims to be at.

---

## 2. Non-blocking debt (acceptable to carry)

These five items do not block approval once F1 and F2 are fixed. Proof-writer should address them during the write phase.

| ID | Owner | Action |
|---|---|---|
| F3 | proof-writer | Add `assert!(is_valid_behavior_name(&name))` to `arb_behavior()` strategy in PO-P1 |
| F4 | proof-writer | Define `arb_verification()` sub-strategy and use `prop::option::of(arb_verification())` in PO-P1 |
| F5 | proof-writer | Produce `trusted-base-ledger.jsonl` alongside Verus specs (one row per §6 anchor + rows for `#[verifier::external_body]` markers) |
| F6 | proof-writer | Rewrite PO-V9 ensures clause as field-by-field extensional equality (5 conjuncts) instead of evaluation-order argument |
| F7 | proof-writer | Ensure `test_add_precondition_order` in bridge (PO-V11) chains via the fluent `&mut Self` return — plan §8 already commits |
| F10 | infra | Host control plane populates `agent-invocation-ledger.jsonl` with `proof-planner` and `proof-plan-reviewer` invocation rows |

---

## 3. Non-blocking observations (no fix needed at this stage)

| ID | Disposition | Reason |
|---|---|---|
| F8 | owner_approved_no_action | Validator composition gap at feature/spec boundary is out of scope for behavior.rs; track for future feature.rs/spec.rs plans |

---

## 4. State machine for re-review

The reviewer can re-review **only after F1 and F2 are fixed**. The rerun will:

1. Re-read `proofs/behavior-proof-plan.md` (now with contract references in each REQ-BH-* row).
2. Re-validate the JSONL against `proof-obligation/v1` schema (now conformant).
3. Re-trace each law L1–L7 against source (already done; no change).
4. Re-check the 3 open algebraic decisions (now resolved by rust-contract ratification).
5. Issue **STATUS: APPROVED** if all checks pass.

The reviewer will **not** redo the algebraic law verification (§2 of the plan-review.md) since L1–L7 all held and the source is unchanged.

**Smallest rerun command:**
```bash
# After F1 and F2 are fixed:
# 1. rust-contract authors clarity-web/src/intent/types/contract.md
# 2. proof-planner regenerates proofs/behavior-obligations.planned.jsonl with 8 added fields
# 3. proof-plan-reviewer re-runs (this skill) against the updated artifacts
# 4. If clean, status flips to APPROVED and proof-writer proceeds.
```

---

## 5. What NOT to do

- **Do not** advance to proof-writer with the current JSONL. The 8 missing fields will cause schema validation to fail downstream.
- **Do not** strip the `INFERRED` tag from clauses to "fix" F1 — the tag is honest; the fix is to ratify.
- **Do not** treat `requires_contract: true` as a runtime gate. It is an indicator only.
- **Do not** produce `trusted-base-ledger.jsonl` rows before F5 is actioned; F5 is owner-approved debt, not a blocker.
- **Do not** modify `clarity-web/src/intent/types/behavior.rs` source. The clippy debt at lines 213/227 is owned by `cl-2q6`, not `cl-ooz`. The plan correctly disowns it.

---

## 6. Files in this review

| Path | Purpose |
|---|---|
| `proofs/behavior-plan-review.md` | The full review (this file's companion) |
| `proofs/verifier-lane-review.jsonl` | 21 `verifier-lane-review/v1` rows (one per obligation) — **all `accepted`** |
| `proofs/proof-plan-findings.jsonl` | 10 `finding/v1` rows (2 blocker + 5 major + 3 minor) |
| `proofs/proof-plan-repair-guide.md` | This file |
| `.beads/cl-ooz/plan-review-report.md` | Bead-level summary |

---

END OF REPAIR GUIDE.
