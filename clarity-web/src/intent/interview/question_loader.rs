//! Question Loader
//!
//! Loads interview questions from CUE files at runtime.
//! Supports custom questions from .intent/custom-questions.cue
//!
//! Ported from intent-cli/src/intent/question_loader.gleam

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use thiserror::Error;

use super::types::{Perspective, Question, QuestionCategory, QuestionPriority};
use crate::intent::interview::question_types::{
  QuestionCategoryType, QuestionPerspective, QuestionPriorityType,
};

/// Default path for custom questions
const CUSTOM_QUESTIONS_PATH: &str = ".intent/custom-questions.cue";

/// Errors for question loading
#[derive(Debug, Clone, Error)]
pub enum QuestionLoadError {
  #[error("file not found: {path}")]
  FileNotFound { path: String },

  #[error("CUE export failed: {message}")]
  CueExportError { message: String },

  #[error("JSON parse error: {message}")]
  JsonParseError { message: String },

  #[error("question parse error: {message}")]
  QuestionParseError { message: String },

  #[error("security error: {message}")]
  SecurityError { message: String },
}

/// Loaded questions database
#[derive(Debug, Clone, Default)]
pub struct QuestionsDatabase {
  pub api: ProfileQuestions,
  pub cli: ProfileQuestions,
  pub event: ProfileQuestions,
  pub data: ProfileQuestions,
  pub workflow: ProfileQuestions,
  pub ui: ProfileQuestions,
  pub common: CommonQuestions,
}

/// Questions for a specific profile by round
#[derive(Debug, Clone, Default)]
pub struct ProfileQuestions {
  pub round_1: Vec<Question>,
  pub round_2: Vec<Question>,
}

/// Common questions shared across all profiles
#[derive(Debug, Clone, Default)]
pub struct CommonQuestions {
  pub round_3: Vec<Question>,
  pub round_4: Vec<Question>,
  pub round_5: Vec<Question>,
}

/// Custom questions - optional overrides/additions
#[derive(Debug, Clone, Default)]
pub struct CustomQuestions {
  pub api: Option<CustomProfileQuestions>,
  pub cli: Option<CustomProfileQuestions>,
  pub event: Option<CustomProfileQuestions>,
  pub data: Option<CustomProfileQuestions>,
  pub workflow: Option<CustomProfileQuestions>,
  pub ui: Option<CustomProfileQuestions>,
  pub common: Option<CustomCommonQuestions>,
}

/// Custom profile questions
#[derive(Debug, Clone, Default)]
pub struct CustomProfileQuestions {
  pub round_1: Option<Vec<Question>>,
  pub round_2: Option<Vec<Question>>,
}

/// Custom common questions
#[derive(Debug, Clone, Default)]
pub struct CustomCommonQuestions {
  pub round_3: Option<Vec<Question>>,
  pub round_4: Option<Vec<Question>>,
  pub round_5: Option<Vec<Question>>,
}

/// Load questions from a CUE file
///
/// # Errors
/// Returns `QuestionLoadError` if the file cannot be read or parsed.
pub fn load_questions(path: &str) -> Result<QuestionsDatabase, QuestionLoadError> {
  if path.is_empty() {
    return Err(QuestionLoadError::FileNotFound {
      path: path.to_string(),
    });
  }

  let contents = std::fs::read_to_string(path).map_err(|_| QuestionLoadError::FileNotFound {
    path: path.to_string(),
  })?;

  parse_questions_json(&contents)
}

/// Load questions from the default schema path, merging with custom questions
///
/// # Errors
/// Returns `QuestionLoadError` if questions cannot be loaded.
pub fn load_default_questions() -> Result<QuestionsDatabase, QuestionLoadError> {
  // Load built-in questions first
  let db = load_questions("schema/questions.cue")?;

  // Try to load custom questions and merge them
  match load_custom_questions(CUSTOM_QUESTIONS_PATH) {
    Ok(custom) => Ok(merge_custom_questions(&db, &custom)),
    Err(_) => Ok(db), // No custom questions or error loading - use defaults
  }
}

/// Load custom questions from a path
///
/// # Errors
/// Returns `QuestionLoadError` if the file cannot be read or parsed.
pub fn load_custom_questions(path: &str) -> Result<CustomQuestions, QuestionLoadError> {
  if path.is_empty() {
    return Err(QuestionLoadError::FileNotFound {
      path: path.to_string(),
    });
  }

  let contents = std::fs::read_to_string(path).map_err(|_| QuestionLoadError::FileNotFound {
    path: path.to_string(),
  })?;

  parse_custom_questions_json(&contents)
}

