# Proof Review — cl-54n (intent/parser.rs)

**Bead:** `cl-54n` — Verification: `intent/parser.rs` (proptest + fuzz)
**Reviewer:** `proof-reviewer` agent
**Date:** 2026-06-21
**Provenance:** distinct agent invocation, no self-approval possible (proof-writer and proof-reviewer are separate roles in the go-skill lifecycle)
**Verdict:** **REJECTED**
**Blocking count:** 4 blockers, 1 high-severity non-blocker, 6 medium/minor, 4 observations
**Recommendation:** see §10

---

## 0. Anti-laundering preflight

Per `proof-reviewer/SKILL.md` §"ANTI-VERIFICATION LAUNDERING MANDATE":

```text
$ rg -n '#\[verifier::external_body\]|assume\(' proofs/parser_proptest.rs proofs/parser_fuzz/fuzz_targets/parse_dsl.rs
(no matches)
$ rg -n '#\[verifier::external_body\]' proofs/parser_proptest.rs proofs/parser_fuzz/fuzz_targets/parse_dsl.rs
(no matches)
$ rg -n '\bassume\(' proofs/parser_proptest.rs proofs/parser_fuzz/fuzz_targets/parse_dsl.rs
(no matches)
```

No `external_body`, no `assume(`. The artifact is not laundering production logic. **Preflight PASS.** Proptest does not have those verifier constructs anyway; this is belt-and-suspenders confirmation.

---

## 1. Property anchor check (Adversarial Q1)

Every property invokes the production API directly. No test shadows production logic locally. Cross-checked against the source modules:

| Property | Production symbol | Source line | Verified |
|----------|-------------------|------------:|:--------:|
| PO-P1 | `parse_spec`, `serde_json::to_string`, `Spec::PartialEq` | parser.rs:71, spec.rs:20 | ✓ |
| PO-P2 | `parse_spec`, `serde_json::to_string` | parser.rs:71 | ✓ |
| PO-P3 | `sanitize_string` | parser.rs:126 | ✓ |
| PO-P4 | `sanitize_string` | parser.rs:127 | ✓ |
| PO-P5 | `sanitize_string` | parser.rs:130 | ✓ |
| PO-P6 | `parse_spec` | parser.rs:71 | ✓ |
| PO-P7a | `parse_spec`, `ParseError::JsonError` | parser.rs:29, 76-83 | ✓ |
| PO-P7b | `parse_spec_from_value`, `ParseError::InvalidType` | parser.rs:100-104, 37-44 | ✓ |
| PO-P7c | `parse_spec_from_value`, `ParseError::MissingField` | parser.rs:33, 107, 166-179 | ✓ |
| PO-P7d | `parse_spec`, `ParseError::EmptyField` | parser.rs:48, 110-112 | ✓ |
| PO-P7e | `parse_spec_from_value`, `ParseError::InvalidType` | parser.rs:37-44, 169-178 | ✓ |
| PO-P8 | `parse_spec`, `parse_spec_from_value` | parser.rs:71, 98 | ✓ |
| PO-P9 | `validate_spec`, `parse_spec` | parser.rs:143-155, 71 | ✓ |
| PO-P9b | `validate_spec`, `parse_spec` | parser.rs:143-155, 71 | ✓ |
| PO-P10 | `parse_spec`, `Spec::PartialEq` | parser.rs:71, spec.rs:20 | ✓ |
| PO-Z1 | `parse_spec` | parser.rs:71 | ✓ |

`use` declarations (lines 82-87) are top-level, not nested behind a module that would shadow names. The `Verification::new(t, String::new())` call at line 165 is verified against verification.rs:26 (`const fn new(verification_type: String, description: String) -> Self` with `example: String::new()` set at line 30).

All public symbols of `parser.rs` are exercised by at least one property. ✓

---

## 2. Round-trip property PO-P1 (Adversarial Q2)

`proptest_p1_roundtrip` (lines 279-296):

```rust
let json = serde_json::to_string(&spec).expect("serialize_spec");
let parsed = parse_spec(&json).expect("parse_spec must succeed for generated valid input");
prop_assert_eq!(parsed, spec);
prop_assert_eq!(parsed.name, spec.name);
```

- **Serializer used:** `serde_json::to_string` (production library, called from the production Cargo dependency at clarity-web/Cargo.toml:10). NOT a test-only serializer. ✓
- **Deserializer:** `parse_spec` (production, parser.rs:71). ✓
- **Equality check:** `prop_assert_eq!(parsed, spec)` exploits `Spec: PartialEq + Eq` (spec.rs:20). ✓

