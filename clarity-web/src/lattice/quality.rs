#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

// Re-export Answer from types to use as the canonical Answer type
pub use crate::types::Answer;

/// Domain errors for quality scoring
#[derive(Debug, Error, PartialEq, Clone)]
pub enum QualityError {
    #[error("empty answers provided")]
    EmptyAnswers,

    #[error("invalid score value: {0}")]
    InvalidScore(String),

    #[error("dimension calculation failed: {0}")]
    DimensionFailed(String),
}

/// Quality dimensions evaluated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum QualityDimension {
    /// Percentage of required fields filled
    Completeness,

    /// Detection of contradictory requirements
    Consistency,

    /// Presence of acceptance criteria in EARS requirements
    Testability,

    /// Sentence complexity and jargon density
    Clarity,

    /// Security considerations (auth, encryption, validation)
    Security,
}

impl QualityDimension {
    /// All dimensions
    pub fn all() -> &'static [QualityDimension] {
        &[
            QualityDimension::Completeness,
            QualityDimension::Consistency,
            QualityDimension::Testability,
            QualityDimension::Clarity,
            QualityDimension::Security,
        ]
    }

    /// Display label
    pub fn label(&self) -> &'static str {
        match self {
            QualityDimension::Completeness => "Completeness",
            QualityDimension::Consistency => "Consistency",
            QualityDimension::Testability => "Testability",
            QualityDimension::Clarity => "Clarity",
            QualityDimension::Security => "Security",
        }
    }

    /// Description of what this dimension measures
    pub fn description(&self) -> &'static str {
        match self {
            QualityDimension::Completeness => "Percentage of required fields filled",
            QualityDimension::Consistency => "Absence of contradictory requirements",
            QualityDimension::Testability => "Presence of acceptance criteria",
            QualityDimension::Clarity => "Readability and minimal jargon",
            QualityDimension::Security => "Security considerations present",
        }
    }
}

/// Score for a single dimension (0-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionScore {
    pub dimension: QualityDimension,
    pub score: u8,
}

impl DimensionScore {
    /// Create a new dimension score, validating the range
    pub fn new(dimension: QualityDimension, score: u8) -> Result<Self, QualityError> {
        match score {
            0..=100 => Ok(DimensionScore { dimension, score }),
            invalid => Err(QualityError::InvalidScore(invalid.to_string())),
        }
    }

    /// Check if score passes threshold
    pub fn passes(&self, threshold: u8) -> bool {
        self.score >= threshold
    }
}

/// Issue explaining a low score
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityIssue {
    pub dimension: QualityDimension,
    pub severity: IssueSeverity,
    pub message: String,
}

impl QualityIssue {
    pub fn new(dimension: QualityDimension, severity: IssueSeverity, message: String) -> Self {
        Self {
            dimension,
            severity,
            message,
        }
    }
}

/// Severity of a quality issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Warning,
    Error,
    Critical,
}

/// Overall quality assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityScore {
    /// Overall score 0-100 (average of dimensions)
    pub overall: u8,
    /// Individual dimension scores
    pub dimensions: Vec<DimensionScore>,
    /// Issues explaining low scores
    pub issues: Vec<QualityIssue>,
}

impl QualityScore {
    /// Create a new quality score
    pub fn new(
        overall: u8,
        dimensions: Vec<DimensionScore>,
        issues: Vec<QualityIssue>,
    ) -> Result<Self, QualityError> {
        match overall {
            0..=100 => Ok(QualityScore {
                overall,
                dimensions,
                issues,
            }),
            invalid => Err(QualityError::InvalidScore(invalid.to_string())),
        }
    }

    /// Check if overall score passes threshold
    pub fn passes(&self, threshold: u8) -> bool {
        self.overall >= threshold
    }

    /// Get score for a specific dimension
    pub fn get_dimension(&self, dimension: QualityDimension) -> Option<&DimensionScore> {
        self.dimensions
            .iter()
            .find(|d| d.dimension == dimension)
    }

