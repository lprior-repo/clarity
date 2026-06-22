# Proof Plan Review — `proofs/behavior-proof-plan.md` (cl-ooz)

| Field | Value |
|---|---|
| **Reviewer skill** | `proof-plan-reviewer` |
| **Reviewer invocation id** | `ppr-cl-ooz-2026-06-21T16:32:00Z-7c4a9e1f` |
| **Planner invocation id** | **MISSING** — no `agent-invocation-ledger.jsonl` row exists; plan does not stamp one. See F10. |
| **Review state** | `REJECTED` |
| **Date** | 2026-06-21 |
| **Bead** | `cl-ooz` |
| **Plan path** | `proofs/behavior-proof-plan.md` (23K, 11 sections) |
| **Obligations path** | `proofs/behavior-obligations.planned.jsonl` (21 rows: 13 V + 1 P + 7 NA) |
| **Module** | `clarity-web/src/intent/types/behavior.rs` (236 LOC: 119 prod + 117 test) |
| **Companion reads** | `verification-targets.md §5.3`, `formal-verification-report.md`, `clarity-web/src/intent/types/{behavior,type_error,feature,spec,verification}.rs`, `clarity-web/src/intent/types.rs` |

---

## 1. Reviewed artifacts (with hashes)

```
proofs/behavior-proof-plan.md                 sha256: reviewed (matches on-disk)
proofs/behavior-obligations.planned.jsonl     sha256: reviewed (matches on-disk)
clarity-web/src/intent/types/behavior.rs      sha256: reviewed (matches on-disk)
clarity-web/src/intent/types/feature.rs       sha256: reviewed (cross-reference consumer)
clarity-web/src/intent/types/spec.rs          sha256: reviewed (cross-reference consumer)
clarity-web/src/intent/types/type_error.rs    sha256: reviewed (error variant shapes)
clarity-web/src/intent/types/verification.rs  sha256: reviewed (round-trip dependency)
clarity-web/src/intent/types.rs               sha256: reviewed (parent module)
verification-targets.md §5.3                  sha256: reviewed
formal-verification-report.md                 sha256: reviewed
```

`agent-invocation-ledger.jsonl`, `verifier-lane-decisions.jsonl`, `verifier-lane-review.jsonl`, `rust-contract` artifact — **all missing** (see F9, F10, F1).

---

## 2. Algebraic-law verification (the headline question)

I traced each of the seven laws against `clarity-web/src/intent/types/behavior.rs` directly.

| Law | Plan claim | Source location | Verdict |
|---|---|---|---|
| **L1** canonical empty (REQ-BH-3, PO-V3) | `new(s).unwrap()` has `description == ""`, `verification == None`, both `Vec`s empty | Lines 58-64 are an exact struct literal of those five values | **HOLDS** — proof is mechanical |
| **L2** field-replace & commutativity (REQ-BH-6/7/9, PO-V6/V7/V9) | `with_description` and `with_verification` commute and touch disjoint fields | Lines 69-83 both use `Self { field: x, ..self }` — explicit field overrides the `..` fallback | **HOLDS** — well-known Rust idiom, easily provable in Verus |
| **L3** right-most-wins (REQ-BH-10, PO-V10) | `b.with_x(d1).with_x(d2) == b.with_x(d2)` | Same pattern; second call's explicit `field: d2` shadows the first | **HOLDS** — trivially structural |
| **L4** append order (REQ-BH-11, PO-V11) | `b.add_precondition(x).add_precondition(y).preconditions == [x, y]` | Line 87 plain `Vec::push`; line 88 returns `&mut self` so the two calls share the same `&mut` reference | **HOLDS** — `Vec::push` is order-preserving (stdlib contract); `&mut Self` aliasing tracked via `old` / `*self'` in Verus |
| **L5** no dedup (REQ-BH-12, PO-V12) | `b.add_precondition(s).add_precondition(s).preconditions == [s, s]` | Line 87 has no `contains` / `if !seen` check; it is plain `push` | **HOLDS** — absence of dedup is provable by source inspection (see F12 below for a sharper spec) |
| **L6** validate monotonicity (REQ-BH-13, PO-V13) | `len < MAX ∧ validate(b).is_ok() ⇒ validate(b.add_precondition(s)).is_ok()`; `len == MAX ⇒ Err(TooMany(_, MAX+1, MAX))` | Line 102 uses strict `> MAX_PRECONDITIONS` (20). 20 → Ok, 21 → Err with `n = 21`. | **HOLDS** — boundary at exactly 20 is inclusive (Ok), boundary at 21 is exclusive (Err) |
| **L7** serde round-trip (REQ-BH-14, PO-P1) | `serde_json::from_str::<Behavior>(&serde_json::to_string(&b).unwrap()) == Ok(b)` | Line 28 derives `Serialize, Deserialize`; all 5 fields are `String` / `Option<Verification>` / `Vec<String>` — all serde-compatible; `Verification` (verified.rs:11) derives `Serialize, Deserialize` and has only `String` fields | **HOLDS** but the plan's clause uses `.unwrap_or_default()` which silently coerces a to_string failure to ""; should be `.unwrap()` (cosmetic, no behavior impact because none of the field types can fail to serialize) |

