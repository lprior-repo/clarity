#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Brutal Truths Prioritizer with VORP (Value Over Replacement Product).
//!
//! This module implements the Four Brutal Truths framework for product management:
//! 1. **Scale is hard**: Growing user base is exponentially difficult
//! 2. **User value back-loaded**: Most value comes after sustained use
//! 3. **Competitive differentiation back-loaded**: True differentiation emerges over time
//! 4. **Sustaining value is hard**: Maintaining value as users/competition evolve
//!
//! # VORP Calculator
//!
//! Value Over Replacement Product (VORP) measures how much better your product is
//! compared to the next best alternative.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Four Brutal Truths
// ============================================================================

/// The Four Brutal Truths of product management.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrutalTruth {
    /// Scale is hard - growing user base is exponentially difficult
    ScaleIsHard,
    /// User value is back-loaded - most value comes after sustained use
    UserValueBackLoaded,
    /// Competitive differentiation is back-loaded - true differentiation emerges over time
    DifferentiationBackLoaded,
    /// Sustaining value is hard - maintaining value as users/competition evolve
    SustainingValueHard,
}

impl BrutalTruth {
    /// Get the name of this brutal truth.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::ScaleIsHard => "Scale is Hard",
            Self::UserValueBackLoaded => "User Value is Back-Loaded",
            Self::DifferentiationBackLoaded => "Differentiation is Back-Loaded",
            Self::SustainingValueHard => "Sustaining Value is Hard",
        }
    }

    /// Get a description of this brutal truth.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::ScaleIsHard => "Growing user base is exponentially difficult. Each 10x growth requires fundamentally different strategies.",
            Self::UserValueBackLoaded => "Most value comes after sustained use. Early experiences often underrepresent true value.",
            Self::DifferentiationBackLoaded => "True differentiation emerges over time. Initial features are easily copied.",
            Self::SustainingValueHard => "Maintaining value as users and competition evolve requires continuous investment.",
        }
    }

    /// Get all four brutal truths.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::ScaleIsHard,
            Self::UserValueBackLoaded,
            Self::DifferentiationBackLoaded,
            Self::SustainingValueHard,
        ]
    }

    /// Get assessment questions for this truth.
    #[must_use]
    pub fn assessment_questions(&self) -> Vec<&'static str> {
        match self {
            Self::ScaleIsHard => vec![
                "What is your current user base scale?",
                "What scale are you targeting?",
                "Do you have a clear path to 10x growth?",
                "Are your strategies appropriate for your current scale?",
            ],
            Self::UserValueBackLoaded => vec![
                "How long until users experience core value?",
                "What is the 'aha moment' for your product?",
                "How do you bridge users to sustained value?",
                "What metrics indicate users are reaching back-loaded value?",
            ],
            Self::DifferentiationBackLoaded => vec![
                "What can competitors copy in 30 days?",
                "What takes 6+ months to replicate?",
                "What is your compounding advantage?",
                "How does differentiation increase over time?",
            ],
            Self::SustainingValueHard => vec![
                "How will user needs evolve?",
                "How will competitive landscape change?",
                "What ongoing investment is required?",
                "What creates lock-in or switching costs?",
            ],
        }
    }
}

// ============================================================================
// VORP Score
// ============================================================================

/// VORP (Value Over Replacement Product) score.
///
/// Measures how much better your product is compared to the next best alternative.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VorpScore {
    /// Overall VORP score (0.0-1.0)
    pub score: f64,
    /// Value dimension score
    pub value_score: f64,
    /// Obvious dimension score
    pub obvious_score: f64,
    /// Real dimension score
    pub real_score: f64,
    /// Possible dimension score
    pub possible_score: f64,
    /// Comparison to replacement product
    pub replacement_comparison: String,
    /// Justification for the score
    pub justification: String,
}