    /// Get issues for a specific dimension
    pub fn get_issues(&self, dimension: QualityDimension) -> Vec<&QualityIssue> {
        self.issues
            .iter()
            .filter(|i| i.dimension == dimension)
            .collect()
    }
}

/// EARS requirement reference for quality scoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EarsRequirementRef {
    pub id: String,
    pub text: String,
    pub has_acceptance_criteria: bool,
}

/// Inversion control (requirement inversion for testing)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InversionControl {
    pub has_inversion_tests: bool,
    pub inverted_count: usize,
}

/// Calculate quality score from requirements data
///
/// # Arguments
/// * `answers` - User answers to prompt steps
/// * `ears` - EARS-formatted requirements
/// * `inversion` - Inversion control data
///
/// # Returns
/// Quality score with all dimensions evaluated
pub fn calculate_quality(
    answers: &[Answer],
    ears: &[EarsRequirementRef],
    _inversion: &InversionControl,
) -> Result<QualityScore, QualityError> {
    if answers.is_empty() {
        return Err(QualityError::EmptyAnswers);
    }

    let mut all_issues = Vec::new();

    // Calculate each dimension
    let completeness = calculate_completeness(answers, &mut all_issues);
    let consistency = calculate_consistency(answers, &mut all_issues);
    let testability = calculate_testability(ears, &mut all_issues);
    let clarity = calculate_clarity(answers, &mut all_issues);
    let security = calculate_security(answers, &mut all_issues);

    let dimensions = vec![completeness, consistency, testability, clarity, security];

    // Overall = average of 5 dimensions
    let overall = dimensions
        .iter()
        .map(|d| u32::from(d.score))
        .sum::<u32>()
        / dimensions.len() as u32;

    let overall = u8::try_from(overall).map_err(|_| {
        QualityError::InvalidScore("overall calculation overflow".to_string())
    })?;

    QualityScore::new(overall, dimensions, all_issues)
}

/// Calculate completeness: % of required fields filled
fn calculate_completeness(
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
) -> DimensionScore {
    let required_patterns = [
        "user_goal",
        "actors",
        "precondition",
        "outcome",
        "acceptance_criteria",
    ];

    let total_required = required_patterns.len();

    let filled_count = required_patterns
        .iter()
        .filter(|pattern| {
            answers
                .iter()
                .any(|a| a.step_id.contains(*pattern) && !a.value.trim().is_empty())
        })
        .count();

    // Check for empty required fields
    for pattern in &required_patterns {
        let has_answer = answers
            .iter()
            .any(|a| a.step_id.contains(pattern) && !a.value.trim().is_empty());

        if !has_answer {
            issues.push(QualityIssue::new(
                QualityDimension::Completeness,
                IssueSeverity::Error,
                format!("Missing required field: {pattern}"),
            ));
        }
    }

    let score = if total_required > 0 {
        u8::try_from((filled_count * 100) / total_required).unwrap_or(100)
    } else {
        100
    };

    DimensionScore::new(QualityDimension::Completeness, score)
        .unwrap_or_else(|_| DimensionScore {
            dimension: QualityDimension::Completeness,
            score: 0,
        })
}

/// Calculate consistency: detect contradictions
fn calculate_consistency(
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
) -> DimensionScore {
    let mut contradictions = 0;
    let total_pairs = answers.len().saturating_sub(1);

    // Simple contradiction detection: look for negations of similar concepts
    let values: Vec<_> = answers
        .iter()
        .map(|a| a.value.to_lowercase())
        .collect();

    for (i, val1) in values.iter().enumerate() {
        for val2 in values.iter().skip(i + 1) {
            // Check for contradictory phrases
            if has_contradiction(val1, val2) {
                contradictions += 1;
            }
        }
    }

    // Score based on contradiction ratio
    let score = if total_pairs > 0 {
        let contradiction_ratio = (contradictions * 100) / total_pairs;
        u8::try_from(100_u32.saturating_sub(contradiction_ratio as u32)).unwrap_or(0)
    } else {
        100
    };

    if contradictions > 0 {
        issues.push(QualityIssue::new(
            QualityDimension::Consistency,
            IssueSeverity::Warning,
            format!("Found {contradictions} potential contradictions in requirements"),
        ));
    }

    DimensionScore::new(QualityDimension::Consistency, score).unwrap_or_else(|_| DimensionScore {
        dimension: QualityDimension::Consistency,
        score: 0,
    })
}

