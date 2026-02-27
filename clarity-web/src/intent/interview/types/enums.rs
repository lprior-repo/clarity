use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::ProfileParseError;

/// Profile type - determines which questions to ask and required fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
  Api,
  Cli,
  Event,
  Data,
  Workflow,
  Ui,
}

impl Default for Profile {
  fn default() -> Self {
    Self::Api
  }
}

impl Profile {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Api => "api",
      Self::Cli => "cli",
      Self::Event => "event",
      Self::Data => "data",
      Self::Workflow => "workflow",
      Self::Ui => "ui",
    }
  }

  /// # Errors
  /// Returns `ProfileParseError` when input is not a known profile.
  pub fn from_str(s: &str) -> Result<Self, ProfileParseError> {
    let normalized = s.trim().to_ascii_lowercase();
    match normalized.as_str() {
      "api" => Ok(Self::Api),
      "cli" => Ok(Self::Cli),
      "event" => Ok(Self::Event),
      "data" => Ok(Self::Data),
      "workflow" => Ok(Self::Workflow),
      "ui" => Ok(Self::Ui),
      _ => Err(ProfileParseError::UnknownProfile {
        input: s.to_string(),
      }),
    }
  }

  #[must_use]
  pub const fn required_fields(&self) -> &'static [&'static str] {
    match self {
      Self::Api => &[
        "base_url",
        "auth_method",
        "happy_path",
        "error_cases",
        "response_format",
      ],
      Self::Cli => &["command_name", "happy_path", "help_text", "exit_codes"],
      Self::Event => &["event_type", "payload_schema", "trigger"],
      Self::Data => &["data_model", "access_patterns", "retention"],
      Self::Workflow => &["steps", "happy_path", "error_recovery"],
      Self::Ui => &["user_flows", "happy_path", "states"],
    }
  }
}

impl FromStr for Profile {
  type Err = ProfileParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::from_str(s)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterviewStage {
  Discovery,
  Refinement,
  Validation,
  Complete,
  Paused,
}

impl Default for InterviewStage {
  fn default() -> Self {
    Self::Discovery
  }
}

impl InterviewStage {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Discovery => "discovery",
      Self::Refinement => "refinement",
      Self::Validation => "validation",
      Self::Complete => "complete",
      Self::Paused => "paused",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Perspective {
  User,
  Developer,
  Ops,
  Security,
  Business,
}

impl Default for Perspective {
  fn default() -> Self {
    Self::User
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionPriority {
  Critical,
  Important,
  NiceToHave,
}

impl Default for QuestionPriority {
  fn default() -> Self {
    Self::Important
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionCategory {
  HappyPath,
  ErrorCase,
  EdgeCase,
  Constraint,
  Dependency,
  NonFunctional,
}

impl Default for QuestionCategory {
  fn default() -> Self {
    Self::HappyPath
  }
}
