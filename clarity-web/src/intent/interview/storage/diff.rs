// Additional clippy lints to allow
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_strip)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]

use super::models::{
  AnswerChangeType, AnswerDiff, AnswerField, AnswerFieldDiff, AnswerFieldsDiff, SessionDiff,
  SessionSnapshot,
};
use crate::intent::interview::types::{Answer, InterviewSession, InterviewStage, Perspective};
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedStage {
  Known(InterviewStage),
  Unknown(String),
}

impl ParsedStage {
  fn parse(raw: &str) -> Self {
    match raw.to_ascii_lowercase().as_str() {
      "discovery" => Self::Known(InterviewStage::Discovery),
      "refinement" => Self::Known(InterviewStage::Refinement),
      "validation" => Self::Known(InterviewStage::Validation),
      "complete" => Self::Known(InterviewStage::Complete),
      "paused" => Self::Known(InterviewStage::Paused),
      _ => Self::Unknown(raw.to_string()),
    }
  }

  fn into_output_string(self) -> String {
    match self {
      Self::Known(stage) => stage.as_str().to_string(),
      Self::Unknown(raw) => raw,
    }
  }
}

fn count_delta(to_count: usize, from_count: usize) -> i32 {
  i32::try_from(to_count).map_or(i32::MAX, |to_i32| {
    i32::try_from(from_count).map_or(i32::MIN, |from_i32| to_i32 - from_i32)
  })
}

/// Compute the difference between two interview sessions.
#[must_use]
pub fn diff_sessions(from: &InterviewSession, to: &InterviewSession) -> SessionDiff {
  let from_answers: HashMap<&str, &str> = from
    .answers
    .iter()
    .map(|answer| (answer.question_id.as_str(), answer.response.as_str()))
    .collect();

  let to_answers: HashMap<&str, &str> = to
    .answers
    .iter()
    .map(|answer| (answer.question_id.as_str(), answer.response.as_str()))
    .collect();

  let answers_added = to
    .answers
    .iter()
    .filter(|answer| !from_answers.contains_key(answer.question_id.as_str()))
    .map(|answer| AnswerDiff {
      question_id: answer.question_id.clone(),
      question_text: answer.question_text.clone(),
      old_response: None,
      new_response: Some(answer.response.clone()),
      change_type: AnswerChangeType::Added,
    })
    .collect::<Vec<_>>();

  let answers_modified = to
    .answers
    .iter()
    .filter(|answer| {
      from_answers
        .get(answer.question_id.as_str())
        .is_some_and(|old| *old != answer.response)
    })
    .map(|answer| AnswerDiff {
      question_id: answer.question_id.clone(),
      question_text: answer.question_text.clone(),
      old_response: from_answers
        .get(answer.question_id.as_str())
        .map(|value| (*value).to_string()),
      new_response: Some(answer.response.clone()),
      change_type: AnswerChangeType::Modified,
    })
    .collect::<Vec<_>>();

  let answers_removed = from
    .answers
    .iter()
    .filter(|answer| !to_answers.contains_key(answer.question_id.as_str()))
    .map(|answer| AnswerDiff {
      question_id: answer.question_id.clone(),
      question_text: answer.question_text.clone(),
      old_response: Some(answer.response.clone()),
      new_response: None,
      change_type: AnswerChangeType::Removed,
    })
    .collect::<Vec<_>>();

  SessionDiff {
    from_session_id: from.id.clone(),
    to_session_id: to.id.clone(),
    from_timestamp: from.updated_at.clone(),
    to_timestamp: to.updated_at.clone(),
    stage_changed: from.stage != to.stage,
    old_stage: Some(from.stage.as_str().to_string()),
    new_stage: Some(to.stage.as_str().to_string()),
    answers_added,
    answers_modified,
    answers_removed,
    gaps_added: count_delta(to.gaps.len(), from.gaps.len()),
    conflicts_added: count_delta(to.conflicts.len(), from.conflicts.len()),
  }
}

