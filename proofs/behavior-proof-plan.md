# Proof Plan — `clarity-web/src/intent/types/behavior.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-ooz` |
| **Target** | `clarity-web/src/intent/types/behavior.rs` |
| **Primary lane** | **V** (Verus) — per `verification-targets.md §5.3` ("Verus the algebra") |
| **Secondary lane** | **P** (proptest) — algebraic round-trip on the DSL type surface |
| **Contract status** | **GAP — no `rust-contract` artifact exists for this module.** Clauses below are *inferred from source* and labelled `INFERRED`. The plan is gated on `rust-contract` ratifying (or correcting) these clauses before `proof-writer` runs. |
| **Module LOC** | 236 (production 119, tests 117) |
| **Clippy hotspot** | `uninlined_format_args` × 2 at lines 213, 227 (inside `#[cfg(test)] mod tests` — see `formal-verification-report.md §4` and the `cl-2q6` blocker bead) |
| **Planner** | `proof-planner` |
| **Date** | 2026-06-21 |

---

## 1. Module characterisation

`behavior.rs` is the **type algebra of behaviors** in the intent DSL: one `serde`-bearing
struct, one validated constructor, two builder methods, two appender methods, one
validator, and a private `snake_case` predicate. Zero I/O, zero concurrency, zero
`unsafe` (`#![forbid(unsafe_code)]` at module top, line 5). The two `MAX_*` constants
are the entire numeric surface and they are honest positive integers (20, 20).

| Item | Lines | Kind | Risk surface |
|---|---|---|---|
| `const MAX_PRECONDITIONS: usize = 20` | 12 | Bound constant | Bounded arithmetic |
| `const MAX_POSTCONDITIONS: usize = 20` | 14 | Bound constant | Bounded arithmetic |
| `fn is_valid_behavior_name(name: &str) -> bool` | 17–25 | `snake_case` predicate | Pure char classifier |
| `struct Behavior { name, description, verification, preconditions, postconditions }` | 29–44 | Five-field struct, all serde-derived | None at struct level |
| `Behavior::new(name: String) -> Result<Self, TypeError>` | 54–65 | Validating constructor | Validity predicate `is_valid_behavior_name` |
| `Behavior::with_description(self, desc) -> Self` | 69–74 | Owned builder (consumes self) | Field-replace preserves all others |
| `Behavior::with_verification(self, v) -> Self` | 78–83 | Owned builder (consumes self) | Field-replace preserves all others |
| `Behavior::add_precondition(&mut self, c) -> &mut Self` | 86–89 | Fluent mutating builder | `Vec::push` order-preserving |
| `Behavior::add_postcondition(&mut self, c) -> &mut Self` | 92–95 | Symmetric appender | `Vec::push` order-preserving |
| `Behavior::validate(&self) -> Result<(), TypeError>` | 101–117 | Bound check | Both `>` comparisons against `MAX_*` |
| `#[cfg(test)] mod tests` | 120–235 | 10 hand-written tests | Test-only; not a proof target. Clippy hotspot (lines 213, 227). |

**Algebraic structure** (the headline per `verification-targets.md §5.3`):

- `new : String -> Result<Behavior, TypeError>` is a *partial* function total on the
  refinement `is_valid_behavior_name`. It returns the **canonical empty element**.
- `with_description` / `with_verification` are **monoid-like builders** over disjoint
  fields: they commute and right-most-wins.
- `add_precondition` / `add_postcondition` are **list appenders**: order-preserving, no
  dedup. Two equal strings yield two equal entries.
- `validate` is a **predicate on bounded lists**: `Ok(()) ⇔ ∀ count ∈ {pre, post}, count ≤ MAX`.
- The whole struct is **serde-derived** and round-trip is the public surface contract.

## 2. Contract gap (honest disclosure)

The bead `cl-ooz` has **no upstream `rust-contract` artifact** under
`clarity-web/src/intent/types/contract.md` (or any path matching `**/contract.md` in
the workspace — confirmed by `ls` of `clarity-web/src/intent/types/` which yields 9
`.rs` files and no `contract.md`). All clauses used in this plan are *inferred by
direct reading of the source*, not authored.

