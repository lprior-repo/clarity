# Proof Plan Review — `clarity-web/src/domain/newtypes.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-0n6` |
| **Target** | `clarity-web/src/domain/newtypes.rs` |
| **Plan** | `proofs/newtypes-proof-plan.md` (321 lines, 11 sections) |
| **Obligations** | `proofs/newtypes-obligations.planned.jsonl` (41 rows: 25 Verus + 9 proptest + 7 not_applicable) |
| **Reviewer** | `proof-plan-reviewer` |
| **Reviewer invocation** | `proof-plan-reviewer@cl-0n6.2026-06-21` (this session) |
| **Review state** | completed |
| **Date** | 2026-06-21 |

---

## 0. Verdict

```
STATUS: REJECTED
```

The plan is well-structured and the lane discipline (`V` primary, `P` secondary, the
other six lanes honestly `not_applicable`) is the right call per
`verification-targets.md §5.1`. The honest disclosure of the contract gap (§2, §9
Blocker #1) is exactly what the doctrine asks of an `INFERRED`-clause plan. The
trusted-base plan (§6) is the right shape. **However, two real blockers must be
fixed before proof-writer is unblocked, plus several non-blocking findings that
should be cleaned up at the same time.**

---

## 1. Reviewed artifacts

| Path | SHA-style fingerprint | Existence confirmed |
|---|---|---|
| `proofs/newtypes-proof-plan.md` | 321 lines, 11 sections | yes |
| `proofs/newtypes-obligations.planned.jsonl` | 41 JSONL rows | yes |
| `verification-targets.md` §5.1 | V + P primary/secondary | yes |
| `formal-verification-report.md` §3 (`cl-2vr` critical finding) | 901 panic-prone sites | yes |
| `Cargo.toml` workspace lints | `unwrap_used = "deny"`, `expect_used = "deny"`, etc. | yes |
| `clarity-web/src/domain/newtypes.rs` | 307 lines, 0 `#[cfg(test)]` blocks | yes |
| `clarity-web/src/domain/mod.rs` | module-level `#![deny(clippy::unwrap_used)]` / `expect_used` / `panic` at lines 7–9 | yes |
| `clarity-web/Cargo.toml` | **no `[lints] workspace = true`** (per `formal-verification-report.md §3`) | yes |

---

## 2. Plan-level assessment

### 2.1 What the plan does well

1. **Lane discipline is correct.** Per `verification-targets.md §5.1`, `domain/newtypes.rs`
   is `V + P`. The plan matches exactly. No Kani, Flux, Loom, Miri, TLA+, fuzz, or
   X obligation is missing — each is `not_applicable` with a stated reason.
2. **Honest contract-gap disclosure.** §2 admits there is no `rust-contract` artifact and
   the 16 clauses in §3 are `INFERRED`. §9 marks this as the first blocker. Every
   `requires_contract: true` obligation surfaces the gate programmatically.
3. **Trusted-base plan (§6) names six concrete trust boundaries** and ties each to the
   obligation rows that depend on it (`String::trim`, `chrono::parse_from_rfc3339`,
   `chrono::Utc::now`, `chrono::DateTime::to_rfc3339`, `serde_json`, `Display::fmt` for
   `String`).
4. **Bridge input (§8) maps every Verus claim to a Rust source ref AND an independent
   proptest function** that proof-writer creates — this satisfies
   `proof-schemas.md §rust-refinement-obligation/v1`'s "independent behavior tests" rule.
5. **Verus mode choices are appropriate** — `exec+spec` for the validators, `exec` for
   the trivial extractors, `exec+external_body` for the `Display::fmt` impls,
   `exec+extern_spec` for `Timestamp::try_from` / `Timestamp::now`. (PO-NT-V-25 has a
   minor mode mis-label — see F-NB-4.)
6. **No behavior waivers.** §7 explicitly states none, and a spot-check confirms every
   `not_applicable` row is justified by a real module-property reason.

### 2.2 What blocks advancement (blockers)

#### F-BLOCK-1 — `E_SCHEMA_DRIFT` — schema version and required-field gaps on all 41 obligation rows

**Evidence:** The `proofs/newtypes-obligations.planned.jsonl` schema declaration is
`"schema_version":"proof-obligations/v1"` (plural). The canonical schema per
`go-skill/references/proof-schemas.md §Required Schema Versions` is
`proof-obligation/v1` (singular). Additionally, the schema lists
`domain_claim` as a required field; the obligations JSONL has no such field on any of
the 41 rows. It carries `contract_clause` (close but not the same), and an extra
`obligation_id` field whose value duplicates `id`.

**Impact:** Schema validator fails on all 41 rows. proof-writer downstream consumes
this JSONL; if the validator is in the path, proof-writer cannot materialise specs.
Even if the validator is not in the path, the obligations are not machine-checkable
against the canonical schema.

**Disposition:** `blocker`.

**Required fix:**
- Change every row's `schema_version` from `"proof-obligations/v1"` to `"proof-obligation/v1"`.
- Add a `domain_claim` field to every row (suggest: a one-sentence plain-English
  statement of the property — for `PO-NT-V-01`: `"AnswerId::try_from is the validator
  that makes whitespace-only inputs unrepresentable inside AnswerId"`; for
  `PO-NT-P-09`: `"serde_json round-trips identity for each newtype on well-formed
  input"`).
- Optional but recommended: collapse the redundant `id` / `obligation_id` duplication.

#### F-BLOCK-2 — `E_TEST_BODY_LINT_CONFLICT` — proptest bodies will fail `cargo clippy` against the module-level deny lints

**Evidence:** `clarity-web/src/domain/mod.rs:7-9` declares:

```rust
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
```

These are **module-level inner attributes**. They apply to all code in the
`domain` module tree, including `#[cfg(test)] mod tests` declared inside
`domain/newtypes.rs`. The plan's §5 row for PO-NT-P-09 includes
`expected_evidence` text `"from_str::<T>(&to_string(&t).expect(...)) == Ok(t.clone())"`,
and the obligation's own `notes` field says:

> *"Uses .expect() inside the property — this is in #[cfg(test)] so the workspace
> unwrap_used deny does not apply. proof-writer may want to use .unwrap_or_default()
> or similar."*

This statement is **factually incorrect** for the `domain` module. The deny lints
in `mod.rs` are unconditional inner attributes, not workspace-level lints, and they
are not gated by `cfg(test)`. `cargo clippy -p clarity-web --all-targets -- -D warnings`
will fire on any `.expect(...)` call inside `domain/newtypes.rs`'s test submodule.

**Impact:** proof-writer will add proptest bodies that fail to compile under the
module-level denies. The plan's own compensating-evidence plan (`PO-NT-P-09`
mentions `.expect(...)`) cannot be implemented as written.

