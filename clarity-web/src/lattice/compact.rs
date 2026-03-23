#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Compact module for summarizing all phase artifacts.
//!
//! This module extracts key information from all phases (Discover, Define, Develop, Deliver)
//! and generates concise, agent-ready summaries. It prioritizes information by phase importance
//! and limits each section to 3-5 bullet points for maximum clarity.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for compact module
#[derive(Debug, Error)]
pub enum CompactError {
  #[error("no answers provided for compaction")]
  NoAnswers,

  #[error("failed to parse phase from step_id: {0}")]
  InvalidPhase(String),
}

/// Phase identifiers with priority ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Phase {
  /// Discover phase: problem exploration and user research
  Discover,
  /// Define phase: requirements and constraints
  Define,
  /// Develop phase: solution design and implementation
  Develop,
  /// Deliver phase: deployment and maintenance
  Deliver,
}

impl Phase {
  /// Get priority score (lower = higher priority)
  #[must_use]
  pub const fn priority(self) -> u8 {
    match self {
      Self::Discover => 1,
      Self::Define => 2,
      Self::Develop => 3,
      Self::Deliver => 4,
    }
  }

  /// Parse from string
  #[must_use]
  pub fn parse(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
      "discover" => Some(Self::Discover),
      "define" => Some(Self::Define),
      "develop" => Some(Self::Develop),
      "deliver" => Some(Self::Deliver),
      _ => None,
    }
  }

  /// Extract phase from `step_id`
  #[must_use]
  pub fn from_step_id(step_id: &str) -> Option<Self> {
    // Try to extract phase from step_id pattern like "discover-xxx" or "discover_xxx"
    step_id.split(['-', '_']).next().and_then(Self::parse)
  }
}

/// Compact summary of a single phase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactSummary {
  /// Problem statement (from Discover)
  #[serde(default)]
  pub problem: Vec<String>,
  /// Solution approach (from Discover/Define)
  #[serde(default)]
  pub solution: Vec<String>,
  /// Requirements list (from Define)
  #[serde(default)]
  pub requirements: Vec<String>,
  /// Constraints and limitations (from Define)
  #[serde(default)]
  pub constraints: Vec<String>,
  /// Implementation tasks (from Develop)
  #[serde(default)]
  pub tasks: Vec<String>,
}

impl CompactSummary {
  /// Create a new empty summary
  #[must_use]
  pub const fn new() -> Self {
    Self {
      problem: Vec::new(),
      solution: Vec::new(),
      requirements: Vec::new(),
      constraints: Vec::new(),
      tasks: Vec::new(),
    }
  }

  /// Add problem bullet point
  #[must_use]
  pub fn with_problem(mut self, point: String) -> Self {
    if !point.trim().is_empty() {
      self.problem.push(point);
    }
    self
  }

  /// Add solution bullet point
  #[must_use]
  pub fn with_solution(mut self, point: String) -> Self {
    if !point.trim().is_empty() {
      self.solution.push(point);
    }
    self
  }

  /// Add requirement bullet point
  #[must_use]
  pub fn with_requirement(mut self, point: String) -> Self {
    if !point.trim().is_empty() {
      self.requirements.push(point);
    }
    self
  }

  /// Add constraint bullet point
  #[must_use]
  pub fn with_constraint(mut self, point: String) -> Self {
    if !point.trim().is_empty() {
      self.constraints.push(point);
    }
    self
  }

  /// Add task bullet point
  #[must_use]
  pub fn with_task(mut self, point: String) -> Self {
    if !point.trim().is_empty() {
      self.tasks.push(point);
    }
    self
  }

  /// Limit each section to max bullet points
  #[must_use]
  pub fn limit_bullets(&self, max_per_section: usize) -> Self {
    Self {
      problem: self.problem.iter().take(max_per_section).cloned().collect(),
      solution: self
        .solution
        .iter()
        .take(max_per_section)
        .cloned()
        .collect(),
      requirements: self
        .requirements
        .iter()
        .take(max_per_section)
        .cloned()
        .collect(),
      constraints: self
        .constraints
        .iter()
        .take(max_per_section)
        .cloned()
        .collect(),
      tasks: self.tasks.iter().take(max_per_section).cloned().collect(),
    }
  }

  /// Count total bullets across all sections
  #[must_use]
  pub const fn total_bullets(&self) -> usize {
    self.problem.len()
      + self.solution.len()
      + self.requirements.len()
      + self.constraints.len()
      + self.tasks.len()
  }
}

impl Default for CompactSummary {
  fn default() -> Self {
    Self::new()
  }
}