**Caveat (not a blocker):** PO-P1 round-trip identity is *also* a property of `serde_json`, not solely of `parse_spec`. The trust ledger (writeup §5 row 1) records this as an `assumption` ("`serde_json::to_string` and `serde_json::from_str` are total on `Spec`-shaped inputs") — correctly scoped. The assertion would still catch a regression in `parse_spec` if it dropped a field, but would NOT catch a regression in `serde_json` independently.

---

## 3. Panic-freedom PO-P6 (Adversarial Q3)

The writeup claims PO-P6 / PO-Z1 panic-freedom is defensible for `parser.rs`. Verification by source scan:

```text
$ rg -nP '\.unwrap\(\)|\.expect\(|panic!|todo!|unimplemented!' \
       clarity-web/src/intent/parser.rs
14 matches in 1 file:

clarity-web/src/intent/parser.rs:218:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:230:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:252:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:324:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:338:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:373:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:422:let spec = parse_spec(json).expect("spec should parse");
clarity-web/src/intent/parser.rs:431:let spec = parse_spec(json).expect("spec should parse");
clarity-web/src/intent/parser.rs:442:let spec = parse_spec(json).expect("spec should parse");
clarity-web/src/intent/parser.rs:459:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:528:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:545:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:555:let spec = result.expect("spec should parse");
clarity-web/src/intent/parser.rs:566:let spec = result.expect("spec should parse");
```

**All 14 `.expect()` calls live inside `#[cfg(test)] mod tests` (parser.rs:193-569).** Zero panic-prone sites in the production parser code (parser.rs:1-192). The `cl-2vr` "901 panic-prone sites" inventory (formal-verification-report.md §3) is real, but `parser.rs` itself is **clean**.

**Verdict on Q3:** the panic-freedom claim is defensible **scoped to parser.rs production code**. The writeup does not over-claim. The trust ledger does not paper over the cl-2vr finding — it correctly bounds PO-P6 / PO-Z1 to the front door of `parse_spec`.

**Subtle issue (non-blocking):** PO-Z1 fuzz target uses `String::from_utf8_lossy(data)` (parse_dsl.rs:83). Non-UTF-8 bytes become `U+FFFD` before reaching `parse_spec`. This means the fuzz path never tests `serde_json::from_str` on non-UTF-8 input — the lossy decode preempts that layer. PO-P6 proptest uses `".*"` regex, which generates **valid UTF-8** only (the `regex` crate matches by Unicode char). So neither lane covers the case where `serde_json::from_str` is given a `&str` containing literal U+FFFD that bypassed the lossy layer. This is a coverage gap, not a correctness gap: the production `parse_spec` signature is `fn(json: &str)`, so the public API cannot receive non-UTF-8 bytes anyway.

---

## 4. Error classification PO-P7a-e (Adversarial Q4)

The writeup claims 5 sub-properties cover all 4 `ParseError` variants + 1 round-trip Err. Mapping:

| Variant | Sub-case | Test |
|---------|----------|------|
| `JsonError(String)` | P7a | malformed JSON `{"no_close_brace` → JsonError |
| `MissingField(String)` | P7c | missing `name` field |
| `InvalidType { field, expected, actual }` | P7b | root is not an object → `field="root"` |
| `InvalidType { field, expected, actual }` | P7e | `name` is a number → `field="name", expected="string"` |
| `EmptyField(String)` | P7d | whitespace-only `name` |

Variants exhaustively covered: 4/4 ✓
- JsonError → P7a
- MissingField → P7c
- InvalidType → P7b, P7e (2 cases for the same variant, exercising different `field` values)
- EmptyField → P7d

**PO-P9b** adds a 5th coverage point: `validate_spec` returns `EmptyField("features")` and round-trips identically. This exercises the EmptyField variant through a different code path (parser.rs:151 vs parser.rs:111).

**Caveat (subtle, non-blocking):** P7d's input is `format!(r#"{{"name": {:?}}}"#, ws)` with `ws in "[ \\t]{1,5}"`. After `sanitize_string` runs (parser.rs:73), the input is trimmed of leading/trailing whitespace — but the **interior** `{"name": "   "}` keeps its inner spaces. `serde_json::from_str` parses `{"name": "   "}` successfully. Then `parse_spec_from_value` extracts name = `"   "`, trims it to `""`, and emits `EmptyField("name")`. ✓ Correct path.

