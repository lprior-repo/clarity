#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Adversarial Generation 2 Testing - Quality Scoring Edge Cases
//!
//! Tests boundary conditions, overflow scenarios, and edge cases in quality.rs

use clarity_web::lattice::quality::*;
use clarity_web::types::Answer;

/// Helper to create test answers
fn create_answer(step_id: &str, value: &str) -> Answer {
    Answer {
        step_id: step_id.to_string(),
        value: value.to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
    }
}

/// Helper to create EARS requirements
fn create_ears(id: &str, text: &str, has_criteria: bool) -> EarsRequirementRef {
    EarsRequirementRef {
        id: id.to_string(),
        text: text.to_string(),
        has_acceptance_criteria: has_criteria,
    }
}

#[test]
fn test_quality_boundary_zero_answers() {
    // BUG: Empty answers are rejected but error handling is inconsistent
    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&[], &ears, &inversion);
    assert!(matches!(result, Err(QualityError::EmptyAnswers)));
}

#[test]
fn test_quality_boundary_maximum_score_edge_case() {
    // Test all dimensions at 100 (boundary condition)
    let answers = vec![
        create_answer("user_goal", "User must authenticate securely"),
        create_answer("actors", "System administrator"),
        create_answer("precondition", "User account exists"),
        create_answer("outcome", "Access granted securely"),
        create_answer("acceptance_criteria", "Authentication completes within 2 seconds"),
        create_answer(
            "security",
            "System must use TLS encryption with perfect forward secrecy and validate all inputs",
        ),
    ];

    let ears = vec![
        create_ears("1", "User shall authenticate", true),
        create_ears("2", "System shall encrypt data", true),
    ];

    let inversion = InversionControl {
        has_inversion_tests: true,
        inverted_count: 2,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Check that overall is bounded correctly
            assert!(score.overall <= 100, "Overall score must not exceed 100");
            assert!(score.overall >= 0, "Overall score must not be negative");

            // Check all dimensions are valid
            for dim in &score.dimensions {
                assert!(dim.score <= 100, "Dimension score must not exceed 100");
                assert!(dim.score >= 0, "Dimension score must not be negative");
            }
        }
        Err(e) => {
            panic!("Quality calculation should succeed with valid input: {:?}", e);
        }
    }
}

#[test]
fn test_quality_division_by_zero_protection() {
    // Test division by zero protection via public API
    // When answers don't match required patterns, calculation should still work

    let answers = vec![
        create_answer("other_field", "Some value"), // No matching patterns
    ];

    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Completeness should be 0 (no required fields)
            assert!(score.overall <= 100);
            assert!(score.overall >= 0);

            // Should have completeness issues
            let completeness_issues = score.get_issues(QualityDimension::Completeness);
            assert!(!completeness_issues.is_empty());
        }
        Err(e) => {
            panic!("Should handle missing fields gracefully: {:?}", e);
        }
    }
}

#[test]
fn test_quality_overflow_in_calculation() {
    // Test potential overflow when calculating overall score
    // Overall = sum(dimensions) / dimensions.len()
    // Maximum sum = 5 dimensions * 100 = 500 (fits in u32)

    let answers = vec![
        create_answer("user_goal", "Goal"),
        create_answer("actors", "Actor"),
        create_answer("precondition", "Precondition"),
        create_answer("outcome", "Outcome"),
        create_answer("acceptance_criteria", "Criteria"),
    ];

    let ears = vec![
        create_ears("1", "Requirement 1", true),
        create_ears("2", "Requirement 2", true),
        create_ears("3", "Requirement 3", true),
        create_ears("4", "Requirement 4", true),
        create_ears("5", "Requirement 5", true),
    ];

    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Overall should be valid
            assert!(score.overall <= 100);
            assert!(score.overall >= 0);

            // Should be exactly 100 since security might not be perfect
            // but other dimensions should be high
        }
        Err(QualityError::InvalidScore(msg)) if msg.contains("overflow") => {
            // This is acceptable - overflow is detected
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

#[test]
fn test_quality_empty_answers_with_ears() {
    // Edge case: Empty answers but with EARS requirements
    let answers = vec![];
    let ears = vec![create_ears("1", "Req", true)];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);
    assert!(matches!(result, Err(QualityError::EmptyAnswers)));
}

