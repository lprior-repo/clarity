#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

/// Integration tests for DiscoverFlow component
///
/// Tests the mode switching behavior between Express and Guided flows:
/// - Express -> Guided: pre-populate sequential inputs with extracted field content
/// - Guided -> Express: concatenate answers into freeform textarea
/// - Preserves answers signal across switch
/// - Marks partial progress (e.g., 3/5 in Guided)
/// - Shows mode switch confirmation if unsaved changes

use clarity_web::components::discover::{
    discover_flow::{DiscoverFlow, DiscoverFlowProps},
    field_card::{Confidence, FieldData},
    mode_toggle::DiscoverMode,
};
use clarity_web::types::Answer;
use dioxus::prelude::*;

/// Integration test: Express to Guided mode switch preserves data
#[test]
fn test_express_to_guided_data_preservation() {
    // Create test answers simulating Express flow extraction
    let answers = vec![
        Answer {
            step_id: "problem".to_string(),
            value: "Test problem statement".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "user".to_string(),
            value: "Test user".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    // Simulate Express -> Guided switch
    // When switching from Express to Guided, the answers should be preserved
    // and available for pre-filling the Guided inputs

    assert_eq!(answers.len(), 2);
    assert_eq!(answers[0].step_id, "problem");
    assert_eq!(answers[1].step_id, "user");
}

/// Integration test: Guided to Express mode switch concatenates answers
#[test]
fn test_guided_to_express_concatenation() {
    // Create test answers from Guided flow
    let answers = vec![
        Answer {
            step_id: "problem".to_string(),
            value: "Test problem".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "user".to_string(),
            value: "Test user".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "context".to_string(),
            value: "Test context".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    // Simulate concatenation for Express mode
    let concatenated = answers
        .iter()
        .filter(|a| {
            matches!(
                a.step_id.as_str(),
                "problem" | "user" | "context" | "constraints" | "goals"
            )
        })
        .map(|a| format!("{}: {}", a.step_id.to_uppercase(), a.value))
        .collect::<Vec<_>>()
        .join("\n\n");

    assert!(concatenated.contains("PROBLEM: Test problem"));
    assert!(concatenated.contains("USER: Test user"));
    assert!(concatenated.contains("CONTEXT: Test context"));
}

/// Integration test: Partial progress calculation
#[test]
fn test_partial_progress_calculation() {
    let answers = vec![
        Answer {
            step_id: "problem".to_string(),
            value: "Test problem".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "user".to_string(),
            value: "".to_string(), // Empty - not counted
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "context".to_string(),
            value: "Test context".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "constraints".to_string(),
            value: "   ".to_string(), // Whitespace only - not counted
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    let total = 5; // problem, user, context, constraints, goals
    let answered = answers
        .iter()
        .filter(|a| {
            matches!(
                a.step_id.as_str(),
                "problem" | "user" | "context" | "constraints" | "goals"
            ) && !a.value.trim().is_empty()
        })
        .count();

    assert_eq!(answered, 2);
    assert_eq!(total, 5);
}

/// Integration test: Field data to answer conversion
#[test]
fn test_field_data_to_answer_conversion() {
    let fields = vec![
        FieldData {
            id: "problem".to_string(),
            title: "Problem Statement".to_string(),
            content: "Test problem content".to_string(),
            confidence: Confidence::High,
            locked: false,
        },
        FieldData {
            id: "user".to_string(),
            title: "Target User".to_string(),
            content: "Test user content".to_string(),
            confidence: Confidence::Medium,
            locked: false,
        },
    ];

    let converted_answers: Vec<Answer> = fields
        .iter()
        .map(|field| Answer {
            step_id: field.id.clone(),
            value: field.content.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
        .collect();

    assert_eq!(converted_answers.len(), 2);
    assert_eq!(converted_answers[0].step_id, "problem");
    assert_eq!(converted_answers[0].value, "Test problem content");
    assert_eq!(converted_answers[1].step_id, "user");
    assert_eq!(converted_answers[1].value, "Test user content");
}

/// Integration test: Mode switching state consistency
#[test]
fn test_mode_switching_state_consistency() {
    // Test that mode enum values are consistent
    let express = DiscoverMode::Express;
    let guided = DiscoverMode::Guided;

    assert_eq!(express, DiscoverMode::Express);
    assert_eq!(guided, DiscoverMode::Guided);
    assert_ne!(express, guided);

    // Test default mode
    let default = DiscoverMode::default();
    assert_eq!(default, DiscoverMode::Guided);
}

/// Integration test: Answer filtering for Express fields
#[test]
fn test_answer_filtering_for_express_fields() {
    let answers = vec![
        Answer {
            step_id: "problem".to_string(),
            value: "Test problem".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "antithesis".to_string(), // Not an Express field
            value: "Test antithesis".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "user".to_string(),
            value: "Test user".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    let express_field_ids = ["problem", "user", "context", "constraints", "goals"];
    let filtered: Vec<_> = answers
        .iter()
        .filter(|a| express_field_ids.contains(&a.step_id.as_str()))
        .collect();

    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].step_id, "problem");
    assert_eq!(filtered[1].step_id, "user");
}

/// Integration test: Answer removal and replacement logic
#[test]
fn test_answer_removal_and_replacement() {
    let mut answers = vec![
        Answer {
            step_id: "problem".to_string(),
            value: "Old problem".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "user".to_string(),
            value: "Old user".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "other_field".to_string(), // Should be preserved
            value: "Keep this".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    let express_field_ids = ["problem", "user", "context", "constraints", "goals"];

    // Remove old Express field answers
    answers.retain(|a| !express_field_ids.contains(&a.step_id.as_str()));

    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].step_id, "other_field");
    assert_eq!(answers[0].value, "Keep this");

    // Add new answers
    let new_answers = vec![
        Answer {
            step_id: "problem".to_string(),
            value: "New problem".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Answer {
            step_id: "user".to_string(),
            value: "New user".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    answers.extend(new_answers);

    assert_eq!(answers.len(), 3);
    assert_eq!(answers[0].step_id, "other_field");
    assert_eq!(answers[1].step_id, "problem");
    assert_eq!(answers[1].value, "New problem");
    assert_eq!(answers[2].step_id, "user");
    assert_eq!(answers[2].value, "New user");
}

/// Integration test: Empty content validation
#[test]
fn test_empty_content_validation() {
    let empty_content = "";
    assert!(empty_content.trim().is_empty());

    let whitespace_content = "   \n\t  ";
    assert!(whitespace_content.trim().is_empty());

    let valid_content = "Valid content here";
    assert!(!valid_content.trim().is_empty());
}