**Disposition:** `blocker`.

**Required fix (any one of):**
- (a) Re-instruct proof-writer to use `.unwrap_or_else(|_| String::new())` or
  `.unwrap_or_default()` instead of `.expect(...)` inside proptest bodies.
- (b) Annotate each proptest function with `#[allow(clippy::expect_used, clippy::unwrap_used)]`.
- (c) Move the proptest file out of `domain/` into `clarity-web/tests/newtypes_proptest.rs`
  (which is outside the `domain` module and not subject to `mod.rs:7-9` denies) —
  this is the cleanest fix.

The plan's `expected_evidence` and `notes` strings should be updated to match the
chosen approach.

### 2.3 Non-blocking findings (must address before proof-writer runs, but do not block approval-on-replan)

#### F-NB-1 — `E_LANE_DECISION_WEAK` — Flux `not_applicable` rationale is author preference, not concrete evidence

The plan §4 row "F (Flux)" says:

> *"Verus covers the same refinement properties with more rigour at the same author
> cost. cargo-flux is installed but unused for this module."*

`verification-lane-policy.md §Non-Applicability` requires **concrete evidence
references** for `not_applicable` decisions. "Verus is more rigorous" is a
preference statement, not a concrete property of the module. The justification
should be one of:
- "No refinement predicate on this module is expressible in Flux but not Verus — all
  five newtypes have at most one refinement shape (`!s.trim().is_empty()` or
  `parse_from_rfc3339(&s).is_ok()`), which Verus already pins via `ensures`."
- "Flux installation version is N.N.N; no refinement type in newtypes.rs falls within
  Flux's lower-bound arithmetic/refinement sweet spot (Verus owns that surface)."