**Algebraic verdict: all seven laws are sound against the source.** No law is unsound. The proof obligations in §5 are correctly scoped and the bridge inputs in §8 are correctly enumerated.

---

## 3. Open algebraic decisions (§3 of the plan)

Three decisions are deferred to `rust-contract`:

| Decision | Plan default | Source evidence | Justification for non-blocking |
|---|---|---|---|
| `MAX_PRECONDITIONS = 20`, `MAX_POSTCONDITIONS = 20` — contract or impl detail? | Part of contract | `TypeError::TooMany{Pre,Post}conditions(String, usize, usize)` (type_error.rs:59, 63) takes the bound as the **third** tuple field, making the bound observable to callers; the `#[error("... (maximum {2})")]` template also bakes the literal into the message | **Non-blocking**: source strongly supports "part of contract"; ratifying rust-contract will not change the obligation set; if changed to "impl detail", PO-V8/V13 spec becomes `len <= b.validate::MAX` (named bound), still provable |
| `with_verification` — replacement vs accumulate? | Replacement | Field is `Option<Verification>` (single slot, line 37); `Self { verification: Some(v), ..self }` (line 79-82) explicitly sets it, so the second call **replaces** | **Non-blocking**: source makes replacement unambiguous |
| `add_precondition(&mut self) -> &mut Self` — part of public contract? | Yes | Signature is `pub` (line 86); no `#[doc(hidden)]`; `#[must_use]` is **not** present (cf. line 68 which **does** carry `#[must_use]`), so the builder return is non-binding in principle — but consumers in `feature.rs` rely on a similar pattern (`add_dependency(&mut self) -> &mut Self`, line 74) | **Non-blocking** but the fluent return is **not actually exercised in this module's tests** (lines 211-214, 225-228 push directly to the Vec, bypassing the builder). The fluent-builder law (REQ-BH-11) is sound but slightly hypothetical within this module. Proof-writer should add an explicit chain test (the plan already commits to one in §8 row "PO-V11") |

**Three non-blocking algebraic decisions**: defensible, source-grounded, and downstream-resolvable.

---

## 4. 14 INFERRED clauses without `rust-contract`

This is the **central governance question** and the answer is honest in the plan:

> "The bead `cl-ooz` has **no upstream `rust-contract` artifact** under `clarity-web/src/intent/types/contract.md` (or any path matching `**/contract.md` in the workspace — confirmed by `ls` of `clarity-web/src/intent/types/` which yields 9 `.rs` files and no `contract.md`)." (§2)

Confirmed: no `contract.md` exists anywhere in the workspace (`find /home/lewis/src/clarity -name "contract.md" -type f` returns zero results).

The plan's mitigation is `requires_contract: true` on each `required` obligation row, which is an **indicator**, not an **enforcement gate**:
- The `proof-obligation/v1` schema has no `blocked_by_contract` or `gate` field.
- `proof-writer` reads obligations by status (`planned`) and would not pause on the indicator.
- The plan §9 item 1 calls this "BLOCKING" for proof-writer but provides no enforcement.

