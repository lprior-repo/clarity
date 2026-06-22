# Type Contract — `clarity-web/src/intent/types/behavior.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-ooz` |
| **Module** | `clarity-web/src/intent/types/behavior.rs` |
| **Artifact version** | `contract/v1` |
| **Author** | `rust-contract` |
| **Date** | 2026-06-21 |
| **Supersedes** | All 14 `INFERRED` clauses in `proofs/behavior-proof-plan.md §3` |

---

## 1. Ubiquitous Language

| Term | Type | Definition |
|---|---|---|
| `Behavior` | Struct | A named, verifiable unit of intent; the primitive of the DSL type algebra |
| `Behavior::new(name)` | Constructor | Total-on-valid-names partial function; produces the canonical empty element |
| `canonical-empty element` | Law L1 | `Behavior::new(s)` when it succeeds yields `description == ""`, `verification == None`, `preconditions == []`, `postconditions == []` |
| `snake_case name` | Predicate | `is_ascii_lowercase(first_char) ∧ all(subsequent_chars ∈ {lowercase, digit, '_'})` |
| `MAX_PRECONDITIONS` | Contract constant | Exact numeric bound = **20**; part of the public contract |
| `MAX_POSTCONDITIONS` | Contract constant | Exact numeric bound = **20**; part of the public contract |
| `with_description` | Builder | Owned (consumes `self`); replaces `description` only; must-use disabled intentionally (not annotated `#[must_use]`) |
| `with_verification` | Builder | Owned (consumes `self`); replaces `verification` only; sets to `Some(v)` |
| `add_precondition` | Builder | Mutable fluent (`&mut self -> &mut self`); appends one `String`; **public contract** |
| `add_postcondition` | Builder | Symmetric to `add_precondition` |
| `validate` | Predicate | `Ok(())` ⇔ `preconditions.len() ≤ 20 ∧ postconditions.len() ≤ 20` |
| `Behavior` serde surface | Law L7 | `Serialize + Deserialize` derived; round-trip is a structural isomorphism |

---

## 2. Algebraic Structure

The module forms a **monoid-like type algebra** with two layers:

### Layer 1 — Field-replace monoid (disjoint fields)

`with_description` and `with_verification` are **commutative field-replacers**:

- They touch disjoint fields (`description` vs `verification`).
- Two calls to the same builder compose with **right-most-wins** (second explicit field shadows first).
- `b.with_description(d1).with_verification(v) == b.with_verification(v).with_description(d1)` — commutative.

### Layer 2 — List append monoid (preconditions / postconditions)

`add_precondition` and `add_postcondition` are **order-preserving appenders**:

- They append in call order; no deduplication.
- The `&mut Self -> &mut Self` signature is **part of the public contract** — callers rely on fluent chaining.
- `Vec::push` is the stdlib contract; no dedup logic exists in the source.

### Boundary — validate predicate

`validate` is a **monotone predicate**: adding a precondition keeps `Ok(())` while `len < MAX`; flipping to `Err` exactly when `len == MAX + 1`.

---

## 3. Contract Clauses (AUTHORED)

Each clause is keyed to its requirement ID and obligation ID from the proof plan.

### REQ-BH-1 / PO-V1

**Clause:** `is_valid_behavior_name(s: &str) -> bool`

`is_valid_behavior_name(s)` returns `true` **iff**:
- `s.chars().next()` is `Some(first)` and `first.is_ascii_lowercase()`, **and**
- `s.chars().skip(1).all(|ch| ch.is_ascii_lowercase() ∨ ch.is_ascii_digit() ∨ ch == '_')`

**Invariants:**
- The predicate is **total**: defined for all `&str` including `""`.
- A `""` name fails (no first character).

**Proof seed lane:** Verus — case analysis over `chars.next()` plus `chars.all(...)`.

---

### REQ-BH-2 / PO-V2

**Clause:** `Behavior::new(name: String) -> Result<Self, TypeError>`

- `Behavior::new(s).is_ok() ⇔ is_valid_behavior_name(&s)`
- When `is_err()`: `err == TypeError::InvalidBehaviorName(s)` exactly (the sole `Err` arm).
- No other `TypeError` variant is reachable from `new`.

**Proof seed lane:** Verus — `ensures`/`requires` linking `Result` arm to `is_valid_behavior_name`.

---

### REQ-BH-3 / PO-V3 (L1 — Canonical Empty Element)

**Clause:** If `Behavior::new(s) = Ok(b)` then:
- `b.name == s`
- `b.description == ""`
- `b.verification == None`
- `b.preconditions == []`
- `b.postconditions == []`

This is the **identity element** for the builder algebra: composing any `Behavior` with the result of `new` and then applying field-replacers yields the field-replacer result.

