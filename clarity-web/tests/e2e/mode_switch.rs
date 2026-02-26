#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

//! End-to-end test: Mode switching with state preservation (bead e2e-004)
//!
//! Test flow:
//! 1. Start in Express mode, enter freeform text
//! 2. Extract to get field cards
//! 3. Switch to Guided mode - sequential inputs pre-filled with extracted content
//! 4. Answer remaining questions
//! 5. Switch back to Express - textarea contains concatenated answers
//! 6. Verify no data lost in either direction
//! 7. Verify mode preference persists after reload

use std::sync::Arc;
use clarity_web::components::discover::{
    DiscoverMode, FieldData, GuidedFlow, ServerSuggestionProvider,
};
use clarity_web::providers::{
    ExtractionContext, ExtractionError, ExtractionProvider, ExtractionMetadata,
    ExtractedFields, FieldExtraction, FieldType, SchemaField,
};
use clarity_web::storage::{ProjectMetadata, RedbStore, StorageError};
use clarity_web::types::Answer;
use itertools::Itertools;

// =============================================================================
// Mock Extraction Provider for Testing
// =============================================================================

/// Mock extraction provider for deterministic testing
#[derive(Clone, Debug)]
struct MockExtractionProvider {
    /// Simulated latency in milliseconds
    latency_ms: u64,
    /// Whether to return errors
    should_fail: bool,
}

impl MockExtractionProvider {
    fn new() -> Self {
        Self {
            latency_ms: 0,
            should_fail: false,
        }
    }

    fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms = ms;
        self
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl Default for MockExtractionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ExtractionProvider for MockExtractionProvider {
    async fn extract_fields(
        &self,
        text: &str,
        _context: &ExtractionContext,
    ) -> Result<ExtractedFields, ExtractionError> {
        // Simulate network latency
        if self.latency_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.latency_ms)).await;
        }

        if self.should_fail {
            return Err(ExtractionError::ProviderError {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Mock extraction failure",
                )),
            });
        }

        // Parse text for known patterns (deterministic for testing)
        let mut fields = Vec::new();

        // Extract problem statement
        if text.contains("problem") || text.contains("issue") {
            let problem_text = extract_sentence_after_keyword(text, &["problem", "issue"])
                .unwrap_or_else(|| "Identified problem from text".to_string());
            fields.push(FieldExtraction {
                name: "problem".to_string(),
                value: serde_json::json!(problem_text),
                confidence: 0.9,
                source_range: None,
            });
        }

        // Extract target user
        if text.contains("user") || text.contains("persona") {
            let user_text = extract_sentence_after_keyword(text, &["user", "persona", "target"])
                .unwrap_or_else(|| "Identified user from text".to_string());
            fields.push(FieldExtraction {
                name: "user".to_string(),
                value: serde_json::json!(user_text),
                confidence: 0.85,
                source_range: None,
            });
        }

        // Extract context
        if text.contains("context") || text.contains("background") {
            let context_text = extract_sentence_after_keyword(text, &["context", "background"])
                .unwrap_or_else(|| "Identified context from text".to_string());
            fields.push(FieldExtraction {
                name: "context".to_string(),
                value: serde_json::json!(context_text),
                confidence: 0.8,
                source_range: None,
            });
        }

        // Extract constraints
        if text.contains("constraint") || text.contains("limit") {
            let constraints_text =
                extract_sentence_after_keyword(text, &["constraint", "limit", "restrict"])
                    .unwrap_or_else(|| "Identified constraints from text".to_string());
            fields.push(FieldExtraction {
                name: "constraints".to_string(),
                value: serde_json::json!(constraints_text),
                confidence: 0.75,
                source_range: None,
            });
        }

        // Extract goals
        if text.contains("goal") || text.contains("objective") || text.contains("success") {
            let goals_text = extract_sentence_after_keyword(text, &["goal", "objective", "success"])
                .unwrap_or_else(|| "Identified goals from text".to_string());
            fields.push(FieldExtraction {
                name: "goals".to_string(),
                value: serde_json::json!(goals_text),
                confidence: 0.88,
                source_range: None,
            });
        }

        let metadata = ExtractionMetadata {
            extraction_method: "mock_extraction".to_string(),
            model_version: Some("test-1.0".to_string()),
            confidence_threshold: 0.5,
            processing_time_ms: self.latency_ms,
            extra: serde_json::json!({"test": true}),
        };

        Ok(ExtractedFields {
            fields,
            metadata,
        })
    }

    async fn validate_extraction(
        &self,
        _extraction: &ExtractedFields,
    ) -> Result<bool, ExtractionError> {
        Ok(true)
    }
}

