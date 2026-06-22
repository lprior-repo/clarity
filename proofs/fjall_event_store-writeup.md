# Proof Writeup — `clarity-web/src/storage/fjall_event_store.rs`

**Bead:** `cl-kse`
**Lane:** Verus (V) primary, Kani (K) secondary
**Verification Targets §5.2:** `storage/fjall_event_store.rs` — Event store over Fjall LSM
**Date:** 2026-06-21
**Producer:** `proof-writer`
**Companion files:**

- `proofs/fjall_event_store_verus.rs` — Verus specs and proofs (this writeup is the map)
- `proofs/fjall_event_store_kani.rs` — Kani harness source (ready when `cl-u04` closes)
- `.beads/cl-kse/proof-writer-report.md` — proof-writer report
- `.beads/cl-kse/proof-evidence.md` — execution evidence template
- `.beads/cl-kse/trusted-base-ledger.jsonl` — trust boundaries (`TB-FJALL-001..005`)

---

## 1. Module characterisation

| Property | Value |
|---|---|
| Source file | `clarity-web/src/storage/fjall_event_store.rs` |
| LOC | 184 (incl. `#[cfg(test)] mod tests`); ~160 LOC of production |
| `unsafe` | **forbidden** — `#![forbid(unsafe_code)]` at module top |
| `unwrap`/`expect`/`panic`/`todo`/`unimplemented` | **denied** at workspace level via `Cargo.toml` `[workspace.lints.clippy]` |
| Concurrency | none (no `Send`, `Sync`, threads, channels, atomics, async) |
| I/O | Fjall LSM (`Database::builder`, `Keyspace`, `db.persist(SyncAll)`) |
| Crypto | `sha2::Sha256` (digest + first 16-byte truncation) |

### Production surface (line-numbered)

| Line | Item | Risk class |
|---:|---|---|
| 21 | `const SESSION_HASH_BYTES: usize = 16;` | Static |
| 23-30 | `enum FjallStoreError { Fjall, Serialization }` | Railway taxonomy |
| 32-38 | `impl From<fjall::Error> for FjallStoreError` | `From` graph |
| 40-53 | `struct EventEnvelope { session_id, seq, ..., event_hash }` | Pure data |
| 55-64 | `struct FjallEventStore { db, events, snapshots, locks, artifacts, gate_results, projection_status, bd_mappings }` | Owned handles |
| 73-85 | `FjallEventStore::open(path) -> Result<Self, FjallStoreError>` | I/O + 7 keyspaces |
| 92-102 | `FjallEventStore::append_event_sync_all(event) -> Result<(), FjallStoreError>` | Write-batching + `SyncAll` persist |
| 110-122 | `FjallEventStore::load_events(session_id) -> Result<Vec<EventEnvelope>, FjallStoreError>` | Prefix iteration + JSON deserialise |
| 126-131 | `pub fn event_key(session_id: &str, seq: u64) -> Vec<u8>` | **Pure arithmetic — primary V target** |
| 134-137 | `pub fn session_key_prefix(session_id: &str) -> Vec<u8>` | **Pure arithmetic — primary V target** |
| 140-143 | `pub fn canonical_sha256(bytes: &[u8]) -> String` | **Pure arithmetic — primary V target** |
| 145-148 | `fn open_keyspace(db, name)` | Thin wrapper |

---

## 2. Spec → source map (Verus)

This artifact is **spec-only** (mirrors the `lattice_quality_verus.rs`
convention in this repo). The spec functions model the production
behaviour line-for-line; the proof lemmas establish the structural
invariants (length, prefix/suffix boundaries, concatenation order,
hex encoding). The bridge to production is recorded below and is
enforced at runtime by the Kani harnesses (`fjall_event_store_kani.rs`)
once `cl-u04` closes.

Production code is **not** modified by this artifact.

