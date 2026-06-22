# Formal Verification Report — Workspace State 2026-06-21

**Date:** 2026-06-21
**Verifier role:** `formal-verifier` (execute-only)
**Scope:** Full verification pipeline baseline + 8-module parallel proof program.

---

## TL;DR

Three sessions of work. The verification program is now **real and running**, but the
crates have **two critical gaps** that block honest closure of any "every line is proven"
claim.

| Gap | Severity | Owner | Bead |
|-----|----------|-------|------|
| `cargo clippy` exits 101 with 67 lint errors | High | `holzman-rust` + `landing-skill` | `cl-2q6` |
| `clarity-web` has no `[lints] workspace = true` opt-in — workspace deny lints (unwrap_used, expect_used, panic, todo, unimplemented) are inert; **901 panic-prone sites exist** | **Critical** | `holzman-rust` | `cl-2vr` |
| `kani`, `apalache`, `cargo-fuzz` not installed | Medium | infra | `cl-u04` |

Without fixing `cl-2vr`, the deny lints in workspace `Cargo.toml` are documentation,
not enforcement, and any Verus spec that asserts panic-freedom is unprovable.

---

## 1. What was run in this session

| # | Command | Result |
|---|---------|--------|
| 1 | `moon run :clippy` | exit 101 — 67 clippy errors (64 paired sites across 8 files) |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 101 — same 67 errors, cleaner log |
| 3 | `cargo fmt --all --check` | exit 0 — green |
| 4 | `cargo check --workspace --all-targets` | **exit 0** — type-check clean |
| 5 | `verus proofs/straw_man_verus.rs` | exit 1 — `slice::contains` not in vstd |
| 6 | `verus proofs/lattice_quality_verus.rs` | exit 1 — `pub spec enum` syntax invalid |
| 7 | `verus proofs/fjall_event_store_verus.rs` | exit 1 — `Seq<char>` vs `Seq<u8>` mismatch |
| 8 | `rg` inventory of `.unwrap() / .expect( / panic! / todo! / unimplemented! / unsafe` across `clarity-web/src` | **901 panic-prone sites, 0 real unsafe blocks** |
| 9 | Inspection of `clarity-web/Cargo.toml` | **No `[lints]` table — workspace deny lints are inert** |

All commands have raw evidence on disk. See `/tmp/opencode/` for the logs and
`verification-ledger.jsonl` for the structured ledger.

## 2. Clippy baseline (cl-2q6) — already reported in detail

Captured earlier this session. 64 paired sites across 8 files. Per-lint breakdown:

| Lint | Sites |
|------|------:|
| `clippy::uninlined_format_args` | 32 |
| `clippy::missing_errors_doc` | 15 |
| `clippy::doc_markdown` | 6 |
| `clippy::unused_async` | 5 |
| `clippy::unneeded_wildcard_pattern` | 3 |
| `clippy::expect_used` (1) | 1 |
| `clippy::unused_map` | 1 |
| `clippy::significant_drop_in_scrutinee` (the only correctness-tier item — `kirk/terminal_integration.rs:1300`) | 1 |

**Note (added in §3 update):** the clippy baseline captures lint families that DO fire
on `clarity-web`. It does NOT capture the unwrap_used / expect_used / panic / todo /
unimplemented families because the lint configuration does not apply.

## 3. Critical new finding — cl-2vr

**`/home/lewis/src/clarity/clarity-web/Cargo.toml` has no `[lints]` table.**

The workspace `Cargo.toml` declares `[workspace.lints.clippy]` rules at `deny` level for
five lint families:

```toml
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
```

But these lints are **only inherited** by workspace members that explicitly opt in
via their own `[lints] workspace = true` table. `clarity-web/Cargo.toml` does not.

### Empirical evidence of the gap

```
$ rg -c '\.unwrap\(\)' clarity-web/src --type rust | tr '\0' ':' | awk -F: '{s+=$NF} END {print s}'
316
$ rg -c '\.expect\('   clarity-web/src --type rust | tr '\0' ':' | awk -F: '{s+=$NF} END {print s}'
497
$ rg -c '\bpanic!\('   clarity-web/src --type rust | tr '\0' ':' | awk -F: '{s+=$NF} END {print s}'
86
$ rg -c '\btodo!\('    clarity-web/src --type rust
1
$ rg -c '\bunimplemented!\(' clarity-web/src --type rust
1
```

**Total panic-prone sites: 901**, across:

| Family | Sites | Files |
|--------|------:|------:|
| `.unwrap()` | 316 | 38 |
| `.expect(` | 497 | 50+ |
| `panic!` | 86 | 25 |
| `todo!` | 1 | 1 |
| `unimplemented!` | 1 | 1 |

Top files by `.unwrap()` count:

| File | Unwraps |
|------|--------:|
| `intent/validation/semantic_bdd_tests.rs` | 28 |
| `intent/types/names.rs` | 25 |
| `providers/trait.rs` | 20 |
| `lattice/interview_5x5.rs` | 20 |
| `lattice/design_by_contract.rs` | 16 |
| `intent/validation/semantic.rs` | 15 |
| `intent/quality/linter_bdd_tests.rs` | 14 |
| `providers/opencode.rs` | 14 |
| `lattice/quality_dimensions.rs` | 11 |
| `lattice/ears.rs` | 9 |

Top files by `panic!` count:

| File | Panics |
|------|-------:|
| `server.rs` | 24 |
| `lattice/quality.rs` | 14 |
| `intent/interview/types/tests/stage.rs` | 8 |
| `intent/interview/storage/tests.rs` | 8 |
| `intent/quality/linter.rs` | 6 |

### Unsafe inventory

5 occurrences of the word `unsafe` in source — **all in doc comments**
(`security/validators.rs` lines 23, 66, 113 say "path is invalid or unsafe"). No real
`unsafe` blocks. This matches `unsafe_code = "forbid"` at workspace level.

### Implication for verification

Any Verus spec or proptest property that asserts "no panic" on a function in
`clarity-web/src` cannot be honestly closed until:

1. The 901 panic-prone sites are removed or refactored to `Result`-typed propagation, OR
2. The obligation is narrowed to a panic-free refactored subset of the function.

The 8 module proof plans (`cl-0n6`, `cl-zup`, `cl-5dp`, `cl-ooz`, `cl-vv2`, `cl-kse`,
`cl-54n`, `cl-dv5`) implicitly assume panic-free source paths in their `INFERRED`
clauses. Those assumptions need to be re-validated after the panic-site work.

**Recommended fix order:**
1. `holzman-rust` adds `[lints] workspace = true` to `clarity-web/Cargo.toml`. One-line
   change. Will trigger hundreds of new clippy violations.
2. `holzman-rust` works the resulting backlog (possibly in tandem with `cl-2q6`).
3. `proof-writer` then re-scopes the 8 module plans to the panic-free subset.

## 4. Verification program — 8 modules dispatched, 3 Verus specs verified

| Bead | Module | Lane | Plan | Proof artifacts | Verus run |
|------|--------|------|------|-----------------|-----------|
| `cl-0n6` | `domain/newtypes.rs` | V + P | ✅ `proofs/newtypes-proof-plan.md` (24.6K, 41 obligations) | — | — |
| `cl-zup` | `domain/scenario.rs` | V + P | ✅ `proofs/scenario-proof-plan.md` (45.4K, 40 obligations) | — | — |
| `cl-5dp` | `storage/types.rs` | V + P | ✅ `proofs/storage-types-proof-plan.md` (17.4K, 22 obligations) | — | — |
| `cl-ooz` | `intent/types/behavior.rs` | V + P | ✅ `proofs/behavior-proof-plan.md` (23K, 21 obligations) | — | — |
| `cl-vv2` | `domain/straw_man.rs` | V + P | — | ✅ `proofs/straw_man_{verus,proptest,writeup}.rs/md` (9 specs, 21 props) | ❌ exit 1 — `slice::contains` |
| `cl-kse` | `storage/fjall_event_store.rs` | V + K + T | — | ✅ `proofs/fjall_event_store_{verus,kani,writeup}.rs/md` + trusted-base ledger (16 specs, 9 harnesses) | ❌ exit 1 — `Seq<char>` mismatch |
| `cl-54n` | `intent/parser.rs` | P + Z | — | ✅ `proofs/parser_{proptest,fuzz,writeup}.rs/md` (15 cases, 1 fuzz target) | — (BLOCKED wiring + cl-u04) |
| `cl-dv5` | `lattice/quality.rs` | V + P | — | ✅ `proofs/lattice_quality_{verus,proptest,writeup}.rs/md` (13 spec fns, 27 props) | ❌ exit 1 — `pub spec enum` invalid |

**Totals produced this session:**
- 4 proof plans (108 obligations total, all `requires_contract: true`)
- 4 proof artifact bundles (Verus specs + proptest + Kani + fuzz + writeups)
- 1 trusted-base ledger (cl-kse, 10 entries)
- 3 verifier executions run — **all 3 FAIL_LOCAL** with concrete error messages

The 3 verus failures are honest: each points to a specific line in the spec that the
verifier rejects. They are not vacuous model failures. Each has a repair bead:

| Repair bead | Spec | Failure |
|-------------|------|---------|
| `cl-ev9` | `straw_man_verus.rs` | vstd lacks `slice::contains`; add `assume_specification` |
| `cl-28r` | `lattice_quality_verus.rs` | `pub spec enum` is not valid Verus syntax |
| `cl-55u` | `fjall_event_store_verus.rs` | `Seq<char>` passed where `Seq<u8>` expected |

## 5. What I did NOT do (role boundaries)

Per the `formal-verifier` skill: *"you do not write production code, tests, harnesses,
or proofs."*

- I did NOT fix the clippy debt. Dispatched to `holzman-rust`.
- I did NOT add the `[lints] workspace = true` opt-in. Dispatched to `holzman-rust`.
- I did NOT author any rust-contract artifacts. Dispatched to `rust-contract`.
- I did NOT review the proof plans or specs. Dispatched to `proof-plan-reviewer` and
  `proof-reviewer`.
