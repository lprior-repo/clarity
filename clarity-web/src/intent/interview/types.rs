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
use thiserror::Error;

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

/// Interview-related errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterviewError {
    /// Gap ID is empty
    #[error("gap ID is empty")]
    EmptyGapId,
    /// Resolution text is empty
    #[error("resolution text is empty")]
    EmptyResolution,
    /// Gap not found
    #[error("gap not found: {0}")]
    GapNotFound(String),
}

/// Errors that can occur during conflict detection and resolution
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConflictDetectionError {
    #[error("session ID is empty")]
    EmptySessionId,
    #[error("conflict not found: {0}")]
    ConflictNotFound(String),
    #[error("conflict already resolved: {0}")]
    ConflictAlreadyResolved(String),
    #[error("invalid option index {index} for conflict {conflict_id} (has {option_count} options)")]
    InvalidOptionIndex {
        conflict_id: String,
        index: i32,
        option_count: usize,
    },
    #[error("option index cannot be negative: {0}")]
    NegativeOptionIndex(i32),
    #[error("answer has empty question_id at index {0}")]
    EmptyQuestionId(usize),
}

/// Errors that can occur during interview session operations
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum InterviewSessionError {
    /// Answer round does not match current session round
    #[error("answer round {answer_round} does not match current round {current_round}")]
    RoundMismatch { answer_round: u32, current_round: u32 },
    /// Session is in a state that doesn't allow modifications
    #[error("cannot modify session in {stage:?} state")]
    SessionNotModifiable { stage: InterviewStage },
    /// Session is paused and must be resumed first
    #[error("session is paused; call resume before modifying")]
    SessionPaused,
    /// Phase number is invalid (must be >= 1)
    #[error("invalid phase number: {phase_number}; phase must be >= 1")]
    InvalidPhaseNumber { phase_number: u32 },
    /// Answer has an empty question_id
    #[error("answer has empty question_id")]
    EmptyQuestionId,
    /// Timestamp is empty
    #[error("timestamp cannot be empty")]
    EmptyTimestamp,
    /// Duplicate answer for the same question in the same round
    #[error("duplicate answer for question '{question_id}' in round {round}")]
    DuplicateAnswer { question_id: String, round: u32 },
    /// Blocking gaps remain unresolved
    #[error("cannot proceed: {count} blocking gap(s) unresolved")]
    BlockingGapsUnresolved { count: usize, gap_ids: Vec<String> },
    /// Session is already complete
    #[error("session already complete")]
    AlreadyComplete,
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

    /// Get required fields for this profile type
    #[must_use]
    pub const fn required_fields(&self) -> &'static [&'static str] {
        match self {
            Self::Api => &["base_url", "auth_method", "happy_path", "error_cases", "response_format"],
            Self::Cli => &["command_name", "happy_path", "help_text", "exit_codes"],
            Self::Event => &["event_type", "payload_schema", "trigger"],
            Self::Data => &["data_model", "access_patterns", "retention"],
            Self::Workflow => &["steps", "happy_path", "error_recovery"],
            Self::Ui => &["user_flows", "happy_path", "states"],
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
#[serde(rename_all = "lowercase")]
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
#[serde(rename_all = "lowercase")]
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

    /// Detect gaps in required fields for the current profile.
    ///
    /// This is a pure function that compares required fields against
    /// extracted fields from all answers to identify missing information.
    #[must_use]
    pub fn detect_gaps(&self) -> Vec<Gap> {
        use std::collections::HashSet;

        let required = self.profile.required_fields();
        let answered_fields: HashSet<&str> = self
            .answers
            .iter()
            .flat_map(|answer| answer.extracted.keys())
            .map(String::as_str)
            .collect();

        required
            .iter()
            .filter(|field| !answered_fields.contains(*field))
            .map(|&field| Gap {
                id: format!("gap-{field}"),
                field: field.to_string(),
                description: format!("Missing required field: {field}"),
                blocking: true,
                suggested_default: String::new(),
                why_needed: String::new(),
                round: self.get_current_round(),
                resolved: false,
                resolution: String::new(),
            })
            .collect()
    }

    /// Get all blocking gaps that are not yet resolved.
    ///
    /// Returns references to gaps for zero-copy access.
    #[must_use]
    pub fn get_blocking_gaps(&self) -> Vec<&Gap> {
        self.gaps
            .iter()
            .filter(|gap| gap.blocking && !gap.resolved)
            .collect()
    }

    /// Resolve a gap by ID with the provided resolution text.
    ///
    /// # Errors
    /// - `InterviewError::EmptyGapId` if gap_id is empty or whitespace
    /// - `InterviewError::EmptyResolution` if resolution is empty or whitespace
    /// - `InterviewError::GapNotFound` if no gap matches the ID
    pub fn resolve_gap(
        &mut self,
        gap_id: &str,
        resolution: &str,
    ) -> Result<(), InterviewError> {
        if gap_id.trim().is_empty() {
            return Err(InterviewError::EmptyGapId);
        }

        if resolution.trim().is_empty() {
            return Err(InterviewError::EmptyResolution);
        }

        let gap = self
            .gaps
            .iter_mut()
            .find(|gap| gap.id == gap_id)
            .ok_or_else(|| InterviewError::GapNotFound(gap_id.to_string()))?;

        gap.resolved = true;
        gap.resolution = resolution.to_string();
        // Update timestamp using current Unix timestamp
        self.updated_at = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        );

        Ok(())
    }

    /// Add an answer to the current round.
    ///
    /// # Errors
    /// - `InterviewSessionError::SessionPaused` if session is paused
    /// - `InterviewSessionError::AlreadyComplete` if session is complete
    /// - `InterviewSessionError::EmptyQuestionId` if answer.question_id is empty
    /// - `InterviewSessionError::EmptyTimestamp` if timestamp is empty
    /// - `InterviewSessionError::RoundMismatch` if answer.round does not match current round
    /// - `InterviewSessionError::DuplicateAnswer` if question_id already answered in current round
    pub fn add_answer(
        &mut self,
        answer: Answer,
        timestamp: &str,
    ) -> Result<(), InterviewSessionError> {
        // Check session state
        if self.stage == InterviewStage::Paused {
            return Err(InterviewSessionError::SessionPaused);
        }
        if self.stage == InterviewStage::Complete {
            return Err(InterviewSessionError::AlreadyComplete);
        }

        // Validate answer
        if answer.question_id.is_empty() {
            return Err(InterviewSessionError::EmptyQuestionId);
        }
        if timestamp.is_empty() {
            return Err(InterviewSessionError::EmptyTimestamp);
        }

        let current_round = self.get_current_round();
        if answer.round != current_round {
            return Err(InterviewSessionError::RoundMismatch {
                answer_round: answer.round,
                current_round,
            });
        }

        // Check for duplicate in current round
        let is_duplicate = self.answers.iter().any(|a| {
            a.question_id == answer.question_id && a.round == current_round
        });
        if is_duplicate {
            return Err(InterviewSessionError::DuplicateAnswer {
                question_id: answer.question_id,
                round: current_round,
            });
        }

        self.answers.push(answer);
        self.updated_at = timestamp.to_string();
        Ok(())
    }

    /// Complete the current round and advance stage if needed.
    ///
    /// Stage transitions:
    /// - Rounds 1-2: stay in Discovery
    /// - Round 3: Discovery -> Refinement
    /// - Round 4: Refinement -> Validation
    /// - Round 5+: Validation -> Complete
    ///
    /// # Errors
    /// - `InterviewSessionError::SessionPaused` if session is paused
    /// - `InterviewSessionError::AlreadyComplete` if session is complete
    /// - `InterviewSessionError::EmptyTimestamp` if timestamp is empty
    pub fn complete_round(&mut self, timestamp: &str) -> Result<(), InterviewSessionError> {
        // Check session state
        if self.stage == InterviewStage::Paused {
            return Err(InterviewSessionError::SessionPaused);
        }
        if self.stage == InterviewStage::Complete {
            return Err(InterviewSessionError::AlreadyComplete);
        }

        if timestamp.is_empty() {
            return Err(InterviewSessionError::EmptyTimestamp);
        }

        self.rounds_completed += 1;

        // Apply stage transitions based on completed rounds
        self.stage = match (self.rounds_completed, self.stage) {
            (1 | 2, _) => InterviewStage::Discovery,
            (3, _) => InterviewStage::Refinement,
            (4, _) => InterviewStage::Validation,
            (_, _) => InterviewStage::Complete,
        };

        self.updated_at = timestamp.to_string();

        if self.stage == InterviewStage::Complete {
            self.completed_at = Some(timestamp.to_string());
        }

        Ok(())
    }

    /// Check if the session can proceed (no blocking unresolved gaps).
    ///
    /// # Errors
    /// Returns `InterviewSessionError::BlockingGapsUnresolved` if any blocking
    /// gaps remain unresolved, listing their IDs.
    pub fn can_proceed(&self) -> Result<(), InterviewSessionError> {
        let blocking_gaps: Vec<&Gap> = self.get_blocking_gaps();

        if blocking_gaps.is_empty() {
            Ok(())
        } else {
            let gap_ids: Vec<String> = blocking_gaps
                .iter()
                .map(|gap| gap.id.clone())
                .collect();
            Err(InterviewSessionError::BlockingGapsUnresolved {
                count: gap_ids.len(),
                gap_ids,
            })
        }
    }

    /// Complete a phase by number.
    ///
    /// # Errors
    /// - `InterviewSessionError::InvalidPhaseNumber` if phase_number is 0
    /// - `InterviewSessionError::EmptyTimestamp` if timestamp is empty
    pub fn complete_phase(
        &mut self,
        phase_number: u32,
        timestamp: &str,
    ) -> Result<(), InterviewSessionError> {
        if phase_number < 1 {
            return Err(InterviewSessionError::InvalidPhaseNumber { phase_number });
        }

        if timestamp.is_empty() {
            return Err(InterviewSessionError::EmptyTimestamp);
        }

        // Add to completed_phases if not already present
        if !self.completed_phases.contains(&phase_number) {
            self.completed_phases.push(phase_number);
        }

        // Update current_phase if completing current phase
        if self.current_phase == phase_number {
            self.current_phase = phase_number + 1;
        }

        self.updated_at = timestamp.to_string();
        Ok(())
    }

    /// Detect conflicts between answers in the session
    ///
    /// Checks for various types of conflicts:
    /// - CAP theorem: fast/latency vs consistent/accurate
    /// - Anonymous + Audit: anonymous vs audit/log
    ///
    /// # Errors
    /// Returns `ConflictDetectionError` if:
    /// - Session ID is empty
    /// - Any answer has an empty question_id
    ///
    /// # Returns
    /// A vector of newly detected conflicts (empty if none found)
    pub fn detect_conflicts(&mut self) -> Result<Vec<Conflict>, ConflictDetectionError> {
        // Validate session ID is non-empty
        if self.id.is_empty() {
            return Err(ConflictDetectionError::EmptySessionId);
        }

        // Validate all answers have non-empty question_id
        for (index, answer) in self.answers.iter().enumerate() {
            if answer.question_id.is_empty() {
                return Err(ConflictDetectionError::EmptyQuestionId(index));
            }
        }

        let mut new_conflicts = Vec::new();

        // Detect CAP theorem conflict
        if let Some(conflict) = detect_cap_conflict(&self.answers) {
            new_conflicts.push(conflict);
        }

        // Detect Anonymous + Audit conflict
        if let Some(conflict) = detect_anonymous_audit_conflict(&self.answers) {
            new_conflicts.push(conflict);
        }

        // Update timestamp and append conflicts
        if !new_conflicts.is_empty() {
            self.updated_at = format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs())
            );
            self.conflicts.extend(new_conflicts.clone());
        }

        Ok(new_conflicts)
    }

    /// Resolve a detected conflict by choosing an option
    ///
    /// # Arguments
    /// * `conflict_id` - The ID of the conflict to resolve
    /// * `chosen_option` - The index of the chosen resolution option
    ///
    /// # Errors
    /// Returns `ConflictDetectionError` if:
    /// - Conflict ID is empty
    /// - Chosen option index is negative
    /// - Conflict not found
    /// - Conflict already resolved
    /// - Chosen option index out of bounds
    pub fn resolve_conflict(
        &mut self,
        conflict_id: &str,
        chosen_option: i32,
    ) -> Result<(), ConflictDetectionError> {
        // Validate conflict_id is non-empty
        if conflict_id.is_empty() {
            return Err(ConflictDetectionError::ConflictNotFound(
                conflict_id.to_string(),
            ));
        }

        // Validate chosen_option is non-negative
        if chosen_option < 0 {
            return Err(ConflictDetectionError::NegativeOptionIndex(
                chosen_option,
            ));
        }

        // Find the conflict
        let conflict = self
            .conflicts
            .iter_mut()
            .find(|c| c.id == conflict_id)
            .ok_or_else(|| ConflictDetectionError::ConflictNotFound(conflict_id.to_string()))?;

        // Check if already resolved
        if conflict.chosen.is_some() {
            return Err(ConflictDetectionError::ConflictAlreadyResolved(
                conflict_id.to_string(),
            ));
        }

        // Validate chosen_option is within bounds
        let option_count = conflict.options.len();
        if (chosen_option as usize) >= option_count {
            return Err(ConflictDetectionError::InvalidOptionIndex {
                conflict_id: conflict_id.to_string(),
                index: chosen_option,
                option_count,
            });
        }

        // Resolve the conflict
        conflict.chosen = Some(chosen_option);
        self.updated_at = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        );

        Ok(())
    }
}

