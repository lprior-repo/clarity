# Parser Verification — Proof Writeup

**Bead:** `cl-54n` — Verification: `intent/parser.rs` (proptest + fuzz)
**Date:** 2026-06-21
**Author:** `proof-writer` agent (Lane **P** primary, Lane **Z** secondary)
**Module under test:** `clarity-web/src/intent/parser.rs` (~570 LOC, 5 public symbols)
**Upstream note:** No `proof-obligations.planned.jsonl` exists in this workspace as of 2026-06-21 (per `verification-ledger.jsonl` row `meta`). Property IDs (`PO-P*`) below are author-assigned locally to this artifact; the canonical obligation ledger is the responsibility of `proof-planner` (see §6).

---

## 1. Module under test — public surface

The five symbols that anchor every property below are listed in
`parser.rs` lines 71, 98, 126, 143, plus the `ParseError` enum at line 25:

| Symbol | Signature | Lines |
|--------|-----------|------:|
| `parse_spec` | `fn(json: &str) -> Result<Spec, ParseError>` | 71–86 |
| `parse_spec_from_value` | `fn(value: &Value) -> Result<Spec, ParseError>` | 98–119 |
| `sanitize_string` | `fn(s: &str) -> String` | 126–132 |
| `validate_spec` | `fn(spec: &Spec) -> Result<(), ParseError>` | 143–155 |
| `ParseError` | enum (4 variants: `JsonError`, `MissingField`, `InvalidType`, `EmptyField`) | 25–49 |

Output type `Spec` lives at `clarity-web/src/intent/types/spec.rs` and is
composed of `Feature`, `Behavior`, `Invariant`, `AntiPattern`, and
`AIHints` from the same subdirectory. All of these derive
`PartialEq + Eq + Serialize + Deserialize`, so round-trip equality is
the strongest assertion the parser lane can make without inventing
language semantics.

---

## 2. Properties written

| ID | Property | Test name | Strategy / input domain | Production API exercised |
|----|----------|-----------|-------------------------|--------------------------|
| **PO-P1** | `Spec → JSON → Spec` round-trip is identity | `proptest_p1_roundtrip` | arbitrary `Spec` (bounded) | `parse_spec` + `serde_json::to_string` |
| **PO-P2** | Re-serializing a parsed Spec yields the same JSON; re-parsing yields the same Spec | `proptest_p2_reserialize_stable` | arbitrary `Spec` (bounded) | `parse_spec` (twice) + `serde_json::to_string` (twice) |
| **PO-P3** | `sanitize_string` is idempotent under composition | `proptest_p3_sanitize_idempotent` | arbitrary `.*` | `sanitize_string` (twice) |
| **PO-P4** | `sanitize_string` output contains no `'\0'` bytes | `proptest_p4_sanitize_strips_nulls` | arbitrary `.*` | `sanitize_string` |
| **PO-P5** | `sanitize_string` output is always trimmed | `proptest_p5_sanitize_trims` | arbitrary `.*` | `sanitize_string` |
| **PO-P6** | `parse_spec` never panics on arbitrary bytes | `proptest_p6_parse_spec_no_panic` | arbitrary `.*` | `parse_spec` |
| **PO-P7a** | Unclosed-object JSON yields `ParseError::JsonError` | `proptest_p7a_malformed_json_is_json_error` | `{prefix}{"no_close_brace` | `parse_spec` |
| **PO-P7b** | Non-object root JSON value yields `ParseError::InvalidType { field: "root", .. }` | `proptest_p7b_non_object_root_is_invalid_type` | `Null / Bool / Number / String / Array` | `parse_spec_from_value` |
| **PO-P7c** | Missing `name` yields `ParseError::MissingField("name")` | `proptest_p7c_missing_name_is_missing_field` | `{"description": ...}` | `parse_spec_from_value` |
| **PO-P7d** | Whitespace-only `name` yields `ParseError::EmptyField("name")` | `proptest_p7d_whitespace_name_is_empty_field` | `{"name": "   "}` | `parse_spec` |
| **PO-P7e** | Numeric `name` yields `ParseError::InvalidType { field: "name", expected: "string", .. }` | `proptest_p7e_wrong_name_type_is_invalid_type` | `{"name": <int>}` | `parse_spec_from_value` |
| **PO-P8** | `parse_spec_from_value` and `parse_spec` agree on JSON-string input | `proptest_p8_from_value_matches_parse_spec` | arbitrary valid `Spec` | both entry points |
| **PO-P9** | A validating Spec stays validating across a round-trip | `proptest_p9_validate_round_trip` | forced valid Spec shape | `parse_spec` + `validate_spec` |
| **PO-P9b** | A failing Spec fails identically (same `ParseError`) across a round-trip | `proptest_p9b_validate_empty_features_preserved` | arbitrary `Spec` name + empty features | `validate_spec` (before & after) |
| **PO-P10** | Every optional defaulted field is preserved through a round-trip | `proptest_p10_field_preservation` | empty Spec | `parse_spec` + `Spec::eq` |

