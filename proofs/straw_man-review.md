# Proof Review — cl-vv2 (`domain/straw_man.rs`)

| Field | Value |
|---|---|
| **Bead** | `cl-vv2` |
| **Module** | `clarity-web/src/domain/straw_man.rs` (303 LOC) |
| **Reviewer** | `proof-reviewer` (adversarial) |
| **Date** | 2026-06-21 |
| **Lane** | V (Verus, primary) + P (proptest, secondary) |
| **Artifacts reviewed** | `proofs/straw_man_verus.rs` (12.1K, 9 exec fns + 3 spec fns), `proofs/straw_man_proptest.rs` (14.7K, 21 `#[test]` functions inside `proptest!`), `proofs/straw_man-writeup.md` (16.9K), `.beads/cl-vv2/proof-writer-report.md` (10K), `.beads/cl-vv2/proof-evidence.md` (20.6K) |
| **Verifier execution evidence** | `/tmp/opencode/verus-straw_man.txt` (raw) + `formal-verification-report.md` §4 row 5 |
| **Verus tool** | `/home/lewis/.local/bin/verus`, `0.2026.05.05.d03e906` (verified present) |

---

## Verdict

**STATUS: REJECTED**

The Verus spec does not verify. The verifier was actually run (per `formal-verification-report.md §4` row 5 and `/tmp/opencode/verus-straw_man.txt`) and rejected with `core::slice::impl&%0::contains is not supported` at `proofs/straw_man_verus.rs:220`. The trust ledger claims `Vec::contains` is in vstd; the verifier output proves it is not. The proof artifacts are therefore not in a state that can be approved, and the writeup's "anti-laundering" claim is internally inconsistent with the actual verifier behavior.

This is a **fix-and-reverify** rejection. The mechanical fix is one `assume_specification` line for `<[T]>::contains` (the verifier error message itself prints the exact signature). The deeper fix is a corrected trust ledger. Neither is a destructive change; both are required before the bundle can be re-submitted.

---

## 1. Summary of raw verifier result

The verifier has already been run on this exact artifact. Raw evidence at `/tmp/opencode/verus-straw_man.txt`:

```text
error: `core::slice::impl&%0::contains` is not supported (note: you may be able
       to add a Verus specification to this function with `assume_specification`)
       (note: the vstd library provides some specification for the Rust std
       library, but it is currently limited)
   --> /home/lewis/src/clarity/proofs/straw_man_verus.rs:220:9
    |
220 |         self.traps_detected.contains(&trap)
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: The following declaration may resolve this error:
            pub assume_specification<T> [core::slice::<impl [T]>::contains]
                (_0: &[T], _1: &T) -> bool
            where
            T: std::cmp::PartialEq,;

error: aborting due to 1 previous error
```

Cross-references:

- `formal-verification-report.md` row 5 of §1 confirms exit 1, attributed to "`slice::contains` not in vstd".
- `formal-verification-report.md` §4 column "Verus run" for `cl-vv2`: `❌ exit 1 — slice::contains`.
- `formal-verification-report.md` §4 row "Repair bead": `cl-ev9` — "vstd lacks `slice::contains`; add `assume_specification`".
- `proofs/straw_man-writeup.md §7` claims expected acceptance criterion: "Exit code 0." The actual result is exit 1.
- `.beads/cl-vv2/proof-writer-report.md §3` lists status as `PENDING_FORMAL_EXECUTION` for the V lane. This contradicts the formal-verifier's `verification-ledger.jsonl` (per the report's own §7), which already has a `verifier_execution` row with `FAIL_LOCAL`.

**This is not transient.** It is a structural vstd gap. The error is the canonical "missing std spec" failure mode and is reproducible.

---

## 2. Findings

Findings ordered by severity. Each cites the artifact path and the line range.

### F1 — BLOCKER — Verus spec does not verify (proof claim contradicted by raw verifier output)