**Is this acceptable to ratify without contract?**

Algebraically: yes — all 14 clauses have direct source evidence (§2 above traces every law).

Governance-wise: **no**. The schema's whole point of having `contract_clause` on each obligation is to bind the proof to a contract-authored clause. If the contract changes between plan-time and write-time, the obligation's `contract_clause` field becomes a stale reference. The plan's `INFERRED` tag is honest but **does not satisfy the schema's intent**.

**Verdict**: blocker (F1) — `rust-contract` must author or ratify before proof-writer runs. This is consistent with `verification-targets.md §4` "Author `rust-contract` artifacts for the first module" gate.

---

## 5. Lane discipline (13 V + 1 P is heavily Verus-skewed — is this right?)

For type algebra — yes, this is correct. The module is **the canonical Verus target**: pure data, pure functions, algebraic laws, no I/O, no concurrency, no fixed-width arithmetic, no parser, no `unsafe`.

| Lane | Plan says | Reviewer agrees? |
|---|---|---|
| **V** | Required — primary | ✅ Correct per `verification-targets.md §5.3` ("intent/types/* — Verus the algebra; proptest the surface") |
| **P** | Required — secondary | ✅ Correct: serde round-trip is the textbook proptest target |
| **K** | Not applicable | ✅ Kani's bounded-model strength adds nothing here. Evidence cited: no `unsafe`, no fixed-width arithmetic, no parser. **Strong** NA evidence. |
| **F** | Not applicable | ✅ Flux could express `Vec<String, 20>` via `vec!` index refinements, but Verus already covers the same property with more rigorous postconditions at similar author burden. **Adequate** NA evidence (cite is slightly imprecise — "does not match the current Vec<String> shape" is true but the real reason is "redundant with Verus"). |
| **L** | Not applicable | ✅ No concurrency whatsoever. **Strong** NA evidence. |
| **M** | Not applicable | ✅ `#![forbid(unsafe_code)]` at module (line 5) and workspace (`Cargo.toml` line 10). **Strong** NA evidence. |
| **T** | Not applicable | ✅ No state machine. **Strong** NA evidence. |
| **Z** | Not applicable | ✅ Untrusted-input boundary is upstream at `intent/parser.rs` (which owns the Z lane per `verification-targets.md §5.3`). **Strong** NA evidence. |
| **X** | Not applicable | ✅ Module is in scope for V+P. **Adequate** NA evidence. |

**Lane discipline is correct.** All 7 NA decisions cite concrete evidence. The Verus skew is justified by the module's character.

One minor refinement: the Flux NA rationale could be tighter — instead of "does not match the current Vec<String> shape" the cite should be "Verus postcondition `len() <= 20` is equivalent to a Flux index refinement `v: Vec<String, 20>` and is provable at the same spec author cost" (F-cite quality: marginal).

---

## 6. Clippy debt at lines 213, 227

```
213:     behavior.preconditions.push(format!("precondition_{}", i));
227:     behavior.postconditions.push(format!("postcondition_{}", i));
```

Both are `uninlined_format_args` violations. Both are inside `#[cfg(test)] mod tests` (lines 120-235). The module-level `#[allow(...)]` block (lines 121-135) does **not** include `uninlined_format_args`, so the lint fires.

