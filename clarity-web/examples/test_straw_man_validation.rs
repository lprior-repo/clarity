// REDQUEEN: Coevolutionary adversarial tests for StrawManValidation
//
// Attacks:
// 1. Invariant integrity: passed <-> traps_detected.is_empty() under all constructor paths
// 2. Hash/Eq consistency: derived traits must agree on StrawManTrap
// 3. Serialization roundtrip: serde must be bijective
// 4. Order independence: semantic operations must be commutative
// 5. Exhaustive property-based: all 2^4 subsets checked against 5 properties
// 6. Edge cases: empty, duplicates, single-trap exhaustiveness
// 7. Clone independence: cloned values fully independent
// 8. Thread safety: Send + Sync bounds verified at compile time

use std::collections::HashSet;

use clarity_web::components::discover::straw_man::{StrawManTrap, StrawManValidation};

fn main() {
  println!("REDQUEEN: Coevolutionary adversarial testing against StrawManValidation\n");

  // ============================================================
  // ATTACK 1: Invariant integrity
  // ============================================================
  {
    println!("ATTACK 1: Invariant integrity...");

    let cases: Vec<Vec<StrawManTrap>> = vec![
      vec![],
      vec![StrawManTrap::IrrationalActor],
      vec![StrawManTrap::StoicMonk],
      vec![StrawManTrap::YourClone],
      vec![StrawManTrap::ManicPixieDreamUser],
      vec![StrawManTrap::IrrationalActor, StrawManTrap::YourClone],
      vec![
        StrawManTrap::IrrationalActor,
        StrawManTrap::ManicPixieDreamUser,
        StrawManTrap::StoicMonk,
        StrawManTrap::YourClone,
      ],
    ];

    for traps in cases {
      let v = StrawManValidation::new(traps.clone());
      assert_eq!(
        v.passed,
        traps.is_empty(),
        "INVARIANT BROKEN: passed={} but traps.is_empty()={} for traps={:?}",
        v.passed,
        traps.is_empty(),
        traps
      );
      assert!(
        v.is_valid(),
        "is_valid() must hold for all constructor-created instances: traps={:?}",
        traps
      );
    }

    let p = StrawManValidation::passing();
    assert!(p.is_valid());
    assert!(p.passed);
    assert!(p.traps_detected.is_empty());

    let d = StrawManValidation::default();
    assert!(d.is_valid());
    assert_eq!(d, StrawManValidation::passing());

    println!("  PASSED");
  }

  // ============================================================
  // ATTACK 2: Hash/Eq consistency
  // ============================================================
  {
    println!("ATTACK 2: Hash/Eq consistency...");

    let a = StrawManValidation::new(vec![StrawManTrap::IrrationalActor]);
    let b = StrawManValidation::new(vec![StrawManTrap::IrrationalActor]);
    assert_eq!(a, b);
    // NOTE: StrawManValidation does NOT implement Hash (only Eq).
    // This is a design observation — HashSet<StrawManValidation> is not possible.

    // Vec equality is order-sensitive
    let c = StrawManValidation::new(vec![StrawManTrap::YourClone, StrawManTrap::IrrationalActor]);
    let d = StrawManValidation::new(vec![StrawManTrap::IrrationalActor, StrawManTrap::YourClone]);
    assert_ne!(c, d, "Vec equality is order-sensitive");
    assert!(c.is_valid());
    assert!(d.is_valid());

    // StrawManTrap Hash/Eq
    let t1 = StrawManTrap::StoicMonk;
    let t2 = StrawManTrap::StoicMonk;
    assert_eq!(t1, t2);
    let mut trap_set = HashSet::new();
    trap_set.insert(t1);
    assert!(trap_set.contains(&t2));

    // All 4 variants distinct
    let all_variants = [
      StrawManTrap::IrrationalActor,
      StrawManTrap::ManicPixieDreamUser,
      StrawManTrap::StoicMonk,
      StrawManTrap::YourClone,
    ];
    for i in 0..all_variants.len() {
      for j in (i + 1)..all_variants.len() {
        assert_ne!(all_variants[i], all_variants[j]);
      }
    }

    println!("  PASSED");
  }

  // ============================================================
  // ATTACK 3: Serialization roundtrip
  // ============================================================
  {
    println!("ATTACK 3: Serialization roundtrip...");

    for trap in [
      StrawManTrap::IrrationalActor,
      StrawManTrap::ManicPixieDreamUser,
      StrawManTrap::StoicMonk,
      StrawManTrap::YourClone,
    ] {
      let json = serde_json::to_string(&trap).expect("trap serialization");
      let back: StrawManTrap = serde_json::from_str(&json).expect("trap deserialization");
      assert_eq!(trap, back, "Trap roundtrip failed for {:?}", trap);
    }

    let roundtrip_cases = vec![
      StrawManValidation::passing(),
      StrawManValidation::new(vec![StrawManTrap::IrrationalActor]),
      StrawManValidation::new(vec![StrawManTrap::StoicMonk, StrawManTrap::YourClone]),
      StrawManValidation::new(vec![
        StrawManTrap::IrrationalActor,
        StrawManTrap::ManicPixieDreamUser,
        StrawManTrap::StoicMonk,
        StrawManTrap::YourClone,
      ]),
    ];

    for original in roundtrip_cases {
      let json = serde_json::to_string(&original).expect("serialization");
      let back: StrawManValidation = serde_json::from_str(&json).expect("deserialization");
      assert_eq!(original, back, "Roundtrip failed for {:?}", original);
      assert_eq!(original.passed, back.passed);
      assert_eq!(original.traps_detected.len(), back.traps_detected.len());
      assert!(back.is_valid(), "Deserialized must satisfy invariants");
    }

    println!("  PASSED");
  }

  // ============================================================
  // ATTACK 4: Order independence (semantic)
  // ============================================================
  {
    println!("ATTACK 4: Order independence...");

    let va = StrawManValidation::new(vec![StrawManTrap::IrrationalActor, StrawManTrap::YourClone]);
    let vb = StrawManValidation::new(vec![StrawManTrap::YourClone, StrawManTrap::IrrationalActor]);

    assert_eq!(va.trap_count(), vb.trap_count());
    assert_eq!(va.passed, vb.passed);
    assert_eq!(
      va.has_trap(StrawManTrap::IrrationalActor),
      vb.has_trap(StrawManTrap::IrrationalActor)
    );
    assert_eq!(
      va.has_trap(StrawManTrap::YourClone),
      vb.has_trap(StrawManTrap::YourClone)
    );
    assert_eq!(va.is_valid(), vb.is_valid());

    println!("  PASSED (noted: PartialEq is order-sensitive via Vec)");
  }

  // ============================================================
  // ATTACK 5: Duplicate trap handling
  // ============================================================
  {
    println!("ATTACK 5: Duplicate trap handling...");

    let dupes = vec![
      StrawManTrap::IrrationalActor,
      StrawManTrap::IrrationalActor,
      StrawManTrap::IrrationalActor,
    ];
    let v = StrawManValidation::new(dupes);

    assert_eq!(v.trap_count(), 3, "new() must not deduplicate");
    assert!(!v.passed);
    assert!(v.is_valid());
    assert!(v.has_trap(StrawManTrap::IrrationalActor));

    println!("  PASSED (duplicates preserved — no dedup in constructor)");
  }

  // ============================================================
  // ATTACK 6: has_trap exhaustiveness
  // ============================================================
  {
    println!("ATTACK 6: has_trap exhaustiveness...");

    let all = StrawManValidation::new(vec![
      StrawManTrap::IrrationalActor,
      StrawManTrap::ManicPixieDreamUser,
      StrawManTrap::StoicMonk,
      StrawManTrap::YourClone,
    ]);

    for &trap in StrawManTrap::all() {
      assert!(all.has_trap(trap), "has_trap must find {:?}", trap);
    }

    let single = StrawManValidation::new(vec![StrawManTrap::StoicMonk]);
    assert!(single.has_trap(StrawManTrap::StoicMonk));
    assert!(!single.has_trap(StrawManTrap::IrrationalActor));
    assert!(!single.has_trap(StrawManTrap::ManicPixieDreamUser));
    assert!(!single.has_trap(StrawManTrap::YourClone));

    let empty = StrawManValidation::passing();
    for &trap in StrawManTrap::all() {
      assert!(!empty.has_trap(trap));
    }

    println!("  PASSED");
  }

  // ============================================================
  // ATTACK 7: Clone independence
  // ============================================================
  {
    println!("ATTACK 7: Clone independence...");

    let original = StrawManValidation::new(vec![StrawManTrap::IrrationalActor]);
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.trap_count(), cloned.trap_count());
    assert_eq!(original.passed, cloned.passed);

    let trap = StrawManTrap::YourClone;
    let trap_copy = trap;
    assert_eq!(trap, trap_copy);

    println!("  PASSED");
  }

  // ============================================================
  // ATTACK 8: Edge cases
  // ============================================================
  {
    println!("ATTACK 8: Edge cases...");

    let empty_via_new = StrawManValidation::new(vec![]);
    assert!(empty_via_new.passed);
    assert_eq!(empty_via_new, StrawManValidation::passing());

    for &trap in StrawManTrap::all() {
      let v = StrawManValidation::new(vec![trap]);
      assert!(!v.passed, "Single trap {:?} must fail", trap);
      assert_eq!(v.trap_count(), 1);
      assert!(v.is_valid());
    }

    assert!(StrawManValidation::passing().is_valid());
    assert!(StrawManValidation::new(vec![StrawManTrap::IrrationalActor]).is_valid());

    println!("  PASSED");
  }

  // ============================================================
  // ATTACK 9: Exhaustive 2^4 subset property check
  // ============================================================
  {
    println!("ATTACK 9: Exhaustive property-based check (16 subsets)...");

    let all_traps = StrawManTrap::all();

    for mask in 0u8..16 {
      let subset: Vec<StrawManTrap> = all_traps
        .iter()
        .enumerate()
        .filter(|(i, _)| (mask & (1 << i)) != 0)
        .map(|(_, &trap)| trap)
        .collect();

      let v = StrawManValidation::new(subset.clone());

      assert_eq!(v.passed, subset.is_empty(), "P1 mask={:04b}", mask);
      assert_eq!(v.trap_count(), subset.len(), "P2 mask={:04b}", mask);
      assert!(v.is_valid(), "P3 mask={:04b}", mask);

      let json = serde_json::to_string(&v).unwrap();
      let back: StrawManValidation = serde_json::from_str(&json).unwrap();
      assert_eq!(v, back, "P4 mask={:04b}", mask);

      for &trap in all_traps {
        assert_eq!(
          v.has_trap(trap),
          subset.contains(&trap),
          "P5 mask={:04b}, trap={:?}",
          mask,
          trap
        );
      }
    }

    println!("  PASSED: All 16 subsets satisfy 5 properties");
  }

  // ============================================================
  // ATTACK 10: Send + Sync compile-time verification
  // ============================================================
  {
    println!("ATTACK 10: Thread safety properties...");

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<StrawManTrap>();
    assert_sync::<StrawManTrap>();
    assert_send::<StrawManValidation>();
    assert_sync::<StrawManValidation>();

    println!("  PASSED: Send + Sync");
  }

  // ============================================================
  // ATTACK 11: StrawManTrap::all() completeness and label/checkbox/description coverage
  // ============================================================
  {
    println!("ATTACK 11: StrawManTrap metadata completeness...");

    let all = StrawManTrap::all();
    assert_eq!(all.len(), 4, "all() must return exactly 4 variants");

    for &trap in all {
      assert!(!trap.label().is_empty(), "label empty for {:?}", trap);
      assert!(
        !trap.description().is_empty(),
        "description empty for {:?}",
        trap
      );
      assert!(
        trap.description().len() > 20,
        "description too short for {:?}",
        trap
      );
      assert!(
        trap.checkbox_label().ends_with('?'),
        "checkbox_label must end with '?' for {:?}: {}",
        trap,
        trap.checkbox_label()
      );
    }

    println!("  PASSED");
  }

  println!("\nAll 11 REDQUEEN attacks passed.");
  println!("\nFINDINGS:");
  println!("  1. No invariant violations — constructor is sound");
  println!("  2. PartialEq is order-sensitive (Vec equality) — by design");
  println!("  3. No deduplication in constructor — duplicates faithfully preserved");
  println!("  4. Serialization roundtrip is bijective across all 16 subsets");
  println!("  5. Types are Send + Sync — no interior mutability concerns");
  println!("  6. DESIGN NOTE: If order-independent equality needed, use HashSet<StrawManTrap>");
}
