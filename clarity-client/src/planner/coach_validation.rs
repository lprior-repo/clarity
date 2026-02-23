//! PME Lattice validation gates for the planning coach.
//!
//! Each planning step is validated synchronously against the relevant PME
//! module heuristics.  Validation runs immediately when an answer is submitted
//! and returns a list of human-readable warnings shown as amber
//! `ValidationBubble` entries in the conversation thread.
//!
//! ## Step → PME module mapping
//! | step_id      | PME module        | gate                                          |
//! |-------------|-------------------|-----------------------------------------------|
//! | problem      | thesis_antithesis | ThesisBuilder — non-empty, ≥ 8 words         |
//! | antithesis   | thesis_antithesis | must differ from `problem` answer             |
//! | solution     | thesis_antithesis | non-empty, no platform-noun language          |
//! | persona      | persona_forge     | mentions role + skill/tools                   |
//! | scenario     | north_star        | ≥ 30 words, narrative verbs, no hand-waving   |
//! | use-cases    | conflict_detection| ≥ 1 "X can Y so that Z" line                  |
//! | constraints  | design_by_contract| mentions tech + deployment context            |
//! | tasks        | (structural)      | ≥ 2 "module: action" lines                    |

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

use clarity_core::pme_lattice::{AntithesisBuilder, ThesisBuilder};

/// A single validation warning surfaced inline in the coach thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoachWarning {
  /// Short label shown in bold (e.g. "Weak thesis")
  pub label: String,
  /// Longer explanation of what to fix
  pub message: String,
}

impl CoachWarning {
  fn new(label: &str, message: &str) -> Self {
    Self {
      label: label.to_string(),
      message: message.to_string(),
    }
  }
}

/// Run PME validation for the given step and value.
///
/// `prior_answers` allows cross-step checks (e.g. antithesis ≠ problem).
/// Returns `Vec::new()` when the answer passes all gates.
#[must_use]
pub fn validate_step(
  step_id: &str,
  value: &str,
  prior_answers: &[(String, String)],
) -> Vec<CoachWarning> {
  match step_id {
    "problem" => validate_problem(value),
    "antithesis" => validate_antithesis(value, prior_answers),
    "solution" => validate_solution(value),
    "persona" => validate_persona(value),
    "scenario" => validate_scenario(value),
    "use-cases" => validate_use_cases(value),
    "constraints" => validate_constraints(value),
    "tasks" => validate_tasks(value),
    _ => Vec::new(),
  }
}

// ─── problem ────────────────────────────────────────────────────────────────

fn validate_problem(value: &str) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();

  match ThesisBuilder::new().statement(value.to_string()).build() {
    Err(_) => {
      warnings.push(CoachWarning::new(
        "Empty problem",
        "Your problem statement is blank. Describe a concrete pain your users face.",
      ));
      return warnings;
    }
    Ok(_) => {}
  }

  let word_count = value.split_whitespace().count();
  if word_count < 8 {
    warnings.push(CoachWarning::new(
      "Too vague",
      "Your problem is very short. Add: who is affected, when it happens, \
             and what the concrete impact is.",
    ));
  }

  let lower = value.to_lowercase();
  for vague in &[
    "hard",
    "difficult",
    "complex",
    "better",
    "improve",
    "easier",
  ] {
    if lower.contains(vague) && word_count < 20 {
      warnings.push(CoachWarning::new(
        "Vague language",
        &format!(
          "'{vague}' is vague. Try: 'Users must manually do X, \
                     which causes Y error/cost/delay.'"
        ),
      ));
      break;
    }
  }

  warnings
}

// ─── antithesis ─────────────────────────────────────────────────────────────

fn validate_antithesis(value: &str, prior_answers: &[(String, String)]) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();

  match AntithesisBuilder::new()
    .counter_statement(value.to_string())
    .build()
  {
    Err(_) => {
      warnings.push(CoachWarning::new(
        "Empty antithesis",
        "Articulate the strongest argument *against* building this.",
      ));
      return warnings;
    }
    Ok(_) => {}
  }

  let problem_val = prior_answers
    .iter()
    .find(|(id, _)| id == "problem")
    .map(|(_, v)| v.as_str())
    .unwrap_or("");

  if !problem_val.is_empty() && value.trim().to_lowercase() == problem_val.trim().to_lowercase() {
    warnings.push(CoachWarning::new(
      "Same as problem",
      "Your antithesis is identical to your problem statement. \
             It should argue the *opposite* — why the status quo might be acceptable.",
    ));
  }

  if value.split_whitespace().count() < 6 {
    warnings.push(CoachWarning::new(
      "Underdeveloped",
      "Add at least one concrete reason the problem might not be worth solving.",
    ));
  }

  warnings
}