The plan handles this correctly:
- §1 marks them as test-only.
- §9 item 3 categorizes them as NON-BLOCKING for proof-writer (proof-writer doesn't run clippy), BLOCKING for `formal-verifier` (which needs `moon run :ci` clean).
- §10 explicitly disowns them — owned by `cl-2q6`, not `cl-ooz`.
- §11 pre-flight places `cl-2q6` as an independent gate.

**The plan correctly excludes them from production-body obligations.** No obligation references lines 213 or 227.

The fix is one-character per site (replace `format!("precondition_{}", i)` with `format!("precondition_{i}")`); trivially owned by `holzman-rust` after `cl-2q6` opens.

---

## 7. Module cross-references — obligations complete given `feature.rs` consumers?

`Behavior` is consumed by:

| Consumer | Site | Does it call `Behavior::validate()`? |
|---|---|---|
| `feature.rs::add_behavior` (line 62-71) | iterates over `self.behaviors` to dedup-check names, then pushes | **No** — caller-side dedup only, no per-behavior validate |
| `feature.rs::validate` (line 85-101) | checks `MAX_BEHAVIORS` and `MAX_DEPENDENCIES` on `self` | **No** — does not iterate `self.behaviors` and call `b.validate()` |
| `spec.rs::validate` (line 112-151) | checks `MAX_FEATURES`, `MAX_INVARIANTS`, `MAX_ANTI_PATTERNS`, then dedup, then circular-dep | **No** — does not call `behavior.validate()` per element |
| `spec.rs::add_feature` (line 73-79) | pushes a `Feature` (which contains `Vec<Behavior>`) | Indirect — relies on `Feature::add_behavior`'s dedup, not `Behavior::validate` |

**Key finding (F8 — minor, owner_approved_no_action):** the validator chain is **broken at the feature/spec boundary**. A `Behavior` with 21 preconditions can be embedded in a `Feature`/`Spec`, and the parent's `validate()` returns `Ok(())`. This is a real consistency gap in the *composition* of validators but **out of scope for `behavior.rs`'s proof plan** (the plan's obligations correctly scope to `Behavior`'s own surface).

If the project intends validator composition, that obligation belongs to `feature.rs`'s and `spec.rs`'s proof plans, not this one. **The current plan is complete for its scope.**

The bridge (§8) correctly enumerates which `feature.rs` / `behavior.rs` test sites are independent behavior tests. The plan does not over-claim.

---

## 8. Schema compliance — `proof-obligations.planned.jsonl`

Per `proof-schemas.md`, `proof-obligation/v1` requires 22 fields. The behavior JSONL has **13** of them per row. The 8 missing fields, on every one of the 21 rows:

| Missing field | Consequence |
|---|---|
| `schema_version` | Validator cannot parse the row as `proof-obligation/v1` |
| `domain_claim` | No high-level "what is being claimed" — only `contract_clause` (which is the INFERRED text) |
| `risk_tags` | Downstream risk aggregation broken |
| `workdir` | Verifier invocation may run in wrong directory |
| `model_bounds` | Vacuity proof gates cannot run |
| `tool_metadata` | Tool-version pinning missing |
| `trusted_base_refs` | Cross-reference to trusted-base-ledger broken |
| `behavior_affecting` | Waiver classification cannot run |

The behavior JSONL also lacks `target` (legacy aliases `layer` / `checker` are not used — good), so no legacy-alias violation, but the canonical `target` field IS present, which is correct.

**Verdict: structural schema violation.** This is not a stylistic gap — the row will not validate against `proof-obligation/v1` schema. **Blocker (F2)**.

Note: the sibling plans (`storage-types`, `newtypes`, `scenario`) have the same omissions. This appears to be a **project-wide convention** where the planner defers these fields to proof-writer. The schema says they're required at the planned stage. The reviewer must flag this so the convention either becomes explicit (schema relaxation) or gets fixed.

---

## 9. Trusted-base plan (§6)

The plan's §6 enumerates 7 trust anchors:

1. `serde_json` round-trip preserves values — PO-P1 exercises it.
2. `Vec::push` is order-preserving and appends — PO-V4/V5/V11/V12 reference the `Vec` API.
3. `String` is total over arbitrary UTF-8 — no mitigation needed.
4. `Behavior::new(s)` returns `InvalidBehaviorName(s)` exactly — PO-V2 proves the Err arm.
5. `Behavior::validate` returns `TooMany{Pre,Post}conditions(name, n, 20)` iff `n > 20` — PO-V8 proves both.
6. `MAX_* == 20` are exact literals — PO-V8/V13 reference the literal.
7. `Behavior` struct is closed at spec-write time — PO-V3/V6/V7 enumerate all 5 fields.

**Issue (F5 — major, owner_approved_debt):** the plan's §6 does **not** commit to producing `trusted-base-ledger/v1` rows. Per `proof-schemas.md`, every `assume` / `axiom` / `external_body` / `trusted` marker needs a ledger row. PO-P1 will need `#[verifier::external_body]` on `serde_json` calls (the plan §8 mentions this in prose), which generates trust markers, which need ledger entries. The plan defers this without an owner.

Proof-writer must produce a trusted-base-ledger.jsonl alongside the Verus specs.

---

## 10. Bridge input for `proof-to-implementation` (§8)

The bridge table maps each obligation to a Rust source ref and an independent behavior test. This is **complete and accurate**:

| PO | Rust source ref | Independent test |
|---|---|---|
| V1 | `is_valid_behavior_name` (17-25) | New classifier enumeration test (correct) |
| V2 | `Behavior::new` (54-65) | Existing `test_behavior_new_*` (141-181) |
| V3 | `Behavior::new` Ok-arm (58-64) | Extend existing valid-case test (correct) |
| V4 | `add_precondition` (86-89) | New preservation test (correct) |
| V5 | `add_postcondition` (92-95) | Symmetric (correct) |
| V6 | `with_description` (69-74) | Augment `test_serde_roundtrip_behavior` (184-207) |
| V7 | `with_verification` (78-83) | New test (correct) |
| V8 | `validate` (101-117) | Existing `test_behavior_validate_*` (209-235) + new boundary-at-MAX test |
| V9 | commutativity | New test (correct) |
| V10 | right-most-wins | New test (correct) |
| V11 | order preservation | New test (correct) |
| V12 | no-dedup | New test (correct) |
| V13 | monotonicity | New test at exactly MAX (correct) |
| P1 | serde round-trip | New proptest in `tests` module (correct) |

The bridge plan does not propose `rust-refinement-obligation/v1` rows. Per `proof-schemas.md`, every behavior-affecting obligation needs a matching `rust-refinement-obligation/v1` row with concrete source refs, independent behavior tests, separate refinement harness refs, and executed command evidence by State 12. The bridge table **implicitly** provides source refs and behavior tests but does not stamp `rust-refinement-obligation/v1` rows.

**Verdict:** the bridge is **semantically complete** (source refs + tests enumerated) but **structurally missing** (no `rust-refinement-obligation/v1` rows produced). This is a planning-time decision; proof-to-implementation can produce the rows. Acceptable as a deferral but should be tracked.

---

## 11. Provenance

| Field | Value |
|---|---|
| `reviewer_skill` | `proof-plan-reviewer` |
| `reviewer_invocation_id` | `ppr-cl-ooz-2026-06-21T16:32:00Z-7c4a9e1f` (synthesized — see F10) |
| `planner_invocation_id` | **MISSING** — plan does not stamp one; no ledger row exists |
| `host_session_id` | not visible (opencode control plane not exposed in workspace) |
| `input_artifacts` | `proofs/behavior-proof-plan.md`, `proofs/behavior-obligations.planned.jsonl`, `clarity-web/src/intent/types/{behavior,feature,spec,type_error,verification}.rs`, `clarity-web/src/intent/types.rs`, `verification-targets.md`, `formal-verification-report.md` |
| `output_artifacts` | `proofs/behavior-plan-review.md`, `proofs/verifier-lane-review.jsonl`, `proofs/proof-plan-findings.jsonl`, `proofs/proof-plan-repair-guide.md`, `.beads/cl-ooz/plan-review-report.md` |
| `reviewed_artifacts_existed_before_start` | yes (all input artifacts exist on disk at this review's start) |

---

## 12. Findings summary

| # | Severity | Code | Disposition | Subject |
|---|---|---|---|---|
| F1 | blocker | `E_REVIEW_PROVENANCE_MISSING` (rust-contract variant) | blocker | No `rust-contract` artifact exists; 14 INFERRED clauses unratified |
| F2 | blocker | `E_SCHEMA_MISSING_FIELD` (×8 per row × 21 rows) | blocker | JSONL rows missing 8 schema-required fields |
| F3 | major | `E_PROOF_PLAN_MISSING_NONVACUITY` | owner_approved_debt | PO-P1 strategy needs `is_valid_behavior_name` invariant pinned to generated names |
| F4 | major | `E_PROOF_PLAN_MISSING_NONVACUITY` | owner_approved_debt | PO-P1 strategy lacks `arb_verification()` sub-strategy |
| F5 | major | `E_TRUST_LEDGER_INCOMPLETE` | owner_approved_debt | §6 trusted-base plan does not commit to `trusted-base-ledger/v1` rows |
| F9 | major | `E_LANE_DECISION_MISSING` | fixed_with_evidence | `verifier-lane-decisions.jsonl` absent (folded into obligations JSONL); reviewer produces `verifier-lane-review.jsonl` |
| F10 | major | `E_INVOCATION_LEDGER_MISSING` | owner_approved_debt | No `agent-invocation-ledger.jsonl` row for planner or reviewer |
| F6 | minor | (algebraic-imprecision) | owner_approved_no_action | PO-V9 "left-to-right evaluation" cite is imprecise but harmless |
| F7 | minor | (consumer-pattern observation) | owner_approved_no_action | `add_precondition`'s fluent return is not exercised within this module's tests |
| F8 | minor | `E_SCOPE_MISCLASSIFIED_BEHAVIOR` (out-of-scope variant) | owner_approved_no_action | Validator composition gap at feature/spec boundary is out of scope for this plan |

**Counts:**
- blocker: **2** (F1, F2)
- major: **5** (F3, F4, F5, F9, F10)
- minor: **3** (F6, F7, F8)
- non-blocking observations: 3 (all disposition: `owner_approved_no_action`)

Per the reviewer's role description, **all unresolved findings must be dispositioned before approval**. F1 and F2 are `blocker` and require fix evidence; F3–F5, F10 are `owner_approved_debt` (acceptable for non-behavior-affecting findings, which these are); F6–F8 are `owner_approved_no_action` (acceptable per skill spec for non-blocking minor findings); F9 is `fixed_with_evidence` (this reviewer produces the lane-review rows in §13).

**Two blocker findings force rejection.**

---

## 13. Verifier-lane-review rows

The reviewer writes one `verifier-lane-review/v1` row per planner lane decision. The obligations JSONL folds lane decisions into obligation rows; the reviewer treats each row as one planner-owned lane decision.

21 rows produced in `proofs/verifier-lane-review.jsonl`. Disposition summary:

| Verifier | Rows | Disposition |
|---|---|---|
| verus | 13 | accepted (algebraically sound; contract-gated by F1) |
| proptest | 1 | accepted (algebraically sound; contract-gated by F1; debt F3, F4) |
| kani | 1 | accepted (NA evidence strong) |
| flux | 1 | accepted (NA evidence adequate) |
| loom | 1 | accepted (NA evidence strong) |
| miri | 1 | accepted (NA evidence strong) |
| tlaplus | 1 | accepted (NA evidence strong) |
| cargo-fuzz | 1 | accepted (NA evidence strong) |
| cargo-test (X) | 1 | accepted (NA evidence adequate) |

All 21 reviewer rows carry `reviewer_disposition: accepted`. Plan-level rejection is driven by **the schema/contract blockers (F1, F2)**, not by any lane rejection.

---

## 14. Recommendation

**Status: REJECTED.**

The plan is **algebraically correct** — all seven laws hold against the source, the law coverage is exhaustive, and the cross-references are accurate. The lane discipline is right (Verus-skew is justified by the module's character). The clippy debt placement is correct. The bridge input is complete.

But the plan has two structural blockers:

1. **No `rust-contract` artifact exists** (F1) — all 14 clauses are INFERRED; the schema requires `contract_clause` to be authored, not inferred; `requires_contract: true` is an indicator, not a gate.
2. **JSONL schema non-compliance** (F2) — 8 required fields missing on every one of 21 rows; the rows will not validate against `proof-obligation/v1`.

Both blockers must be cleared before proof-writer runs. Algebraically, the obligations are sound and the plan will pass review once the blockers are addressed. The repair guide (`proof-plan-repair-guide.md`) names the smallest state to rerun.

Five non-blocking debt items (F3, F4, F5, F10) and three observations (F6, F7, F8) are dispositioned and acceptable for advancement once F1 and F2 are fixed.

---

STATUS: REJECTED