**Action required before `proof-writer` runs:** `rust-contract` must ratify or
correct the clauses marked `INFERRED` in §3. If the inferred clauses are wrong,
`proof-writer` will write specs against the wrong contract. The obligations JSONL
sets `requires_contract: true` on every inferred-clause row so this gate is visible.

`rust-contract` should produce (at minimum):

- `contract.md` for `clarity-web/src/intent/types/` with clauses keyed to the
  requirement IDs in §3.
- An explicit decision on whether `MAX_PRECONDITIONS == 20` and
  `MAX_POSTCONDITIONS == 20` are **part of the contract** (likely yes; the
  `TypeError::TooMany{Pre,Post}conditions` constructors at
  `clarity-web/src/intent/types/type_error.rs:59,63` accept `(String, usize, usize)`
  and bake the bound into the error message template).
- An explicit decision on whether `description` and `verification` are
  contractually optional (the source has `#[serde(default)]` on both, which is the
  weak form of optional). The proptest round-trip covers both.

## 3. Requirements & inferred contract clauses

Each row carries a `clause_origin` of either `INFERRED` (this plan) or `AUTHORED`
(rust-contract — currently none). Algebraic-law rows are tagged with their law
identifier (L1–L7) for traceability to §1's algebra map.

| Req ID | Source | Inferred clause | Law | Origin |
|---|---|---|---|---|
| REQ-BH-1 | `is_valid_behavior_name` (lines 17–25) | `is_valid_behavior_name(s) ⇔` first char `is_ascii_lowercase` ∧ all chars ∈ `{lowercase, digit, '_'}` | — | INFERRED |
| REQ-BH-2 | `Behavior::new` (lines 54–65) | `new(s).is_ok() ⇔ is_valid_behavior_name(&s)`; `new(s).unwrap_err() == InvalidBehaviorName(s)` | — | INFERRED |
| REQ-BH-3 | `Behavior::new` body (lines 58–64) | If `new(s) = Ok(b)` then `b.name == s ∧ b.description == "" ∧ b.verification == None ∧ b.preconditions.is_empty() ∧ b.postconditions.is_empty()` | L1 | INFERRED |
| REQ-BH-4 | `add_precondition` body (lines 87–88) | After `add_precondition(c)`, `*self.preconditions == old(*self.preconditions) ++ [c]`; all other fields unchanged; no deduplication | L4, L5 | INFERRED |
| REQ-BH-5 | `add_postcondition` body (lines 93–94) | Symmetric to REQ-BH-4 on `postconditions` | L4, L5 | INFERRED |
| REQ-BH-6 | `with_description` body (lines 70–73) | `result.description == desc ∧ result.{name, verification, preconditions, postconditions} == self.{…}` | L2 | INFERRED |
| REQ-BH-7 | `with_verification` body (lines 79–82) | `result.verification == Some(v) ∧ result.{name, description, preconditions, postconditions} == self.{…}` | L2 | INFERRED |
| REQ-BH-8 | `Behavior::validate` body (lines 102–115) | `validate(&self).is_ok() ⇔ self.preconditions.len() ≤ 20 ∧ self.postconditions.len() ≤ 20`. Failure returns `TooManyPreconditions(name, n, 20)` iff `n > 20` on preconditions, symmetric on postconditions | — | INFERRED |
| REQ-BH-9 | Disjoint-field property of `with_description` × `with_verification` | `b.with_description(d).with_verification(v) == b.with_verification(v).with_description(d)` | L2 | INFERRED |
| REQ-BH-10 | Right-most-wins for `with_description` | `b.with_description(d1).with_description(d2) == b.with_description(d2)`; symmetric for `with_verification` | L3 | INFERRED |
| REQ-BH-11 | Append order on `add_precondition` | `b.add_precondition(x).add_precondition(y).preconditions == [x, y]` (the two writes share `*self`); symmetric | L4 | INFERRED |
| REQ-BH-12 | No dedup on `add_precondition` | `b.add_precondition(s).add_precondition(s).preconditions == [s, s]` (length 2) | L5 | INFERRED |
| REQ-BH-13 | Monotonicity of `validate` w.r.t. appends | If `validate(b).is_ok() ∧ |b.preconditions| < MAX` then `validate(b.add_precondition(s)).is_ok()`; if `|b.preconditions| == MAX` then `validate(b.add_precondition(s)) == Err(TooManyPreconditions(_, MAX+1, MAX))` | L6 | INFERRED |
| REQ-BH-14 | `Serialize` + `Deserialize` derives on `Behavior` (line 28) | `∀ b : Behavior, serde_json::from_str::<Behavior>(&serde_json::to_string(&b).unwrap_or_default()) == Ok(b)` | L7 | INFERRED |

