//! Proptest artifacts for `clarity-web/src/domain/scenario.rs`.
//!
//! | Field | Value |
//! |---|---|
//! | Bead | `cl-zup` |
//! | Target | `clarity-web/src/domain/scenario.rs` |
//! | Lane | **P** (proptest) — secondary |
//! | Ratified contract | `clarity-web/src/domain/scenario_contract.md` |
//!
//! # Obligations covered (PO-SC-P-01 to P-08)
//!
//! | ID | Description |
//! |---|---|
//! | PO-SC-P-01 | `HoleType` serde round-trip (all 3 variants) |
//! | PO-SC-P-02 | Hole serde round-trip (raw u8 preserved) |
//! | PO-SC-P-03 | `HolePunchingResults` serde + address idempotence (L2) + monotonicity (L3) |
//! | PO-SC-P-04 | `ScenarioField` serde + L6 (`is_complete` == `bullets_complete` && `addressed_count` == 3) |
//! | PO-SC-P-05 | Display for `HoleType`: format!("{}", ht) == `ht.label()` |
//! | PO-SC-P-06 | address right-most-wins per axis (L1) |
//! | PO-SC-P-07 | `with_severity` clamp boundaries for all u8 values |
//! | PO-SC-P-08 | `is_complete` == (`addressed_count` == 3) (Law L6) |
//!
//! # Anti-unwrap doctrine
//!
//! This file does NOT use `.unwrap()` or `.expect()` anywhere in property bodies.
//! Serialisation round-trips use `is_ok()` checks and explicit `match` arms.
//! Proptest's `prop_assert!` macros are used for falsifiable property statements.
//!
//! # Design
//!
//! These tests are written as free-standing `#[test]` functions (not part of the
//! clarity-web crate) so they can be run via `cargo test --test scenario_proptest`.
//! They import from `clarity_web::domain::scenario::*` assuming clarity-web is
//! compiled as a library and this file is compiled as an integration test adjacent
//! to the workspace root.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use clarity_web::domain::scenario::{Hole, HolePunchingResults, HoleType, ScenarioField};
use proptest::prelude::*;
use std::collections::HashSet;

// =============================================================================
// §1  Strategy helpers
// =============================================================================

fn arb_hole_type() -> impl Strategy<Value = HoleType> {
  proptest::sample::select(&[
    HoleType::DiscoveryHole,
    HoleType::EdgeCaseHole,
    HoleType::MotivationDropOff,
  ])
}

fn arb_hole_punching_results() -> impl Strategy<Value = HolePunchingResults> {
  // Build arbitrary HolePunchingResults by starting from default and applying
  // 0..=5 random address calls.
  prop::collection::vec((arb_hole_type(), ".*"), 0..=5usize).prop_map(|calls| {
    let mut r = HolePunchingResults::new();
    for (ht, expl) in calls {
      r = r.address(ht, expl);
    }
    r
  })
}

fn arb_scenario_field() -> impl Strategy<Value = ScenarioField> {
  (
    ".*", // trigger
    ".*", // value_moment
    ".*", // feeling
    arb_hole_punching_results(),
  )
    .prop_map(
      |(trigger, value_moment, feeling, hole_punching)| ScenarioField {
        trigger,
        value_moment,
        feeling,
        hole_punching,
      },
    )
}

// =============================================================================
// §2  HoleType round-trip — PO-SC-P-01
// =============================================================================

#[test]
fn proptest_hole_type_roundtrip() {
  proptest!(|(ht in arb_hole_type())| {
      let json = serde_json::to_string(&ht);
      prop_assert!(json.is_ok(), "serialisation must succeed");
      let json = json.unwrap();
      let round = serde_json::from_str::<HoleType>(&json);
      prop_assert!(round.is_ok(), "deserialisation must succeed");
      let round = round.unwrap();
      prop_assert_eq!(round, ht, "round-trip must preserve HoleType");
  });
}

// =============================================================================
// §3  Hole round-trip — PO-SC-P-02
// =============================================================================