impl VorpScore {
    /// Create a new VORP score.
    #[must_use]
    pub fn new(
        value_score: f64,
        obvious_score: f64,
        real_score: f64,
        possible_score: f64,
    ) -> Self {
        let score = (value_score + obvious_score + real_score + possible_score) / 4.0;

        Self {
            score: score.clamp(0.0, 1.0),
            value_score: value_score.clamp(0.0, 1.0),
            obvious_score: obvious_score.clamp(0.0, 1.0),
            real_score: real_score.clamp(0.0, 1.0),
            possible_score: possible_score.clamp(0.0, 1.0),
            replacement_comparison: String::new(),
            justification: String::new(),
        }
    }

    /// Add replacement product comparison.
    #[must_use]
    pub fn with_replacement(mut self, comparison: String) -> Self {
        self.replacement_comparison = comparison;
        self
    }

    /// Add justification.
    #[must_use]
    pub fn with_justification(mut self, justification: String) -> Self {
        self.justification = justification;
        self
    }

    /// Check if VORP passes threshold.
    #[must_use]
    pub fn passes(&self) -> bool {
        self.score >= 0.5
    }

    /// Get the weakest dimension.
    #[must_use]
    pub fn weakest_dimension(&self) -> (&'static str, f64) {
        let dimensions = [
            ("Value", self.value_score),
            ("Obvious", self.obvious_score),
            ("Real", self.real_score),
            ("Possible", self.possible_score),
        ];

        dimensions
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|&(name, score)| (name, score))
            .unwrap_or(("Value", 0.0))
    }

    /// Get recommendations based on weak dimensions.
    #[must_use]
    pub fn recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.value_score < 0.5 {
            recommendations.push(
                "Focus on demonstrating clear, measurable value to users".to_string(),
            );
        }
        if self.obvious_score < 0.5 {
            recommendations.push(
                "Make the value proposition immediately apparent to new users".to_string(),
            );
        }
        if self.real_score < 0.5 {
            recommendations.push(
                "Validate that users and problem are real through research".to_string(),
            );
        }
        if self.possible_score < 0.5 {
            recommendations.push(
                "Assess technical and resource feasibility more carefully".to_string(),
            );
        }

        recommendations
    }
}

impl Default for VorpScore {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

// ============================================================================
// VORP Calculator
// ============================================================================

/// Calculator for VORP scores.
pub struct VorpCalculator {
    /// Value assessment (0.0-1.0)
    value: f64,
    /// Obvious assessment (0.0-1.0)
    obvious: f64,
    /// Real assessment (0.0-1.0)
    real: f64,
    /// Possible assessment (0.0-1.0)
    possible: f64,
    /// Replacement product description
    replacement: String,
    /// Justification notes
    justification: String,
}

impl VorpCalculator {
    /// Create a new VORP calculator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: 0.0,
            obvious: 0.0,
            real: 0.0,
            possible: 0.0,
            replacement: String::new(),
            justification: String::new(),
        }
    }

    /// Set value dimension score.
    #[must_use]
    pub fn with_value(mut self, score: f64) -> Self {
        self.value = score.clamp(0.0, 1.0);
        self
    }

    /// Set obvious dimension score.
    #[must_use]
    pub fn with_obvious(mut self, score: f64) -> Self {
        self.obvious = score.clamp(0.0, 1.0);
        self
    }

    /// Set real dimension score.
    #[must_use]
    pub fn with_real(mut self, score: f64) -> Self {
        self.real = score.clamp(0.0, 1.0);
        self
    }

    /// Set possible dimension score.
    #[must_use]
    pub fn with_possible(mut self, score: f64) -> Self {
        self.possible = score.clamp(0.0, 1.0);
        self
    }

    /// Set replacement product.
    #[must_use]
    pub fn with_replacement(mut self, replacement: String) -> Self {
        self.replacement = replacement;
        self
    }

    /// Set justification.
    #[must_use]
    pub fn with_justification(mut self, justification: String) -> Self {
        self.justification = justification;
        self
    }

    /// Calculate the VORP score.
    #[must_use]
    pub fn calculate(&self) -> VorpScore {
        VorpScore::new(self.value, self.obvious, self.real, self.possible)
            .with_replacement(self.replacement.clone())
            .with_justification(self.justification.clone())
    }

    /// Quick assessment based on item description.
    #[must_use]
    pub fn quick_assess(description: &str, replacement: &str) -> VorpScore {
        let lower = description.to_lowercase();

        // Assess value based on keywords
        let value = if contains_any(&lower, &["critical", "essential", "must-have", "vital"]) {
            0.9
        } else if contains_any(&lower, &["important", "valuable", "useful", "helpful"]) {
            0.7
        } else if contains_any(&lower, &["nice", "would be", "could", "maybe"]) {
            0.4
        } else {
            0.5
        };

        // Assess obviousness
        let obvious = if contains_any(&lower, &["simple", "intuitive", "obvious", "clear"]) {
            0.8
        } else if contains_any(&lower, &["complex", "confusing", "hidden", "unclear"]) {
            0.3
        } else {
            0.5
        };

        // Assess reality
        let real = if contains_any(&lower, &["validated", "proven", "research", "users want"]) {
            0.8
        } else if contains_any(&lower, &["hypothesis", "assume", "might", "perhaps"]) {
            0.4
        } else {
            0.5
        };

        // Assess possibility
        let possible = if contains_any(&lower, &["easy", "straightforward", "simple", "ready"]) {
            0.8
        } else if contains_any(&lower, &["difficult", "complex", "challenging", "risky"]) {
            0.4
        } else {
            0.5
        };

        VorpScore::new(value, obvious, real, possible)
            .with_replacement(replacement.to_string())
            .with_justification(format!("Quick assessment based on: {}", description))
    }
}