**Open questions for `rust-contract`** (need answers before proof-writer starts):

1. Are `MAX_PRECONDITIONS = 20` and `MAX_POSTCONDITIONS = 20` part of the contract as
   exact numeric literals, or are they implementation details that could change? (The
   `TypeError::TooMany{Pre,Post}conditions` error variant bakes `20` into its message,
   so they are de-facto contract.) If they change, the `validate` spec needs a named
   bound, not a literal.
2. Is the `verify` builder monotonic in `verification` — i.e., is the second
   `with_verification` call supposed to *replace* (current source) or *accumulate*? The
   source replaces (one `Option<Verification>` field, not a list); the spec assumes
   replacement.
3. Is `add_precondition`'s return of `&mut Self` part of the public contract (current
   source), or is it allowed to change to `()` or to consume-and-return without breaking
   callers? The fluent-builder law (REQ-BH-11) depends on this signature.

## 4. Verifier lane decisions

Per `verifier-trigger-matrix.md`, classify the proof seeds across
Verus / Kani / Flux / Loom / proptest / fuzz.

| Lane | Decision | Evidence / rationale |
|---|---|---|
| **V** (Verus) | **REQUIRED — primary** | Module is pure data + 8 pure functions; the algebraic-law list (L1–L6) and the constructor/validator predicates are exactly what Verus specs. Per `verification-targets.md §5.3` ("Verus the algebra"). Verus is installed (`/home/lewis/.local/bin/verus`, v0.2026.05.05). |
| **P** (proptest) | **REQUIRED — secondary** | `Behavior` derives `Serialize`/`Deserialize`; the round-trip property (L7) and the boundary cases at exactly `MAX_PRECONDITIONS`/`MAX_POSTCONDITIONS` are the natural proptest targets. proptest is a workspace `dev-dependency` (`clarity-web/Cargo.toml:44`). |
| **K** (Kani) | **NOT APPLICABLE** | Kani's strength (bounded model check of `unsafe`, fixed-width arithmetic, parser bounds) does not apply. The only arithmetic is `> MAX_PRECONDITIONS` / `> MAX_POSTCONDITIONS` on `Vec::len`, which is trivially provable in Verus. No `#[kani::proof]` harness is warranted. Cite: `#![forbid(unsafe_code)]` (line 5), no parser, no fixed-width numeric types, no index-based access. Kani is not installed per `verification-targets.md §3`; even if applicable, install would be required first. |
| **F** (Flux) | **NOT APPLICABLE** | Flux is the lightweight refinement-type alternative to Verus. Since Verus covers the refinement properties (snake_case name, bounded lists) with more rigour and the same author burden, Flux would be redundant. `cargo-flux` is installed but unused for this module. |
| **L** (Loom) | **NOT APPLICABLE** | No concurrency in this module — no threads, channels, atomics, `Send + Sync` interactions, async, or spawn calls. The `add_precondition(&mut self) -> &mut Self` signature is the *only* non-owned builder, and it cannot race with itself within one thread. |
| **M** (Miri) | **NOT APPLICABLE** | `#![forbid(unsafe_code)]` at module top (line 5) and workspace level (`Cargo.toml` line 10). No `unsafe` blocks anywhere in `behavior.rs`. |
| **T** (TLA+) | **NOT APPLICABLE** | No temporal workflow — no state machine transitions, no retries, no leases, no batch ordering. The module is pure type algebra. A TLA+ spec for `intent/beads` and `intent/batch` is mentioned in `verification-targets.md §5.3` but is out of scope here. |
| **Z** (fuzz) | **NOT APPLICABLE** | `Behavior` constructors are typed Rust functions; the only string input is the behavior name (already validated by `is_valid_behavior_name`). The untrusted-input parser boundary is `intent/parser.rs` (per `verification-targets.md §5.3`: Z+P lane for `intent/parser.rs`). `cargo-fuzz` is not installed; even if it were, no fuzz target exists in this module. |
| **X** (exercise-only) | **NOT APPLICABLE** | The whole module is in scope for V + P coverage. The `#[cfg(test)] mod tests` block is covered by behaviour tests and does not need a separate `X` lane. |