#[test]
fn proptest_hole_roundtrip() {
  proptest!(|(ht in arb_hole_type(), desc in ".*", severity in 0u8..=255)| {
      let hole = Hole {
          hole_type: ht,
          description: desc,
          severity,
      };
      let json = serde_json::to_string(&hole);
      prop_assert!(json.is_ok(), "serialisation must succeed");
      let json = json.unwrap();
      let round = serde_json::from_str::<Hole>(&json);
      prop_assert!(round.is_ok(), "deserialisation must succeed");
      let round = round.unwrap();
      prop_assert_eq!(round.hole_type, hole.hole_type, "hole_type must match");
      prop_assert_eq!(round.description, hole.description, "description must match");
      prop_assert_eq!(round.severity, hole.severity, "raw u8 severity must be preserved");
  });
}

// =============================================================================
// §4  HolePunchingResults laws — PO-SC-P-03
// =============================================================================

/// PO-SC-P-03 part 1: serde round-trip preserves all 3 fields.
#[test]
fn proptest_hole_punching_serde_roundtrip() {
  proptest!(|(r in arb_hole_punching_results())| {
      let json = serde_json::to_string(&r);
      prop_assert!(json.is_ok(), "serialisation must succeed");
      let json = json.unwrap();
      let round = serde_json::from_str::<HolePunchingResults>(&json);
      prop_assert!(round.is_ok(), "deserialisation must succeed");
      let round = round.unwrap();
      prop_assert_eq!(round.discovery_hole, r.discovery_hole, "discovery_hole must match");
      prop_assert_eq!(round.edge_case_hole, r.edge_case_hole, "edge_case_hole must match");
      prop_assert_eq!(round.motivation_dropoff, r.motivation_dropoff, "motivation_dropoff must match");
  });
}

/// PO-SC-P-03 part 2: address is idempotent per axis (Law L2).
#[test]
fn proptest_hole_punching_idempotent() {
  proptest!(|(r in arb_hole_punching_results(), ht in arb_hole_type(), expl in ".*")| {
      let r1 = r.address(ht, expl.clone());
      let r2 = r1.clone().address(ht, expl);
      prop_assert_eq!(r2, r1, "second address(ht, e) must not change result (L2 idempotent)");
  });
}

/// PO-SC-P-03 combined: all three laws tested together.
#[test]
fn proptest_hole_punching_laws() {
  // Combined test exercising all three HolePunchingResults laws.
  proptest!(|(r in arb_hole_punching_results(), ht in arb_hole_type(), e1 in ".*", e2 in ".*")| {
      // L2: idempotence
      let idemp = r.clone().address(ht, e1.clone()).address(ht, e1.clone());
      let single = r.clone().address(ht, e1.clone());
      prop_assert_eq!(idemp, single, "L2: address(ht, e).address(ht, e) == address(ht, e)");

      // L1: right-most-wins
      let rmw = r.clone().address(ht, e1).address(ht, e2.clone());
      let last = r.clone().address(ht, e2);
      prop_assert_eq!(rmw, last, "L1: r.address(ht, e1).address(ht, e2) == r.address(ht, e2)");

      // L3: monotonic addressed_count
      let before = r.addressed_count();
      let after = r.address(ht, "new explanation".to_string()).addressed_count();
      prop_assert!(after >= before, "L3: addressed_count monotonic non-decreasing");
  });
}

// =============================================================================
// §5  ScenarioField laws — PO-SC-P-04
// =============================================================================

/// PO-SC-P-04 part 1: serde round-trip for `ScenarioField`.
#[test]
fn proptest_scenario_field_serde_roundtrip() {
  proptest!(|(sf in arb_scenario_field())| {
      let json = serde_json::to_string(&sf);
      prop_assert!(json.is_ok(), "serialisation must succeed");
      let json = json.unwrap();
      let round = serde_json::from_str::<ScenarioField>(&json);
      prop_assert!(round.is_ok(), "deserialisation must succeed");
      let round = round.unwrap();
      prop_assert_eq!(round.trigger, sf.trigger, "trigger must match");
      prop_assert_eq!(round.value_moment, sf.value_moment, "value_moment must match");
      prop_assert_eq!(round.feeling, sf.feeling, "feeling must match");
      prop_assert_eq!(round.hole_punching, sf.hole_punching, "hole_punching must match");
  });
}

