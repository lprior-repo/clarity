# Proof Plan Review — `clarity-web/src/storage/types.rs`

| Field | Value |
|---|---|
| **Bead** | `cl-5dp` |
| **Target** | `clarity-web/src/storage/types.rs` (440 LOC, 5 record types, 4 table constants) |
| **Plan reviewed** | `proofs/storage-types-proof-plan.md` (17.4K, 11 sections, 12 INFERRED clauses) |
| **Obligations reviewed** | `proofs/storage-types-obligations.planned.jsonl` (22 rows: 10 V + 5 P + 7 NA) |
| **Reviewer** | `proof-plan-reviewer` |
| **Reviewer invocation ID** | `proof-plan-reviewer::cl-5dp::2026-06-21T17:05Z` |
| **Review state** | `rejected` |
| **Status** | `STATUS: REJECTED` |

---

## 1. Provenance

| Item | Value |
|------|-------|
| Reviewer skill | `proof-plan-reviewer` |
| Reviewer invocation ID | `proof-plan-reviewer::cl-5dp::2026-06-21T17:05Z` |
| Planner invocation ID (claimed) | `proof-planner::cl-5dp::2026-06-21T15:51Z` (from plan header line 10 + file mtime) |
| Independent invocation | YES — different skill + different invocation timestamp |
| Artifacts reviewed | `proofs/storage-types-proof-plan.md`, `proofs/storage-types-obligations.planned.jsonl` |
| Reviewed artifacts existed before start | YES — both created at 2026-06-21T15:51-15:52Z per `ls -la` |
| Companion reads | `verification-targets.md §5.2`, `formal-verification-report.md`, `clarity-web/src/storage/types.rs`, `clarity-web/src/storage/mod.rs`, `redb_store.rs:1-200`, `fjall_event_store.rs:1-120` |
| Findings emitted | `proofs/storage-types-plan-findings.jsonl` (11 rows) |
| Lane review rows | `proofs/storage-types-verifier-lane-review.jsonl` (22 rows, one per obligation) |
| Repair guide | `proofs/storage-types-plan-repair-guide.md` |

No `agent-invocation-ledger.jsonl` exists in this workspace (verified: no matches under `/home/lewis/src/clarity`). The host control plane does not expose a provenance ledger. Reviewer therefore relies on file mtimes + plan header for provenance, which is the strongest available evidence.

---

## 2. Verdict

**STATUS: REJECTED.** 6 blocking findings + 5 non-blocking observations. The plan's intent and lane classification are largely sound, but four structural gaps make the plan non-actionable for `proof-writer` in its current form:

1. The obligations JSONL is **schema-non-compliant** (8 of 21 required fields missing per `proof-obligation/v1`).
2. The two RFC 3339 obligations (PO-V7, PO-V9) are **vacuous** under the planned `#[verifier::external_body]` strategy — the spec delegates the property it claims to the body itself.
3. The 5 trust assumptions in plan §6 are **unledgered** — no `trusted-base-ledger.jsonl` rows.
4. There is **no `verifier-lane-decisions.jsonl`** for the planner's lane decisions, so the reviewer must use obligation IDs as fallback keys.

Additionally, one open question (non-emptiness of map keys) is **misclassified as non-blocking** when it is in fact behavior-affecting via `redb_store.rs:106,141`.

The plan's strongest points are: (a) the honest contract-gap disclosure in §2, (b) the right primary/secondary lane selection (V+P), (c) sound non-applicability evidence for Kani/Loom/Miri/TLA+, (d) correct identification that `fjall_event_store.rs` does NOT consume these types and is out of scope. The plan is close to actionable — the repairs are mechanical (schema re-render, obligation split, ledger commit), not architectural.

---

## 3. Findings summary

| ID | Code | Severity | Disposition | Title |
|----|------|----------|-------------|-------|
| F-ST-001 | `E_SCHEMA_MISSING_FIELD` | blocker | blocker | Obligations JSONL missing 8 of 21 proof-obligation/v1 fields |
| F-ST-002 | `E_SCHEMA_ALIAS_FIELD` | blocker | blocker | `risk` used as alias for `risk_tags` |
| F-ST-003 | `E_LANE_REVIEW_MISSING` | blocker | blocker | No `verifier-lane-decisions.jsonl` for the planner's lane decisions |
| F-ST-004 | `E_PROOF_PLAN_MISSING_NONVACUITY` | blocker | blocker | PO-V7/PO-V9 RFC 3339 obligations are vacuous under `external_body` |
| F-ST-005 | `E_TRUST_LEDGER_INCOMPLETE` | blocker | blocker | 5 trust assumptions in plan §6 are unledgered |
| F-ST-006 | `E_BEHAVIOR_WAIVER` | high | blocker | step_id/input_hash/phase non-emptiness is behavior-affecting; misclassified as non-blocking |
| F-ST-007 | `E_PROOF_PLAN_MISSING_VERUS` | observation | owner_approved_no_action | cl-2vr cited as 'independent' but formal-verification-report.md §3 lists cl-5dp among plans requiring re-validation |
| F-ST-008 | `E_SCOPE_MISCLASSIFIED_BEHAVIOR` | low | owner_approved_no_action | Plan §1 says '10 hand-written tests'; actually 11 |
| F-ST-009 | `E_SOURCE_REF_SHAPE` | low | owner_approved_no_action | Plan §8 bridge is informal Markdown, not structured `rust-refinement-obligation/v1` |
| F-ST-010 | `E_LANE_DECISION_WEAK` | low | owner_approved_no_action | PO-Z1 stacks two reasons (no target + tool not installed); PO-F1 reasoning weakens after F-ST-004 |
| F-ST-011 | `E_REFINEMENT_HARNESS_MISSING` | low | owner_approved_no_action | PO-V7/PO-V9 lack a separate refinement harness (connected to F-ST-004) |