Two infrastructure gaps block adjacent lanes (`K`, `Z`) but those lanes are
`not_applicable` to this module regardless, so the gaps are not blockers for *this*
plan. They are noted for the landing-skill pre-flight in `verification-targets.md §4`.

## 5. Proof coverage matrix

| Req ID | Law | Lane | Obligation ID | Targets | Evidence |
|---|---|---|---|---|---|
| REQ-BH-1 | — | V | PO-V1 | `is_valid_behavior_name` | Verus spec fn `name_valid(s: &str) -> bool` matches the char-classifier shape; case analysis over `chars.next()` and `chars.all(...)`. |
| REQ-BH-2 | — | V | PO-V2 | `Behavior::new` Ok/Err predicate | Verus ensures/requires on `new` linking the `Result` arm to `is_valid_behavior_name`. |
| REQ-BH-3 | L1 | V | PO-V3 | `Behavior::new` Ok-arm fields | Verus proves the canonical-empty element invariant on `description == ""`, `verification == None`, both `Vec`s empty. |
| REQ-BH-4 | L4, L5 | V | PO-V4 | `Behavior::add_precondition` | Verus postcondition: `*self'.preconditions == old(*self.preconditions) ++ [c]`, all other fields unchanged, no dedup. |
| REQ-BH-5 | L4, L5 | V | PO-V5 | `Behavior::add_postcondition` | Symmetric to PO-V4 on `postconditions`. |
| REQ-BH-6 | L2 | V | PO-V6 | `Behavior::with_description` | Verus proves field-replace semantics: `description` updated, other 4 fields copied. |
| REQ-BH-7 | L2 | V | PO-V7 | `Behavior::with_verification` | Symmetric to PO-V6 on `verification`. |
| REQ-BH-8 | — | V | PO-V8 | `Behavior::validate` | Verus proves `Ok(()) ⇔ preconditions.len() ≤ 20 ∧ postconditions.len() ≤ 20` plus the two error-variant mappings. |
| REQ-BH-9 | L2 | V | PO-V9 | `with_description` × `with_verification` commutativity | Verus proves the two composition orders produce equal structs (extensional equality on all 5 fields). |
| REQ-BH-10 | L3 | V | PO-V10 | `with_description` right-most-wins | Verus proves `b.with_description(d1).with_description(d2) == b.with_description(d2)` (symmetric for `with_verification`). |
| REQ-BH-11 | L4 | V | PO-V11 | `add_precondition` order preservation | Verus proves that `b.add_precondition(x).add_precondition(y).preconditions == [x, y]` for any `x, y`. |
| REQ-BH-12 | L5 | V | PO-V12 | `add_precondition` no-dedup | Verus proves that `b.add_precondition(s).add_precondition(s).preconditions == [s, s]` for any `s`. |
| REQ-BH-13 | L6 | V | PO-V13 | `validate` monotonicity | Verus proves the boundary cases: `len < MAX` keeps `Ok(())`; `len == MAX` flips to `Err(TooManyPreconditions(_, MAX+1, MAX))`. |
| REQ-BH-14 | L7 | P | PO-P1 | `Behavior` serde round-trip | proptest over `Behavior` generated by a `proptest::prelude::Strategy` combining `is_valid_behavior_name`-compliant names, arbitrary descriptions, optional `Verification`, and vectors of pre/postconditions up to `MAX + 1` to exercise the boundary in both directions. |

## 6. Trusted base plan

These are the assumptions the proofs lean on. Each is either explicitly trusted or
has its own obligation.