// ─── solution ───────────────────────────────────────────────────────────────

fn validate_solution(value: &str) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();

  match ThesisBuilder::new().statement(value.to_string()).build() {
    Err(_) => {
      warnings.push(CoachWarning::new(
        "Empty solution",
        "Describe what the software *does*, not what it is.",
      ));
      return warnings;
    }
    Ok(_) => {}
  }

  let lower = value.to_lowercase();
  for noun in &["platform", "system", "tool", "app", "solution"] {
    if lower.contains(noun) {
      warnings.push(CoachWarning::new(
        "Product-noun language",
        &format!(
          "Avoid '{noun}'. Use action framing: \
                     'Automatically X so that Y' rather than 'A {noun} that…'"
        ),
      ));
      break;
    }
  }

  let inner_sentence_count = value.trim_end_matches('.').matches('.').count();
  if inner_sentence_count > 1 {
    warnings.push(CoachWarning::new(
      "Multiple sentences",
      "One clear sentence. If you need more, your scope may be too broad.",
    ));
  }

  warnings
}

// ─── persona ────────────────────────────────────────────────────────────────

fn validate_persona(value: &str) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();
  let lower = value.to_lowercase();

  let has_role = lower.contains("developer")
    || lower.contains("engineer")
    || lower.contains("designer")
    || lower.contains("manager")
    || lower.contains("founder")
    || lower.contains("analyst")
    || lower.contains("user")
    || lower.contains("team")
    || lower.contains("solo");

  if !has_role {
    warnings.push(CoachWarning::new(
      "Missing role",
      "Describe your user's *role* (e.g. 'solo developer', 'engineering manager'). \
             Without a role, you're designing for nobody.",
    ));
  }

  let has_context = lower.contains("cli")
    || lower.contains("typescript")
    || lower.contains("python")
    || lower.contains("rust")
    || lower.contains("experienced")
    || lower.contains("junior")
    || lower.contains("senior")
    || lower.contains("beginner")
    || lower.contains("expert")
    || lower.contains("comfortable")
    || lower.contains("uses ");

  if !has_context {
    warnings.push(CoachWarning::new(
      "Missing context",
      "Add the tools your persona uses and their technical level. \
             Without this you risk designing for a Straw Man user.",
    ));
  }

  warnings
}

// ─── scenario ───────────────────────────────────────────────────────────────

fn validate_scenario(value: &str) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();

  let word_count = value.split_whitespace().count();
  if word_count < 30 {
    warnings.push(CoachWarning::new(
      "Too brief",
      "A North Star Scenario should be a short story (30+ words). \
             Include: trigger → friction → using your tool → outcome.",
    ));
  }

  let lower = value.to_lowercase();
  let has_narrative = lower.contains(" then ")
    || lower.contains(" when ")
    || lower.contains(" after ")
    || lower.contains(" she ")
    || lower.contains(" he ")
    || lower.contains(" they ")
    || lower.contains(" the user ");

  if !has_narrative && word_count >= 30 {
    warnings.push(CoachWarning::new(
      "Not a narrative",
      "Write this as a story with a character and a sequence of events. \
             E.g. 'Alex is deploying and notices… She opens the tool… It automatically…'",
    ));
  }

  for hedge in &[
    "somehow",
    "magically",
    "automatically",
    "easily",
    "just works",
  ] {
    if lower.contains(hedge) {
      warnings.push(CoachWarning::new(
        "Hand-wavy step",
        &format!(
          "'{hedge}' hides an assumption. What *exactly* happens here? \
                     These are your unknowns — resolve them before agent handoff."
        ),
      ));
      break;
    }
  }

  warnings
}

// ─── use-cases ──────────────────────────────────────────────────────────────

fn validate_use_cases(value: &str) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();

  let lines: Vec<&str> = value
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty())
    .collect();

  if lines.is_empty() {
    warnings.push(CoachWarning::new(
      "No use cases",
      "Write at least one: '[User] can [action] so that [outcome]'.",
    ));
    return warnings;
  }

  let well_formed = lines
    .iter()
    .filter(|l| {
      let lower = l.to_lowercase();
      lower.contains(" can ") && lower.contains(" so that ")
    })
    .count();

  if well_formed == 0 {
    warnings.push(CoachWarning::new(
      "Wrong format",
      "None follow 'X can Y so that Z'. \
             This format makes capabilities explicitly testable.",
    ));
  } else if well_formed < lines.len() {
    let malformed = lines.len() - well_formed;
    warnings.push(CoachWarning::new(
      "Partial format",
      &format!(
        "{malformed} line(s) don't follow 'X can Y so that Z'. \
                 Reformat them so every use case is testable."
      ),
    ));
  }

  warnings
}

