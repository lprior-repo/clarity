#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

// TODO: Implement export_contract_to_json function
// use clarity_web::components::discover::progressive_discover::export_contract_to_json;
// use clarity_web::kirk::progressive_discover::KirkContract16;

// #[test]
// fn test_export_contract_serializes_to_pretty_json() {
//     // Given a KirkContract16, export_contract_to_json should produce valid pretty-printed JSON
//     let contract = KirkContract16::new();
//     let result = export_contract_to_json(&contract);
//
//     // Should succeed with valid JSON
//     assert!(result.is_ok(), "export_contract_to_json should succeed");
//
//     let json = result.expect("checked is_ok");
//     // Pretty-printed JSON should contain newlines
//     assert!(json.contains('\n'), "Pretty-printed JSON should contain newlines");
//     // Should be parseable back
//     let parsed: Result<KirkContract16, _> = serde_json::from_str(&json);
//     assert!(parsed.is_ok(), "Output should be valid JSON that parses back to KirkContract16");
// }