This is medium severity — the lane decision is likely correct but the rationale
is not validator-friendly.

**Disposition:** `owner_approved_debt` (non-blocking — already deferred if the
owner ratifies the lane choice on those grounds).

#### F-NB-2 — `E_TRUST_LEDGER_DEFERRED` — `trusted-base-ledger.jsonl` is named but not materialised

Per `proof-schemas.md §trusted-base-ledger/v1`, every `external_body`, `extern_spec`,
`assume`, `axiom`, `admit`, `ignore`, stub, or model reduction needs a
`trusted-base-ledger/v1` row. The plan §6 names six trusts but defers the ledger
to State 12 closure. This is technically correct per the workflow — the ledger
is a proof-writer artifact, not a plan-review artifact — but the plan's obligation
rows already declare `trusted_base_refs` (e.g., `"Verus stdlib String::trim spec"`,
`"extern_spec on chrono::DateTime::parse_from_rfc3339"`) without those references
existing in any file. proof-writer needs the ledger to materialise before it can
verify the obligations against the trusted-base policy.

**Disposition:** `owner_approved_debt` — track as a follow-up bead against `cl-0n6`.
The plan's decision to defer is acceptable; the obligation rows should add a
`trusted_base_ledger_ref` field (or be regenerated by proof-writer once the
ledger exists).

#### F-NB-3 — `E_CLIPPY_GATE_OMISSION` — plan §9 omits the `cl-2vr` (CRITICAL P0) finding

