#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

use clarity_web::kirk::progressive_discover::KirkContract16;

/// Test that KirkContract16 can be created and has the expected structure
/// for Bead Factory integration.
///
/// This test validates that when the user clicks "Continue to Bead Factory"
/// from the Locked phase, the KirkContract16 is ready to be passed to
/// the Bead Factory route.
#[test]
fn test_kirk_contract_has_16_sections_for_bead_factory() {
    let contract = KirkContract16::new();

    // The contract must have exactly 16 sections for KIRK methodology
    assert_eq!(
        contract.sections.len(),
        16,
        "KirkContract16 must have 16 sections for Bead Factory integration"
    );
}