#[test]
fn test_quality_single_answer() {
    // Boundary: Single answer (minimum valid input)
    let answers = vec![create_answer("user_goal", "Goal")];
    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            assert!(score.overall <= 100);
            assert_eq!(score.dimensions.len(), 5);

            // Testability should be 0 (no EARS)
            let testability = score.get_dimension(QualityDimension::Testability);
            assert!(testability.is_some());
            assert_eq!(testability.unwrap().score, 0);
        }
        Err(e) => {
            panic!("Should handle single answer: {:?}", e);
        }
    }
}

#[test]
fn test_quality_consistency_single_answer() {
    // BUG: Consistency calculation with single answer (total_pairs = 0)
    let answers = vec![create_answer("req1", "Single requirement")];
    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Single answer should have 100% consistency (no contradictions possible)
            let consistency = score.get_dimension(QualityDimension::Consistency);
            assert!(consistency.is_some());
            assert_eq!(consistency.unwrap().score, 100);
        }
        Err(e) => {
            panic!("Should handle single answer: {:?}", e);
        }
    }
}

#[test]
fn test_quality_testability_empty_ears() {
    // Edge case: No EARS requirements
    let answers = vec![create_answer("req1", "Requirement")];
    let ears: Vec<EarsRequirementRef> = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Testability should be 0 (no EARS)
            let testability = score.get_dimension(QualityDimension::Testability);
            assert!(testability.is_some());
            assert_eq!(testability.unwrap().score, 0);

            // Should have testability issue
            let testability_issues = score.get_issues(QualityDimension::Testability);
            assert!(!testability_issues.is_empty());
            assert!(testability_issues[0].message.contains("No EARS"));
        }
        Err(e) => {
            panic!("Should handle empty EARS: {:?}", e);
        }
    }
}

#[test]
fn test_quality_clarity_empty_answers() {
    // Edge case: Empty answers return early (EmptyAnswers error)
    let answers: Vec<Answer> = vec![];
    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    // Should return EmptyAnswers error before reaching clarity calculation
    assert!(matches!(result, Err(QualityError::EmptyAnswers)));
}

#[test]
fn test_quality_security_empty_answers() {
    // Edge case: Empty answers return early (EmptyAnswers error)
    let answers: Vec<Answer> = vec![];
    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    // Should return EmptyAnswers error before reaching security calculation
    assert!(matches!(result, Err(QualityError::EmptyAnswers)));
}

#[test]
fn test_quality_dimension_score_boundary_100() {
    // Test maximum valid score
    let result = DimensionScore::new(QualityDimension::Completeness, 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().score, 100);
}

#[test]
fn test_quality_dimension_score_boundary_0() {
    // Test minimum valid score
    let result = DimensionScore::new(QualityDimension::Completeness, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().score, 0);
}

#[test]
fn test_quality_dimension_score_invalid_101() {
    // Test score just above maximum
    let result = DimensionScore::new(QualityDimension::Completeness, 101);
    assert!(matches!(result, Err(QualityError::InvalidScore(_))));
}

#[test]
fn test_quality_dimension_score_invalid_255() {
    // Test maximum u8 value
    let result = DimensionScore::new(QualityDimension::Completeness, 255);
    assert!(matches!(result, Err(QualityError::InvalidScore(_))));
}

#[test]
fn test_quality_score_boundary_100() {
    // Test maximum valid overall score
    let dimensions = vec![
        DimensionScore::new(QualityDimension::Completeness, 100).unwrap(),
        DimensionScore::new(QualityDimension::Consistency, 100).unwrap(),
        DimensionScore::new(QualityDimension::Testability, 100).unwrap(),
        DimensionScore::new(QualityDimension::Clarity, 100).unwrap(),
        DimensionScore::new(QualityDimension::Security, 100).unwrap(),
    ];

    let result = QualityScore::new(100, dimensions, vec![]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().overall, 100);
}

#[test]
fn test_quality_score_boundary_0() {
    // Test minimum valid overall score
    let dimensions = vec![
        DimensionScore::new(QualityDimension::Completeness, 0).unwrap(),
        DimensionScore::new(QualityDimension::Consistency, 0).unwrap(),
        DimensionScore::new(QualityDimension::Testability, 0).unwrap(),
        DimensionScore::new(QualityDimension::Clarity, 0).unwrap(),
        DimensionScore::new(QualityDimension::Security, 0).unwrap(),
    ];

    let result = QualityScore::new(0, dimensions, vec![]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().overall, 0);
}

#[test]
fn test_quality_score_invalid_overall_101() {
    // Test invalid overall score
    let dimensions = vec![];
    let result = QualityScore::new(101, dimensions, vec![]);
    assert!(matches!(result, Err(QualityError::InvalidScore(_))));
}

