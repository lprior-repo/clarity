//! Quality Submodule
//!
//! Quality analysis and improvement including:
//! - Coverage, clarity, testability, AI-readiness scoring
//! - Second-order effect detection (WP29)
//! - Spec improvement suggestions (WP30)
//! - Spec linting

pub mod analyzer;
pub mod effects;
pub mod improver;

// Re-export main types for convenience
pub use analyzer::{
    analyze_spec, calculate_ai_readiness_score, calculate_clarity_score, calculate_coverage_score,
    calculate_overall_score, calculate_testability_score, format_report, QualityIssue,
    QualityReport,
};

// Re-export effects types (WP29)
pub use effects::{
    analyze_behavior, analyze_behavior_report, analyze_feature,
    analyze_spec as analyze_spec_effects, behaviors_with_effect_type, count_effects_by_type,
    has_critical_effects, has_high_severity_effects, max_effect_severity, Effect, EffectSeverity,
    EffectType, EffectsError, EffectsReport, EffectsResult, EffectsSummary, SpecEffectsReport,
};

// Re-export improver types (WP30)
// Note: improver::QualityReport is renamed to ImproverQualityReport to avoid conflict
// with analyzer::QualityReport
pub use improver::{
    suggest_improvements, suggest_missing_tests, suggest_vague_rules_improvements,
    suggest_examples_improvements, ImprovementSuggestion, ImproverError,
    IssueCategory, QualityIssueReport as ImproverIssueReport,
    QualityReport as ImproverQualityReport,
};