/// Parse JSON content into questions database
fn parse_questions_json(json_str: &str) -> Result<QuestionsDatabase, QuestionLoadError> {
  let value: serde_json::Value =
    serde_json::from_str(json_str).map_err(|error| QuestionLoadError::JsonParseError {
      message: error.to_string(),
    })?;

  parse_database(&value)
}

/// Parse database from JSON value
fn parse_database(value: &serde_json::Value) -> Result<QuestionsDatabase, QuestionLoadError> {
  let obj = value
    .as_object()
    .ok_or_else(|| QuestionLoadError::QuestionParseError {
      message: "Root must be an object".to_string(),
    })?;

  Ok(QuestionsDatabase {
    api: parse_profile_questions(obj.get("api"))?,
    cli: parse_profile_questions(obj.get("cli"))?,
    event: parse_profile_questions(obj.get("event"))?,
    data: parse_profile_questions(obj.get("data"))?,
    workflow: parse_profile_questions(obj.get("workflow"))?,
    ui: parse_profile_questions(obj.get("ui"))?,
    common: parse_common_questions(obj.get("common"))?,
  })
}

/// Parse profile questions from JSON value
fn parse_profile_questions(
  value: Option<&serde_json::Value>,
) -> Result<ProfileQuestions, QuestionLoadError> {
  match value {
    Some(v) => {
      let obj = v
        .as_object()
        .ok_or_else(|| QuestionLoadError::QuestionParseError {
          message: "Profile questions must be an object".to_string(),
        })?;

      Ok(ProfileQuestions {
        round_1: parse_question_list(obj.get("round_1"))?,
        round_2: parse_question_list(obj.get("round_2"))?,
      })
    }
    None => Ok(ProfileQuestions::default()),
  }
}

/// Parse common questions from JSON value
fn parse_common_questions(
  value: Option<&serde_json::Value>,
) -> Result<CommonQuestions, QuestionLoadError> {
  match value {
    Some(v) => {
      let obj = v
        .as_object()
        .ok_or_else(|| QuestionLoadError::QuestionParseError {
          message: "Common questions must be an object".to_string(),
        })?;

      Ok(CommonQuestions {
        round_3: parse_question_list(obj.get("round_3"))?,
        round_4: parse_question_list(obj.get("round_4"))?,
        round_5: parse_question_list(obj.get("round_5"))?,
      })
    }
    None => Ok(CommonQuestions::default()),
  }
}

/// Parse a list of questions from JSON value
fn parse_question_list(
  value: Option<&serde_json::Value>,
) -> Result<Vec<Question>, QuestionLoadError> {
  match value {
    Some(serde_json::Value::Array(arr)) => arr.iter().map(parse_question).collect(),
    _ => Ok(Vec::new()),
  }
}

/// Parse a single question from JSON value
fn parse_question(value: &serde_json::Value) -> Result<Question, QuestionLoadError> {
  let obj = value
    .as_object()
    .ok_or_else(|| QuestionLoadError::QuestionParseError {
      message: "Question must be an object".to_string(),
    })?;

  let id = get_string_field(obj, "id")?;
  let round = get_u32_field(obj, "round")?;
  let perspective_str = get_string_field(obj, "perspective")?;
  let category_str = get_string_field(obj, "category")?;
  let priority_str = get_string_field(obj, "priority")?;
  let question_text = get_string_field(obj, "question")?;
  let context = get_string_field(obj, "context")?;
  let example = get_string_field(obj, "example")?;

  // Convert string types to enum types
  let perspective = convert_perspective(&perspective_str);
  let category = convert_category(&category_str);
  let priority = convert_priority(&priority_str);

  Ok(Question {
    id,
    round,
    perspective,
    category,
    priority,
    question: question_text,
    context,
    example,
    expected_type: get_optional_string_field(obj, "expected_type")
      .map_or_else(|| "text".to_string(), |v| v),
    extract_into: get_optional_string_list(obj, "extract_into").unwrap_or_default(),
    depends_on: get_optional_string_list(obj, "depends_on").unwrap_or_default(),
    blocks: get_optional_string_list(obj, "blocks").unwrap_or_default(),
  })
}