impl Default for VorpCalculator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Prioritized Item
// ============================================================================

/// An item that has been prioritized using Brutal Truths and VORP.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrioritizedItem {
    /// Item identifier
    pub id: String,
    /// Item title/description
    pub title: String,
    /// VORP score
    pub vorp_score: VorpScore,
    /// Brutal truth scores (truth -> score)
    pub brutal_truth_scores: Vec<(BrutalTruth, f64)>,
    /// Overall priority (0-100, higher = more important)
    pub priority: u8,
    /// Risk level (0-100, higher = more risky)
    pub risk: u8,
    /// Category for grouping
    pub category: String,
    /// Dependencies on other items
    pub dependencies: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl PrioritizedItem {
    /// Create a new prioritized item.
    #[must_use]
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            vorp_score: VorpScore::default(),
            brutal_truth_scores: BrutalTruth::all()
                .iter()
                .map(|&t| (t, 0.5))
                .collect(),
            priority: 50,
            risk: 50,
            category: String::new(),
            dependencies: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Set VORP score.
    #[must_use]
    pub fn with_vorp(mut self, score: VorpScore) -> Self {
        self.vorp_score = score;
        self
    }

    /// Set brutal truth score.
    #[must_use]
    pub fn with_brutal_truth_score(mut self, truth: BrutalTruth, score: f64) -> Self {
        for (t, s) in &mut self.brutal_truth_scores {
            if *t == truth {
                *s = score.clamp(0.0, 1.0);
                break;
            }
        }
        self
    }

    /// Set priority.
    #[must_use]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set risk level.
    #[must_use]
    pub fn with_risk(mut self, risk: u8) -> Self {
        self.risk = risk;
        self
    }

    /// Set category.
    #[must_use]
    pub fn with_category(mut self, category: String) -> Self {
        self.category = category;
        self
    }

    /// Add dependency.
    #[must_use]
    pub fn with_dependency(mut self, dep_id: String) -> Self {
        self.dependencies.push(dep_id);
        self
    }

    /// Calculate adjusted priority based on VORP and brutal truths.
    #[must_use]
    pub fn calculate_adjusted_priority(&self) -> u8 {
        // Base priority
        let base = f64::from(self.priority);

        // VORP adjustment (0.0-1.0 -> -20 to +20)
        let vorp_adj = (self.vorp_score.score - 0.5) * 40.0;

        // Brutal truths adjustment (average of truth scores)
        let truth_avg = self
            .brutal_truth_scores
            .iter()
            .map(|(_, s)| s)
            .sum::<f64>()
            / f64::from(u8::try_from(self.brutal_truth_scores.len()).unwrap_or(4));
        let truth_adj = (truth_avg - 0.5) * 20.0;

        // Risk penalty
        let risk_penalty = -f64::from(self.risk) * 0.1;

        let adjusted = base + vorp_adj + truth_adj + risk_penalty;
        adjusted.clamp(0.0, 100.0) as u8
    }

    /// Get score for a specific brutal truth.
    #[must_use]
    pub fn get_truth_score(&self, truth: BrutalTruth) -> f64 {
        self.brutal_truth_scores
            .iter()
            .find(|(t, _)| *t == truth)
            .map(|(_, s)| *s)
            .unwrap_or(0.0)
    }

    /// Identify the biggest concern based on brutal truths.
    #[must_use]
    pub fn biggest_concern(&self) -> Option<(BrutalTruth, f64)> {
        self.brutal_truth_scores
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(t, s)| (*t, *s))
    }
}

