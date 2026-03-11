//! Tests for straw man trap helper functions

#![allow(clippy::unwrap_used)]

use clarity_web::server::parse_traps_from_fields;
use clarity_web::components::discover::straw_man::StrawManTrap;
use clarity_web::providers::{FieldExtraction, FieldType};
use serde_json::json;

/// Test parse_traps_from_fields extracts traps correctly
#[test]
fn test_parse_traps_from_fields_extracts_traps() {
    let fields = vec![
        FieldExtraction {
            name: "irrational_actor_detected".to_string(),
            value: json!(true),
            field_type: FieldType::Boolean,
            confidence: 1.0,
            justification: None,
        },
        FieldExtraction {
            name: "manic_pixie_dream_user_detected".to_string(),
            value: json!(false),
            field_type: FieldType::Boolean,
            confidence: 1.0,
            justification: None,
        },
        FieldExtraction {
            name: "stoic_monk_detected".to_string(),
            value: json!(true),
            field_type: FieldType::Boolean,
            confidence: 1.0,
            justification: None,
        },
        FieldExtraction {
            name: "your_clone_detected".to_string(),
            value: json!(false),
            field_type: FieldType::Boolean,
            confidence: 1.0,
            justification: None,
        },
    ];

    let traps = parse_traps_from_fields(&fields);
    assert_eq!(traps.len(), 2);
    assert!(traps.contains(&StrawManTrap::IrrationalActor));
    assert!(traps.contains(&StrawManTrap::StoicMonk));
}
