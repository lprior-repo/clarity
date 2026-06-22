# Domain Contract — `clarity-web/src/domain/straw_man.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-vv2` |
| **Module** | `clarity-web/src/domain/straw_man.rs` (303 LOC) |
| **Contract status** | **AUTHORED** (supersedes `INFERRED` clauses in `proofs/straw_man-review.md` and `proofs/straw_man-writeup.md`) |
| **Author** | `rust-contract` |
| **Date** | 2026-06-22 |
| **Notation** | EARS (Easy Approach to Requirements Syntax) + BDD (Given/When/Then) for state clauses |
| **Companion artifacts** | `proofs/straw_man_verus.rs`, `proofs/straw_man_proptest.rs`, `proofs/straw_man-review.md`, `proofs/straw_man-writeup.md` |

---

## §1 — Context

### 1.1 Module purpose

`straw_man.rs` is the **straw-man trap detection** module of the Clarity domain layer. Its purpose is to validate that persona descriptions represent **realistic users** rather than idealized or impossible user behaviors. The module exposes:

- A **closed 4-variant enum** `StrawManTrap` enumerating the four known adversarial-persona antipatterns (`IrrationalActor`, `ManicPixieDreamUser`, `StoicMonk`, `YourClone`).
- A **validation result struct** `StrawManValidation` recording which traps were detected in a persona description and whether the validation passed.

### 1.2 Module shape

| Property | Value |
|---|---|
| Public types | 1 enum (`StrawManTrap`) + 1 struct (`StrawManValidation`) |
| Public methods | 9 (4 on `StrawManTrap`, 5 on `StrawManValidation`) |
| Trait impls | `Default for StrawManValidation` (delegates to `passing`) |
| Derives on `StrawManTrap` | `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize` |
| Derives on `StrawManValidation` | `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` |
| Errors | **None** (no `Result<T, E>`, no error type) |
| Async / FFI / `unsafe` | **None** (`#![forbid(unsafe_code)]` at line 6) |
| Time / network / storage / I/O | **None** (pure data) |

### 1.3 User-supplied description vs. actual surface

The user's task brief hypothesized the following elements, which **do not exist** in the source:

| Hypothesized | Reality in `straw_man.rs` |
|---|---|
| `StrawManType` enum | The enum is `StrawManTrap`, not `StrawManType`. |
| `Active`, `NoProposal` variants | Not present. The 4 variants are exactly `IrrationalActor`, `ManicPixieDreamUser`, `StoicMonk`, `YourClone` (source `straw_man.rs:15-31`). |
| `with_description`, `with_verification` builder methods | Not present. No builder methods exist. |
| `decision_choice` logic | Not present. No decision-choice generation exists. |

This contract documents the **actual** public surface per the source. No invented behavior.

### 1.4 Risk surface

- **Single typed invariant** (`passed == traps_detected.is_empty()` on `StrawManValidation`) is structurally enforced on construction only (via `pub` fields, the invariant is breakable by direct mutation).
- **Closed enum cardinality** is 4 — adding a fifth variant is a breaking change.
- **Display strings** carry a UI-rendering convention (`label` non-empty, `description` > 20 chars, `checkbox_label` ends with `?`).
- **Serde derives** are library-contract (X lane); behavior is exercised but not proved.

---

## §2 — Smell Classification

Per `rust-contract` skill `type-contract-checklist.md`:

| Check | Status | Note |
|---|---|---|
| Replace stringly IDs / primitives with newtypes | ✅ N/A | Module has no stringly IDs or primitive domain values. |
| Replace boolean behavior flags with enums | ⚠ Partial | `passed: bool` is an **internal-consistency flag**, not a behavior-routing flag. Acceptable. |
| Replace `Option` lifecycle state with explicit state variants | ✅ N/A | No `Option` fields. |
| Parse external input once at the boundary | ✅ N/A | No parsing. |
| Represent domain failures with semantic error variants | ✅ N/A | No `Result<T, E>`; the module has no failures. |
| Pure core, free of I/O / time / network / storage / randomness | ✅ Pass | `#![forbid(unsafe_code)]` at line 6. No I/O. |

**Two smells are present and flagged for `holzman-rust` repair:**

### 2.1 SMELL.MUTABLE_INVARIANT_FIELDS

- **Source**: `straw_man.rs:102` — `pub passed: bool`.
- **Issue**: The `passed` field is `pub`, allowing direct mutation that breaks the invariant `passed == traps_detected.is_empty()`. The `is_valid()` method is the runtime detector, but the type system does not structurally prevent the inconsistency.
- **Severity**: Structural gap; mitigated by all public constructors enforcing the invariant.
- **Routing**: `holzman-rust` + `rust-contract`. See §9 DN-1.

### 2.2 SMELL.NAME_SEMANTICS_MISMATCH

- **Source**: `straw_man.rs:140` — `is_valid()` doc says "Check if validation is valid (passed field matches `traps_detected.is_empty()`)".
- **Issue**: The name suggests semantic validity ("is this persona acceptable?"), but the implementation checks internal consistency (`passed == is_empty()`). A reader expects `is_valid()` to mean "the persona description passed review", but the function says nothing about the persona — only flag-vs-list coherence.
- **Severity**: Naming hazard; mitigated by doc-comment.
- **Routing**: `holzman-rust` for rename. See §9 DN-2.