/// Convert perspective string to enum
fn convert_perspective(s: &str) -> Perspective {
  match QuestionPerspective::parse(s) {
    Ok(QuestionPerspective::Developer) => Perspective::Developer,
    Ok(QuestionPerspective::Ops) => Perspective::Ops,
    Ok(QuestionPerspective::Security) => Perspective::Security,
    Ok(QuestionPerspective::Business) => Perspective::Business,
    Ok(QuestionPerspective::User) | Err(_) => Perspective::User,
  }
}

/// Convert category string to enum
fn convert_category(s: &str) -> QuestionCategory {
  match QuestionCategoryType::parse(s) {
    Ok(QuestionCategoryType::ErrorCase) => QuestionCategory::ErrorCase,
    Ok(QuestionCategoryType::EdgeCase) => QuestionCategory::EdgeCase,
    Ok(QuestionCategoryType::Constraint) => QuestionCategory::Constraint,
    Ok(QuestionCategoryType::Dependency) => QuestionCategory::Dependency,
    Ok(QuestionCategoryType::NonFunctional) => QuestionCategory::NonFunctional,
    Ok(QuestionCategoryType::HappyPath) | Err(_) => QuestionCategory::HappyPath,
  }
}

/// Convert priority string to enum
fn convert_priority(s: &str) -> QuestionPriority {
  match QuestionPriorityType::parse(s) {
    Ok(QuestionPriorityType::Critical) => QuestionPriority::Critical,
    Ok(QuestionPriorityType::NiceToHave) => QuestionPriority::NiceToHave,
    Ok(QuestionPriorityType::Important) | Err(_) => QuestionPriority::Important,
  }
}

/// Get required string field from object
fn get_string_field(
  obj: &serde_json::Map<String, serde_json::Value>,
  key: &str,
) -> Result<String, QuestionLoadError> {
  obj
    .get(key)
    .and_then(|v| v.as_str())
    .map(String::from)
    .ok_or_else(|| QuestionLoadError::QuestionParseError {
      message: format!("Missing or invalid field: {key}"),
    })
}

/// Get required u32 field from object
fn get_u32_field(
  obj: &serde_json::Map<String, serde_json::Value>,
  key: &str,
) -> Result<u32, QuestionLoadError> {
  obj
    .get(key)
    .and_then(serde_json::Value::as_u64)
    .map(|n| u32::try_from(n).map_or(1, |v| v))
    .ok_or_else(|| QuestionLoadError::QuestionParseError {
      message: format!("Missing or invalid field: {key}"),
    })
}

/// Get optional string field from object
fn get_optional_string_field(
  obj: &serde_json::Map<String, serde_json::Value>,
  key: &str,
) -> Option<String> {
  obj.get(key).and_then(|v| v.as_str()).map(String::from)
}

/// Get optional string list field from object
fn get_optional_string_list(
  obj: &serde_json::Map<String, serde_json::Value>,
  key: &str,
) -> Option<Vec<String>> {
  obj.get(key).and_then(|v| {
    v.as_array().map(|arr| {
      arr
        .iter()
        .filter_map(|item| item.as_str().map(String::from))
        .collect()
    })
  })
}

/// Parse custom questions JSON
fn parse_custom_questions_json(json_str: &str) -> Result<CustomQuestions, QuestionLoadError> {
  let value: serde_json::Value =
    serde_json::from_str(json_str).map_err(|error| QuestionLoadError::JsonParseError {
      message: error.to_string(),
    })?;

  parse_custom_database(&value)
}

/// Parse custom database from JSON value
fn parse_custom_database(value: &serde_json::Value) -> Result<CustomQuestions, QuestionLoadError> {
  let obj = value.as_object();

  match obj {
    Some(obj) => Ok(CustomQuestions {
      api: parse_custom_profile_questions(obj.get("api"))?,
      cli: parse_custom_profile_questions(obj.get("cli"))?,
      event: parse_custom_profile_questions(obj.get("event"))?,
      data: parse_custom_profile_questions(obj.get("data"))?,
      workflow: parse_custom_profile_questions(obj.get("workflow"))?,
      ui: parse_custom_profile_questions(obj.get("ui"))?,
      common: parse_custom_common_questions(obj.get("common"))?,
    }),
    None => Ok(CustomQuestions::default()),
  }
}

/// Parse custom profile questions
fn parse_custom_profile_questions(
  value: Option<&serde_json::Value>,
) -> Result<Option<CustomProfileQuestions>, QuestionLoadError> {
  match value {
    Some(v) if v.is_object() => {
      let obj = v
        .as_object()
        .ok_or_else(|| QuestionLoadError::QuestionParseError {
          message: "Custom profile questions must be an object".to_string(),
        })?;

      Ok(Some(CustomProfileQuestions {
        round_1: parse_question_list(obj.get("round_1")).ok(),
        round_2: parse_question_list(obj.get("round_2")).ok(),
      }))
    }
    _ => Ok(None),
  }
}

