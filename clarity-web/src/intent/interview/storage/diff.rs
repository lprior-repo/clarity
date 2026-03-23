use super::models::{AnswerChangeType, AnswerDiff, SessionDiff, SessionSnapshot};
use crate::intent::interview::types::{InterviewSession, InterviewStage};
use std::collections::HashMap;

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
#[allow(clippy::too_many_lines)]
pub fn format_diff(diff: &SessionDiff) -> String {
  use std::fmt::Write;

  const MAX_RESPONSE_LEN: usize = 50;

  fn truncate(value: &str) -> String {
    if value.chars().count() > MAX_RESPONSE_LEN {
      format!(
        "{}...",
        value.chars().take(MAX_RESPONSE_LEN).collect::<String>()
      )
    } else {
      value.to_string()
    }
  }

  let format_response = |response: &Option<String>| {
    response
      .as_ref()
      .map_or_else(|| "(none)".to_string(), |value| truncate(value))
  };

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
      diff.old_stage.as_deref().map_or("(none)", |s| s),
      diff.new_stage.as_deref().map_or("(none)", |s| s)
    );
  }

  if !diff.answers_added.is_empty() {
    let _ = writeln!(output, "Answers Added ({}):", diff.answers_added.len());
    for answer in &diff.answers_added {
      let _ = writeln!(
        output,
        "  + [{}] {}: {}",
        answer.question_id,
        truncate(&answer.question_text),
        format_response(&answer.new_response)
      );
    }
    output.push('\n');
  }

  if !diff.answers_modified.is_empty() {
    let _ = writeln!(
      output,
      "Answers Modified ({}):",
      diff.answers_modified.len()
    );
    for answer in &diff.answers_modified {
      let _ = writeln!(
        output,
        "  ~ [{}] {}:\n    {} -> {}",
        answer.question_id,
        truncate(&answer.question_text),
        format_response(&answer.old_response),
        format_response(&answer.new_response)
      );
    }
    output.push('\n');
  }

  if !diff.answers_removed.is_empty() {
    let _ = writeln!(output, "Answers Removed ({}):", diff.answers_removed.len());
    for answer in &diff.answers_removed {
      let _ = writeln!(
        output,
        "  - [{}] {}: {}",
        answer.question_id,
        truncate(&answer.question_text),
        format_response(&answer.old_response)
      );
    }
    output.push('\n');
  }

  match diff.gaps_added.cmp(&0) {
    std::cmp::Ordering::Greater => {
      let _ = writeln!(output, "Gaps: +{} new gap(s)", diff.gaps_added);
    }
    std::cmp::Ordering::Less => {
      let _ = writeln!(output, "Gaps: {} gap(s) resolved", -diff.gaps_added);
    }
    std::cmp::Ordering::Equal => {
      let _ = writeln!(output, "Gaps: No change");
    }
  }

  match diff.conflicts_added.cmp(&0) {
    std::cmp::Ordering::Greater => {
      let _ = writeln!(
        output,
        "Conflicts: +{} new conflict(s)",
        diff.conflicts_added
      );
    }
    std::cmp::Ordering::Less => {
      let _ = writeln!(
        output,
        "Conflicts: {} conflict(s) resolved",
        -diff.conflicts_added
      );
    }
    std::cmp::Ordering::Equal => {
      let _ = writeln!(output, "Conflicts: No change");
    }
  }

  output
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