| Spec | Type | Source ref | Property |
|---|---|---|---|
| `spec_sha256` | `open spec fn` | `sha2::Sha256::digest(...)` (called at lines 135, 141) | **Trusted — TB-FJALL-001** (length-32 trusted to NIST FIPS 180-4) |
| `spec_session_key_prefix` | `open spec fn` | lines 135-136 (`.iter().take(SESSION_HASH_BYTES).copied().collect()`) | First 16 bytes of SHA-256 |
| `spec_event_key` | `open spec fn` | lines 126-131 | `session_key_prefix(session_id) ++ seq.to_be_bytes()` |
| `spec_canonical_sha256_prefix` | `open spec fn` | line 142 | Literal 7 ASCII chars `"sha256:"` |
| `spec_hex_encode` | `open spec fn` | line 142 (`format!("{:x}", digest)`) | Lowercase hex (2 chars per byte) — **TB-FJALL-004** |
| `spec_canonical_sha256` | `open spec fn` | lines 140-143 | `"sha256:" ++ hex(digest)` |
| `seq_to_be_bytes_spec` | `open spec fn` | line 129 (`seq.to_be_bytes()`) | 8-byte BE encoding — **TB-FJALL-005** |
| `key_length_invariant` | `open spec fn` | lines 126-131 | `event_key(input, seq).len() == 24` — **PO-V1** |
| `key_suffix_invariant` | `open spec fn` | lines 126-131 | Last 8 bytes == `seq.to_be_bytes()` — **PO-V2** |
| `key_prefix_invariant` | `open spec fn` | lines 126-131 | First 16 bytes == `session_key_prefix(input)` — **PO-V3** |
| `key_distinct_invariant` | `open spec fn` | lines 126-131 | `seq_a != seq_b ⇒ key_a != key_b` — **PO-V4** |
| `session_prefix_length_invariant` | `open spec fn` | lines 134-137 | `session_key_prefix(input).len() == 16` — **PO-V5** |
| `canonical_sha256_invariant` | `open spec fn` | lines 140-143 | Length 71 + prefix `"sha256:"` — **PO-V6** |
| `append_event_key_invariant` | `open spec fn` | lines 92-102 | The key written is `event_key(event.session_id, event.seq)` — **PO-V7** |
| `axiom_sha256_length` | `proof fn` | sha2 contract | `spec_sha256(input).len() == 32` (**TB-FJALL-001**) |
| `axiom_truncate_is_prefix` | `proof fn` | lines 135-136 | prefix length 16, prefix-of-digest (**TB-FJALL-002**) |
| `axiom_be_bytes_injective` | `proof fn` (admits) | line 129 | BE encoding injective (**TB-FJALL-005**, single `admit()`) |
| `lemma_key_length` | `proof fn` | — | Proves PO-V1 |
| `lemma_key_suffix` | `proof fn` | — | Proves PO-V2 from PO-V1 + spec |
| `lemma_key_prefix` | `proof fn` | — | Proves PO-V3 from PO-V1 + spec |
| `lemma_key_distinct` | `proof fn` | — | Proves PO-V4 via BE-encoding injectivity |
| `lemma_session_prefix_length` | `proof fn` | — | Proves PO-V5 from `axiom_truncate_is_prefix` |
| `lemma_canonical_sha256` | `proof fn` | — | Proves PO-V6 from spec definition |
| `lemma_append_event_key` | `proof fn` | — | Trivial from spec |

### PO-V7 write-batching invariant

Production's `append_event_sync_all` (lines 92-102) writes to Fjall using a key
computed by `event_key(&event.session_id, event.seq)`. The I/O success path
is owned by Lane K (PO-K1..K9 below) and by TLA+ (separate spec, owned by
`tla-plus` — not this bead). The Verus spec pins only the **structural key
contract** that bridges the two lanes:

> When `append_event_sync_all` successfully returns `Ok(())`, the key it
> wrote into the `events` keyspace equals `event_key(event.session_id, event.seq)`.

This is the "write-batching invariant" named in `verification-targets.md §5.2`.

---

## 3. Spec → source map (Kani)

Every harness in `fjall_event_store_kani.rs` calls a production function
directly. No harness re-implements production logic to satisfy itself.