| Trust | Why trusted | Mitigation in obligations |
|---|---|---|
| `serde_json` round-trip preserves values | Library contract; not our code | PO-P1 explicitly exercises the round-trip on `Behavior`; if serde silently corrupted values, the property test fails. |
| `Vec::push` is order-preserving and appends (no dedup) | `std` library contract | PO-V4, V5, V11, V12 prove these properties via the `old(self)` / `*self'` pattern; the spec references the `Vec` API directly. |
| `String` is a total type over arbitrary UTF-8 | Rust stdlib | No mitigation needed; PO-P1 generates arbitrary `String` contents. |
| `Behavior::new(s)` returns `InvalidBehaviorName(s)` exactly (no other arm) | Source inspection | PO-V2 proves the `Err` arm carries the input string unchanged. |
| `Behavior::validate` returns `TooManyPreconditions(name, n, 20)` iff `n > 20` on `preconditions` | Source inspection | PO-V8 proves both the `>` comparison and the error payload shape (including the name clone). |
| `MAX_PRECONDITIONS == MAX_POSTCONDITIONS == 20` are exact literals | Source const declarations | PO-V8 references the literal `20` (or, if `rust-contract` decides, a named bound); PO-V13 uses the same bound. |
| The `Behavior` struct fields are closed at the time the spec is written | Type-system fact | PO-V3, V6, V7 enumerate all 5 fields explicitly; adding a 6th would break the spec at compile time. |

## 7. Waiver candidates

**None.** All in-scope behaviour is provable under the chosen lanes. The
non-applicable lanes (K, F, L, M, T, Z, X) have concrete evidence in §4 and do not
require waivers — they are genuinely not needed.

If `rust-contract` decides that `MAX_PRECONDITIONS` / `MAX_POSTCONDITIONS` are
**implementation details** (not part of the contract), the `validate` spec needs to
be re-expressed as `validate(b).is_ok() ⇔ b.preconditions.len() ≤ validate::MAX`
with a named bound, and PO-V8 / PO-V13 change shape. That is an additive change, not
a waiver of behaviour.

## 8. Bridge input for `proof-to-implementation`

The proof-writer will produce Verus specs and proptest properties. The bridge agent
maps them to:

| Proof claim | Rust source ref | Independent behaviour test |
|---|---|---|
| PO-V1 | `is_valid_behavior_name` (lines 17–25) | New `#[cfg(test)]` test `test_is_valid_behavior_name_classifier` enumerating: `""`, `"_"`, `"1abc"`, `"abc-def"`, `"Abc"`, `"abc_def"`, `"abc123"`, `"a"`. |
| PO-V2 | `Behavior::new` (lines 54–65) | Existing `test_behavior_new_valid` / `test_behavior_new_invalid_*` (lines 141–181) cover the predicate. |
| PO-V3 | `Behavior::new` Ok-arm (lines 58–64) | Extend existing valid-case test with assertions on all 4 optional fields being default. |
| PO-V4 | `Behavior::add_precondition` (lines 86–89) | New test `test_add_precondition_preserves_others` builds a `Behavior`, adds 1 precondition, checks the other 4 fields unchanged. |
| PO-V5 | `Behavior::add_postcondition` (lines 92–95) | Symmetric to PO-V4. |
| PO-V6 | `Behavior::with_description` (lines 69–74) | Existing `test_serde_roundtrip_behavior` (lines 184–207) implicitly checks description preservation; add explicit equality. |
| PO-V7 | `Behavior::with_verification` (lines 78–83) | New test `test_with_verification_preserves_others`. |
| PO-V8 | `Behavior::validate` (lines 101–117) | Existing `test_behavior_validate_too_many_preconditions` / `test_behavior_validate_too_many_postconditions` (lines 209–235) cover the upper boundary; add tests at exactly `MAX = 20` to lock the inclusive boundary. |
| PO-V9 | `with_description` × `with_verification` commutativity | New test `test_builder_commutativity` builds both orderings and asserts equality. |
| PO-V10 | `with_description` right-most-wins | New test `test_builder_right_most_wins` calls `with_description` twice with distinct values and asserts the second wins. |
| PO-V11 | `add_precondition` order preservation | New test `test_add_precondition_order` calls `add_precondition("a").add_precondition("b")` and asserts `preconditions == ["a", "b"]`. |
| PO-V12 | `add_precondition` no-dedup | New test `test_add_precondition_no_dedup` calls `add_precondition("x")` twice and asserts length 2 with both equal. |
| PO-V13 | `validate` monotonicity | New test `test_validate_boundary_inclusive` adds exactly `MAX_PRECONDITIONS` and asserts `Ok(())`, then adds one more and asserts `Err(TooManyPreconditions(_, MAX+1, MAX))`. |
| PO-P1 | `Behavior` serde round-trip | New proptest function in `#[cfg(test)] mod tests` (extend the existing module): `proptest! { #[test] fn proptest_behavior_roundtrip(b in arb_behavior()) { let j = serde_json::to_string(&b).unwrap(); assert_eq!(serde_json::from_str::<Behavior>(&j).unwrap(), b); } }`. The `arb_behavior()` strategy combines: snake_case names via `proptest::string::string_regex("[a-z][a-z0-9_]{0,30}").unwrap()`, arbitrary descriptions via `.*`, optional `Verification`, and `Vec` of pre/postconditions with `prop::collection::vec` capped at `MAX + 1`. |

