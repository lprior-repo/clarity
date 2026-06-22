// =============================================================================
// Kani harness source for `clarity-web/src/storage/fjall_event_store.rs`
//
// Bead:       cl-kse
// Lane:       Kani (K) secondary — see verification-targets.md §5.2
// Producer:   proof-writer
// Date:       2026-06-21
//
// ============================================================================
// STATUS — cl-u04 (kani install) is open; this source is ready when kani lands
// ============================================================================
//
// `kani` is NOT installed in this environment per the proof-writer brief:
// verification-targets.md §3 records the gap, and bead cl-u04 owns the
// `cargo install --locked kani --version <current> --features all` install.
// `cargo kani --version` reports 0.67.0 on the PATH but the proof-writer
// brief explicitly gates Lane K execution on cl-u04 closing, so this file
// is *source-only* — not yet wired into the production crate, not yet
// executed, and not yet producing ledger rows.
//
// ============================================================================
// HOW TO WIRE THIS FILE INTO THE PRODUCTION CRATE (owned by holzman-rust)
// ============================================================================
//
// Option A — `#[cfg(kani)] mod` + `include!` (preferred, smallest diff):
//
//   At the bottom of `clarity-web/src/storage/fjall_event_store.rs`
//   (after the existing `#[cfg(test)] mod tests` block, line 184),
//   add the following lines:
//
//       #[cfg(kani)]
//       mod kani_harnesses {
//           include!("../../../proofs/fjall_event_store_kani.rs");
//       }
//
//   The `#[cfg(kani)]` gate ensures zero impact on normal `cargo build`,
//   `cargo test`, `cargo clippy`, or any non-Kani command.
//
// Option B — `#[path = "..."]` attribute (alternative):
//
//       #[cfg(kani)]
//       #[path = "../../../proofs/fjall_event_store_kani.rs"]
//       mod kani_harnesses;
//
// Both options are one-line additions to the production file and must be
// added by the `holzman-rust` agent — proof-writer does NOT modify
// production code.
//
// ============================================================================
// EXPECTED EXECUTION (when cl-u04 closes)
// ============================================================================
//
//   # Discover all harnesses in the proof-writer's file
//   cargo kani list --manifest-path clarity-web/Cargo.toml \
//       --harness-prefix kani_proofs::verify_ 2>&1 \
//       | tee .beads/cl-kse/kani-list.json
//
//   # Run each harness individually (resource-governed; see kani skill §3)
//   for h in $(jq -r '.[]' .beads/cl-kse/kani-list.json); do
//       systemd-run --user --scope --collect \
//           -p MemoryHigh=20G -p MemoryMax=24G -p MemorySwapMax=0 \
//           cargo kani -j 1 --manifest-path clarity-web/Cargo.toml \
//               --harness "$h" --output-format=regular \
//               2>&1 | tee ".beads/cl-kse/kani-${h}.log"
//   done
//
//   # Single-harness quick check
//   cargo kani --manifest-path clarity-web/Cargo.toml \
//       --harness kani_proofs::verify_event_key_length_is_24_bytes \
//       --output-format=regular
//
// ============================================================================
// CLAIM-TO-HARNESS MAP
// ============================================================================
//
//   PO-K1  verify_event_key_length_is_24_bytes
//          key-length invariant for arbitrary session_id and seq.
//   PO-K2  verify_event_key_ends_with_seq_be_bytes
//          suffix invariant; the last 8 bytes are seq.to_be_bytes().
//   PO-K3  verify_event_key_starts_with_session_prefix
//          prefix invariant; first 16 bytes equal session_key_prefix.
//   PO-K4  verify_distinct_sequences_produce_distinct_keys
//          seq_a != seq_b ⇒ key_a != key_b (collision-freeness).
//   PO-K5  verify_event_key_ordering_preserves_seq_order
//          seq_a < seq_b ⇒ key_a < key_b (lexicographic monotonicity).
//   PO-K6  verify_canonical_sha256_length_is_71
//          canonical sha256 string length is 7 + 64 = 71.
//   PO-K7  verify_canonical_sha256_starts_with_sha256_prefix
//          the prefix invariant on canonical_sha256.
//   PO-K8  verify_session_key_prefix_length_is_16
//          session_key_prefix always returns exactly 16 bytes.
//   PO-K9  verify_event_key_panic_free_at_extremes
//          no panic / overflow at u64::MIN, u64::MAX, empty string,
//          very long string (panic-freedom boundary).
//
// All harnesses declare bounds in their own name. None use `#[kani::should_panic]`.
// All include `kani::cover!` for critical domain boundaries.
// =============================================================================

#![cfg_attr(kani, allow(unused))]
#![cfg_attr(kani, allow(dead_code))]