- **Artifact:** `proofs/straw_man_verus.rs:220`
- **Obligation:** PO-V7 (`StrawManValidation::has_trap` — `ensures r == self.traps_detected@.contains(trap)`)
- **Severity:** `blocker` — proof is not in a state where it can verify; cannot be approved.
- **Evidence:** `/tmp/opencode/verus-straw_man.txt:1-12` shows the verifier rejecting the body at line 220 with the explicit "you may be able to add a Verus specification ... with `assume_specification`" hint. The error is a hard error (`aborting due to 1 previous error`), not a warning.
- **Required fix:** Add the `assume_specification` declaration shown in the verifier help message, inside the `verus!` block (e.g. immediately after the type definitions and before the `impl StrawManTrap { ... }` block, or near the top of the `impl StrawManValidation` block):
  ```rust
  pub assume_specification<T> [core::slice::<impl [T]>::contains]
      (_0: &[T], _1: &T) -> bool where T: std::cmp::PartialEq;
  ```
  After adding, re-run `verus proofs/straw_man_verus.rs` and capture the exit-0 evidence in `verification-ledger.jsonl`. Update `proof-evidence.md §4` closure criteria to mark the verifier run as `PASS` rather than `PENDING_FORMAL_EXECUTION`.
- **Disposition:** `fixed_with_evidence` (required before re-review).

### F2 — BLOCKER — Trust ledger is inconsistent with actual verifier behavior