/// Check if text contains any of the given keywords (case-insensitive)
fn contains_keywords(text: &str, keywords: &[&str]) -> bool {
    let lower = text.to_lowercase();
    keywords.iter().any(|k| lower.contains(&k.to_lowercase()))
}

/// Detect CAP theorem conflict (fast/latency vs consistent/accurate)
fn detect_cap_conflict(answers: &[Answer]) -> Option<Conflict> {
    let fast_keywords = ["fast", "latency", "speed", "quick", "low-latency"];
    let consistent_keywords = ["consistent", "accurate", "correct", "reliable", "precise"];

    // Find answers mentioning fast/latency
    let fast_answer = answers
        .iter()
        .find(|a| contains_keywords(&a.response, &fast_keywords))?;

    // Find answers mentioning consistent/accurate
    let consistent_answer = answers
        .iter()
        .find(|a| {
            contains_keywords(&a.response, &consistent_keywords)
                && a.question_id != fast_answer.question_id
        })?;

    Some(Conflict {
        id: "conflict-cap-0".to_string(),
        between: (fast_answer.question_id.clone(), consistent_answer.question_id.clone()),
        description: "CAP theorem conflict: The system cannot simultaneously guarantee low latency and strong consistency. You've indicated requirements for both speed and data accuracy.".to_string(),
        impact: "Without resolution, the system may fail to meet performance expectations or data integrity requirements under load.".to_string(),
        options: vec![
            ConflictResolution {
                option: "prioritize-speed".to_string(),
                description: "Optimize for low latency with eventual consistency".to_string(),
                tradeoffs: "Faster responses but data may be temporarily stale".to_string(),
                recommendation: false,
            },
            ConflictResolution {
                option: "prioritize-consistency".to_string(),
                description: "Optimize for strong consistency with higher latency".to_string(),
                tradeoffs: "Always accurate data but slower response times".to_string(),
                recommendation: true,
            },
        ],
        chosen: None,
    })
}

