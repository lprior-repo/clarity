use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::ProfileParseError;

/// Profile type - determines which questions to ask and required fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Profile {
  #[default]
  Api,
  Cli,
  Event,
  Data,
  Workflow,
  Ui,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum InterviewStage {
  #[default]
  Discovery,
  Refinement,
  Validation,
  Complete,
  Paused,
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
#[derive(Default)]
pub enum Perspective {
  #[default]
  User,
  Developer,
  Ops,
  Security,
  Business,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum QuestionPriority {
  Critical,
  #[default]
  Important,
  NiceToHave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum QuestionCategory {
  #[default]
  HappyPath,
  ErrorCase,
  EdgeCase,
  Constraint,
  Dependency,
  NonFunctional,
}