| Obligation | Harness | Property | Bounded inputs |
|---|---|---|---|
| **PO-K1** | `verify_event_key_length_is_24_bytes` | `event_key(s, seq).len() == 24` for any `s`, `seq` | unbounded; cover at `seq=0`, `seq=u64::MAX`, `s=""` |
| **PO-K2** | `verify_event_key_ends_with_seq_be_bytes` | `event_key(s, seq)[16..] == seq.to_be_bytes()` | unbounded; cover at BE=zero, BE=high |
| **PO-K3** | `verify_event_key_starts_with_session_prefix` | `event_key(s, seq)[..16] == session_key_prefix(s)` | unbounded |
| **PO-K4** | `verify_distinct_sequences_produce_distinct_keys` | `seq_a != seq_b ⇒ key_a != key_b` | `kani::assume(seq_a != seq_b)`; cover at low/high boundaries |
| **PO-K5** | `verify_event_key_ordering_preserves_seq_order` | `seq_a < seq_b ⇒ key_a < key_b` (lex) | `kani::assume(seq_a < seq_b)` |
| **PO-K6** | `verify_canonical_sha256_length_is_71` | `canonical_sha256(b).len() == 71` | bounded `Vec<u8>` ≤ 64 bytes |
| **PO-K7** | `verify_canonical_sha256_starts_with_sha256_prefix` | starts with literal `"sha256:"` | bounded `Vec<u8>` ≤ 64 bytes |
| **PO-K8** | `verify_session_key_prefix_length_is_16` | `session_key_prefix(s).len() == 16` | unbounded; cover at `s=""` |
| **PO-K9** | `verify_event_key_panic_free_at_extremes` | no panic at u64 MIN/MAX-1/MAX and empty/long session_id | concrete enumeration |

### Unwind bounds

| Harness | Loop(s) in production | Unwind set | Justification |
|---|---|---:|---|
| `verify_event_key_length_is_24_bytes` | 16 (session prefix) + 8 (BE suffix) | 32 | prefix 16 + suffix 8 + 8 margin |
| `verify_event_key_ends_with_seq_be_bytes` | same | 32 | same |
| `verify_event_key_starts_with_session_prefix` | same | 32 | same |
| `verify_distinct_sequences_produce_distinct_keys` | same | 32 | same |
| `verify_event_key_ordering_preserves_seq_order` | same | 32 | same |
| `verify_canonical_sha256_length_is_71` | 32 (hex loop in `canonical_sha256`) | 33 | hex 32 + 1 margin |
| `verify_canonical_sha256_starts_with_sha256_prefix` | same | 33 | same |
| `verify_session_key_prefix_length_is_16` | 16 (truncation) | 32 | 16 + 16 margin |
| `verify_event_key_panic_free_at_extremes` | same as K1 | 32 | same as K1 |

### Stubs and contracts (none)

This harness set does not use `#[kani::stub]`, `#[kani::stub_verified]`, or
function contracts (`-Z function-contracts`). All harnesses call into the
**pure arithmetic helpers** (`event_key`, `session_key_prefix`,
`canonical_sha256`) only — none touch the Fjall I/O path. Adding a
`verify_open_rejects_missing_directory` or similar negative harness is
gated on `kani::stub_verified` contracts being defined for the Fjall
crate, which is out of scope for this bead.

### Resource governance (when executed)

When `cl-u04` closes and the harnesses are wired in:

```bash
# Per-harness, single-job, cgroup-capped.  See kani skill §3.
systemd-run --user --scope --collect \
    -p MemoryHigh=20G -p MemoryMax=24G -p MemorySwapMax=0 \
    cargo kani -j 1 --manifest-path clarity-web/Cargo.toml \
        --harness kani_proofs::verify_event_key_length_is_24_bytes \
        --output-format=regular
```

Memory cap is set per the `kani` skill's resource-governance rule. No
`-j > 1` is used; no `--no-*-checks` flags are passed.

---

## 4. TLA+ (NOT IN THIS BEAD — owned by `tla-plus`)

The TLA+ spec for `fjall_event_store.rs` (write-batch ack, recovery,
snapshot-consistency temporal property) is owned by the `tla-plus` agent
per `verification-targets.md §5.2` and the bead description. It is
**explicitly excluded** from this artifact:

> A TLA+ spec is a separate concern — do not write TLA+ here; that is
> owned by `tla-plus`. — proof-writer brief for `cl-kse`

The Verus `append_event_key_invariant` (PO-V7) is the *static* part of the
contract; the *temporal* part (commit durability, recovery ordering,
snapshot consistency) is what TLA+ will cover.

---

## 5. What is intentionally out of scope