**Total properties:** 10 obligation IDs, **15 distinct test cases** (P7
is split into 5 sub-cases a–e because the four `ParseError` variants
deserve separate coverage; P9 is split into OK and Err branches
because `ParseError: PartialEq` lets us assert error-equality
directly).

**Generators are intentionally bounded** (≤ 4 features, ≤ 3 behaviors
per feature, ≤ 3 invariants, ≤ 3 anti-patterns, ≤ 2 deps per feature,
short strings). The `Spec::validate` cardinality ceilings
(100/100/100/50/20) are runtime-checked there, not by the parser, and
are out of scope for the parser lane. Cranking the bounds up would
multiply shrink time without exercising new parser paths.

---

## 3. Fuzz target written

| ID | Target | Path | Property |
|----|--------|------|----------|
| **PO-Z1** | `parse_dsl` | `proofs/parser_fuzz/fuzz_targets/parse_dsl.rs` | `parse_spec` is panic-free on arbitrary `&[u8]` |

The fuzz target is the same panic-freedom contract as PO-P6, but
exercised under libFuzzer's sanitizer run with truly adversarial
bytes — proptest cannot reproduce pathologically malformed UTF-8,
very large inputs, or structurally adversarial shapes. The harness
calls `parse_spec` (production) directly; there is no local
re-implementation.

Sanitizer coverage (AddressSanitizer + UBSan) is the default for
cargo-fuzz and catches:

- heap-buffer-overflow in `serde_json::from_str` adversarial input
- use-after-free in `extract_string_field` (parser.rs lines 162–179)
- signed integer overflow in the JSON line/column formatters
  (parser.rs lines 76–83)

MemorySanitizer is not enabled (requires C++ stdlib rebuild); it is
a follow-on if leaks appear in CI.

---

## 4. Tooling status & expected commands

### 4.1 proptest — expected command

```text
cargo test -p clarity-web --test parser_proptest -- --nocapture
```

**Status:** `BLOCKED_TOOLING` (wiring only; not a tooling install).

The artifact lives at `proofs/parser_proptest.rs` per the `proof-writer`
skill's allowed-edits boundary. Cargo does **not** automatically pick
up test files outside `clarity-web/tests/`. Two unblock paths exist:

1. **Add a `[[test]]` entry** to `clarity-web/Cargo.toml`:
   ```toml
   [[test]]
   name = "parser_proptest"
   path = "../../proofs/parser_proptest.rs"
   ```
   This is a build-config edit, owned by `holzman-rust` /
   `landing-skill` (not by `proof-writer`, per skill boundary).

2. **Symlink** `clarity-web/tests/parser_proptest.rs` →
   `../../proofs/parser_proptest.rs` and run
   `cargo test -p clarity-web --test parser_proptest`.

3. **`include!`** the artifact from a one-line
   `clarity-web/tests/parser_proptest.rs` driver. This is the lightest
   option and keeps the artifact canonical at `proofs/`.

The artifact file itself is fully self-contained: it `use`s the public
API (`clarity_web::intent::parser::*`, `clarity_web::intent::types::*`),
and `proptest = "1.10.0"` is already declared as a `dev-dependency` in
`clarity-web/Cargo.toml` line 44. No `Cargo.toml` edits are required
by the artifact itself.

**Exact wiring steps for holzman-rust / landing-skill:**

Option A — `[[test]]` entry (recommended):
1. Open `clarity-web/Cargo.toml`
2. Find the `[[test]]` section (or create one if absent under `[package]`)
3. Add:
   ```toml
   [[test]]
   name = "parser_proptest"
   path = "../../proofs/parser_proptest.rs"
   ```
4. Run `cargo test -p clarity-web --test parser_proptest -- --nocapture` to confirm the file resolves

Option B — `include!` driver (lightest):
1. Create `clarity-web/tests/parser_proptest.rs` as a one-line driver:
   ```rust
   include!("../../proofs/parser_proptest.rs");
   ```
2. No Cargo.toml edit needed
3. Run `cargo test -p clarity-web --test parser_proptest -- --nocapture`

