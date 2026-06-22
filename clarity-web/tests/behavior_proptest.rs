//! proptest properties for `clarity-web/src/intent/types/behavior.rs`.
//!
//! | Field | Value |
//! |---|---|
//! | Bead | `cl-ooz` |
//! | Target | `clarity-web/src/intent/types/behavior.rs` (236 LOC) |
//! | Primary lane | **P** (proptest) — per `verification-targets.md §5.1` |
//! | Secondary lane | **V** (Verus) — see `behavior_verus.rs` |
//!
//! # Wiring note
//!
//! This file lives at `proofs/behavior_proptest.rs` per the proof-writer brief.
//! To execute it as a `cargo test` integration test, copy or move the file to
//! `clarity-web/tests/behavior_proptest.rs`. The expected command then becomes:
//!
//! ```text
//! cargo test -p clarity-web --test behavior_proptest
//! ```
//!
//! Wiring requires no changes to production code; the existing
//! `[dev-dependencies] proptest = "1.10.0"` in `clarity-web/Cargo.toml` covers
//! the import. The `#![allow(...)]` line below matches the convention used by
//! other integration-test files in `clarity-web/tests/` — the workspace
//! `unwrap_used = "deny"` lint is intentionally relaxed in tests because
//! panicking on a failed assertion is the test-running contract.
//!
//! # Source mapping
//!
//! Each property cites `clarity-web/src/intent/types/behavior.rs:LINE` against the
//! production function it constrains.
//!
//! # Anti-verification-laundering
//!
//! Every property invokes the production API via `use clarity_web::intent::types::Behavior;`.
//! No property rewrites or shadows the production functions; no production
//! mutation; no test-of-a-test.
//!
//! # Trusted base
//!
//! | Trust | Why trusted | Mitigation |
//! |---|---|
//! | `serde_json` round-trip preserves values | Library contract; not our code | Each round-trip property re-parses the same JSON and asserts equality. |
//! | proptest's `proptest!` macro | Library contract; not our code | Standard proptest machinery; no custom shrinkers. |

#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::match_same_arms
)]

use proptest::prelude::*;

use clarity_web::intent::types::{Behavior, TypeError, Verification};

const MAX_PRECONDITIONS: usize = 20;
const MAX_POSTCONDITIONS: usize = 20;

// ============================================================
// Generators
// ============================================================

/// Generate a `snake_case` behavior name (lowercase letters, digits, underscores).
/// Source: `behavior.rs:17-25`.
fn arb_snake_case_name() -> impl Strategy<Value = String> {
  "[a-z][a-z0-9_]*".prop_filter_map("valid snake_case", |s| {
    if s.is_empty() || !s.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
      None
    } else {
      Some(s)
    }
  })
}

/// Generate an arbitrary description string (any UTF-8).
fn arb_description() -> impl Strategy<Value = String> {
  ".*".prop_filter_map("non-empty description", |s| {
    if s.is_empty() {
      Some("a".to_string())
    } else {
      Some(s)
    }
  })
}

/// Generate an arbitrary Verification.
fn arb_verification() -> impl Strategy<Value = Verification> {
  (arb_snake_case_name(), arb_description()).prop_map(|(t, d)| Verification::new(t, d))
}

/// Generate a Vec<String> of preconditions within [0, MAX+1] range.
/// MAX+1 = 21 exercises the boundary rejection in `validate()`.
fn arb_preconditions_vec() -> impl Strategy<Value = Vec<String>> {
  proptest::collection::vec(arb_description(), 0..=MAX_PRECONDITIONS + 1)
}

/// Generate a Vec<String> of postconditions within [0, MAX+1] range.
fn arb_postconditions_vec() -> impl Strategy<Value = Vec<String>> {
  proptest::collection::vec(arb_description(), 0..=MAX_POSTCONDITIONS + 1)
}