/// Format a session diff as human-readable text.
#[must_use]
pub fn format_diff(diff: &SessionDiff) -> String {
  let mut output = String::new();
  let _ = writeln!(
    output,
    "Session Diff: {} -> {}",
    diff.from_session_id, diff.to_session_id
  );
  let _ = writeln!(
    output,
    "Timestamps: {} -> {}\n",
    diff.from_timestamp, diff.to_timestamp
  );

  if diff.stage_changed {
    let _ = writeln!(
      output,
      "Stage: {} -> {}\n",
      diff.old_stage.as_deref().unwrap_or("(none)"),
      diff.new_stage.as_deref().unwrap_or("(none)")
    );
  }

  write_added_answers(&mut output, &diff.answers_added);
  write_modified_answers(&mut output, &diff.answers_modified);
  write_removed_answers(&mut output, &diff.answers_removed);
  write_delta_line(
    &mut output,
    "Gaps",
    diff.gaps_added,
    "new gap(s)",
    "gap(s) resolved",
  );
  write_delta_line(
    &mut output,
    "Conflicts",
    diff.conflicts_added,
    "new conflict(s)",
    "conflict(s) resolved",
  );

  output
}

fn truncate(value: &str) -> String {
  const MAX_RESPONSE_LEN: usize = 50;
  if value.chars().count() > MAX_RESPONSE_LEN {
    format!(
      "{}...",
      value.chars().take(MAX_RESPONSE_LEN).collect::<String>()
    )
  } else {
    value.to_string()
  }
}

fn format_response(response: Option<&String>) -> String {
  response.map_or_else(|| "(none)".to_string(), |value| truncate(value))
}

fn write_added_answers(output: &mut String, answers: &[AnswerDiff]) {
  if answers.is_empty() {
    return;
  }

  let _ = writeln!(output, "Answers Added ({}):", answers.len());
  for answer in answers {
    let _ = writeln!(
      output,
      "  + [{}] {}: {}",
      answer.question_id,
      truncate(&answer.question_text),
      format_response(answer.new_response.as_ref())
    );
  }
  output.push('\n');
}

fn write_modified_answers(output: &mut String, answers: &[AnswerDiff]) {
  if answers.is_empty() {
    return;
  }

  let _ = writeln!(output, "Answers Modified ({}):", answers.len());
  for answer in answers {
    let _ = writeln!(
      output,
      "  ~ [{}] {}:\n    {} -> {}",
      answer.question_id,
      truncate(&answer.question_text),
      format_response(answer.old_response.as_ref()),
      format_response(answer.new_response.as_ref())
    );
  }
  output.push('\n');
}

fn write_removed_answers(output: &mut String, answers: &[AnswerDiff]) {
  if answers.is_empty() {
    return;
  }

  let _ = writeln!(output, "Answers Removed ({}):", answers.len());
  for answer in answers {
    let _ = writeln!(
      output,
      "  - [{}] {}: {}",
      answer.question_id,
      truncate(&answer.question_text),
      format_response(answer.old_response.as_ref())
    );
  }
  output.push('\n');
}

fn write_delta_line(
  output: &mut String,
  label: &str,
  delta: i32,
  positive_suffix: &str,
  negative_suffix: &str,
) {
  let _ = match delta.cmp(&0) {
    std::cmp::Ordering::Greater => writeln!(output, "{label}: +{delta} {positive_suffix}"),
    std::cmp::Ordering::Less => writeln!(output, "{label}: {} {negative_suffix}", -delta),
    std::cmp::Ordering::Equal => writeln!(output, "{label}: No change"),
  };
}

/// Compute the difference between two session snapshots.
#[must_use]
pub fn diff_snapshots(from: &SessionSnapshot, to: &SessionSnapshot) -> SessionDiff {
  let answers_added = to
    .answers
    .iter()
    .filter(|(id, _)| !from.answers.contains_key(*id))
    .map(|(id, response)| AnswerDiff {
      question_id: id.clone(),
      question_text: id.clone(),
      old_response: None,
      new_response: Some(response.clone()),
      change_type: AnswerChangeType::Added,
    })
    .collect::<Vec<_>>();

  let answers_modified = to
    .answers
    .iter()
    .filter(|(id, response)| from.answers.get(*id).is_some_and(|old| old != *response))
    .map(|(id, response)| AnswerDiff {
      question_id: id.clone(),
      question_text: id.clone(),
      old_response: from.answers.get(id).cloned(),
      new_response: Some(response.clone()),
      change_type: AnswerChangeType::Modified,
    })
    .collect::<Vec<_>>();

  let answers_removed = from
    .answers
    .iter()
    .filter(|(id, _)| !to.answers.contains_key(*id))
    .map(|(id, response)| AnswerDiff {
      question_id: id.clone(),
      question_text: id.clone(),
      old_response: Some(response.clone()),
      new_response: None,
      change_type: AnswerChangeType::Removed,
    })
    .collect::<Vec<_>>();

  let parsed_from_stage = ParsedStage::parse(&from.stage);
  let parsed_to_stage = ParsedStage::parse(&to.stage);
  let stage_changed = parsed_from_stage != parsed_to_stage;

  SessionDiff {
    from_session_id: from.session_id.clone(),
    to_session_id: to.session_id.clone(),
    from_timestamp: from.timestamp.clone(),
    to_timestamp: to.timestamp.clone(),
    stage_changed,
    old_stage: Some(parsed_from_stage.into_output_string()),
    new_stage: Some(parsed_to_stage.into_output_string()),
    answers_added,
    answers_modified,
    answers_removed,
    gaps_added: count_delta(to.gaps_count, from.gaps_count),
    conflicts_added: count_delta(to.conflicts_count, from.conflicts_count),
  }
}

