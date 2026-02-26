#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Example usage of the quality scoring module.
//!
//! This demonstrates how to:
//! - Calculate quality scores from requirements
//! - Apply configurable gate thresholds
//! - Extract and display quality issues

use crate::lattice::quality::{
    calculate_quality, Answer, EarsRequirementRef, InversionControl, IssueSeverity,
    QualityError,
};

/// Default quality gate threshold (70%)
#[allow(dead_code)]
const DEFAULT_QUALITY_GATE: u8 = 70;

/// Check if requirements pass quality gate
///
/// # Arguments
/// * `threshold` - Minimum overall score (0-100, default 70)
///
/// # Returns
/// `Ok(())` if passing, `Err` with quality issues if failing
#[allow(dead_code)]
pub fn check_quality_gate(
    answers: &[Answer],
    ears: &[EarsRequirementRef],
    inversion: &InversionControl,
    threshold: Option<u8>,
) -> Result<(), QualityError> {
    let gate_threshold = threshold.unwrap_or(DEFAULT_QUALITY_GATE);

    let score = calculate_quality(answers, ears, inversion)?;

    if score.passes(gate_threshold) {
        Ok(())
    } else {
        // Collect blocking issues
        let blocking_issues: Vec<_> = score
            .issues
            .iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Error | IssueSeverity::Critical))
            .map(|i| format!("{}: {}", i.dimension.label(), i.message))
            .collect();

        if blocking_issues.is_empty() {
            // Score below threshold but no critical issues
            Err(QualityError::DimensionFailed(format!(
                "Overall score {}% below threshold {}%",
                score.overall, gate_threshold
            )))
        } else {
            // Critical issues present
            Err(QualityError::DimensionFailed(format!(
                "Quality gate failed - {} blocking issue(s): {}",
                blocking_issues.len(),
                blocking_issues.join("; ")
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_answer(step_id: &str, value: &str) -> Answer {
        Answer {
            step_id: step_id.to_string(),
            value: value.to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    fn create_ears(id: &str, text: &str, has_criteria: bool) -> EarsRequirementRef {
        EarsRequirementRef {
            id: id.to_string(),
            text: text.to_string(),
            has_acceptance_criteria: has_criteria,
        }
    }

    #[test]
    fn test_quality_gate_pass() {
        let answers = vec![
            create_answer("user_goal", "Goal"),
            create_answer("actors", "Admin"),
            create_answer("precondition", "Precondition"),
            create_answer("outcome", "Success"),
            create_answer("acceptance_criteria", "Criteria"),
            create_answer(
                "security",
                "System uses TLS encryption and validates all inputs with authentication",
            ),
        ];

        let ears = vec![
            create_ears("1", "Req 1", true),
            create_ears("2", "Req 2", true),
        ];

        let inversion = InversionControl {
            has_inversion_tests: true,
            inverted_count: 2,
        };

        // Should pass with threshold 70 (score should be 100)
        let result = check_quality_gate(&answers, &ears, &inversion, Some(70));
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_gate_fail_low_score() {
        let answers = vec![
            create_answer("user_goal", "Minimal goal"), // Missing most fields
        ];

        let ears = vec![]; // No requirements

        let inversion = InversionControl {
            has_inversion_tests: false,
            inverted_count: 0,
        };

        // Should fail with threshold 70 (score will be very low)
        let result = check_quality_gate(&answers, &ears, &inversion, Some(70));
        assert!(result.is_err());

        if let Err(QualityError::DimensionFailed(msg)) = result {
            assert!(msg.contains("below threshold") || msg.contains("Quality gate failed"));
        } else {
            panic!("Expected DimensionFailed error");
        }
    }

    #[test]
    fn test_quality_gate_default_threshold() {
        let answers = vec![
            create_answer("user_goal", "Goal"),
            create_answer("actors", "Admin"),
        ];

        let ears = vec![];

        let inversion = InversionControl {
            has_inversion_tests: false,
            inverted_count: 0,
        };

        // Uses default threshold of 70
        let result = check_quality_gate(&answers, &ears, &inversion, None);
        // Should fail due to missing security, completeness issues
        assert!(result.is_err());
    }

    #[test]
    fn test_quality_gate_permissive_threshold() {
        let answers = vec![
            create_answer("user_goal", "Goal"),
        ];

        let ears = vec![];

        let inversion = InversionControl {
            has_inversion_tests: false,
            inverted_count: 0,
        };

        // Very permissive threshold - should pass
        let result = check_quality_gate(&answers, &ears, &inversion, Some(10));
        assert!(result.is_ok());
    }
}
