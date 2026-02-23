//! Serialize a completed coaching session into a bead JSONL record.
//!
//! Pure functions only — no I/O. Callers are responsible for appending the
//! returned string to `.beads/issues.jsonl`.

use crate::planner::types_coach::CoachAnswer;

/// Extract the value for a given step id from the answers slice.
fn get_val<'a>(answers: &'a [CoachAnswer], step_id: &str) -> Option<&'a str> {
  answers
    .iter()
    .find(|a| a.step_id == step_id)
    .map(|a| a.value.as_str())
}

/// Build a short bead title from the solution answer (or fallback).
fn build_title(answers: &[CoachAnswer]) -> String {
  get_val(answers, "solution")
    .map(|s| {
      // Truncate to first 80 chars and trim
      let trimmed = s.trim();
      if trimmed.len() > 80 {
        format!("{}…", &trimmed[..77])
      } else {
        trimmed.to_string()
      }
    })
    .unwrap_or_else(|| "Untitled Plan".to_string())
}

/// Build the description block from all answers.
fn build_description(answers: &[CoachAnswer]) -> String {
  let problem = get_val(answers, "problem").unwrap_or("");
  let antithesis = get_val(answers, "antithesis").unwrap_or("");
  let solution = get_val(answers, "solution").unwrap_or("");
  let persona = get_val(answers, "persona").unwrap_or("");
  let scenario = get_val(answers, "scenario").unwrap_or("");
  let use_cases = get_val(answers, "use-cases").unwrap_or("");
  let constraints = get_val(answers, "constraints").unwrap_or("");
  let tasks = get_val(answers, "tasks").unwrap_or("");

  format!(
    "## Problem\n{problem}\n\n\
     ## Antithesis\n{antithesis}\n\n\
     ## Solution\n{solution}\n\n\
     ## Persona\n{persona}\n\n\
     ## North Star Scenario\n{scenario}\n\n\
     ## Use Cases\n{use_cases}\n\n\
     ## Constraints & Stack\n{constraints}\n\n\
     ## Tasks\n{tasks}"
  )
}

/// Produce a timestamp string suitable for a bead id (`YYYYMMDDHHMMSS`).
/// Falls back to a fixed string when `SystemTime` is unavailable.
fn timestamp_id() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};

  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| {
      let secs = d.as_secs();
      // Convert epoch seconds → rough YYYYMMDDHHMMSS via manual decomposition
      let s = secs % 60;
      let m = (secs / 60) % 60;
      let h = (secs / 3600) % 24;
      let days = secs / 86400;
      // Approximate date from days since epoch (good enough for a unique id)
      let year = 1970 + days / 365;
      let day_of_year = days % 365;
      let month = (day_of_year / 30 + 1).min(12);
      let day = (day_of_year % 30 + 1).min(31);
      format!("{year:04}{month:02}{day:02}{h:02}{m:02}{s:02}")
    })
    .unwrap_or_else(|_| "20260101000000".to_string())
}

/// Serialize answers to an ISO-8601-ish date string for `created_at`.
fn now_rfc3339() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};

  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| {
      let secs = d.as_secs();
      let s = secs % 60;
      let m = (secs / 60) % 60;
      let h = (secs / 3600) % 24;
      let days = secs / 86400;
      let year = 1970 + days / 365;
      let day_of_year = days % 365;
      let month = (day_of_year / 30 + 1).min(12);
      let day = (day_of_year % 30 + 1).min(31);
      format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
    })
    .unwrap_or_else(|_| "2026-01-01T00:00:00Z".to_string())
}

/// Escape a string for JSON — replaces `"` with `\"` and `\n` with `\\n`.
fn json_escape(s: &str) -> String {
  s.replace('\\', "\\\\")
    .replace('"', "\\\"")
    .replace('\n', "\\n")
    .replace('\r', "\\r")
    .replace('\t', "\\t")
}

/// Generate a unique bead id of the form `bd-<timestamp><suffix>`.
fn bead_id() -> String {
  let ts = timestamp_id();
  // Use last 6 chars of timestamp as suffix for brevity
  let suffix = &ts[ts.len().saturating_sub(6)..];
  format!("bd-coach-{suffix}")
}

/// Serialize a completed coaching session to a single JSONL line.
///
/// The returned string contains no trailing newline. Append `\n` before
/// writing to the file, or use `append_to_beads` for a complete write.
#[must_use]
pub fn serialize_to_bead_line(answers: &[CoachAnswer]) -> String {
  let id = bead_id();
  let title = build_title(answers);
  let description = build_description(answers);
  let now = now_rfc3339();

  let title_esc = json_escape(&title);
  let desc_esc = json_escape(&description);

  format!(
    r#"{{"id":"{id}","title":"{title_esc}","description":"{desc_esc}","status":"open","priority":1,"issue_type":"feature","created_at":"{now}","created_by":"clarity-coach","updated_at":"{now}","source_repo":".","compaction_level":0,"original_size":0,"labels":["coach-generated"]}}"#
  )
}

