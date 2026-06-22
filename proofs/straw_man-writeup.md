# Proof Writeup — `clarity-web/src/domain/straw_man.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-vv2` |
| **Verifier** | `proof-writer` |
| **Target** | `clarity-web/src/domain/straw_man.rs` (303 LOC) |
| **Primary lane** | **V** (Verus) |
| **Secondary lane** | **P** (proptest) |
| **Date** | 2026-06-21 |
| **Companion files** | `formal-verification-report.md`, `verification-targets.md §5.1`, `proofs/straw_man_verus.rs`, `proofs/straw_man_proptest.rs` |
| **Production lint posture** | `unwrap_used=deny`, `expect_used=deny`, `panic=deny`, `todo=deny`, `unimplemented=deny`, `unsafe_code=forbid` (workspace-level, `Cargo.toml:9-22`) |

---

## 1. Upstream note (no approved plan)

No `proof-plan-review.md` or `proof-obligations.planned.jsonl` exists yet for this
module. The Verifying-Targets roadmap at `verification-targets.md §5.1` classifies
`straw_man.rs` as **V + P**. All clauses below are **inferred from source** and
labelled `INFERRED`. They will need ratification by `rust-contract` before the
`proof-to-implementation` bridge can map them to behavior tests.

The plan for `storage/types.rs` (`proofs/storage-types-proof-plan.md`) sets the
template; this writeup follows it for the `domain/straw_man.rs` module.

---

## 2. Module characterisation

`straw_man.rs` is **pure data**: zero I/O, zero concurrency, zero `unsafe`
(`#![forbid(unsafe_code)]` at module top, line 6). It declares:

- One `serde::{Serialize, Deserialize}`-bearing enum `StrawManTrap` (4 variants).
- One `serde::{Serialize, Deserialize}`-bearing struct `StrawManValidation`
  with a `pub` invariant: `passed == traps_detected.is_empty()`.
- 9 `pub` methods total, all const fn or trivial `Vec::contains`/`Vec::len`.
- One `Default` impl (delegates to `passing()`).
- A `#[cfg(test)] mod tests` block of 9 hand-written tests (lines 170–303).

| Item | Lines | Kind | Risk surface |
|---|---|---|---|
| `enum StrawManTrap { IrrationalActor, ManicPixieDreamUser, StoicMonk, YourClone }` | 14–31 | 4-variant bounded enum | Serde mapping (X lane), Hash/Clone/Copy traits |
| `impl StrawManTrap::all` | 36–43 | Returns 4-element slice, fixed order | Total enumeration, order |
| `impl StrawManTrap::label` | 47–54 | Returns per-variant `&'static str` | Non-emptiness |
| `impl StrawManTrap::description` | 58–77 | Returns per-variant `&'static str` | Non-emptiness, >20 char convention (test-only) |
| `impl StrawManTrap::checkbox_label` | 81–88 | Returns per-variant `&'static str` | Non-emptiness, ends-with-`?` convention |
| `struct StrawManValidation { traps_detected, passed }` | 96–103 | Owned `Vec` + `bool` | **Invariant**: `passed == is_empty()` |
| `impl StrawManValidation::new` | 108–114 | Total constructor | Enforces invariant |
| `impl StrawManValidation::passing` | 118–123 | Empty + passed=true constructor | Enforces invariant |
| `impl StrawManValidation::has_trap` | 127–129 | Membership check | `Vec::contains` semantics |
| `impl StrawManValidation::trap_count` | 133–135 | Length | `Vec::len` semantics |
| `impl StrawManValidation::is_valid` | 140–142 | Invariant predicate | Doc says "is valid"; impl is "passed == is_empty" |
| `impl Default for StrawManValidation` | 145–149 | Delegates to `passing()` | Transitive from `passing()` |

Refinement-style properties that are **documented** but **not enforced** by the
type system:

- `description()` minimum length 20 — enforced by unit test (line 200), not by
  a type-system invariant. The Verus spec upgrades this to a spec-level
  invariant on the function.
- `checkbox_label()` ends-with-`?` — enforced by unit test (line 216). The Verus
  spec upgrades this to a spec-level invariant.
- The `passed == is_empty` invariant on `StrawManValidation` is **not**
  type-system-enforced because the `passed: bool` field is `pub`. Direct field
  assignment can break it. `is_valid()` is the runtime detector.

---

## 3. Contract-gap flags

These are gaps in the **production contract** that the proof artifacts must
navigate. Each is labelled `CONTRACT_GAP` and routed to `rust-contract` /
`holzman-rust` for closure. None of them block proof writing — the proofs pin
the existing observable behavior.

### Flag 1 — `passed: bool` is `pub`, breaking the invariant

- **Location**: `straw_man.rs:102`
- **Risk**: External code can mutate `passed` directly, e.g.
  `v.passed = true; v.traps_detected.push(StoicMonk);` — `is_valid()` returns
  `false` afterwards.
- **Proof posture**: Verus spec on `new`, `passing`, and `Default::default` pins
  the invariant **on construction**. `is_valid()` is proven to detect a
  violation. The structural-enforcement gap is documented; the runtime check
  is verified.
- **Recommended fix**: Make `passed` `pub(crate)` and add a constructor-only
  mutator API, or wrap the field in a private tuple struct. Owner: `rust-contract`.

### Flag 2 — `is_valid()` name vs. semantics

- **Location**: `straw_man.rs:140`
- **Doc comment**: "Check if validation is valid (passed field matches `traps_detected.is_empty()`)"
- **Implementation**: returns `self.passed == self.traps_detected.is_empty()`
- **Issue**: The name suggests "is this a valid validation result" (semantic),
  but the implementation checks internal consistency (syntactic). A reader
  expects `is_valid()` to mean "the persona description was acceptable" — but
  the function says nothing about the persona description; it only checks the
  flag-vs-list coherence.
- **Proof posture**: The Verus spec faithfully mirrors the production name and
  body, but flags the ambiguity in a doc comment.
- **Recommended fix**: Rename to `invariant_holds()` or `is_consistent()`.
  Owner: `holzman-rust` for the rename; `proof-writer` to update the spec.

### Flag 3 — `description()` minimum-length is test-only, not contract

- **Location**: `straw_man.rs:200` (test assertion); `straw_man.rs:58-77` (impl).
- **Issue**: The unit test asserts `desc.len() > 20` but no spec on the
  production code enforces this. If a future edit reduces a description to 19
  chars, no compile-time check fails; the test fails at CI.
- **Proof posture**: The Verus spec on `description()` **upgrades** this to a
  spec-level invariant: `ensures r@.len() > 20`. This pins the test convention
  into the executable spec. If the production code changes to a shorter
  description, the Verus artifact must be updated — caught at spec compile time.
- **Recommended fix**: Promote to a `#[doc]` requirement on the function and/or
  add a `#[test]` that documents the invariant in the production crate.
  Owner: `holzman-rust` for documentation; `rust-contract` for ratification.

### Flag 4 — `Default::default()` and `new(vec![])` are observationally equal but topologically divergent

- **Location**: `straw_man.rs:118-123` (`passing()`), `straw_man.rs:145-149` (`Default`).
- **Issue**: `Default::default()` calls `passing()`, not `new(vec![])`. Both
  produce `passed = true`, empty list. If `passing()` ever evolves (e.g. to
  include a `validated_by: Option<ActorId>` field for auditing), `default()`
  picks up the change but `new(vec![])` does not. A silent fork is possible.
- **Proof posture**: proptest property `prop_default_equals_passing` asserts the
  current equality; if the production code diverges, the property test fails.
- **Recommended fix**: Have `Default::default()` call `new(vec![])`, or have
  `new(vec![])` delegate to `passing()`. Owner: `holzman-rust`.