// ============================================================================
// Output
// ============================================================================

/// Output from the Brutal Truths Prioritizer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BrutalTruthsOutput {
    /// Prioritized items
    pub items: Vec<PrioritizedItem>,
    /// Summary statistics
    pub stats: BrutalTruthsStats,
    /// Recommendations for the prioritization
    pub recommendations: Vec<String>,
    /// Brutal truth averages across all items
    pub truth_averages: Vec<(BrutalTruth, f64)>,
}

/// Statistics from the prioritization process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BrutalTruthsStats {
    /// Total items prioritized
    pub total_items: usize,
    /// Average priority score
    pub avg_priority: f64,
    /// Average VORP score
    pub avg_vorp: f64,
    /// Average risk level
    pub avg_risk: f64,
    /// High priority items (priority >= 70)
    pub high_priority_count: usize,
    /// Low risk items (risk <= 30)
    pub low_risk_count: usize,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the Brutal Truths Prioritizer.
#[derive(Debug, Error)]
pub enum PrioritizerError {
    /// No items to prioritize
    #[error("No items provided for prioritization")]
    EmptyInput,

    /// Invalid score
    #[error("Invalid score {score} for dimension {dimension}")]
    InvalidScore {
        /// The invalid score
        score: f64,
        /// The dimension name
        dimension: String,
    },
}

// ============================================================================
// Prioritizer Implementation
// ============================================================================

/// The Brutal Truths Prioritizer.
///
/// Applies the Four Brutal Truths framework and VORP scoring to prioritize items.
pub struct BrutalTruthsPrioritizer;

impl BrutalTruthsPrioritizer {
    /// Prioritize a list of items.
    ///
    /// # Errors
    ///
    /// Returns an error if no items are provided.
    pub fn prioritize(items: Vec<PrioritizedItem>) -> Result<BrutalTruthsOutput, PrioritizerError> {
        if items.is_empty() {
            return Err(PrioritizerError::EmptyInput);
        }

        // Calculate adjusted priorities
        let adjusted_items: Vec<PrioritizedItem> = items
            .into_iter()
            .map(|mut item| {
                let adjusted = item.calculate_adjusted_priority();
                item.priority = adjusted;
                item.recommendations = Self::generate_item_recommendations(&item);
                item
            })
            .sorted_by(|a, b| b.priority.cmp(&a.priority))
            .collect();

        // Calculate statistics
        let stats = Self::calculate_stats(&adjusted_items);

        // Calculate brutal truth averages
        let truth_averages = Self::calculate_truth_averages(&adjusted_items);

        // Generate overall recommendations
        let recommendations = Self::generate_recommendations(&adjusted_items, &truth_averages);

        Ok(BrutalTruthsOutput {
            items: adjusted_items,
            stats,
            recommendations,
            truth_averages,
        })
    }

