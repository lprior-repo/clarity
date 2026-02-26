// Simple test to verify StrawManValidation logic
// This tests the validation logic independently of the server function

use serde::{Deserialize, Serialize};

/// Straw Man trap types that indicate unrealistic user persona assumptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum StrawManTrap {
    IrrationalActor,
    ManicPixieDreamUser,
    StoicMonk,
    YourClone,
}

/// Result of validating a persona description against straw man traps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrawManValidation {
    pub traps_detected: Vec<StrawManTrap>,
    pub passed: bool,
}

impl StrawManValidation {
    pub fn new(traps_detected: Vec<StrawManTrap>) -> Self {
        let passed = traps_detected.is_empty();
        Self {
            traps_detected,
            passed,
        }
    }

    pub fn passing() -> Self {
        Self {
            traps_detected: vec![],
            passed: true,
        }
    }

    pub fn has_trap(&self, trap: StrawManTrap) -> bool {
        self.traps_detected.contains(&trap)
    }

    pub fn trap_count(&self) -> usize {
        self.traps_detected.len()
    }

    pub fn is_valid(&self) -> bool {
        self.passed == self.traps_detected.is_empty()
    }
}

impl Default for StrawManValidation {
    fn default() -> Self {
        Self::passing()
    }
}

fn main() {
    println!("Testing StrawManValidation logic...\n");

    // Test 1: Passing validation
    let passing = StrawManValidation::passing();
    assert!(passing.passed, "Passing validation should have passed=true");
    assert!(passing.traps_detected.is_empty(), "Passing validation should have no traps");
    assert_eq!(passing.trap_count(), 0, "Passing validation should have 0 traps");
    println!("✅ Test 1 passed: Passing validation works correctly");

    // Test 2: Single trap detection
    let single_trap = StrawManValidation::new(vec![StrawManTrap::IrrationalActor]);
    assert!(!single_trap.passed, "Single trap should have passed=false");
    assert_eq!(single_trap.trap_count(), 1, "Should have 1 trap");
    assert!(single_trap.has_trap(StrawManTrap::IrrationalActor), "Should have IrrationalActor");
    println!("✅ Test 2 passed: Single trap detection works correctly");

    // Test 3: Multiple traps
    let multi_trap = StrawManValidation::new(vec![
        StrawManTrap::ManicPixieDreamUser,
        StrawManTrap::StoicMonk,
        StrawManTrap::YourClone,
    ]);
    assert!(!multi_trap.passed, "Multiple traps should have passed=false");
    assert_eq!(multi_trap.trap_count(), 3, "Should have 3 traps");
    assert!(multi_trap.has_trap(StrawManTrap::ManicPixieDreamUser), "Should have ManicPixieDreamUser");
    assert!(multi_trap.has_trap(StrawManTrap::StoicMonk), "Should have StoicMonk");
    assert!(multi_trap.has_trap(StrawManTrap::YourClone), "Should have YourClone");
    assert!(!multi_trap.has_trap(StrawManTrap::IrrationalActor), "Should not have IrrationalActor");
    println!("✅ Test 3 passed: Multiple traps detection works correctly");

    // Test 4: Serialization
    let validation = StrawManValidation::new(vec![
        StrawManTrap::IrrationalActor,
        StrawManTrap::YourClone,
    ]);
    let serialized = serde_json::to_string(&validation).expect("Serialization should succeed");
    println!("Serialized: {}", serialized);

    let deserialized: StrawManValidation = serde_json::from_str(&serialized)
        .expect("Deserialization should succeed");
    assert_eq!(deserialized.traps_detected.len(), 2, "Should have 2 traps after deserialization");
    assert!(!deserialized.passed, "Should be failed after deserialization");
    println!("✅ Test 4 passed: Serialization works correctly");

    // Test 5: All trap types
    let all_traps = StrawManValidation::new(vec![
        StrawManTrap::IrrationalActor,
        StrawManTrap::ManicPixieDreamUser,
        StrawManTrap::StoicMonk,
        StrawManTrap::YourClone,
    ]);
    assert_eq!(all_traps.trap_count(), 4, "Should have all 4 traps");
    for trap in [
        StrawManTrap::IrrationalActor,
        StrawManTrap::ManicPixieDreamUser,
        StrawManTrap::StoicMonk,
        StrawManTrap::YourClone,
    ] {
        assert!(all_traps.has_trap(trap), "Should have {:?}", trap);
    }
    println!("✅ Test 5 passed: All trap types work correctly");

    // Test 6: Invariant validation
    let passing = StrawManValidation::passing();
    assert!(passing.is_valid(), "Passing validation should be valid");
    let failing = StrawManValidation::new(vec![StrawManTrap::IrrationalActor]);
    assert!(failing.is_valid(), "Failing validation should maintain invariant");
    println!("✅ Test 6 passed: Invariant validation works correctly");

    println!("\n🎉 All tests passed!");
}