// ============================================================================
// Field-Level Answer Diff Functions
// ============================================================================

/// Compute field-level diff between two Answer instances.
///
/// This compares all fields of two answers and returns a structured diff
/// showing exactly which fields changed and their old/new values.
#[must_use]
pub fn diff_answer_fields(old: &Answer, new: &Answer) -> AnswerFieldsDiff {
  let field_changes = compute_field_changes(old, new);

  let change_type = if old.question_id.is_empty() {
    AnswerChangeType::Added
  } else if new.question_id.is_empty() {
    AnswerChangeType::Removed
  } else if !field_changes.is_empty() {
    AnswerChangeType::Modified
  } else {
    // No changes - still consider it modified if called with same answer
    AnswerChangeType::Modified
  };

  AnswerFieldsDiff {
    question_id: new.question_id.clone(),
    question_text: new.question_text.clone(),
    change_type,
    field_changes,
  }
}

/// Compute field-level diff for an added answer (no prior version).
#[must_use]
pub fn diff_answer_added(answer: &Answer) -> AnswerFieldsDiff {
  AnswerFieldsDiff {
    question_id: answer.question_id.clone(),
    question_text: answer.question_text.clone(),
    change_type: AnswerChangeType::Added,
    field_changes: vec![
      AnswerFieldDiff {
        field: AnswerField::QuestionId,
        old_value: None,
        new_value: Some(answer.question_id.clone()),
      },
      AnswerFieldDiff {
        field: AnswerField::QuestionText,
        old_value: None,
        new_value: Some(answer.question_text.clone()),
      },
      AnswerFieldDiff {
        field: AnswerField::Perspective,
        old_value: None,
        new_value: Some(perspective_to_string(&answer.perspective)),
      },
      AnswerFieldDiff {
        field: AnswerField::Round,
        old_value: None,
        new_value: Some(answer.round.to_string()),
      },
      AnswerFieldDiff {
        field: AnswerField::Response,
        old_value: None,
        new_value: Some(answer.response.clone()),
      },
      AnswerFieldDiff {
        field: AnswerField::Extracted,
        old_value: None,
        new_value: Some(hashmap_to_string(&answer.extracted)),
      },
      AnswerFieldDiff {
        field: AnswerField::Confidence,
        old_value: None,
        new_value: Some(format_confidence(answer.confidence)),
      },
      AnswerFieldDiff {
        field: AnswerField::Notes,
        old_value: None,
        new_value: Some(answer.notes.clone()),
      },
      AnswerFieldDiff {
        field: AnswerField::Timestamp,
        old_value: None,
        new_value: Some(answer.timestamp.clone()),
      },
    ],
  }
}