#[test]
fn test_quality_completeness_with_whitespace_only() {
    // Edge case: Answers with only whitespace
    let answers = vec![
        create_answer("user_goal", "   "),
        create_answer("actors", "\t"),
        create_answer("precondition", "\n"),
    ];

    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Should treat whitespace as empty (low completeness)
            let completeness = score.get_dimension(QualityDimension::Completeness);
            assert!(completeness.is_some());
            assert!(completeness.unwrap().score < 100);

            // Should have completeness issues
            let completeness_issues = score.get_issues(QualityDimension::Completeness);
            assert!(!completeness_issues.is_empty());
        }
        Err(e) => {
            panic!("Should handle whitespace: {:?}", e);
        }
    }
}

#[test]
fn test_quality_consistency_contradiction_self() {
    // Edge case: Single answer with self-contradiction
    let answers = vec![create_answer(
        "req1",
        "Users must authenticate but must not authenticate",
    )];

    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Current implementation checks pairs of answers
            // Single answer won't trigger contradiction detection
            let consistency = score.get_dimension(QualityDimension::Consistency);
            assert!(consistency.is_some());
            assert_eq!(consistency.unwrap().score, 100); // No pairs to compare
        }
        Err(e) => {
            panic!("Should handle self-contradiction: {:?}", e);
        }
    }
}

#[test]
fn test_quality_clarity_extremely_long_sentence() {
    // Edge case: Extremely long sentence
    let long_text = "The system shall, under normal operating conditions, provided that all prerequisites are met, and assuming no external interference, and considering all edge cases, and with proper error handling, process the data securely, efficiently, and reliably, while maintaining performance, security, and usability.".repeat(10);

    let answers = vec![create_answer("req1", &long_text)];

    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Should penalize heavily but not crash
            assert!(score.overall >= 0);
            assert!(score.overall <= 100);

            let clarity = score.get_dimension(QualityDimension::Clarity);
            assert!(clarity.is_some());
            assert!(clarity.unwrap().score < 100);
        }
        Err(e) => {
            panic!("Should handle long sentences: {:?}", e);
        }
    }
}

#[test]
fn test_quality_security_all_keywords_repeated() {
    // Edge case: All security keywords mentioned multiple times
    let security_text = "auth auth auth authentication login password encrypt decrypt hash salt tls ssl https validate sanitize escape csrf xss injection authentication authentication encryption encryption validation validation";

    let answers = vec![create_answer("security", security_text)];

    let ears = vec![];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // Should cap at 100
            let security = score.get_dimension(QualityDimension::Security);
            assert!(security.is_some());
            assert_eq!(security.unwrap().score, 100);
        }
        Err(e) => {
            panic!("Should handle repeated keywords: {:?}", e);
        }
    }
}

#[test]
fn test_quality_testability_mixed_criteria() {
    // Edge case: Mix of EARS with and without criteria
    let ears = vec![
        create_ears("1", "Req 1", true),
        create_ears("2", "Req 2", false),
        create_ears("3", "Req 3", true),
        create_ears("4", "Req 4", false),
        create_ears("5", "Req 5", true),
    ];

    let answers = vec![create_answer("req1", "Requirement")];
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);

    match result {
        Ok(score) => {
            // 3 out of 5 = 60%
            let testability = score.get_dimension(QualityDimension::Testability);
            assert!(testability.is_some());
            assert_eq!(testability.unwrap().score, 60);

            let testability_issues = score.get_issues(QualityDimension::Testability);
            assert_eq!(testability_issues.len(), 1);
        }
        Err(e) => {
            panic!("Should handle mixed criteria: {:?}", e);
        }
    }
}

#[test]
fn test_quality_inversion_control_not_used() {
    // Document that inversion control is accepted but not used in calculation
    let answers = vec![create_answer("req1", "Requirement")];
    let ears = vec![];

    let inversion_with_tests = InversionControl {
        has_inversion_tests: true,
        inverted_count: 100,
    };

    let inversion_without_tests = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let result1 = calculate_quality(&answers, &ears, &inversion_with_tests);
    let result2 = calculate_quality(&answers, &ears, &inversion_without_tests);

    // Both should produce same result (inversion not used in scoring)
    match (result1, result2) {
        (Ok(score1), Ok(score2)) => {
            assert_eq!(score1.overall, score2.overall);
        }
        _ => {
            panic!("Both should succeed or both should fail");
        }
    }
}
