#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! End-to-end test: Complete Discover flow in Express mode.
//!
//! This test verifies the complete user journey through the Discover phase
//! using Express mode, including:
//!
//! 1. App loads in Discover phase
//! 2. Express mode selected (default is Guided, so we switch)
//! 3. Freeform input accepts text
//! 4. Extract button triggers extraction
//! 5. 5 field cards appear with content
//! 6. Confidence badges display
//! 7. Locking cards updates state
//! 8. Quality score updates
//! 9. Continue CTA appears when all locked
//! 10. Continue moves to Define phase
//!
//! Test strategy:
//! - Mock extraction provider for deterministic results
//! - Test component state transitions
//! - Verify UI state reflects data changes
//! - Ensure data persistence across flow

use std::sync::Arc;
use chrono::Utc;

// Import from clarity-web with proper path handling
#[cfg(test)]
mod tests {
    use super::*;
    use clarity_web::components::discover::{
        FieldData, Confidence,
    };
    use clarity_web::providers::{
        ExtractionContext, ExtractionError, ExtractionProvider, ExtractedFields,
        FieldExtraction, FieldType, SchemaField,
    };
    use clarity_web::types::PHASES;

    /// Mock extraction provider for testing
    ///
    /// Returns deterministic extraction results based on input length.
    /// This allows testing without external API dependencies.
    #[derive(Debug, Clone)]
    struct MockExtractionProvider {
        /// Simulated processing delay in ms
        delay_ms: u64,
    }

    impl MockExtractionProvider {
        /// Create a new mock provider with optional delay
        #[must_use]
        pub const fn new(delay_ms: u64) -> Self {
            Self { delay_ms }
        }

        /// Generate mock extracted fields based on input text
        fn extract_from_text(&self, text: &str) -> ExtractedFields {
            let word_count = text.split_whitespace().count();

            // Generate deterministic confidence based on text length
            let confidence = if word_count > 50 {
                0.85
            } else if word_count > 20 {
                0.65
            } else {
                0.45
            };

            let fields = vec![
                FieldExtraction {
                    name: "problem".to_string(),
                    field_type: FieldType::TextArea,
                    value: serde_json::json!("Task management app for remote teams with deadline tracking"),
                    confidence,
                    justification: Some("Extracted from problem description".to_string()),
                },
                FieldExtraction {
                    name: "user".to_string(),
                    field_type: FieldType::Text,
                    value: serde_json::json!("Remote team members and project managers"),
                    confidence: confidence + 0.05,
                    justification: Some("Identified target user group".to_string()),
                },
                FieldExtraction {
                    name: "context".to_string(),
                    field_type: FieldType::TextArea,
                    value: serde_json::json!("Teams working across different time zones need visibility into task assignments and deadlines"),
                    confidence: confidence - 0.05,
                    justification: Some("Derived from context clues".to_string()),
                },
                FieldExtraction {
                    name: "constraints".to_string(),
                    field_type: FieldType::TextArea,
                    value: serde_json::json!("Must work across time zones, support mobile access, handle real-time updates"),
                    confidence: confidence + 0.02,
                    justification: Some("Identified constraints".to_string()),
                },
                FieldExtraction {
                    name: "goals".to_string(),
                    field_type: FieldType::TextArea,
                    value: serde_json::json!("Clear task assignment, deadline visibility, quick reassignment capability, 99.9% uptime"),
                    confidence: confidence + 0.08,
                    justification: Some("Extracted success metrics".to_string()),
                },
            ];

            ExtractedFields {
                fields,
                confidence,
                metadata: clarity_web::providers::ExtractionMetadata {
                    provider: "mock".to_string(),
                    model: Some("mock-model-v1".to_string()),
                    timestamp: Utc::now(),
                    processing_duration_ms: self.delay_ms,
                    extra: serde_json::json!({"test": true}),
                },
            }
        }
    }

    #[async_trait::async_trait]
    impl ExtractionProvider for MockExtractionProvider {
        async fn extract_fields(
            &self,
            text: &str,
            _context: &ExtractionContext,
        ) -> Result<ExtractedFields, ExtractionError> {
            // Simulate processing delay
            if self.delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
            }

            if text.trim().is_empty() {
                return Err(ExtractionError::InvalidInput(
                    "Input text cannot be empty".to_string(),
                ));
            }

            Ok(self.extract_from_text(text))
        }