/// Parse custom common questions
fn parse_custom_common_questions(
  value: Option<&serde_json::Value>,
) -> Result<Option<CustomCommonQuestions>, QuestionLoadError> {
  match value {
    Some(v) if v.is_object() => {
      let obj = v
        .as_object()
        .ok_or_else(|| QuestionLoadError::QuestionParseError {
          message: "Custom common questions must be an object".to_string(),
        })?;

      Ok(Some(CustomCommonQuestions {
        round_3: parse_question_list(obj.get("round_3")).ok(),
        round_4: parse_question_list(obj.get("round_4")).ok(),
        round_5: parse_question_list(obj.get("round_5")).ok(),
      }))
    }
    _ => Ok(None),
  }
}

/// Merge custom questions with built-in questions
/// Custom questions with same ID override built-ins; new IDs are added
fn merge_custom_questions(db: &QuestionsDatabase, custom: &CustomQuestions) -> QuestionsDatabase {
  QuestionsDatabase {
    api: merge_profile(&db.api, custom.api.as_ref()),
    cli: merge_profile(&db.cli, custom.cli.as_ref()),
    event: merge_profile(&db.event, custom.event.as_ref()),
    data: merge_profile(&db.data, custom.data.as_ref()),
    workflow: merge_profile(&db.workflow, custom.workflow.as_ref()),
    ui: merge_profile(&db.ui, custom.ui.as_ref()),
    common: merge_common(&db.common, custom.common.as_ref()),
  }
}

/// Merge profile questions
fn merge_profile(
  base: &ProfileQuestions,
  custom: Option<&CustomProfileQuestions>,
) -> ProfileQuestions {
  custom.map_or_else(
    || base.clone(),
    |c| ProfileQuestions {
      round_1: merge_question_list(&base.round_1, c.round_1.as_deref()),
      round_2: merge_question_list(&base.round_2, c.round_2.as_deref()),
    },
  )
}

/// Merge common questions
fn merge_common(base: &CommonQuestions, custom: Option<&CustomCommonQuestions>) -> CommonQuestions {
  custom.map_or_else(
    || base.clone(),
    |c| CommonQuestions {
      round_3: merge_question_list(&base.round_3, c.round_3.as_deref()),
      round_4: merge_question_list(&base.round_4, c.round_4.as_deref()),
      round_5: merge_question_list(&base.round_5, c.round_5.as_deref()),
    },
  )
}

/// Merge question lists, with custom overriding by ID
fn merge_question_list(base: &[Question], custom: Option<&[Question]>) -> Vec<Question> {
  custom.map_or_else(
    || base.to_vec(),
    |custom_questions| {
      base
        .iter()
        .filter(|question| {
          custom_questions
            .iter()
            .all(|custom_question| custom_question.id != question.id)
        })
        .cloned()
        .chain(custom_questions.iter().cloned())
        .collect()
    },
  )
}

/// Get questions for a specific profile and round from a loaded database
#[must_use]
pub fn get_questions(db: &QuestionsDatabase, profile: &str, round: u32) -> Vec<Question> {
  match (profile, round) {
    ("api", 1) => db.api.round_1.clone(),
    ("api", 2) => db.api.round_2.clone(),
    ("cli", 1) => db.cli.round_1.clone(),
    ("cli", 2) => db.cli.round_2.clone(),
    ("event", 1) => db.event.round_1.clone(),
    ("event", 2) => db.event.round_2.clone(),
    ("data", 1) => db.data.round_1.clone(),
    ("data", 2) => db.data.round_2.clone(),
    ("workflow", 1) => db.workflow.round_1.clone(),
    ("workflow", 2) => db.workflow.round_2.clone(),
    ("ui", 1) => db.ui.round_1.clone(),
    ("ui", 2) => db.ui.round_2.clone(),
    (_, 3) => db.common.round_3.clone(),
    (_, 4) => db.common.round_4.clone(),
    (_, 5) => db.common.round_5.clone(),
    _ => Vec::new(),
  }
}