/// Output of the compact operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactOutput {
  /// The compacted summary
  pub summary: CompactSummary,
  /// Total number of artifacts processed
  pub artifact_count: usize,
}

impl CompactOutput {
  /// Create a new compact output
  #[must_use]
  pub const fn new(summary: CompactSummary, artifact_count: usize) -> Self {
    Self {
      summary,
      artifact_count,
    }
  }

  /// Generate agent-ready formatted output
  #[must_use]
  pub fn to_agent_format(&self) -> String {
    let sections: [(&str, &[String]); 5] = [
      ("Problem", &self.summary.problem),
      ("Solution", &self.summary.solution),
      ("Requirements", &self.summary.requirements),
      ("Constraints", &self.summary.constraints),
      ("Tasks", &self.summary.tasks),
    ];

    let formatted_sections: String = sections
      .iter()
      .filter(|(_, items)| !items.is_empty())
      .map(|(title, items)| {
        let items_text = items
          .iter()
          .map(|item| format!("* {item}"))
          .collect::<Vec<_>>()
          .join("\n");
        format!("## {title}\n{items_text}\n")
      })
      .collect::<Vec<_>>()
      .join("\n");

    format!(
      "# Project Summary\n\n{formatted_sections}\n---\n**Artifacts processed**: {}",
      self.artifact_count
    )
  }
}

/// Input answer from a prompt step for compact operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactAnswer {
  /// Step identifier (e.g., "discover-problem", "define-requirements")
  pub step_id: String,
  /// The answer value
  pub value: String,
  /// Timestamp (ISO 8601)
  #[serde(default)]
  pub timestamp: String,
}

impl CompactAnswer {
  /// Extract phase from this answer
  #[must_use]
  pub fn phase(&self) -> Option<Phase> {
    Phase::from_step_id(&self.step_id)
  }

  /// Check if `step_id` contains a keyword
  #[must_use]
  fn step_contains(&self, keyword: &str) -> bool {
    self
      .step_id
      .to_lowercase()
      .contains(&keyword.to_lowercase())
  }
}

/// Compact all answers into a prioritized summary
///
/// # Arguments
/// * `all_answers` - All answers from all phases
///
/// # Returns
/// * `CompactOutput` containing the summarized view and artifact count
///
/// # Errors
/// * `CompactError::NoAnswers` if no answers provided
pub fn compact_artifacts(all_answers: Vec<CompactAnswer>) -> Result<CompactOutput, CompactError> {
  if all_answers.is_empty() {
    Err(CompactError::NoAnswers)
  } else {
    let artifact_count = all_answers.len();
    let summary = build_summary(all_answers);

    Ok(CompactOutput::new(summary.limit_bullets(5), artifact_count))
  }
}

/// Build summary by categorizing and prioritizing answers
fn build_summary(answers: Vec<CompactAnswer>) -> CompactSummary {
  // Sort by phase priority (Discover first, then Define, etc.)
  let prioritized = answers
    .into_iter()
    .filter_map(|answer| answer.phase().map(|phase| (phase, answer)))
    .sorted_by_key(|(phase, _)| phase.priority());

  // Categorize answers by step_id patterns
  prioritized.fold(CompactSummary::new(), |mut summary, (phase, answer)| {
    let value = clean_text(&answer.value);

    // Skip empty values
    if value.is_empty() {
      return summary;
    }

    match phase {
      Phase::Discover => {
        // Problem and solution from discovery
        if answer.step_contains("problem") {
          summary = summary.with_problem(value);
        } else if answer.step_contains("solution") || answer.step_contains("approach") {
          summary = summary.with_solution(value);
        } else if answer.step_contains("user") || answer.step_contains("need") {
          // User needs contribute to problem understanding
          summary = summary.with_problem(value);
        }
      }
      Phase::Define => {
        // Requirements and constraints from definition
        if answer.step_contains("require") {
          summary = summary.with_requirement(value);
        } else if answer.step_contains("constraint") || answer.step_contains("limit") {
          summary = summary.with_constraint(value);
        } else if answer.step_contains("criteria") {
          summary = summary.with_requirement(value);
        } else if answer.step_contains("scope") {
          summary = summary.with_constraint(value);
        }
      }
      Phase::Develop => {
        // Tasks and implementation details
        if answer.step_contains("task")
          || answer.step_contains("implement")
          || answer.step_contains("feature")
        {
          summary = summary.with_task(value);
        } else if answer.step_contains("design") {
          summary = summary.with_solution(value);
        }
      }
      Phase::Deliver => {
        // Deployment and maintenance tasks
        if answer.step_contains("deploy")
          || answer.step_contains("release")
          || answer.step_contains("maintain")
        {
          summary = summary.with_task(value);
        }
      }
    }

    summary
  })
}