**Blocking count: 6.** All six are `disposition: blocker` and prevent advancement to `proof-writer`. **Non-blocking count: 5.** All five are `owner_approved_no_action` (no `fixed_with_evidence` because the artifacts do not yet exist; no `owner_approved_debt` because the gaps are mechanical rather than debt).

---

## 4. Reviewer notes per adversarial question

### Q1 — Are 12 INFERRED clauses sufficient?

**Mostly yes, with two additions required.** The 12 clauses cover the constructor totality, serde mapping, RFC 3339 timestamp generation, and table-name distinctness. Missing or under-specified:

- **Map-key non-emptiness** (REQ-ST-1..12 does not assert this): `redb_store.rs:106` uses `ExtractionCache::input_hash` as a redb table key; `redb_store.rs:141` uses `LatticeCache::phase` as a key; `redb_store.rs:175` and `redb_store.rs:188` use `AnswerRecord::step_id` as a key via `AnswerRecord::from_answer(answer.step_id.clone(), …)`. Empty keys are reachable through the public `new(...)` constructors (which take `step_id: String` without validation). This is a real correctness gap, captured in F-ST-006.
- **`ProjectMetadata::created_at <= updated_at` cross-method invariant**: `with_current_timestamp` produces `created_at == updated_at`. Any subsequent field update should preserve `created_at <= updated_at`. No single-function invariant; would need a Verus spec that references the pair as a monotonic pair. Optional addition; not a blocker because no update path exists in this module.
- **Confidence derive traits** (REQ-ST-12): listed but explicitly has no obligation. Acceptable — covered transitively by PO-V1/V2/PO-P1.

### Q2 — Cross-module coupling; types.rs only?

**Acceptable as scoped, with one misclassification.** `storage/types.rs` is consumed by:

| Consumer | Uses | Plan scope |
|----------|------|------------|
| `redb_store.rs:25` | All 4 record types + tables::* | Has own plan (verification-targets.md §5.2) — V+K |
| `redb_transcript_store.rs` | (not opened in this review; listed in mod.rs) | Has own plan (verification-targets.md §5.2) — V+K |
| `transcript_store.rs` | None (only the word 'Confidence' as a doc-comment example at line 99) | Out of scope |
| `integration_test.rs` | ExtractionCache, ProjectMetadata | X (not proof) |
| `fjall_event_store.rs:1-184` | NONE — defines own `EventEnvelope`, uses raw `serde_json::Value` | No coupling; TLA+ spec is for Fjall store itself, not these types |

The plan correctly identifies that the `cl-kse` plan covers the Fjall store. The `fjall_event_store.rs` does NOT consume these types — the plan is right to exclude a TLA+ spec here. The map-key concern (F-ST-006) is the only place where `types.rs` obligations need to *protect consumers* (by asserting non-emptiness, so a redb_store misuse is caught at the type boundary).

### Q3 — Trusted base boundary sound?

**No, missing the formal ledger commit.** The 5 trust assumptions in plan §6 are individually sound (serde_json library contract, chrono library contract, enum closure, String totality, const-evaluation). But the plan never commits these to a `trusted-base-ledger.jsonl`. Two `#[verifier::external_body]` markers in PO-V2 and PO-V7/PO-V9 are particularly concerning: external_body is a trust marker that the formal-verifier skill REQUIRES to be in the ledger (proof-schemas.md trusted-base-ledger/v1 rules). F-ST-005 is the blocker.

### Q4 — Lane discipline: 10 V + 5 P right mix?