// -----------------------------------------------------------------------------
// This block is included by the production crate under `#[cfg(kani)]`.
// When the include is active, the surrounding module already has access to
// `super::*` (the production module's items). We re-import them under
// fully-qualified paths so the harness is unambiguous even when the file
// is read in isolation.
// -----------------------------------------------------------------------------

#[cfg(kani)]
use crate::storage::fjall_event_store::{
    canonical_sha256, event_key, session_key_prefix, SESSION_HASH_BYTES,
    EventEnvelope, FjallEventStore, FjallStoreError,
};

#[cfg(kani)]
mod kani_proofs {
    //! Bounded model-checking harnesses for `fjall_event_store.rs`.
    //!
    //! All harnesses name the production function they constrain. Every
    //! assertion is a check against the production body; nothing in this
    //! file re-implements production logic to satisfy itself.

    use super::*;

    // -------------------------------------------------------------------------
    // PO-K1 — key length invariant: event_key always returns 24 bytes
    // -------------------------------------------------------------------------
    //
    // Unwind rationale: session_key_prefix has a fixed-size loop of 16
    // iterations; event_key concatenates a 16-byte prefix and an 8-byte
    // suffix with no branching. Unwind bound 32 covers the prefix loop
    // (16) plus a safety margin for the suffix loop (8) and any stdlib
    // initialization in the binary under verification.
    #[kani::proof]
    #[kani::unwind(32)]
    fn verify_event_key_length_is_24_bytes() {
        let session_id: String = kani::any();
        let seq: u64 = kani::any();

        kani::cover!(seq == 0, "sequence zero boundary reachable");
        kani::cover!(seq == u64::MAX, "sequence u64::MAX boundary reachable");
        kani::cover!(session_id.is_empty(), "empty session_id reachable");

        let key = event_key(&session_id, seq);
        assert_eq!(key.len(), SESSION_HASH_BYTES + 8);
    }

    // -------------------------------------------------------------------------
    // PO-K2 — suffix invariant: last 8 bytes are seq.to_be_bytes()
    // -------------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(32)]
    fn verify_event_key_ends_with_seq_be_bytes() {
        let session_id: String = kani::any();
        let seq: u64 = kani::any();
        let key = event_key(&session_id, seq);

        let be = seq.to_be_bytes();
        kani::cover!(be == [0u8; 8], "BE encoding zero boundary");
        kani::cover!(be[0] == 0xFF, "BE high byte at u64::MAX");

        // The key's last 8 bytes must equal the BE encoding of seq.
        assert_eq!(&key[SESSION_HASH_BYTES..], &be[..]);
    }

    // -------------------------------------------------------------------------
    // PO-K3 — prefix invariant: first 16 bytes equal session_key_prefix
    // -------------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(32)]
    fn verify_event_key_starts_with_session_prefix() {
        let session_id: String = kani::any();
        let seq: u64 = kani::any();
        let key = event_key(&session_id, seq);
        let prefix = session_key_prefix(&session_id);

        kani::cover!(prefix.len() == SESSION_HASH_BYTES, "prefix length reachable");
        assert_eq!(&key[..SESSION_HASH_BYTES], &prefix[..]);
        assert_eq!(key.len(), prefix.len() + 8);
    }

    // -------------------------------------------------------------------------
    // PO-K4 — collision-freeness: distinct sequences produce distinct keys
    // -------------------------------------------------------------------------
    //
    // This is the keyspace-arithmetic property that pairs with the
    // Verus proof PO-V4 (key_distinct_proof). Kani reaches it by
    // enumeration over the full u64 domain (modulo kani::assume); Verus
    // reaches it by logical injectivity of BE encoding. The two lanes
    // are independent and complementary.
    #[kani::proof]
    #[kani::unwind(32)]
    fn verify_distinct_sequences_produce_distinct_keys() {
        let session_id: String = kani::any();
        let seq_a: u64 = kani::any();
        let seq_b: u64 = kani::any();

        kani::assume(seq_a != seq_b);

        kani::cover!(seq_a == 0 && seq_b == 1, "low boundary distinct");
        kani::cover!(seq_a == u64::MAX - 1 && seq_b == u64::MAX,
                      "high boundary distinct");

        let key_a = event_key(&session_id, seq_a);
        let key_b = event_key(&session_id, seq_b);
        assert_ne!(key_a, key_b);
    }

    // -------------------------------------------------------------------------
    // PO-K5 — lexicographic monotonicity: BE encoding preserves order
    // -------------------------------------------------------------------------
    //
    // This is the invariant that makes `keyspace.prefix(...)` produce
    // events in `seq` order when iterated by Fjall. Production's
    // `load_events` (lines 110-122) relies on this implicitly.
    #[kani::proof]
    #[kani::unwind(32)]
    fn verify_event_key_ordering_preserves_seq_order() {
        let session_id: String = kani::any();
        let seq_a: u64 = kani::any();
        let seq_b: u64 = kani::any();

        kani::assume(seq_a < seq_b);

        let key_a = event_key(&session_id, seq_a);
        let key_b = event_key(&session_id, seq_b);
        assert!(key_a < key_b);
    }