    /// Quick prioritize a list of titles with automatic assessment.
    ///
    /// # Errors
    ///
    /// Returns an error if no items are provided.
    pub fn quick_prioritize(
        items: Vec<(String, String)>,
    ) -> Result<BrutalTruthsOutput, PrioritizerError> {
        if items.is_empty() {
            return Err(PrioritizerError::EmptyInput);
        }

        let prioritized: Vec<PrioritizedItem> = items
            .into_iter()
            .enumerate()
            .map(|(_idx, (id, title))| {
                let vorp = VorpCalculator::quick_assess(&title, "competitor");

                // Assess brutal truths based on title keywords
                let lower = title.to_lowercase();
                let mut item = PrioritizedItem::new(id.clone(), title.clone())
                    .with_vorp(vorp);

                // Scale assessment
                let scale_score = if contains_any(&lower, &["scale", "growth", "viral", "network"]) {
                    0.6
                } else {
                    0.7
                };
                item = item.with_brutal_truth_score(BrutalTruth::ScaleIsHard, scale_score);

                // User value assessment
                let user_value = if contains_any(&lower, &["onboarding", "first time", "quick"]) {
                    0.7
                } else {
                    0.5
                };
                item = item.with_brutal_truth_score(BrutalTruth::UserValueBackLoaded, user_value);

                // Differentiation assessment
                let diff_score = if contains_any(&lower, &["unique", "proprietary", "patent", "exclusive"]) {
                    0.8
                } else if contains_any(&lower, &["common", "standard", "typical"]) {
                    0.3
                } else {
                    0.5
                };
                item = item.with_brutal_truth_score(BrutalTruth::DifferentiationBackLoaded, diff_score);

                // Sustaining value assessment
                let sustain_score = if contains_any(&lower, &["long-term", "sustainable", "ongoing", "continuous"]) {
                    0.7
                } else {
                    0.5
                };
                item = item.with_brutal_truth_score(BrutalTruth::SustainingValueHard, sustain_score);

                // Set priority based on VORP
                let base_priority = (item.vorp_score.score * 80.0) as u8;
                item = item.with_priority(base_priority);

                // Set risk based on brutal truth scores
                let risk = (100.0 - item.brutal_truth_scores.iter().map(|(_, s)| s).sum::<f64>() * 25.0) as u8;
                item = item.with_risk(risk.min(100));

                item
            })
            .collect();

        Self::prioritize(prioritized)
    }

    /// Calculate statistics from prioritized items.
    fn calculate_stats(items: &[PrioritizedItem]) -> BrutalTruthsStats {
        let total = items.len();
        let avg_priority = items.iter().map(|i| f64::from(i.priority)).sum::<f64>() / f64::from(u8::try_from(total).unwrap_or(1));
        let avg_vorp = items.iter().map(|i| i.vorp_score.score).sum::<f64>() / f64::from(u8::try_from(total).unwrap_or(1));
        let avg_risk = items.iter().map(|i| f64::from(i.risk)).sum::<f64>() / f64::from(u8::try_from(total).unwrap_or(1));

        BrutalTruthsStats {
            total_items: total,
            avg_priority,
            avg_vorp,
            avg_risk,
            high_priority_count: items.iter().filter(|i| i.priority >= 70).count(),
            low_risk_count: items.iter().filter(|i| i.risk <= 30).count(),
        }
    }

    /// Calculate average scores for each brutal truth.
    fn calculate_truth_averages(items: &[PrioritizedItem]) -> Vec<(BrutalTruth, f64)> {
        BrutalTruth::all()
            .iter()
            .map(|&truth| {
                let avg = items
                    .iter()
                    .map(|item| item.get_truth_score(truth))
                    .sum::<f64>()
                    / f64::from(u8::try_from(items.len()).unwrap_or(1));
                (truth, avg)
            })
            .collect()
    }