/// Format a `QuestionLoadError` as a human-readable string
#[must_use]
pub fn format_error(error: &QuestionLoadError) -> String {
  match error {
    QuestionLoadError::FileNotFound { path } => format!("Questions file not found: {path}"),
    QuestionLoadError::CueExportError { message } => format!("CUE export failed:\n{message}"),
    QuestionLoadError::JsonParseError { message } => format!("JSON parse error: {message}"),
    QuestionLoadError::QuestionParseError { message } => {
      format!("Question parse error: {message}")
    }
    QuestionLoadError::SecurityError { message } => message.clone(),
  }
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
  fn test_parse_questions_json_empty() {
    let json = r"{}";
    let db = parse_questions_json(json).unwrap();
    assert!(db.api.round_1.is_empty());
    assert!(db.common.round_3.is_empty());
  }

  #[test]
  fn test_parse_questions_json_with_questions() {
    let json = r#"{
      "api": {
        "round_1": [
          {
            "id": "q1",
            "round": 1,
            "perspective": "user",
            "category": "happy_path",
            "priority": "critical",
            "question": "What should this API do?",
            "context": "Core purpose",
            "example": "Describe the API"
          }
        ]
      }
    }"#;
    let db = parse_questions_json(json).unwrap();
    assert_eq!(db.api.round_1.len(), 1);
    assert_eq!(db.api.round_1[0].id, "q1");
  }

  #[test]
  fn test_get_questions_api_round_1() {
    let db = QuestionsDatabase {
      api: ProfileQuestions {
        round_1: vec![Question {
          id: "test".to_string(),
          ..Question::default()
        }],
        round_2: Vec::new(),
      },
      ..QuestionsDatabase::default()
    };
    let questions = get_questions(&db, "api", 1);
    assert_eq!(questions.len(), 1);
  }

  #[test]
  fn test_get_questions_unknown_profile() {
    let db = QuestionsDatabase::default();
    let questions = get_questions(&db, "unknown", 1);
    assert!(questions.is_empty());
  }

  #[test]
  fn test_get_questions_common_rounds() {
    let db = QuestionsDatabase {
      common: CommonQuestions {
        round_3: vec![Question {
          id: "common-q".to_string(),
          ..Question::default()
        }],
        ..CommonQuestions::default()
      },
      ..QuestionsDatabase::default()
    };
    let questions = get_questions(&db, "api", 3);
    assert_eq!(questions.len(), 1);
  }

  #[test]
  fn test_merge_question_list_no_custom() {
    let base = vec![Question {
      id: "base".to_string(),
      ..Question::default()
    }];
    let merged = merge_question_list(&base, None);
    assert_eq!(merged.len(), 1);
  }

  #[test]
  fn test_merge_question_list_with_override() {
    let base = vec![Question {
      id: "q1".to_string(),
      question: "base question".to_string(),
      ..Question::default()
    }];
    let custom = vec![Question {
      id: "q1".to_string(),
      question: "custom question".to_string(),
      ..Question::default()
    }];
    let merged = merge_question_list(&base, Some(&custom));
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].question, "custom question");
  }

  #[test]
  fn test_merge_question_list_with_new() {
    let base = vec![Question {
      id: "q1".to_string(),
      ..Question::default()
    }];
    let custom = vec![Question {
      id: "q2".to_string(),
      ..Question::default()
    }];
    let merged = merge_question_list(&base, Some(&custom));
    assert_eq!(merged.len(), 2);
  }

  #[test]
  fn test_format_error_file_not_found() {
    let error = QuestionLoadError::FileNotFound {
      path: "test.cue".to_string(),
    };
    assert!(format_error(&error).contains("test.cue"));
  }

  #[test]
  fn test_load_questions_empty_path() {
    let result = load_questions("");
    assert!(matches!(
      result,
      Err(QuestionLoadError::FileNotFound { path }) if path.is_empty()
    ));
  }

  #[test]
  fn test_parse_questions_json_invalid_json_has_typed_payload() {
    let result = parse_questions_json("not json");

    assert!(matches!(
      result,
      Err(QuestionLoadError::JsonParseError { message }) if !message.is_empty()
    ));
  }

  #[test]
  fn test_parse_questions_json_non_object_root_has_typed_payload() {
    let result = parse_questions_json("[]");

    assert!(matches!(
      result,
      Err(QuestionLoadError::QuestionParseError { message })
        if message == "Root must be an object"
    ));
  }

  #[test]
  fn test_default_questions_database() {
    let db = QuestionsDatabase::default();
    assert!(db.api.round_1.is_empty());
    assert!(db.common.round_3.is_empty());
  }
}
