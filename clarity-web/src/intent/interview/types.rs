//! Interview Types - Core data structures for interview sessions
//!
//! This module defines the core types used throughout the interview system:
//! - `Profile` - Type of system being specified (Api, Cli, Event, etc.)
//! - `InterviewStage` - State machine states for interview lifecycle
//! - `Perspective` - Viewpoint for questions (User, Developer, Ops, etc.)
//! - `Answer` - Response to an interview question
//! - `Gap` - Missing information blocking spec completion
//! - `Conflict` - Contradictions between requirements
//! - `InterviewSession` - Central state machine for an interview

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Profile type - determines which questions to ask and required fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// REST/GraphQL API specification
    Api,
    /// Command-line interface
    Cli,
    /// Event-driven system
    Event,
    /// Data model/storage
    Data,
    /// Workflow/orchestration
    Workflow,
    /// User interface
    Ui,
}

impl Default for Profile {
    fn default() -> Self {
        Self::Api
    }
}

impl Profile {
    /// Convert profile to string representation
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

    /// Parse profile from string
    ///
    /// # Errors
    /// Returns error if the string doesn't match a known profile
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "api" => Ok(Self::Api),
            "cli" => Ok(Self::Cli),
            "event" => Ok(Self::Event),
            "data" => Ok(Self::Data),
            "workflow" => Ok(Self::Workflow),
            "ui" => Ok(Self::Ui),
            _ => Err(format!("Unknown profile type: {s}")),
        }
    }
}

/// Interview stage - persistent state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterviewStage {
    /// Initial discovery phase
    Discovery,
    /// Refining requirements
    Refinement,
    /// Validating completeness
    Validation,
    /// Interview complete
    Complete,
    /// Paused for later
    Paused,
}

impl Default for InterviewStage {
    fn default() -> Self {
        Self::Discovery
    }
}

impl InterviewStage {
    /// Convert stage to string representation
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

/// Perspective - viewpoint for interview questions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Perspective {
    /// End user perspective
    User,
    /// Developer/maintainer perspective
    Developer,
    /// Operations/SRE perspective
    Ops,
    /// Security/compliance perspective
    Security,
    /// Business/stakeholder perspective
    Business,
}

impl Default for Perspective {
    fn default() -> Self {
        Self::User
    }
}

/// Question priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionPriority {
    /// Must be answered for spec to be valid
    Critical,
    /// Important for spec quality
    Important,
    /// Nice to have for completeness
    NiceToHave,
}

impl Default for QuestionPriority {
    fn default() -> Self {
        Self::Important
    }
}

/// Question category for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionCategory {
    /// Happy path scenarios
    HappyPath,
    /// Error handling cases
    ErrorCase,
    /// Edge cases and boundaries
    EdgeCase,
    /// System constraints
    Constraint,
    /// External dependencies
    Dependency,
    /// Non-functional requirements
    NonFunctional,
}

impl Default for QuestionCategory {
    fn default() -> Self {
        Self::HappyPath
    }
}

/// A single answer with metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    /// Unique question identifier
    pub question_id: String,
    /// The question text that was asked
    pub question_text: String,
    /// Perspective this question was asked from
    pub perspective: Perspective,
    /// Round number (interviews can have multiple rounds)
    pub round: u32,
    /// The response text
    pub response: String,
    /// Fields extracted from the response
    pub extracted: HashMap<String, String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Additional notes
    pub notes: String,
    /// When this answer was recorded
    pub timestamp: String,
}

impl Default for Answer {
    fn default() -> Self {
        Self {
            question_id: String::new(),
            question_text: String::new(),
            perspective: Perspective::default(),
            round: 1,
            response: String::new(),
            extracted: HashMap::new(),
            confidence: 0.0,
            notes: String::new(),
            timestamp: String::new(),
        }
    }
}

/// Gap - missing information blocking spec completion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
    /// Unique gap identifier
    pub id: String,
    /// The field that's missing
    pub field: String,
    /// Description of what's missing
    pub description: String,
    /// Whether this gap blocks progression
    pub blocking: bool,
    /// Suggested default value
    pub suggested_default: String,
    /// Why this field is needed
    pub why_needed: String,
    /// Round when gap was detected
    pub round: u32,
    /// Whether gap has been resolved
    pub resolved: bool,
    /// Resolution text if resolved
    pub resolution: String,
}

impl Default for Gap {
    fn default() -> Self {
        Self {
            id: String::new(),
            field: String::new(),
            description: String::new(),
            blocking: true,
            suggested_default: String::new(),
            why_needed: String::new(),
            round: 1,
            resolved: false,
            resolution: String::new(),
        }
    }
}

/// Conflict - contradictions between requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    /// Unique conflict identifier
    pub id: String,
    /// The two conflicting fields/answers
    pub between: (String, String),
    /// Description of the conflict
    pub description: String,
    /// Impact of not resolving
    pub impact: String,
    /// Available resolution options
    pub options: Vec<ConflictResolution>,
    /// Index of chosen option (if resolved)
    pub chosen: Option<i32>,
}

impl Default for Conflict {
    fn default() -> Self {
        Self {
            id: String::new(),
            between: (String::new(), String::new()),
            description: String::new(),
            impact: String::new(),
            options: Vec::new(),
            chosen: None,
        }
    }
}

/// Resolution option for a conflict
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// Option identifier
    pub option: String,
    /// Description of this resolution
    pub description: String,
    /// Tradeoffs of choosing this option
    pub tradeoffs: String,
    /// Whether this is recommended
    pub recommendation: bool,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        Self {
            option: String::new(),
            description: String::new(),
            tradeoffs: String::new(),
            recommendation: false,
        }
    }
}