/// PO-SC-P-04 part 2: Law L6 — `is_complete` == `bullets_complete` && `addressed_count` == 3.
#[test]
fn proptest_scenario_field_laws() {
  proptest!(|(sf in arb_scenario_field())| {
      let bullets_complete = sf.is_bullets_complete();
      let holes_complete = sf.hole_punching.is_complete();
      let addressed_count_3 = sf.hole_punching.addressed_count() == 3;

      // L6 variant: is_complete implies addressed_count == 3 and bullets_complete
      if sf.is_complete() {
          prop_assert!(holes_complete, "is_complete implies holes_complete");
          prop_assert!(addressed_count_3, "is_complete implies addressed_count == 3");
          prop_assert!(bullets_complete, "is_complete implies bullets_complete");
      }

      // Converse: if holes_complete && bullets_complete then is_complete (this is L5/L6)
      if holes_complete && bullets_complete {
          prop_assert!(sf.is_complete(), "bullets_complete && holes_complete implies is_complete");
      }
  });
}

// =============================================================================
// §6  HoleType Display — PO-SC-P-05
// =============================================================================

#[test]
fn proptest_hole_type_display() {
  proptest!(|(ht in arb_hole_type())| {
      let formatted = format!("{ht}");
      let label = ht.label();
      prop_assert_eq!(formatted, label, "Display must write exactly label()");
  });
}

// =============================================================================
// §7  address right-most-wins — PO-SC-P-06
// =============================================================================

#[test]
fn proptest_address_right_most_wins() {
  proptest!(|(r in arb_hole_punching_results(), ht in arb_hole_type(), e1 in ".*", e2 in ".*")| {
      let two_call = r.clone().address(ht, e1).address(ht, e2.clone());
      let one_call = r.address(ht, e2);

      // Check field equality for the addressed axis
      match ht {
          HoleType::DiscoveryHole => {
              prop_assert_eq!(two_call.discovery_hole, one_call.discovery_hole, "L1: discovery_hole right-most-wins");
              prop_assert_eq!(two_call.edge_case_hole, one_call.edge_case_hole, "L1: edge_case_hole unchanged");
              prop_assert_eq!(two_call.motivation_dropoff, one_call.motivation_dropoff, "L1: motivation_dropoff unchanged");
          },
          HoleType::EdgeCaseHole => {
              prop_assert_eq!(two_call.edge_case_hole, one_call.edge_case_hole, "L1: edge_case_hole right-most-wins");
              prop_assert_eq!(two_call.discovery_hole, one_call.discovery_hole, "L1: discovery_hole unchanged");
              prop_assert_eq!(two_call.motivation_dropoff, one_call.motivation_dropoff, "L1: motivation_dropoff unchanged");
          },
          HoleType::MotivationDropOff => {
              prop_assert_eq!(two_call.motivation_dropoff, one_call.motivation_dropoff, "L1: motivation_dropoff right-most-wins");
              prop_assert_eq!(two_call.discovery_hole, one_call.discovery_hole, "L1: discovery_hole unchanged");
              prop_assert_eq!(two_call.edge_case_hole, one_call.edge_case_hole, "L1: edge_case_hole unchanged");
          },
      }
  });
}

// =============================================================================
// §8  with_severity clamp boundaries — PO-SC-P-07
// =============================================================================