/// Compute field-level diff for a removed answer (no new version).
#[must_use]
pub fn diff_answer_removed(answer: &Answer) -> AnswerFieldsDiff {
  AnswerFieldsDiff {
    question_id: answer.question_id.clone(),
    question_text: answer.question_text.clone(),
    change_type: AnswerChangeType::Removed,
    field_changes: vec![
      AnswerFieldDiff {
        field: AnswerField::QuestionId,
        old_value: Some(answer.question_id.clone()),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::QuestionText,
        old_value: Some(answer.question_text.clone()),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::Perspective,
        old_value: Some(perspective_to_string(&answer.perspective)),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::Round,
        old_value: Some(answer.round.to_string()),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::Response,
        old_value: Some(answer.response.clone()),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::Extracted,
        old_value: Some(hashmap_to_string(&answer.extracted)),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::Confidence,
        old_value: Some(format_confidence(answer.confidence)),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::Notes,
        old_value: Some(answer.notes.clone()),
        new_value: None,
      },
      AnswerFieldDiff {
        field: AnswerField::Timestamp,
        old_value: Some(answer.timestamp.clone()),
        new_value: None,
      },
    ],
  }
}

/// Compute field-level diffs for all answers between two sessions.
#[must_use]
pub fn diff_sessions_field_level(
  from: &InterviewSession,
  to: &InterviewSession,
) -> Vec<AnswerFieldsDiff> {
  let from_answers: HashMap<&str, &Answer> = from
    .answers
    .iter()
    .map(|answer| (answer.question_id.as_str(), answer))
    .collect();

  let to_answers: HashMap<&str, &Answer> = to
    .answers
    .iter()
    .map(|answer| (answer.question_id.as_str(), answer))
    .collect();

  let added = to
    .answers
    .iter()
    .filter(|answer| !from_answers.contains_key(answer.question_id.as_str()))
    .map(diff_answer_added);

  let modified = to.answers.iter().filter_map(|answer| {
    from_answers
      .get(answer.question_id.as_str())
      .filter(|old| *old != &answer)
      .map(|old| diff_answer_fields(old, answer))
  });

  let removed = from
    .answers
    .iter()
    .filter(|answer| !to_answers.contains_key(answer.question_id.as_str()))
    .map(diff_answer_removed);

  added.chain(modified).chain(removed).collect()
}

// ============================================================================
// Internal Helper Functions
// ============================================================================

fn compute_field_changes(old: &Answer, new: &Answer) -> Vec<AnswerFieldDiff> {
  let changes = Vec::new();

  // Check each field and collect changes
  let changes = add_string_field_change(
    changes,
    AnswerField::QuestionId,
    &old.question_id,
    &new.question_id,
  );

  let changes = add_string_field_change(
    changes,
    AnswerField::QuestionText,
    &old.question_text,
    &new.question_text,
  );

  let changes = add_perspective_field_change(changes, old.perspective, new.perspective);

  let changes = add_u32_field_change(changes, AnswerField::Round, old.round, new.round);

  let changes =
    add_string_field_change(changes, AnswerField::Response, &old.response, &new.response);

  let changes = add_hashmap_field_change(
    changes,
    AnswerField::Extracted,
    &old.extracted,
    &new.extracted,
  );

  let changes = add_f64_field_change(
    changes,
    AnswerField::Confidence,
    old.confidence,
    new.confidence,
  );

  let changes = add_string_field_change(changes, AnswerField::Notes, &old.notes, &new.notes);

  add_string_field_change(
    changes,
    AnswerField::Timestamp,
    &old.timestamp,
    &new.timestamp,
  )
}

fn add_string_field_change(
  changes: Vec<AnswerFieldDiff>,
  field: AnswerField,
  old: &str,
  new: &str,
) -> Vec<AnswerFieldDiff> {
  if old == new {
    changes
  } else {
    let mut changes = changes;
    changes.push(AnswerFieldDiff {
      field,
      old_value: Some(old.to_string()),
      new_value: Some(new.to_string()),
    });
    changes
  }
}

fn add_u32_field_change(
  changes: Vec<AnswerFieldDiff>,
  field: AnswerField,
  old: u32,
  new: u32,
) -> Vec<AnswerFieldDiff> {
  if old == new {
    changes
  } else {
    let mut changes = changes;
    changes.push(AnswerFieldDiff {
      field,
      old_value: Some(old.to_string()),
      new_value: Some(new.to_string()),
    });
    changes
  }
}

fn add_f64_field_change(
  changes: Vec<AnswerFieldDiff>,
  field: AnswerField,
  old: f64,
  new: f64,
) -> Vec<AnswerFieldDiff> {
  // Use approximate comparison for floats
  if (old - new).abs() < f64::EPSILON {
    changes
  } else {
    let mut changes = changes;
    changes.push(AnswerFieldDiff {
      field,
      old_value: Some(format_confidence(old)),
      new_value: Some(format_confidence(new)),
    });
    changes
  }
}