**Proof seed lane:** Verus — struct literal equality proof.

---

### REQ-BH-4 / PO-V4 (L4 — Append Order)

**Clause:** After `b.add_precondition(c)`:
- `b.preconditions == old(b.preconditions) ++ [c]` (exactly one element appended)
- All other fields (`name`, `description`, `verification`, `postconditions`) are **byte-equal** to `old(...)`.
- No deduplication occurs; `Vec::len` increases by exactly 1.

**Note on `&mut Self`:** `add_precondition(&mut self, c) -> &mut self` is part of the **public contract** (public visibility, no `#[doc(hidden)]`). The fluent return enables call-site chaining like `b.add_precondition("x").add_precondition("y")`.

**Proof seed lane:** Verus — `old(self.preconditions) ++ [c]` postcondition via `old`/`*self'` pattern.

---

### REQ-BH-5 / PO-V5 (L4 — Symmetric Append)

**Clause:** Symmetric to REQ-BH-4 for `add_postcondition` on `postconditions`.

**Proof seed lane:** Verus.

---

### REQ-BH-6 / PO-V6 (L2 — Field Replace)

**Clause:** `b.with_description(d)` returns `Self` where:
- `result.description == d`
- `result.name == old(b.name)`
- `result.verification == old(b.verification)`
- `result.preconditions == old(b.preconditions)`
- `result.postconditions == old(b.postconditions)`

**Proof seed lane:** Verus — field-by-field copy proof.

---

### REQ-BH-7 / PO-V7 (L2 — Field Replace, Verification)

**Clause:** `b.with_verification(v)` returns `Self` where:
- `result.verification == Some(v)`
- All other fields copied from `old(b)`.

**Proof seed lane:** Verus.

---

### REQ-BH-8 / PO-V8 (validate — Bounded Predicate)

**Clause:** `validate(&self) -> Result<(), TypeError>`

`validate(&self).is_ok() ⇔`
- `self.preconditions.len() ≤ 20` **and**
- `self.postconditions.len() ≤ 20`

**Error cases:**
- If `self.preconditions.len() > 20`: `Err(TypeError::TooManyPreconditions(name, n, 20))` where `n = self.preconditions.len()`.
- If `self.postconditions.len() > 20`: `Err(TypeError::TooManyPostconditions(name, n, 20))` where `n = self.postconditions.len()`.
- The `name` in the error is `self.name.clone()`.

**On `MAX_*` as contract constants:** `MAX_PRECONDITIONS = 20` and `MAX_POSTCONDITIONS = 20` are **contractual invariants**, not implementation details. They are observable via `TypeError` payloads and the `#[error(...maximum {2}...)]` format string. Changing these constants requires a breaking change to the error surface. The proof specs should use a named bound (`validate::MAX` or equivalent) rather than the literal `20`, so that spec and source stay in sync if the bound changes.

**Proof seed lane:** Verus — two `>` comparisons and error-payload structural equality.

---

### REQ-BH-9 / PO-V9 (L2 — Commutativity)

**Clause:** For any `b`, `d`, `v`:
```
b.with_description(d).with_verification(v) == b.with_verification(v).with_description(d)
```
Equality is extensional across all 5 fields.

**Proof seed lane:** Verus — extensional equality proof on all 5 fields.

---

### REQ-BH-10 / PO-V10 (L3 — Right-Most-Wins)

**Clause:**
```
b.with_description(d1).with_description(d2) == b.with_description(d2)
b.with_verification(v1).with_verification(v2) == b.with_verification(v2)
```
Equality is extensional across all 5 fields.

**Proof seed lane:** Verus — idempotence proof via struct expression semantics.

---

### REQ-BH-11 / PO-V11 (L4 — Append Order via Fluent Chain)

**Clause:** For any `b`, `x`, `y`:
```
b.add_precondition(x).add_precondition(y).preconditions == [x, y]
```

**Proof seed lane:** Verus — fluent chaining proof via `old`/`*self'` pattern with two sequential pushes.

---

### REQ-BH-12 / PO-V12 (L5 — No Deduplication)

**Clause:** For any `b`, `s`:
```
b.add_precondition(s).add_precondition(s).preconditions.len() == 2
```
Both entries compare equal to `s`.

**Proof seed lane:** Verus — absence of dedup proven by source inspection (no `contains`, no `if` guard on the push).

---

### REQ-BH-13 / PO-V13 (L6 — Validate Monotonicity)

**Clause:**
1. If `b.preconditions.len() < 20 ∧ validate(b).is_ok()` then `validate(b.add_precondition(s)).is_ok()`.
2. If `b.preconditions.len() == 20` then `validate(b.add_precondition(s)) == Err(TooManyPreconditions(name, 21, 20))`.
3. Symmetric for `postconditions`.