// ─── constraints ────────────────────────────────────────────────────────────

fn validate_constraints(value: &str) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();
  let lower = value.to_lowercase();

  let has_tech = lower.contains("rust")
    || lower.contains("typescript")
    || lower.contains("python")
    || lower.contains("go ")
    || lower.contains("node")
    || lower.contains("react")
    || lower.contains("next")
    || lower.contains("axum")
    || lower.contains("postgres")
    || lower.contains("sqlite")
    || lower.contains("docker")
    || lower.contains("aws")
    || lower.contains("vercel")
    || lower.contains("supabase")
    || lower.contains("railway");

  if !has_tech {
    warnings.push(CoachWarning::new(
      "No tech mentioned",
      "List at least the primary language and framework so the agent generates idiomatic code.",
    ));
  }

  if value.split_whitespace().count() < 5 {
    warnings.push(CoachWarning::new(
      "Too sparse",
      "Add deployment target, database, auth approach, or conventions to follow.",
    ));
  }

  warnings
}

// ─── tasks ──────────────────────────────────────────────────────────────────

fn validate_tasks(value: &str) -> Vec<CoachWarning> {
  let mut warnings = Vec::new();

  let lines: Vec<&str> = value
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty())
    .collect();

  if lines.is_empty() {
    warnings.push(CoachWarning::new(
      "No tasks",
      "Break use cases into tasks formatted as 'module: what it does'.",
    ));
    return warnings;
  }

  let with_module = lines.iter().filter(|l| l.contains(':')).count();
  if with_module == 0 {
    warnings.push(CoachWarning::new(
      "Missing module prefix",
      "Prefix each task with its module: 'auth: implement login endpoint'. \
             This helps agents scope their work correctly.",
    ));
  }

  if lines.len() < 2 {
    warnings.push(CoachWarning::new(
      "Too few tasks",
      "Most features need multiple tasks. Each task should be completable in one agent session.",
    ));
  }

  warnings
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty_problem_produces_warning() {
    let w = validate_step("problem", "", &[]);
    assert!(!w.is_empty(), "empty problem should warn");
  }

  #[test]
  fn sufficient_problem_passes() {
    let w = validate_step(
            "problem",
            "Developers manually copy API tokens and manage expiry, causing production auth errors on every deploy.",
            &[],
        );
    assert!(w.is_empty(), "clear problem should pass, got: {w:?}");
  }

  #[test]
  fn antithesis_identical_to_problem_warns() {
    let problem = "The auth process is painful and error-prone.";
    let priors = vec![("problem".to_string(), problem.to_string())];
    let w = validate_step("antithesis", problem, &priors);
    assert!(
      w.iter().any(|x| x.label == "Same as problem"),
      "identical antithesis should warn, got: {w:?}"
    );
  }

  #[test]
  fn antithesis_different_passes() {
    let priors = vec![(
      "problem".to_string(),
      "Auth tokens expire and cause outages.".to_string(),
    )];
    let w = validate_step(
      "antithesis",
      "Developers can already use Vault or AWS Secrets Manager; this solves nothing new.",
      &priors,
    );
    assert!(w.is_empty(), "clear antithesis should pass, got: {w:?}");
  }

  #[test]
  fn use_cases_wrong_format_warns() {
    let w = validate_step("use-cases", "User logs in\nUser views dashboard", &[]);
    assert!(
      w.iter().any(|x| x.label == "Wrong format"),
      "use cases without 'can/so that' should warn, got: {w:?}"
    );
  }

  #[test]
  fn use_cases_correct_format_passes() {
    let w = validate_step(
      "use-cases",
      "Developer can rotate API keys automatically so that tokens never expire in production.",
      &[],
    );
    assert!(w.is_empty(), "well-formed use case should pass, got: {w:?}");
  }

  #[test]
  fn tasks_without_module_warns() {
    let w = validate_step("tasks", "implement login\nadd dashboard", &[]);
    assert!(
      w.iter().any(|x| x.label == "Missing module prefix"),
      "tasks without 'module:' prefix should warn, got: {w:?}"
    );
  }

  #[test]
  fn tasks_with_module_passes() {
    let w = validate_step(
      "tasks",
      "auth: implement login endpoint\nui: build login form",
      &[],
    );
    assert!(w.is_empty(), "well-formed tasks should pass, got: {w:?}");
  }

  #[test]
  fn unknown_step_returns_empty() {
    let w = validate_step("review", "anything", &[]);
    assert!(w.is_empty());
  }
}