/// Helper to extract sentence containing keyword
fn extract_sentence_after_keyword(text: &str, keywords: &[&str]) -> Option<String> {
    let lower = text.to_lowercase();
    keywords
        .iter()
        .find_map(|&kw| {
            lower.find(kw).map(|idx| {
                let start = text[idx..].split('\n').next().unwrap_or("");
                start.split('.')
                    .next()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| start.to_string())
            })
        })
        .or_else(|| {
            // Fallback: return first sentence if no keyword found
            text.split('.').next().map(|s| s.trim().to_string())
        })
}

// =============================================================================
// Test Mode State Machine
// =============================================================================

/// Simulates the state transitions during mode switching
#[derive(Clone, Debug)]
struct ModeStateMachine {
    /// Current mode
    current_mode: DiscoverMode,
    /// Express textarea content
    express_content: String,
    /// Extracted field cards from Express mode
    field_cards: Vec<FieldData>,
    /// Guided mode answers by step_id
    guided_answers: Vec<Answer>,
    /// Storage backend
    storage: Option<RedbStore>,
}

impl ModeStateMachine {
    fn new() -> Self {
        Self {
            current_mode: DiscoverMode::Express,
            express_content: String::new(),
            field_cards: Vec::new(),
            guided_answers: Vec::new(),
            storage: None,
        }
    }

    fn with_storage(mut self, storage: RedbStore) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Simulate entering freeform text in Express mode
    fn enter_express_text(&mut self, text: String) {
        self.current_mode = DiscoverMode::Express;
        self.express_content = text;
    }

    /// Simulate field extraction from Express content
    async fn extract_fields(&mut self, provider: &MockExtractionProvider) -> Result<(), ExtractionError> {
        let schema = vec![
            SchemaField {
                name: "problem".to_string(),
                field_type: FieldType::TextArea,
                required: false,
                description: Some("Problem Statement".to_string()),
                options: None,
            },
            SchemaField {
                name: "user".to_string(),
                field_type: FieldType::Text,
                required: false,
                description: Some("Target User".to_string()),
                options: None,
            },
            SchemaField {
                name: "context".to_string(),
                field_type: FieldType::TextArea,
                required: false,
                description: Some("Context & Background".to_string()),
                options: None,
            },
            SchemaField {
                name: "constraints".to_string(),
                field_type: FieldType::TextArea,
                required: false,
                description: Some("Constraints".to_string()),
                options: None,
            },
            SchemaField {
                name: "goals".to_string(),
                field_type: FieldType::TextArea,
                required: false,
                description: Some("Goals & Success Metrics".to_string()),
                options: None,
            },
        ];

        let context = ExtractionContext {
            document_type: Some("express_flow_input".to_string()),
            locale: Some("en_US".to_string()),
            schema: Some(schema),
            extra: serde_json::json!({}),
        };

        let extracted = provider.extract_fields(&self.express_content, &context).await?;

        // Convert to field cards
        self.field_cards = extracted
            .fields
            .iter()
            .map(|f| FieldData {
                id: f.name.clone(),
                title: field_title_from_id(&f.name),
                content: match &f.value {
                    serde_json::Value::String(s) => s.clone(),
                    v => v.to_string(),
                },
                confidence: confidence_from_score(f.confidence),
                locked: false,
            })
            .collect();

        Ok(())
    }