**Proof seed lane:** Verus — boundary case proof combining PO-V4 (strict `len += 1`) and PO-V8 (inclusive bound).

---

### REQ-BH-14 / PO-P1 (L7 — Serde Round-Trip)

**Clause:** For all `b: Behavior`:
```
serde_json::from_str::<Behavior>(&serde_json::to_string(&b).unwrap()) == Ok(b)
```
- All 5 fields (`name: String`, `description: String`, `verification: Option<Verification>`, `preconditions: Vec<String>`, `postconditions: Vec<String>`) are serde-compatible.
- `Verification` (from `verification.rs`) also derives `Serialize + Deserialize` and contains only `String` fields.
- The round-trip is a **structural isomorphism** — no information is lost or transformed.

**Proof seed lane:** proptest — strategy covering snake_case names, arbitrary descriptions, optional `Verification`, and vectors of pre/postconditions with lengths in `[0, MAX+1]` to exercise both valid and boundary-rejected constructions.

---

## 4. Error Taxonomy

| Error variant | Origin | Payload |
|---|---|---|
| `TypeError::InvalidBehaviorName(String)` | `Behavior::new` | The rejected name |
| `TypeError::TooManyPreconditions(String, usize, usize)` | `Behavior::validate` | `(name, actual_count, MAX=20)` |
| `TypeError::TooManyPostconditions(String, usize, usize)` | `Behavior::validate` | `(name, actual_count, MAX=20)` |

All `TypeError` variants in this module are **deterministic** — same input always produces same error. No probabilistic or timing-variant errors.

---

## 5. Boundary Map

```
┌─────────────────────────────────────────────────────────────────┐
│  IMPERATIVE SHELL (serde boundaries)                            │
│  Input:  JSON → serde_json::from_str → Behavior                 │
│  Output: Behavior → serde_json::to_string → JSON                │
│  Trust basis: serde library contract; exercised by PO-P1        │
├─────────────────────────────────────────────────────────────────┤
│  PURE CORE                                                      │
│  Behavior::new → is_valid_behavior_name (pure char classifier)  │
│  Behavior::with_description / with_verification (owned builders)│
│  Behavior::add_precondition / add_postcondition (fluent append) │
│  Behavior::validate (pure predicate on bounded Vec lengths)      │
│  #![forbid(unsafe_code)] — no unsafe in this module             │
└─────────────────────────────────────────────────────────────────┘
```

No async, no concurrency, no storage, no network, no FFI, no time boundaries in this module.

---

## 6. Hazard Analysis

| Hazard | Category | Present? | Mitigation |
|---|---|---|---|
| Bounded arithmetic overflow | `MAX_*` arithmetic | No | `MAX_*` are `usize` compile-time constants; `Vec::len` returns `usize` with no overflow risk at ≤20 elements |
| String injection in error messages | Hostile input | No | `is_valid_behavior_name` rejects non-snake_case; error messages embed the raw name via `{}` formatter, not interpolation |
| Deserialization corruption | Serde boundary | No | L7 round-trip proven; `#[serde(default)]` on optional fields preserves absent fields as `None`/empty |
| Temporal: Behavior lifecycle | Temporal workflow | No | `Behavior` is constructed and consumed in one call; no retained state within the module |
| Concurrency: shared-mutable access | Concurrency | No | `#![forbid(unsafe_code)]`; no `Send+Sync` concerns; `&mut self` is single-threaded by construction |
| Fluent builder aliasing | Rust lifetime | Mitigated | `add_precondition(&mut self) -> &mut Self` is self-aliasing; the `old`/`*self'` pattern in Verus proofs tracks aliasing precisely |

---

## 7. Ownership and Mutation Policy

| Method | Consumes self? | Returns `&mut Self`? | Notes |
|---|---|---|---|
| `Behavior::new` | N/A (constructor) | No | Partial function total on valid names |
| `with_description` | **Yes** (owned) | No | Move semantics; `#[must_use]` deliberately absent |
| `with_verification` | **Yes** (owned) | No | Move semantics |
| `add_precondition` | **No** (`&mut self`) | **Yes** | Fluent; **public contract**; `#[must_use]` deliberately absent |
| `add_postcondition` | **No** (`&mut self`) | **Yes** | Fluent; symmetric |
| `validate` | **No** (`&self`) | No | Read-only predicate |

---

## 8. Type-System Invariants (Closed Set)

At time of contract writing, `Behavior` has exactly **5 fields**. The contract clauses PO-V3, V6, V7 enumerate them explicitly. Adding a 6th field will break the Verus specs at compile time (the struct literal patterns in the specs would become non-exhaustive), which is the correct enforcement mechanism.

---

*End of contract. Proof seeds emitted separately in `proof-seeds.jsonl`.*