Option C — symlink:
1. `ln -s ../../proofs/parser_proptest.rs clarity-web/tests/parser_proptest.rs`
2. Run `cargo test -p clarity-web --test parser_proptest -- --nocapture`

After wiring, `formal-verifier` captures the output as a `task_summary` row in `verification-ledger.jsonl`.

### 4.2 fuzz — expected command (gated on cl-u04)

```text
cargo +nightly fuzz run parse_dsl -- -runs=100000 -max_total_time=60
```

**Status:** `BLOCKED_TOOLING` — `cargo-fuzz` is not installed (bead
`cl-u04`).

Two unblock paths, in dependency order:

1. **Install `cargo-fuzz`** via `cargo install cargo-fuzz`. Requires a
   nightly Rust toolchain (`rustup toolchain install nightly`).
2. **Wire the fuzz crate.** `cargo-fuzz` expects a `fuzz/` directory
   at the workspace root with `Cargo.toml` and `fuzz_targets/`. This
   is owned by `holzman-rust` / `landing-skill`, not by `proof-writer`.

The `proofs/parser_fuzz/` directory contains only the canonical
fuzz-target source. The `fuzz/Cargo.toml` is out of scope for this
bead; recording it as `BLOCKED_TOOLING` (cl-u04) is sufficient.

### 4.3 cargo-fuzz wiring (informational, owned by infra)

```toml
# fuzz/Cargo.toml (NOT in this commit; created when cl-u04 closes)
[package]
name = "clarity-web-fuzz"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
clarity-web = { path = "../clarity-web" }
libfuzzer-sys = "0.4"

[[bin]]
name = "parse_dsl"
path = "fuzz_targets/parse_dsl.rs"
```

---

## 5. Trusted-base ledger entries (preliminary)

Per `proof-writer` skill §6 and `trusted-base-writing-guide.md`, the
following entries belong in `trusted-base-ledger.jsonl` once the
canonical ledger exists. They are recorded here for downstream
consumption:

| Entry kind | Reason | Scope | Impact | Compensating evidence | Owner | Expiry |
|------------|--------|-------|--------|----------------------|-------|--------|
| `assumption` | `serde_json::to_string` and `serde_json::from_str` are total on `Spec`-shaped inputs | PO-P1, PO-P2, PO-P10 | A regression in serde_json would manifest as our test failures | `cargo test --workspace --all-features` baseline | `formal-verifier` | next clippy debt closure |
| `assumption` | `parse_spec` accepts the same JSON shape that `serde_json::to_string(&spec)` produces | PO-P1, PO-P2, PO-P8 | If the parser ever diverges from the serializer, P1/P2 will fail | manual review of `parser.rs:115` `serde_json::from_value` call | `proof-writer` | when production code changes |
| `assumption` | `ParseError` variants `JsonError`, `MissingField`, `InvalidType`, `EmptyField` are exhaustive | PO-P7a–e | A new variant would silently pass | hand-checked against `parser.rs:25-49` | `proof-writer` | when production code changes |
| `model_reduction` | Bounded generator sizes (≤4 features, ≤3 behaviors, etc.) | PO-P1, PO-P2, PO-P8, PO-P10 | `Spec::validate` cardinality ceilings (100/50/20) are out of scope for the parser lane | validation lane has separate coverage in `verification-targets.md §5.3` | `proof-planner` | none (by design) |
| `disabled_check` | MemorySanitizer is not enabled in the fuzz run | PO-Z1 | Leaks would not be caught | AddressSanitizer + UBSan catch heap/stack/UBSan issues; CI will surface `cargo +nightly fuzz` output | infra | if leaks appear in CI |
| `fuzz_budget` | `-runs=100000 -max_total_time=60` is the smoke budget | PO-Z1 | Missed coverage of deep paths on first run | bump to `-runs=1000000` for nightly / pre-merge | infra | reviewed per release |

---

## 6. What this artifact does **not** prove

The proof-writer skill is explicit (§5 — "PENDING_FORMAL_EXECUTION" only after smoke evidence exists) and `verification-targets.md` §1 ("line-by-line proofs refused") define the boundary. Concretely, this artifact does **not** prove:

1. **Behavioral semantics of the DSL.** The properties test the
   parser's *syntactic* surface — round-trip and error
   classification. Whether the parser's notion of `name` matches the
   DSL's actual semantics is a `rust-contract` question, not a
   proof-writer one. That contract does not exist yet (upstream note
   in the bead brief).
2. **Cardinality ceilings enforced by `Spec::validate`.** Those
   limits (100 features, 50 behaviors, 20 deps/pre/postconditions)
   are part of the validation lane, not the parser lane.
   `verification-targets.md §5.3` splits `intent/validation/`
   separately.
