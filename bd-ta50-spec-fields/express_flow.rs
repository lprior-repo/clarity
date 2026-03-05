#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::components::discover::field_card::{Confidence, FieldData, FieldCard};
use crate::providers::{ExtractionContext, ExtractionProvider, FieldType, SchemaField};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Character limit for freeform input
const MAX_CHARS: usize = 2000;

/// Minimum characters to trigger extraction
const MIN_EXTRACTION_CHARS: usize = 50;

/// Field definitions for Express extraction
const EXPRESS_FIELDS: &[(&str, &str, FieldType)] = &[
    ("problem", "Problem Statement", FieldType::TextArea),
    ("user", "Target User", FieldType::Text),
    ("context", "Context & Background", FieldType::TextArea),
    ("constraints", "Constraints", FieldType::TextArea),
    ("goals", "Goals & Success Metrics", FieldType::TextArea),
];

/// Props for ExpressFlow component
#[derive(Clone, Props)]
pub struct ExpressFlowProps {
    /// Extraction provider for AI field extraction
    pub extraction_provider: Option<Arc<dyn ExtractionProvider>>,

    /// Callback when extraction is complete and user confirms
    pub on_complete: Callback<Vec<FieldData>>,

    /// Initial content to pre-fill textarea (for mode switching from Guided)
    #[props(default)]
    pub initial_content: Option<String>,
}

impl PartialEq for ExpressFlowProps {
    fn eq(&self, other: &Self) -> bool {
        // Cannot compare Arc<dyn ExtractionProvider> or Callback,
        // so we compare only what we can
        self.initial_content == other.initial_content
    }
}
impl PartialEq for ExpressFlowProps {
    fn eq(&self, _other: &Self) -> bool {
        // Cannot compare Arc<dyn ExtractionProvider> or Callback,
        // so we assume props are equal if they refer to the same component instance
        false
    }
}

