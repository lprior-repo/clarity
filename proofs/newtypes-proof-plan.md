# Proof Plan — `clarity-web/src/domain/newtypes.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-0n6` (Phase 1 verification seed epic) |
| **Target** | `clarity-web/src/domain/newtypes.rs` |
| **Primary lane** | **V** (Verus) — per `verification-targets.md §5.1` |
| **Secondary lane** | **P** (proptest) — boundary coverage on the validators |
| **Contract status** | **GAP — no `rust-contract` artifact exists for this module.** Clauses below are *inferred from source* and labelled `INFERRED`. The plan is gated on `rust-contract` ratifying (or correcting) these clauses before `proof-writer` runs. |
| **Planner** | `proof-planner` |
| **Date** | 2026-06-21 |

---

## 1. Module characterisation

`domain/newtypes.rs` is **pure validated data**: zero I/O, zero concurrency, zero `unsafe`
(`#![forbid(unsafe_code)]` at `clarity-web/src/domain/mod.rs:12` and workspace-level
`Cargo.toml:10`). It declares five `String`-wrapping newtypes plus a single `NewtypeError`
variant. All validation logic lives in `try_from(String)` and `from_str(&str)`.

| Item | Lines | Kind | Risk surface |
|---|---|---|---|
| `enum NewtypeError { Empty }` | 11–16 | Single-variant error, derives `Clone + PartialEq + Eq + Serialize + Deserialize` | None |
| `struct AnswerId(String)` + `new` / `as_str` / `try_from` / `from_str` / `From<AnswerId> for String` / `Display` | 22–72 | Validated newtype — rejects empty / whitespace-only, preserves original string | **Illegal-state exclusion** |
| `struct StepId(String)` + same surface | 78–128 | Same contract as `AnswerId` | Illegal-state exclusion |
| `struct BeadId(String)` + same surface | 134–184 | Same contract as `AnswerId` | Illegal-state exclusion |
| `struct AnswerValue(String)` + `new` (const) / `is_empty` (const) / `as_str` / `From<String>` / `From<&str>` / `From<AnswerValue> for String` / `Display` / `Default` | 190–237 | Unconstrained newtype — accepts any String | None (deliberately unconstrained) |
| `struct Timestamp(String)` + `new` / `now` / `as_str` / `try_from` / `from_str` / `From<Timestamp> for String` / `Display` / `Default` | 243–307 | Validated newtype — rejects empty/whitespace and requires RFC 3339 parse | **Format compliance** |

**Behaviour-affecting properties observed in source** (all `INFERRED`):

- `AnswerId` / `StepId` / `BeadId` `try_from(String)` — line 59/115/171:
  `if s.trim().is_empty() { Err(NewtypeError::Empty) } else { Ok(Self(s)) }`.
  The constructor **does not trim the inner String**; the original `s` is stored verbatim.
  A string such as `"  abc  "` is accepted and `as_str()` returns `"  abc  "`.
- `Timestamp::try_from(String)` — line 285: returns `Err(NewtypeError::Empty)` when
  `s.trim().is_empty()` OR `chrono::DateTime::parse_from_rfc3339(&s)` fails; otherwise
  `Ok(Self(s))`. The parse error is collapsed onto `NewtypeError::Empty` (one error variant
  for two distinct failure modes — this is a design observation worth flagging).
- `Timestamp::now()` — line 259: `Self(chrono::Utc::now().to_rfc3339())`. Wraps a
  wall-clock read in `String`. Non-pure from the verifier standpoint; requires an
  `extern_spec` or trusted-boundary treatment.