| Item | Reason | Where it lives |
|---|---|---|
| TLA+ spec for `fjall_event_store.rs` | Separate bead, separate agent | TLA+ owned by `tla-plus` |
| Line-by-line proofs of `serde_json::to_vec` / `serde_json::from_slice` | Stdlib contract; not our code | proptest by `test-writer` |
| Loom model of Fjall LSM I/O | No `Send + Sync` interactions in production; the Fjall library is single-threaded per `Database` | (none — Loom lane N/A) |
| Miri run | `#![forbid(unsafe_code)]` at workspace and module level | (none — Miri lane N/A) |
| Fuzz targets | No hand-written parser/codec | (none — fuzz lane N/A) |
| Kani execution (PO-K1..K9) | `kani` not installed; cl-u04 open | `proof-writer` produces source; `formal-verifier` runs when cl-u04 closes |
| `append_event_sync_all` I/O success/failure mode | Fjall runtime concerns owned by Kani (stubs when land) + TLA+ | Future beads |

---

## 6. Acceptance gates (for `formal-verifier` when cl-u04 closes)

### Verus — `verus proofs/fjall_event_store_verus.rs`

- Expected exit code: **0**
- Expected verifier summary: "7 verified, 0 errors" (PO-V1..V7 proofs accepted)
- Trusted-boundary scan: `rg -n 'admit\(|\[verifier::external_body\]' proofs/fjall_event_store_verus.rs`
  must report **1 `admit()` site** (TB-FJALL-005 inside
  `axiom_be_bytes_injective`) and **0 `#[verifier::external_body]`** sites.
  No other admits or external bodies are tolerated.
- `verusfmt --check proofs/fjall_event_store_verus.rs` (if `verusfmt`
  installed): **0 diffs** expected.

### Kani — `cargo kani --harness kani_proofs::verify_*` (per-harness)

- 9 harnesses, each runs in isolation
- Expected per-harness result: `VERIFICATION:- SUCCESSFUL`
- Expected summary line: `Check 9: <assertion>.SUCCESS`
- 0 unsound assumptions, 0 unwinding failures, 0 unsupported-feature
  warnings (production path uses only stdlib `Vec`, `String`, `[u8; N]`)

### Cross-lane consistency check

- PO-V1 (Verus `key_length_invariant`) and PO-K1 (Kani
  `verify_event_key_length_is_24_bytes`) **must agree**: both conclude
  `event_key.len() == 24`. A disagreement is a proof or harness bug.
- PO-V4 (Verus `key_distinct_proof`) and PO-K4 (Kani
  `verify_distinct_sequences_produce_distinct_keys`) **must agree**:
  both conclude `seq_a != seq_b ⇒ key_a != key_b`.
- PO-V6 (Verus `canonical_sha256_proof`) and PO-K6 + PO-K7 (Kani
  `verify_canonical_sha256_length_is_71` + `..._starts_with_sha256_prefix`)
  **must agree** on length and prefix.

---

## 7. Honest disclosure (per `proof-writer` skill rule §10)

| Item | Status |
|---|---|
| `cargo verus` or `verus` actually run on this artifact | **No** — proof-writer brief says "Do NOT run verus, kani, or cargo." Source written; execution owned by `formal-verifier`. |
| `cargo kani` actually run on this artifact | **No** — `kani` not installed (cl-u04 open). Source written; execution owned by `formal-verifier` after cl-u04 closes. |
| `rust-contract` artifact exists for this module | **No** — proof-writer brief anchors specs to source per bead description, not to an approved contract. All clauses in this writeup are **INFERRED**. |
| `proof-plan-review.md` approved | **No** — no upstream plan exists. This artifact is the *writer* output, not a *plan* output. The plan reviewer is owed a separate pre-flight by `proof-planner` before this work is treated as a closing pipeline. |
| `cl-2q6` (clippy debt) closed | **No** — `cargo clippy --workspace --all-targets -- -D warnings` still fails with 64 lint sites. Verus itself does not require clippy, so this does not block the Verus source artifact, but `cargo kani` and the wider `moon run :ci` pipeline remain blocked. |

---

## 8. Cross-references

- `verification-targets.md §5.2` — module/lane classification
- `verification-ledger.jsonl` row 18 — meta note "No proof-obligations.planned.jsonl exists"
- `formal-verification-report.md §6` — "What this report does not cover"
- `.beads/cl-kse/trusted-base-ledger.jsonl` — TB-FJALL-001..005 trust records
- `.beads/cl-kse/proof-writer-report.md` — report back
- `.beads/cl-kse/proof-evidence.md` — execution evidence template
- `proofs/storage-types-proof-plan.md` — prior proof plan for the
  adjacent `storage/types.rs` module (different module, same convention)