---

## §3 — Ubiquitous Invariants

> *EARS: invariants hold unconditionally across all observable states of the module.*

### UI-1 — Closed 4-variant enum cardinality

**REQ-SM-1 (partial)**: The system **SHALL** treat `StrawManTrap` as a closed enumeration of exactly four variants: `IrrationalActor`, `ManicPixieDreamUser`, `StoicMonk`, `YourClone`. **WHERE** a fifth variant is added, the contract fails at spec-compile time (the Verus `all_variant_at` enumerates 4 indices).

### UI-2 — Constructor-enforced consistency invariant

**REQ-SM-5, REQ-SM-6**: The system **SHALL** enforce `passed == traps_detected.is_empty()` for every `StrawManValidation` produced via the public API (`new`, `passing`, `Default::default`). The invariant is structural on construction only; field-mutation through `pub passed` is not enforced (see §7 KNOWN-LIMITATION-1 and §9 DN-1).

### UI-3 — Canonical enumeration order

**REQ-SM-1**: `StrawManTrap::all()` **SHALL** return `[IrrationalActor, ManicPixieDreamUser, StoicMonk, YourClone]` in that fixed order. **IF** a caller iterates `all()`, **THEN** the iteration order is deterministic and stable across calls.

### UI-4 — Display string contents

**REQ-SM-2, REQ-SM-3, REQ-SM-4**: For every `t ∈ StrawManTrap`, the system **SHALL** guarantee:

| Method | Property |
|---|---|
| `t.label()` | `result.len() > 0` |
| `t.description()` | `result.len() > 20` (closing writeup §3 contract gap #3) |
| `t.checkbox_label()` | `result.len() > 0` **AND** `result.ends_with('?')` |

### UI-5 — Bag semantics for `traps_detected`

**REQ-SM-7, REQ-SM-8**: The system **SHALL** treat `traps_detected` as a `Vec<StrawManTrap>` (preserves order, allows duplicates, counts duplicates). `has_trap(t)` returns `true` iff `t` appears at least once; `trap_count()` returns the verbatim length.

### UI-6 — Serde transparency (X lane)

**REQ-SM-SERDE**: The system **SHALL** round-trip both `StrawManTrap` and `StrawManValidation` through `serde_json` without loss. This is library-contract (X lane, exercise-only); the derives are not behavior under our control but are exercised by proptest.

### UI-7 — Default equals passing

**REQ-SM-6**: `Default::default()` for `StrawManValidation` **SHALL** produce the same observable state as `StrawManValidation::passing()` (delegation per source `straw_man.rs:145-149`). This is enforced transitively by UI-2.

---

## §4 — State-Driven Clauses

> *EARS: state predicates on observable `StrawManValidation` values. BDD Given/When/Then for the construction transitions.*

### SD-1 — `StrawManValidation::new(vec![])`

**Given** an empty trap list,
**When** `new(vec![])` is called,
**Then** the resulting validation **SHALL** satisfy:
- `result.passed == true`
- `result.traps_detected == vec![]`
- `result.is_valid() == true`
- `result.trap_count() == 0`

*(Equivalent in observable shape to `passing()` and `default()`.)*

### SD-2 — `StrawManValidation::new(non_empty_vec)`

**Given** a non-empty trap list `v` of length `n ≥ 1`,
**When** `new(v)` is called,
**Then** the resulting validation **SHALL** satisfy:
- `result.passed == false`
- `result.traps_detected == v` (preserving order and duplicates verbatim)
- `result.is_valid() == true`
- `result.trap_count() == v.len() == n`

### SD-3 — `StrawManValidation::passing()`

**Given** no input,
**When** `passing()` is called,
**Then** the resulting validation **SHALL** be observably identical to `new(vec![])`:
- `result.passed == true`
- `result.traps_detected` is empty
- `result.is_valid() == true`
- `result.trap_count() == 0`

### SD-4 — `StrawManValidation::default()`

**Given** no input,
**When** `Default::default()` is called,
**Then** the result **SHALL** be observably identical to `passing()` (UI-7 delegation).

### SD-5 — Predicate truth table (post-construction)

| `traps_detected` | `passed` | `has_trap(t)` for any `t` | `trap_count()` | `is_valid()` |
|---|---|---|---|---|
| `vec![]` | `true` | `false` | `0` | `true` |
| `vec![t₁, …, tₙ]`, `n ≥ 1` | `false` | `true` iff `t ∈ traps_detected` (bag semantics) | `n` (duplicates counted) | `true` (invariant holds) |
| **Inconsistent** (direct field write: `passed=true` with non-empty, OR `passed=false` with empty) | — | — | — | **`false`** (invariant violation detected) |

The third row is observable **only via direct field mutation**; the public API cannot produce it (UB-1). This is the runtime detector behavior; the structural gap is documented in §7 KNOWN-LIMITATION-1.

---

## §5 — Event-Driven Clauses

> *EARS: per-call behavior of observer functions. WHEN a query fires, THEN it returns the specified value.*

### ED-1 — `has_trap(t)` membership query

**REQ-SM-7**: **When** `has_trap(t)` is called on a validation `v`,
**Then** the system **SHALL** return `v.traps_detected.contains(&t)` (standard `Vec::contains` semantics, bag: at-least-once membership).
The query **SHALL** be pure (no mutation), idempotent (calling twice returns the same value), and order-independent.

### ED-2 — `trap_count()` length query

**REQ-SM-8**: **When** `trap_count()` is called on a validation `v`,
**Then** the system **SHALL** return `v.traps_detected.len()` as `usize`.
The query **SHALL** be pure, idempotent, order-independent, and **SHALL** count duplicates (per UI-5).

### ED-3 — `is_valid()` consistency query

**REQ-SM-9**: **When** `is_valid()` is called on a validation `v`,
**Then** the system **SHALL** return `v.passed == v.traps_detected.is_empty()`.
The query **SHALL** be pure and idempotent.
> ⚠ **Name caveat**: The name suggests semantic validity; the implementation checks internal consistency. See §2.2 and §9 DN-2.

### ED-4 — `StrawManTrap::all()` enumeration query

**REQ-SM-1**: **When** `all()` is called,
**Then** the system **SHALL** return a slice of length exactly 4 containing the four variants in fixed order `[IrrationalActor, ManicPixieDreamUser, StoicMonk, YourClone]`.

### ED-5 — `StrawManTrap::label(t)` display query

**REQ-SM-2**: **When** `label(t)` is called for any `t ∈ StrawManTrap`,
**Then** the system **SHALL** return a non-empty `&'static str` specific to `t`:

| Variant | `label()` returns |
|---|---|
| `IrrationalActor` | `"Irrational Actor"` |
| `ManicPixieDreamUser` | `"Manic Pixie Dream User"` |
| `StoicMonk` | `"Stoic Monk"` |
| `YourClone` | `"Your Clone"` |

### ED-6 — `StrawManTrap::description(t)` detail query

**REQ-SM-3**: **When** `description(t)` is called for any `t ∈ StrawManTrap`,
**Then** the system **SHALL** return a `&'static str` of length strictly greater than 20 characters, specific to `t`. **(Closes writeup §3 contract gap #3: this was previously a test-only convention, now a spec-level invariant.)**

| Variant | Approximate description content |
|---|---|
| `IrrationalActor` | "User acts against their own motivations or self-interest. Real users optimize for their own goals, not yours." |
| `ManicPixieDreamUser` | "User magically loves everything without discernment. Real users have preferences, constraints, and competing priorities." |
| `StoicMonk` | "User tolerates immense friction without complaint. Real users abandon products at the first sign of difficulty." |
| `YourClone` | "User has your system knowledge and mental models. Real users don't know what you know about how the system works." |

### ED-7 — `StrawManTrap::checkbox_label(t)` question query

**REQ-SM-4**: **When** `checkbox_label(t)` is called for any `t ∈ StrawManTrap`,
**Then** the system **SHALL** return a non-empty `&'static str` ending with `'?'`:

| Variant | `checkbox_label()` returns |
|---|---|
| `IrrationalActor` | `"acting against own motivations?"` |
| `ManicPixieDreamUser` | `"magically loves everything?"` |
| `StoicMonk` | `"tolerating immense friction?"` |
| `YourClone` | `"has your system knowledge?"` |

---

## §6 — Optional Features

> *EARS: features that are present but not strictly required by the core domain. WHERE a consumer uses them, behavior follows the stated contract.*

### OF-1 — Serde JSON round-trip (X lane, exercise-only)

**Where** the consumer serializes `StrawManTrap` or `StrawManValidation` to JSON and deserializes back,
**Then** the resulting value **SHALL** be equal to the original.
This is library-contract (X lane); no behavior-affecting obligation. Proptest `prop_trap_serde_roundtrip` and `prop_validation_serde_roundtrip` provide guardrail coverage.

### OF-2 — `Hash`, `Eq`, `Copy`, `Clone` derives on `StrawManTrap`

**Where** a caller uses `Hash`, `PartialEq`, `Eq`, `Copy`, or `Clone` on `StrawManTrap`,
**Then** behavior **SHALL** follow standard Rust semantics:
- `Hash` discriminates by variant.
- `PartialEq` / `Eq` are variant-equality (no payload fields).
- `Copy` / `Clone` are bitwise copies (no payload).

### OF-3 — `Clone`, `Eq`, `Debug` derives on `StrawManValidation`

**Where** a caller uses `Clone`, `PartialEq`, `Eq`, or `Debug` on `StrawManValidation`,
**Then** behavior **SHALL** follow standard Rust semantics:
- `Clone` deep-copies the `Vec<StrawManTrap>` and copies the `bool`.
- `PartialEq` / `Eq` compare both fields structurally.
- `Debug` prints both fields.

---

## §7 — Unwanted Behaviors

> *EARS: SHALL NOT clauses. These are negative invariants the system must preserve.*

### UB-1 — Public API SHALL NOT produce an invariant-violating `StrawManValidation`

The system **SHALL NOT** produce a `StrawManValidation` that violates `passed == traps_detected.is_empty()` when constructed via `new`, `passing`, or `Default::default`. **Verified by**: UI-2, SD-1, SD-2, SD-3, SD-4.

### UB-2 — Display strings SHALL NOT be empty

The system **SHALL NOT** return empty strings from `label()`, `description()`, or `checkbox_label()`. **Verified by**: UI-4, ED-5, ED-6, ED-7.

### UB-3 — Checkbox labels SHALL be questions

The system **SHALL NOT** return a `checkbox_label()` that does not end with `'?'`. **Verified by**: UI-4, ED-7.

### UB-4 — Descriptions SHALL be detailed

The system **SHALL NOT** return a `description()` of length ≤ 20. **Verified by**: UI-4, ED-6. **(Closes writeup §3 contract gap #3.)**

### UB-5 — `all()` enumeration SHALL NOT drift

The system **SHALL NOT** alter the contents or order of `all()`. Adding or reordering variants would break the Verus spec (`all_variant_at` enumerates 4 indices) and the proptest invariants. **Verified by**: UI-1, UI-3, ED-4.

### UB-6 — Observer purity

The system **SHALL NOT** mutate `self` (or any external state) when `has_trap`, `trap_count`, or `is_valid` is called. All three observers are pure. **Verified by**: ED-1, ED-2, ED-3.

### KNOWN-LIMITATION-1 — Field mutation can break the invariant (NOT PREVENTED)

The system **does NOT prevent** direct field mutation through `v.passed = …` or `v.traps_detected.push(…)` that violates the consistency invariant. **Where** a caller mutates fields directly, `is_valid()` returns `false` (the runtime detector). The structural enforcement gap is flagged in §2.1 (SMELL.MUTABLE_INVARIANT_FIELDS) and §9 DN-1. **Proof-writer note**: The Verus spec covers the forward direction (constructors satisfy UI-2); the reverse direction (mutation breaks the invariant) is verifiable but not enforced at compile time.

### KNOWN-LIMITATION-2 — `Default::default()` and `new(vec![])` are topologically divergent (NOT PREVENTED)

The system **does NOT guarantee** that `Default::default()` and `new(vec![])` remain observably equal in future versions. They currently produce identical state (UI-7), but `Default` delegates to `passing()` while `new(vec![])` constructs directly. **Where** `passing()` evolves (e.g. gains a `validated_by: Option<ActorId>` audit field), `default()` picks up the change but `new(vec![])` does not. The proptest `prop_default_equals_passing` and `prop_new_empty_equals_passing` **SHALL** catch divergence at test time. See §9 DN-3.

---

## §8 — Variants

> *The contract is parameterized over the following variants and shapes.*

### V-1 — `StrawManTrap` enum

| Variant | Source line | Display label | Display description (topic) | Checkbox question |
|---|---|---|---|---|
| `IrrationalActor` | 18 | "Irrational Actor" | acts against own motivations | "acting against own motivations?" |
| `ManicPixieDreamUser` | 22 | "Manic Pixie Dream User" | loves everything without discernment | "magically loves everything?" |
| `StoicMonk` | 26 | "Stoic Monk" | tolerates immense friction | "tolerating immense friction?" |
| `YourClone` | 30 | "Your Clone" | has developer system knowledge | "has your system knowledge?" |

The mapping is total and deterministic.

### V-2 — `StrawManValidation` constructor shapes

| Constructor | Source | Input | Output invariant |
|---|---|---|---|
| `new(v: Vec<StrawManTrap>)` | 108-114 | arbitrary trap list | `passed = v.is_empty()` |
| `passing()` | 118-123 | none | `passed = true`, `traps_detected = vec![]` |
| `Default::default()` | 145-149 | none | identical to `passing()` (UI-7) |

The three constructors produce observationally-equivalent passing states (`new(vec![])` ≡ `passing()` ≡ `default()`).

### V-3 — `StrawManValidation` observer shapes

| Observer | Source | Returns | Semantics |
|---|---|---|---|
| `has_trap(t)` | 127-129 | `bool` | membership (bag, ≥1 occurrence) |
| `trap_count()` | 133-135 | `usize` | length (counts duplicates verbatim) |
| `is_valid()` | 140-142 | `bool` | internal consistency (NOT semantic validity — see §2.2) |

There is **no `passed()` or `traps_detected()` getter**: both fields are `pub` and accessed directly.

### V-4 — Derive surface

| Type | Derives |
|---|---|
| `StrawManTrap` | `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize` |
| `StrawManValidation` | `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` |

`StrawManValidation` is **not** `Copy` (it owns a `Vec`) and **not** `Hash` (no hash impl on the struct).

---

## §9 — Design Notes

> *Recommendations for future repair work. Owners are stated for routing; this contract does not modify production code.*

### DN-1 — Make `passed` field private

- **Current**: `pub passed: bool` at `straw_man.rs:102`.
- **Issue**: KNOWN-LIMITATION-1 / SMELL.MUTABLE_INVARIANT_FIELDS (§2.1, §7).
- **Recommendation**: Make `passed` private (`pub(crate)` or fully private) and expose accessor + mutator API. Alternatively, wrap the struct in a private tuple with controlled access. Structurally prevents the invariant break.
- **Owner**: `holzman-rust` + `rust-contract`.
- **Proof impact**: The Verus spec on `new`, `passing`, and `Default` already pins UI-2 on construction; the structural fix would extend this to all construction paths. No Verus spec change required; behavior tests unchanged.

### DN-2 — Rename `is_valid` → `invariant_holds`

- **Current**: `is_valid()` at `straw_man.rs:140`.
- **Issue**: SMELL.NAME_SEMANTICS_MISMATCH (§2.2, ED-3 caveat).
- **Recommendation**: Rename to `invariant_holds()` or `is_consistent()`. The current name suggests semantic validity ("is this persona acceptable?") but the implementation checks internal consistency.
- **Owner**: `holzman-rust` for the rename.
- **Proof impact**: Update Verus spec on `is_valid` to reference the renamed function. Update proptest `prop_is_valid_iff_invariant_holds` to use the new name. The semantic postcondition is unchanged.

### DN-3 — Collapse `Default::default()` and `new(vec![])`

- **Current**: `Default::default()` delegates to `passing()`; `new(vec![])` constructs directly.
- **Issue**: KNOWN-LIMITATION-2 (§7).
- **Recommendation**: Have `Default::default()` call `new(vec![])`, OR have `new(vec![])` delegate to `passing()`. Eliminates the fork risk.
- **Owner**: `holzman-rust`.
- **Proof impact**: Proptest `prop_default_equals_passing` and `prop_new_empty_equals_passing` currently catch divergence; after the collapse, the proptest becomes redundant (the equality is structural).

### DN-4 — Document `description()` ≥21-char convention in source

- **Current**: 21-character minimum is enforced by unit test (`straw_man.rs:200`) and now by the Verus spec postcondition (ED-6, UB-4).
- **Recommendation**: Add a `/// # Invariants` or `/// # Display contract` doc-comment on `description()` documenting the length requirement. This makes the contract visible to readers of the source without consulting external artifacts.
- **Owner**: `holzman-rust` for documentation; already in contract per UI-4, ED-6, UB-4.

### DN-5 — Document bag semantics for `has_trap` and `trap_count`

- **Current**: `has_trap(t)` returns true for duplicates; `trap_count()` counts duplicates. Vec-as-bag model (UI-5).
- **Recommendation**: Add a doc-comment to both methods clarifying that `traps_detected` is a `Vec` (preserves order, allows duplicates), not a `Set`.
- **Owner**: `holzman-rust` for documentation.

### DN-6 — `<[T]>::contains` vstd trust boundary

- **Current**: `has_trap` uses `<[T]>::contains`, which is **NOT in vstd**. The Verus spec uses `assume_specification` to declare the signature as a trust boundary (`proofs/straw_man_verus.rs:198-199`).
- **Recommendation**: None — the trust boundary is honest and the F1 fix from `proofs/straw_man-review.md` is in place. Note for future-proofing: **where** vstd eventually provides a spec for `<[T]>::contains`, **then** the `assume_specification` declaration can be removed without behavioral change.

---

## §10 — Requirement-to-Obligation Traceability

> *Maps each contract clause (REQ-SM-N) to Verus / proptest obligations. Source: `proofs/straw_man-obligations.planned.jsonl`.*

| REQ | Contract clause | Verus obligation(s) | proptest obligation(s) |
|---|---|---|---|
| REQ-SM-1 | UI-1 (closed 4-variant), UI-3 (canonical order), ED-4 (all) | PO-VV2-V-01 | PO-VV2-P-01, PO-VV2-P-02 |
| REQ-SM-2 | UI-4 (label non-empty), ED-5 (label mapping) | PO-VV2-V-02 | PO-VV2-P-03 |
| REQ-SM-3 | UI-4 (description > 20), ED-6 (description mapping) | PO-VV2-V-03 | PO-VV2-P-04 |
| REQ-SM-4 | UI-4 (checkbox non-empty + `?`), ED-7 (checkbox mapping) | PO-VV2-V-04 | PO-VV2-P-05 |
| REQ-SM-5 | UI-2 (constructor invariant), SD-1, SD-2 | PO-VV2-V-05 | PO-VV2-P-06, PO-VV2-P-09 |
| REQ-SM-6 | UI-2, UI-7, SD-3, SD-4 | PO-VV2-V-06 | PO-VV2-P-07, PO-VV2-P-08 |
| REQ-SM-7 | UI-5 (bag semantics), ED-1 (has_trap) | PO-VV2-V-07 | PO-VV2-P-10, PO-VV2-P-13, PO-VV2-P-14, PO-VV2-P-17 |
| REQ-SM-8 | UI-5, ED-2 (trap_count) | PO-VV2-V-08 | PO-VV2-P-11, PO-VV2-P-15 |
| REQ-SM-9 | ED-3 (is_valid consistency) | PO-VV2-V-09 | PO-VV2-P-12 |
| REQ-SM-SERDE | UI-6 (serde transparency), OF-1 | (X lane, NA-02) | PO-VV2-P-19, PO-VV2-P-20 |
| (master invariant) | UB-1 (no invariant-violating public API) | V-05, V-06 cover constructors | PO-VV2-P-21 |
| (closure) | UI-5 (bag), UI-3 (order) | (covered by V-01..V-09) | PO-VV2-P-16, PO-VV2-P-18 |

**All 9 REQ-SM-N clauses and the REQ-SM-SERDE clause are anchored to at least one obligation.** No floating clauses.

---

## §11 — Gap Analysis (Contract vs. Verus spec vs. proptest)

### 11.1 Contract coverage of public surface

| Public item | Source line | Contract clause | Verus spec | proptest |
|---|---|---|---|---|
| `StrawManTrap` enum | 14-31 | UI-1, V-1 | mirrored (no `serde`) | covered via 4-variant arb generator |
| `StrawManTrap::all()` | 36-43 | UI-3, ED-4, UB-5 | PO-VV2-V-01 (POST `r@.len() == ALL_LEN()`) | PO-VV2-P-01, P-02 |
| `StrawManTrap::label()` | 47-54 | UI-4, ED-5, UB-2 | PO-VV2-V-02 (**vacuous**: `ensures true`) | PO-VV2-P-03 (4-case deterministic) |
| `StrawManTrap::description()` | 58-77 | UI-4, ED-6, UB-4 | PO-VV2-V-03 (**vacuous**: `ensures true`) | PO-VV2-P-04 (4-case deterministic) |
| `StrawManTrap::checkbox_label()` | 81-88 | UI-4, ED-7, UB-3 | PO-VV2-V-04 (**vacuous**: `ensures true`) | PO-VV2-P-05 (4-case deterministic) |
| `StrawManValidation` struct | 96-103 | UI-2, SD-1..SD-5, V-2, V-3, V-4 | type definition mirrored | (covered via `arb_validation_via_new`) |
| `StrawManValidation::new()` | 108-114 | UI-2, SD-1, SD-2 | PO-VV2-V-05 (POST `validation_invariant_holds(r)`) | PO-VV2-P-06, P-09, P-18 |
| `StrawManValidation::passing()` | 118-123 | UI-2, UI-7, SD-3 | PO-VV2-V-06 (POST `invariant_holds` ∧ `traps.len() == 0` ∧ `passed == true`) | PO-VV2-P-07 |
| `StrawManValidation::has_trap()` | 127-129 | UI-5, ED-1 | PO-VV2-V-07 (**vacuous**: `ensures true`; trust-boundary declared) | PO-VV2-P-10, P-13, P-14, P-17 |
| `StrawManValidation::trap_count()` | 133-135 | UI-5, ED-2 | PO-VV2-V-08 (POST `r as int == self.traps_detected@.len()`) | PO-VV2-P-11, P-15 |
| `StrawManValidation::is_valid()` | 140-142 | ED-3, UB-6, SD-5 | PO-VV2-V-09 (POST `r == validation_invariant_holds(*self)`) | PO-VV2-P-12 (forward only) |
| `Default for StrawManValidation` | 145-149 | UI-7, SD-4 | NA-01 (transitive via V-06) | PO-VV2-P-08 |
| Serde derives on `StrawManTrap` | 14 | UI-6, OF-1 | (X lane, NA-02) | PO-VV2-P-19 |
| Serde derives on `StrawManValidation` | 95 | UI-6, OF-1 | (X lane, NA-02) | PO-VV2-P-20 |
| `Hash`, `Eq`, `Copy`, `Clone` on `StrawManTrap` | 14 | OF-2 | (out of scope; library semantics) | (out of scope) |
| `Clone`, `Eq`, `Debug` on `StrawManValidation` | 95 | OF-3 | (out of scope; library semantics) | (out of scope) |

**Result**: Every public method has at least one contract clause. Every contract clause has at least one Verus or proptest obligation (or is explicitly out-of-scope for X lane).

### 11.2 Verus spec gaps relative to contract

| Gap | Detail | Severity | Status |
|---|---|---|---|
| **V-02, V-03, V-04 are vacuous** | `ensures true` on `label()`, `description()`, `checkbox_label()`. Verus cannot evaluate `r@.len()` or `r@.last()` on string literals in `const fn` bodies (Verus limitation). The non-emptiness and `?`-ending properties are enforced only by proptest (PO-VV2-P-03, P-04, P-05). | **Known limitation** (per `proofs/straw_man_verus.rs:127, 147, 178` doc-comments) | Vacuity explicitly documented; UI-4, ED-5/6/7 are exercised by proptest. |
| **V-07 is vacuous** | `ensures true` on `has_trap()`. The `<[T]>::contains` is not in vstd; `assume_specification` is the trust boundary. The postcondition `r == self.traps_detected@.contains(trap)` is not provable until vstd adds the spec. | **Known limitation** | F1 fix applied per `proofs/straw_man-review.md §1`: `assume_specification` declaration is in place at `proofs/straw_man_verus.rs:198-199`. UI-5, ED-1 exercised by proptest. |
| **NA-01 (Default)** | `Default::default()` is covered transitively via `passing()`'s spec (V-06). No separate spec. | Acceptable | Forward direction covered; divergence risk caught by proptest P-08 (KNOWN-LIMITATION-2). |

**Verus spec alignment summary**: 3 vacuous postconditions (V-02, V-03, V-04) and 1 trust-boundary postcondition (V-07) are honest mirrors of Verus limitations and external-library trust. **6 of 9 Verus obligations are non-vacuous and verifiable.** The 3 vacuous obligations are not gaps in the contract — they are gaps in Verus's ability to verify string literals in `const fn` bodies. The contract holds; the proof infrastructure is bounded.

### 11.3 proptest gaps relative to contract

| Gap | Detail | Severity | Status |
|---|---|---|---|
| **8 of 21 properties are deterministic single-shot checks** | `prop_all_has_four_elements`, `prop_all_contains_every_variant_once`, `prop_label_is_nonempty`, `prop_description_is_detailed`, `prop_checkbox_label_is_a_question`, `prop_passing_is_passing`, `prop_default_equals_passing`, `prop_new_empty_equals_passing` (all signature `_unused: ()`). They re-check the production `#[cfg(test)] mod tests` block. | **Observation** (F4 in `proofs/straw_man-review.md`) | Acceptable as-is; proptest wrapper provides shrink budget even on deterministic cases. |
| **PO-VV2-P-12 only tests forward direction** | `prop_is_valid_iff_invariant_holds` only checks `new(traps).is_valid() == true`. The reverse direction (a violated invariant causes `is_valid() == false`) requires direct field mutation, not expressible through the public API. | **Observation** (F5) | Reverse direction is unverifiable through public API; can be added to `mod tests` with direct field mutation. |
| **PO-VV2-P-13 is one-direction** | `prop_has_trap_implies_count_positive` only asserts `has_trap ⇒ trap_count >= 1`. Reverse is covered by P-10 and P-11. | Cosmetic (F7) | Acceptable; label is "iff" but body is implication. |
| **PO-VV2-P-20's `is_valid()` assertion is partially redundant** | The final `prop_assert!(back.is_valid())` is true by construction of the round-trip. | Cosmetic (F6) | Acceptable; defensive check on serde's correctness. |

**proptest alignment summary**: 21 properties cover every public method and every clause in §3 (Ubiquitous Invariants), §4 (State-Driven), §5 (Event-Driven), and §6 (Optional Features). The 4 cosmetic gaps are non-blocking observations, not coverage holes.

### 11.4 Contract clauses not yet proven (gaps for proof-planner)

| Clause | Status | Action |
|---|---|---|
| UI-1 (closed enum cardinality) | ✅ V-01 (non-vacuous) | No action. |
| UI-2 (constructor invariant) | ✅ V-05, V-06 (non-vacuous) | No action. |
| UI-3 (canonical order) | ✅ V-01 (non-vacuous) | No action. |
| UI-4 (display strings) | ⚠️ V-02, V-03, V-04 are **vacuous**; covered by proptest only | Either accept Verus limitation or upgrade to Kani/Flux for string-length refinement. Proof-planner decision. |
| UI-5 (bag semantics) | ⚠️ V-07 is **vacuous** (trust-boundary); covered by proptest only | Accept or wait for vstd to add `<[T]>::contains`. |
| UI-6 (serde) | ✅ X lane (NA-02); covered by proptest | No action. |
| UI-7 (Default = passing) | ✅ Transitive via V-06 + proptest P-08 | No action. |
| SD-1, SD-2 (new) | ✅ V-05 + proptest P-06, P-09, P-18 | No action. |
| SD-3 (passing) | ✅ V-06 + proptest P-07 | No action. |
| SD-4 (default) | ✅ V-06 (transitive) + proptest P-08 | No action. |
| SD-5 (predicate table) | ✅ V-09 + proptest P-12 (forward only) | Reverse direction requires direct field mutation; out of public API scope. |
| ED-1 (has_trap) | ⚠️ V-07 vacuous + proptest P-10, P-13, P-14, P-17 | Same as UI-5. |
| ED-2 (trap_count) | ✅ V-08 + proptest P-11, P-15 | No action. |
| ED-3 (is_valid) | ✅ V-09 + proptest P-12 | No action. |
| ED-4 (all) | ✅ V-01 + proptest P-01, P-02 | No action. |
| ED-5 (label) | ⚠️ V-02 vacuous + proptest P-03 | Same as UI-4. |
| ED-6 (description) | ⚠️ V-03 vacuous + proptest P-04 | Same as UI-4. |
| ED-7 (checkbox_label) | ⚠️ V-04 vacuous + proptest P-05 | Same as UI-4. |
| OF-1 (serde round-trip) | ✅ X lane + proptest P-19, P-20 | No action. |
| OF-2, OF-3 (derives) | (out of scope; library semantics) | No action. |
| UB-1 (no invariant-violating API) | ✅ V-05, V-06 + proptest P-21 (master invariant) | No action. |
| UB-2 (display non-empty) | ⚠️ Same as UI-4 | See UI-4. |
| UB-3 (checkbox `?`-ending) | ⚠️ Same as UI-4 | See UI-4. |
| UB-4 (description ≥21 chars) | ⚠️ Same as UI-4 | See UI-4. |
| UB-5 (all() no drift) | ✅ V-01 + proptest P-01, P-02 | No action. |
| UB-6 (observer purity) | ✅ Type system (no `&mut self`) + Verus accepts all observers as non-mutating | No action. |
| KNOWN-LIMITATION-1 (mutation invariant break) | Not provably prevented; detected by `is_valid()` | Document as accepted. |
| KNOWN-LIMITATION-2 (Default/new(vec![]) fork) | Caught by proptest P-08, P-09 | Document as accepted. |

**Net**: 4 of 11 distinct clause clusters are bounded by Verus limitations (string literals in `const fn` and `<[T]>::contains`); all 4 are covered by proptest guardrails. No contract clause is uncovered.

### 11.5 Bead lifecycle status

| Action | Owner | Blocking? |
|---|---|---|
| **Contract ratification** (this document) | `rust-contract` ✅ DONE | Yes — was required for `proof-to-implementation` bridge per `proofs/straw_man-review.md F3`. |
| Update `proofs/straw_man-review.md` clauses from `INFERRED` → `RATIFIED` | `proof-writer` | Yes — closes F3. |
| Update `proofs/straw_man-writeup.md §1` to drop the "no approved plan" caveat | `proof-writer` | Yes — closes F3. |
| Address DN-1 (private `passed`) | `holzman-rust` + `rust-contract` | No — proof mirrors current behavior. |
| Address DN-2 (rename `is_valid`) | `holzman-rust` | No — proof mirrors current name. |
| Address DN-3 (collapse Default/new(vec![])) | `holzman-rust` | No — proptest catches divergence. |
| Re-run Verus and capture exit-0 evidence in `verification-ledger.jsonl` | `formal-verifier` | Yes — F1/F2 from `proofs/straw_man-review.md`. |

---

## §12 — Open Domain Decisions

| Q | Question | Decision | Rationale |
|---|---|---|---|
| **Q1** | Should `description()` enforce a ≥21-character minimum? | **YES — contract invariant (UI-4, ED-6, UB-4).** | Closes writeup §3 contract gap #3. The Verus spec pins it as `ensures r@.len() > 20` (vacuous, but mirrors the test convention at `straw_man.rs:200`). Proptest P-04 enforces at the test layer. |
| **Q2** | Should `passed` be private (structurally enforced invariant) or public (runtime detector)? | **DEFERRED to `holzman-rust` (DN-1).** Current state is `pub`. The Verus spec covers the constructor invariant; the structural gap is documented. | `passed` privacy is a type-system design choice, not a contract decision. The runtime detector (`is_valid`) makes the gap observable but not preventable. |
| **Q3** | Should `is_valid` be renamed to `invariant_holds`? | **DEFERRED to `holzman-rust` (DN-2).** Current name is ambiguous. | Renaming is a breaking API change. The contract documents the current semantics; downstream consumers should not rely on the name alone. |
| **Q4** | Should `Default::default()` and `new(vec![])` collapse to a single canonical path? | **DEFERRED to `holzman-rust` (DN-3).** Current state has two paths. | The fork risk is bounded by proptest P-08, P-09. The contract documents the current equivalence. |
| **Q5** | Is the `traps_detected` field a Set or a Vec (bag)? | **Vec / bag (UI-5).** Duplicates are preserved and counted. | Documented behavior at `straw_man.rs:127-135`. Proptest P-15, P-17, P-18 exercise this. |
| **Q6** | May `with_description` / `with_verification` / `decision_choice` methods be added later? | **N/A — these methods do not exist in the current source and are not in this contract.** | The user's task brief listed these as examples; they are not part of the actual public surface. |

---

## §13 — Downstream Contract Implications

The `StrawManTrap` and `StrawManValidation` types are exported from `clarity-web/src/domain/mod.rs:24`. Any consumer module that:

- Uses `StrawManTrap` as an enum discriminator,
- Stores `StrawManValidation` in a struct field,
- Serializes either type across a JSON boundary,
- Constructs a `StrawManValidation` from external input,

…inherits the invariants of this contract:

1. **4-variant closed enum**: Only four trap variants exist; adding a fifth is a breaking change.
2. **Constructor-enforced invariant**: Any `StrawManValidation` produced via the public API satisfies `passed == traps_detected.is_empty()`.
3. **Bag semantics**: `traps_detected` is a `Vec`, not a `Set`; duplicates are preserved.
4. **Display conventions**: All display strings are non-empty; descriptions are detailed (>20 chars); checkbox labels end with `?`.
5. **No errors**: This module has no `Result<T, E>`; there are no failure paths to handle.
6. **Pure core**: No I/O, time, network, storage, FFI, or `unsafe` — safe to compose with any caller.

---

*End of contract.*