If `ws` had been `" "` (single space) and `sanitize_string` had run on the **whole input** (not the value), then the JSON `{"name": " "}` would have been transformed... but `sanitize_string` trims the **outer** whitespace, leaving the JSON content intact. Verified correct.

---

## 5. Fuzz target PO-Z1 (Adversarial Q5)

`parse_dsl.rs` (3.6 KB):

- **Calls `parse_spec` directly** (line 65): ✓ no local re-implementation.
- **Takes `&[u8]`**: ✓ real cargo-fuzz signature.
- **Uses `String::from_utf8_lossy`**: a thin byte-to-string shim.
- **No `Arbitrary` impl** for structured DSL input.

**Coverage concern (non-blocking):** Random bytes are mostly invalid JSON. libFuzzer will spend the first ~thousands of iterations on the `serde_json::from_str` Err path before discovering structurally valid JSON shapes. The fuzz harness thus primarily exercises the `serde_json` layer rather than parser.rs-specific logic.

The writeup acknowledges this (parse_dsl.rs:46-51: "We do not seed the corpus"). The panic-freedom property IS still being tested — `serde_json::from_str` errors should not panic, and that is part of the surface the parser relies on. But the parser.rs-specific logic (the `extract_string_field` / `parse_spec_from_value` branches) gets less fuzz coverage than a targeted `Arbitrary` impl would provide.

**No seeds, no `Arbitrary` impl, no structural guidance.** Acceptable for a smoke-budget lane Z but not a defense-in-depth fuzz campaign. The writeup lists this as a `fuzz_budget` entry in trust ledger §5 (100K runs / 60s smoke).

---

## 6. Wiring blocker (Adversarial Q6)

`proofs/parser_proptest.rs` lives **outside** `clarity-web/tests/`. Verified:

```text
$ ls clarity-web/tests/parser_proptest.rs
(no such file)
$ ls clarity-web/tests/ | head
adversarial_gen2_errors.rs
adversarial_gen2_quality.rs
... (19 files, none named parser_proptest)
```

Without wiring (one-line `[[test]]` entry in `clarity-web/Cargo.toml`, or `include!` shim), `cargo test -p clarity-web --test parser_proptest` cannot resolve. The writeup §4.1 documents this as `BLOCKED_TOOLING.wiring`, owned by `holzman-rust`.

**Interaction with `cl-2vr`:** if `holzman-rust` adds `[lints] workspace = true` to `clarity-web/Cargo.toml` (formal-verification-report.md §3), hundreds of new clippy violations surface in `clarity-web/src`. The proptest artifact itself uses `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, ...)]` (lines 62-76) — so the artifact will continue to compile even with deny lints enabled. **No cascading risk to the wiring.** ✓

The wiring blocker is independent of `cl-2vr` and is correct as scoped. Not a content defect in this artifact.

---

## 7. No plan — gate-skip (Adversarial Q7)

`verification-ledger.jsonl` row 18 (`meta`, created_at: 2026-06-21) states explicitly:

> *"No proof-obligations.planned.jsonl, rust-refinement-obligations.jsonl, verifier-lane-decisions.jsonl, trusted-base-ledger.jsonl, or formal-waivers.jsonl exist for this workspace."*

The go-skill lifecycle (`proof-planner/SKILL.md`, `proof-plan-reviewer/SKILL.md`) requires:

```
rust-contract → proof-planner → proof-plan-reviewer → proof-writer
                                                ↓
                                          proof-reviewer ← this agent
                                                ↓
                                  proof-to-implementation
                                                ↓
                              test-planner / test-writer
                                                ↓
                                       formal-verifier
```

cl-54n **skipped `proof-planner` and `proof-plan-reviewer`**. The writeup §6 acknowledges this:

> *"Author the canonical `proof-obligations.planned.jsonl` mapping PO-P1…P10 + PO-Z1 to the lanes and IDs in this document — `proof-planner` — Required for the verifier to know what to execute and to align ledger rows."*

This is a **process gate-skip**, not a content defect. The properties are still reviewable in isolation, but without a plan:

- Property IDs are **author-assigned** (writeup §1) rather than ledger-assigned. PO-P1 has no external meaning.
- The 6 trust-ledger entries in writeup §5 are **not in any `trusted-base-ledger.jsonl`** file. Verified:

  ```text
  $ find /home/lewis/src/clarity -name 'trusted-base*' -type f
  /home/lewis/src/clarity/.beads/cl-kse/trusted-base-ledger.jsonl
  (cl-54n has none)
  ```

  Per the skill ("unledgered trust marker or pending trusted-base disposition" is a Lethal Finding): the trust ledger is **unledgered**. This is a blocker.