fn add_perspective_field_change(
  changes: Vec<AnswerFieldDiff>,
  old: Perspective,
  new: Perspective,
) -> Vec<AnswerFieldDiff> {
  if old == new {
    changes
  } else {
    let mut changes = changes;
    changes.push(AnswerFieldDiff {
      field: AnswerField::Perspective,
      old_value: Some(perspective_to_string(&old)),
      new_value: Some(perspective_to_string(&new)),
    });
    changes
  }
}

fn add_hashmap_field_change(
  changes: Vec<AnswerFieldDiff>,
  field: AnswerField,
  old: &HashMap<String, String>,
  new: &HashMap<String, String>,
) -> Vec<AnswerFieldDiff> {
  if old == new {
    changes
  } else {
    let mut changes = changes;
    changes.push(AnswerFieldDiff {
      field,
      old_value: Some(hashmap_to_string(old)),
      new_value: Some(hashmap_to_string(new)),
    });
    changes
  }
}

fn perspective_to_string(p: &Perspective) -> String {
  match p {
    Perspective::User => "User".to_string(),
    Perspective::Developer => "Developer".to_string(),
    Perspective::Ops => "Ops".to_string(),
    Perspective::Security => "Security".to_string(),
    Perspective::Business => "Business".to_string(),
  }
}

fn hashmap_to_string(map: &HashMap<String, String>) -> String {
  if map.is_empty() {
    return "{}".to_string();
  }
  let entries: Vec<String> = map.iter().map(|(k, v)| format!("{k}: {v}")).collect();
  format!("{{{}}}", entries.join(", "))
}