/// Detect Anonymous + Audit conflict
fn detect_anonymous_audit_conflict(answers: &[Answer]) -> Option<Conflict> {
    let anonymous_keywords = ["anonymous", "anonymized", "privacy", "no-tracking", "private"];
    let audit_keywords = ["audit", "log", "track", "trail", "accountability"];

    // Find answers mentioning anonymous
    let anonymous_answer = answers
        .iter()
        .find(|a| contains_keywords(&a.response, &anonymous_keywords))?;

    // Find answers mentioning audit/log
    let audit_answer = answers
        .iter()
        .find(|a| {
            contains_keywords(&a.response, &audit_keywords)
                && a.question_id != anonymous_answer.question_id
        })?;

    Some(Conflict {
        id: "conflict-anonymous-audit-0".to_string(),
        between: (anonymous_answer.question_id.clone(), audit_answer.question_id.clone()),
        description: "Privacy vs Accountability conflict: You've indicated requirements for user anonymity while also requiring audit trails. These requirements are fundamentally at odds.".to_string(),
        impact: "Without resolution, the system cannot provide both complete user privacy and comprehensive audit trails.".to_string(),
        options: vec![
            ConflictResolution {
                option: "prioritize-privacy".to_string(),
                description: "Remove detailed audit logging to protect user privacy".to_string(),
                tradeoffs: "Reduced accountability and harder incident investigation".to_string(),
                recommendation: false,
            },
            ConflictResolution {
                option: "pseudonymous-audit".to_string(),
                description: "Use pseudonymous identifiers in audit logs instead of real identities".to_string(),
                tradeoffs: "Partial privacy with some accountability; may not satisfy strict anonymity requirements".to_string(),
                recommendation: true,
            },
        ],
        chosen: None,
    })
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
    fn test_detect_conflicts_empty_session_id() {
        let mut session = InterviewSession::default();
        let result = session.detect_conflicts();
        assert!(matches!(result, Err(ConflictDetectionError::EmptySessionId)));
    }

    #[test]
    fn test_detect_conflicts_empty_question_id() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.answers.push(Answer {
            question_id: String::new(),
            question_text: "Q1".to_string(),
            response: "fast response".to_string(),
            ..Answer::default()
        });
        let result = session.detect_conflicts();
        assert!(matches!(result, Err(ConflictDetectionError::EmptyQuestionId(0))));
    }

    #[test]
    fn test_detect_conflicts_no_conflicts() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.answers.push(Answer {
            question_id: "q1".to_string(),
            question_text: "Q1".to_string(),
            response: "simple response".to_string(),
            ..Answer::default()
        });
        let result = session.detect_conflicts();
        assert!(result.is_ok());
        let conflicts = result.expect("should have conflicts");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_conflicts_cap_theorem() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.answers.push(Answer {
            question_id: "q-perf".to_string(),
            question_text: "Performance requirements".to_string(),
            response: "We need fast response times with low latency".to_string(),
            ..Answer::default()
        });
        session.answers.push(Answer {
            question_id: "q-data".to_string(),
            question_text: "Data requirements".to_string(),
            response: "Data must be consistent and accurate at all times".to_string(),
            ..Answer::default()
        });

        let result = session.detect_conflicts();
        assert!(result.is_ok());
        let conflicts = result.expect("should have conflicts");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].id, "conflict-cap-0");
        assert_eq!(conflicts[0].between, ("q-perf".to_string(), "q-data".to_string()));
        assert_eq!(conflicts[0].options.len(), 2);
        assert!(conflicts[0].chosen.is_none());
    }

    #[test]
    fn test_detect_conflicts_anonymous_audit() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.answers.push(Answer {
            question_id: "q-privacy".to_string(),
            question_text: "Privacy requirements".to_string(),
            response: "Users must remain anonymous".to_string(),
            ..Answer::default()
        });
        session.answers.push(Answer {
            question_id: "q-audit".to_string(),
            question_text: "Audit requirements".to_string(),
            response: "We need full audit trail for compliance".to_string(),
            ..Answer::default()
        });

        let result = session.detect_conflicts();
        assert!(result.is_ok());
        let conflicts = result.expect("should have conflicts");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].id, "conflict-anonymous-audit-0");
        assert_eq!(conflicts[0].between, ("q-privacy".to_string(), "q-audit".to_string()));
        assert_eq!(conflicts[0].options.len(), 2);
        assert!(conflicts[0].chosen.is_none());
    }

    #[test]
    fn test_resolve_conflict_not_found() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        let result = session.resolve_conflict("nonexistent", 0);
        assert!(matches!(result, Err(ConflictDetectionError::ConflictNotFound(_))));
    }

    #[test]
    fn test_resolve_conflict_empty_id() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        let result = session.resolve_conflict("", 0);
        assert!(matches!(result, Err(ConflictDetectionError::ConflictNotFound(_))));
    }

    #[test]
    fn test_resolve_conflict_negative_index() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.conflicts.push(Conflict {
            id: "conflict-1".to_string(),
            between: ("a".to_string(), "b".to_string()),
            description: "test".to_string(),
            impact: "test".to_string(),
            options: vec![
                ConflictResolution {
                    option: "opt1".to_string(),
                    description: "option 1".to_string(),
                    tradeoffs: "tradeoffs".to_string(),
                    recommendation: false,
                },
            ],
            chosen: None,
        });
        let result = session.resolve_conflict("conflict-1", -1);
        assert!(matches!(result, Err(ConflictDetectionError::NegativeOptionIndex(-1))));
    }

    #[test]
    fn test_resolve_conflict_invalid_index() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.conflicts.push(Conflict {
            id: "conflict-1".to_string(),
            between: ("a".to_string(), "b".to_string()),
            description: "test".to_string(),
            impact: "test".to_string(),
            options: vec![
                ConflictResolution {
                    option: "opt1".to_string(),
                    description: "option 1".to_string(),
                    tradeoffs: "tradeoffs".to_string(),
                    recommendation: false,
                },
            ],
            chosen: None,
        });
        let result = session.resolve_conflict("conflict-1", 5);
        assert!(matches!(result, Err(ConflictDetectionError::InvalidOptionIndex { .. })));
    }

    #[test]
    fn test_resolve_conflict_already_resolved() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.conflicts.push(Conflict {
            id: "conflict-1".to_string(),
            between: ("a".to_string(), "b".to_string()),
            description: "test".to_string(),
            impact: "test".to_string(),
            options: vec![
                ConflictResolution {
                    option: "opt1".to_string(),
                    description: "option 1".to_string(),
                    tradeoffs: "tradeoffs".to_string(),
                    recommendation: false,
                },
            ],
            chosen: Some(0),
        });
        let result = session.resolve_conflict("conflict-1", 0);
        assert!(matches!(result, Err(ConflictDetectionError::ConflictAlreadyResolved(_))));
    }

    #[test]
    fn test_resolve_conflict_success() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.conflicts.push(Conflict {
            id: "conflict-1".to_string(),
            between: ("a".to_string(), "b".to_string()),
            description: "test".to_string(),
            impact: "test".to_string(),
            options: vec![
                ConflictResolution {
                    option: "opt1".to_string(),
                    description: "option 1".to_string(),
                    tradeoffs: "tradeoffs".to_string(),
                    recommendation: false,
                },
                ConflictResolution {
                    option: "opt2".to_string(),
                    description: "option 2".to_string(),
                    tradeoffs: "tradeoffs".to_string(),
                    recommendation: true,
                },
            ],
            chosen: None,
        });

        let result = session.resolve_conflict("conflict-1", 1);
        assert!(result.is_ok());
        assert_eq!(session.conflicts[0].chosen, Some(1));
    }

    #[test]
    fn test_contains_keywords() {
        assert!(contains_keywords("This is FAST", &["fast"]));
        assert!(contains_keywords("Low Latency System", &["latency"]));
        assert!(contains_keywords("Consistent Data", &["CONSISTENT"]));
        assert!(!contains_keywords("No match here", &["fast", "latency"]));
    }

    // Gap Detection Tests (WP13)

    #[test]
    fn test_detect_gaps_no_answers() {
        let session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let gaps = session.detect_gaps();

        // API profile has 5 required fields
        assert_eq!(gaps.len(), 5);
        assert!(gaps.iter().all(|g| g.blocking));
        assert!(gaps.iter().all(|g| !g.resolved));
        assert!(gaps.iter().any(|g| g.field == "base_url"));
        assert!(gaps.iter().any(|g| g.field == "auth_method"));
    }

    #[test]
    fn test_detect_gaps_partial_answers() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        // Add an answer with some extracted fields
        let mut extracted = HashMap::new();
        extracted.insert("base_url".to_string(), "https://api.example.com".to_string());
        extracted.insert("auth_method".to_string(), "Bearer".to_string());

        session.answers.push(Answer {
            question_id: "q1".to_string(),
            question_text: "What is the base URL?".to_string(),
            perspective: Perspective::Developer,
            round: 1,
            response: "The base URL is https://api.example.com with Bearer auth".to_string(),
            extracted,
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:00:00Z".to_string(),
        });

        let gaps = session.detect_gaps();

        // 3 fields still missing: happy_path, error_cases, response_format
        assert_eq!(gaps.len(), 3);
        assert!(gaps.iter().all(|g| g.field != "base_url"));
        assert!(gaps.iter().all(|g| g.field != "auth_method"));
    }

    #[test]
    fn test_detect_gaps_all_answered() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Cli,
            "2026-02-27T00:00:00Z".to_string(),
        );

        // Add answer with all required CLI fields
        let mut extracted = HashMap::new();
        extracted.insert("command_name".to_string(), "mycli".to_string());
        extracted.insert("happy_path".to_string(), "runs successfully".to_string());
        extracted.insert("help_text".to_string(), "Displays help".to_string());
        extracted.insert("exit_codes".to_string(), "0=success,1=error".to_string());

        session.answers.push(Answer {
            question_id: "q1".to_string(),
            question_text: "Describe the CLI".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "Complete CLI description".to_string(),
            extracted,
            confidence: 1.0,
            notes: String::new(),
            timestamp: "2026-02-27T00:00:00Z".to_string(),
        });

        let gaps = session.detect_gaps();

        assert!(gaps.is_empty());
    }

    #[test]
    fn test_get_blocking_gaps_empty() {
        let session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let blocking = session.get_blocking_gaps();

        assert!(blocking.is_empty());
    }

    #[test]
    fn test_get_blocking_gaps_filters_resolved() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        session.gaps = vec![
            Gap {
                id: "gap-1".to_string(),
                field: "field1".to_string(),
                blocking: true,
                resolved: false,
                ..Gap::default()
            },
            Gap {
                id: "gap-2".to_string(),
                field: "field2".to_string(),
                blocking: true,
                resolved: true,
                ..Gap::default()
            },
            Gap {
                id: "gap-3".to_string(),
                field: "field3".to_string(),
                blocking: false,
                resolved: false,
                ..Gap::default()
            },
        ];

        let blocking = session.get_blocking_gaps();

        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0].id, "gap-1");
    }

    #[test]
    fn test_resolve_gap_success() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        session.gaps.push(Gap {
            id: "gap-base_url".to_string(),
            field: "base_url".to_string(),
            blocking: true,
            resolved: false,
            resolution: String::new(),
            ..Gap::default()
        });

        let result = session.resolve_gap("gap-base_url", "https://api.example.com");

        assert!(result.is_ok());
        assert!(session.gaps[0].resolved);
        assert_eq!(session.gaps[0].resolution, "https://api.example.com");
    }

    #[test]
    fn test_resolve_gap_empty_id() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let result = session.resolve_gap("", "resolution");

        assert_eq!(result, Err(InterviewError::EmptyGapId));
    }

    #[test]
    fn test_resolve_gap_whitespace_id() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let result = session.resolve_gap("   ", "resolution");

        assert_eq!(result, Err(InterviewError::EmptyGapId));
    }

    #[test]
    fn test_resolve_gap_empty_resolution() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        session.gaps.push(Gap {
            id: "gap-1".to_string(),
            field: "field1".to_string(),
            ..Gap::default()
        });

        let result = session.resolve_gap("gap-1", "");

        assert_eq!(result, Err(InterviewError::EmptyResolution));
    }

    #[test]
    fn test_resolve_gap_not_found() {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let result = session.resolve_gap("nonexistent", "some resolution");

        assert_eq!(result, Err(InterviewError::GapNotFound("nonexistent".to_string())));
    }

    #[test]
    fn test_required_fields_for_all_profiles() {
        assert_eq!(Profile::Api.required_fields().len(), 5);
        assert_eq!(Profile::Cli.required_fields().len(), 4);
        assert_eq!(Profile::Event.required_fields().len(), 3);
        assert_eq!(Profile::Data.required_fields().len(), 3);
        assert_eq!(Profile::Workflow.required_fields().len(), 3);
        assert_eq!(Profile::Ui.required_fields().len(), 3);
    }

    // ==================== add_answer tests (WP12) ====================

    #[test]
    fn test_add_answer_success() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let answer = Answer {
            question_id: "q1".to_string(),
            question_text: "What is the API?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "REST API".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:01:00Z".to_string(),
        };

        let result = session.add_answer(answer, "2026-02-27T00:01:00Z");
        assert!(result.is_ok());
        assert_eq!(session.answers.len(), 1);
        assert_eq!(session.updated_at, "2026-02-27T00:01:00Z");
    }

    #[test]
    fn test_add_answer_empty_question_id() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let answer = Answer {
            question_id: String::new(),
            question_text: "What?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "Answer".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:01:00Z".to_string(),
        };

        let result = session.add_answer(answer, "2026-02-27T00:01:00Z");
        assert!(matches!(result, Err(InterviewSessionError::EmptyQuestionId)));
    }

    #[test]
    fn test_add_answer_empty_timestamp() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let answer = Answer {
            question_id: "q1".to_string(),
            question_text: "What?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "Answer".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:01:00Z".to_string(),
        };

        let result = session.add_answer(answer, "");
        assert!(matches!(result, Err(InterviewSessionError::EmptyTimestamp)));
    }

    #[test]
    fn test_add_answer_round_mismatch() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let answer = Answer {
            question_id: "q1".to_string(),
            question_text: "What?".to_string(),
            perspective: Perspective::User,
            round: 5, // Wrong round
            response: "Answer".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:01:00Z".to_string(),
        };

        let result = session.add_answer(answer, "2026-02-27T00:01:00Z");
        assert!(matches!(
            result,
            Err(InterviewSessionError::RoundMismatch {
                answer_round: 5,
                current_round: 1
            })
        ));
    }

    #[test]
    fn test_add_answer_duplicate() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let answer1 = Answer {
            question_id: "q1".to_string(),
            question_text: "What?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "First".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:01:00Z".to_string(),
        };

        let result = session.add_answer(answer1, "2026-02-27T00:01:00Z");
        assert!(result.is_ok());

        let answer2 = Answer {
            question_id: "q1".to_string(), // Same question_id
            question_text: "What?".to_string(),
            perspective: Perspective::User,
            round: 1, // Same round
            response: "Second".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:02:00Z".to_string(),
        };

        let result = session.add_answer(answer2, "2026-02-27T00:02:00Z");
        assert!(matches!(
            result,
            Err(InterviewSessionError::DuplicateAnswer {
                question_id,
                round: 1
            }) if question_id == "q1"
        ));
    }

    #[test]
    fn test_add_answer_paused_session() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.stage = InterviewStage::Paused;

        let answer = Answer {
            question_id: "q1".to_string(),
            question_text: "What?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "Answer".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:01:00Z".to_string(),
        };

        let result = session.add_answer(answer, "2026-02-27T00:01:00Z");
        assert!(matches!(result, Err(InterviewSessionError::SessionPaused)));
    }

    #[test]
    fn test_add_answer_complete_session() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.stage = InterviewStage::Complete;

        let answer = Answer {
            question_id: "q1".to_string(),
            question_text: "What?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "Answer".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:01:00Z".to_string(),
        };

        let result = session.add_answer(answer, "2026-02-27T00:01:00Z");
        assert!(matches!(result, Err(InterviewSessionError::AlreadyComplete)));
    }

    // ==================== complete_round tests (WP12) ====================

    #[test]
    fn test_complete_round_transitions() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        // Round 1: Discovery -> Discovery
        session.complete_round("t1").ok();
        assert_eq!(session.rounds_completed, 1);
        assert_eq!(session.stage, InterviewStage::Discovery);

        // Round 2: Discovery -> Discovery
        session.complete_round("t2").ok();
        assert_eq!(session.rounds_completed, 2);
        assert_eq!(session.stage, InterviewStage::Discovery);

        // Round 3: Discovery -> Refinement
        session.complete_round("t3").ok();
        assert_eq!(session.rounds_completed, 3);
        assert_eq!(session.stage, InterviewStage::Refinement);

        // Round 4: Refinement -> Validation
        session.complete_round("t4").ok();
        assert_eq!(session.rounds_completed, 4);
        assert_eq!(session.stage, InterviewStage::Validation);

        // Round 5: Validation -> Complete
        session.complete_round("t5").ok();
        assert_eq!(session.rounds_completed, 5);
        assert_eq!(session.stage, InterviewStage::Complete);
        assert_eq!(session.completed_at, Some("t5".to_string()));
    }

    #[test]
    fn test_complete_round_paused() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.stage = InterviewStage::Paused;

        let result = session.complete_round("t1");
        assert!(matches!(result, Err(InterviewSessionError::SessionPaused)));
    }

    #[test]
    fn test_complete_round_complete() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        session.stage = InterviewStage::Complete;

        let result = session.complete_round("t1");
        assert!(matches!(result, Err(InterviewSessionError::AlreadyComplete)));
    }

    #[test]
    fn test_complete_round_empty_timestamp() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let result = session.complete_round("");
        assert!(matches!(result, Err(InterviewSessionError::EmptyTimestamp)));
    }

    // ==================== can_proceed tests (WP12) ====================

    #[test]
    fn test_can_proceed_no_gaps() {
        let session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        assert!(session.can_proceed().is_ok());
    }

    #[test]
    fn test_can_proceed_with_blocking_gap() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        session.gaps.push(Gap {
            id: "gap-1".to_string(),
            field: "test".to_string(),
            description: "Missing".to_string(),
            blocking: true,
            resolved: false,
            ..Gap::default()
        });

        let result = session.can_proceed();
        assert!(matches!(
            result,
            Err(InterviewSessionError::BlockingGapsUnresolved {
                count: 1,
                gap_ids
            }) if gap_ids == vec!["gap-1".to_string()]
        ));
    }

    #[test]
    fn test_can_proceed_with_resolved_blocking_gap() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        session.gaps.push(Gap {
            id: "gap-1".to_string(),
            field: "test".to_string(),
            description: "Missing".to_string(),
            blocking: true,
            resolved: true, // Resolved
            ..Gap::default()
        });

        assert!(session.can_proceed().is_ok());
    }

    #[test]
    fn test_can_proceed_with_non_blocking_gap() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        session.gaps.push(Gap {
            id: "gap-1".to_string(),
            field: "test".to_string(),
            description: "Missing".to_string(),
            blocking: false, // Non-blocking
            resolved: false,
            ..Gap::default()
        });

        assert!(session.can_proceed().is_ok());
    }

    // ==================== complete_phase tests (WP12) ====================

    #[test]
    fn test_complete_phase_success() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        assert!(session.current_phase == 1);
        assert!(session.completed_phases.is_empty());

        let result = session.complete_phase(1, "2026-02-27T01:00:00Z");
        assert!(result.is_ok());
        assert!(session.completed_phases.contains(&1));
        assert_eq!(session.current_phase, 2);
    }

    #[test]
    fn test_complete_phase_invalid_zero() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let result = session.complete_phase(0, "t1");
        assert!(matches!(
            result,
            Err(InterviewSessionError::InvalidPhaseNumber { phase_number: 0 })
        ));
    }

    #[test]
    fn test_complete_phase_empty_timestamp() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        let result = session.complete_phase(1, "");
        assert!(matches!(result, Err(InterviewSessionError::EmptyTimestamp)));
    }

    #[test]
    fn test_complete_phase_no_duplicate() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        session.complete_phase(1, "t1").ok();
        assert!(session.completed_phases.len() == 1);

        // Complete same phase again - should not add duplicate
        session.complete_phase(1, "t2").ok();
        assert!(session.completed_phases.len() == 1);
    }

    #[test]
    fn test_complete_phase_out_of_order() {
        let mut session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );

        // Complete phase 3 before current phase (1)
        session.complete_phase(3, "t1").ok();
        assert!(session.completed_phases.contains(&3));
        // current_phase should NOT change since we're not completing current phase
        assert_eq!(session.current_phase, 1);
    }

    // ==================== InterviewSessionError serde test ====================

    #[test]
    fn test_interview_session_error_serde_roundtrip() {
        let errors = vec![
            InterviewSessionError::RoundMismatch {
                answer_round: 2,
                current_round: 1,
            },
            InterviewSessionError::SessionNotModifiable {
                stage: InterviewStage::Complete,
            },
            InterviewSessionError::SessionPaused,
            InterviewSessionError::InvalidPhaseNumber { phase_number: 0 },
            InterviewSessionError::EmptyQuestionId,
            InterviewSessionError::EmptyTimestamp,
            InterviewSessionError::DuplicateAnswer {
                question_id: "q1".to_string(),
                round: 1,
            },
            InterviewSessionError::BlockingGapsUnresolved {
                count: 2,
                gap_ids: vec!["g1".to_string(), "g2".to_string()],
            },
            InterviewSessionError::AlreadyComplete,
        ];

        for error in errors {
            let json = serde_json::to_string(&error).expect("Failed to serialize");
            let parsed: InterviewSessionError =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(error, parsed);
        }
    }
}