    /// Generate recommendations for a single item.
    fn generate_item_recommendations(item: &PrioritizedItem) -> Vec<String> {
        let mut recs = Vec::new();

        // VORP-based recommendations
        recs.extend(item.vorp_score.recommendations());

        // Brutal truth recommendations
        if let Some((truth, score)) = item.biggest_concern() {
            if score < 0.5 {
                recs.push(format!(
                    "Address '{}' concern: {}",
                    truth.name(),
                    truth.description()
                ));
            }
        }

        // Risk-based recommendations
        if item.risk > 70 {
            recs.push("High risk item - consider breaking down or de-risking".to_string());
        }

        recs
    }

    /// Generate overall recommendations.
    fn generate_recommendations(
        items: &[PrioritizedItem],
        truth_averages: &[(BrutalTruth, f64)],
    ) -> Vec<String> {
        let mut recs = Vec::new();

        // Check for weak brutal truths
        for (truth, avg) in truth_averages {
            if *avg < 0.5 {
                recs.push(format!(
                    "Portfolio weakness: '{}' scores low ({:.1}). Consider more items that address this.",
                    truth.name(),
                    avg
                ));
            }
        }

        // Check for high-risk concentration
        let high_risk_count = items.iter().filter(|i| i.risk > 70).count();
        if high_risk_count > items.len() / 2 {
            recs.push(
                "Warning: More than half of items are high-risk. Consider de-risking strategies."
                    .to_string(),
            );
        }

        // Check for low VORP items in high priority
        let low_vorp_high_priority = items
            .iter()
            .filter(|i| i.priority >= 70 && i.vorp_score.score < 0.5)
            .count();
        if low_vorp_high_priority > 0 {
            recs.push(format!(
                "{} high-priority items have low VORP. Reassess their true value.",
                low_vorp_high_priority
            ));
        }

        recs
    }
}