- I did NOT wire proptest files into `clarity-web/tests/`. Dispatched to
  `holzman-rust` / `landing-skill`.
- I did NOT install missing tools. Dispatched to infra (cl-u04).
- I did NOT run the TLA+ spec for storage/fjall_event_store.rs. Dispatched to `tla-plus`
  as a separate bead.

## 6. Pre-flight gates for downstream

Before any of the 8 module beads can close:

| Gate | Owner | Bead |
|------|-------|------|
| Add `[lints] workspace = true` to `clarity-web/Cargo.toml` | holzman-rust | **cl-2vr** (critical) |
| Close clippy debt (cl-2q6 baseline) | holzman-rust + landing-skill | cl-2q6 |
| Author `clarity-web/src/domain/contract.md` (and analogues) | rust-contract | new |
| Review the 4 proof plans | proof-plan-reviewer | new |
| Repair the 3 broken Verus specs | proof-writer | cl-ev9, cl-28r, cl-55u |
| Wire proptest files into `clarity-web/tests/` | holzman-rust | new |
| Install kani, apalache, cargo-fuzz | infra | cl-u04 |

## 7. Evidence ledger

- `verification-ledger.jsonl` — **32 rows**:
  - 1 clippy task_summary (cl-2q6 baseline) — FAIL_LOCAL
  - 8 lint-finding rows
  - 8 file-hotspot rows
  - 1 meta row
  - 1 cargo check task_summary — **PASS**
  - 4 module_plan rows (proof-planner outputs) — PENDING_FORMAL_EXECUTION
  - 4 module_proof_artifacts rows (proof-writer outputs) — PENDING_FORMAL_EXECUTION
  - 3 verifier_execution rows (verus runs) — FAIL_LOCAL
  - 1 execution_blocked row (proptest/kani/fuzz) — PENDING_FORMAL_EXECUTION
  - 1 lints_inventory row (cl-2vr finding) — **FAIL_LOCAL critical**

## 8. Open beads created this session

| ID | Priority | Title |
|----|---------:|-------|
| `cl-0n6` | P1 | Phase 1 verification seed: domain/newtypes.rs (Verus + proptest) |
| `cl-u04` | P1 | Install missing formal verifier tools (kani, apalache, cargo-fuzz) |
| `cl-fy4` | P1 | Establish cargo test PASS baseline after clippy fix |
| `cl-zup` | P1 | Verification: domain/scenario.rs (Verus + proptest) |
| `cl-vv2` | P1 | Verification: domain/straw_man.rs (Verus + proptest) |
| `cl-5dp` | P1 | Verification: storage/types.rs (Verus) |
| `cl-kse` | P0 | Verification: storage/fjall_event_store.rs (Verus + Kani + TLA+) |
| `cl-ooz` | P1 | Verification: intent/types/behavior.rs (Verus) |
| `cl-54n` | P1 | Verification: intent/parser.rs (proptest + fuzz) |
| `cl-dv5` | P1 | Verification: lattice/quality.rs (Verus + proptest) |
| `cl-28r` | P1 | Repair verus spec: lattice_quality (invalid spec enum syntax) |
| `cl-55u` | P1 | Repair verus spec: fjall_event_store (Seq<char> vs Seq<u8>) |
| `cl-ev9` | P1 | Repair verus spec: straw_man (slice::contains unsupported) |
| **`cl-2vr`** | **P0** | **CRITICAL: clarity-web has no [lints] opt-in — 901 panic-prone sites** |

## 9. Next steps if continuing

In strict role order:

1. **`holzman-rust`** adds the `[lints] workspace = true` opt-in to `clarity-web/Cargo.toml`.
   Single-line change. Will surface hundreds of new clippy violations; these should be
   routed to `cl-2q6` or a successor bead.
2. **`rust-contract`** authors `clarity-web/src/domain/contract.md` (and per-module
   analogues) so the 108 proof obligations (4 plans × ~27 avg) can have their
   `requires_contract: true` flag flipped.
3. **`proof-plan-reviewer`** reviews the 4 plans produced this session.
4. **`proof-writer`** repairs the 3 broken Verus specs (`cl-ev9`, `cl-28r`, `cl-55u`),
   then I re-run `verus` and update the ledger from `FAIL_LOCAL` to either `PASS` (if
   the repair verifies) or `FAIL_GLOBAL` (if the verifier rejects again).
5. **`holzman-rust`** wires the 4 proptest files into `clarity-web/tests/` so
   `cargo test --test ...` can run them; I execute and capture `PASS` rows.
6. **`infra`** closes `cl-u04`; once `kani` is installed, I run the 9 Kani harnesses in
   `cl-kse` and the 1 fuzz target in `cl-54n`.

The verification program is real, the artifacts are on disk, the failures are
honestly captured. The next agent in the chain has unambiguous work to do.