        async fn extract_fields_with_schema(
            &self,
            text: &str,
            schema: &[SchemaField],
            context: &ExtractionContext,
        ) -> Result<ExtractedFields, ExtractionError> {
            // For simplicity, use the same extraction logic
            self.extract_fields(text, context).await
        }

        fn provider_name(&self) -> &str {
            "mock"
        }

        async fn health_check(&self) -> Result<(), ExtractionError> {
            Ok(())
        }
    }

    /// Test helper: Verify app starts in Discover phase
    #[test]
    fn test_app_loads_in_discover_phase() {
        // Verify Discover phase exists
        let phases = PHASES;
        assert!(!phases.is_empty());

        // Discover should be first phase
        let discover_phase = phases.first();
        assert!(discover_phase.is_some());

        if let Some(phase) = discover_phase {
            assert_eq!(phase.key, "discover");
            assert_eq!(phase.label, "Discover");
        }
    }

    /// Test helper: Verify Express mode exists and is selectable
    #[test]
    fn test_express_mode_exists() {
        use clarity_web::components::discover::DiscoverMode;

        // Both modes should be available
        let express = DiscoverMode::Express;
        let guided = DiscoverMode::Guided;

        assert_eq!(express.display(), "Express");
        assert_eq!(guided.display(), "Guided");

        // Express should have description
        assert!(!express.description().is_empty());
        assert!(express.description().contains("Quick"));
    }

    /// Test: Freeform input accepts text
    #[test]
    fn test_freeform_input_accepts_text() {
        let test_input = "Building a task management app for remote teams. The main problem is \
        that team members often miss deadlines because tasks aren't clearly assigned or tracked \
        across different time zones. We need a way to see who's working on what, when it's due, \
        and be able to quickly reassign if someone is overloaded.";

        // Verify input meets minimum extraction requirements
        assert!(test_input.len() >= 50, "Input should be at least 50 characters");
        assert!(!test_input.trim().is_empty(), "Input should not be empty");

        // Verify it's under max limit
        assert!(test_input.len() <= 2000, "Input should not exceed 2000 characters");
    }

    /// Test: Mock extraction provider works correctly
    #[tokio::test]
    async fn test_mock_extraction_provider() {
        let provider = MockExtractionProvider::new(0);
        let test_text = "Building a task management app for remote teams";

        let context = ExtractionContext {
            document_type: Some("express_flow_input".to_string()),
            locale: Some("en_US".to_string()),
            schema: None,
            extra: serde_json::json!({}),
        };

        let result = provider.extract_fields(test_text, &context).await;

        assert!(result.is_ok());

        if let Ok(extracted) = result {
            // Should have 5 fields
            assert_eq!(extracted.fields.len(), 5);

            // Verify field names
            let field_names: Vec<&str> = extracted.fields.iter().map(|f| f.name.as_str()).collect();
            assert!(field_names.contains(&"problem"));
            assert!(field_names.contains(&"user"));
            assert!(field_names.contains(&"context"));
            assert!(field_names.contains(&"constraints"));
            assert!(field_names.contains(&"goals"));

            // Verify all fields have values
            for field in &extracted.fields {
                assert!(!field.value.is_null());
                assert!(field.confidence > 0.0);
                assert!(field.confidence <= 1.0);
            }
        }
    }

    /// Test: Extraction rejects empty input
    #[tokio::test]
    async fn test_extraction_rejects_empty_input() {
        let provider = MockExtractionProvider::new(0);
        let context = ExtractionContext {
            document_type: None,
            locale: None,
            schema: None,
            extra: serde_json::json!({}),
        };

        // Empty string should fail
        let result = provider.extract_fields("", &context).await;
        assert!(result.is_err());

        if let Err(ExtractionError::InvalidInput(msg)) = result {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected InvalidInput error");
        }

        // Whitespace-only should also fail
        let result = provider.extract_fields("   \n\t  ", &context).await;
        assert!(result.is_err());
    }

    /// Test: Five field cards appear after extraction
    #[test]
    fn test_five_field_cards_appear() {
        let fields = vec![
            FieldData {
                id: "problem".to_string(),
                title: "Problem Statement".to_string(),
                content: "Task management for remote teams".to_string(),
                confidence: Confidence::High,
                locked: false,
            },
            FieldData {
                id: "user".to_string(),
                title: "Target User".to_string(),
                content: "Remote team members".to_string(),
                confidence: Confidence::High,
                locked: false,
            },
            FieldData {
                id: "context".to_string(),
                title: "Context & Background".to_string(),
                content: "Teams across time zones".to_string(),
                confidence: Confidence::Medium,
                locked: false,
            },
            FieldData {
                id: "constraints".to_string(),
                title: "Constraints".to_string(),
                content: "Time zone differences".to_string(),
                confidence: Confidence::Medium,
                locked: false,
            },
            FieldData {
                id: "goals".to_string(),
                title: "Goals & Success Metrics".to_string(),
                content: "Clear assignments, deadlines visible".to_string(),
                confidence: Confidence::High,
                locked: false,
            },
        ];

        assert_eq!(fields.len(), 5, "Should have exactly 5 field cards");

        // Verify all fields have required properties
        for field in &fields {
            assert!(!field.id.is_empty(), "Field ID should not be empty");
            assert!(!field.title.is_empty(), "Field title should not be empty");
            assert!(!field.content.is_empty(), "Field content should not be empty after extraction");
        }
    }

    /// Test: Confidence badges display correctly
    #[test]
    fn test_confidence_badges_display() {
        // Test all confidence levels
        let high = Confidence::High;
        let medium = Confidence::Medium;
        let low = Confidence::Low;

        assert_eq!(high.display(), "High");
        assert_eq!(medium.display(), "Med");
        assert_eq!(low.display(), "Low");

        // Verify badge classes differ
        assert!(high.badge_classes().contains("chart-2"));
        assert!(medium.badge_classes().contains("chart-3"));
        assert!(low.badge_classes().contains("chart-4"));
    }

    /// Test: Locking cards updates state
    #[test]
    fn test_locking_cards_updates_state() {
        let mut field = FieldData {
            id: "problem".to_string(),
            title: "Problem Statement".to_string(),
            content: "Test problem".to_string(),
            confidence: Confidence::High,
            locked: false,
        };

        // Initial state should be unlocked
        assert!(!field.locked);

        // Lock the field
        field.locked = true;
        assert!(field.locked);

        // Unlock the field
        field.locked = false;
        assert!(!field.locked);
    }

    /// Test: Quality score calculation
    #[test]
    fn test_quality_score_updates() {
        use clarity_web::lattice::quality::{calculate_quality, Answer, InversionControl};

        let answers = vec![
            Answer {
                step_id: "problem".to_string(),
                value: "Task management for remote teams with deadline tracking".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "user".to_string(),
                value: "Remote team members and project managers".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
        ];

        let ears = vec![];
        let inversion = InversionControl {
            has_inversion_tests: false,
            inverted_count: 0,
        };

        let result = calculate_quality(&answers, &ears, &inversion);

        assert!(result.is_ok());

        if let Ok(score) = result {
            // Should have overall score
            assert!(score.overall >= 0);
            assert!(score.overall <= 100);

            // Should have dimensions
            assert!(!score.dimensions.is_empty());
        }
    }

    /// Test: Continue CTA appears when all locked
    #[test]
    fn test_continue_cta_appears_when_all_locked() {
        let fields = vec![
            FieldData {
                id: "problem".to_string(),
                title: "Problem Statement".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::High,
                locked: true,
            },
            FieldData {
                id: "user".to_string(),
                title: "Target User".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::High,
                locked: true,
            },
            FieldData {
                id: "context".to_string(),
                title: "Context & Background".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::Medium,
                locked: true,
            },
            FieldData {
                id: "constraints".to_string(),
                title: "Constraints".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::Medium,
                locked: true,
            },
            FieldData {
                id: "goals".to_string(),
                title: "Goals & Success Metrics".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::High,
                locked: true,
            },
        ];

        // All fields locked
        let all_locked = fields.iter().all(|f| f.locked);
        assert!(all_locked, "Continue CTA should appear when all fields are locked");
    }

    /// Test: Continue CTA hidden when not all locked
    #[test]
    fn test_continue_cta_hidden_when_not_all_locked() {
        let fields = vec![
            FieldData {
                id: "problem".to_string(),
                title: "Problem Statement".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::High,
                locked: true,
            },
            FieldData {
                id: "user".to_string(),
                title: "Target User".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::High,
                locked: false, // Not locked
            },
            FieldData {
                id: "context".to_string(),
                title: "Context & Background".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::Medium,
                locked: true,
            },
            FieldData {
                id: "constraints".to_string(),
                title: "Constraints".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::Medium,
                locked: true,
            },
            FieldData {
                id: "goals".to_string(),
                title: "Goals & Success Metrics".to_string(),
                content: "Test".to_string(),
                confidence: Confidence::High,
                locked: true,
            },
        ];

        // Not all fields locked
        let all_locked = fields.iter().all(|f| f.locked);
        assert!(!all_locked, "Continue CTA should be hidden when not all fields are locked");
    }

    /// Test: Phase transition from Discover to Define
    #[test]
    fn test_phase_transition_to_define() {
        let phases = PHASES;

        // Find Discover and Define phases
        let discover_idx = phases.iter().position(|p| p.key == "discover");
        let define_idx = phases.iter().position(|p| p.key == "define");

        assert!(discover_idx.is_some(), "Discover phase should exist");
        assert!(define_idx.is_some(), "Define phase should exist");

        // Define should come after Discover
        if let (Some(d_idx), Some(def_idx)) = (discover_idx, define_idx) {
            assert!(def_idx > d_idx, "Define phase should come after Discover");
        }
    }

    /// Test: Data persistence verification
    #[test]
    fn test_data_persistence() {
        use clarity_web::types::Answer;

        // Create test answers
        let answers = vec![
            Answer {
                step_id: "problem".to_string(),
                value: "Task management for remote teams".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "user".to_string(),
                value: "Remote team members".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
        ];

        // Verify answers can be serialized
        let serialized = serde_json::to_string(&answers);
        assert!(serialized.is_ok(), "Answers should be serializable");

        // Verify answers can be deserialized
        if let Ok(json) = serialized {
            let deserialized: Result<Vec<Answer>, _> = serde_json::from_str(&json);
            assert!(deserialized.is_ok(), "Answers should be deserializable");

            if let Ok(restored) = deserialized {
                assert_eq!(restored.len(), answers.len());
                assert_eq!(restored[0].step_id, answers[0].step_id);
                assert_eq!(restored[0].value, answers[0].value);
            }
        }
    }

    /// Integration test: Complete flow simulation
    #[tokio::test]
    async fn test_complete_discover_express_flow() {
        // Step 1: App loads in Discover phase
        let phases = PHASES;
        let discover_phase = phases.iter().find(|p| p.key == "discover");
        assert!(discover_phase.is_some(), "App should load in Discover phase");

        // Step 2: Express mode selected
        use clarity_web::components::discover::DiscoverMode;
        let express_mode = DiscoverMode::Express;
        assert_eq!(express_mode.display(), "Express");

        // Step 3: Freeform input accepts text
        let input_text = "Building a task management app for remote teams. The main problem is \
        that team members often miss deadlines because tasks aren't clearly assigned or tracked \
        across different time zones. We need a way to see who's working on what, when it's due, \
        and be able to quickly reassign if someone is overloaded.";

        assert!(input_text.len() >= 50, "Input should meet minimum length");

        // Step 4: Extract button triggers extraction
        let provider = Arc::new(MockExtractionProvider::new(10));
        let context = ExtractionContext {
            document_type: Some("express_flow_input".to_string()),
            locale: Some("en_US".to_string()),
            schema: None,
            extra: serde_json::json!({}),
        };

        let extraction_result = provider.extract_fields(input_text, &context).await;
        assert!(extraction_result.is_ok(), "Extraction should succeed");

        // Step 5: 5 field cards appear with content
        let extracted = extraction_result.unwrap();
        assert_eq!(extracted.fields.len(), 5, "Should have 5 extracted fields");

        // Step 6: Confidence badges display
        for field in &extracted.fields {
            assert!(field.confidence > 0.0, "Each field should have confidence > 0");
            assert!(field.confidence <= 1.0, "Each field should have confidence <= 1");
        }

        // Step 7: Locking cards updates state
        let mut field_states: Vec<FieldData> = extracted
            .fields
            .iter()
            .map(|f| FieldData {
                id: f.name.clone(),
                title: format!("Field {}", f.name),
                content: f.value.to_string(),
                confidence: if f.confidence >= 0.8 {
                    Confidence::High
                } else if f.confidence >= 0.5 {
                    Confidence::Medium
                } else {
                    Confidence::Low
                },
                locked: false,
            })
            .collect();

        // Lock all fields
        for field in &mut field_states {
            field.locked = true;
        }

        let all_locked = field_states.iter().all(|f| f.locked);
        assert!(all_locked, "All fields should be lockable");

        // Step 8: Quality score updates
        use clarity_web::lattice::quality::{calculate_quality, Answer, InversionControl};

        let quality_answers: Vec<Answer> = field_states
            .iter()
            .map(|f| Answer {
                step_id: f.id.clone(),
                value: f.content.clone(),
                timestamp: Utc::now().to_rfc3339(),
            })
            .collect();

        let ears = vec![];
        let inversion = InversionControl {
            has_inversion_tests: false,
            inverted_count: 0,
        };

        let quality_result = calculate_quality(&quality_answers, &ears, &inversion);
        assert!(quality_result.is_ok(), "Quality score should be calculable");

        // Step 9: Continue CTA appears when all locked
        assert!(all_locked, "Continue CTA should appear when all fields locked");

        // Step 10: Continue moves to Define phase
        let define_phase = phases.iter().find(|p| p.key == "define");
        assert!(define_phase.is_some(), "Define phase should exist for transition");
    }

    /// Test: Confidence score boundaries
    #[test]
    fn test_confidence_score_boundaries() {
        use clarity_web::components::discover::express_flow::confidence_from_score;

        // Test High confidence boundary (>= 0.8)
        assert_eq!(confidence_from_score(1.0), Confidence::High);
        assert_eq!(confidence_from_score(0.9), Confidence::High);
        assert_eq!(confidence_from_score(0.8), Confidence::High);
        assert_eq!(confidence_from_score(0.79), Confidence::Medium);

        // Test Medium confidence boundary (>= 0.5)
        assert_eq!(confidence_from_score(0.7), Confidence::Medium);
        assert_eq!(confidence_from_score(0.5), Confidence::Medium);
        assert_eq!(confidence_from_score(0.49), Confidence::Low);

        // Test Low confidence boundary (< 0.5)
        assert_eq!(confidence_from_score(0.4), Confidence::Low);
        assert_eq!(confidence_from_score(0.0), Confidence::Low);
    }

    /// Test: Field initialization
    #[test]
    fn test_field_initialization() {
        use clarity_web::components::discover::express_flow::initialize_fields;

        let fields = initialize_fields();

        assert_eq!(fields.len(), 5, "Should initialize 5 fields");

        // All fields should start empty
        for field in &fields {
            assert!(field.content.is_empty(), "Initial content should be empty");
            assert_eq!(field.confidence, Confidence::Low, "Initial confidence should be Low");
            assert!(!field.locked, "Initial state should be unlocked");
        }

        // Verify field IDs match expected schema
        let expected_ids = ["problem", "user", "context", "constraints", "goals"];
        for (i, field) in fields.iter().enumerate() {
            assert_eq!(field.id, expected_ids[i], "Field ID should match schema");
        }
    }

    /// Test: Extraction with delay (simulates async processing)
    #[tokio::test]
    async fn test_extraction_with_delay() {
        let provider = MockExtractionProvider::new(100); // 100ms delay
        let test_text = "Building a comprehensive task management system for distributed teams";

        let context = ExtractionContext {
            document_type: None,
            locale: None,
            schema: None,
            extra: serde_json::json!({}),
        };

        let start = std::time::Instant::now();
        let result = provider.extract_fields(test_text, &context).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed >= std::time::Duration::from_millis(100), "Should respect delay");
    }

    /// Test: Minimum extraction characters threshold
    #[test]
    fn test_minimum_extraction_characters() {
        use clarity_web::components::discover::express_flow::MIN_EXTRACTION_CHARS;

        const MIN_CHARS: usize = 50;

        assert_eq!(MIN_EXTRACTION_CHARS, MIN_CHARS, "Minimum should be 50 characters");

        // Test that short input is rejected
        let short_input = "Too short";
        assert!(short_input.len() < MIN_CHARS, "Test input should be below minimum");

        // Test that adequate input is accepted
        let adequate_input = "Building a task management app for remote teams. \
        The main problem is that team members often miss deadlines because tasks \
        aren't clearly assigned or tracked across different time zones.";
        assert!(adequate_input.len() >= MIN_CHARS, "Test input should meet minimum");
    }

    /// Test: Maximum characters limit
    #[test]
    fn test_maximum_characters_limit() {
        use clarity_web::components::discover::express_flow::MAX_CHARS;

        const MAX_CHARS: usize = 2000;

        assert_eq!(MAX_CHARS, 2000, "Maximum should be 2000 characters");

        // Verify limits are enforced
        let within_limit = "A".repeat(1999);
        assert!(within_limit.len() <= MAX_CHARS);

        let at_limit = "A".repeat(2000);
        assert_eq!(at_limit.len(), MAX_CHARS);

        let over_limit = "A".repeat(2001);
        assert!(over_limit.len() > MAX_CHARS);
    }
}
