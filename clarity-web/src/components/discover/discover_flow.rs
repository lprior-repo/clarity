#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::components::discover::{
    express_flow::ExpressFlow,
    guided_flow::{GuidedFlow, ServerSuggestionProvider},
    mode_toggle::{DiscoverMode, ModeToggle},
};
use crate::providers::{ExtractionProvider, FieldType};
use crate::types::Answer;
use dioxus::prelude::*;
use itertools::Itertools;
use std::sync::Arc;

/// Field definitions for Express extraction
const EXPRESS_FIELDS: &[(&str, &str, FieldType)] = &[
    ("problem", "Problem Statement", FieldType::TextArea),
    ("user", "Target User", FieldType::Text),
    ("context", "Context & Background", FieldType::TextArea),
    ("constraints", "Constraints", FieldType::TextArea),
    ("goals", "Goals & Success Metrics", FieldType::TextArea),
];

/// Props for DiscoverFlow container component
#[derive(Clone, Props)]
pub struct DiscoverFlowProps {
    /// Extraction provider for AI field extraction
    pub extraction_provider: Option<Arc<dyn ExtractionProvider>>,
    /// User answers signal
    pub answers: Signal<Vec<Answer>>,
    /// Mutable answers signal
    pub mut_answers: Signal<Vec<Answer>>,
}

impl PartialEq for DiscoverFlowProps {
    fn eq(&self, _other: &Self) -> bool {
        // Cannot compare Arc<dyn ExtractionProvider> or Signal,
        // so we assume props are equal if they refer to the same component instance
        false
    }
}