/// Clean and normalize text for bullet points
#[must_use]
pub fn clean_text(text: &str) -> String {
  text
    .lines()
    .filter_map(|line| {
      let trimmed = line.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    })
    .take(1) // Only first line for bullet points
    .collect::<Vec<_>>()
    .join(" ")
    .split_whitespace() // Collapse multiple spaces
    .collect::<Vec<_>>()
    .join(" ")
    .chars()
    .take(100) // Limit to 100 chars per bullet
    .collect()
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;

  #[test]
  fn test_phase_priority_ordering() {
    assert!(Phase::Discover.priority() < Phase::Define.priority());
    assert!(Phase::Define.priority() < Phase::Develop.priority());
    assert!(Phase::Develop.priority() < Phase::Deliver.priority());
  }

  #[test]
  fn test_phase_from_str() {
    assert_eq!(Phase::parse("discover"), Some(Phase::Discover));
    assert_eq!(Phase::parse("DISCOVER"), Some(Phase::Discover));
    assert_eq!(Phase::parse("Define"), Some(Phase::Define));
    assert_eq!(Phase::parse("develop"), Some(Phase::Develop));
    assert_eq!(Phase::parse("Deliver"), Some(Phase::Deliver));
    assert_eq!(Phase::parse("invalid"), None);
  }

  #[test]
  fn test_phase_from_step_id() {
    assert_eq!(
      Phase::from_step_id("discover-problem"),
      Some(Phase::Discover)
    );
    assert_eq!(
      Phase::from_step_id("define-requirements"),
      Some(Phase::Define)
    );
    assert_eq!(Phase::from_step_id("develop-tasks"), Some(Phase::Develop));
    assert_eq!(Phase::from_step_id("deliver-deploy"), Some(Phase::Deliver));
    assert_eq!(Phase::from_step_id("invalid-step"), None);
  }

  #[test]
  fn test_phase_from_step_id_with_underscore() {
    assert_eq!(
      Phase::from_step_id("discover_problem"),
      Some(Phase::Discover)
    );
    assert_eq!(
      Phase::from_step_id("define_requirements"),
      Some(Phase::Define)
    );
  }

  #[test]
  fn test_compact_summary_new() {
    let summary = CompactSummary::new();
    assert!(summary.problem.is_empty());
    assert!(summary.solution.is_empty());
    assert!(summary.requirements.is_empty());
    assert!(summary.constraints.is_empty());
    assert!(summary.tasks.is_empty());
  }

  #[test]
  fn test_compact_summary_with_points() {
    let summary = CompactSummary::new()
      .with_problem("Problem 1".to_string())
      .with_problem("Problem 2".to_string())
      .with_solution("Solution 1".to_string())
      .with_requirement("Requirement 1".to_string())
      .with_constraint("Constraint 1".to_string())
      .with_task("Task 1".to_string());

    assert_eq!(summary.problem.len(), 2);
    assert_eq!(summary.solution.len(), 1);
    assert_eq!(summary.requirements.len(), 1);
    assert_eq!(summary.constraints.len(), 1);
    assert_eq!(summary.tasks.len(), 1);
  }

  #[test]
  fn test_compact_summary_ignores_empty_points() {
    let summary = CompactSummary::new()
      .with_problem(String::new())
      .with_problem("   ".to_string())
      .with_problem("Valid problem".to_string());

    assert_eq!(summary.problem.len(), 1);
    assert_eq!(summary.problem[0], "Valid problem");
  }

  #[test]
  fn test_compact_summary_limit_bullets() {
    let summary = CompactSummary::new()
      .with_problem("P1".to_string())
      .with_problem("P2".to_string())
      .with_problem("P3".to_string())
      .with_problem("P4".to_string())
      .with_problem("P5".to_string())
      .with_problem("P6".to_string());

    let limited = summary.limit_bullets(3);
    assert_eq!(limited.problem.len(), 3);
    assert_eq!(limited.problem, vec!["P1", "P2", "P3"]);
  }

  #[test]
  fn test_compact_summary_total_bullets() {
    let summary = CompactSummary::new()
      .with_problem("P1".to_string())
      .with_problem("P2".to_string())
      .with_solution("S1".to_string())
      .with_requirement("R1".to_string())
      .with_constraint("C1".to_string())
      .with_task("T1".to_string());

    assert_eq!(summary.total_bullets(), 6);
  }

  #[test]
  fn test_compact_artifacts_empty() {
    let result = compact_artifacts(vec![]);
    assert!(matches!(result, Err(CompactError::NoAnswers)));
  }

  #[test]
  fn test_compact_artifacts_categorizes_discover() {
    let answers = vec![
      CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: "The system is too slow".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "discover-solution".to_string(),
        value: "Use caching".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };
    assert_eq!(result.summary.problem.len(), 1);
    assert_eq!(result.summary.problem[0], "The system is too slow");
    assert_eq!(result.summary.solution.len(), 1);
    assert_eq!(result.summary.solution[0], "Use caching");
    assert_eq!(result.artifact_count, 2);
  }

  #[test]
  fn test_compact_artifacts_categorizes_define() {
    let answers = vec![
      CompactAnswer {
        step_id: "define-requirements".to_string(),
        value: "Must support 1000 users".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "define-constraints".to_string(),
        value: "Budget limited to $10k".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };
    assert_eq!(result.summary.requirements.len(), 1);
    assert_eq!(result.summary.requirements[0], "Must support 1000 users");
    assert_eq!(result.summary.constraints.len(), 1);
    assert_eq!(result.summary.constraints[0], "Budget limited to $10k");
  }

  #[test]
  fn test_compact_artifacts_categorizes_develop() {
    let answers = vec![
      CompactAnswer {
        step_id: "develop-tasks".to_string(),
        value: "Implement authentication".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "develop-design".to_string(),
        value: "Use JWT tokens".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };
    assert_eq!(result.summary.tasks.len(), 1);
    assert_eq!(result.summary.tasks[0], "Implement authentication");
    assert_eq!(result.summary.solution.len(), 1);
    assert_eq!(result.summary.solution[0], "Use JWT tokens");
  }

  #[test]
  fn test_compact_artifacts_prioritizes_by_phase() {
    let answers = vec![
      CompactAnswer {
        step_id: "develop-tasks".to_string(),
        value: "Develop task".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: "Discover problem".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "define-requirements".to_string(),
        value: "Define requirement".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };
    // Should process in priority order, but categorization is what matters
    assert_eq!(result.summary.problem.len(), 1);
    assert_eq!(result.summary.requirements.len(), 1);
    assert_eq!(result.summary.tasks.len(), 1);
  }

  #[test]
  fn test_compact_artifacts_limits_bullets() {
    let answers = (0..10)
      .map(|i| CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: format!("Problem {i}"),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      })
      .collect();

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };
    // Should limit to 5 bullets max
    assert_eq!(result.summary.problem.len(), 5);
    assert_eq!(result.artifact_count, 10);
  }

  #[test]
  fn test_compact_artifacts_ignores_empty_values() {
    let answers = vec![
      CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: String::new(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: "   ".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: "Valid problem".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };
    assert_eq!(result.summary.problem.len(), 1);
    assert_eq!(result.summary.problem[0], "Valid problem");
  }

  #[test]
  fn test_compact_artifacts_ignores_invalid_phase() {
    let answers = vec![
      CompactAnswer {
        step_id: "invalid-step".to_string(),
        value: "This should be ignored".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: "Valid problem".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };
    // Invalid phase answers are filtered out
    assert_eq!(result.summary.problem.len(), 1);
    assert_eq!(result.artifact_count, 2); // But still counted in artifact_count
  }

  #[test]
  fn test_clean_text_trims_whitespace() {
    let cleaned = clean_text("  too    many   spaces  ");
    assert_eq!(cleaned, "too many spaces");
  }

  #[test]
  fn test_clean_text_limits_length() {
    let long_text = "a".repeat(200);
    let cleaned = clean_text(&long_text);
    assert!(cleaned.len() <= 100);
  }

  #[test]
  fn test_clean_text_takes_first_line() {
    let cleaned = clean_text("First line\nSecond line\nThird line");
    assert_eq!(cleaned, "First line");
  }

  #[test]
  fn test_clean_text_empty_input() {
    assert_eq!(clean_text(""), "");
    assert_eq!(clean_text("   "), "");
  }

  #[test]
  fn test_compact_output_to_agent_format() {
    let summary = CompactSummary::new()
      .with_problem("Problem 1".to_string())
      .with_solution("Solution 1".to_string())
      .with_requirement("Requirement 1".to_string())
      .with_constraint("Constraint 1".to_string())
      .with_task("Task 1".to_string());

    let output = CompactOutput::new(summary, 5);
    let formatted = output.to_agent_format();

    assert!(formatted.contains("# Project Summary"));
    assert!(formatted.contains("## Problem"));
    assert!(formatted.contains("* Problem 1"));
    assert!(formatted.contains("## Solution"));
    assert!(formatted.contains("* Solution 1"));
    assert!(formatted.contains("## Requirements"));
    assert!(formatted.contains("* Requirement 1"));
    assert!(formatted.contains("## Constraints"));
    assert!(formatted.contains("* Constraint 1"));
    assert!(formatted.contains("## Tasks"));
    assert!(formatted.contains("* Task 1"));
    assert!(formatted.contains("**Artifacts processed**: 5"));
  }

  #[test]
  fn test_compact_output_to_agent_format_empty_sections() {
    let summary = CompactSummary::new().with_problem("Only problem".to_string());
    let output = CompactOutput::new(summary, 1);
    let formatted = output.to_agent_format();

    assert!(formatted.contains("## Problem"));
    assert!(formatted.contains("* Only problem"));
    // Empty sections should not appear
    assert!(!formatted.contains("## Solution"));
  }

  #[test]
  fn test_answer_phase_extraction() {
    let answer = CompactAnswer {
      step_id: "discover-problem".to_string(),
      value: "Test".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
    };

    assert_eq!(answer.phase(), Some(Phase::Discover));
  }

  #[test]
  fn test_answer_step_contains() {
    let answer = CompactAnswer {
      step_id: "discover-problem-statement".to_string(),
      value: "Test".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
    };

    assert!(answer.step_contains("problem"));
    assert!(answer.step_contains("PROBLEM"));
    assert!(!answer.step_contains("solution"));
  }

  #[test]
  fn test_compact_artifacts_mixed_phases() {
    let answers = vec![
      // Discover
      CompactAnswer {
        step_id: "discover-problem".to_string(),
        value: "System is slow".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "discover-user-needs".to_string(),
        value: "Users need faster response".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      // Define
      CompactAnswer {
        step_id: "define-requirements".to_string(),
        value: "Must respond in 100ms".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "define-constraints".to_string(),
        value: "Limited to 2GB memory".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      // Develop
      CompactAnswer {
        step_id: "develop-tasks".to_string(),
        value: "Implement caching".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };

    assert_eq!(result.summary.problem.len(), 2);
    assert_eq!(result.summary.requirements.len(), 1);
    assert_eq!(result.summary.constraints.len(), 1);
    assert_eq!(result.summary.tasks.len(), 1);
    assert_eq!(result.artifact_count, 5);
  }

  #[test]
  fn test_compact_artifacts_deliver_phase() {
    let answers = vec![
      CompactAnswer {
        step_id: "deliver-deploy".to_string(),
        value: "Deploy to production".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
      CompactAnswer {
        step_id: "deliver-maintain".to_string(),
        value: "Set up monitoring".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
      },
    ];

    let result = compact_artifacts(answers);
    assert!(result.is_ok());
    let Some(result) = result.ok() else {
      return;
    };

    assert_eq!(result.summary.tasks.len(), 2);
    assert!(result.summary.tasks[0].contains("production"));
    assert!(result.summary.tasks[1].contains("monitoring"));
  }

  #[test]
  fn test_serialization() {
    let summary = CompactSummary::new()
      .with_problem("Problem 1".to_string())
      .with_solution("Solution 1".to_string());

    let json = serde_json::to_string(&summary);
    assert!(json.is_ok());
    let Some(json) = json.ok() else {
      return;
    };
    let deserialized: Result<CompactSummary, _> = serde_json::from_str(&json);
    assert!(deserialized.is_ok());
    let Some(deserialized) = deserialized.ok() else {
      return;
    };

    assert_eq!(deserialized, summary);
  }

  #[test]
  fn test_compact_output_serialization() {
    let summary = CompactSummary::new().with_problem("Problem 1".to_string());
    let output = CompactOutput::new(summary, 1);

    let json = serde_json::to_string(&output);
    assert!(json.is_ok());
    let Some(json) = json.ok() else {
      return;
    };
    let deserialized: Result<CompactOutput, _> = serde_json::from_str(&json);
    assert!(deserialized.is_ok());
    let Some(deserialized) = deserialized.ok() else {
      return;
    };

    assert_eq!(deserialized, output);
  }

  #[test]
  fn test_phase_serialization() {
    let phases = vec![
      Phase::Discover,
      Phase::Define,
      Phase::Develop,
      Phase::Deliver,
    ];

    for phase in phases {
      let json = serde_json::to_string(&phase);
      assert!(json.is_ok());
      let Some(json) = json.ok() else {
        return;
      };
      let deserialized: Result<Phase, _> = serde_json::from_str(&json);
      assert!(deserialized.is_ok());
      let Some(deserialized) = deserialized.ok() else {
        return;
      };

      assert_eq!(deserialized, phase);
    }
  }
}