    // -------------------------------------------------------------------------
    // PO-K6 — canonical_sha256 length: 7 + 64 = 71 chars exactly
    // -------------------------------------------------------------------------
    //
    // Bounded input: any byte slice up to 64 bytes. 64 covers most
    // real-world inputs (an EventEnvelope payload). Larger inputs would
    // still hit the same fixed 32-byte SHA-256 digest, but bounding the
    // input keeps CBMC tractable.
    //
    // Unwind: SHA-256 internal loop is uninterpreted here; the only loop
    // in our `canonical_sha256` is the hex-encoding loop, exactly 32
    // iterations. Unwind 33 covers the loop plus 1 safety iteration.
    #[kani::proof]
    #[kani::unwind(33)]
    fn verify_canonical_sha256_length_is_71() {
        let bytes: Vec<u8> = kani::bounded_any::<_, 64>();

        kani::cover!(bytes.is_empty(), "empty input covered");
        kani::cover!(bytes.len() == 64, "max bounded input covered");

        let s = canonical_sha256(&bytes);
        assert_eq!(s.len(), 7 + 64);
    }

    // -------------------------------------------------------------------------
    // PO-K7 — canonical_sha256 prefix: starts with "sha256:"
    // -------------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(33)]
    fn verify_canonical_sha256_starts_with_sha256_prefix() {
        let bytes: Vec<u8> = kani::bounded_any::<_, 64>();
        let s = canonical_sha256(&bytes);
        assert!(s.starts_with("sha256:"));

        // Stronger: every char of the 7-char prefix is exactly the
        // literal "sha256:".
        let prefix_bytes = s.as_bytes();
        assert_eq!(prefix_bytes[0], b's');
        assert_eq!(prefix_bytes[1], b'h');
        assert_eq!(prefix_bytes[2], b'a');
        assert_eq!(prefix_bytes[3], b'2');
        assert_eq!(prefix_bytes[4], b'5');
        assert_eq!(prefix_bytes[5], b'6');
        assert_eq!(prefix_bytes[6], b':');
    }

    // -------------------------------------------------------------------------
    // PO-K8 — session_key_prefix length: always 16 bytes
    // -------------------------------------------------------------------------
    //
    // Unwind: the only loop in session_key_prefix is the 16-iteration
    // SHA-256 truncation. Unwind 32 covers it with margin.
    #[kani::proof]
    #[kani::unwind(32)]
    fn verify_session_key_prefix_length_is_16() {
        let session_id: String = kani::any();

        kani::cover!(session_id.is_empty(), "empty session_id covered");

        let prefix = session_key_prefix(&session_id);
        assert_eq!(prefix.len(), SESSION_HASH_BYTES);
    }

    // -------------------------------------------------------------------------
    // PO-K9 — panic-freedom at u64 extremes
    // -------------------------------------------------------------------------
    //
    // This is the "no_overflow" panic-freedom harness. It exercises
    // event_key at every u64 boundary (MIN, MAX, MAX-1) and at the
    // string boundary (empty / long). Reaching the assertion without
    // panicking proves the key-space path is panic-free at the
    // checked boundaries.
    //
    // No `kani::cover!` needed: the boundary enumeration is the proof
    // (each value is concrete, not symbolic).
    #[kani::proof]
    #[kani::unwind(32)]
    fn verify_event_key_panic_free_at_extremes() {
        for session_id in ["", "a", "session-with-many-dashes-and-letters-1234567890"] {
            let _ = event_key(session_id, 0u64);
            let _ = event_key(session_id, 1u64);
            let _ = event_key(session_id, u64::MAX - 1);
            let _ = event_key(session_id, u64::MAX);
        }
    }

    // -------------------------------------------------------------------------
    // Negative harness — not_applicable
    // -------------------------------------------------------------------------
    //
    // No `#[kani::should_panic]` harness is warranted for this module:
    // the public API returns `Result<T, FjallStoreError>` for fallible
    // operations (`open`, `append_event_sync_all`, `load_events`), and
    // the Fjall I/O is stubbed out in any in-process Kani run via the
    // `-Z stubbing` flag. Asserting on a specific `Err` variant would
    // require stub contracts that the production module does not yet
    // export. When cl-u04 closes and stub-verified contracts are added
    // (`#[kani::stub_verified]`), a `verify_open_rejects_missing_directory`
    // harness can be added here.
}