---

## 8. Findings (machine-readable)

```jsonl
{"finding/v1": {"id": "PR-CL54N-B1", "artifact": "proofs/parser_proptest.rs + proofs/parser_fuzz/fuzz_targets/parse_dsl.rs", "obligation": "all", "severity": "blocker", "disposition": "owner_approved_debt_pending", "summary": "No execution evidence (PASS/FAIL rows) in verification-ledger.jsonl for cl-54n. Artifact is PENDING_FORMAL_EXECUTION + BLOCKED_TOOLING (wiring + cl-u04).", "fix": "Either close BLOCKED_TOOLING.wiring + BLOCKED_TOOLING.cl-u04 and capture a verifier_execution row, or convert status to formal-waivers.jsonl entry."}}
{"finding/v1": {"id": "PR-CL54N-B2", "artifact": "proofs/parser-writeup.md §5", "obligation": "PO-P1..P10, PO-Z1", "severity": "blocker", "disposition": "owner_approved_debt_pending", "summary": "Trust-ledger entries (3 assumptions, 1 model_reduction, 1 disabled_check, 1 fuzz_budget = 6 rows) are documented in writeup §5 but NOT written to .beads/cl-54n/trusted-base-ledger.jsonl. Per proof-reviewer Lethal Findings: unledgered trust marker.", "fix": "Create .beads/cl-54n/trusted-base-ledger.jsonl with 6 rows matching the schema in cl-kse/trusted-base-ledger.jsonl, then re-submit."}}
{"finding/v1": {"id": "PR-CL54N-B3", "artifact": "proofs/parser-writeup.md §1 + proofs/parser_proptest.rs", "obligation": "all", "severity": "blocker", "disposition": "owner_approved_debt_pending", "summary": "cl-54n dispatched directly to proof-writer without proof-planner plan or proof-plan-reviewer pass. Property IDs (PO-P1..P10, PO-Z1) are author-assigned locally; no canonical proof-obligations.planned.jsonl exists for cl-54n (or for the workspace).", "fix": "Either (a) author proof-obligations.planned.jsonl via proof-planner and route through proof-plan-reviewer, OR (b) accept the gate-skip as owner-approved-debt and document the waiver in formal-waivers.jsonl."}}
{"finding/v1": {"id": "PR-CL54N-B4", "artifact": "proofs/parser_proptest.rs", "obligation": "all", "severity": "blocker", "disposition": "owner_approved_debt_pending", "summary": "Proptest artifact is not wired into clarity-web/tests/. cargo test -p clarity-web --test parser_proptest cannot resolve the file. BLOCKED_TOOLING.wiring per writeup §4.1.", "fix": "Holzman-rust adds [[test]] entry to clarity-web/Cargo.toml pointing at proofs/parser_proptest.rs, OR include! shim from a one-line driver. Then formal-verifier captures a task_summary PASS row."}}
{"finding/v1": {"id": "PR-CL54N-H1", "artifact": "proofs/parser_proptest.rs::proptest_p10_field_preservation", "obligation": "PO-P10", "severity": "high", "disposition": "owner_approved_debt", "summary": "PO-P10 uses .len() comparison for Vec fields (features/invariants/anti_patterns) rather than full content equality. Combined with bounded generators (MAX_FEATURES=4, MAX_INVARIANTS=3), this only proves length preservation at the bounded sizes, not content preservation across the full Validators range. PO-P1 round-trip already covers content preservation end-to-end, so PO-P10 is partially redundant.", "fix": "Either tighten assertions to prop_assert_eq!(parsed.features, original.features) (full content), or narrow PO-P10's scope to length-only and document the redundancy with PO-P1."}}
{"finding/v1": {"id": "PR-CL54N-M1", "artifact": "proofs/parser_fuzz/fuzz_targets/parse_dsl.rs", "obligation": "PO-Z1", "severity": "medium", "disposition": "owner_approved_debt", "summary": "No corpus seeding and no Arbitrary impl. Random bytes spend ~first-thousand iterations on serde_json Err path before reaching parser.rs-specific logic. PO-Z1's panic-freedom is correct but coverage is weaker than targeted Arbitrary would be.", "fix": "Add a small JSON-shaped seed corpus (e.g. {\"name\":\"x\"}, {\"name\":\"x\",\"features\":[{}]} ) to proofs/parser_fuzz/fuzz_corpus/ and re-budget to -runs=1000000 for nightly."}}
{"finding/v1": {"id": "PR-CL54N-M2", "artifact": "proofs/parser_proptest.rs::arb_behavior (line 152-170)", "obligation": "PO-P1, PO-P8", "severity": "medium", "disposition": "owner_approved_debt", "summary": "Behavior generator always uses Verification::new(t, \"\"), leaving the example field permanently String::new(). Round-trip still holds (PO-P1 catches any divergence) but the generator does not exercise non-empty Verification::example values.", "fix": "Generator should proptest::option::of(Verification { verification_type, description, example }) — full struct construction with example as an arb_short_string()."}}
{"finding/v1": {"id": "PR-CL54N-M3", "artifact": "proofs/parser_proptest.rs::proptest_p6_parse_spec_no_panic (line 374-378)", "obligation": "PO-P6", "severity": "medium", "disposition": "owner_approved_debt", "summary": "PO-P6 uses .* regex (valid UTF-8 only); PO-Z1 uses from_utf8_lossy (replaces invalid bytes with U+FFFD). Neither lane exercises serde_json::from_str on a &str containing literal U+FFFD that bypassed the lossy layer. Coverage gap, not correctness gap.", "fix": "Add a proptest arm that generates strings containing U+FFFD explicitly, or document this as a model_reduction in the trust ledger."}}
{"finding/v1": {"id": "PR-CL54N-M4", "artifact": "proofs/parser_proptest.rs (bounded generators)", "obligation": "PO-P1, PO-P2, PO-P8, PO-P10", "severity": "medium", "disposition": "owner_approved_debt", "summary": "Generators are bounded (MAX_FEATURES=4, MAX_BEHAVIORS_PER_FEATURE=3, MAX_INVARIANTS=3, MAX_ANTI_PATTERNS=3, MAX_DEPS_PER_FEATURE=2). Serde round-trip is independent of Spec::validate cardinality ceilings (100/50/20). The bounds avoid coverage of large Spec shapes that serde_json must serialize/parse identically.", "fix": "Document the bound as model_reduction in trust ledger (already done in writeup §5 row 4). Consider adding a second proptest arm with full-size generators (100 features) to confirm round-trip holds at validate-capacities."}}
{"finding/v1": {"id": "PR-CL54N-M5", "artifact": "proofs/parser_proptest.rs::proptest_p2_reserialize_stable (line 310-319)", "obligation": "PO-P2", "severity": "minor", "disposition": "owner_approved_no_action", "summary": "PO-P2 asserts json1==json2 and parsed1==parsed2. Both equalities are properties of serde_json (deterministic Serialize) more than parser.rs. The property is meaningful (catches hidden mutable state in parser.rs) but primarily tests serde_json.", "fix": "No fix needed; this is correctly scoped in writeup §6.4 ('Serde-derived invariants')."}}
{"finding/v1": {"id": "PR-CL54N-M6", "artifact": "proofs/parser_proptest.rs::proptest_p8_from_value_matches_parse_spec (line 474-484)", "obligation": "PO-P8", "severity": "minor", "disposition": "owner_approved_no_action", "summary": "PO-P8 has two assertions: (a) from_value == from_string, and (b) from_value == spec. (b) implies (a) given (b) holds for the original spec. Redundant but harmless.", "fix": "Drop (a) or document the intent."}}
{"finding/v1": {"id": "PR-CL54N-O1", "artifact": "clarity-web/src/intent/parser.rs lines 294, 314, 470", "obligation": "n/a", "severity": "observation", "disposition": "owner_approved_debt", "summary": "3 unneeded_wildcard_pattern clippy lints in test asserts (cl-2q6 baseline). Not in scope for proof-writer (per writeup §7); owned by holzman-rust.", "fix": "Holzman-rust edits the three asserts from `ParseError::InvalidType { field, expected: _, .. }` to `ParseError::InvalidType { field, .. }` (the `..` already covers `expected`)."}}
{"finding/v1": {"id": "PR-CL54N-O2", "artifact": "proofs/parser_proptest.rs (overall)", "obligation": "n/a", "severity": "observation", "disposition": "owner_approved_no_action", "summary": "All 16 test artifacts (15 proptest + 1 fuzz) use explicit #![allow(clippy::unwrap_used, ...)] for the test body. This is the standard proptest pattern (test code unwraps on assertion failure) and matches the convention in parser.rs:193-209."}}
{"finding/v1": {"id": "PR-CL54N-O3", "artifact": "proofs/parser_proptest.rs (Behavior generator)", "obligation": "n/a", "severity": "observation", "disposition": "owner_approved_no_action", "summary": "arb_behavior_name uses [a-z][a-z0-9_]{0,15} (snake_case). The parser layer accepts any string for behavior.name; the constructor Behavior::new enforces snake_case. Generator biases toward snake_case shapes; non-snake_case inputs reach the parser through JSON but not through proptest generators. This is correctly documented in writeup §6.3 as out-of-scope."}}
{"finding/v1": {"id": "PR-CL54N-O4", "artifact": "proofs/parser_proptest.rs::proptest_p7b", "obligation": "PO-P7b", "severity": "observation", "disposition": "owner_approved_no_action", "summary": "P7b uses prop_oneof![Just(...), ...] which doesn't shrink via proptest's default shrinker (Just has no shrinker). If a counterexample is found, shrink won't reduce it. Property is structural so this is acceptable."}}
```