The proof-writer must **not** modify any production function body. Verus
`#[verifier::external_body]` is the correct tool for the `serde_json` calls that
should not be re-verified.

## 9. Blockers for proof-writer

1. **Contract ratification (BLOCKING).** `rust-contract` must author `contract.md` (or
   equivalent) for `clarity-web/src/intent/types/` and either ratify or correct the 14
   clauses in §3. All 14 `planned` obligations carry `requires_contract: true` to make
   this gate visible (13 Verus + 1 proptest).
2. **Open algebraic decisions (NON-BLOCKING but recommended).** §3 lists three open
   questions on `MAX_*` contract status, `with_verification` semantics, and the
   `&mut Self` builder signature. Defaulting them as in §3 yields the obligation set
   above.
3. **Clippy debt (NON-BLOCKING for proof-writer, BLOCKING for `formal-verifier`).**
   `behavior.rs` has 2 `uninlined_format_args` errors at lines 213, 227 inside
   `#[cfg(test)] mod tests` (see `formal-verification-report.md §4`). These are
   mechanical test-code fixes owned by the `cl-2q6` blocker (`holzman-rust` +
   `landing-skill`). They do not affect the production function bodies that the proofs
   target. `proof-writer` can author Verus specs without clippy being green; however,
   `formal-verifier` cannot run `moon run :ci` honestly until `cl-2q6` is closed.
4. **No tooling gaps for this plan.** Verus is installed; proptest is a workspace
   `dev-dependency`. Kani / cargo-fuzz gaps exist but do not block this module
   (those lanes are `not_applicable` per §4).

## 10. Non-targets (explicit)

Per `verification-targets.md §8`, the following are NOT in this plan:

- **Line-by-line proofs.** Refused; not cost-effective for glue/serde code.
- **The 10 hand-written tests in `#[cfg(test)] mod tests`** (lines 120–235) are
  exercised behaviour, not proof targets. They are referenced by the bridge (§8) but
  not themselves proved.
- **The 2 clippy `uninlined_format_args` errors** at lines 213, 227 — owned by
  `cl-2q6`, not by `cl-ooz`. They are inside the test module and do not change
  production behaviour.
- **Miri, Loom, TLA+, fuzz, Flux, Kani.** All not applicable (§4). No
  `not_applicable` obligation rows for these will be promoted to `waived`.
- **Production `unsafe`** — `forbid` at workspace level (`Cargo.toml` line 10) and
  module level (line 5); no obligation needed.
- **The clippy-debt sites themselves.** Verification does not author `cargo clippy`
  fixes.

## 11. Pre-flight checklist for landing this plan

- [ ] `rust-contract` produces contract clauses for the 14 inferred items in §3.
- [ ] `proof-plan-reviewer` reviews this file and the obligations JSONL.
- [ ] Verus invocation command confirmed: per-module spec files at
      `proofs/verus/behavior_*.rs` invoked via `verus <file>` (or the project's
      `cargo verus` integration — to be confirmed by `formal-verifier`).
- [ ] proptest invocation confirmed: `moon run :test -- --test
      behavior_proptests` (or `cargo test -p clarity-web --lib
      intent::types::behavior::tests::proptest_behavior_roundtrip`); blocked by `cl-2q6`
      clippy gate until that closes.
- [ ] `cl-2q6` clippy gate is independent of this plan (the module's `#[cfg(test)] mod
      tests` uses `clippy::unwrap_used` etc. under `#[allow(...)]` so it does not
      contribute to the lint debt on its own; the 2 `uninlined_format_args` sites are
      the only debt owned by this file).

---

*End of plan.*