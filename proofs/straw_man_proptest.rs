//! proptest properties for `clarity-web/src/domain/straw_man.rs`.
//!
//! | Field | Value |
//! |---|---|
//! | Bead | `cl-vv2` |
//! | Target | `clarity-web/src/domain/straw_man.rs` (303 LOC) |
//! | Primary lane | **P** (proptest) — per `verification-targets.md §5.1` |
//! | Secondary lane | **V** (Verus) — see `straw_man_verus.rs` |
//!
//! # Wiring note
//!
//! This file lives at `proofs/straw_man_proptest.rs` per the proof-writer brief.
//! To execute it as a `cargo test` integration test, copy or move the file to
//! `clarity-web/tests/straw_man_proptest.rs`. The expected command then becomes:
//!
//! ```text
//! cargo test -p clarity-web --test straw_man_proptest
//! ```
//!
//! Wiring requires no changes to production code; the existing
//! `[dev-dependencies] proptest = "1.10.0"` in `clarity-web/Cargo.toml` covers
//! the import. The `#![allow(...)]` line below matches the convention used by
//! the 15 other integration-test files in `clarity-web/tests/` — the workspace
//! `unwrap_used = "deny"` lint is intentionally relaxed in tests because
//! panicking on a failed assertion is the test-running contract.
//!
//! # Source mapping
//!
//! Each property cites `clarity-web/src/domain/straw_man.rs:LINE` against the
//! production function it constrains.
//!
//! # Anti-verification-laundering
//!
//! Every property invokes the production API via `use clarity_web::domain::straw_man::*;`.
//! No property rewrites or shadows the production functions; no production
//! mutation; no test-of-a-test.
//!
//! # Trusted base
//!
//! | Trust | Why trusted | Mitigation |
//! |---|---|---|
//! | `serde_json` round-trip preserves values | Library contract; not our code | Each round-trip property re-parses the same JSON and asserts equality. |
//! | proptest's `proptest!` macro | Library contract; not our code | Standard proptest machinery; no custom shrinkers. |
//! | `prop_oneof!` over `StrawManTrap` | Library contract; not our code | Enumerates 4 variants explicitly to mirror the production enum. |

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::needless_collect,
    clippy::match_same_arms
)]

use proptest::prelude::*;

use clarity_web::domain::straw_man::{StrawManTrap, StrawManValidation};

// ============================================================
// Generators
// ============================================================

/// Generate one of the four `StrawManTrap` variants uniformly.
/// Mirrors the production enum at `straw_man.rs:15-31`.
fn arb_trap() -> impl Strategy<Value = StrawManTrap> {
    prop_oneof![
        Just(StrawManTrap::IrrationalActor),     // straw_man.rs:18
        Just(StrawManTrap::ManicPixieDreamUser), // straw_man.rs:22
        Just(StrawManTrap::StoicMonk),           // straw_man.rs:26
        Just(StrawManTrap::YourClone),           // straw_man.rs:30
    ]
}

/// Generate any `Vec<StrawManTrap>`, including the empty vec.
/// Mirrors the production field `traps_detected: Vec<StrawManTrap>`
/// at `straw_man.rs:98`.
fn arb_trap_vec() -> impl Strategy<Value = Vec<StrawManTrap>> {
    proptest::collection::vec(arb_trap(), 0..16)
}

/// Generate a `StrawManValidation` built via the production `new` constructor
/// from a generator over arbitrary trap lists. The constructor at
/// `straw_man.rs:108-114` is the single canonical entry point.
fn arb_validation_via_new() -> impl Strategy<Value = StrawManValidation> {
    arb_trap_vec().prop_map(StrawManValidation::new)
}