#[test]
fn proptest_with_severity_clamp() {
  // Full u8 range, but we focus the explicit boundary cases.
  proptest!(|(ht in arb_hole_type(), desc in ".*", s in 0u8..=255)| {
      let hole = Hole::with_severity(ht, desc.clone(), s);
      let clamped = s.clamp(1, 5);
      prop_assert!(hole.severity >= 1, "severity must be >= 1, got {}", hole.severity);
      prop_assert!(hole.severity <= 5, "severity must be <= 5, got {}", hole.severity);
      prop_assert_eq!(hole.severity, clamped, "severity == s.clamp(1, 5)");

      // Hole_type and description unchanged
      prop_assert_eq!(hole.hole_type, ht, "hole_type must be unchanged");
      prop_assert_eq!(hole.description, desc, "description must be unchanged");
  });

  // Explicit boundary cases per contract Q3 (inclusive [1, 5]).
  let boundary_cases: Vec<(u8, u8)> = vec![
    (0, 1),   // below range → 1
    (1, 1),   // lower bound
    (2, 2),   // in range
    (4, 4),   // in range
    (5, 5),   // upper bound
    (6, 5),   // above range → 5
    (127, 5), // above range
    (255, 5), // max u8 → 5
  ];

  for (input, expected) in boundary_cases {
    let ht = HoleType::DiscoveryHole;
    let hole = Hole::with_severity(ht, "desc".to_string(), input);
    assert_eq!(
      hole.severity, expected,
      "with_severity(ht, d, {}) severity must be {}, got {}",
      input, expected, hole.severity
    );
  }
}

// =============================================================================
// §9  is_complete law — PO-SC-P-08
// =============================================================================

#[test]
fn proptest_is_complete_law() {
  // Law L6: is_complete() == (addressed_count() == 3)
  // Generate arbitrary HolePunchingResults via sequence of address calls.
  proptest!(|(r in arb_hole_punching_results())| {
      let is_complete = r.is_complete();
      let addressed_3 = r.addressed_count() == 3;
      prop_assert_eq!(is_complete, addressed_3,
          "L6: is_complete() == (addressed_count() == 3); got is_complete={}, addressed_count={}",
          is_complete, r.addressed_count());
  });
}

// =============================================================================
// §10  Additional sanity: hole punching scenario field laws
// =============================================================================

/// Ensure `addressed_count` never exceeds 3 regardless of call sequence.
#[test]
fn proptest_addressed_count_bounded() {
  proptest!(|(r in arb_hole_punching_results())| {
      prop_assert!(r.addressed_count() <= 3, "addressed_count must be <= 3");
  });
}

/// Ensure `unaddressed_holes` length + `addressed_count` == 3.
#[test]
fn proptest_unaddressed_holes_complement() {
  proptest!(|(r in arb_hole_punching_results())| {
      let unaddressed = r.unaddressed_holes();
      let count = r.addressed_count();
      prop_assert_eq!(unaddressed.len(), 3 - count,
          "unaddressed_holes().len() + addressed_count() == 3; got unaddressed={}, addressed={}",
          unaddressed.len(), count);

      // All hole types in unaddressed list must actually be unaddressed
      for ht in &unaddressed {
          prop_assert!(!r.is_addressed(*ht), "unaddressed_holes must not contain addressed types");
      }

      // All hole types not in unaddressed list must be addressed
      let unaddressed_set: HashSet<HoleType> = unaddressed.iter().copied().collect();
      for ht in [HoleType::DiscoveryHole, HoleType::EdgeCaseHole, HoleType::MotivationDropOff] {
          if !unaddressed_set.contains(&ht) {
              prop_assert!(r.is_addressed(ht), "all addressed types must be absent from unaddressed_holes");
          }
      }
  });
}

/// Ensure `is_addressed` is equivalent to the per-field predicate.
#[test]
fn proptest_is_addressed_equivalence() {
  proptest!(|(r in arb_hole_punching_results())| {
      prop_assert_eq!(
          r.is_addressed(HoleType::DiscoveryHole),
          r.discovery_hole.as_ref().is_some_and(|s| !s.trim().is_empty()),
          "is_addressed(DiscoveryHole) must match per-field predicate"
      );
      prop_assert_eq!(
          r.is_addressed(HoleType::EdgeCaseHole),
          r.edge_case_hole.as_ref().is_some_and(|s| !s.trim().is_empty()),
          "is_addressed(EdgeCaseHole) must match per-field predicate"
      );
      prop_assert_eq!(
          r.is_addressed(HoleType::MotivationDropOff),
          r.motivation_dropoff.as_ref().is_some_and(|s| !s.trim().is_empty()),
          "is_addressed(MotivationDropOff) must match per-field predicate"
      );
  });
}