fn format_confidence(value: f64) -> String {
  format!("{value:.2}")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod field_diff_tests {
  use super::*;
  use std::collections::HashMap;

  fn make_answer(
    question_id: &str,
    response: &str,
    confidence: f64,
    extracted: HashMap<String, String>,
  ) -> Answer {
    Answer {
      question_id: question_id.to_string(),
      question_text: format!("Question for {question_id}"),
      perspective: Perspective::User,
      round: 1,
      response: response.to_string(),
      extracted,
      confidence,
      notes: String::new(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
    }
  }

  #[test]
  fn test_diff_answer_fields_no_change() {
    let answer = make_answer("q1", "response", 0.9, HashMap::new());
    let diff = diff_answer_fields(&answer, &answer);

    assert_eq!(diff.question_id, "q1");
    assert_eq!(diff.change_type, AnswerChangeType::Modified);
    assert!(diff.field_changes.is_empty());
    assert!(!diff.has_changes());
  }

  #[test]
  fn test_diff_answer_fields_response_changed() {
    let old = make_answer("q1", "old response", 0.9, HashMap::new());
    let new = make_answer("q1", "new response", 0.9, HashMap::new());
    let diff = diff_answer_fields(&old, &new);

    assert_eq!(diff.change_type, AnswerChangeType::Modified);
    assert!(diff.has_changes());
    assert!(diff.has_field_change(AnswerField::Response));
    assert!(!diff.has_field_change(AnswerField::Confidence));

    let response_diff = diff.get_field_diff(AnswerField::Response);
    assert!(response_diff.is_some());
    let response_diff = response_diff.unwrap();
    assert_eq!(response_diff.old_value, Some("old response".to_string()));
    assert_eq!(response_diff.new_value, Some("new response".to_string()));
  }

  #[test]
  fn test_diff_answer_fields_confidence_changed() {
    let old = make_answer("q1", "response", 0.5, HashMap::new());
    let new = make_answer("q1", "response", 0.9, HashMap::new());
    let diff = diff_answer_fields(&old, &new);

    assert_eq!(diff.changed_field_count(), 1);
    assert!(diff.has_field_change(AnswerField::Confidence));

    let confidence_diff = diff.get_field_diff(AnswerField::Confidence).unwrap();
    assert_eq!(confidence_diff.old_value, Some("0.50".to_string()));
    assert_eq!(confidence_diff.new_value, Some("0.90".to_string()));
  }

  #[test]
  fn test_diff_answer_fields_multiple_changes() {
    let old = make_answer("q1", "old", 0.5, HashMap::new());
    let mut new = make_answer("q1", "new", 0.9, HashMap::new());
    new.round = 2;
    new.notes = "updated".to_string();

    let diff = diff_answer_fields(&old, &new);

    assert_eq!(diff.changed_field_count(), 4);
    assert!(diff.has_field_change(AnswerField::Response));
    assert!(diff.has_field_change(AnswerField::Confidence));
    assert!(diff.has_field_change(AnswerField::Round));
    assert!(diff.has_field_change(AnswerField::Notes));
  }

  #[test]
  fn test_diff_answer_fields_extracted_changed() {
    let mut old_extracted = HashMap::new();
    old_extracted.insert("key1".to_string(), "value1".to_string());

    let mut new_extracted = HashMap::new();
    new_extracted.insert("key1".to_string(), "value1_updated".to_string());
    new_extracted.insert("key2".to_string(), "value2".to_string());

    let old = make_answer("q1", "response", 0.9, old_extracted);
    let new = make_answer("q1", "response", 0.9, new_extracted);
    let diff = diff_answer_fields(&old, &new);

    assert_eq!(diff.changed_field_count(), 1);
    assert!(diff.has_field_change(AnswerField::Extracted));
  }

  #[test]
  fn test_diff_answer_added() {
    let answer = make_answer("q1", "response", 0.9, HashMap::new());
    let diff = diff_answer_added(&answer);

    assert_eq!(diff.change_type, AnswerChangeType::Added);
    assert_eq!(diff.changed_field_count(), 9);

    // All fields should have new_value but no old_value
    for field_diff in &diff.field_changes {
      assert!(field_diff.old_value.is_none());
      assert!(field_diff.new_value.is_some());
    }
  }

  #[test]
  fn test_diff_answer_removed() {
    let answer = make_answer("q1", "response", 0.9, HashMap::new());
    let diff = diff_answer_removed(&answer);

    assert_eq!(diff.change_type, AnswerChangeType::Removed);
    assert_eq!(diff.changed_field_count(), 9);

    // All fields should have old_value but no new_value
    for field_diff in &diff.field_changes {
      assert!(field_diff.old_value.is_some());
      assert!(field_diff.new_value.is_none());
    }
  }

  #[test]
  fn test_diff_sessions_field_level() {
    let mut from_session = InterviewSession::default();
    from_session.id = "session-1".to_string();
    from_session
      .answers
      .push(make_answer("q1", "old1", 0.5, HashMap::new()));
    from_session
      .answers
      .push(make_answer("q2", "removed", 0.9, HashMap::new()));

    let mut to_session = InterviewSession::default();
    to_session.id = "session-2".to_string();
    to_session
      .answers
      .push(make_answer("q1", "new1", 0.9, HashMap::new()));
    to_session
      .answers
      .push(make_answer("q3", "added", 0.7, HashMap::new()));

    let diffs = diff_sessions_field_level(&from_session, &to_session);

    assert_eq!(diffs.len(), 3);

    let q1_diff = diffs.iter().find(|d| d.question_id == "q1");
    assert!(q1_diff.is_some());
    let q1_diff = q1_diff.unwrap();
    assert_eq!(q1_diff.change_type, AnswerChangeType::Modified);
    assert!(q1_diff.has_field_change(AnswerField::Response));
    assert!(q1_diff.has_field_change(AnswerField::Confidence));

    let q2_diff = diffs.iter().find(|d| d.question_id == "q2");
    assert!(q2_diff.is_some());
    assert_eq!(q2_diff.unwrap().change_type, AnswerChangeType::Removed);

    let q3_diff = diffs.iter().find(|d| d.question_id == "q3");
    assert!(q3_diff.is_some());
    assert_eq!(q3_diff.unwrap().change_type, AnswerChangeType::Added);
  }

  #[test]
  fn test_perspective_change() {
    let old = make_answer("q1", "response", 0.9, HashMap::new());
    let mut new = old.clone();
    new.perspective = Perspective::Developer;

    let diff = diff_answer_fields(&old, &new);

    assert_eq!(diff.changed_field_count(), 1);
    assert!(diff.has_field_change(AnswerField::Perspective));

    let perspective_diff = diff.get_field_diff(AnswerField::Perspective).unwrap();
    assert_eq!(perspective_diff.old_value, Some("User".to_string()));
    assert_eq!(perspective_diff.new_value, Some("Developer".to_string()));
  }
}