- `AnswerValue` is **explicitly unconstrained** (per doc comment line 190 "can be empty,
  represents user input"). No validator invariants to prove; only structural/serde round-trip.
- All five newtypes derive `Serialize + Deserialize` and use `#[serde(try_from = "String",
  into = "String")]` (or `#[serde(transparent)]` for `AnswerValue`).

---

## 2. Contract gap (honest disclosure)

The bead `cl-0n6` has **no upstream `rust-contract` artifact** under
`clarity-web/src/domain/contract.md` (or any path matching `**/contract.md` in the workspace).
All clauses used in this plan are *inferred by direct reading of the source*, not authored.

**Action required before `proof-writer` runs:** `rust-contract` must ratify or correct the
clauses marked `INFERRED` below. If the inferred clauses are wrong, `proof-writer` will write
specs against the wrong contract. The obligations JSONL sets `requires_contract: true` on
every inferred-clause row so this gate is visible.

`rust-contract` should produce (at minimum):

- `contract.md` for `clarity-web/src/domain/` with clauses keyed to the requirement IDs in §3.
- Explicit decision on whether `AnswerId` / `StepId` / `BeadId` whitespace-only rejection is
  the *full* contract or whether surrounding-whitespace rejection is also required (the
  current source only checks `trim().is_empty()` and preserves the inner string verbatim —
  this is a deliberate or accidental property that the contract must ratify).
- Explicit decision on whether `Timestamp::try_from` collapsing parse-error onto
  `NewtypeError::Empty` is contractual or a bug. If the latter, an additional `NewtypeError`
  variant is needed (out of scope for this plan, but the plan documents the assumption).
- Explicit decision on whether `AnswerValue` has any contract beyond "any String" (the
  current doc says no — plan accepts that).

---

## 3. Requirements & inferred contract clauses

Each row carries a `clause_origin` of either `INFERRED` (this plan) or `AUTHORED`
(rust-contract — currently none).

| Req ID | Source | Inferred clause | Origin |
|---|---|---|---|
| REQ-NT-1 | `AnswerId::try_from` lines 55–64 | `try_from(s)` returns `Ok(Self(s))` iff `!s.trim().is_empty()`; otherwise `Err(NewtypeError::Empty)`. Inner field is `s` verbatim. | INFERRED |
| REQ-NT-2 | `StepId::try_from` lines 111–120 | Same contract as REQ-NT-1 for `StepId`. | INFERRED |
| REQ-NT-3 | `BeadId::try_from` lines 167–176 | Same contract as REQ-NT-1 for `BeadId`. | INFERRED |
| REQ-NT-4 | `AnswerId::from_str` lines 66–72 | `from_str(s)` is equivalent to `try_from(s.to_string())` (round-trip via the owned String detour). | INFERRED |
| REQ-NT-5 | `StepId::from_str` lines 122–128 | Same as REQ-NT-4 for `StepId`. | INFERRED |
| REQ-NT-6 | `BeadId::from_str` lines 178–184 | Same as REQ-NT-4 for `BeadId`. | INFERRED |
| REQ-NT-7 | `Timestamp::try_from` lines 282–293 | Returns `Err(NewtypeError::Empty)` if `s.trim().is_empty()` OR `chrono::DateTime::parse_from_rfc3339(&s)` fails; otherwise `Ok(Self(s))`. | INFERRED |
| REQ-NT-8 | `Timestamp::from_str` lines 295–301 | Same equivalence as REQ-NT-4 for `Timestamp`. | INFERRED |
| REQ-NT-9 | `From<X> for String` lines 49–53, 105–109, 161–165, 276–280 | `String::from(x) == x.0` (extracts inner String). | INFERRED |
| REQ-NT-10 | `as_str` lines 38–40, 94–96, 150–152, 210–212, 265–267 | `t.as_str() == &t.0[..]`. | INFERRED |
| REQ-NT-11 | `Display::fmt` lines 43–47, 99–103, 155–159, 215–219, 270–274 | `format!("{}", t) == t.0` for each newtype. (Display writes the inner String verbatim.) | INFERRED |
| REQ-NT-12 | `AnswerValue` lines 190–237 | Unconstrained newtype: `new` accepts any String; `is_empty()` mirrors inner; `as_str()` returns inner; `From<String>` / `From<&str>` / `From<AnswerValue> for String` are identity conversions; `Default::default()` is empty. | INFERRED |
| REQ-NT-13 | `Timestamp::now` line 259 | Result `r` satisfies `r.as_str()` parses via `chrono::DateTime::parse_from_rfc3339` (chrono library contract). | INFERRED |
| REQ-NT-14 | `Timestamp::default` lines 303–307 | `Timestamp::default() == Timestamp::now()` (same expression, evaluated once at default time). | INFERRED |
| REQ-NT-15 | `#[serde(try_from = "String", into = "String")]` on 4 types | For each validated type, JSON round-trip is identity for well-formed input. | INFERRED |
| REQ-NT-16 | `#[serde(transparent)]` on `AnswerValue` | JSON representation equals the inner String verbatim. | INFERRED |

**Open questions for `rust-contract`** (need answers before proof-writer starts):

1. Is "whitespace-only rejected, original-string preserved verbatim" the full contract for
   `AnswerId` / `StepId` / `BeadId`? Or should surrounding whitespace also be rejected?
   The current source accepts `"  abc  "` and stores it unchanged.
2. Is collapsing RFC 3339 parse-error onto `NewtypeError::Empty` contractual, or is it
   a missing variant? Affects whether REQ-NT-7 needs a refinement distinction.
3. Is `AnswerValue` truly unconstrained? If yes, REQ-NT-12 stands as-is and the proptest
   obligation PO-NT-P-08 is the only behavioural test.

---

## 4. Verifier lane decisions

Per `verifier-trigger-matrix.md`, classify the proof seeds across
Verus / Kani / Flux / Loom / proptest / fuzz.

| Lane | Decision | Evidence / rationale |
|---|---|---|
| **V** (Verus) | **REQUIRED — primary** | Module is pure; validation contracts are exactly what Verus specs. Per `verification-targets.md §5.1`. Verus is installed (`/home/lewis/.local/bin/verus`, v0.2026.05.05). |
| **P** (proptest) | **REQUIRED — secondary** | Validators are small but need broad-input coverage on whitespace edge cases (Unicode whitespace, RTL marks, BOM, large inputs). proptest is bundled with cargo via `clarity-web/Cargo.toml:44`. |
| **K** (Kani) | **NOT APPLICABLE** | Kani's strength (bounded model check of unsafe, fixed-width arithmetic, parser bounds) does not apply. No `unsafe` (`forbid` at workspace and module level), no arithmetic, no hand-written parser, no fixed-width index space. Kani is not installed (per `verification-targets.md §3`); even if applicable, install would be required first. |
| **F** (Flux) | **NOT APPLICABLE** | Verus covers the refinement properties (non-empty after trim, RFC 3339 round-trip) with more rigour at the same author cost. `cargo-flux` is installed but unused for this module. |
| **L** (Loom) | **NOT APPLICABLE** | No concurrency in this module — no threads, channels, atomics, `Send + Sync` interactions, async, or spawn calls. |
| **M** (Miri) | **NOT APPLICABLE** | `#![forbid(unsafe_code)]` at module top (line 12) and workspace level (`Cargo.toml:10`). No `unsafe` blocks anywhere. |
| **T** (TLA+) | **NOT APPLICABLE** | No temporal workflow — no state machine transitions, no retries, no leases, no batch ordering. Pure validated newtypes. |
| **Z** (fuzz) | **NOT APPLICABLE** | No hand-written parser; no codec; no persisted bytes. The deserialisation boundary is structured `serde_json` of typed Rust records (REQ-NT-15/16) — covered by proptest, not fuzz. `cargo-fuzz` is not installed (per `verification-targets.md §3`); even if installed, no parser target exists. |
| **X** (exercise-only) | **NOT APPLICABLE** | The whole module is in scope for V + P coverage. There is no `#[cfg(test)] mod tests` block in `newtypes.rs` to relegate to exercise-only. |

Two infrastructure gaps block adjacent lanes (`K`, `Z`) but those lanes are
`not_applicable` to this module regardless, so the gaps are not blockers for *this* plan.
They are noted for the landing-skill pre-flight in `verification-targets.md §4`.

---

## 5. Proof coverage matrix

| Req ID | Lane | Obligation ID | Targets | Evidence | Verus mode |
|---|---|---|---|---|---|
| REQ-NT-1 | V | PO-NT-V-01 | `AnswerId::try_from` | Verus `requires`/`ensures` prove `result.is_ok() ⇔ !s.trim().is_empty()` and on success inner equals `s`. | exec + spec |
| REQ-NT-2 | V | PO-NT-V-02 | `StepId::try_from` | Same contract as PO-NT-V-01. | exec + spec |
| REQ-NT-3 | V | PO-NT-V-03 | `BeadId::try_from` | Same contract as PO-NT-V-01. | exec + spec |
| REQ-NT-4 | V | PO-NT-V-04 | `AnswerId::from_str` | Verus ensures: `from_str(s).is_ok() ⇔ !s.trim().is_empty()` and on success inner equals `s.to_string()`. | exec + spec |
| REQ-NT-5 | V | PO-NT-V-05 | `StepId::from_str` | Same contract as PO-NT-V-04. | exec + spec |
| REQ-NT-6 | V | PO-NT-V-06 | `BeadId::from_str` | Same contract as PO-NT-V-04. | exec + spec |
| REQ-NT-7 | V | PO-NT-V-07 | `Timestamp::try_from` | Verus ensures: `result.is_ok() ⇔ (!s.trim().is_empty() ∧ chrono::parse_from_rfc3339(s).is_ok())`. Requires trusted extern_spec for `chrono::DateTime::parse_from_rfc3339` (§6). | exec + spec + extern_spec |
| REQ-NT-8 | V | PO-NT-V-08 | `Timestamp::from_str` | Equivalence with `try_from(s.to_string())`. | exec + spec |
| REQ-NT-9 | V | PO-NT-V-09 | `From<AnswerId> for String` | Verus ensures: `String::from(id) == id.0`. | exec |
| REQ-NT-9 | V | PO-NT-V-10 | `From<StepId> for String` | Same. | exec |
| REQ-NT-9 | V | PO-NT-V-11 | `From<BeadId> for String` | Same. | exec |
| REQ-NT-9 | V | PO-NT-V-12 | `From<Timestamp> for String` | Same. | exec |
| REQ-NT-10 | V | PO-NT-V-13 | `AnswerId::as_str` | Verus ensures: `id.as_str() == &id.0[..]`. | exec |
| REQ-NT-10 | V | PO-NT-V-14 | `StepId::as_str` | Same. | exec |
| REQ-NT-10 | V | PO-NT-V-15 | `BeadId::as_str` | Same. | exec |
| REQ-NT-10 | V | PO-NT-V-16 | `Timestamp::as_str` | Same. | exec |
| REQ-NT-11 | V | PO-NT-V-17 | `AnswerId::Display::fmt` | Verus ensures via `#[verifier::external_body]` for the fmt trait body; spec'd postcondition: written output equals inner String. | exec + external_body |
| REQ-NT-11 | V | PO-NT-V-18 | `StepId::Display::fmt` | Same. | exec + external_body |
| REQ-NT-11 | V | PO-NT-V-19 | `BeadId::Display::fmt` | Same. | exec + external_body |
| REQ-NT-11 | V | PO-NT-V-20 | `Timestamp::Display::fmt` | Same. | exec + external_body |
| REQ-NT-11 | V | PO-NT-V-21 | `AnswerValue::Display::fmt` | Same. | exec + external_body |
| REQ-NT-12 | V | PO-NT-V-22 | `AnswerValue::new` / `is_empty` / `as_str` | Verus ensures: `new(s).0 == s`; `is_empty() == s.is_empty()`; `as_str() == &s[..]`. All const fn — body unchanged. | exec |
| REQ-NT-12 | V | PO-NT-V-23 | `From<String> / From<&str> / From<AnswerValue> for String` | Identity conversions; Verus ensures each preserves the inner String. | exec |
| REQ-NT-13 | V | PO-NT-V-24 | `Timestamp::now` | Verus ensures (via trusted extern_spec on `chrono::Utc::now().to_rfc3339()`): `result.as_str()` parses back via `chrono::DateTime::parse_from_rfc3339`. Wall-clock non-determinism is not in the spec. | exec + extern_spec |
| REQ-NT-14 | V | PO-NT-V-25 | `Timestamp::default` | Verus ensures: `Timestamp::default()` produces a Timestamp that round-trips (same property as PO-NT-V-24, since `default` delegates to `now`). | exec |
| REQ-NT-1,2,3 | P | PO-NT-P-01 | `AnswerId` boundary | proptest: any whitespace-only input → `Err`; any input with non-whitespace content → `Ok` with inner preserved verbatim. | n/a (Rust test) |
| REQ-NT-2 | P | PO-NT-P-02 | `StepId` boundary | Same shape as PO-NT-P-01. | n/a |
| REQ-NT-3 | P | PO-NT-P-03 | `BeadId` boundary | Same shape as PO-NT-P-01. | n/a |
| REQ-NT-4,5,6 | P | PO-NT-P-04 | `FromStr` equivalence | proptest: for each of `AnswerId` / `StepId` / `BeadId`, `T::from_str(s) == T::try_from(s.to_string())` (both Ok or both Err). | n/a |
| REQ-NT-7 | P | PO-NT-P-05 | `Timestamp::try_from` | proptest: well-formed RFC 3339 strings round-trip via `as_str`; arbitrary non-RFC-3339 strings rejected; empty/whitespace rejected. | n/a |
| REQ-NT-13 | P | PO-NT-P-06 | `Timestamp::now` | proptest: `Timestamp::default().as_str()` parses back via `chrono::DateTime::parse_from_rfc3339`. | n/a |
| REQ-NT-11 | P | PO-NT-P-07 | `Display` round-trip for all 5 types | proptest: `format!("{}", t) == t.as_str()`. | n/a |
| REQ-NT-12 | P | PO-NT-P-08 | `AnswerValue` round-trip | proptest: `AnswerValue::new(s).as_str() == s.as_str()` and `String::from(v) == s`. | n/a |
| REQ-NT-15,16 | P | PO-NT-P-09 | serde JSON round-trip | proptest: for each of the 4 validated newtypes and `AnswerValue`, `serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap_or_default()) == Ok(t.clone())`. (Note: this is proptest on the boundary, not the proof of the boundary itself.) | n/a |
| n/a | K | PO-NT-K1 | newtypes.rs | n/a | not_applicable |
| n/a | F | PO-NT-F1 | newtypes.rs | n/a | not_applicable |
| n/a | L | PO-NT-L1 | newtypes.rs | n/a | not_applicable |
| n/a | M | PO-NT-M1 | newtypes.rs | n/a | not_applicable |
| n/a | T | PO-NT-T1 | newtypes.rs | n/a | not_applicable |
| n/a | Z | PO-NT-Z1 | newtypes.rs | n/a | not_applicable |
| n/a | X | PO-NT-X1 | newtypes.rs | n/a | not_applicable |

**Unwind bounds:** none of the obligations target loops. `try_from` calls `s.trim().is_empty()`
which is stdlib (Verus ships with verified specs for `String::trim`). No `#[verifier::loop_isolation]`
or `#[kani::unwind]` needed.

**Verus mode summary:**
- 22 obligations use `exec` mode (the original function body, with `requires`/`ensures`).
- 4 obligations (PO-NT-V-17 through PO-NT-V-21, the `Display::fmt` bodies) use
  `exec + #[verifier::external_body]` — the `fmt::Formatter` trait surface is not Verus-friendly
  to spec inline; the spec asserts the postcondition on a ghost witness and trusts the fmt
  machinery.
- 2 obligations (PO-NT-V-07, PO-NT-V-24) require trusted `#[verifier::extern_spec]` on chrono
  functions — see §6 trusted-base plan.
- 9 proptest obligations run as ordinary `#[test]` functions in a `#[cfg(test)] mod tests`
  submodule inside `clarity-web/src/domain/newtypes.rs` (proof-writer adds; no production
  body changes).
- 7 not_applicable rows cite concrete evidence in §4.

---

## 6. Trusted-base plan

These are the assumptions the proofs lean on. Each is either explicitly trusted or has its
own obligation. This section is the source for `trusted-base-ledger/v1` rows in the
downstream ledger (State 12 closure).

| Trust | Why trusted | Mitigation in obligations |
|---|---|---|
| `String::trim()` and `String::trim().is_empty()` are spec-correct in Verus stdlib | Verus distribution ships verified specs for these methods | PO-NT-V-01…V-06, V-08 assume the stdlib spec is correct; if Verus ships a known-bad spec, the entire plan fails and that is a Verus toolchain defect, not our defect. |
| `chrono::DateTime::parse_from_rfc3339` round-trips with `DateTime<FixedOffset>` | `chrono` library contract; not our code | PO-NT-V-07, PO-NT-V-08 require an `#[verifier::extern_spec]` for this function. The extern_spec asserts the postcondition (`Ok(_) ⇔ input is valid RFC 3339`) and trusts the implementation. If chrono breaks the contract, the extern_spec is invalid and PO-NT-V-07 must be repaired. |
| `chrono::Utc::now().to_rfc3339()` returns an RFC 3339 string | `chrono` library contract; not our code | PO-NT-V-24 requires an `#[verifier::extern_spec]` for `Utc::now()` returning a `DateTime<Utc>` and for `DateTime::to_rfc3339()` returning a parseable string. The wall-clock non-determinism is captured in the trusted base; the spec asserts only that the result *is* RFC 3339, not that it equals any expected value. |
| `serde_json::to_string` and `from_str` are total on the well-formed newtypes | serde library contract | PO-NT-P-09 exercises the round-trip directly. If serde silently corrupts values, the property fails. |
| `Display::fmt` for `String` writes the inner String bytes | Rust stdlib | PO-NT-V-17…V-21 assert the newtype `Display::fmt` postcondition (writes inner String). The trait method body is `#[verifier::external_body]` — the Rust stdlib contract is trusted. |
| The five newtype structs are closed (no future fields added without spec update) | Type-system fact at spec-write time | Each obligation's `target` names the struct literal; adding a field is a breaking change that requires spec update — caught at compile time of the spec file. |

No `axiom`, `admit`, or `#[verifier::trusted]` is required **inside** `newtypes.rs` — every
trust boundary is an `extern_spec` on a foreign function or a `#[verifier::external_body]` on
a trait impl. Both are honest and have explicit `compensating_evidence` (proptest rows for the
same property).

---

## 7. Waiver candidates

**None.** All in-scope behaviour is provable under the chosen lanes. The non-applicable
lanes (K, F, L, M, T, Z, X) have concrete evidence in §4 and do not require waivers — they
are genuinely not needed.

If `rust-contract` decides that `AnswerId` / `StepId` / `BeadId` should also reject
surrounding whitespace (not just whitespace-only), an additional refinement predicate
obligation may need to be added (likely Flux or a strengthened Verus `ensures`). That is
an additive change, not a waiver of behaviour.

---

## 8. Bridge input for `proof-to-implementation`

The proof-writer will produce Verus specs and proptest properties. The bridge agent maps
them to:

| Proof claim | Rust source ref | Independent behaviour test |
|---|---|---|
| PO-NT-V-01 | `AnswerId::try_from` lines 55–64 | New `proptest_answer_id_boundary` in `#[cfg(test)] mod tests`. |
| PO-NT-V-02 | `StepId::try_from` lines 111–120 | New `proptest_step_id_boundary`. |
| PO-NT-V-03 | `BeadId::try_from` lines 167–176 | New `proptest_bead_id_boundary`. |
| PO-NT-V-04 | `AnswerId::from_str` lines 66–72 | New `proptest_answer_id_from_str_equiv`. |
| PO-NT-V-05 | `StepId::from_str` lines 122–128 | New `proptest_step_id_from_str_equiv`. |
| PO-NT-V-06 | `BeadId::from_str` lines 178–184 | New `proptest_bead_id_from_str_equiv`. |
| PO-NT-V-07 | `Timestamp::try_from` lines 282–293 | New `proptest_timestamp_try_from`; depends on extern_spec for `chrono::DateTime::parse_from_rfc3339`. |
| PO-NT-V-08 | `Timestamp::from_str` lines 295–301 | New `proptest_timestamp_from_str_equiv`. |
| PO-NT-V-09…V-12 | `From<X> for String` impls | Property assertion in the corresponding proptest boundary test (`String::from(t).as_str() == t.as_str()`). |
| PO-NT-V-13…V-16 | `as_str` methods | Same. |
| PO-NT-V-17…V-21 | `Display::fmt` impls | `proptest_display_roundtrip` covers all 5 types. |
| PO-NT-V-22, V-23 | `AnswerValue` constructor + `From` impls | `proptest_answer_value_roundtrip`. |
| PO-NT-V-24, V-25 | `Timestamp::now` + `Default` | `proptest_timestamp_now_roundtrip`. |
| PO-NT-P-09 | All `Serialize + Deserialize` derives | New `proptest_serde_roundtrip_<type>` per type (5 total). |

The proof-writer must **not** modify any production function body. Verus
`#[verifier::external_body]` is the correct tool for the `fmt::Formatter` impls; trusted
`#[verifier::extern_spec]` is the correct tool for `chrono::DateTime::parse_from_rfc3339`
and `chrono::Utc::now().to_rfc3339()`.

---

## 9. Blockers for proof-writer

1. **Contract ratification (BLOCKING).** `rust-contract` must author `contract.md` (or
   equivalent) for `clarity-web/src/domain/` and either ratify or correct the 16 clauses
   in §3. All 34 `planned` obligations carry `requires_contract: true` to make this gate
   visible (25 Verus + 9 proptest).
2. **Open refinement decisions (NON-BLOCKING but recommended).** §3 lists three open
   questions. Defaulting them to "current source behaviour" yields the obligation set
   above. Defaulting them differently shrinks or grows the set; still tractable but not yet
   planned.
3. **No tooling gaps for this plan.** Verus is installed (v0.2026.05.05); `cargo verus` is
   installed; proptest is bundled via `clarity-web/Cargo.toml:44`. Kani / cargo-fuzz gaps
   exist but do not block this module.
4. **Clippy debt (`cl-2q6`) is independent of this plan.** `domain/newtypes.rs` does not
   contribute to the 64-site lint baseline (verified against the report in
   `formal-verification-report.md §4`). However, the proptest functions added by
   `proof-writer` must pass `cargo clippy --workspace --all-targets -- -D warnings` once
   `cl-2q6` is closed.

---

## 10. Non-targets (explicit)

Per `verification-targets.md §8`, the following are NOT in this plan:

- **Line-by-line proofs.** Refused; not cost-effective for the `From<X> for String`
  and `as_str` trivial extractors. They are spec'd because the spec cost is near-zero and
  the contract gap is fully closed for the type, but no per-line proof ceremony is implied.
- **Miri, Loom, TLA+, fuzz.** All not applicable (§4). No `not_applicable` obligation rows
  for these will be promoted to `waived`.
- **Production `unsafe`** — `forbid` at workspace level (`Cargo.toml:10`) and module level
  (`clarity-web/src/domain/mod.rs:12`); no obligation needed.
- **The `From<&str> for AnswerValue` allocation behaviour** — `s.to_string()` is stdlib;
  no contract beyond the conversion being total.

---

## 11. Pre-flight checklist for landing this plan

- [ ] `rust-contract` produces contract clauses for the 16 inferred items in §3 (or
      supersedes them).
- [ ] `proof-plan-reviewer` reviews this file and the obligations JSONL
      (`newtypes-obligations.planned.jsonl`) and writes `verifier-lane-review.jsonl` with
      independent disposition.
- [ ] Verus invocation command confirmed: `cargo verus verify --manifest-path
      clarity-web/Cargo.toml --crate-type lib` from `/home/lewis/src/clarity`. (Alternative
      single-file form: `verus clarity-web/src/domain/newtypes.rs` — to be confirmed by
      `formal-verifier` based on `cargo verus` integration tests.)
- [ ] proptest invocation confirmed: `cargo test -p clarity-web --lib domain::newtypes
      -- --nocapture` (or the specific `proptest_<name>` filter), with the new property
      functions added in `#[cfg(test)] mod tests_proptest`.
- [ ] `cl-2q6` clippy gate is independent of this plan; once closed, the proptest additions
      must remain clippy-clean.

---

*End of plan.*