/// Check if two statements contradict each other
fn has_contradiction(val1: &str, val2: &str) -> bool {
    let contradictions = [
        ("must", "must not"),
        ("required", "optional"),
        ("always", "never"),
        ("enabled", "disabled"),
        ("allow", "deny"),
        ("include", "exclude"),
    ];

    contradictions.iter().any(|(pos, neg)| {
        (val1.contains(pos) && val2.contains(neg)) || (val1.contains(neg) && val2.contains(pos))
    })
}

/// Calculate testability: % of EARS with acceptance criteria
fn calculate_testability(
    ears: &[EarsRequirementRef],
    issues: &mut Vec<QualityIssue>,
) -> DimensionScore {
    if ears.is_empty() {
        issues.push(QualityIssue::new(
            QualityDimension::Testability,
            IssueSeverity::Error,
            "No EARS requirements defined".to_string(),
        ));
        return DimensionScore::new(QualityDimension::Testability, 0).unwrap_or_else(|_| DimensionScore {
            dimension: QualityDimension::Testability,
            score: 0,
        });
    }

    let with_criteria = ears
        .iter()
        .filter(|e| e.has_acceptance_criteria)
        .count();

    let score = u8::try_from((with_criteria * 100) / ears.len()).unwrap_or(100);

    let without = ears.len() - with_criteria;
    if without > 0 {
        issues.push(QualityIssue::new(
            QualityDimension::Testability,
            IssueSeverity::Warning,
            format!("{without} requirement(s) missing acceptance criteria"),
        ));
    }

    DimensionScore::new(QualityDimension::Testability, score).unwrap_or_else(|_| DimensionScore {
        dimension: QualityDimension::Testability,
        score: 0,
    })
}

/// Calculate clarity: sentence complexity and jargon density
fn calculate_clarity(answers: &[Answer], issues: &mut Vec<QualityIssue>) -> DimensionScore {
    let mut total_sentences = 0;
    let mut complex_sentences = 0;
    let mut jargon_count = 0;

    let jargon_terms = [
        "microservice", "kubernetes", "orchestration", "containerization",
        "blockchain", "ai/ml", "serverless", "event-driven",
    ];

    for answer in answers {
        let text = &answer.value;

        // Count sentences (rough heuristic by period/exclamation count)
        let sentence_count = text.matches(&['.', '!', '?'][..]).count().max(1);
        total_sentences += sentence_count;

        // Complex sentence: more than 3 commas or 30 words
        let comma_count = text.matches(',').count();
        let word_count = text.split_whitespace().count();

        if comma_count > 3 || word_count > 30 {
            complex_sentences += 1;
        }

        // Count jargon terms
        let lower = text.to_lowercase();
        jargon_count += jargon_terms.iter().filter(|term| lower.contains(*term)).count();
    }

    // Score = 100 - (complex_sentence_ratio + jargon_penalty)
    let complex_ratio = if total_sentences > 0 {
        (complex_sentences * 100) / total_sentences
    } else {
        0
    };

    let jargon_penalty = (jargon_count * 5).min(50);

    let score = u8::try_from(
        100_u32
            .saturating_sub(complex_ratio as u32)
            .saturating_sub(jargon_penalty as u32),
    )
    .unwrap_or(0);

    if complex_sentences > 0 {
        issues.push(QualityIssue::new(
            QualityDimension::Clarity,
            IssueSeverity::Warning,
            format!("{complex_sentences} complex sentence(s) detected (consider simplifying)"),
        ));
    }

    if jargon_count > 2 {
        issues.push(QualityIssue::new(
            QualityDimension::Clarity,
            IssueSeverity::Warning,
            format!("High jargon density ({jargon_count} terms) - consider explaining terminology"),
        ));
    }

    DimensionScore::new(QualityDimension::Clarity, score).unwrap_or_else(|_| DimensionScore {
        dimension: QualityDimension::Clarity,
        score: 0,
    })
}