/// ExpressFlow component
///
/// Freeform input with AI-powered field extraction:
/// - Large textarea (2000 char soft limit)
/// - Guided placeholder with example
/// - "Extract Details" button
/// - 5 FieldCard components below
/// - Confirm All button
/// - Continue to Define CTA
#[component]
pub fn ExpressFlow(props: ExpressFlowProps) -> Element {
    let input_text = use_signal(|| match props.initial_content.clone() {
        Some(content) => content,
        None => String::new(),
    });
    let field_list = use_signal(initialize_fields);
    let extraction_trigger = use_signal(|| None as Option<String>);
    let show_continue_cta = use_signal(|| false);

    // Create individual signals for each field
    let field_0 = use_signal(|| FieldData::default());
    let field_1 = use_signal(|| FieldData::default());
    let field_2 = use_signal(|| FieldData::default());
    let field_3 = use_signal(|| FieldData::default());
    let field_4 = use_signal(|| FieldData::default());

    // Initialize field signals from field_list
    use_effect({
        let field_list_clone = field_list.clone();
        let mut field_0_clone = field_0.clone();
        let mut field_1_clone = field_1.clone();
        let mut field_2_clone = field_2.clone();
        let mut field_3_clone = field_3.clone();
        let mut field_4_clone = field_4.clone();

        move || {
            let fields = field_list_clone.read();
            if let Some(f) = fields.get(0) {
                let mut field = field_0_clone.write();
                *field = f.clone();
            }
            if let Some(f) = fields.get(1) {
                let mut field = field_1_clone.write();
                *field = f.clone();
            }
            if let Some(f) = fields.get(2) {
                let mut field = field_2_clone.write();
                *field = f.clone();
            }
            if let Some(f) = fields.get(3) {
                let mut field = field_3_clone.write();
                *field = f.clone();
            }
            if let Some(f) = fields.get(4) {
                let mut field = field_4_clone.write();
                *field = f.clone();
            }
        }
    });

    // Extraction resource that runs when trigger changes
    let extraction_result = use_resource(move || {
        let trigger = extraction_trigger.read().clone();
        let provider = props.extraction_provider.clone();

        async move {
            let text = match trigger {
                Some(t) => t,
                None => return None,
            };

            let provider = match provider {
                Some(p) => p,
                None => return None,
            };

            let schema = EXPRESS_FIELDS
                .iter()
                .map(|(id, title, field_type)| SchemaField {
                    name: id.to_string(),
                    field_type: field_type.clone(),
                    required: false,
                    description: Some(title.to_string()),
                    options: None,
                })
                .collect::<Vec<_>>();

            let context = ExtractionContext {
                document_type: Some("express_flow_input".to_string()),
                locale: Some("en_US".to_string()),
                schema: Some(schema),
                extra: serde_json::json!({}),
            };

            match provider.extract_fields(&text, &context).await {
                Ok(extracted) => Some(extracted),
                Err(_) => None,
            }
        }
    });

    // Update fields when extraction completes
    use_effect({
        let extraction_result = extraction_result.clone();
        // Clone the signals for use in the closure
        // Each clone can be independently mutated
        let mut field_0_clone = field_0.clone();
        let mut field_1_clone = field_1.clone();
        let mut field_2_clone = field_2.clone();
        let mut field_3_clone = field_3.clone();
        let mut field_4_clone = field_4.clone();

        move || {
            if let Some(Some(extracted)) = extraction_result.read().as_ref() {
                // Build a map of field updates
                let updates: HashMap<String, (String, Confidence)> = extracted
                    .fields
                    .iter()
                    .map(|field_extraction| {
                        let value_str = match &field_extraction.value {
                            serde_json::Value::String(s) => s.clone(),
                            v => v.to_string(),
                        };
                        let confidence = confidence_from_score(field_extraction.confidence);
                        (field_extraction.name.clone(), (value_str, confidence))
                    })
                    .collect();

                // Apply updates sequentially (one mutable borrow at a time)
                if let Some((value, confidence)) = updates.get("problem") {
                    let mut field = field_0_clone.write();
                    field.content = value.clone();
                    field.confidence = *confidence;
                }

                if let Some((value, confidence)) = updates.get("user") {
                    let mut field = field_1_clone.write();
                    field.content = value.clone();
                    field.confidence = *confidence;
                }

                if let Some((value, confidence)) = updates.get("context") {
                    let mut field = field_2_clone.write();
                    field.content = value.clone();
                    field.confidence = *confidence;
                }

                if let Some((value, confidence)) = updates.get("constraints") {
                    let mut field = field_3_clone.write();
                    field.content = value.clone();
                    field.confidence = *confidence;
                }

                if let Some((value, confidence)) = updates.get("goals") {
                    let mut field = field_4_clone.write();
                    field.content = value.clone();
                    field.confidence = *confidence;
                }
            }
        }
    });

    // Manual extraction trigger
    let on_extract_details = {
        let mut extraction_trigger = extraction_trigger.clone();

        move |_| {
            let text = input_text.read().clone();
            if !text.trim().is_empty() && text.len() >= MIN_EXTRACTION_CHARS {
                *extraction_trigger.write() = Some(text);
            }
        }
    };

    // Input change handler
    let on_input_change = {
        move |e: Event<FormData>| {
            let text = e.value();
            *input_text.write() = text;
        }
    };

    // Confirm all fields
    let on_confirm_all = {
        let mut field_0 = field_0.clone();
        let mut field_1 = field_1.clone();
        let mut field_2 = field_2.clone();
        let mut field_3 = field_3.clone();
        let mut field_4 = field_4.clone();
        let mut show_continue_cta = show_continue_cta.clone();
        let on_complete = props.on_complete.clone();

        move |_| {
            let confirmed_fields = vec![
                field_0.read().clone(),
                field_1.read().clone(),
                field_2.read().clone(),
                field_3.read().clone(),
                field_4.read().clone(),
            ];
            *show_continue_cta.write() = true;
            on_complete.call(confirmed_fields);
        }
    };

    // Continue to Define phase
    let on_continue = {
        move |_| {
            // Navigate to Define phase
            // TODO: Implement navigation when routing is set up
        }
    };

    let input_len = input_text.read().len();
    let has_text = input_len > 0;
    let is_extracting = extraction_result.read().is_some();
    let extraction_complete = extraction_result.read().as_ref().is_some_and(|r| r.is_some());

    rsx! {
        div {
            class: "flex w-full flex-col gap-6",

            // Input section
            div {
                class: "flex flex-col gap-3",

                // Textarea with character counter
                div {
                    class: "relative",

                    textarea {
                        value: "{input_text.read()}",
                        oninput: on_input_change,
                        placeholder: get_placeholder_text(),
                        rows: "8",
                        maxlength: "{MAX_CHARS}",
                        class: "w-full resize-none rounded-lg border border-border/50 bg-background px-4 py-3 text-sm text-foreground placeholder:text-muted-foreground/50 focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-ring/20 transition-all",
                    }

                    // Character counter
                    div {
                        class: "absolute bottom-3 right-3 text-xs text-muted-foreground/60 font-mono",
                        "{input_len}/{MAX_CHARS}"
                    }
                }

                // Extract button
                button {
                    "type": "button",
                    onclick: on_extract_details,
                    disabled: !has_text || is_extracting,
                    class: format!(
                        "inline-flex items-center justify-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed {}",
                        if is_extracting {
                            "bg-muted text-muted-foreground"
                        } else {
                            "bg-primary text-primary-foreground hover:bg-primary/90"
                        }
                    ),

                    if is_extracting {
                        // Loading spinner
                        svg {
                            class: "h-4 w-4 animate-spin",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            circle {
                                class: "opacity-25",
                                cx: "12",
                                cy: "12",
                                r: "10",
                                stroke: "currentColor",
                                stroke_width: "4"
                            }
                            path {
                                class: "opacity-75",
                                fill: "currentColor",
                                d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            }
                        }
                        "Extracting..."
                    } else {
                        // Sparkles icon
                        svg {
                            class: "h-4 w-4",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z" }
                        }
                        "Extract Details"
                    }
                }
            }

            // Field cards section
            if extraction_complete {
                div {
                    class: "flex flex-col gap-4",

                    div {
                        class: "flex items-center justify-between",
                        h3 {
                            class: "text-sm font-semibold text-foreground",
                            "Extracted Fields"
                        }
                        span {
                            class: "text-xs text-muted-foreground",
                            "Review and edit as needed"
                        }
                    }

                    // Field cards grid
                    div {
                        class: "grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3",

                        // Field 0: Problem Statement
                        FieldCard {
                            field: field_0.clone(),
                            on_edit: None,
                        }

                        // Field 1: Target User
                        FieldCard {
                            field: field_1.clone(),
                            on_edit: None,
                        }

                        // Field 2: Context & Background
                        FieldCard {
                            field: field_2.clone(),
                            on_edit: None,
                        }

                        // Field 3: Constraints
                        FieldCard {
                            field: field_3.clone(),
                            on_edit: None,
                        }

                        // Field 4: Goals & Success Metrics
                        FieldCard {
                            field: field_4.clone(),
                            on_edit: None,
                        }
                    }

                    // Confirm All button
                    button {
                        "type": "button",
                        onclick: on_confirm_all,
                        class: "w-full rounded-md border border-primary/50 bg-primary/10 px-4 py-3 text-sm font-medium text-primary hover:bg-primary/20 transition-colors",
                        svg {
                            class: "mr-2 inline h-4 w-4",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M5 13l4 4L19 7" }
                        }
                        "Confirm All Fields"
                    }
                }
            }

            // Continue CTA (appears after confirmation)
            if *show_continue_cta.read() {
                div {
                    class: "rounded-lg border border-border/50 bg-muted/30 px-6 py-4",

                    div {
                        class: "flex items-center justify-between",

                        div {
                            class: "flex flex-col gap-1",
                            h4 {
                                class: "text-sm font-semibold text-foreground",
                                "Ready to Define?"
                            }
                            p {
                                class: "text-xs text-muted-foreground",
                                "Continue to the Define phase to refine your problem statement"
                            }
                        }

                        button {
                            "type": "button",
                            onclick: on_continue,
                            class: "inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors",
                            "Continue to Define"
                            svg {
                                class: "h-4 w-4",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M5 12h14M12 5l7 7-7 7" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Initialize empty field data
fn initialize_fields() -> Vec<FieldData> {
    EXPRESS_FIELDS
        .iter()
        .map(|(id, title, _)| FieldData {
            id: id.to_string(),
            title: title.to_string(),
            content: String::new(),
            confidence: Confidence::Low,
            locked: false,
        })
        .collect()
}

/// Convert confidence score (0-1) to Confidence enum
fn confidence_from_score(score: f64) -> Confidence {
    if score >= 0.8 {
        Confidence::High
    } else if score >= 0.5 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

/// Get guided placeholder text with example
fn get_placeholder_text() -> &'static str {
    r#"Describe your idea or scenario in your own words. For example:

"I'm building a task management app for remote teams. The main problem is that team members often miss deadlines because tasks aren't clearly assigned or tracked across different time zones. We need a way to see who's working on what, when it's due, and be able to quickly reassign if someone is overloaded."

The more detail you provide, the better the extraction will work."#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_fields() {
        let fields = initialize_fields();

        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0].id, "problem");
        assert_eq!(fields[0].title, "Problem Statement");
        assert_eq!(fields[1].id, "user");
        assert_eq!(fields[1].title, "Target User");
        assert_eq!(fields[2].id, "context");
        assert_eq!(fields[2].title, "Context & Background");
        assert_eq!(fields[3].id, "constraints");
        assert_eq!(fields[3].title, "Constraints");
        assert_eq!(fields[4].id, "goals");
        assert_eq!(fields[4].title, "Goals & Success Metrics");
    }

    #[test]
    fn test_confidence_from_score() {
        assert_eq!(confidence_from_score(0.9), Confidence::High);
        assert_eq!(confidence_from_score(0.8), Confidence::High);
        assert_eq!(confidence_from_score(0.7), Confidence::Medium);
        assert_eq!(confidence_from_score(0.5), Confidence::Medium);
        assert_eq!(confidence_from_score(0.3), Confidence::Low);
        assert_eq!(confidence_from_score(0.0), Confidence::Low);
    }

    #[test]
    fn test_confidence_from_score_boundaries() {
        // Test boundary conditions
        assert_eq!(confidence_from_score(0.80), Confidence::High);
        assert_eq!(confidence_from_score(0.79), Confidence::Medium);
        assert_eq!(confidence_from_score(0.50), Confidence::Medium);
        assert_eq!(confidence_from_score(0.49), Confidence::Low);
    }

    #[test]
    fn test_get_placeholder_text() {
        let placeholder = get_placeholder_text();

        assert!(placeholder.contains("task management"));
        assert!(placeholder.contains("remote teams"));
        assert!(placeholder.contains("Describe your idea"));
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
    fn test_max_chars_constant() {
        assert_eq!(MAX_CHARS, 2000);
    }

    #[test]
    fn test_min_extraction_chars_constant() {
        assert_eq!(MIN_EXTRACTION_CHARS, 50);
    }

    #[test]
    fn test_field_data_initialization() {
        let fields = initialize_fields();

        for field in &fields {
            assert!(!field.id.is_empty());
            assert!(!field.title.is_empty());
            assert!(field.content.is_empty());
            assert_eq!(field.confidence, Confidence::Low);
            assert!(!field.locked);
        }
    }

    #[test]
    fn test_express_flow_props_default() {
        // Test that props can be created with None provider
        let props = ExpressFlowProps {
            extraction_provider: None,
            on_complete: Callback::new(|_| {}),
        };

        assert!(props.extraction_provider.is_none());
    }

    #[test]
    fn test_confidence_levels_coverage() {
        // Ensure all confidence levels are reachable
        let high = confidence_from_score(1.0);
        let medium = confidence_from_score(0.6);
        let low = confidence_from_score(0.2);

        assert_eq!(high, Confidence::High);
        assert_eq!(medium, Confidence::Medium);
        assert_eq!(low, Confidence::Low);
    }
}