- **Artifact:** `proofs/straw_man-writeup.md §10` and `proofs/straw_man_verus.rs:28-35` (Trusted base block)
- **Obligation:** PO-V7, PO-V8 (and trust ledger entries #1 and #2 in §10)
- **Severity:** `blocker` — the writeup's anti-laundering affirmation is contradictory: it claims `Vec::contains` is trusted via "Verus std `external_type_specification`" and "no spec gap", but the verifier just rejected the very use site with "the vstd library provides some specification for the Rust std library, but it is currently limited". The trust ledger is not a faithful description of what the verifier actually accepts.
- **Evidence:**
  - Writeup §10 row 1: "**`vstd::seq::Seq::contains` semantics** | Verus std spec, not our code" — claim correct for `Seq::contains` (used in postcondition `self.traps_detected@.contains(trap)`).
  - Writeup §10 row 2: "**`Vec::is_empty`, `Vec::len`, `Vec::contains(&T)`** | Rust std lib contract | Verus std external_type_specification; no spec gap." — claim **wrong** for `Vec::contains` / `<[T]>::contains`. vstd has spec for `Vec::is_empty` and `Vec::len` but **not** for `[T]::contains`. The verifier output proves this.
  - `proof-evidence.md §3` anti-laundering scan regex `'assume\(|\#\[verifier::external_body\]|\#\[verifier::external\]|axiom'` — passes (no production shortcuts), but the trust ledger is a separate claim that also needs to be truthful.
- **Required fix:** Rewrite trust ledger rows to honestly reflect the vstd surface. Suggested corrected rows:
  1. `vstd::seq::Seq::contains` — Verus std spec, not our code. Used in PO-V7 postcondition. **Correct as written.**
  2. `Vec::is_empty` and `Vec::len` — Rust std lib contract, vstd external_type_specification available. **Used in PO-V5, PO-V6, PO-V8, PO-V9.**
  3. `<[T]>::contains` (slice method) — **Not in vstd.** `assume_specification` required at use site (`straw_man_verus.rs:220`). Once added, the body `self.traps_detected.contains(&trap)` returns `bool`; the postcondition `r == self.traps_detected@.contains(trap)` then becomes provable via the explicit link.
  4. `&'static [Self; N]` slice coercion to `&'static [Self]` — Rust language rule. **Correct as written.**
  5. `StrawManTrap` is closed (no future variant) — Type-system fact at spec time. `all_variant_at` enumerates 4 indices; adding a 5th variant breaks the spec at compile time. **Correct as written.**
  6. `serde_json` round-trip — Library contract, X lane. **Correct as written.**
  7. vstd `Seq::last`, `Seq::len` — Verus std spec, used in PO-V1, PO-V4. **Correct as written.**
- **Disposition:** `fixed_with_evidence` (required before re-review).

### F3 — BLOCKER — No upstream plan ratification; obligations labelled `INFERRED`

- **Artifact:** `proofs/straw_man-writeup.md §1` ("Upstream note (no approved plan)"), `.beads/cl-vv2/proof-writer-report.md §1` ("no approved plan exists yet — all clauses are **INFERRED** from source")
- **Severity:** `blocker` (procedural). The proof obligations are not anchored to a `proof-plan-review.md` (none exists for this module, per writeup §1) or to ratified `contract.md` (per `formal-verification-report.md §6` gate list: "Author `clarity-web/src/domain/contract.md`" is a new bead, not yet closed). The proof-to-implementation bridge requires contract ratification (per writeup §8 row 1: "Yes — required for proof-to-implementation bridge"). Without it, the obligations are floating.
- **Evidence:** `formal-verification-report.md §6` pre-flight gates: "Author `clarity-web/src/domain/contract.md` (and per-module analogues) | rust-contract | new" — this bead is unstarted. The 4 plans in the parallel proof program that do have plans (`cl-0n6`, `cl-zup`, `cl-5dp`, `cl-ooz`) still have `requires_contract: true` obligations flagged.
- **Required fix:** Out of `proof-writer`/`proof-reviewer` scope, but blocking: `rust-contract` must ratify the 9 Verus clauses and 21 proptest properties against `clarity-web/src/domain/contract.md` (or analogous per-module file) before the bridge can map them to behavior tests. `proof-plan-reviewer` must then approve. The writeup should be updated to mark the clauses `RATIFIED` (not `INFERRED`) once that lands. Until then, the bridge cannot begin.
- **Disposition:** `blocker` (owned by `rust-contract` + `proof-plan-reviewer`); not directly fixable in this bundle but blocks advancement.

### F4 — NON-BLOCKING — 8 of 21 proptest "properties" are deterministic single-shot checks

- **Artifact:** `proofs/straw_man_proptest.rs:97, 105, 125, 136, 147, 173, 184, 195`
- **Severity:** observation. The writeup claims "21 proptest properties" — accurate as a count of `#[test]` functions inside `proptest!`, but 8 of them (`prop_all_has_four_elements`, `prop_all_contains_every_variant_once`, `prop_label_is_nonempty`, `prop_description_is_detailed`, `prop_checkbox_label_is_a_question`, `prop_passing_is_passing`, `prop_default_equals_passing`, `prop_new_empty_equals_passing`) have signature `fn prop_xxx(_unused: ())` — no generator, no shrink, no variation. They are essentially the same checks already present in the production crate's `#[cfg(test)] mod tests` (lines 170-303 of `straw_man.rs`), re-wrapped in the proptest harness.
- **Evidence:** 8 matches for `_unused: ()` in the proptest file (counted via `rg -c '_unused: ()'` = 8); 13 properties have actual `in arb_...` generators.
- **Required fix (optional):** Either (a) move the 8 deterministic checks to the production crate's `#[cfg(test)] mod tests` and leave the 13 generator-based properties in `proofs/straw_man_proptest.rs`, or (b) keep them where they are and add a note in §6 of the writeup that 8/21 are deterministic re-checks of the production unit tests. Either is acceptable; the current state is just imprecise framing.
- **Disposition:** `owner_approved_debt` (proof-reviewer; not blocking).

### F5 — NON-BLOCKING — `prop_is_valid_iff_invariant_holds` only tests the forward direction

- **Artifact:** `proofs/straw_man_proptest.rs:230-233`
- **Severity:** observation. The name says "iff" but the body only checks the forward direction: `StrawManValidation::new(traps).is_valid()` is true. It does **not** test that a violation of the invariant causes `is_valid()` to return false (which would require direct field mutation, not possible through the public API in normal code). The reverse direction is therefore not exercised. This is a property of the public API (the constructors are total and the invariant holds by construction), so the missing direction is unavoidable in proptest without adding a `#[cfg(test)]` test helper that mutates fields.
- **Required fix (optional):** Either rename to `prop_new_produces_valid` (drop the "iff" claim), or add a separate `#[test]` in the production crate's `mod tests` that mutates `passed`/`traps_detected` directly to test the reverse direction (this is the only way to exercise `is_valid()` returning false, since the public constructors all satisfy the invariant).
- **Disposition:** `owner_approved_debt` (proof-reviewer; not blocking).

### F6 — NON-BLOCKING — `prop_validation_serde_roundtrip` invariant check is partially vacuous

- **Artifact:** `proofs/straw_man_proptest.rs:336-343`
- **Severity:** observation. The final assertion `prop_assert!(back.is_valid())` is true by construction of the round-trip: serde preserves both `passed` and `traps_detected` as a pair, so the invariant holds after parsing iff it held before serialization. The interesting test (the round-trip preserves the invariant **even when the input was constructed via direct field mutation**) cannot be expressed through proptest alone. The existing assertions on `back.passed == v.passed` and `back.traps_detected == v.traps_detected` are the real content; the `is_valid()` assertion is redundant.
- **Required fix (optional):** Drop the `back.is_valid()` assertion, or document that it is a defensive check on serde's own correctness rather than a separate property.
- **Disposition:** `owner_approved_no_action` (proof-reviewer; cosmetic).

### F7 — NON-BLOCKING — `prop_has_trap_implies_count_positive` is a one-direction implication

- **Artifact:** `proofs/straw_man_proptest.rs:239-247`
- **Severity:** observation. The property asserts `has_trap(trap) ⇒ trap_count() >= 1`. The reverse implication (`trap_count() >= 1 ⇒ has_trap(some_trap)`) is not tested but is well-covered by `prop_has_trap_iff_member` and `prop_trap_count_equals_len`. The "iff" name is therefore slightly inaccurate; the property is a one-direction implication with a tighter bound. Not a problem, just a label gap.
- **Disposition:** `owner_approved_no_action` (proof-reviewer; cosmetic).

### F8 — NON-BLOCKING — No `[lints] workspace = true` opt-in (workspace-level, not this artifact)

- **Artifact:** `clarity-web/Cargo.toml` (no `[lints]` table)
- **Severity:** observation, owned elsewhere. Per `formal-verification-report.md §3`, `cl-2vr` is P0 critical. It is a workspace-level gap that affects the whole crate, not this proof artifact. Mentioned here for completeness because the writeup's §7 acceptance criterion #6 (`cargo clippy --workspace --all-targets -- -D warnings` exits 0) is **explicitly stated as NOT owned by `cl-vv2`** (writeup §8: "Yes — `cl-2q6` baseline is `FAIL_LOCAL` (67 errors) per `formal-verification-report.md §2`").
- **Disposition:** `owner_approved_debt` (proof-reviewer; routed to `cl-2vr` / `cl-2q6`).

### F9 — NON-BLOCKING — 5 contract-gap flags honestly classified

- **Artifact:** `proofs/straw_man-writeup.md §3` flags 1-5
- **Severity:** observation. All 5 flags are honestly classified:
  1. STRUCTURAL_INVARIANT_VIOLATION_RISK — correct; `passed: bool` is `pub` (verified at `straw_man.rs:102`). Mitigation by `rust-contract` is correct routing.
  2. NAMING_AMBIGUITY — correct; the doc-vs-impl divergence is real.
  3. UNSPECIFIED_INVARIANT — the writeup claims this is "closed by PO-V3" — and indeed, the Verus spec pins `r@.len() > 20` as a postcondition, which means any future change to a shorter description **does** break the spec at compile time. The closure path is honest: the spec level upgrades the test convention. Still routed for production-code documentation update.
  4. IMPLICIT_DUPLICATION — correct; `Default::default()` calls `passing()` directly while `new(vec![])` goes through the constructor. The proptest `prop_default_equals_passing` and `prop_new_empty_equals_passing` catch divergence.
  5. API_GAP (informational, `has_trap` duplicates) — correct; current behavior is the documented "Vec-as-bag" model and is exercised by `prop_has_trap_ignores_duplicates`.
- **Disposition:** `owner_approved_debt` (proof-reviewer; flags are routed, no behavior-affecting gaps in current code).

---

## 3. Per-adversarial-question answers

### Q1 — Is the verifier rejection real, transient, env, or architectural?

**Real and structural, not transient.** Raw log at `/tmp/opencode/verus-straw_man.txt` shows exit 1 with a stable, reproducible vstd limitation. The error message itself prints the exact `assume_specification` signature that would resolve it. The fix is **mechanically trivial** (one declaration inside `verus!`) but the artifact as written does not verify. This is **not** an env issue, **not** a tool version issue, and **not** a deeper architectural problem in the spec — the spec structure is sound; only the std-trust boundary is underspecified.

The verifier help message says: "the vstd library provides some specification for the Rust std library, but it is currently limited". This is honest about the vstd surface. The trust ledger should reflect this.

### Q2 — Anti-laundering audit

- **Zero `#[verifier::external_body]`** on production code: confirmed. The only match in the file (line 21) is in a comment that says "We do **not** use `#[verifier::external_body]` to bind production functions to specs".
- **Zero `assume(`:** confirmed. No matches.
- **Zero `axiom`:** confirmed. No matches.
- **Zero `#[verifier::external]`:** confirmed. No matches.
- **`external_type_specification` usage:** not used in the artifact — instead, the spec relies on vstd's built-in `Seq` view for `Vec` and the implicit vstd spec for `Vec::is_empty` / `Vec::len`. This is idiomatic Verus. The only trust boundary that needs an explicit declaration is `<[T]>::contains` (which vstd does not provide), and that declaration is **missing** (which is what the verifier rejected). The trust ledger claim that the std-trust boundary is "pinned" is misleading.

**Verdict on anti-laundering:** The body-copies are verbatim and well-cited, but the spec is not self-sufficient — it depends on a vstd capability (`<[T]>::contains`) that does not exist. The verifier caught this. The proof-writer's claim of "no `assume`/`axiom`/`external_body` shortcuts on production code" is technically true but the trust ledger claim of "no spec gap" is **false**.

### Q3 — 5 contract-gap flags, honestly classified?

Yes (see F9). All 5 flags are honestly classified. None are behavior-affecting in the current code. The `UNSPECIFIED_INVARIANT` (flag 3) is genuinely closed by PO-V3: the Verus spec pins the convention, so a future regression breaks the spec compile.

### Q4 — 21 proptest properties, each invokes production API?

All 21 invoke the production API. No test-of-a-test detected. No local re-implementation. No shadow functions. The `arb_trap()` and `arb_trap_vec()` generators are minimal and correct (they enumerate the 4 production variants explicitly, so they don't smuggle in extra inputs).

However, 8 of 21 properties have no generator (signature `(traps in arb_trap_vec())` and similar are absent; they take `_unused: ()`). These are deterministic single-shot checks duplicated from the production `mod tests` block. They are not "property-based" in the proptest sense — they are unit tests re-wrapped in the proptest macro. (See F4.)

### Q5 — 9/10 Verus functions spec'd; is that the right call?

Yes. `Default::default()` at `straw_man.rs:145-149` is a one-line delegation to `passing()`. The transitive invariant is covered by PO-V6 (`passing()`'s spec). The proptest `prop_default_equals_passing` (PO-P8) catches any future divergence. There is no useful additional invariant to prove on `Default` itself.

The deferred-function call is honest. The writeup §5 documents it explicitly.

### Q6 — 6 trust ledger entries, each verified?

| # | Entry | Verdict | Evidence |
|---|---|---|---|
| 1 | `vstd::seq::Seq::contains` | Correct | Used in PO-V7 postcondition; vstd does provide this. |
| 2 | `Vec::is_empty`, `Vec::len`, `Vec::contains` | **Partially wrong** | `is_empty` and `len` are in vstd; `Vec::contains` is not — the verifier error at line 220 proves the absence. The trust entry is overstated. (See F2.) |
| 3 | `&'static [Self; N]` slice coercion | Correct | Rust language rule; no verifier interaction. |
| 4 | `StrawManTrap` is closed | Correct | Adding a 5th variant requires updating `all_variant_at` and `ALL_LEN`; would break the spec at compile time. |
| 5 | `serde_json` round-trip | Correct | Library contract, X lane; exercised by proptest PO-P19, PO-P20. |
| 6 | `vstd` semantics (`Seq::last`, `Seq::contains`, `Seq::len`) | Correct | `Seq::last` used in PO-V4; `Seq::contains` in PO-V7; `Seq::len` in PO-V1, PO-V6, PO-V8. All in vstd. |

Entry #2 is the only one that needs correction. The other 5 are honest.

---

## 4. Anti-laundering affirmation (reviewer's audit)

The proof-writer's anti-laundering claim holds for the most part:

- Verus `exec fn` bodies are **verbatim** copies of the production functions (verified by side-by-side comparison for `all`, `label`, `description`, `checkbox_label`, `new`, `passing`, `has_trap`, `trap_count`, `is_valid`).
- Every proptest property calls the production API via `use clarity_web::domain::straw_man::{StrawManTrap, StrawManValidation};` (verified at line 57).
- No `#[verifier::external_body]`, no `assume`, no `axiom`, no `#[verifier::external]` (verified by grep).
- The 9 `exec fn` → 9 production methods mapping is 1-to-1 and accurate.

**The one place the anti-laundering claim breaks down is the trust ledger.** It claims `Vec::contains` is pinned via vstd external_type_specification. It is not. The verifier error is the canonical evidence.

This is not a lazy proof — it is a **mis-documented** proof. The proof is structurally sound; the trust ledger is the only dishonest element.

---

## 5. Recommendation

**Fix-and-reverify.** The mechanical fix is one `assume_specification` line in `proofs/straw_man_verus.rs`. The trust ledger needs a correction. After both:

1. Re-run `verus proofs/straw_man_verus.rs` and capture exit-0 evidence in `verification-ledger.jsonl` (per `formal-verification-report.md §9` step 4).
2. Update `proof-evidence.md §4` closure criteria to mark PO-V1..PO-V9 as PASS.
3. Address the upstream `cl-ev9` repair bead and the workspace-level `cl-2vr` (P0 critical, separate bead).

Do **not** advance to `proof-to-implementation` bridge until (a) Verus verifies with exit 0, (b) trust ledger is corrected, and (c) `rust-contract` ratifies the 9 Verus + 21 proptest clauses in `clarity-web/src/domain/contract.md`.

---

## 6. Disposition summary

| Finding | Severity | Disposition |
|---|---|---|
| F1 — Verus spec does not verify | blocker | `fixed_with_evidence` (required) |
| F2 — Trust ledger inconsistent with verifier | blocker | `fixed_with_evidence` (required) |
| F3 — No upstream plan ratification | blocker (procedural) | routed to `rust-contract` + `proof-plan-reviewer` |
| F4 — 8/21 proptest properties are deterministic | observation | `owner_approved_debt` |
| F5 — `prop_is_valid_iff_invariant_holds` is one-direction | observation | `owner_approved_debt` |
| F6 — `prop_validation_serde_roundtrip` partially vacuous | observation | `owner_approved_no_action` |
| F7 — `prop_has_trap_implies_count_positive` is one-direction | observation | `owner_approved_no_action` |
| F8 — Workspace lints not opted in (cl-2vr) | observation | routed to `cl-2vr` (out of scope) |
| F9 — 5 contract-gap flags honestly classified | observation | `owner_approved_debt` |

**Blockers: 3.** Non-blocking: 6 observations. No fixed_with_evidence yet (none resolved). No low/minor/observation/informational findings left unresolved that block approval — all observations carry an explicit disposition.

**Final status: REJECTED.** Advancement to `proof-to-implementation` bridge is blocked until F1, F2, and F3 are resolved.

```
STATUS: REJECTED
```