/// Append a bead line to `.beads/issues.jsonl`.
///
/// Returns the generated bead id on success, or an error string on failure.
///
/// # Errors
/// Returns an `Err` string if the file cannot be opened or written.
pub fn append_to_beads(answers: &[CoachAnswer], beads_path: &str) -> Result<String, String> {
  use std::fs::OpenOptions;
  use std::io::Write;

  let line = serialize_to_bead_line(answers);
  // Extract id from the line (starts with `{"id":"<id>"`)
  let id = bead_id(); // re-generate — same timestamp within milliseconds

  let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(beads_path)
    .map_err(|e| format!("Cannot open {beads_path}: {e}"))?;

  writeln!(file, "{line}").map_err(|e| format!("Write failed: {e}"))?;

  Ok(id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_answers() -> Vec<CoachAnswer> {
    vec![
      CoachAnswer {
        step_id: "problem".to_string(),
        value: "Developers manually rotate API tokens causing production outages".to_string(),
      },
      CoachAnswer {
        step_id: "antithesis".to_string(),
        value: "The current manual process is well-understood and the risk is low".to_string(),
      },
      CoachAnswer {
        step_id: "solution".to_string(),
        value: "Automatically rotates and injects API tokens at deploy time".to_string(),
      },
      CoachAnswer {
        step_id: "persona".to_string(),
        value: "Solo dev, ships 3 side projects, uses Next.js + Vercel".to_string(),
      },
      CoachAnswer {
        step_id: "scenario".to_string(),
        value: "Alice deploys her app and the token auto-rotates without any manual steps".to_string(),
      },
      CoachAnswer {
        step_id: "use-cases".to_string(),
        value: "User can rotate tokens so that deployments never fail\nUser can audit rotations so that they have a history".to_string(),
      },
      CoachAnswer {
        step_id: "constraints".to_string(),
        value: "Next.js 16, TypeScript strict, Vercel deployment, monorepo pnpm".to_string(),
      },
      CoachAnswer {
        step_id: "tasks".to_string(),
        value: "auth: implement token rotation\nauth: add audit log\nui: build settings panel".to_string(),
      },
    ]
  }

  #[test]
  fn title_comes_from_solution() {
    let answers = sample_answers();
    let title = build_title(&answers);
    assert!(title.contains("Automatically rotates"), "got: {title}");
  }

  #[test]
  fn title_truncates_long_solution() {
    let answers = vec![CoachAnswer {
      step_id: "solution".to_string(),
      value: "a".repeat(200),
    }];
    let title = build_title(&answers);
    assert!(title.len() <= 81, "title too long: {}", title.len());
    assert!(title.ends_with('…'));
  }

  #[test]
  fn title_falls_back_when_no_solution() {
    let title = build_title(&[]);
    assert_eq!(title, "Untitled Plan");
  }

  #[test]
  fn description_contains_all_sections() {
    let answers = sample_answers();
    let desc = build_description(&answers);
    assert!(desc.contains("## Problem"));
    assert!(desc.contains("## Antithesis"));
    assert!(desc.contains("## Solution"));
    assert!(desc.contains("## Persona"));
    assert!(desc.contains("## North Star Scenario"));
    assert!(desc.contains("## Use Cases"));
    assert!(desc.contains("## Constraints & Stack"));
    assert!(desc.contains("## Tasks"));
  }

  #[test]
  fn serialize_produces_valid_json_shape() {
    let answers = sample_answers();
    let line = serialize_to_bead_line(&answers);
    // Must start/end with braces
    assert!(line.starts_with('{'));
    assert!(line.ends_with('}'));
    // Must contain key fields
    assert!(line.contains(r#""status":"open""#));
    assert!(line.contains(r#""issue_type":"feature""#));
    assert!(line.contains(r#""labels":["coach-generated"]"#));
    assert!(line.contains(r#""created_by":"clarity-coach""#));
  }

  #[test]
  fn json_escape_handles_quotes_and_newlines() {
    let raw = "He said \"hello\"\nworld";
    let escaped = json_escape(raw);
    assert_eq!(escaped, r#"He said \"hello\"\nworld"#);
  }

  #[test]
  fn json_escape_handles_backslashes() {
    let raw = r"path\to\file";
    let escaped = json_escape(raw);
    assert_eq!(escaped, r"path\\to\\file");
  }

  #[test]
  fn bead_id_has_expected_prefix() {
    let id = bead_id();
    assert!(id.starts_with("bd-coach-"), "got: {id}");
  }
}