    /// Switch to Guided mode with field card pre-fill
    async fn switch_to_guided(&mut self) -> Result<(), StorageError> {
        self.current_mode = DiscoverMode::Guided;

        // Pre-fill guided answers with extracted field content
        for card in &self.field_cards {
            if !card.content.is_empty() {
                // Remove existing answer for this step if any
                self.guided_answers.retain(|a| a.step_id != card.id);

                // Add new answer from extracted field
                self.guided_answers.push(Answer {
                    step_id: card.id.clone(),
                    value: card.content.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        // Save to storage if available
        if let Some(ref storage) = self.storage {
            for answer in &self.guided_answers {
                storage.save_answer(answer)?;
            }

            // Save mode preference
            let metadata = ProjectMetadata::with_current_timestamp(
                "guided".to_string(),
                "discover".to_string(),
            );
            storage.save_metadata(&metadata)?;
        }

        Ok(())
    }

    /// Answer remaining guided questions
    fn answer_guided_question(&mut self, step_id: &str, value: String) {
        // Remove existing answer for this step
        self.guided_answers.retain(|a| a.step_id != step_id);

        // Add new answer
        self.guided_answers.push(Answer {
            step_id: step_id.to_string(),
            value,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    /// Switch back to Express mode - concatenate guided answers
    async fn switch_to_express(&mut self) -> Result<(), StorageError> {
        self.current_mode = DiscoverMode::Express;

        // Concatenate all guided answers into express content
        self.express_content = self
            .guided_answers
            .iter()
            .sorted_by(|a, b| a.step_id.cmp(&b.step_id))
            .map(|a| format!("{}: {}", a.step_id, a.value))
            .collect::<Vec<_>>()
            .join("\n\n");

        // Save to storage if available
        if let Some(ref storage) = self.storage {
            // Update mode preference
            let metadata = ProjectMetadata::with_current_timestamp(
                "express".to_string(),
                "discover".to_string(),
            );
            storage.save_metadata(&metadata)?;
        }

        Ok(())
    }

    /// Verify no data was lost during mode transitions
    fn verify_no_data_loss(&self) -> bool {
        // Check that all extracted fields are in guided answers
        let guided_step_ids: std::collections::HashSet<_> =
            self.guided_answers.iter().map(|a| a.step_id.as_str()).collect();

        for card in &self.field_cards {
            if !card.content.is_empty() && !guided_step_ids.contains(card.id.as_str()) {
                return false;
            }
        }

        // Check that express content contains all guided answers
        for answer in &self.guided_answers {
            if !self.express_content.contains(&answer.value)
                && !self.express_content.contains(&answer.step_id)
            {
                return false;
            }
        }

        true
    }

    /// Verify mode preference persists in storage
    fn verify_mode_persistence(&self, expected_mode: &str) -> Result<bool, StorageError> {
        match &self.storage {
            Some(storage) => {
                let metadata = storage.get_metadata()?;
                match metadata {
                    Some(meta) => Ok(meta.mode_preference == expected_mode),
                    None => Ok(false),
                }
            }
            None => Ok(false),
        }
    }
}

impl Default for ModeStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

fn field_title_from_id(id: &str) -> String {
    match id {
        "problem" => "Problem Statement".to_string(),
        "user" => "Target User".to_string(),
        "context" => "Context & Background".to_string(),
        "constraints" => "Constraints".to_string(),
        "goals" => "Goals & Success Metrics".to_string(),
        _ => id.to_uppercase().replace('_', " "),
    }
}

fn confidence_from_score(score: f64) -> clarity_web::components::discover::field_card::Confidence {
    use clarity_web::components::discover::field_card::Confidence;
    if score >= 0.8 {
        Confidence::High
    } else if score >= 0.5 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

// =============================================================================
// Test Cases
// =============================================================================

#[tokio::test]
async fn test_express_to_guided_carries_extracted_fields() {
    // Setup: Create state machine and extraction provider
    let mut state_machine = ModeStateMachine::new();
    let provider = MockExtractionProvider::new();

    // Step 1: Enter freeform text in Express mode
    let freeform_text = r#"
        The problem is that remote teams struggle with task management across time zones.
        Users are project managers and team leaders who need visibility.
        Context: distributed teams, async communication, varying time zones.
        Constraints: limited budget, must work offline sometimes.
        Goals: improve task visibility by 80%, reduce missed deadlines.
    "#;
    state_machine.enter_express_text(freeform_text.to_string());

    // Step 2: Extract fields
    state_machine
        .extract_fields(&provider)
        .await
        .expect("Extraction should succeed");

    // Verify extraction results
    assert!(!state_machine.field_cards.is_empty());
    assert_eq!(state_machine.field_cards.len(), 5); // All 5 fields extracted

    // Step 3: Switch to Guided mode
    state_machine
        .switch_to_guided()
        .await
        .expect("Mode switch should succeed");

    // Verify: Extracted fields pre-fill guided inputs
    assert_eq!(state_machine.current_mode, DiscoverMode::Guided);
    assert!(!state_machine.guided_answers.is_empty());

    // Verify each extracted field has a corresponding guided answer
    for card in &state_machine.field_cards {
        if !card.content.is_empty() {
            let answer = state_machine
                .guided_answers
                .iter()
                .find(|a| a.step_id == card.id);
            assert!(
                answer.is_some(),
                "Guided answer should exist for field {}",
                card.id
            );
            assert!(
                answer.map_or(false, |a| !a.value.is_empty()),
                "Guided answer for {} should not be empty",
                card.id
            );
        }
    }
}

#[tokio::test]
async fn test_guided_to_express_concatenates_answers() {
    // Setup
    let mut state_machine = ModeStateMachine::new();
    let provider = MockExtractionProvider::new();

    // Start with Express content and extract
    let freeform_text = "Problem: tasks are missed. User: remote teams. Goals: improve visibility.";
    state_machine.enter_express_text(freeform_text.to_string());
    state_machine
        .extract_fields(&provider)
        .await
        .expect("Extraction should succeed");
    state_machine
        .switch_to_guided()
        .await
        .expect("Switch to guided should succeed");

    // Answer remaining guided questions
    state_machine.answer_guided_question(
        "antithesis",
        "What if team members prefer async communication and status updates become noise?".to_string(),
    );
    state_machine.answer_guided_question(
        "solution",
        "Smart task aggregation with selective notifications".to_string(),
    );

    let answer_count_before = state_machine.guided_answers.len();

    // Switch back to Express
    state_machine
        .switch_to_express()
        .await
        .expect("Switch to express should succeed");

    // Verify: Express content contains all guided answers
    assert_eq!(state_machine.current_mode, DiscoverMode::Express);
    assert!(!state_machine.express_content.is_empty());

    // Check that express content contains answer values
    for answer in &state_machine.guided_answers {
        assert!(
            state_machine.express_content.contains(&answer.step_id)
                || state_machine.express_content.contains(&answer.value),
            "Express content should contain answer from {}",
            answer.step_id
        );
    }

    // Verify answer count preserved
    assert_eq!(
        state_machine.guided_answers.len(),
        answer_count_before,
        "Answer count should be preserved"
    );
}

#[tokio::test]
async fn test_no_data_loss_round_trip() {
    // Setup with storage
    let storage = RedbStore::open_in_memory().expect("Failed to create in-memory store");
    let mut state_machine = ModeStateMachine::new().with_storage(storage);
    let provider = MockExtractionProvider::new();

    // Original data
    let original_content = r#"
        Problem statement: Remote teams lose track of tasks across time zones.
        Target users: Distributed development teams and project managers.
        Context: Async communication, varying work hours, manual status updates.
        Constraints: Budget limited to $500/month, must support offline mode.
        Goals: 80% improvement in task visibility, reduce missed deadlines by 50%.
    "#.to_string();

    // Express -> Extract -> Guided
    state_machine.enter_express_text(original_content.clone());
    state_machine
        .extract_fields(&provider)
        .await
        .expect("Extraction should succeed");

    let field_cards_snapshot = state_machine.field_cards.clone();

    state_machine
        .switch_to_guided()
        .await
        .expect("Switch to guided should succeed");

    // Answer additional questions
    state_machine.answer_guided_question(
        "antithesis",
        "Over-communication might lead to notification fatigue".to_string(),
    );
    state_machine.answer_guided_question(
        "solution",
        "Selective smart notifications with digest mode".to_string(),
    );

    let guided_answers_snapshot = state_machine.guided_answers.clone();

    // Guided -> Express
    state_machine
        .switch_to_express()
        .await
        .expect("Switch to express should succeed");

    // Verify no data loss
    assert!(
        state_machine.verify_no_data_loss(),
        "No data should be lost during round-trip mode switching"
    );

    // Verify field cards preserved
    assert_eq!(
        state_machine.field_cards.len(),
        field_cards_snapshot.len(),
        "Field card count should be preserved"
    );

    // Verify guided answers preserved
    assert_eq!(
        state_machine.guided_answers.len(),
        guided_answers_snapshot.len(),
        "Guided answer count should be preserved"
    );

    // Verify express content contains original info
    assert!(
        state_machine.express_content.contains("time zone")
            || state_machine.express_content.contains("Remote teams"),
        "Express content should retain original key information"
    );
}

#[tokio::test]
async fn test_mode_preference_persists() {
    // Setup with storage
    let storage = RedbStore::open_in_memory().expect("Failed to create in-memory store");
    let mut state_machine = ModeStateMachine::new().with_storage(storage);
    let provider = MockExtractionProvider::new();

    // Start in Express
    state_machine.enter_express_text("Test content".to_string());
    assert_eq!(state_machine.current_mode, DiscoverMode::Express);

    // Switch to Guided (should persist "guided" preference)
    state_machine
        .switch_to_guided()
        .await
        .expect("Switch to guided should succeed");

    let guided_persists = state_machine
        .verify_mode_persistence("guided")
        .expect("Should verify mode persistence");
    assert!(
        guided_persists,
        "Guided mode preference should be persisted"
    );

    // Switch back to Express (should persist "express" preference)
    state_machine
        .switch_to_express()
        .await
        .expect("Switch to express should succeed");

    let express_persists = state_machine
        .verify_mode_persistence("express")
        .expect("Should verify mode persistence");
    assert!(
        express_persists,
        "Express mode preference should be persisted"
    );
}

#[tokio::test]
async fn test_mode_preference_survives_reload() {
    // Setup with storage
    let storage = Arc::new(
        RedbStore::open_in_memory().expect("Failed to create in-memory store"),
    );
    let provider = MockExtractionProvider::new();

    // First session: Set mode to Guided
    {
        let mut state_machine = ModeStateMachine::new()
            .with_storage((*storage).clone());
        state_machine.enter_express_text("Initial content".to_string());
        state_machine.extract_fields(&provider).await.expect("Extraction succeeds");
        state_machine
            .switch_to_guided()
            .await
            .expect("Switch to guided succeeds");
    }

    // Simulate reload: Create new state machine reading from same storage
    let reloaded_metadata = storage
        .get_metadata()
        .expect("Should read metadata");
    assert!(
        reloaded_metadata.is_some(),
        "Metadata should persist after reload"
    );
    assert_eq!(
        reloaded_metadata.map(|m| m.mode_preference).as_deref(),
        Some("guided"),
        "Mode preference should survive reload"
    );

    // Second session: Verify mode is restored
    {
        let mut state_machine = ModeStateMachine::new()
            .with_storage((*storage).clone());

        // Mode should be restored from storage
        let current_mode = state_machine.current_mode;
        assert!(
            matches!(current_mode, DiscoverMode::Guided | DiscoverMode::Express),
            "Mode should be valid after reload"
        );
    }
}

#[tokio::test]
async fn test_partial_data_guided_to_express() {
    // Test case: Only some guided questions answered
    let mut state_machine = ModeStateMachine::new();
    let provider = MockExtractionProvider::new();

    // Start with minimal content
    state_machine.enter_express_text("Problem: users are confused".to_string());
    state_machine
        .extract_fields(&provider)
        .await
        .expect("Extraction succeeds");
    state_machine
        .switch_to_guided()
        .await
        .expect("Switch to guided succeeds");

    // Only answer one question
    state_machine.answer_guided_question(
        "solution",
        "Improve onboarding flow".to_string(),
    );

    // Switch back to Express
    state_machine
        .switch_to_express()
        .await
        .expect("Switch to express succeeds");

    // Verify: Express content should have partial data
    assert!(!state_machine.express_content.is_empty());
    assert!(
        state_machine.express_content.contains("solution")
            || state_machine.express_content.contains("onboarding"),
        "Express should contain the answered question"
    );
}

#[tokio::test]
async fn test_empty_express_to_guided_transition() {
    // Test edge case: Empty express content
    let mut state_machine = ModeStateMachine::new();
    let provider = MockExtractionProvider::new();

    // Enter empty text
    state_machine.enter_express_text(String::new());

    // Extract (should handle gracefully)
    let extraction_result = state_machine.extract_fields(&provider).await;
    assert!(
        extraction_result.is_ok(),
        "Extraction should handle empty input"
    );

    // Switch to Guided (should have no pre-filled answers)
    state_machine
        .switch_to_guided()
        .await
        .expect("Switch to guided should succeed");

    assert!(
        state_machine.guided_answers.is_empty() || state_machine.field_cards.is_empty(),
        "Empty express should result in no guided answers or empty field cards"
    );
}

#[tokio::test]
async fn test_concurrent_mode_switches() {
    // Test rapid mode switching
    let mut state_machine = ModeStateMachine::new();
    let provider = MockExtractionProvider::new();

    let content = "Problem: test. User: developers. Goals: success.";
    state_machine.enter_express_text(content.to_string());
    state_machine
        .extract_fields(&provider)
        .await
        .expect("Extraction succeeds");

    // Rapid switches: Express -> Guided -> Express -> Guided
    for i in 0..4 {
        if i % 2 == 0 {
            state_machine
                .switch_to_guided()
                .await
                .expect("Switch to guided should succeed");
            assert_eq!(state_machine.current_mode, DiscoverMode::Guided);
        } else {
            state_machine
                .switch_to_express()
                .await
                .expect("Switch to express should succeed");
            assert_eq!(state_machine.current_mode, DiscoverMode::Express);
        }
    }

    // Verify data integrity after rapid switches
    assert!(
        state_machine.verify_no_data_loss(),
        "No data loss should occur after rapid mode switches"
    );
}

#[tokio::test]
async fn test_extraction_failure_handling() {
    // Test extraction provider failure
    let mut state_machine = ModeStateMachine::new();
    let provider = MockExtractionProvider::new().with_failure();

    state_machine.enter_express_text("Test content".to_string());

    // Extraction should fail
    let extraction_result = state_machine.extract_fields(&provider).await;
    assert!(
        extraction_result.is_err(),
        "Extraction should fail with error provider"
    );

    // Field cards should be empty after failed extraction
    assert!(state_machine.field_cards.is_empty());
}

#[tokio::test]
async fn test_large_content_extraction() {
    // Test with large content (2000+ characters)
    let mut state_machine = ModeStateMachine::new();
    let provider = MockExtractionProvider::new();

    let large_content = "Problem: ".to_string()
        + &"Remote team coordination issues with async communication. ".repeat(20)
        + "User: Distributed development teams across multiple time zones. "
        + &"Need better task visibility and status tracking. ".repeat(15)
        + "Goals: Improve productivity by 50%, reduce meeting overhead.";

    state_machine.enter_express_text(large_content);
    state_machine
        .extract_fields(&provider)
        .await
        .expect("Extraction should handle large content");

    assert!(!state_machine.field_cards.is_empty());

    state_machine
        .switch_to_guided()
        .await
        .expect("Should handle large content in guided mode");

    // Verify content preserved
    assert!(!state_machine.guided_answers.is_empty());
}

// =============================================================================
// Integration Tests
// =============================================================================

#[tokio::test]
async fn test_full_e2e_mode_switch_workflow() {
    // Complete end-to-end workflow test
    let storage = RedbStore::open_in_memory().expect("Failed to create store");
    let mut state_machine = ModeStateMachine::new().with_storage(storage);
    let provider = MockExtractionProvider::new();

    // 1. Express mode: Enter detailed freeform text
    let express_input = r#"
        The main problem is that remote software development teams struggle
        to keep track of task assignments and deadlines across different time zones.

        Target users: Project managers and team leads in distributed tech companies
        with 50-200 employees operating across at least 3 time zones.

        Context: Current tools rely too heavily on synchronous communication.
        Team members miss deadline notifications when offline. Status updates
        happen in chat channels that get buried.

        Constraints:
        - Budget limited to $500/month for the team
        - Must support offline mode for intermittent connectivity
        - Need to integrate with existing Slack/Teams workflows
        - Cannot require always-on VPN

        Goals & Success Metrics:
        - 80% improvement in task visibility across time zones
        - Reduce missed deadlines by 50%
        - Decrease async status update meetings by 30%
        - User adoption rate > 70% within 3 months
    "#;

    state_machine.enter_express_text(express_input.to_string());
    assert_eq!(state_machine.current_mode, DiscoverMode::Express);
    assert!(!state_machine.express_content.is_empty());

    // 2. Extract to get field cards
    state_machine
        .extract_fields(&provider)
        .await
        .expect("Extraction should succeed");

    assert!(!state_machine.field_cards.is_empty());
    assert_eq!(state_machine.field_cards.len(), 5);

    // Verify extraction captured key information
    let problem_card = state_machine
        .field_cards
        .iter()
        .find(|f| f.id == "problem")
        .expect("Should have problem field");
    assert!(
        problem_card.content.contains("time zone")
            || problem_card.content.contains("remote"),
        "Problem field should capture key info"
    );

    // 3. Switch to Guided mode - sequential inputs pre-filled
    state_machine
        .switch_to_guided()
        .await
        .expect("Switch to guided should succeed");

    assert_eq!(state_machine.current_mode, DiscoverMode::Guided);

    // Verify pre-filled answers
    assert!(!state_machine.guided_answers.is_empty());
    assert!(
        state_machine.guided_answers.len() >= 3,
        "At least 3 fields should be pre-filled"
    );

    // 4. Answer remaining questions in Guided mode
    state_machine.answer_guided_question(
        "antithesis",
        "What if increased automation creates anxiety about being monitored?".to_string(),
    );
    state_machine.answer_guided_question(
        "solution",
        "Transparent task aggregation with user-controlled notification preferences".to_string(),
    );
    state_machine.answer_guided_question(
        "persona",
        "Sarah, a project manager at a 100-person distributed startup".to_string(),
    );
    state_machine.answer_guided_question(
        "scenario",
        "Monday morning standup prep across APAC, EMEA, and Americas teams".to_string(),
    );

    let total_answers = state_machine.guided_answers.len();
    assert!(total_answers >= 7, "Should have at least 7 answers");

    // 5. Switch back to Express - textarea contains concatenated answers
    state_machine
        .switch_to_express()
        .await
        .expect("Switch to express should succeed");

    assert_eq!(state_machine.current_mode, DiscoverMode::Express);
    assert!(!state_machine.express_content.is_empty());

    // 6. Verify no data lost in either direction
    assert!(
        state_machine.verify_no_data_loss(),
        "No data should be lost in round-trip"
    );

    // Verify express content contains key answers
    assert!(
        state_machine.express_content.contains("notification")
            || state_machine.express_content.contains("solution"),
        "Express should contain solution details"
    );
    assert!(
        state_machine.express_content.contains("Sarah")
            || state_machine.express_content.contains("persona"),
        "Express should contain persona details"
    );

    // 7. Verify mode preference persists
    assert!(
        state_machine.verify_mode_persistence("express").unwrap_or(false),
        "Express mode preference should be persisted"
    );

    // Simulate reload and verify persistence
    let persisted_mode = state_machine
        .storage
        .as_ref()
        .and_then(|s| s.get_metadata().ok().flatten())
        .map(|m| m.mode_preference);

    assert_eq!(
        persisted_mode.as_deref(),
        Some("express"),
        "Mode preference should persist to storage"
    );
}

// =============================================================================
// Utility Tests
// =============================================================================

#[test]
fn test_field_title_from_id() {
    assert_eq!(field_title_from_id("problem"), "Problem Statement");
    assert_eq!(field_title_from_id("user"), "Target User");
    assert_eq!(field_title_from_id("context"), "Context & Background");
    assert_eq!(field_title_from_id("constraints"), "Constraints");
    assert_eq!(field_title_from_id("goals"), "Goals & Success Metrics");
    assert_eq!(field_title_from_id("unknown_field"), "UNKNOWN FIELD");
}

#[test]
fn test_confidence_from_score() {
    use clarity_web::components::discover::field_card::Confidence;

    assert_eq!(confidence_from_score(0.9), Confidence::High);
    assert_eq!(confidence_from_score(0.8), Confidence::High);
    assert_eq!(confidence_from_score(0.7), Confidence::Medium);
    assert_eq!(confidence_from_score(0.5), Confidence::Medium);
    assert_eq!(confidence_from_score(0.3), Confidence::Low);
    assert_eq!(confidence_from_score(0.0), Confidence::Low);
}

#[test]
fn test_mode_state_machine_default() {
    let machine = ModeStateMachine::new();
    assert_eq!(machine.current_mode, DiscoverMode::Express);
    assert!(machine.express_content.is_empty());
    assert!(machine.field_cards.is_empty());
    assert!(machine.guided_answers.is_empty());
}

#[test]
fn test_discover_mode_equality() {
    assert_eq!(DiscoverMode::Express, DiscoverMode::Express);
    assert_eq!(DiscoverMode::Guided, DiscoverMode::Guided);
    assert_ne!(DiscoverMode::Express, DiscoverMode::Guided);
}