/// PO-P1: Generate a complete Behavior via the public API.
/// Uses `snake_case` names, arbitrary descriptions, optional verification,
/// and pre/postcondition vectors in [0, MAX+1] to exercise both
/// valid and boundary-rejected constructions.
fn arb_behavior() -> impl Strategy<Value = Behavior> {
  (
    arb_snake_case_name(),
    arb_description(),
    prop::option::of(arb_verification()),
    arb_preconditions_vec(),
    arb_postconditions_vec(),
  )
    .prop_map(
      |(name, desc, verification, preconditions, postconditions)| {
        let behavior = Behavior::new(name).expect("valid snake_case name");
        let behavior = behavior.with_description(desc);
        let behavior = match verification {
          Some(v) => behavior.with_verification(v),
          None => behavior,
        };
        let mut b = behavior;
        for pre in preconditions {
          b.add_precondition(pre);
        }
        for post in postconditions {
          b.add_postcondition(post);
        }
        b
      },
    )
}

proptest! {
    // ============================================================
    // Group A — Construction properties (PO-V2, PO-V3)
    // Source: behavior.rs:54-65
    // ============================================================

    /// Valid snake_case names always produce `Ok`.
    /// PO-V2: `Behavior::new(s).is_ok() ⇔ is_valid_behavior_name(&s)`
    /// Source: behavior.rs:54-65.
    #[test]
    fn prop_new_valid_name_always_ok(name in arb_snake_case_name()) {
        let result = Behavior::new(name.clone());
        prop_assert!(result.is_ok(), "valid name {:?} should produce Ok", name);
    }

    /// `Behavior::new` canonical-empty element: empty description, None verification,
    /// empty preconditions and postconditions.
    /// PO-V3.
    /// Source: behavior.rs:58-64.
    #[test]
    fn prop_new_canonical_empty(name in arb_snake_case_name()) {
        let b = Behavior::new(name.clone()).expect("valid name");
        prop_assert_eq!(b.name, name);
        prop_assert_eq!(b.description, "");
        prop_assert!(b.verification.is_none());
        prop_assert!(b.preconditions.is_empty());
        prop_assert!(b.postconditions.is_empty());
    }

    // ============================================================
    // Group B — Field-replace builder properties (PO-V6, PO-V7)
    // Source: behavior.rs:69-83
    // ============================================================

    /// `with_description` only changes description; all other fields preserved.
    /// PO-V6.
    /// Source: behavior.rs:69-74.
    #[test]
    fn prop_with_description_only_changes_description(
        name in arb_snake_case_name(),
        desc1 in arb_description(),
        desc2 in arb_description(),
    ) {
        let b1 = Behavior::new(name.clone()).expect("valid");
        let b1 = b1.with_description(desc1);
        let preconditions_before = b1.preconditions.clone();
        let postconditions_before = b1.postconditions.clone();
        let verification_before = b1.verification.clone();
        let b2 = b1.with_description(desc2.clone());
        prop_assert_eq!(b2.name, name);
        prop_assert_eq!(b2.description, desc2);
        prop_assert_eq!(b2.verification, verification_before);
        prop_assert_eq!(b2.preconditions, preconditions_before);
        prop_assert_eq!(b2.postconditions, postconditions_before);
    }

    /// `with_verification` only changes verification; all other fields preserved.
    /// PO-V7.
    /// Source: behavior.rs:78-83.
    #[test]
    fn prop_with_verification_only_changes_verification(
        name in arb_snake_case_name(),
        desc in arb_description(),
    ) {
        let b1 = Behavior::new(name.clone()).expect("valid");
        let b1 = b1.with_description(desc.clone());
        let preconditions_before = b1.preconditions.clone();
        let postconditions_before = b1.postconditions.clone();
        let v = Verification::new("type".to_string(), desc.clone());
        let b2 = b1.with_verification(v.clone());
        prop_assert_eq!(b2.name, name);
        prop_assert_eq!(b2.description, desc);
        prop_assert_eq!(b2.verification, Some(v));
        prop_assert_eq!(b2.preconditions, preconditions_before);
        prop_assert_eq!(b2.postconditions, postconditions_before);
    }

    // ============================================================
    // Group C — Append order and no-dedup (PO-V4, PO-V5, PO-V11, PO-V12)
    // Source: behavior.rs:86-95
    // ============================================================

    /// `add_precondition` appends exactly one element.
    /// PO-V4.
    /// Source: behavior.rs:86-89.
    #[test]
    fn prop_add_precondition_appends_one(
        name in arb_snake_case_name(),
        pre in arb_description(),
    ) {
        let mut b = Behavior::new(name).expect("valid");
        let len_before = b.preconditions.len();
        b.add_precondition(pre);
        prop_assert_eq!(b.preconditions.len(), len_before + 1);
    }

    /// `add_precondition` preserves all other fields.
    /// PO-V4.
    /// Source: behavior.rs:86-89.
    #[test]
    fn prop_add_precondition_preserves_other_fields(
        name in arb_snake_case_name(),
        desc in arb_description(),
    ) {
        let mut b = Behavior::new(name.clone()).expect("valid");
        b = b.with_description(desc.clone());
        let preconditions_before = b.preconditions.len();
        let postconditions_before = b.postconditions.len();
        let verification_before = b.verification.clone();

        b.add_precondition("condition".to_string());

        prop_assert_eq!(b.name, name);
        prop_assert_eq!(b.description, desc);
        prop_assert_eq!(b.verification, verification_before);
        prop_assert_eq!(b.postconditions.len(), postconditions_before);
        prop_assert_eq!(b.preconditions.len(), preconditions_before + 1);
    }

    /// `add_precondition` fluent chaining preserves order.
    /// PO-V11.
    /// Source: behavior.rs:86-89.
    #[test]
    fn prop_add_precondition_order_preserved(
        name in arb_snake_case_name(),
        x in arb_description(),
        y in arb_description(),
    ) {
        let mut b = Behavior::new(name).expect("valid");
        b.add_precondition(x.clone()).add_precondition(y.clone());
        prop_assert_eq!(b.preconditions.len(), 2);
        prop_assert_eq!(&b.preconditions[0], &x);
        prop_assert_eq!(&b.preconditions[1], &y);
    }

    /// `add_precondition` does not deduplicate.
    /// PO-V12.
    /// Source: behavior.rs:86-89.
    #[test]
    fn prop_add_precondition_no_dedup(name in arb_snake_case_name()) {
        let mut b = Behavior::new(name).expect("valid");
        let same = "same_condition".to_string();
        b.add_precondition(same.clone()).add_precondition(same.clone());
        prop_assert_eq!(b.preconditions.len(), 2);
        prop_assert_eq!(&b.preconditions[0], &same);
        prop_assert_eq!(&b.preconditions[1], &same);
    }

    /// `add_postcondition` appends exactly one element.
    /// PO-V5.
    /// Source: behavior.rs:92-95.
    #[test]
    fn prop_add_postcondition_appends_one(
        name in arb_snake_case_name(),
        post in arb_description(),
    ) {
        let mut b = Behavior::new(name).expect("valid");
        let len_before = b.postconditions.len();
        b.add_postcondition(post);
        prop_assert_eq!(b.postconditions.len(), len_before + 1);
    }

    /// Symmetric no-dedup for postconditions.
    #[test]
    fn prop_add_postcondition_no_dedup(name in arb_snake_case_name()) {
        let mut b = Behavior::new(name).expect("valid");
        let same = "same_postcondition".to_string();
        b.add_postcondition(same.clone()).add_postcondition(same.clone());
        prop_assert_eq!(b.postconditions.len(), 2);
        prop_assert_eq!(&b.postconditions[0], &same);
        prop_assert_eq!(&b.postconditions[1], &same);
    }

    // ============================================================
    // Group D — validate bounds and monotonicity (PO-V8, PO-V13)
    // Source: behavior.rs:101-117
    // ============================================================

    /// `validate` passes when preconditions and postconditions are within bounds.
    /// PO-V8.
    /// Source: behavior.rs:101-117.
    #[test]
    fn prop_validate_ok_within_bounds(
        name in arb_snake_case_name(),
    ) {
        let mut b = Behavior::new(name).expect("valid");
        for i in 0..20 {
            b.add_precondition(format!("pre_{i}"));
        }
        for i in 0..20 {
            b.add_postcondition(format!("post_{i}"));
        }
        prop_assert!(b.validate().is_ok());
    }

    /// `validate` fails when preconditions exceed 20.
    /// PO-V8 (error case).
    /// Source: behavior.rs:102-108.
    #[test]
    fn prop_validate_err_too_many_preconditions(
        name in arb_snake_case_name(),
    ) {
        let mut b = Behavior::new(name.clone()).expect("valid");
        for i in 0..21 {
            b.add_precondition(format!("pre_{i}"));
        }
        let result = b.validate();
        prop_assert!(result.is_err());
        if let Err(TypeError::TooManyPreconditions(n, count, max)) = result {
            prop_assert_eq!(n, name);
            prop_assert_eq!(count, 21);
            prop_assert_eq!(max, 20);
        }
    }

    /// `validate` fails when postconditions exceed 20.
    /// PO-V8 (error case).
    /// Source: behavior.rs:109-115.
    #[test]
    fn prop_validate_err_too_many_postconditions(
        name in arb_snake_case_name(),
    ) {
        let mut b = Behavior::new(name.clone()).expect("valid");
        for i in 0..21 {
            b.add_postcondition(format!("post_{i}"));
        }
        let result = b.validate();
        prop_assert!(result.is_err());
        if let Err(TypeError::TooManyPostconditions(n, count, max)) = result {
            prop_assert_eq!(n, name);
            prop_assert_eq!(count, 21);
            prop_assert_eq!(max, 20);
        }
    }

    /// PO-V13: validate monotonicity — adding one precondition keeps Ok when len < 20.
    /// Source: behavior.rs:101-117.
    #[test]
    fn prop_validate_monotonic_precondition_below_bound(
        name in arb_snake_case_name(),
    ) {
        let mut b = Behavior::new(name).expect("valid");
        for i in 0..19 {
            b.add_precondition(format!("pre_{i}"));
        }
        prop_assert!(b.validate().is_ok());
        b.add_precondition("pre_19".to_string());
        prop_assert_eq!(b.preconditions.len(), 20);
        prop_assert!(b.validate().is_ok());
    }

    /// PO-V13: validate monotonicity — adding one precondition flips to Err exactly at len == 21.
    /// Source: behavior.rs:101-117.
    #[test]
    fn prop_validate_monotonic_precondition_at_boundary(
        name in arb_snake_case_name(),
    ) {
        let mut b = Behavior::new(name.clone()).expect("valid");
        for i in 0..20 {
            b.add_precondition(format!("pre_{i}"));
        }
        prop_assert!(b.validate().is_ok());
        b.add_precondition("pre_20".to_string());
        prop_assert_eq!(b.preconditions.len(), 21);
        let result = b.validate();
        prop_assert!(result.is_err());
        if let Err(TypeError::TooManyPreconditions(n, 21, 20)) = result {
            prop_assert_eq!(n, name);
        }
    }

    // ============================================================
    // Group E — Serde round-trip (PO-P1 / REQ-BH-14)
    // Source: behavior.rs:28-44
    // ============================================================

    /// `Behavior` round-trips through serde_json without losing identity.
    /// PO-P1 (REQ-BH-14).
    /// Source: behavior.rs:28-44 (Serialize + Deserialize derives).
    #[test]
    fn prop_behavior_serde_roundtrip(b in arb_behavior()) {
        let json_result = serde_json::to_string(&b);
        prop_assert!(json_result.is_ok(), "serialization should succeed");
        let json = json_result.ok().unwrap();
        let back_result = serde_json::from_str::<Behavior>(&json);
        prop_assert!(back_result.is_ok(), "deserialization should succeed");
        let back = back_result.ok().unwrap();
        prop_assert_eq!(back.name, b.name);
        prop_assert_eq!(back.description, b.description);
        prop_assert_eq!(back.verification, b.verification);
        prop_assert_eq!(back.preconditions, b.preconditions);
        prop_assert_eq!(back.postconditions, b.postconditions);
    }

    /// Serde round-trip preserves validate() outcome.
    /// Source: behavior.rs:101-117.
    #[test]
    fn prop_serde_roundtrip_preserves_validate(b in arb_behavior()) {
        let json_result = serde_json::to_string(&b);
        let json = json_result.ok().unwrap();
        let back: Behavior = serde_json::from_str(&json).ok().unwrap();
        prop_assert_eq!(b.validate().is_ok(), back.validate().is_ok());
    }
}