**Yes, for pure value types.** The plan correctly:
- Uses Verus for typestate-style invariants (PO-V3 from_answer) and faithful-copy postconditions.
- Uses proptest for serde round-trip (the natural proptest target).
- Rejects Kani (no unsafe, no arithmetic, no parser — sound).
- Rejects Flux (would be redundant if Verus's external_body story works — but F-ST-004 undermines this).
- Rejects Loom/Miri (no concurrency/unsafe — sound).
- Rejects TLA+ (no state machine — sound).
- Rejects fuzz (no parser target — sound but the reasoning should be strengthened; F-ST-010).

One observation: PO-V10 (tables constants non-empty and pairwise distinct) is verifiable as a `const _: () = assert!(...)` check in a doctest, which would be lighter than a Verus spec. But Verus is honest and not wrong.

### Q5 — Three open refinement questions behavior-affecting?

**Q1 is behavior-affecting (F-ST-006).** Q2 and Q3 are arguably behavior-affecting if the consumer deserializes `ExtractionCache.fields` into a structured type or interprets `mode_preference` as an enum. The plan §9 marks all three as "NON-BLOCKING but recommended" — that classification is wrong for Q1 because the empty-key issue is reachable through public `new()` constructors. The fix in F-ST-006 is to elevate Q1 to BLOCKING and require rust-contract to ratify before proof-writer runs.

### Q6 — Clippy/lint impact; test wiring justified?

**Partly.** Plan §11 line 171 claims cl-2q6 is independent. For the production code (lines 1-187 of types.rs), this is true: 0 panic-prone sites. For the test module (11 .expect() under #[allow(...)]) it is also fine. But the plan does not cite cl-2vr (the critical finding from formal-verification-report.md §3 that affects 901 panic-prone sites across clarity-web). The plan's INFERRED clauses do not claim panic-freedom, so cl-2vr is not directly blocking this module, but the plan should at minimum cite cl-2vr as a project-wide pre-flight. Captured as F-ST-007 (observation).

On the proptest wiring: plan §11 line 170 asserts `cargo test -p clarity-web --lib storage::types::tests::proptest_confidence_roundtrip` is the invocation, but the proptest function does not exist yet (only test_confidence_serialization at line 209 exists). The plan §10 acknowledges the test module is to be EXTENDED, so the invocation is aspirational. This is captured in VLR-ST-11 (proptest lane review) — the obligation is rejected as aspirational until a captured PASS row exists.

---

## 5. What is sound in this plan (do not re-litigate on repair)

- **Lane selection (V + P)** is the right call. Do not change to F or Z.
- **Contract gap disclosure** in §2 is honest and well-posed; rust-contract does not need to invent something new, only ratify or correct.
- **Non-applicability evidence** for K, L, M, T is concrete and correct.
- **Module scoping** is right — types.rs is pure data and proof-amenable.
- **Bridge plan §8** correctly maps each obligation to a named existing test. The line numbers are accurate (verified against types.rs).
- **Trusted base §6** is conceptually right; the fix is to commit it to a ledger, not to re-think the assumptions.

---

## 6. What proof-writer cannot do today (gates from the review)

- Cannot validate the JSONL against the schema — 8 of 21 fields are missing.
- Cannot write a non-vacuous Verus spec for PO-V7/PO-V9 without first choosing between (a) splitting the obligation, (b) lifting the hand-written test, or (c) adding a chrono extern_spec linkage.
- Cannot produce a trusted-base-ledger.jsonl — the planner did not produce one.
- Cannot run `verus` on `proofs/verus/storage_types_*.rs` (the files do not exist; the plan §5 references them as if they do).
- Cannot close the contract gap (this is rust-contract's job, not proof-writer's).

---

## 7. Recommendation

**REPAIR, not REWRITE.** The plan's architecture is right. The repair path is:

1. **Re-render the obligations JSONL** against the canonical `proof-obligation/v1` schema, using `proofs/newtypes-obligations.planned.jsonl` as the reference shape. (F-ST-001, F-ST-002)
2. **Emit a `verifier-lane-decisions.jsonl`** with one `verifier-lane-decision/v1` row per obligation so the reviewer can match by canonical decision ID. (F-ST-003)
3. **Split PO-V7 and PO-V9** into (a) faithful-copy and (b) RFC 3339 parseability, where (b) either lifts the hand-written test, uses a chrono extern_spec, or moves to a different verifier. (F-ST-004)
4. **Commit a `trusted-base-ledger.jsonl`** with one row per trust marker. (F-ST-005)
5. **Reclassify Q1 (map-key non-emptiness) as BLOCKING** in the plan's open questions. (F-ST-006)
6. Apply the 5 observation fixes (F-ST-007 through F-ST-011).

After repair, the plan is APPROVE-able in a single re-review pass. The reviewer will then flip 22 verifier-lane-review/v1 rows from `rejected` to `accepted`.

---

## 8. STATUS

**STATUS: REJECTED**

The plan is structurally close to actionable but six blocking findings prevent advancement to `proof-writer`. See `proofs/storage-types-plan-repair-guide.md` for the smallest state to rerun, and `proofs/storage-types-plan-findings.jsonl` for the 11 findings.