**Counts:**
- Blocker: 4 (B1, B2, B3, B4)
- High: 1 (H1)
- Medium: 4 (M1, M2, M3, M4)
- Minor: 2 (M5, M6)
- Observation: 4 (O1, O2, O3, O4)
- **Total: 15 findings**

---

## 9. Lethal-finding sweep

| Lethal pattern | Found? |
|----------------|:------:|
| `#[verifier::external_body]` mocking production | NO |
| `assume(...)` removing bad inputs | NO |
| `cover!` used as proof | NO |
| `assert(true)` or tautological assertion | NO |
| Hardcoded structural inputs | NO |
| Disconnected Verus spec / proof in `requires` | N/A (no Verus spec) |
| Loom model mismatching production | N/A (no Loom) |
| Flux broad `trusted`/`ignore` | N/A (no Flux) |
| Proof artifact with merge-conflict markers | NO |
| Missing command evidence | **YES (B1)** — execution evidence absent |
| Nonexistent file refs | NO (all referenced files exist) |
| Stale rejected review status | NO |
| Unledgered trust marker | **YES (B2)** |
| Pending trusted-base disposition | **YES (B2)** |

Two lethal findings surface, both process/execution (B1, B2). No content-level lethal findings.

---

## 10. Recommendation

**STATUS: REJECTED.**