/// Helper function to check if text contains any keywords.
fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|&k| text.contains(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brutal_truth_all() {
        let all = BrutalTruth::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&BrutalTruth::ScaleIsHard));
        assert!(all.contains(&BrutalTruth::UserValueBackLoaded));
        assert!(all.contains(&BrutalTruth::DifferentiationBackLoaded));
        assert!(all.contains(&BrutalTruth::SustainingValueHard));
    }

    #[test]
    fn test_brutal_truth_descriptions() {
        assert!(!BrutalTruth::ScaleIsHard.description().is_empty());
        assert!(!BrutalTruth::UserValueBackLoaded.description().is_empty());
    }

    #[test]
    fn test_brutal_truth_questions() {
        let questions = BrutalTruth::ScaleIsHard.assessment_questions();
        assert!(!questions.is_empty());
        assert!(questions.iter().all(|q| !q.is_empty()));
    }

    #[test]
    fn test_vorp_score_creation() {
        let score = VorpScore::new(0.8, 0.7, 0.9, 0.6);
        assert!((score.score - 0.75).abs() < 0.01);
        assert_eq!(score.value_score, 0.8);
        assert_eq!(score.obvious_score, 0.7);
        assert_eq!(score.real_score, 0.9);
        assert_eq!(score.possible_score, 0.6);
    }

    #[test]
    fn test_vorp_score_clamping() {
        let score = VorpScore::new(1.5, -0.5, 2.0, 0.5);
        assert_eq!(score.value_score, 1.0);
        assert_eq!(score.obvious_score, 0.0);
        assert_eq!(score.real_score, 1.0);
    }

    #[test]
    fn test_vorp_score_passes() {
        let passing = VorpScore::new(0.6, 0.6, 0.6, 0.6);
        assert!(passing.passes());

        let failing = VorpScore::new(0.4, 0.4, 0.4, 0.4);
        assert!(!failing.passes());
    }

    #[test]
    fn test_vorp_weakest_dimension() {
        let score = VorpScore::new(0.9, 0.3, 0.8, 0.7);
        let (name, value) = score.weakest_dimension();
        assert_eq!(name, "Obvious");
        assert!((value - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_vorp_recommendations() {
        let score = VorpScore::new(0.3, 0.8, 0.8, 0.8);
        let recs = score.recommendations();
        assert!(!recs.is_empty());
        assert!(recs.iter().any(|r| r.contains("value")));
    }

    #[test]
    fn test_vorp_calculator() {
        let score = VorpCalculator::new()
            .with_value(0.8)
            .with_obvious(0.7)
            .with_real(0.9)
            .with_possible(0.6)
            .with_replacement("Competitor X".to_string())
            .with_justification("Test justification".to_string())
            .calculate();

        assert!((score.score - 0.75).abs() < 0.01);
        assert_eq!(score.replacement_comparison, "Competitor X");
    }

    #[test]
    fn test_vorp_quick_assess() {
        let score = VorpCalculator::quick_assess(
            "Critical feature that is simple and validated",
            "Alternative",
        );

        assert!(score.value_score >= 0.8); // "critical" keyword
        assert!(score.obvious_score >= 0.5); // "simple" keyword
        assert!(score.real_score >= 0.5); // "validated" keyword
    }

    #[test]
    fn test_prioritized_item_creation() {
        let item = PrioritizedItem::new("item1".to_string(), "Test Item".to_string())
            .with_priority(80)
            .with_risk(30);

        assert_eq!(item.id, "item1");
        assert_eq!(item.title, "Test Item");
        assert_eq!(item.priority, 80);
        assert_eq!(item.risk, 30);
    }

    #[test]
    fn test_prioritized_item_adjusted_priority() {
        let item = PrioritizedItem::new("item1".to_string(), "Test".to_string())
            .with_vorp(VorpScore::new(0.8, 0.8, 0.8, 0.8)) // High VORP
            .with_priority(50)
            .with_risk(20);

        let adjusted = item.calculate_adjusted_priority();
        assert!(adjusted > 50, "High VORP should increase priority");
    }

    #[test]
    fn test_prioritized_item_biggest_concern() {
        let item = PrioritizedItem::new("item1".to_string(), "Test".to_string())
            .with_brutal_truth_score(BrutalTruth::ScaleIsHard, 0.3)
            .with_brutal_truth_score(BrutalTruth::UserValueBackLoaded, 0.8);

        let concern = item.biggest_concern();
        assert!(concern.is_some());
        let (truth, score) = concern.unwrap();
        assert_eq!(truth, BrutalTruth::ScaleIsHard);
        assert!((score - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_prioritizer_empty_input() {
        let result = BrutalTruthsPrioritizer::prioritize(vec![]);
        assert!(result.is_err());
        assert!(matches!(result, Err(PrioritizerError::EmptyInput)));
    }

    #[test]
    fn test_prioritizer_single_item() {
        let item = PrioritizedItem::new("i1".to_string(), "Test".to_string())
            .with_vorp(VorpScore::new(0.7, 0.7, 0.7, 0.7));

        let output = BrutalTruthsPrioritizer::prioritize(vec![item]).expect("Should succeed");

        assert_eq!(output.items.len(), 1);
        assert_eq!(output.stats.total_items, 1);
    }

    #[test]
    fn test_prioritizer_sorts_by_priority() {
        let items = vec![
            PrioritizedItem::new("low".to_string(), "Low".to_string())
                .with_vorp(VorpScore::new(0.4, 0.4, 0.4, 0.4))
                .with_priority(30),
            PrioritizedItem::new("high".to_string(), "High".to_string())
                .with_vorp(VorpScore::new(0.9, 0.9, 0.9, 0.9))
                .with_priority(90),
            PrioritizedItem::new("mid".to_string(), "Mid".to_string())
                .with_vorp(VorpScore::new(0.6, 0.6, 0.6, 0.6))
                .with_priority(60),
        ];

        let output = BrutalTruthsPrioritizer::prioritize(items).expect("Should succeed");

        assert_eq!(output.items[0].id, "high");
        assert_eq!(output.items[1].id, "mid");
        assert_eq!(output.items[2].id, "low");
    }

    #[test]
    fn test_quick_prioritize() {
        let items = vec![
            ("i1".to_string(), "Critical unique feature".to_string()),
            ("i2".to_string(), "Nice to have common feature".to_string()),
        ];

        let output = BrutalTruthsPrioritizer::quick_prioritize(items).expect("Should succeed");

        assert_eq!(output.items.len(), 2);
        // First item should have higher priority (critical, unique)
        assert!(output.items[0].vorp_score.value_score > output.items[1].vorp_score.value_score);
    }

    #[test]
    fn test_truth_averages_calculation() {
        let items = vec![
            PrioritizedItem::new("i1".to_string(), "Test".to_string())
                .with_brutal_truth_score(BrutalTruth::ScaleIsHard, 0.8),
            PrioritizedItem::new("i2".to_string(), "Test".to_string())
                .with_brutal_truth_score(BrutalTruth::ScaleIsHard, 0.6),
        ];

        let output = BrutalTruthsPrioritizer::prioritize(items).expect("Should succeed");

        let scale_avg = output
            .truth_averages
            .iter()
            .find(|(t, _)| *t == BrutalTruth::ScaleIsHard)
            .map(|(_, s)| *s);

        assert!(scale_avg.is_some());
        assert!((scale_avg.unwrap() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_stats_calculation() {
        let items = vec![
            PrioritizedItem::new("i1".to_string(), "Test".to_string())
                .with_priority(80)
                .with_risk(20)
                .with_vorp(VorpScore::new(0.8, 0.8, 0.8, 0.8)),
            PrioritizedItem::new("i2".to_string(), "Test".to_string())
                .with_priority(60)
                .with_risk(40)
                .with_vorp(VorpScore::new(0.6, 0.6, 0.6, 0.6)),
        ];

        let output = BrutalTruthsPrioritizer::prioritize(items).expect("Should succeed");

        assert_eq!(output.stats.total_items, 2);
        assert_eq!(output.stats.high_priority_count, 1);
        assert_eq!(output.stats.low_risk_count, 1);
    }

    #[test]
    fn test_item_recommendations_generated() {
        let item = PrioritizedItem::new("i1".to_string(), "Test".to_string())
            .with_vorp(VorpScore::new(0.3, 0.8, 0.8, 0.8)) // Low value
            .with_risk(80); // High risk

        let output = BrutalTruthsPrioritizer::prioritize(vec![item]).expect("Should succeed");

        let recs = &output.items[0].recommendations;
        assert!(!recs.is_empty(), "Should generate recommendations");
        assert!(
            recs.iter().any(|r| r.to_lowercase().contains("value") || r.to_lowercase().contains("risk")),
            "Should mention value or risk concerns"
        );
    }

    #[test]
    fn test_vorp_with_replacement() {
        let score = VorpScore::new(0.7, 0.7, 0.7, 0.7)
            .with_replacement("Competitor Product X".to_string());

        assert_eq!(score.replacement_comparison, "Competitor Product X");
    }

    #[test]
    fn test_prioritized_item_with_category() {
        let item = PrioritizedItem::new("i1".to_string(), "Test".to_string())
            .with_category("Security".to_string());

        assert_eq!(item.category, "Security");
    }

    #[test]
    fn test_prioritized_item_with_dependencies() {
        let item = PrioritizedItem::new("i1".to_string(), "Test".to_string())
            .with_dependency("i0".to_string())
            .with_dependency("i-1".to_string());

        assert_eq!(item.dependencies.len(), 2);
        assert!(item.dependencies.contains(&"i0".to_string()));
    }

    #[test]
    fn test_contains_any_helper() {
        assert!(contains_any("this is a test", &["test", "other"]));
        assert!(contains_any("hello world", &["world"]));
        assert!(!contains_any("hello world", &["foo", "bar"]));
    }
}