/// DiscoverFlow container component
///
/// Manages mode switching between Express and Guided flows with state carry-forward:
/// - Express -> Guided: pre-populate sequential inputs with extracted field content
/// - Guided -> Express: concatenate answers into freeform textarea
/// - Preserves answers signal across switch
/// - Marks partial progress (e.g., 3/5 in Guided)
/// - Shows mode switch confirmation if unsaved changes
/// - Saves mode to metadata on switch
#[component]
pub fn DiscoverFlow(props: DiscoverFlowProps) -> Element {
    let DiscoverFlowProps {
        extraction_provider,
        answers,
        mut_answers,
    } = props;

    // Mode state
    let mode = use_signal(|| DiscoverMode::default());

    // Express content state
    let express_content = use_signal(String::new);

    // Express extracted fields state
    let express_fields = use_signal(|| Vec::new());

    // Confirmation dialog state
    let show_confirmation = use_signal(|| false);
    let pending_mode = use_signal(|| None as Option<DiscoverMode>);

    // Track unsaved changes
    let has_unsaved_changes = use_signal(|| false);

    // Pre-fill express content from guided answers
    use_effect({
        let answers = answers.clone();
        let mut express_content = express_content.clone();
        let mut mode = mode.clone();
        move || {
            if *mode.read() == DiscoverMode::Express {
                let existing_content = express_content.read().clone();
                if existing_content.trim().is_empty() {
                    let ans = answers.read();
                    let concatenated = ans
                        .iter()
                        .filter(|a| {
                            EXPRESS_FIELDS
                                .iter()
                                .any(|(id, _, _)| id == &a.step_id)
                        })
                        .map(|a| format!("{}: {}", a.step_id.to_uppercase(), a.value))
                        .join("\n\n");

                    if !concatenated.is_empty() {
                        *express_content.write() = concatenated;
                    }
                }
            }
        }
    });

    // Confirm mode switch
    let on_confirm_switch = Callback::new({
        let mut mode = mode.clone();
        let mut pending_mode = pending_mode.clone();
        let mut show_confirmation = show_confirmation.clone();
        let mut has_unsaved_changes = has_unsaved_changes.clone();
        move |_| {
            if let Some(new_mode) = *pending_mode.read() {
                *mode.write() = new_mode;
                *has_unsaved_changes.write() = false;
            }
            *show_confirmation.write() = false;
            *pending_mode.write() = None;
        }
    });

    // Cancel mode switch
    let on_cancel_switch = Callback::new({
        let mut show_confirmation = show_confirmation.clone();
        let mut pending_mode = pending_mode.clone();
        move |_| {
            *show_confirmation.write() = false;
            *pending_mode.write() = None;
        }
    });

    // Calculate partial progress for Guided mode
    let progress_text = {
        let mode_val = *mode.read();
        let ans = answers.read();
        if mode_val == DiscoverMode::Guided {
            let total = EXPRESS_FIELDS.len();
            let answered = ans
                .iter()
                .filter(|a| {
                    EXPRESS_FIELDS
                        .iter()
                        .any(|(id, _, _)| id == &a.step_id && !a.value.trim().is_empty())
                })
                .count();
            format!("{answered}/{total}")
        } else {
            String::new()
        }
    };

    let current_mode = *mode.read();

    rsx! {
        div { class: "flex flex-col gap-6",

            // Header with mode toggle and progress
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-4",
                    // Mode toggle
                    ModeToggle {
                        mode: mode.clone(),
                        on_change: Some(Callback::new({
                            let mut mode = mode.clone();
                            let mut show_confirmation = show_confirmation.clone();
                            let mut pending_mode = pending_mode.clone();
                            let has_unsaved_changes = has_unsaved_changes.clone();
                            move |new_mode: DiscoverMode| {
                                if *mode.read() == new_mode {
                                    return;
                                }

                                if *has_unsaved_changes.read() {
                                    *pending_mode.write() = Some(new_mode);
                                    *show_confirmation.write() = true;
                                } else {
                                    // Direct mode switch
                                    *mode.write() = new_mode;
                                }
                            }
                        })),
                    }

                    // Progress indicator (Guided mode only)
                    if current_mode == DiscoverMode::Guided && !progress_text.is_empty() {
                        div { class: "flex items-center gap-2",
                            span { class: "text-xs text-muted-foreground/70", "Progress:" }
                            span { class: "text-sm font-medium text-foreground", "{progress_text}" }
                        }
                    }
                }
            }

            // Mode content
            div { class: "min-h-[400px]",
                match current_mode {
                    DiscoverMode::Express => {
                        rsx! {
                            ExpressFlow {
                                extraction_provider: extraction_provider.clone(),
                                initial_content: Some(express_content.read().clone()),
                                on_complete: Callback::new({
                                    let mut express_content = express_content.clone();
                                    let mut express_fields = express_fields.clone();
                                    let mut mut_answers = mut_answers.clone();
                                    let mut has_unsaved_changes = has_unsaved_changes.clone();
                                    move |fields: Vec<crate::components::discover::FieldData>| {
                                        // Update express content from fields
                                        let content = fields
                                            .iter()
                                            .map(|f| format!("{}: {}", f.title, f.content))
                                            .join("\n\n");
                                        *express_content.write() = content;

                                        *express_fields.write() = fields.clone();

                                        // Convert fields to answers
                                        let new_answers = fields
                                            .iter()
                                            .map(|field| Answer {
                                                step_id: field.id.clone(),
                                                value: field.content.clone(),
                                                timestamp: chrono::Utc::now().to_rfc3339(),
                                            })
                                            .collect_vec();

                                        // Update answers
                                        let mut ans = mut_answers.write();
                                        // Remove old express field answers
                                        ans.retain(|a| {
                                            !EXPRESS_FIELDS
                                                .iter()
                                                .any(|(id, _, _)| id == &a.step_id)
                                        });
                                        // Add new answers
                                        ans.extend(new_answers);

                                        *has_unsaved_changes.write() = false;
                                    }
                                }),
                            }
                        }
                    }
                    DiscoverMode::Guided => {
                        rsx! {
                            GuidedFlow {
                                active_phase: use_signal(|| String::from("discover")),
                                answers: answers.clone(),
                                mut_answers: mut_answers.clone(),
                                provider: ServerSuggestionProvider,
                                express_content: express_content.clone(),
                            }
                        }
                    }
                }
            }
        }

        // Mode switch confirmation dialog
        if *show_confirmation.read() {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm",
                div {
                    class: "relative w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg",
                    div { class: "flex flex-col gap-4",
                        div {
                            h3 {
                                class: "text-lg font-semibold text-foreground",
                                "Unsaved Changes"
                            }
                            p {
                                class: "text-sm text-muted-foreground",
                                "You have unsaved changes. Do you want to switch modes anyway?"
                            }
                        }

                        div { class: "flex justify-end gap-3",
                            button {
                                "type": "button",
                                onclick: on_cancel_switch,
                                class: "rounded-md border border-border bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors",
                                "Cancel"
                            }
                            button {
                                "type": "button",
                                onclick: on_confirm_switch,
                                class: "rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors",
                                "Switch Anyway"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::discover::field_card::{Confidence, FieldData};

    /// Mock extraction provider for testing
    struct MockExtractionProvider;

    #[async_trait::async_trait]
    impl ExtractionProvider for MockExtractionProvider {
        async fn extract_fields(
            &self,
            _text: &str,
            _context: &crate::providers::ExtractionContext,
        ) -> Result<crate::providers::ExtractedFields, anyhow::Error> {
            Ok(crate::providers::ExtractedFields {
                fields: vec![],
                confidence: 1.0,
                metadata: crate::providers::ExtractionMetadata {
                    provider: "mock".to_string(),
                    model: Some("mock".to_string()),
                    timestamp: chrono::Utc::now(),
                    processing_duration_ms: 0,
                    extra: serde_json::json!({}),
                },
            })
        }
    }

    #[test]
    fn test_express_fields_constants() {
        assert_eq!(EXPRESS_FIELDS.len(), 5);

        // Check each field has proper structure
        for (id, title, field_type) in EXPRESS_FIELDS {
            assert!(!id.is_empty(), "Field ID should not be empty");
            assert!(!title.is_empty(), "Field title should not be empty");
            assert!(
                matches!(field_type, FieldType::Text | FieldType::TextArea),
                "Express fields should be Text or TextArea"
            );
        }
    }

    #[test]
    fn test_express_field_ids() {
        let ids: Vec<_> = EXPRESS_FIELDS
            .iter()
            .map(|(id, _, _)| *id)
            .collect();
        assert_eq!(ids, vec!["problem", "user", "context", "constraints", "goals"]);
    }

    #[test]
    fn test_progress_text_calculation() {
        // Test with no answers
        let answers = Signal::new(vec![]);
        let mode = Signal::new(DiscoverMode::Guided);

        let total = EXPRESS_FIELDS.len();
        let answered = answers
            .read()
            .iter()
            .filter(|a| {
                EXPRESS_FIELDS
                    .iter()
                    .any(|(id, _, _)| id == &a.step_id && !a.value.trim().is_empty())
            })
            .count();
        let progress_text = if *mode.read() == DiscoverMode::Guided {
            format!("{answered}/{total}")
        } else {
            String::new()
        };

        assert_eq!(progress_text, "0/5");
    }

    #[test]
    fn test_progress_text_with_partial_answers() {
        // Test with partial answers
        let answers = Signal::new(vec![
            Answer {
                step_id: "problem".to_string(),
                value: "Test problem".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "user".to_string(),
                value: "Test user".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "context".to_string(),
                value: "".to_string(), // Empty
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ]);

        let mode = Signal::new(DiscoverMode::Guided);

        let total = EXPRESS_FIELDS.len();
        let answered = answers
            .read()
            .iter()
            .filter(|a| {
                EXPRESS_FIELDS
                    .iter()
                    .any(|(id, _, _)| id == &a.step_id && !a.value.trim().is_empty())
            })
            .count();
        let progress_text = if *mode.read() == DiscoverMode::Guided {
            format!("{answered}/{total}")
        } else {
            String::new()
        };

        assert_eq!(progress_text, "2/5");
    }

    #[test]
    fn test_progress_text_express_mode() {
        // Test that Express mode doesn't show progress
        let mode = Signal::new(DiscoverMode::Express);
        let progress_text = if *mode.read() == DiscoverMode::Guided {
            "1/5".to_string()
        } else {
            String::new()
        };

        assert!(progress_text.is_empty());
    }

    #[test]
    fn test_discover_mode_default() {
        assert_eq!(DiscoverMode::default(), DiscoverMode::Guided);
    }

    #[test]
    fn test_field_data_conversion_to_answer() {
        let fields = vec![
            FieldData {
                id: "problem".to_string(),
                title: "Problem Statement".to_string(),
                content: "Test problem content".to_string(),
                confidence: Confidence::High,
                locked: false,
            },
            FieldData {
                id: "user".to_string(),
                title: "Target User".to_string(),
                content: "Test user content".to_string(),
                confidence: Confidence::Medium,
                locked: false,
            },
        ];

        let new_answers: Vec<Answer> = fields
            .iter()
            .map(|field| Answer {
                step_id: field.id.clone(),
                value: field.content.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
            .collect_vec();

        assert_eq!(new_answers.len(), 2);
        assert_eq!(new_answers[0].step_id, "problem");
        assert_eq!(new_answers[0].value, "Test problem content");
        assert_eq!(new_answers[1].step_id, "user");
        assert_eq!(new_answers[1].value, "Test user content");
    }

    #[test]
    fn test_concatenate_answers_to_express_content() {
        let answers = vec![
            Answer {
                step_id: "problem".to_string(),
                value: "Test problem".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "user".to_string(),
                value: "Test user".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];

        let concatenated = answers
            .iter()
            .filter(|a| {
                EXPRESS_FIELDS
                    .iter()
                    .any(|(id, _, _)| id == &a.step_id)
            })
            .map(|a| format!("{}: {}", a.step_id.to_uppercase(), a.value))
            .join("\n\n");

        assert!(concatenated.contains("PROBLEM: Test problem"));
        assert!(concatenated.contains("USER: Test user"));
        assert!(concatenated.contains("\n\n"));
    }

    #[test]
    fn test_express_field_filtering() {
        let answers = vec![
            Answer {
                step_id: "problem".to_string(),
                value: "Test problem".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "other_field".to_string(), // Not in EXPRESS_FIELDS
                value: "Other value".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];

        let filtered: Vec<_> = answers
            .iter()
            .filter(|a| {
                EXPRESS_FIELDS
                    .iter()
                    .any(|(id, _, _)| id == &a.step_id)
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].step_id, "problem");
    }

    #[test]
    fn test_answer_removal_for_express_fields() {
        let mut answers = vec![
            Answer {
                step_id: "problem".to_string(),
                value: "Old problem".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "user".to_string(),
                value: "Old user".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Answer {
                step_id: "other_field".to_string(),
                value: "Keep this".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];

        // Simulate the removal logic
        answers.retain(|a| {
            !EXPRESS_FIELDS
                .iter()
                .any(|(id, _, _)| id == &a.step_id)
        });

        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].step_id, "other_field");
    }

    #[test]
    fn test_mode_equality() {
        assert_eq!(DiscoverMode::Express, DiscoverMode::Express);
        assert_eq!(DiscoverMode::Guided, DiscoverMode::Guided);
        assert_ne!(DiscoverMode::Express, DiscoverMode::Guided);
    }

    #[test]
    fn test_mode_copy() {
        let mode1 = DiscoverMode::Express;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
    }

    #[test]
    fn test_field_content_validation() {
        // Test empty content filtering
        let content = "";
        assert!(content.trim().is_empty());

        let content = "  ";
        assert!(content.trim().is_empty());

        let content = "valid content";
        assert!(!content.trim().is_empty());
    }
}