/// Question definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    /// Unique question identifier
    pub id: String,
    /// Round this question appears in
    pub round: u32,
    /// Perspective for this question
    pub perspective: Perspective,
    /// Question category
    pub category: QuestionCategory,
    /// Priority level
    pub priority: QuestionPriority,
    /// The question text
    pub question: String,
    /// Context/help text
    pub context: String,
    /// Example answer
    pub example: String,
    /// Expected type of answer
    pub expected_type: String,
    /// Fields to extract from answer
    pub extract_into: Vec<String>,
    /// Question IDs this depends on
    pub depends_on: Vec<String>,
    /// Question IDs this blocks
    pub blocks: Vec<String>,
}

impl Default for Question {
    fn default() -> Self {
        Self {
            id: String::new(),
            round: 1,
            perspective: Perspective::default(),
            category: QuestionCategory::default(),
            priority: QuestionPriority::default(),
            question: String::new(),
            context: String::new(),
            example: String::new(),
            expected_type: String::new(),
            extract_into: Vec::new(),
            depends_on: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

/// Interview session - persistent state machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterviewSession {
    /// Unique session identifier
    pub id: String,
    /// Profile type for this interview
    pub profile: Profile,
    /// When session was created
    pub created_at: String,
    /// When session was last updated
    pub updated_at: String,
    /// When session was completed (if complete)
    pub completed_at: Option<String>,
    /// Current stage in interview lifecycle
    pub stage: InterviewStage,
    /// Number of completed rounds
    pub rounds_completed: u32,
    /// Collected answers
    pub answers: Vec<Answer>,
    /// Detected gaps
    pub gaps: Vec<Gap>,
    /// Detected conflicts
    pub conflicts: Vec<Conflict>,
    /// Raw notes text
    pub raw_notes: String,
    /// Current phase number
    pub current_phase: u32,
    /// Completed phase numbers
    pub completed_phases: Vec<u32>,
}

impl Default for InterviewSession {
    fn default() -> Self {
        Self {
            id: String::new(),
            profile: Profile::default(),
            created_at: String::new(),
            updated_at: String::new(),
            completed_at: None,
            stage: InterviewStage::default(),
            rounds_completed: 0,
            answers: Vec::new(),
            gaps: Vec::new(),
            conflicts: Vec::new(),
            raw_notes: String::new(),
            current_phase: 1,
            completed_phases: Vec::new(),
        }
    }
}

impl InterviewSession {
    /// Create a new interview session
    ///
    /// # Arguments
    /// * `id` - Unique session identifier
    /// * `profile` - Profile type for this interview
    /// * `timestamp` - Creation timestamp (ISO 8601)
    #[must_use]
    pub fn new(id: String, profile: Profile, timestamp: String) -> Self {
        Self {
            id,
            profile,
            created_at: timestamp.clone(),
            updated_at: timestamp,
            stage: InterviewStage::Discovery,
            ..Self::default()
        }
    }

    /// Get current round number
    #[must_use]
    pub const fn get_current_round(&self) -> u32 {
        self.rounds_completed + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_roundtrip() {
        let profiles = [
            Profile::Api,
            Profile::Cli,
            Profile::Event,
            Profile::Data,
            Profile::Workflow,
            Profile::Ui,
        ];

        for profile in profiles {
            let s = profile.as_str();
            let parsed = Profile::from_str(s);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap(), profile);
        }
    }

    #[test]
    fn test_profile_from_str_error() {
        let result = Profile::from_str("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_interview_session_new() {
        let session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        assert_eq!(session.id, "test-session");
        assert_eq!(session.profile, Profile::Api);
        assert_eq!(session.stage, InterviewStage::Discovery);
        assert_eq!(session.get_current_round(), 1);
        assert!(session.answers.is_empty());
        assert!(session.gaps.is_empty());
        assert!(session.conflicts.is_empty());
    }

    #[test]
    fn test_serde_roundtrip() {
        let session = InterviewSession::new(
            "serde-test".to_string(),
            Profile::Cli,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let json = serde_json::to_string(&session).expect("Failed to serialize");
        let parsed: InterviewSession =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(session, parsed);
    }

    #[test]
    fn test_question_priority_snake_case_serialization() {
        // Ensure QuestionPriority serializes with snake_case to match schema
        assert_eq!(
            serde_json::to_string(&QuestionPriority::Critical).unwrap(),
            "\"critical\""
        );
        assert_eq!(
            serde_json::to_string(&QuestionPriority::Important).unwrap(),
            "\"important\""
        );
        assert_eq!(
            serde_json::to_string(&QuestionPriority::NiceToHave).unwrap(),
            "\"nice_to_have\""
        );

        // Test roundtrip
        let parsed: QuestionPriority = serde_json::from_str("\"nice_to_have\"").unwrap();
        assert_eq!(parsed, QuestionPriority::NiceToHave);
    }

    #[test]
    fn test_question_category_snake_case_serialization() {
        // Ensure QuestionCategory serializes with snake_case to match schema
        assert_eq!(
            serde_json::to_string(&QuestionCategory::HappyPath).unwrap(),
            "\"happy_path\""
        );
        assert_eq!(
            serde_json::to_string(&QuestionCategory::ErrorCase).unwrap(),
            "\"error_case\""
        );
        assert_eq!(
            serde_json::to_string(&QuestionCategory::EdgeCase).unwrap(),
            "\"edge_case\""
        );
        assert_eq!(
            serde_json::to_string(&QuestionCategory::NonFunctional).unwrap(),
            "\"non_functional\""
        );

        // Test roundtrip
        let parsed: QuestionCategory = serde_json::from_str("\"happy_path\"").unwrap();
        assert_eq!(parsed, QuestionCategory::HappyPath);
    }
}