`formal-verification-report.md §3` flags `cl-2vr` as **CRITICAL P0**: the
workspace deny lints (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`)
are **inert for `clarity-web`** because `clarity-web/Cargo.toml` has no
`[lints] workspace = true` opt-in. 901 panic-prone sites exist.

The plan §9 Blocker #4 mentions `cl-2q6` (the standard clippy baseline) but does
**not** mention `cl-2vr`. The plan's obligations implicitly assume panic-free
production paths. For `domain/newtypes.rs` specifically, the assumption is
**independently defensible** — the source has zero `.unwrap()`, `.expect(...)`,
`panic!`, `todo!`, `unimplemented!`, or `unsafe` — so the module-level panic-
freedom assumption is sound regardless of `cl-2vr`. But the plan should state this
explicitly: cite the actual inventory of this module (0 sites) as the basis for
panic-freedom, not the workspace lint configuration.

**Disposition:** `owner_approved_no_action` (low severity, but a one-line addition
to §9 Blocker #4 closes the loop).

#### F-NB-4 — `E_VERIFIER_MODE_INCONSISTENCY` — PO-NT-V-25 mode is mis-labelled

PO-NT-V-25 (`Timestamp::default`) is declared with `"verus_mode":"exec"`. The body
(line 305) calls `Self::now()`, which has `extern_spec` on `chrono::Utc::now()` and
`chrono::DateTime::to_rfc3339()`. The mode should be `"exec+extern_spec"` for
consistency with PO-NT-V-24.

**Disposition:** `fixed_with_evidence` — trivial fix; the planner updates the row.

#### F-NB-5 — `E_VACUITY_DISPLAY_FMT` — five Verus Display obligations are vacuous under `external_body`

PO-NT-V-17 through PO-NT-V-21 (`Display::fmt` for each newtype) use
`verus_mode: "exec+external_body"`. Under `external_body`, Verus does **not**
verify that the body produces the spec's postcondition — it asserts the spec is
consistent and trusts the body. The spec postcondition
(`format!("{}", t) == t.as_str()`) is therefore not derived; it is asserted.

This is the canonical way to handle `Display::fmt` (Verus cannot fully model
`fmt::Formatter`), so the choice is correct. **However**, the plan does not
cross-reference the compensating evidence: PO-NT-P-07 (`proptest_display_roundtrip`)
exercises the same property via executable Rust. Each Display obligation row should
add a `compensating_evidence_refs: ["PO-NT-P-07"]` field so the bridge and the
reviewer can see the vacuity + compensation together.

**Disposition:** `owner_approved_debt` — medium severity. The plan's bridge table
in §8 does mention PO-NT-P-07 for Display obligations, but the obligation rows
themselves don't.

#### F-NB-6 — `E_REDUNDANT_DEFAULT_OBLIGATION` — PO-NT-V-25 is structurally a duplicate of PO-NT-V-24

`Timestamp::default()` body is `Self::now()` (line 305). PO-NT-V-25 is functionally
covered by PO-NT-V-24. The plan §5 acknowledges this ("Body is a one-liner that
delegates to now(); the spec asserts the same postcondition") and keeps the
separate row for coverage. This is acceptable but a `notes` cross-reference to
PO-NT-V-24 would prevent future readers from re-deriving the property.

**Disposition:** `owner_approved_no_action` — low severity.

#### F-NB-7 — `E_COMMAND_EVIDENCE_MIXED` — PO-NT-P-09 evidence field mixes prose and Rust code

PO-NT-P-09's `expected_evidence` field contains a Rust expression in prose
(`from_str::<T>(&to_string(&t).expect(...))`). This mixes source-level syntax with
executable-evidence expectation. A separate `notes` field (or a stringified Rust
expression in backticks) is more validator-friendly.

**Disposition:** `owner_approved_no_action` — low severity; cosmetic.

#### F-NB-8 — `E_INVOCATION_PROVENANCE_MISSING` — planner invocation ID not visible

Per `go-skill/references/review-provenance.md`, reviewer and planner invocations
must differ. The plan does not record the planner invocation ID in any
machine-readable field. This reviewer assumes the host control plane tracks
this; the workspace Markdown is not enough. The proof-writer and proof-reviewer
downstream should add `planner_invocation_id` to obligation rows so the chain
is hashable.

**Disposition:** `owner_approved_no_action` — informational; depends on host
control plane.

### 2.4 Vacuity / non-vacuity check (per `verification-lane-policy.md §Proof-Theater Rejections`)

| Obligation cluster | Vacuous? | Justification |
|---|---|---|
| V-01..V-08 (try_from / from_str) | **No** | These pin the validator contract — the spec is non-trivial (`!s.trim().is_empty()` predicate). |
| V-09..V-12 (From<X> for String) | **Borderline** | Trivial byte-equality extractors. Not vacuous — they pin the conversion for downstream storage layers — but the marginal assurance over the type system is small. |
| V-13..V-16 (as_str) | **Borderline** | Same shape as V-09..V-12. |
| V-17..V-21 (Display::fmt) | **Yes, under external_body** | The body is trusted; the spec is asserted. Compensated by PO-NT-P-07 (see F-NB-5). |
| V-22, V-23 (AnswerValue) | **Borderline** | Identity conversions; spec is trivial. |
| V-24, V-25 (Timestamp::now / default) | **No** | External-world behaviour (wall clock, RFC 3339 format) — non-vacuous by construction. |
| P-01..P-09 (proptest) | **No** | Executable property tests; non-vacuous by construction. |
| K, F, L, M, T, Z, X (not_applicable) | **N/A** | Genuine non-applicability per the plan's §4 evidence. |

The plan satisfies the non-vacuity rule on the **safety** and **panic-freedom**
obligations (V-01..V-08, V-24, V-25). The trivial-extractor cluster (V-09..V-16,
V-22, V-23) is borderline but acceptable — the obligation rows are explicit, the
spec is concrete, and the cost is near-zero (the plan §10 says so). The Display
cluster (V-17..V-21) is vacuous in isolation but compensated.

### 2.5 Behavior-affecting audit (per `proof-schemas.md §E_SCOPE_MISCLASSIFIED_BEHAVIOR`)

All 25 Verus obligations are marked `behavior_affecting: true`. All 9 proptest
obligations are `behavior_affecting: true`. All 7 `not_applicable` obligations
are `behavior_affecting: false` — correct.

Spot-check: the trivial extractors (V-09..V-12, V-13..V-16, V-22, V-23) ARE
behavior-affecting because they pin the conversion contract for downstream
serialization layers. Marking them `behavior_affecting: false` would be a
classifier error; the plan is correct.

### 2.6 Waiver candidates

§7 says "None." Verified: no `waiver-candidate/v1` rows in the obligations, no
`formal-waiver/v1` rows, no `WAIVED` verification-ledger rows in
`verification-ledger.jsonl`. Plan is waiver-free.

### 2.7 Lane coverage (per `verification-lane-policy.md §Default Rust-Implementation Profile`)

Default required: Verus, Kani, Flux, proptest. Plan decisions:
- **Verus:** REQUIRED (primary). ✓ Matches the profile.
- **Kani:** NOT_APPLICABLE. ✓ Concrete evidence: no unsafe, no arithmetic, no parser.
- **Flux:** NOT_APPLICABLE. ⚠ Weak rationale (see F-NB-1).
- **proptest:** REQUIRED (secondary). ✓ Matches the profile.

Conditional: Loom (no concurrency), fuzz (no parser). Both correctly NOT_APPLICABLE.

---

## 3. Bridge plan (§8) — review

The §8 table maps every proof claim to:
- a Rust source ref (file + line range)
- an independent behaviour test that proof-writer creates

This is correct shape per `proof-schemas.md §rust-refinement-obligation/v1`. The
21 row entries cover all 25 Verus obligations (some share rows) and all 9 proptest
obligations. **No missing bridge rows.**

One small concern: §8 says "New `proptest_answer_id_boundary`" etc. for PO-NT-V-01..
V-03. The proptest boundary test is the same as PO-NT-P-01..P-03. The bridge row
should be `PO-NT-V-01 ↔ PO-NT-P-01` rather than naming a new test — this is what
proof-to-implementation will do, but the plan should pre-disambiguate to avoid
duplication.

**Disposition:** `owner_approved_no_action` — bridge agent will normalise.

---

## 4. Pre-flight gates per `verification-targets.md §4` — readiness check

| Gate | Status | Notes |
|---|---|---|
| Close `cl-2q6` (clippy debt) | not closed | Plan §9 Blocker #4 acknowledges but does not block this plan |
| Close `cl-2vr` (lint opt-in) | not closed | Plan §9 **does not** acknowledge (see F-NB-3); for THIS module, the lint gap is not a blocker because the module has zero panic-prone sites |
| Install kani, apalache, cargo-fuzz | not done | Plan §4 correctly notes these are `not_applicable` regardless |
| Re-run `cargo test --workspace --all-features` | unknown | Should be a precondition; not in the plan |
| Author `rust-contract` artifacts for first module | **not done** | Plan §2 + §9 Block #1 names this as the first blocker |
| Author `proof-obligations.planned.jsonl` | **done** | This plan's input |
| Review the plan | **in progress** | This document |

---

## 5. Trust marker scan

Per `proof-schemas.md §trusted-base-ledger/v1`, every `assume`, `axiom`, `admit`,
`external_body`, `trusted`, `ignore`, stub, disabled check, model bound, or model
reduction needs a row. The plan's obligations declare:

- `external_body`: 5 obligations (PO-NT-V-17..V-21) — 5 ledger rows needed.
- `extern_spec`: 3 obligations (PO-NT-V-07, V-24, V-25) — 3 ledger rows needed.

Total: **8 trust rows** required before proof-writer runs. The plan §6 lists 6
trusts (Verus stdlib `String::trim`, Verus stdlib `str::to_string`, Verus stdlib
String indexing, Rust stdlib `Display for String`, `chrono::parse_from_rfc3339`,
`chrono::Utc::now().to_rfc3339()`, `serde_json::to_string`/`from_str`). The
ledger needs **at minimum 8 rows** (the 6 named + 1 for the `external_body` family
on `Display::fmt` + 1 for the `extern_spec` family on `chrono::DateTime::to_rfc3339`
+ 1 for `chrono::Utc::now` — depending on how the planner wants to consolidate).

**Disposition:** `owner_approved_debt` — proof-writer creates the ledger before
verifying the first obligation. Plan §6 references this correctly.

---

## 6. Required replan actions (smallest state to rerun)

The plan should be re-submitted with these changes:

| # | Change | Owner | Severity |
|---|---|---|---|
| 1 | Rename `schema_version` to `proof-obligation/v1` (singular) on all 41 obligation rows | proof-planner (re-plan) | blocker |
| 2 | Add `domain_claim` field to all 41 obligation rows | proof-planner | blocker |
| 3 | Either re-instruct proof-writer to use `.unwrap_or_default()` / `.unwrap_or_else` in proptest bodies, OR add `#[allow(clippy::expect_used)]`, OR move proptest file to `clarity-web/tests/newtypes_proptest.rs` | proof-planner (re-plan) | blocker |
| 4 | Strengthen Flux non-applicability rationale to cite a concrete module property (see F-NB-1) | proof-planner | non-blocking |
| 5 | Add `compensating_evidence_refs: ["PO-NT-P-07"]` to PO-NT-V-17..V-21 | proof-planner | non-blocking |
| 6 | Fix PO-NT-V-25 `verus_mode` from `"exec"` to `"exec+extern_spec"` | proof-planner | non-blocking |
| 7 | Add a one-line note to §9 Blocker #4 that the module's panic-freedom basis is the module-level inventory (0 sites), not the workspace lint config | proof-planner | non-blocking |
| 8 | Add cross-reference from PO-NT-V-25 to PO-NT-V-24 in `notes` field | proof-planner | non-blocking |
| 9 | Re-organise PO-NT-P-09 `expected_evidence` so the Rust snippet is in a separate `notes` field | proof-planner | non-blocking |

After these changes, the plan can be APPROVED-WITH-CONDITIONS (the condition
being: `rust-contract` must author `clarity-web/src/domain/contract.md` and ratify
the 16 INFERRED clauses before proof-writer runs).

---

## 7. Acceptance criteria for the next round

A re-submitted plan will be APPROVED-WITH-CONDITIONS when:

1. All 9 row-level fixes above are applied.
2. The obligations JSONL validates against `proof-obligation/v1` (singular) with
   `domain_claim` present on every row.
3. A new section "Test-body lint compliance" (or equivalent) explicitly states
   which proptest-body pattern is used (a/b/c above) and updates the obligations'
   `expected_evidence` and `notes` to match.
4. The plan's §9 Blockers explicitly cite `cl-2vr` and state that this module
   is panic-free by inventory.

The condition for the next stage (proof-writer unblock) is **rust-contract
ratification**, not the plan itself.

---

## 8. Findings count

| Severity | Count | Disposition |
|---|---:|---|
| blocker | 2 | both must be fixed before re-plan approval |
| medium (non-blocking) | 4 | F-NB-1, F-NB-2, F-NB-3, F-NB-5 |
| low (non-blocking) | 4 | F-NB-4, F-NB-6, F-NB-7, F-NB-8 |
| **total** | **10** | — |

---

## 9. Recommendation to dispatcher

**Do NOT proceed to proof-writer.** Send the plan back to `proof-planner` for
the 9 fixes in §6. Once re-submitted, the plan is likely APPROVED-WITH-CONDITIONS
(pending `rust-contract` ratification of the 16 INFERRED clauses, which is a
parallel workstream, not a plan-replan dependency).

The two blockers are tractable:
- Schema fix is a search-and-replace + field addition. ~30 minutes.
- Lint-conflict fix is a decision + ~20 lines of obligation-row updates. ~30 minutes.

The non-blocking findings are clean-ups that strengthen the plan but do not
gate proof-writer.

---

## 10. Output artifacts

| Path | Purpose |
|---|---|
| `proofs/newtypes-plan-review.md` | this document |
| `proofs/newtypes-verifier-lane-review.jsonl` | one review row per planner lane decision |
| `proofs/newtypes-plan-findings.jsonl` | one row per finding (F-BLOCK-1..2, F-NB-1..8) |
| `proofs/newtypes-plan-repair-guide.md` | exact repair steps for the 9 fixes |
| `.beads/cl-0n6/plan-review-report.md` | per-skill report (bead-local copy) |

---

## 11. Sign-off

The reviewer stage of `cl-0n6` is **complete with a REJECT verdict**.

The plan demonstrates strong domain understanding (the lane choices, the
contract-gap disclosure, the trusted-base plan, the bridge table). The blockers
are mechanical, not conceptual. A focused replan pass should bring this plan to
APPROVED-WITH-CONDITIONS.

The next agent in the chain is `proof-planner` (replan) and `rust-contract`
(ratify the 16 INFERRED clauses — a parallel workstream). proof-writer is gated
until both are done.

```
STATUS: REJECTED
```