proptest! {
    // ============================================================
    // Group A — `StrawManTrap::all()` enumeration
    // Source: straw_man.rs:36-43
    // ============================================================

    /// `all()` returns exactly 4 elements.
    /// Source: straw_man.rs:36-43 (postcondition); straw_man.rs:171-178 (test).
    #[test]
    fn prop_all_has_four_elements(_unused: ()) {
        let all = StrawManTrap::all();
        prop_assert_eq!(all.len(), 4);
    }

    /// `all()` contains every variant of `StrawManTrap` exactly once.
    /// Source: straw_man.rs:36-43; `test_straw_man_trap_all_returns_four_variants`.
    #[test]
    fn prop_all_contains_every_variant_once(_unused: ()) {
        let all = StrawManTrap::all();
        for variant in [
            StrawManTrap::IrrationalActor,
            StrawManTrap::ManicPixieDreamUser,
            StrawManTrap::StoicMonk,
            StrawManTrap::YourClone,
        ] {
            prop_assert_eq!(all.iter().filter(|t| **t == variant).count(), 1);
        }
    }

    // ============================================================
    // Group B — String-content invariants on label/description/checkbox_label
    // Source: straw_man.rs:47-88
    // ============================================================

    /// Every trap's label is non-empty.
    /// Source: straw_man.rs:47-54; unit test straw_man.rs:182-189.
    #[test]
    fn prop_label_is_nonempty(_unused: ()) {
        for trap in StrawManTrap::all() {
            prop_assert!(!trap.label().is_empty());
        }
    }

    /// Every trap's description is non-empty and detailed (>20 chars).
    /// Source: straw_man.rs:58-77; unit test straw_man.rs:192-204.
    /// Note: this contract is currently enforced only by tests at line 200;
    /// the Verus spec in `straw_man_verus.rs` upgrades it to a spec-level invariant.
    #[test]
    fn prop_description_is_detailed(_unused: ()) {
        for trap in StrawManTrap::all() {
            let desc = trap.description();
            prop_assert!(!desc.is_empty());
            prop_assert!(desc.len() > 20, "description too short for {trap:?}: {desc}");
        }
    }

    /// Every trap's checkbox label is a question (non-empty, ends with '?').
    /// Source: straw_man.rs:81-88; unit test straw_man.rs:207-219.
    #[test]
    fn prop_checkbox_label_is_a_question(_unused: ()) {
        for trap in StrawManTrap::all() {
            let cbl = trap.checkbox_label();
            prop_assert!(!cbl.is_empty());
            prop_assert!(cbl.ends_with('?'), "checkbox label not a question for {trap:?}: {cbl}");
        }
    }

    // ============================================================
    // Group C — Invariant preservation under construction
    // Source: straw_man.rs:108-114 (new), straw_man.rs:118-123 (passing)
    // ============================================================

    /// `new(traps)` always produces a result where `passed == traps.is_empty()`.
    /// This is the **core consistency invariant** of `StrawManValidation`.
    /// Source: straw_man.rs:108-114; unit test straw_man.rs:260-279.
    #[test]
    fn prop_new_preserves_passed_matches_empty(traps in arb_trap_vec()) {
        let v = StrawManValidation::new(traps.clone());
        prop_assert_eq!(v.passed, traps.is_empty());
        prop_assert!(v.is_valid());
    }

    /// `passing()` always yields `passed == true` and an empty trap list.
    /// Source: straw_man.rs:118-123; unit test straw_man.rs:230-235.
    #[test]
    fn prop_passing_is_passing(_unused: ()) {
        let v = StrawManValidation::passing();
        prop_assert!(v.passed);
        prop_assert!(v.traps_detected.is_empty());
        prop_assert_eq!(v.trap_count(), 0);
        prop_assert!(v.is_valid());
    }

    /// `Default::default()` equals `passing()`.
    /// Source: straw_man.rs:145-149 (Default impl) delegates to `passing()`.
    #[test]
    fn prop_default_equals_passing(_unused: ()) {
        let d: StrawManValidation = Default::default();
        let p = StrawManValidation::passing();
        prop_assert_eq!(d.passed, p.passed);
        prop_assert_eq!(d.traps_detected, p.traps_detected);
        prop_assert!(d.is_valid());
    }

    /// `new(vec![])` equals `passing()` in observable shape.
    /// Source: straw_man.rs:108-123 (both constructors).
    #[test]
    fn prop_new_empty_equals_passing(_unused: ()) {
        let n = StrawManValidation::new(vec![]);
        let p = StrawManValidation::passing();
        prop_assert_eq!(n.passed, p.passed);
        prop_assert_eq!(n.traps_detected, p.traps_detected);
        prop_assert_eq!(n.trap_count(), p.trap_count());
    }

    // ============================================================
    // Group D — Membership / count consistency
    // Source: straw_man.rs:127-135
    // ============================================================

    /// `has_trap(t)` returns true iff `t` is in the trap list.
    /// Source: straw_man.rs:127-129; unit test straw_man.rs:249-257.
    #[test]
    fn prop_has_trap_iff_member(
        traps in arb_trap_vec(),
        candidate in arb_trap(),
    ) {
        let v = StrawManValidation::new(traps.clone());
        prop_assert_eq!(v.has_trap(candidate), traps.contains(&candidate));
    }

    /// `trap_count()` returns `traps_detected.len()`.
    /// Source: straw_man.rs:133-135; unit test straw_man.rs:243-244.
    #[test]
    fn prop_trap_count_equals_len(traps in arb_trap_vec()) {
        let v = StrawManValidation::new(traps.clone());
        prop_assert_eq!(v.trap_count(), traps.len());
    }

    /// `is_valid()` returns true iff `passed == traps.is_empty()`.
    /// Source: straw_man.rs:140-142; unit test straw_man.rs:260-279.
    #[test]
    fn prop_is_valid_iff_invariant_holds(traps in arb_trap_vec()) {
        let v = StrawManValidation::new(traps);
        prop_assert!(v.is_valid());
    }

    /// `has_trap` and `trap_count` consistency: if `has_trap(t)` is true, then
    /// `trap_count() >= 1`. (Not strictly equal because of duplicates.)
    /// Source: straw_man.rs:127-135.
    #[test]
    fn prop_has_trap_implies_count_positive(
        trap in arb_trap(),
    ) {
        let mut traps = Vec::new();
        traps.push(trap);
        let v = StrawManValidation::new(traps);
        prop_assert!(v.has_trap(trap));
        prop_assert!(v.trap_count() >= 1);
    }

    // ============================================================
    // Group E — Order / shuffle invariance
    // Source: straw_man.rs:108-142 (no order assumptions on trap list)
    // ============================================================

    /// `has_trap(t)` is independent of element order in `traps_detected`.
    /// Source: straw_man.rs:127-129; the function uses `Vec::contains` which
    /// is order-independent.
    #[test]
    fn prop_has_trap_is_order_invariant(
        traps in arb_trap_vec(),
        candidate in arb_trap(),
    ) {
        let v1 = StrawManValidation::new(traps.clone());
        let mut reversed = traps.clone();
        reversed.reverse();
        let v2 = StrawManValidation::new(reversed);
        prop_assert_eq!(v1.has_trap(candidate), v2.has_trap(candidate));
    }

    /// `trap_count()` is independent of element order.
    /// Source: straw_man.rs:133-135.
    #[test]
    fn prop_trap_count_is_order_invariant(traps in arb_trap_vec()) {
        let v1 = StrawManValidation::new(traps.clone());
        let mut reversed = traps.clone();
        reversed.reverse();
        let v2 = StrawManValidation::new(reversed);
        prop_assert_eq!(v1.trap_count(), v2.trap_count());
    }

    /// `passed` is independent of element order.
    /// Source: straw_man.rs:108-114 (`passed = traps.is_empty()`).
    #[test]
    fn prop_passed_is_order_invariant(traps in arb_trap_vec()) {
        let v1 = StrawManValidation::new(traps.clone());
        let mut reversed = traps.clone();
        reversed.reverse();
        let v2 = StrawManValidation::new(reversed);
        prop_assert_eq!(v1.passed, v2.passed);
    }

    // ============================================================
    // Group F — Duplicate invariance
    // Source: straw_man.rs:108-135
    // ============================================================

    /// `has_trap(t)` ignores duplicates in `traps_detected`.
    /// Source: straw_man.rs:127-129.
    #[test]
    fn prop_has_trap_ignores_duplicates(trap in arb_trap()) {
        let traps = vec![trap, trap, trap];
        let v = StrawManValidation::new(traps);
        prop_assert!(v.has_trap(trap));
        prop_assert_eq!(v.trap_count(), 3); // duplicates preserved in count
    }

    /// Duplicates do not change `passed`.
    /// Source: straw_man.rs:108-114.
    #[test]
    fn prop_duplicates_dont_change_passed(trap in arb_trap()) {
        let traps = vec![trap, trap, trap];
        let v = StrawManValidation::new(traps);
        prop_assert!(!v.passed);
    }

    // ============================================================
    // Group G — Serde round-trip (library contract, X lane)
    // The proptest checks exercise the production serde derives from
    // `straw_man.rs:14, 95`. serde is the X lane (exercise-only); the
    // property here guards against accidental encoding breakage.
    // Source: unit tests straw_man.rs:282-303; proptest wrapper below.
    // ============================================================

    /// `StrawManTrap` round-trips through serde_json without losing identity.
    /// Source: straw_man.rs:14 (derive), straw_man.rs:282-289 (unit test).
    #[test]
    fn prop_trap_serde_roundtrip(trap in arb_trap()) {
        let json = serde_json::to_string(&trap).expect("serialize");
        let back: StrawManTrap = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(back, trap);
    }

    /// `StrawManValidation` round-trips through serde_json, preserving
    /// trap list, passed flag, and the invariant.
    /// Source: straw_man.rs:95 (derive), straw_man.rs:292-303 (unit test).
    #[test]
    fn prop_validation_serde_roundtrip(v in arb_validation_via_new()) {
        let json = serde_json::to_string(&v).expect("serialize");
        let back: StrawManValidation = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(back.passed, v.passed);
        prop_assert_eq!(back.traps_detected, v.traps_detected);
        prop_assert_eq!(back.trap_count(), v.trap_count());
        prop_assert!(back.is_valid());
    }

    // ============================================================
    // Group H — Closure: invariants under arbitrary constructor input
    // ============================================================

    /// Any `StrawManValidation` produced by the public API satisfies `is_valid()`.
    /// This is the master invariant: any externally-observable `StrawManValidation`
    /// value built via `new`, `passing`, or `Default::default` must have
    /// `passed == traps_detected.is_empty()`.
    /// Source: straw_man.rs:108-149.
    #[test]
    fn prop_any_public_construction_is_valid(
        // Build any value via one of the three public constructors.
        choice in 0u8..3,
        traps in arb_trap_vec(),
    ) {
        let v = match choice {
            0 => StrawManValidation::new(traps),
            1 => StrawManValidation::passing(),
            _ => StrawManValidation::default(),
        };
        prop_assert!(v.is_valid(), "invariant violated by public-API construction");
    }
}