3. **Name-shape constraints enforced by the type constructors**
   (`Behavior::new` rejects non-`snake_case`). The parser layer
   bypasses those constructors and accepts any string. The proptest
   generators use `snake_case` only to keep round-trips tractable;
   the property does not assert that the parser rejects other
   shapes. That is also a constructor-level invariant and belongs
   on the Verus lane for `intent/types/behavior.rs`.
4. **Serde-derived invariants** (`#[serde(default)]` behaviour, tag
   collisions). Those are properties of `serde_derive` and `serde_json`,
   not of `clarity-web`.
5. **Performance.** proptest's 256-case default and a 60s fuzz run
   are smoke budgets. They catch panics and obvious mistakes, not
   algorithmic regressions. Performance evidence is `velocity` /
   `holzman-rust` work, gated behind a separate benchmark crate.

---

## 7. Production code touched

**None.** This artifact edits only files under `proofs/` and
`.beads/cl-54n/`. The clippy hotspot on `parser.rs` lines 294, 314,
470 (`unneeded_wildcard_pattern` in test asserts) is explicitly
**not** addressed — that is `holzman-rust` work (per the brief and
per `formal-verification-report.md` §5).

### cl-2vr interaction — workspace lints opt-in

**When `cl-2vr` closes** (holzman-rust adds `[lints] workspace = true`
to `clarity-web/Cargo.toml`), hundreds of new clippy violations will
surface across `clarity-web/src`. The proptest artifact at
`proofs/parser_proptest.rs` uses an explicit `#![allow(...)]` block
(lines 62–76) to keep the artifact compilable under deny-level lints:

```rust
#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value,
  clippy::redundant_closure_for_method_calls
)]
```

**When cl-2vr closes, holzman-rust must verify** that this `#![allow(...)]`
block covers all new lint violations that will surface in
`proofs/parser_proptest.rs` specifically. The current allow list covers:
- `unwrap_used`, `expect_used`, `panic` — the proptest `prop_assert!*`
  macros and test helper code intentionally use these in test assertions
- `float_cmp`, `needless_collect`, `match_same_arms`,
  `option_if_let_else`, `suspicious_else_formatting`,
  `manual_let_else`, `match_wild_err_arm`,
  `match_like_matches_macro`, `needless_pass_by_value`,
  `redundant_closure_for_method_calls` — idiomatic proptest patterns
  that fire clippy pedantic/nursery lints

If `cargo clippy -p clarity-web` with `[lints] workspace = true`
reports additional lint errors in `proofs/parser_proptest.rs` after
cl-2vr lands, those lints must be added to the `#![allow(...)]` block
before the proptest artifact can be merged. The proptest artifact
itself is **not** affected by clippy violations in `clarity-web/src`
production code — it is a separate test binary.

---

## 8. Next dispatch

| Bead / work | Owner agent | Why |
|-------------|-------------|-----|
| Wire `clarity-web/Cargo.toml` `[[test]]` entry (or `include!` shim) so `cargo test --test parser_proptest` resolves | `holzman-rust` + `landing-skill` | Build-config edit; out of `proof-writer` scope. Unblocks the smoke run. |
| Close `cl-u04` (install `cargo-fuzz`, wire `fuzz/Cargo.toml`) | infra | Unblocks Lane Z execution. |
| Run the smoke command and capture PASS/FAIL into `verification-ledger.jsonl` as a `task_summary` row | `formal-verifier` | Closes the obligations once tooling lands. |
| Review the proptest artifact for assertion strength, shrink behavior, generator non-vacuity | `proof-reviewer` | Adversarial gate. |
| Author the canonical `proof-obligations.planned.jsonl` mapping PO-P1…P10 + PO-Z1 to the lanes and IDs in this document | `proof-planner` | Required for the verifier to know what to execute and to align ledger rows. |
| Author the bridge from this artifact to behavior-test obligations and to source-line refs (`parse_spec` line 71, `parse_spec_from_value` line 98, etc.) | `proof-to-implementation` | Per `verification-targets.md` §0, the bridge is owned by `proof-to-implementation`, not `proof-writer`. |

---

## 9. Closure status

This artifact is **ready for review**. It is **not** a closure of the
bead. Closure requires:

1. `proof-reviewer` approval of the artifact.
2. `proof-to-implementation` bridge to source-line refs.
3. Wiring (holzman-rust) + tooling (cl-u04) for execution.
4. `formal-verifier` PASS row in `verification-ledger.jsonl`.

Until (4), the obligations remain open.