### Flag 5 — `has_trap(t)` does not check duplicates

- **Location**: `straw_man.rs:127-129`.
- **Issue**: `has_trap(t)` returns true even if `t` appears 100 times in
  `traps_detected`. `trap_count()` returns 100. This is consistent with the
  "Vec-as-bag" data model (duplicates are preserved), but a reader expecting
  set semantics might be surprised.
- **Proof posture**: proptest `prop_has_trap_ignores_duplicates` and
  `prop_trap_count_equals_len` document this behavior. No contradiction.
- **Recommended fix**: None — current behavior is documented and tested.

**Total contract-gap flags: 5.**

---

## 4. Spec'd functions (Verus)

Each row names the production function, the spec location in
`proofs/straw_man_verus.rs`, and the spec kind.

| # | Function | Source line | Spec location | Kind |
|---|---|---|---|---|
| 1 | `StrawManTrap::all` | `straw_man.rs:36-43` | `verus!` impl block, `all()` | `requires` none; `ensures r@.len() == 4` and `forall i: 0..4, r@[i] == all_variant_at(i)` |
| 2 | `StrawManTrap::label` | `straw_man.rs:47-54` | `verus!` impl block, `label()` | `ensures r@.len() > 0` |
| 3 | `StrawManTrap::description` | `straw_man.rs:58-77` | `verus!` impl block, `description()` | `ensures r@.len() > 20` (closes contract gap #3) |
| 4 | `StrawManTrap::checkbox_label` | `straw_man.rs:81-88` | `verus!` impl block, `checkbox_label()` | `ensures r@.len() > 0` and `r@.last() == '?'` |
| 5 | `StrawManValidation::new` | `straw_man.rs:108-114` | `verus!` impl block, `new()` | `ensures validation_invariant_holds(r)` |
| 6 | `StrawManValidation::passing` | `straw_man.rs:118-123` | `verus!` impl block, `passing()` | `ensures validation_invariant_holds(r)` ∧ `traps@.len() == 0` ∧ `passed == true` |
| 7 | `StrawManValidation::has_trap` | `straw_man.rs:127-129` | `verus!` impl block, `has_trap()` | `ensures r == self.traps_detected@.contains(trap)` |
| 8 | `StrawManValidation::trap_count` | `straw_man.rs:133-135` | `verus!` impl block, `trap_count()` | `ensures r as int == self.traps_detected@.len()` |
| 9 | `StrawManValidation::is_valid` | `straw_man.rs:140-142` | `verus!` impl block, `is_valid()` | `ensures r == validation_invariant_holds(*self)` |

**Verus spec count: 9 functions spec'd + 1 enum type (`StrawManTrap`) + 1 struct type (`StrawManValidation`) + 3 closed spec fns (`ALL_LEN`, `all_variant_at`, `validation_invariant_holds`).**

---

## 5. Deferred functions

| # | Function | Source line | Reason for deferral |
|---|---|---|---|
| 1 | `Default::default()` | `straw_man.rs:145-149` | Delegates to `passing()`. The transitive invariant is covered by `passing()`'s spec. No additional proof needed. (If `Default` ever diverges, see contract gap #4.) |

**Deferred function count: 1.**

---

## 6. Proptest properties

`proofs/straw_man_proptest.rs` contains **21 properties** across 8 groups:

| Group | Property | Source line | Property count |
|---|---|---|---|
| A | `prop_all_has_four_elements`, `prop_all_contains_every_variant_once` | `straw_man.rs:36-43` | 2 |
| B | `prop_label_is_nonempty`, `prop_description_is_detailed`, `prop_checkbox_label_is_a_question` | `straw_man.rs:47-88` | 3 |
| C | `prop_new_preserves_passed_matches_empty`, `prop_passing_is_passing`, `prop_default_equals_passing`, `prop_new_empty_equals_passing` | `straw_man.rs:108-149` | 4 |
| D | `prop_has_trap_iff_member`, `prop_trap_count_equals_len`, `prop_is_valid_iff_invariant_holds`, `prop_has_trap_implies_count_positive` | `straw_man.rs:127-142` | 4 |
| E | `prop_has_trap_is_order_invariant`, `prop_trap_count_is_order_invariant`, `prop_passed_is_order_invariant` | `straw_man.rs:108-142` | 3 |
| F | `prop_has_trap_ignores_duplicates`, `prop_duplicates_dont_change_passed` | `straw_man.rs:108-135` | 2 |
| G | `prop_trap_serde_roundtrip`, `prop_validation_serde_roundtrip` | `straw_man.rs:14, 95, 282-303` | 2 |
| H | `prop_any_public_construction_is_valid` | `straw_man.rs:108-149` | 1 |

**Total proptest property count: 21.**

(Counted by the `#[test]` annotations under `proptest!`.)

---

## 7. Verifier command expectations

### Verus

The Verus artifact at `proofs/straw_man_verus.rs` is a standalone file with
`use vstd::prelude::*;` and a `verus! { ... }` block. The expected command is:

```bash
verus proofs/straw_man_verus.rs
```

Tooling precondition: `command -v verus` resolves to `/home/lewis/.local/bin/verus`
(version `0.2026.05.05.d03e906`, per `formal-verification-report.md`).

Acceptance criteria:

- Exit code 0.
- No `assume`, `#[verifier::external_body]` (for our own code), or `axiom` outside
  the standard vstd. The std-trust boundary (`Vec::contains` via
  `external_type_specification`) is allowed by the verifier-commands reference.
- Each of the 9 `exec fn` postconditions verifies.

### proptest

The proptest artifact at `proofs/straw_man_proptest.rs` imports from
`clarity_web::domain::straw_man::*;`. To execute it, the file must be reachable
from `cargo test`. Two wiring options, in order of preference:

1. **Move the file** to `clarity-web/tests/straw_man_proptest.rs` (recommended).
   The file as written compiles against `clarity-web`'s published API and uses
   the existing `[dev-dependencies] proptest = "1.10.0"` declaration. Command:

   ```bash
   cargo test -p clarity-web --test straw_man_proptest
   ```

2. **Reference the file** from an existing integration test (e.g. add
   `#[path = "../../proofs/straw_man_proptest.rs"] mod straw_man_proptest;` at
   the top of `clarity-web/tests/quality_gate_integration_test.rs`). This
   requires no file move but pollutes the existing test module.

The expected command is option 1:

```bash
cargo test -p clarity-web --test straw_man_proptest
```

---

## 8. Open obligations / handoff

| Action | Owner | Blocking? |
|---|---|---|
| Ratify the 9 Verus clauses and 21 proptest properties above as part of the `straw_man` contract | `rust-contract` | Yes — required for `proof-to-implementation` bridge |
| Decide on contract-gap #1 (structurally enforce `passed == is_empty` via private field) | `rust-contract` + `holzman-rust` | No — proof mirrors current behavior |
| Decide on contract-gap #2 (rename `is_valid` → `invariant_holds`) | `holzman-rust` | No — proof mirrors current name |
| Decide on contract-gap #4 (collapse `Default`/`new(vec![])` duplication) | `holzman-rust` | No — proptest catches divergence |
| Re-run `cargo clippy --workspace --all-targets -- -D warnings` after the rename / collapse fixes | `formal-verifier` | Yes — `cl-2q6` baseline is `FAIL_LOCAL` (67 errors) per `formal-verification-report.md §2` |
| Build a `proof-to-implementation` bridge: map each Verus spec + proptest property to (a) source `path:line`, (b) independent behavior test in `#[cfg(test)] mod tests` or new `tests/`, (c) refinement harness ref | `proof-to-implementation` | Yes — required before `formal-verifier` closes obligations |

---

## 9. Non-targets (explicit)

Per `verification-targets.md §8`:

- **Line-by-line proofs.** Refused; cost-benefit is wrong for glue/serde code.
- **UI rendering proofs.** Out of scope; `checkbox_label()` and `label()` are
  exercised by the proptest string-content properties, not by a UI test.
- **Pure serde round-trip proofs.** Serde is the **X lane** (exercise-only);
  the proptest `prop_*_serde_roundtrip` properties are sufficient guardrails.
- **The `#[cfg(test)] mod tests` block** (`straw_man.rs:151-303`) is exercised
  behavior; the proptest properties above supersede (but do not remove) it.
- **Miri / Loom / TLA+ / fuzz / Kani / Flux** — all not applicable to this
  module. No `not_applicable` obligation rows for these will be promoted to
  `waived` (none of them are owed for this module).

---

## 10. Trust base summary

| Trust | Why trusted | Mitigation in this artifact |
|---|---|---|
| `Vec::is_empty()`, `Vec::len()` | Rust std lib contract; vstd provides `external_type_specification` for these | Used in PO-V5, PO-V6, PO-V8, PO-V9 specs |
| `<[T]>::contains` (slice method) | **NOT in vstd.** `assume_specification` required at use site (`straw_man_verus.rs:220` — PO-V7). The declaration is an honest trust boundary: it asserts the method returns `bool` with standard `PartialEq` semantics, pinning the postcondition `r == self.traps_detected@.contains(trap)`. | `assume_specification` declaration in `verus!` block before `impl StrawManValidation`; postcondition then provable via the `Seq` view |
| `vstd::seq::Seq::contains` semantics | Verus std spec, not our code | Used in PO-V7 postcondition `self.traps_detected@.contains(trap)`; vstd provides this |
| `&'static [Self; N]` slice coercion | Rust language rule | Production code uses the same coercion |
| `StrawManTrap` is closed (no future variant) | Type-system at spec time | `all_variant_at` enumerates 4 indices; adding a 5th variant breaks the spec |
| `serde_json` round-trip preserves values | Library contract; not our code | proptest `prop_*_serde_roundtrip` re-parses and asserts equality |
| `vstd` semantics (`Seq::last`, `Seq::len`) | Verus std spec | Direct spec fn calls; `Seq::last` in PO-V4, `Seq::len` in PO-V1, PO-V6, PO-V8 |

No `assume`, `axiom`, `admit`, or `external_body` shortcuts on **our** production
code. The std-trust boundary for `<[T]>::contains` is the only external trust that
requires an explicit `assume_specification` declaration. `Vec::is_empty` and
`Vec::len` are in vstd and need no declaration. Both are the standard idiomatic
pattern for Verus.

---

## 11. Pre-flight checklist for landing this writeup

- [x] All 9 public functions spec'd in Verus.
- [x] Default impl covered transitively via `passing()` spec.
- [x] 21 proptest properties cover the same surface + serde round-trip + order/duplicate invariance.
- [x] 5 contract-gap flags raised and routed.
- [x] Path:line citations against `straw_man.rs` on every spec/property.
- [x] No production code edited; `proofs/` only.
- [ ] `rust-contract` ratifies the inferred clauses.
- [ ] `proof-plan-reviewer` reviews this writeup.
- [ ] Wiring step (move `straw_man_proptest.rs` to `clarity-web/tests/`) is logged before `cargo test` runs.
- [ ] `cl-2q6` clippy gate is closed independently (this artifact does not depend on it).

---

## 12. File index

| File | Purpose | Lane |
|---|---|---|
| `proofs/straw_man_verus.rs` | Verus spec/proof artifact (9 functions spec'd, 3 closed spec fns, type defs) | V |
| `proofs/straw_man_proptest.rs` | proptest property artifact (21 properties across 8 groups) | P |
| `proofs/straw_man-writeup.md` | This writeup | — |
