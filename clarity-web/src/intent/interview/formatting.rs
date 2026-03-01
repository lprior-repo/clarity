//! Interview formatting utilities for terminal display.
//!
//! Provides human-readable progress reports for interview sessions
//! and question formatting for interview display.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::fmt::Write;

use super::types::{Question, QuestionCategory, QuestionPriority};

/// Format interview progress as a human-readable summary.
///
/// Creates a progress report showing:
/// - Completion status (percentage complete)
/// - Current phase number
/// - Number of questions answered vs total
///
/// # Arguments
/// * `total_questions` - Total number of questions in the interview
/// * `answered` - Number of questions answered so far
/// * `current_phase` - The current interview phase (1-indexed)
///
/// # Returns
/// A formatted string suitable for terminal output.
///
/// # Example
/// ```
/// use clarity_web::intent::interview::format_progress;
/// let progress = format_progress(10, 3, 1);
/// assert!(progress.contains("Phase: 1"));
/// assert!(progress.contains("30%"));
/// assert!(progress.contains("3/10"));
/// ```
#[must_use]
pub fn format_progress(total_questions: usize, answered: usize, current_phase: u32) -> String {
  let mut output = String::new();

  let _ = writeln!(output, "Interview Progress");
  let _ = writeln!(output, "------------------");

  // Phase info
  let _ = writeln!(output, "Phase: {current_phase}");

  // Completion percentage
  let percentage = calculate_percentage(answered, total_questions);
  let _ = writeln!(output, "Completion: {percentage}%");

  // Questions answered
  let _ = writeln!(output, "Questions: {answered}/{total_questions} answered");

  // Status indicator
  let status = determine_status(answered, total_questions);
  let _ = writeln!(output, "Status: {status}");

  // Visual progress bar
  let progress_bar = create_progress_bar(answered, total_questions);
  let _ = writeln!(output, "[{progress_bar}]");

  output
}

/// Calculate completion percentage, handling division by zero.
fn calculate_percentage(answered: usize, total: usize) -> u32 {
  if total == 0 {
    return if answered == 0 { 0 } else { 100 };
  }

  // Avoid f64 conversion when possible for small values
  if answered >= total {
    return 100;
  }

  // Use integer arithmetic to avoid float conversion issues
  let percentage = (answered * 100) / total;

  // Ensure we don't exceed 100 (can happen due to integer division)
  if percentage > 100 {
    100
  } else {
    percentage as u32
  }
}

/// Determine the current status based on progress.
const fn determine_status(answered: usize, total: usize) -> &'static str {
  match (answered, total) {
    (0, _) => "Not started",
    (a, t) if a >= t && t > 0 => "Complete",
    (a, _) if a > 0 => "In progress",
    _ => "Ready",
  }
}

/// Create a visual progress bar string.
fn create_progress_bar(answered: usize, total: usize) -> String {
  const BAR_WIDTH: usize = 20;

  if total == 0 {
    return "                    ".to_string();
  }

  let filled = if answered >= total {
    BAR_WIDTH
  } else {
    // Use integer arithmetic to avoid float conversion issues
    let filled = (answered * BAR_WIDTH) / total;
    filled.min(BAR_WIDTH)
  };

  let empty = BAR_WIDTH.saturating_sub(filled);

  std::iter::repeat('=')
    .take(filled)
    .chain(std::iter::repeat(' ').take(empty))
    .collect()
}

// =============================================================================
// Question Formatting
// =============================================================================

/// Format a question for terminal display.
///
/// Takes a question ID, question text, and phase number and returns
/// a formatted string suitable for terminal output.
///
/// # Arguments
/// * `question_id` - Unique identifier for the question
/// * `question_text` - The main question text to display
/// * `phase` - The interview phase/round number
///
/// # Returns
/// A formatted string with the question metadata and text.
///
/// # Example
/// ```
/// use clarity_web::intent::interview::format_question;
/// let output = format_question("q1", "What is the purpose?", 1);
/// assert!(output.contains("Phase 1"));
/// assert!(output.contains("q1"));
/// assert!(output.contains("What is the purpose?"));
/// ```
#[must_use]
pub fn format_question(question_id: &str, question_text: &str, phase: u32) -> String {
  let header = format_question_header(question_id, phase);
  let body = format_question_body(question_text);

  format!("{header}\n{body}")
}

