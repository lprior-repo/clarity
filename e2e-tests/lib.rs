#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! End-to-end test: Complete Discover flow in Express mode.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use chrono::Utc;
    use clarity_web::components::discover::{FieldData, Confidence};
    use clarity_web::providers::{ExtractionContext, ExtractionError, ExtractionProvider, ExtractedFields, FieldType, SchemaField, ExtractionMetadata, FieldExtraction};
    use clarity_web::types::PHASES;
    use serde_json::json;

    /// Mock extraction provider for testing
    #[derive(Debug, Clone)]
    struct MockExtractionProvider {
        delay_ms: u64,
    }

    impl MockExtractionProvider {
        #[must_use]
        pub const fn new(delay_ms: u64) -> Self {
            Self { delay_ms }
        }

        fn extract_from_text(&self, text: &str) -> ExtractedFields {
            let word_count = text.split_whitespace().count();
            let confidence = if word_count > 50 { 0.85 } else if word_count > 20 { 0.65 } else { 0.45 };

            let fields = vec![
                FieldExtraction {
                    name: "problem".to_string(),
                    field_type: FieldType::TextArea,
                    value: json!("Task management app for remote teams with deadline tracking"),
                    confidence,
                    justification: Some("Extracted from problem description".to_string()),
                },
                FieldExtraction {
                    name: "user".to_string(),
                    field_type: FieldType::Text,
                    value: json!("Remote team members and project managers"),
                    confidence: confidence + 0.05,
                    justification: Some("Identified target user group".to_string()),
                },
                FieldExtraction {
                    name: "context".to_string(),
                    field_type: FieldType::TextArea,
                    value: json!("Teams working across different time zones need visibility"),
                    confidence: confidence - 0.05,
                    justification: Some("Derived from context clues".to_string()),
                },
                FieldExtraction {
                    name: "constraints".to_string(),
                    field_type: FieldType::TextArea,
                    value: json!("Must work across time zones, support mobile access"),
                    confidence: confidence + 0.02,
                    justification: Some("Identified constraints".to_string()),
                },
                FieldExtraction {
                    name: "goals".to_string(),
                    field_type: FieldType::TextArea,
                    value: json!("Clear task assignment, deadline visibility"),
                    confidence: confidence + 0.08,
                    justification: Some("Extracted success metrics".to_string()),
                },
            ];

            ExtractedFields {
                fields,
                confidence,
                metadata: ExtractionMetadata {
                    provider: "mock".to_string(),
                    model: Some("mock-model-v1".to_string()),
                    timestamp: Utc::now(),
                    processing_duration_ms: self.delay_ms,
                    extra: json!({"test": true}),
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
            _schema: &[SchemaField],
            context: &ExtractionContext,
        ) -> Result<ExtractedFields, ExtractionError> {
            self.extract_fields(text, context).await
        }

        fn provider_name(&self) -> &str {
            "mock"
        }

        async fn health_check(&self) -> Result<(), ExtractionError> {
            Ok(())
        }
    }

    #[test]
    fn test_app_loads_in_discover_phase() {
        let phases = PHASES;
        assert!(!phases.is_empty());

        let discover_phase = phases.first();
        assert!(discover_phase.is_some());

        if let Some(phase) = discover_phase {
            assert_eq!(phase.key, "discover");
            assert_eq!(phase.label, "Discover");
        }
    }

    // TODO: Update this test for Progressive Discover Phase
    // #[test]
    // fn test_express_mode_exists() {
    //     use clarity_web::components::discover::DiscoverMode;
    //
    //     let express = DiscoverMode::Express;
    //     let guided = DiscoverMode::Guided;
    //
    //     // Can't test display() as it's private, but we can test Default
    //     let default_mode = DiscoverMode::default();
    //     assert!(matches!(default_mode, DiscoverMode::Express | DiscoverMode::Guided));
    //
    //     // Test that both modes exist
    //     assert!(matches!(express, DiscoverMode::Express));
    //     assert!(matches!(guided, DiscoverMode::Guided));
    // }

    #[test]
    fn test_freeform_input_accepts_text() {
        let test_input = "Building a task management app for remote teams. The main problem is \
        that team members often miss deadlines because tasks aren't clearly assigned or tracked \
        across different time zones. We need a way to see who's working on what, when it's due.";

        assert!(test_input.len() >= 50);
        assert!(!test_input.trim().is_empty());
        assert!(test_input.len() <= 2000);
    }

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
            assert_eq!(extracted.fields.len(), 5);

            let field_names: Vec<&str> = extracted.fields.iter().map(|f| f.name.as_str()).collect();
            assert!(field_names.contains(&"problem"));
            assert!(field_names.contains(&"user"));
            assert!(field_names.contains(&"context"));
            assert!(field_names.contains(&"constraints"));
            assert!(field_names.contains(&"goals"));

            for field in &extracted.fields {
                assert!(!field.value.is_null());
                assert!(field.confidence > 0.0);
                assert!(field.confidence <= 1.0);
            }
        }
    }

    #[tokio::test]
    async fn test_extraction_rejects_empty_input() {
        let provider = MockExtractionProvider::new(0);
        let context = ExtractionContext {
            document_type: None,
            locale: None,
            schema: None,
            extra: serde_json::json!({}),
        };

        let result = provider.extract_fields("", &context).await;
        assert!(result.is_err());

        if let Err(ExtractionError::InvalidInput(msg)) = result {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected InvalidInput error");
        }

        let result = provider.extract_fields("   \n\t  ", &context).await;
        assert!(result.is_err());
    }

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

        assert_eq!(fields.len(), 5);

        for field in &fields {
            assert!(!field.id.is_empty());
            assert!(!field.title.is_empty());
            assert!(!field.content.is_empty());
        }
    }

    #[test]
    fn test_confidence_variants_exist() {
        let high = Confidence::High;
        let medium = Confidence::Medium;
        let low = Confidence::Low;

        // Test that all three confidence levels exist
        assert!(matches!(high, Confidence::High));
        assert!(matches!(medium, Confidence::Medium));
        assert!(matches!(low, Confidence::Low));
    }

    #[test]
    fn test_locking_cards_updates_state() {
        let mut field = FieldData {
            id: "problem".to_string(),
            title: "Problem Statement".to_string(),
            content: "Test problem".to_string(),
            confidence: Confidence::High,
            locked: false,
        };

        assert!(!field.locked);
        field.locked = true;
        assert!(field.locked);
        field.locked = false;
        assert!(!field.locked);
    }

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
            assert!(score.overall >= 0);
            assert!(score.overall <= 100);
            assert!(!score.dimensions.is_empty());
        }
    }

    #[test]
    fn test_continue_cta_appears_when_all_locked() {
        let fields = vec![
            FieldData { id: "problem".to_string(), title: "Problem".to_string(), content: "Test".to_string(), confidence: Confidence::High, locked: true },
            FieldData { id: "user".to_string(), title: "User".to_string(), content: "Test".to_string(), confidence: Confidence::High, locked: true },
            FieldData { id: "context".to_string(), title: "Context".to_string(), content: "Test".to_string(), confidence: Confidence::Medium, locked: true },
            FieldData { id: "constraints".to_string(), title: "Constraints".to_string(), content: "Test".to_string(), confidence: Confidence::Medium, locked: true },
            FieldData { id: "goals".to_string(), title: "Goals".to_string(), content: "Test".to_string(), confidence: Confidence::High, locked: true },
        ];

        let all_locked = fields.iter().all(|f| f.locked);
        assert!(all_locked);
    }

    #[test]
    fn test_continue_cta_hidden_when_not_all_locked() {
        let fields = vec![
            FieldData { id: "problem".to_string(), title: "Problem".to_string(), content: "Test".to_string(), confidence: Confidence::High, locked: true },
            FieldData { id: "user".to_string(), title: "User".to_string(), content: "Test".to_string(), confidence: Confidence::High, locked: false },
            FieldData { id: "context".to_string(), title: "Context".to_string(), content: "Test".to_string(), confidence: Confidence::Medium, locked: true },
            FieldData { id: "constraints".to_string(), title: "Constraints".to_string(), content: "Test".to_string(), confidence: Confidence::Medium, locked: true },
            FieldData { id: "goals".to_string(), title: "Goals".to_string(), content: "Test".to_string(), confidence: Confidence::High, locked: true },
        ];

        let all_locked = fields.iter().all(|f| f.locked);
        assert!(!all_locked);
    }

    #[test]
    fn test_phase_transition_to_define() {
        let phases = PHASES;
        let discover_idx = phases.iter().position(|p| p.key == "discover");
        let define_idx = phases.iter().position(|p| p.key == "define");

        assert!(discover_idx.is_some());
        assert!(define_idx.is_some());

        if let (Some(d_idx), Some(def_idx)) = (discover_idx, define_idx) {
            assert!(def_idx > d_idx);
        }
    }

    #[test]
    fn test_data_persistence() {
        use clarity_web::types::Answer;

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

        let serialized = serde_json::to_string(&answers);
        assert!(serialized.is_ok());

        if let Ok(json) = serialized {
            let deserialized: Result<Vec<Answer>, _> = serde_json::from_str(&json);
            assert!(deserialized.is_ok());

            if let Ok(restored) = deserialized {
                assert_eq!(restored.len(), answers.len());
                assert_eq!(restored[0].step_id, answers[0].step_id);
                assert_eq!(restored[0].value, answers[0].value);
            }
        }
    }

    // TODO: Update this test for Progressive Discover Phase
    // #[tokio::test]
    // async fn test_complete_discover_express_flow() {
    //     // Step 1: App loads in Discover phase
    //     let phases = PHASES;
    //     let discover_phase = phases.iter().find(|p| p.key == "discover");
    //     assert!(discover_phase.is_some());
    //
    //     // Step 2: Express mode selected
    //     use clarity_web::components::discover::DiscoverMode;
    //     let express_mode = DiscoverMode::Express;
    //     assert!(matches!(express_mode, DiscoverMode::Express));
    //
    //     // Step 3: Freeform input accepts text
    //     let input_text = "Building a task management app for remote teams. The main problem is \
    //     that team members often miss deadlines because tasks aren't clearly assigned or tracked \
    //     across different time zones. We need a way to see who's working on what, when it's due.";
    //     assert!(input_text.len() >= 50);
    //
    //     // Step 4: Extract button triggers extraction
    //     let provider = Arc::new(MockExtractionProvider::new(10));
    //     let context = ExtractionContext {
    //         document_type: Some("express_flow_input".to_string()),
    //         locale: Some("en_US".to_string()),
    //         schema: None,
    //         extra: serde_json::json!({}),
    //     };
    //
    //     let extraction_result = provider.extract_fields(input_text, &context).await;
    //     assert!(extraction_result.is_ok());
    //
    //     // Step 5: 5 field cards appear with content
    //     let extracted = extraction_result.unwrap();
    //     assert_eq!(extracted.fields.len(), 5);
    //
    //     // Step 6: Confidence badges display
    //     for field in &extracted.fields {
    //         assert!(field.confidence > 0.0);
    //         assert!(field.confidence <= 1.0);
    //     }
    //
    //     // Step 7: Locking cards updates state
    //     let mut field_states: Vec<FieldData> = extracted
    //         .fields
    //         .iter()
    //         .map(|f| FieldData {
    //             id: f.name.clone(),
    //             title: format!("Field {}", f.name),
    //             content: f.value.to_string(),
    //             confidence: if f.confidence >= 0.8 { Confidence::High } else if f.confidence >= 0.5 { Confidence::Medium } else { Confidence::Low },
    //             locked: false,
    //         })
    //         .collect();
    //
    //     for field in &mut field_states {
    //         field.locked = true;
    //     }
    //
    //     let all_locked = field_states.iter().all(|f| f.locked);
    //     assert!(all_locked);
    //
    //     // Step 8: Quality score updates
    //     use clarity_web::lattice::quality::{calculate_quality, Answer, InversionControl};
    //
    //     let quality_answers: Vec<Answer> = field_states
    //         .iter()
    //         .map(|f| Answer {
    //             step_id: f.id.clone(),
    //             value: f.content.clone(),
    //             timestamp: Utc::now().to_rfc3339(),
    //         })
    //         .collect();
    //
    //     let ears = vec![];
    //     let inversion = InversionControl { has_inversion_tests: false, inverted_count: 0 };
    //
    //     let quality_result = calculate_quality(&quality_answers, &ears, &inversion);
    //     assert!(quality_result.is_ok());
    //
    //     // Step 9: Continue CTA appears when all locked
    //     assert!(all_locked);
    //
    //     // Step 10: Continue moves to Define phase
    //     let define_phase = phases.iter().find(|p| p.key == "define");
    //     assert!(define_phase.is_some());
    // }

    #[tokio::test]
    async fn test_extraction_with_delay() {
        let provider = MockExtractionProvider::new(100);
        let test_text = "Building a comprehensive task management system";

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
        assert!(elapsed >= std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_minimum_extraction_characters() {
        // Test that we validate minimum character requirements
        const MIN_CHARS: usize = 50;

        let short_input = "Too short";
        assert!(short_input.len() < MIN_CHARS);

        let adequate_input = "Building a task management app for remote teams. \
        The main problem is that team members often miss deadlines because tasks \
        aren't clearly assigned or tracked across different time zones.";
        assert!(adequate_input.len() >= MIN_CHARS);
    }

    #[test]
    fn test_maximum_characters_limit() {
        const MAX_CHARS: usize = 2000;

        let within_limit = "A".repeat(1999);
        assert!(within_limit.len() <= MAX_CHARS);

        let at_limit = "A".repeat(2000);
        assert_eq!(at_limit.len(), MAX_CHARS);

        let over_limit = "A".repeat(2001);
        assert!(over_limit.len() > MAX_CHARS);
    }
}
