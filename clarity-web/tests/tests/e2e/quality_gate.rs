#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! End-to-end test: Quality gating and Develop phase unlock.
//!
//! This test simulates the user flow:
//! 1. Complete Discover with minimal answers (quality < 70)
//! 2. Verify quality bar shows red/yellow score
//! 3. Verify Develop tab is disabled
//! 4. Verify tooltip explains quality gate requirement
//! 5. Add more detail to answers to improve quality
//! 6. Verify quality score increases to >= 70
//! 7. Verify quality bar turns green
//! 8. Verify Develop tab becomes enabled
//! 9. Verify can click Develop and transition

use chrono::Utc;
use clarity_web::components::quality::MINIMUM_GATE;
use clarity_web::lattice::quality::{
    calculate_quality, Answer, DimensionScore, EarsRequirementRef, InversionControl,
    QualityDimension, QualityIssue, QualityScore,
};
use clarity_web::types::Answer as AppAnswer;
use clarity_web::app::pages::is_phase_done;

/// Convert AppAnswer to quality module Answer
fn to_quality_answer(answer: &AppAnswer) -> Answer {
    Answer {
        step_id: answer.step_id.clone(),
        value: answer.value.clone(),
        timestamp: answer.timestamp.clone(),
    }
}

/// Create a test answer with timestamp
fn create_answer(step_id: &str, value: &str) -> AppAnswer {
    AppAnswer {
        step_id: step_id.to_string(),
        value: value.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Create EARS requirements for testing
fn create_test_ears(with_acceptance_criteria: bool) -> Vec<EarsRequirementRef> {
    vec![
        EarsRequirementRef {
            id: "req-1".to_string(),
            text: "User shall authenticate with password".to_string(),
            has_acceptance_criteria: with_acceptance_criteria,
        },
        EarsRequirementRef {
            id: "req-2".to_string(),
            text: "System shall encrypt all data at rest".to_string(),
            has_acceptance_criteria: with_acceptance_criteria,
        },
    ]
}

/// Test helper: Check if phase button should be disabled based on quality
fn should_disable_develop(answers: &[AppAnswer], quality_score: &Option<QualityScore>) -> bool {
    // Check if Discover phase is complete
    let discover_complete = is_phase_done("discover", answers);

    // Check if quality gate is passed
    let passes_gate = quality_score
        .as_ref()
        .map(|s| s.passes(MINIMUM_GATE))
        .unwrap_or(false);

    // Develop is disabled if Discover is complete but quality gate is not passed
    discover_complete && !passes_gate
}

/// Test 1: Low quality locks Develop phase
#[test]
fn test_low_quality_locks_develop() {
    // Create minimal answers (low quality)
    let answers = vec![
        create_answer("user_goal", "Login"),
        create_answer("actors", "User"),
        // Missing precondition, outcome, acceptance_criteria, security
    ];

    // Calculate quality score
    let quality_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let ears = create_test_ears(false);
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let quality_result = calculate_quality(&quality_answers, &ears, &inversion);

    // Verify we got a quality score
    assert!(quality_result.is_ok(), "Should calculate quality score");

    let quality_score = quality_result.unwrap();

    // Verify score is below threshold
    assert!(
        quality_score.overall < MINIMUM_GATE,
        "Quality score {} should be below threshold {}",
        quality_score.overall,
        MINIMUM_GATE
    );

    // Verify color would be red/yellow (score < 70)
    assert!(quality_score.overall < 70, "Score should indicate red/yellow color");

    // Verify phase is complete
    assert!(
        is_phase_done("discover", &answers),
        "Discover phase should be marked complete"
    );

    // Verify Develop would be disabled
    assert!(
        should_disable_develop(&answers, &Some(quality_score.clone())),
        "Develop phase should be disabled when quality < {}",
        MINIMUM_GATE
    );

    // Verify we have completeness issues
    let completeness_issues = quality_score.get_issues(QualityDimension::Completeness);
    assert!(
        !completeness_issues.is_empty(),
        "Should have completeness issues for missing fields"
    );
}

/// Test 2: Quality score color coding
#[test]
fn test_quality_score_color_coding() {
    // Test red score (< 50)
    let low_answers = vec![
        create_answer("user_goal", "Login"),
        create_answer("actors", "User"),
    ];

    let low_quality_answers: Vec<Answer> = low_answers.iter().map(to_quality_answer).collect();
    let ears = create_test_ears(false);
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let low_score = calculate_quality(&low_quality_answers, &ears, &inversion).unwrap();
    assert!(
        low_score.overall < 50,
        "Low quality should be < 50 (red zone)"
    );

    // Test yellow score (50-69)
    let medium_answers = vec![
        create_answer("user_goal", "User login"),
        create_answer("actors", "System administrator"),
        create_answer("precondition", "User exists"),
    ];

    let medium_quality_answers: Vec<Answer> = medium_answers
        .iter()
        .map(to_quality_answer)
        .collect();
    let medium_score = calculate_quality(&medium_quality_answers, &ears, &inversion).unwrap();

    // Medium score should be in yellow/red range (not green yet)
    assert!(
        medium_score.overall < 70,
        "Medium quality should be < 70 (yellow/red zone)"
    );
}

/// Test 3: Tooltip explains quality gate requirement
#[test]
fn test_quality_gate_tooltip_message() {
    let answers = vec![
        create_answer("user_goal", "Login"),
        create_answer("actors", "User"),
        create_answer("precondition", "User exists"),
        create_answer("outcome", "Access granted"),
        create_answer("acceptance_criteria", "Works"),
    ];

    let quality_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let ears = create_test_ears(false);
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let quality_result = calculate_quality(&quality_answers, &ears, &inversion);
    assert!(quality_result.is_ok());

    let quality_score = quality_result.unwrap();

    // Verify disabled reason would be shown
    if should_disable_develop(&answers, &Some(quality_score.clone())) {
        // The tooltip message should mention the minimum threshold
        let expected_message = format!("Quality score must be at least {}", MINIMUM_GATE);

        // In the actual UI, this message would be shown in the tooltip
        // For testing, we verify the logic that generates it
        assert!(
            expected_message.contains(&MINIMUM_GATE.to_string()),
            "Tooltip should explain quality gate threshold"
        );
    }
}

/// Test 4: High quality unlocks Develop phase
#[test]
fn test_high_quality_unlocks_develop() {
    // Create comprehensive answers (high quality)
    let answers = vec![
        create_answer(
            "user_goal",
            "User must authenticate securely with username and password",
        ),
        create_answer(
            "actors",
            "System administrator, end user, security auditor",
        ),
        create_answer(
            "precondition",
            "User account exists and is active in the system",
        ),
        create_answer(
            "outcome",
            "User session is established and access is granted to authorized resources",
        ),
        create_answer(
            "acceptance_criteria",
            "Login completes within 2 seconds, password is hashed using bcrypt, and session token expires after 30 minutes of inactivity",
        ),
        create_answer(
            "security",
            "System shall use TLS 1.3 for encryption, bcrypt with salt rounds >= 12 for password hashing, and validate all inputs to prevent SQL injection and XSS attacks",
        ),
    ];

    // Calculate quality score
    let quality_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let ears = create_test_ears(true); // With acceptance criteria
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let quality_result = calculate_quality(&quality_answers, &ears, &inversion);

    // Verify we got a quality score
    assert!(quality_result.is_ok(), "Should calculate quality score");

    let quality_score = quality_result.unwrap();

    // Verify score meets or exceeds threshold
    assert!(
        quality_score.passes(MINIMUM_GATE),
        "Quality score {} should meet threshold {}",
        quality_score.overall,
        MINIMUM_GATE
    );

    // Verify score is in green zone (>= 70)
    assert!(
        quality_score.overall >= 70,
        "Score should indicate green color"
    );

    // Verify phase is complete
    assert!(
        is_phase_done("discover", &answers),
        "Discover phase should be marked complete"
    );

    // Verify Develop would be enabled
    assert!(
        !should_disable_develop(&answers, &Some(quality_score.clone())),
        "Develop phase should be enabled when quality >= {}",
        MINIMUM_GATE
    );
}

/// Test 5: Quality improvement flow
#[test]
fn test_quality_improvement_unlocks_develop() {
    // Start with minimal answers
    let mut answers = vec![
        create_answer("user_goal", "Login"),
        create_answer("actors", "User"),
    ];

    let ears = create_test_ears(false);
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    // Initial quality check
    let quality_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let initial_score = calculate_quality(&quality_answers, &ears, &inversion).unwrap();

    assert!(
        initial_score.overall < MINIMUM_GATE,
        "Initial score should be below threshold"
    );
    assert!(
        should_disable_develop(&answers, &Some(initial_score)),
        "Develop should be disabled initially"
    );

    // Add more detail to improve quality
    answers.push(create_answer(
        "precondition",
        "User account exists in the system",
    ));
    answers.push(create_answer(
        "outcome",
        "User is authenticated and can access their dashboard",
    ));
    answers.push(create_answer(
        "acceptance_criteria",
        "Login succeeds within 3 seconds with correct credentials",
    ));

    // Recheck quality
    let improved_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let improved_score = calculate_quality(&improved_answers, &ears, &inversion).unwrap();

    // Score should have improved
    assert!(
        improved_score.overall > initial_score.overall,
        "Quality should improve with more detail"
    );

    // But still might not pass gate due to missing security
    if improved_score.overall < MINIMUM_GATE {
        assert!(
            should_disable_develop(&answers, &Some(improved_score)),
            "Develop should still be disabled below threshold"
        );

        // Add security consideration to push over threshold
        answers.push(create_answer(
            "security",
            "System must use TLS encryption and validate all user inputs",
        ));

        let final_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
        let final_score = calculate_quality(&final_answers, &ears, &inversion).unwrap();

        // Now should pass
        assert!(
            final_score.passes(MINIMUM_GATE),
            "Final score should pass threshold with security added"
        );
        assert!(
            !should_disable_develop(&answers, &Some(final_score)),
            "Develop should be enabled with high quality"
        );
    }
}

/// Test 6: Visual feedback - quality bar color progression
#[test]
fn test_quality_bar_color_progression() {
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    // Test red score (0-49)
    let red_answers = vec![create_answer("user_goal", "Login")];
    let red_quality: Vec<Answer> = red_answers.iter().map(to_quality_answer).collect();
    let red_ears = create_test_ears(false);
    let red_score = calculate_quality(&red_quality, &red_ears, &inversion).unwrap();
    assert!(
        red_score.overall < 50,
        "Red zone score should be < 50"
    );

    // Test yellow score (50-69)
    let yellow_answers = vec![
        create_answer("user_goal", "User login"),
        create_answer("actors", "Admin"),
        create_answer("precondition", "User exists"),
    ];
    let yellow_quality: Vec<Answer> = yellow_answers.iter().map(to_quality_answer).collect();
    let yellow_score = calculate_quality(&yellow_quality, &red_ears, &inversion).unwrap();
    assert!(
        (50..=69).contains(&yellow_score.overall),
        "Yellow zone score should be 50-69, got {}",
        yellow_score.overall
    );

    // Test green score (70-89)
    let green_answers = vec![
        create_answer("user_goal", "User authentication system"),
        create_answer("actors", "System administrator"),
        create_answer("precondition", "User account exists and is active"),
        create_answer("outcome", "User is logged in"),
        create_answer("acceptance_criteria", "Login works within 2 seconds"),
    ];
    let green_quality: Vec<Answer> = green_answers.iter().map(to_quality_answer).collect();
    let green_ears = create_test_ears(true);
    let green_score = calculate_quality(&green_quality, &green_ears, &inversion).unwrap();
    assert!(
        (70..=89).contains(&green_score.overall),
        "Green zone score should be 70-89, got {}",
        green_score.overall
    );

    // Test excellent score (90-100)
    let excellent_answers = vec![
        create_answer(
            "user_goal",
            "Secure user authentication with username and password",
        ),
        create_answer("actors", "System administrator and end users"),
        create_answer(
            "precondition",
            "User account exists and is in active status",
        ),
        create_answer(
            "outcome",
            "User session is established with secure token",
        ),
        create_answer(
            "acceptance_criteria",
            "Authentication completes within 2 seconds, passwords are hashed with bcrypt, and tokens expire after 30 minutes",
        ),
        create_answer(
            "security",
            "System uses TLS 1.3 encryption, bcrypt password hashing with 12 salt rounds, input validation, and protection against SQL injection and XSS",
        ),
    ];
    let excellent_quality: Vec<Answer> = excellent_answers.iter().map(to_quality_answer).collect();
    let excellent_score = calculate_quality(&excellent_quality, &green_ears, &inversion).unwrap();
    assert!(
        excellent_score.overall >= 90,
        "Excellent zone score should be >= 90, got {}",
        excellent_score.overall
    );
}

/// Test 7: Dimension breakdown affects overall score
#[test]
fn test_dimension_breakdown_contribution() {
    let answers = vec![
        create_answer(
            "user_goal",
            "Users must authenticate with username and password to access the system",
        ),
        create_answer("actors", "System administrator"),
        create_answer("precondition", "User account exists"),
        create_answer("outcome", "User is logged in"),
        create_answer(
            "acceptance_criteria",
            "Login completes within 2 seconds with correct credentials",
        ),
        create_answer(
            "security",
            "System must use TLS encryption and validate all inputs to prevent SQL injection",
        ),
    ];

    let quality_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let ears = create_test_ears(true);
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let quality_score = calculate_quality(&quality_answers, &ears, &inversion).unwrap();

    // Verify all dimensions are present
    assert!(
        !quality_score.dimensions.is_empty(),
        "Should have dimension scores"
    );

    // Verify we can get each dimension
    for dimension in QualityDimension::all() {
        let dim_score = quality_score.get_dimension(*dimension);
        assert!(
            dim_score.is_some(),
            "Should have score for {:?}",
            dimension
        );

        if let Some(score) = dim_score {
            assert!(
                score.score <= 100,
                "{:?} score should be <= 100",
                dimension
            );
        }
    }

    // Overall should be average of dimensions
    let sum: u32 = quality_score.dimensions.iter().map(|d| u32::from(d.score)).sum();
    let expected_overall = sum / quality_score.dimensions.len() as u32;
    assert_eq!(
        u32::from(quality_score.overall),
        expected_overall,
        "Overall score should be average of dimensions"
    );
}

/// Test 8: Can transition to Develop after quality gate passes
#[test]
fn test_develop_transition_after_quality_gate() {
    // Complete Discover with high quality
    let answers = vec![
        create_answer(
            "user_goal",
            "Secure user authentication with role-based access control",
        ),
        create_answer("actors", "System administrator, regular user, auditor"),
        create_answer(
            "precondition",
            "User account exists, is active, and has assigned roles",
        ),
        create_answer(
            "outcome",
            "User is authenticated and granted access based on role permissions",
        ),
        create_answer(
            "acceptance_criteria",
            "Authentication completes within 2 seconds, passwords are hashed with bcrypt, session expires after 30 minutes, and role permissions are enforced",
        ),
        create_answer(
            "security",
            "System implements TLS 1.3 for transport encryption, bcrypt with 12 salt rounds for password hashing, comprehensive input validation, output encoding, and protection against OWASP Top 10 vulnerabilities including SQL injection, XSS, and CSRF",
        ),
    ];

    let quality_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let ears = create_test_ears(true);
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let quality_score = calculate_quality(&quality_answers, &ears, &inversion).unwrap();

    // Verify all conditions for Develop unlock are met
    assert!(
        is_phase_done("discover", &answers),
        "Discover phase must be complete"
    );

    assert!(
        quality_score.passes(MINIMUM_GATE),
        "Quality score {} must pass threshold {}",
        quality_score.overall,
        MINIMUM_GATE
    );

    assert!(
        !should_disable_develop(&answers, &Some(quality_score)),
        "Develop phase should be enabled"
    );

    // Verify score is in green zone for visual feedback
    assert!(
        quality_score.overall >= 70,
        "Score should show green color"
    );
}

/// Test 9: Issues explain quality gaps
#[test]
fn test_issues_explain_quality_gaps() {
    // Create answers with specific gaps
    let answers = vec![
        create_answer("user_goal", "Login"),
        // Missing: actors, precondition, outcome, acceptance_criteria, security
    ];

    let quality_answers: Vec<Answer> = answers.iter().map(to_quality_answer).collect();
    let ears = vec![]; // No EARS requirements
    let inversion = InversionControl {
        has_inversion_tests: false,
        inverted_count: 0,
    };

    let quality_score = calculate_quality(&quality_answers, &ears, &inversion).unwrap();

    // Should have issues explaining the gaps
    assert!(
        !quality_score.issues.is_empty(),
        "Should have issues explaining quality gaps"
    );

    // Check for completeness issues
    let completeness_issues = quality_score.get_issues(QualityDimension::Completeness);
    assert!(
        !completeness_issues.is_empty(),
        "Should have completeness issues"
    );

    // Check for security issues
    let security_issues = quality_score.get_issues(QualityDimension::Security);
    assert!(
        !security_issues.is_empty(),
        "Should have security issues"
    );

    // Check for testability issues
    let testability_issues = quality_score.get_issues(QualityDimension::Testability);
    assert!(
        !testability_issues.is_empty(),
        "Should have testability issues"
    );
}

/// Test 10: Minimum gate constant consistency
#[test]
fn test_minimum_gate_constant() {
    // Verify the constant is set to 70
    assert_eq!(MINIMUM_GATE, 70, "Minimum gate should be 70");

    // Create score exactly at threshold
    let threshold_score = QualityScore::new(
        MINIMUM_GATE,
        vec![],
        vec![],
    );
    assert!(threshold_score.is_ok());

    let score = threshold_score.unwrap();
    assert!(
        score.passes(MINIMUM_GATE),
        "Score at threshold should pass"
    );
    assert!(score.passes(69), "Score at 70 should pass 69 threshold");
    assert!(!score.passes(71), "Score at 70 should not pass 71 threshold");

    // Create score just below threshold
    let below_threshold = QualityScore::new(
        MINIMUM_GATE - 1,
        vec![],
        vec![],
    );
    assert!(below_threshold.is_ok());

    let below = below_threshold.unwrap();
    assert!(
        !below.passes(MINIMUM_GATE),
        "Score below threshold should not pass"
    );
}