/// Format a full `Question` struct for terminal display.
///
/// This is the comprehensive formatter that includes all question metadata,
/// context, and examples.
///
/// # Arguments
/// * `question` - The question to format
///
/// # Returns
/// A multi-line formatted string with all question details.
#[must_use]
pub fn format_question_full(question: &Question) -> String {
  let header = format_question_header(&question.id, question.round);
  let body = format_question_body(&question.question);
  let meta = format_question_metadata(question);
  let context = format_question_context(&question.context);
  let example = format_question_example(&question.example);

  let parts: Vec<&str> = [&header, &body, &meta, &context, &example]
    .into_iter()
    .filter(|s| !s.is_empty())
    .map(|s| s.as_str())
    .collect();

  parts.join("\n\n")
}

/// Format a question briefly (single line with ID prefix).
///
/// # Arguments
/// * `question_id` - Unique identifier for the question
/// * `question_text` - The main question text to display
///
/// # Returns
/// A single-line formatted string.
#[must_use]
pub fn format_question_brief(question_id: &str, question_text: &str) -> String {
  let truncated = truncate_text(question_text, 60);
  format!("[{question_id}] {truncated}")
}

/// Format a list of questions as a summary.
///
/// # Arguments
/// * `questions` - Slice of questions to format
///
/// # Returns
/// A multi-line string with each question on its own line.
#[must_use]
pub fn format_question_list(questions: &[Question]) -> String {
  if questions.is_empty() {
    return "No questions available.".to_string();
  }

  questions
    .iter()
    .map(|q| format_question_brief(&q.id, &q.question))
    .collect::<Vec<_>>()
    .join("\n")
}

/// Format the question header with ID and phase.
fn format_question_header(question_id: &str, phase: u32) -> String {
  format!("=== Phase {phase}: {question_id} ===")
}

/// Format the question body text.
fn format_question_body(question_text: &str) -> String {
  question_text.trim().to_string()
}

/// Format question metadata (priority, category, dependencies).
fn format_question_metadata(question: &Question) -> String {
  let priority = format_question_priority(question.priority);
  let category = format_question_category(question.category);

  let mut parts = vec![priority, category];

  if !question.depends_on.is_empty() {
    parts.push(format!("Depends on: {}", question.depends_on.join(", ")));
  }

  if !question.blocks.is_empty() {
    parts.push(format!("Blocks: {}", question.blocks.join(", ")));
  }

  parts.join(" | ")
}

/// Format the context section.
fn format_question_context(context: &str) -> String {
  if context.trim().is_empty() {
    return String::new();
  }
  format!("Context: {}", context.trim())
}

/// Format the example section.
fn format_question_example(example: &str) -> String {
  if example.trim().is_empty() {
    return String::new();
  }
  format!("Example: {}", example.trim())
}

/// Format priority with visual indicator.
fn format_question_priority(priority: QuestionPriority) -> String {
  match priority {
    QuestionPriority::Critical => "[!!!] Critical".to_string(),
    QuestionPriority::Important => "[!!] Important".to_string(),
    QuestionPriority::NiceToHave => "[!] Nice to have".to_string(),
  }
}

/// Format category with label.
fn format_question_category(category: QuestionCategory) -> String {
  let label = match category {
    QuestionCategory::HappyPath => "Happy Path",
    QuestionCategory::ErrorCase => "Error Case",
    QuestionCategory::EdgeCase => "Edge Case",
    QuestionCategory::Constraint => "Constraint",
    QuestionCategory::Dependency => "Dependency",
    QuestionCategory::NonFunctional => "Non-Functional",
  };
  label.to_string()
}