The artifact's **content** is sound:
- Every property is anchored to production code with verified line refs.
- No `external_body`, no `assume`, no shadowing, no vacuous assertions.
- Panic-freedom claim is correctly scoped to parser.rs (which is clean).
- Error classification covers all 4 `ParseError` variants + the round-trip Err.
- Fuzz target calls `parse_spec` directly.
- 14 high/medium/minor findings are addressable without re-architecting.

But the artifact's **process gates** are unmet:
1. **B1** — no execution evidence (zero rows in verification-ledger.jsonl for cl-54n).
2. **B2** — trust ledger is documented but unfiled; cl-54n has no `trusted-base-ledger.jsonl`.
3. **B3** — gate-skip on proof-planner / proof-plan-reviewer; PO-P* IDs are author-assigned, not ledger-assigned.
4. **B4** — proptest cannot execute without wiring (`BLOCKED_TOOLING.wiring`).

**Unblock path:**
- (in parallel) Holzman-rust: add `[[test]]` entry to `clarity-web/Cargo.toml` so `cargo test --test parser_proptest` resolves. Capture `cargo test` output as a `task_summary` row.
- (after wiring) Proof-writer: write `proofs/parser-trusted-base.jsonl` (or `.beads/cl-54n/trusted-base-ledger.jsonl`) with the 6 rows from writeup §5, following the schema in `.beads/cl-kse/trusted-base-ledger.jsonl`.
- (optional, recommended) Proof-planner: author `proof-obligations.planned.jsonl` for cl-54n mapping PO-P1..P10 + PO-Z1 to the canonical schema. Route through proof-plan-reviewer.
- (infra) Close `cl-u04` for fuzz execution; capture `cargo +nightly fuzz run parse_dsl` output.

Once B1-B4 close, re-submit for re-review. The content review is substantively complete; only the process scaffolding is missing.

**Verdict:** REJECTED
**Blocking count:** 4 (B1-B4)
**Non-blocking count:** 11 (H1 + M1-M6 + O1-O4)
**Recommendation:** unblock B1-B4 and re-submit.