/// Calculate security: auth/encryption/validation mentions
fn calculate_security(
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
) -> DimensionScore {
    let security_keywords = [
        "auth", "authentication", "authorization", "login", "password",
        "encrypt", "decrypt", "hash", "salt", "tls", "ssl", "https",
        "validat", "sanitiz", "escape", "csrf", "xss", "injection",
    ];

    let mut mentions = 0;
    let mut covered_areas = HashSet::new();

    for answer in answers {
        let lower = answer.value.to_lowercase();

        for keyword in &security_keywords {
            if lower.contains(keyword) {
                mentions += 1;

                // Categorize into areas
                if keyword.contains("auth") || keyword.contains("login") || keyword.contains("password") {
                    covered_areas.insert("authentication");
                }
                if keyword.contains("encrypt") || keyword.contains("tls") || keyword.contains("ssl") {
                    covered_areas.insert("encryption");
                }
                if keyword.contains("validat") || keyword.contains("sanitiz") || keyword.contains("escape") {
                    covered_areas.insert("validation");
                }
            }
        }
    }

    // Score based on coverage of security areas
    let coverage_score = covered_areas.len() * 30; // max 90
    let mention_bonus = mentions.min(5) * 2; // max 10
    let total = coverage_score + mention_bonus;

    let score = u8::try_from(total.min(100)).unwrap_or(100);

    if covered_areas.is_empty() {
        issues.push(QualityIssue::new(
            QualityDimension::Security,
            IssueSeverity::Error,
            "No security considerations mentioned".to_string(),
        ));
    } else if covered_areas.len() < 3 {
        let missing = ["authentication", "encryption", "validation"]
            .iter()
            .filter(|area| !covered_areas.contains(*area))
            .join(", ");

        issues.push(QualityIssue::new(
            QualityDimension::Security,
            IssueSeverity::Warning,
            format!("Security considerations incomplete: missing {missing}"),
        ));
    }

    DimensionScore::new(QualityDimension::Security, score).unwrap_or_else(|_| DimensionScore {
        dimension: QualityDimension::Security,
        score: 0,
    })
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
    fn test_dimension_score_valid_range() {
        let score = DimensionScore::new(QualityDimension::Completeness, 75);
        assert!(score.is_ok());
        if let Ok(s) = score {
            assert_eq!(s.score, 75);
        } else {
            panic!("Expected Ok");
        }
    }

    #[test]
    fn test_dimension_score_invalid_too_high() {
        let score = DimensionScore::new(QualityDimension::Completeness, 101);
        assert!(matches!(score, Err(QualityError::InvalidScore(_))));
    }

    #[test]
    fn test_dimension_score_passes_threshold() {
        let score = DimensionScore::new(QualityDimension::Completeness, 80);
        let score = match score {
            Ok(s) => s,
            Err(_) => panic!("Expected valid score"),
        };
        assert!(score.passes(70));
        assert!(!score.passes(90));
    }

    #[test]
    fn test_quality_score_passes_threshold() {
        let dimensions = vec![
            match DimensionScore::new(QualityDimension::Completeness, 80) {
                Ok(s) => s,
                Err(_) => panic!("Expected valid score"),
            },
            match DimensionScore::new(QualityDimension::Consistency, 75) {
                Ok(s) => s,
                Err(_) => panic!("Expected valid score"),
            },
            match DimensionScore::new(QualityDimension::Testability, 70) {
                Ok(s) => s,
                Err(_) => panic!("Expected valid score"),
            },
            match DimensionScore::new(QualityDimension::Clarity, 85) {
                Ok(s) => s,
                Err(_) => panic!("Expected valid score"),
            },
            match DimensionScore::new(QualityDimension::Security, 90) {
                Ok(s) => s,
                Err(_) => panic!("Expected valid score"),
            },
        ];

        let score = match QualityScore::new(80, dimensions, vec![]) {
            Ok(s) => s,
            Err(_) => panic!("Expected valid score"),
        };
        assert!(score.passes(70));
        assert!(!score.passes(90));
    }

    #[test]
    fn test_quality_score_get_dimension() {
        let dimensions = vec![
            match DimensionScore::new(QualityDimension::Completeness, 80) {
                Ok(s) => s,
                Err(_) => panic!("Expected valid score"),
            },
            match DimensionScore::new(QualityDimension::Consistency, 75) {
                Ok(s) => s,
                Err(_) => panic!("Expected valid score"),
            },
        ];

        let score = match QualityScore::new(77, dimensions, vec![]) {
            Ok(s) => s,
            Err(_) => panic!("Expected valid score"),
        };

        let completeness = score.get_dimension(QualityDimension::Completeness);
        assert!(completeness.is_some());
        if let Some(c) = completeness {
            assert_eq!(c.score, 80);
        }

        let security = score.get_dimension(QualityDimension::Security);
        assert!(security.is_none());
    }

    #[test]
    fn test_quality_score_get_issues() {
        let issues = vec![
            QualityIssue::new(
                QualityDimension::Completeness,
                IssueSeverity::Error,
                "Missing field".to_string(),
            ),
            QualityIssue::new(
                QualityDimension::Consistency,
                IssueSeverity::Warning,
                "Contradiction".to_string(),
            ),
        ];

        let score = match QualityScore::new(50, vec![], issues.clone()) {
            Ok(s) => s,
            Err(_) => panic!("Expected valid score"),
        };

        let completeness_issues = score.get_issues(QualityDimension::Completeness);
        assert_eq!(completeness_issues.len(), 1);
        assert_eq!(completeness_issues[0].severity, IssueSeverity::Error);

        let consistency_issues = score.get_issues(QualityDimension::Consistency);
        assert_eq!(consistency_issues.len(), 1);

        let security_issues = score.get_issues(QualityDimension::Security);
        assert!(security_issues.is_empty());
    }

    #[test]
    fn test_calculate_quality_empty_answers() {
        let ears = vec![];
        let inversion = InversionControl {
            has_inversion_tests: false,
            inverted_count: 0,
        };

        let result = calculate_quality(&[], &ears, &inversion);
        assert!(matches!(result, Err(QualityError::EmptyAnswers)));
    }

    #[test]
    fn test_calculate_quality_perfect_scores() {
        let answers = vec![
            create_answer("user_goal", "User must authenticate"),
            create_answer("actors", "System admin"),
            create_answer("precondition", "User exists"),
            create_answer("outcome", "Access granted"),
            create_answer("acceptance_criteria", "Login within 2 seconds"),
            create_answer(
                "security",
                "System must use TLS encryption and validate all inputs",
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
        assert!(result.is_ok());

        let score = match result {
            Ok(s) => s,
            Err(_) => panic!("Expected Ok result"),
        };
        assert_eq!(score.overall, 100); // All perfect scores

        // Check no critical issues
        let critical = score.issues.iter().filter(|i| i.severity == IssueSeverity::Critical);
        assert_eq!(critical.count(), 0);
    }

    #[test]
    fn test_calculate_completeness_missing_fields() {
        let answers = vec![
            create_answer("user_goal", "Goal"),
            // Missing actors, precondition, outcome, acceptance_criteria
        ];

        let mut issues = vec![];
        let score = calculate_completeness(&answers, &mut issues);

        // Should be 20% (1 out of 5 required fields)
        assert_eq!(score.score, 20);

        // Should have 4 issues (missing 4 fields)
        assert_eq!(issues.len(), 4);
        assert!(issues.iter().all(|i| i.dimension == QualityDimension::Completeness));
    }

    #[test]
    fn test_calculate_completeness_all_fields() {
        let answers = vec![
            create_answer("user_goal", "Goal"),
            create_answer("actors", "Admin"),
            create_answer("precondition", "Precondition"),
            create_answer("outcome", "Success"),
            create_answer("acceptance_criteria", "Criteria"),
        ];

        let mut issues = vec![];
        let score = calculate_completeness(&answers, &mut issues);

        assert_eq!(score.score, 100);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_calculate_consistency_contradictions() {
        let answers = vec![
            create_answer("req1", "User must authenticate"),
            create_answer("req2", "User must not authenticate"),
            create_answer("req3", "Data is required"),
        ];

        let mut issues = vec![];
        let score = calculate_consistency(&answers, &mut issues);

        // Should detect contradiction between "must" and "must not"
        assert!(score.score < 100);
        assert!(!issues.is_empty());
        assert!(issues[0].message.contains("contradiction"));
    }

    #[test]
    fn test_calculate_consistency_no_contradictions() {
        let answers = vec![
            create_answer("req1", "User must authenticate"),
            create_answer("req2", "Admin must authorize"),
        ];

        let mut issues = vec![];
        let score = calculate_consistency(&answers, &mut issues);

        assert_eq!(score.score, 100);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_has_contradiction() {
        assert!(has_contradiction("must allow access", "must deny access"));
        assert!(has_contradiction("always enabled", "never enabled"));
        assert!(has_contradiction("required field", "optional field"));

        assert!(!has_contradiction("must allow access", "should allow access"));
        assert!(!has_contradiction("enabled feature", "enabled setting"));
    }

    #[test]
    fn test_calculate_testability_with_criteria() {
        let ears = vec![
            create_ears("1", "Req 1", true),
            create_ears("2", "Req 2", true),
            create_ears("3", "Req 3", false), // One without
        ];

        let mut issues = vec![];
        let score = calculate_testability(&ears, &mut issues);

        // 2 out of 3 = 66%
        assert_eq!(score.score, 66);

        // Should have 1 issue
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("1"));
    }

    #[test]
    fn test_calculate_testability_no_ears() {
        let ears = vec![];
        let mut issues = vec![];
        let score = calculate_testability(&ears, &mut issues);

        assert_eq!(score.score, 0);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("No EARS"));
    }

    #[test]
    fn test_calculate_testability_all_with_criteria() {
        let ears = vec![
            create_ears("1", "Req 1", true),
            create_ears("2", "Req 2", true),
        ];

        let mut issues = vec![];
        let score = calculate_testability(&ears, &mut issues);

        assert_eq!(score.score, 100);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_calculate_clarity_complex_sentences() {
        let answers = vec![
            create_answer(
                "req1",
                "The system shall, under normal operating conditions, provided that all prerequisites are met, and assuming no external interference, process the data.",
            ),
        ];

        let mut issues = vec![];
        let score = calculate_clarity(&answers, &mut issues);

        // Complex sentence should reduce score
        assert!(score.score < 100);
        assert!(!issues.is_empty());
        assert!(issues[0].message.contains("complex"));
    }

    #[test]
    fn test_calculate_clarity_jargon() {
        let answers = vec![
            create_answer(
                "req1",
                "Implement microservice architecture with Kubernetes orchestration and serverless event-driven blockchain integration.",
            ),
        ];

        let mut issues = vec![];
        let score = calculate_clarity(&answers, &mut issues);

        // High jargon should reduce score
        assert!(score.score < 100);

        // Should have jargon issue
        let jargon_issues: Vec<_> = issues.iter().filter(|i| i.message.contains("jargon")).collect();
        assert!(!jargon_issues.is_empty());
    }

    #[test]
    fn test_calculate_clarity_perfect() {
        let answers = vec![
            create_answer("req1", "Users must log in."),
            create_answer("req2", "Data is saved securely."),
        ];

        let mut issues = vec![];
        let score = calculate_clarity(&answers, &mut issues);

        assert_eq!(score.score, 100);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_calculate_security_no_mentions() {
        let answers = vec![
            create_answer("req1", "Process data"),
            create_answer("req2", "Save results"),
        ];

        let mut issues = vec![];
        let score = calculate_security(&answers, &mut issues);

        assert_eq!(score.score, 0);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("No security"));
    }

    #[test]
    fn test_calculate_security_partial_coverage() {
        let answers = vec![
            create_answer("req1", "Users must authenticate with password"),
        ];

        let mut issues = vec![];
        let score = calculate_security(&answers, &mut issues);

        // Should have some coverage (authentication)
        assert!(score.score > 0);
        assert!(score.score < 100);

        // Should have warning about missing areas
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_calculate_security_full_coverage() {
        let answers = vec![
            create_answer(
                "req1",
                "Users authenticate with password. Data encrypted with TLS. Inputs validated and sanitized.",
            ),
        ];

        let mut issues = vec![];
        let score = calculate_security(&answers, &mut issues);

        // Should have high coverage
        assert!(score.score >= 90);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_quality_dimension_labels() {
        assert_eq!(QualityDimension::Completeness.label(), "Completeness");
        assert_eq!(QualityDimension::Consistency.label(), "Consistency");
        assert_eq!(QualityDimension::Testability.label(), "Testability");
        assert_eq!(QualityDimension::Clarity.label(), "Clarity");
        assert_eq!(QualityDimension::Security.label(), "Security");
    }

    #[test]
    fn test_quality_dimension_descriptions() {
        for dim in QualityDimension::all() {
            let desc = dim.description();
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_quality_dimension_all() {
        let all = QualityDimension::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&QualityDimension::Completeness));
        assert!(all.contains(&QualityDimension::Consistency));
        assert!(all.contains(&QualityDimension::Testability));
        assert!(all.contains(&QualityDimension::Clarity));
        assert!(all.contains(&QualityDimension::Security));
    }

    #[test]
    fn test_overall_score_calculation() {
        let answers = vec![
            create_answer("user_goal", "Goal"),
            create_answer("actors", "Admin"),
            create_answer("precondition", "Precondition"),
            create_answer("outcome", "Success"),
            create_answer("acceptance_criteria", "Criteria"),
        ];

        let ears = vec![
            create_ears("1", "Req 1", true),
        ];

        let inversion = InversionControl {
            has_inversion_tests: false,
            inverted_count: 0,
        };

        let result = calculate_quality(&answers, &ears, &inversion);
        assert!(result.is_ok());

        let score = match result {
            Ok(s) => s,
            Err(_) => panic!("Expected Ok result"),
        };

        // With 100% completeness, 100% consistency (single answer), 100% testability,
        // 100% clarity, and 0% security = 80% average
        assert_eq!(score.overall, 80);

        // Should have security issue
        let security_issues = score.get_issues(QualityDimension::Security);
        assert!(!security_issues.is_empty());
    }

    #[test]
    fn test_issue_severity_variants() {
        let error = QualityIssue::new(
            QualityDimension::Completeness,
            IssueSeverity::Error,
            "Error message".to_string(),
        );

        let warning = QualityIssue::new(
            QualityDimension::Consistency,
            IssueSeverity::Warning,
            "Warning message".to_string(),
        );

        let critical = QualityIssue::new(
            QualityDimension::Security,
            IssueSeverity::Critical,
            "Critical message".to_string(),
        );

        assert_eq!(error.severity, IssueSeverity::Error);
        assert_eq!(warning.severity, IssueSeverity::Warning);
        assert_eq!(critical.severity, IssueSeverity::Critical);
    }
}