/// Truncate text with ellipsis if it exceeds max length.
fn truncate_text(text: &str, max_len: usize) -> String {
  let trimmed = text.trim();
  if trimmed.len() <= max_len {
    return trimmed.to_string();
  }

  // Find a good break point (space) near the max length
  let break_point = trimmed
    .char_indices()
    .take_while(|(idx, _)| *idx < max_len)
    .last()
    .map_or(max_len, |(idx, _)| idx);

  // Try to break at a word boundary
  let break_at = trimmed[..break_point]
    .rfind(' ')
    .map_or(break_point, |space_idx| space_idx);

  let truncated = trimmed.chars().take(break_at).collect::<String>();
  format!("{truncated}...")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format_progress_not_started() {
    let result = format_progress(10, 0, 1);
    assert!(result.contains("Phase: 1"));
    assert!(result.contains("Completion: 0%"));
    assert!(result.contains("Questions: 0/10 answered"));
    assert!(result.contains("Status: Not started"));
  }

  #[test]
  fn test_format_progress_in_progress() {
    let result = format_progress(10, 5, 2);
    assert!(result.contains("Phase: 2"));
    assert!(result.contains("Completion: 50%"));
    assert!(result.contains("Questions: 5/10 answered"));
    assert!(result.contains("Status: In progress"));
  }

  #[test]
  fn test_format_progress_complete() {
    let result = format_progress(10, 10, 3);
    assert!(result.contains("Phase: 3"));
    assert!(result.contains("Completion: 100%"));
    assert!(result.contains("Questions: 10/10 answered"));
    assert!(result.contains("Status: Complete"));
  }

  #[test]
  fn test_format_progress_zero_total() {
    let result = format_progress(0, 0, 1);
    assert!(result.contains("Questions: 0/0 answered"));
    assert!(result.contains("Status: Not started"));
  }

  #[test]
  fn test_format_progress_answered_exceeds_total() {
    let result = format_progress(5, 10, 1);
    assert!(result.contains("Completion: 100%"));
    assert!(result.contains("Status: Complete"));
  }

  #[test]
  fn test_calculate_percentage_zero_total() {
    assert_eq!(calculate_percentage(0, 0), 0);
    assert_eq!(calculate_percentage(5, 0), 100);
  }

  #[test]
  fn test_calculate_percentage_normal() {
    assert_eq!(calculate_percentage(0, 10), 0);
    assert_eq!(calculate_percentage(5, 10), 50);
    assert_eq!(calculate_percentage(10, 10), 100);
    assert_eq!(calculate_percentage(1, 3), 33);
    assert_eq!(calculate_percentage(2, 3), 66);
  }

  #[test]
  fn test_determine_status() {
    assert_eq!(determine_status(0, 10), "Not started");
    assert_eq!(determine_status(0, 0), "Not started");
    assert_eq!(determine_status(1, 10), "In progress");
    assert_eq!(determine_status(9, 10), "In progress");
    assert_eq!(determine_status(10, 10), "Complete");
    assert_eq!(determine_status(15, 10), "Complete");
  }

  #[test]
  fn test_create_progress_bar_empty() {
    let bar = create_progress_bar(0, 10);
    assert_eq!(bar.chars().filter(|&c| c == '=').count(), 0);
    assert_eq!(bar.len(), 20);
  }

  #[test]
  fn test_create_progress_bar_half() {
    let bar = create_progress_bar(5, 10);
    assert_eq!(bar.chars().filter(|&c| c == '=').count(), 10);
    assert_eq!(bar.len(), 20);
  }

  #[test]
  fn test_create_progress_bar_full() {
    let bar = create_progress_bar(10, 10);
    assert_eq!(bar.chars().filter(|&c| c == '=').count(), 20);
    assert_eq!(bar.len(), 20);
  }

  #[test]
  fn test_create_progress_bar_zero_total() {
    let bar = create_progress_bar(0, 0);
    assert_eq!(bar.chars().filter(|&c| c == '=').count(), 0);
    assert_eq!(bar.len(), 20);
  }

  #[test]
  fn test_format_progress_output_structure() {
    let result = format_progress(10, 3, 1);

    // Verify header
    assert!(result.starts_with("Interview Progress\n"));
    assert!(result.contains("------------------\n"));

    // Verify all sections present
    assert!(result.contains("Phase:"));
    assert!(result.contains("Completion:"));
    assert!(result.contains("Questions:"));
    assert!(result.contains("Status:"));
    assert!(result.contains("[") && result.contains("]"));
  }

  // ========================================
  // Question Formatting Tests
  // ========================================

  #[test]
  fn test_format_question_basic() {
    let result = format_question("q1", "What is the purpose?", 1);
    assert!(result.contains("Phase 1"));
    assert!(result.contains("q1"));
    assert!(result.contains("What is the purpose?"));
  }

  #[test]
  fn test_format_question_with_whitespace() {
    let result = format_question("q2", "  Trimmed text  ", 2);
    assert!(result.contains("Trimmed text"));
    assert!(!result.contains("  Trimmed"));
  }

  #[test]
  fn test_format_question_brief() {
    let result = format_question_brief("q1", "What is the purpose?");
    assert!(result.starts_with("[q1]"));
    assert!(result.contains("What is the purpose?"));
  }

  #[test]
  fn test_format_question_brief_truncation() {
    let long_text =
      "This is a very long question that should be truncated because it exceeds the maximum length";
    let result = format_question_brief("q1", long_text);
    assert!(result.len() < long_text.len() + 10);
    assert!(result.ends_with("..."));
  }

  #[test]
  fn test_format_question_brief_exact_length() {
    // 60 characters exactly: "aaaaaaaaaa..." x 6
    let text = "123456789012345678901234567890123456789012345678901234567890";
    assert_eq!(text.len(), 60);
    let result = format_question_brief("q1", text);
    // If text is exactly 60 chars, truncate_text returns it as-is
    // The result will contain the full text without truncation ellipsis
    assert!(result.ends_with("1234567890"));
  }

  #[test]
  fn test_format_question_full() {
    let question = Question {
      id: "test-q".to_string(),
      round: 1,
      question: "What is the purpose?".to_string(),
      context: "Understanding the goal".to_string(),
      example: "A REST API for users".to_string(),
      priority: QuestionPriority::Critical,
      category: QuestionCategory::HappyPath,
      ..Question::default()
    };

    let result = format_question_full(&question);
    assert!(result.contains("Phase 1"));
    assert!(result.contains("test-q"));
    assert!(result.contains("What is the purpose?"));
    assert!(result.contains("Context:"));
    assert!(result.contains("Example:"));
    assert!(result.contains("Critical"));
    assert!(result.contains("Happy Path"));
  }

  #[test]
  fn test_format_question_full_minimal() {
    let question = Question {
      id: "minimal".to_string(),
      round: 1,
      question: "Simple question?".to_string(),
      context: String::new(),
      example: String::new(),
      ..Question::default()
    };

    let result = format_question_full(&question);
    assert!(result.contains("minimal"));
    assert!(result.contains("Simple question?"));
    assert!(!result.contains("Context:"));
    assert!(!result.contains("Example:"));
  }

  #[test]
  fn test_format_question_full_with_dependencies() {
    let question = Question {
      id: "dep-q".to_string(),
      round: 2,
      question: "Follow-up question?".to_string(),
      depends_on: vec!["q1".to_string(), "q2".to_string()],
      blocks: vec!["q3".to_string()],
      ..Question::default()
    };

    let result = format_question_full(&question);
    assert!(result.contains("Depends on: q1, q2"));
    assert!(result.contains("Blocks: q3"));
  }

  #[test]
  fn test_format_question_priority_all_variants() {
    assert!(format_question_priority(QuestionPriority::Critical).contains("Critical"));
    assert!(format_question_priority(QuestionPriority::Important).contains("Important"));
    assert!(format_question_priority(QuestionPriority::NiceToHave).contains("Nice to have"));
  }

  #[test]
  fn test_format_question_category_all_variants() {
    assert!(format_question_category(QuestionCategory::HappyPath).contains("Happy Path"));
    assert!(format_question_category(QuestionCategory::ErrorCase).contains("Error Case"));
    assert!(format_question_category(QuestionCategory::EdgeCase).contains("Edge Case"));
    assert!(format_question_category(QuestionCategory::Constraint).contains("Constraint"));
    assert!(format_question_category(QuestionCategory::Dependency).contains("Dependency"));
    assert!(format_question_category(QuestionCategory::NonFunctional).contains("Non-Functional"));
  }

  #[test]
  fn test_format_question_list_empty() {
    let result = format_question_list(&[]);
    assert!(result.contains("No questions"));
  }

  #[test]
  fn test_format_question_list_multiple() {
    let questions = vec![
      Question {
        id: "q1".to_string(),
        question: "First question?".to_string(),
        ..Question::default()
      },
      Question {
        id: "q2".to_string(),
        question: "Second question?".to_string(),
        ..Question::default()
      },
    ];

    let result = format_question_list(&questions);
    assert!(result.contains("[q1]"));
    assert!(result.contains("[q2]"));
    assert!(result.contains("First question"));
    assert!(result.contains("Second question"));
  }

  #[test]
  fn test_truncate_text_short() {
    let result = truncate_text("Short", 60);
    assert_eq!(result, "Short");
  }

  #[test]
  fn test_truncate_text_exact() {
    // 60 characters exactly
    let text = "123456789012345678901234567890123456789012345678901234567890";
    assert_eq!(text.len(), 60);
    let result = truncate_text(text, 60);
    assert_eq!(result, text);
  }

  #[test]
  fn test_truncate_text_long() {
    let long_text =
      "This is a very long question that should be truncated because it exceeds the maximum length";
    let result = truncate_text(long_text, 60);
    assert!(result.len() < long_text.len());
    assert!(result.ends_with("..."));
  }

  #[test]
  fn test_truncate_text_preserves_word_boundary() {
    let text = "This is a question that has many words in it";
    let result = truncate_text(text, 20);
    // Should break at a word boundary, not mid-word
    assert!(!result.contains("question th"));
    assert!(result.ends_with("..."));
  }
}
