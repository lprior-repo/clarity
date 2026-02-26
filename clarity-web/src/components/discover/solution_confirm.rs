#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::quality_score::{QualityDimension, QualityScore, QualityScoreBar};
use crate::ui::{Button, Textarea};
use crate::ui::button::ButtonVariant;

/// VORP (Value Over Replacement Product) justification fields
///
/// VORP answers four critical questions:
/// - Value: What value does this provide?
/// - Obvious: Is the benefit immediately obvious?
/// - Real: Is this solving a real problem?
/// - Possible: Is this actually buildable?
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VorpFields {
    /// What value does this solution provide to the user?
    pub value: String,
    /// Is the benefit immediately obvious to users?
    pub obvious: String,
    /// Is this solving a real, validated problem?
    pub real: String,
    /// Is this actually possible to build with available resources?
    pub possible: String,
}

impl VorpFields {
    /// Create new empty VORP fields
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create VORP fields with values
    #[must_use]
    pub fn with_values(value: String, obvious: String, real: String, possible: String) -> Self {
        Self {
            value,
            obvious,
            real,
            possible,
        }
    }

    /// Check if all VORP fields are filled
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.value.trim().is_empty()
            && !self.obvious.trim().is_empty()
            && !self.real.trim().is_empty()
            && !self.possible.trim().is_empty()
    }

    /// Get count of filled fields (0-4)
    #[must_use]
    pub fn filled_count(&self) -> usize {
        usize::from(!self.value.trim().is_empty())
            + usize::from(!self.obvious.trim().is_empty())
            + usize::from(!self.real.trim().is_empty())
            + usize::from(!self.possible.trim().is_empty())
    }

    /// Calculate overall VORP score (0-100)
    #[must_use]
    pub fn score(&self) -> u8 {
        if !self.is_complete() {
            return (self.filled_count() * 15) as u8;
        }

        // Base score for completion
        let base = 60u8;

        // Bonus for depth of answers
        let value_bonus = if self.value.split_whitespace().count() > 10 { 10 } else { 0 };
        let obvious_bonus = if self.obvious.split_whitespace().count() > 5 { 10 } else { 0 };
        let real_bonus = if self.real.split_whitespace().count() > 10 { 10 } else { 0 };
        let possible_bonus = if self.possible.split_whitespace().count() > 5 { 10 } else { 0 };

        (base + value_bonus + obvious_bonus + real_bonus + possible_bonus).min(100)
    }
}

/// Props for SolutionDisplay component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct SolutionDisplayProps {
    /// The solution text to display/edit
    pub solution: Signal<String>,
    /// Placeholder text for the textarea
    #[props(default = String::from("Describe your solution..."))]
    pub placeholder: String,
    /// Whether the solution text is editable
    #[props(default = true)]
    pub editable: bool,
}

/// SolutionDisplay component
///
/// Displays and allows editing of the solution description.
#[component]
pub fn SolutionDisplay(props: SolutionDisplayProps) -> Element {
    let solution = props.solution;
    let mut local_solution = use_signal(|| solution.read().clone());

    // Sync local solution when external signal changes
    use_effect({
        let solution = solution.clone();
        move || {
            let external = solution.read().clone();
            let local = local_solution.read().clone();
            if external != local {
                *local_solution.write() = external;
            }
        }
    });

    let on_input = {
        let mut solution = solution.clone();
        move |value: String| {
            *local_solution.write() = value.clone();
            *solution.write() = value;
        }
    };

    rsx! {
        div {
            class: "space-y-2",
            label {
                class: "text-sm font-medium text-foreground",
                "Based on what you wrote, here's the solution I see:"
            }
            Textarea {
                value: local_solution.read().clone(),
                placeholder: props.placeholder.clone(),
                disabled: !props.editable,
                rows: 4,
                oninput: on_input,
            }
        }
    }
}

/// Props for VorpInput component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct VorpInputProps {
    /// The VORP fields
    pub vorp: Signal<VorpFields>,
    /// Whether inputs are enabled
    #[props(default = true)]
    pub enabled: bool,
}

/// VorpInput component
///
/// Displays four input fields for VORP justification:
/// - Value: What value does this provide?
/// - Obvious: Is the benefit obvious?
/// - Real: Is this a real problem?
/// - Possible: Is this buildable?
#[component]
pub fn VorpInput(props: VorpInputProps) -> Element {
    let vorp = props.vorp;
    let mut local_vorp = use_signal(|| vorp.read().clone());

    // Sync local vorp when external signal changes
    use_effect({
        let vorp = vorp.clone();
        move || {
            let external = vorp.read().clone();
            let local = local_vorp.read().clone();
            if external != local {
                *local_vorp.write() = external;
            }
        }
    });

    let mut update_field = {
        let mut vorp = vorp.clone();
        move |field: &str, value: String| {
            let current = local_vorp.read().clone();
            let new_vorp = match field {
                "value" => VorpFields {
                    value,
                    ..current
                },
                "obvious" => VorpFields {
                    obvious: value,
                    ..current
                },
                "real" => VorpFields {
                    real: value,
                    ..current
                },
                "possible" => VorpFields {
                    possible: value,
                    ..current
                },
                _ => current,
            };
            *local_vorp.write() = new_vorp.clone();
            *vorp.write() = new_vorp;
        }
    };

    let vorp_labels = [
        ("value", "Value", "What value does this provide to users?"),
        ("obvious", "Obvious", "Is the benefit immediately obvious?"),
        ("real", "Real", "Is this solving a real, validated problem?"),
        ("possible", "Possible", "Is this buildable with available resources?"),
    ];

    rsx! {
        div {
            class: "space-y-3",
            label {
                class: "text-sm font-medium text-foreground",
                "Justify your solution with VORP (Value Over Replacement Product):"
            }
            div {
                class: "space-y-3 rounded-lg border border-border/50 bg-muted/20 p-4",
                p {
                    class: "text-xs text-muted-foreground mb-3",
                    "Answer these questions to validate your solution is worth building:"
                }
                for (field_key, label, description) in vorp_labels {
                    div {
                        class: "space-y-1",
                        div {
                            class: "flex items-center gap-2",
                            span {
                                class: "text-sm font-medium text-foreground",
                                "{label}"
                            }
                            span {
                                class: "text-xs text-muted-foreground",
                                "- {description}"
                            }
                        }
                        input {
                            r#type: "text",
                            value: match field_key {
                                "value" => local_vorp.read().value.clone(),
                                "obvious" => local_vorp.read().obvious.clone(),
                                "real" => local_vorp.read().real.clone(),
                                "possible" => local_vorp.read().possible.clone(),
                                _ => String::new(),
                            },
                            disabled: !props.enabled,
                            placeholder: "{description}",
                            class: "w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
                            oninput: {
                                let mut update_field = update_field.clone();
                                let field_key = field_key.to_string();
                                move |e: Event<FormData>| {
                                    update_field(&field_key, e.value());
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Props for VorpQuality component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct VorpQualityProps {
    /// The VORP fields to evaluate
    pub vorp: Signal<VorpFields>,
    /// Whether to show expanded details
    #[props(default = false)]
    pub expanded: bool,
}

/// VorpQuality component
///
/// Displays quality metrics for the VORP justification.
#[component]
pub fn VorpQuality(props: VorpQualityProps) -> Element {
    let vorp = props.vorp;

    // Calculate quality dimensions based on VORP
    let quality_score = use_memo({
        let vorp = vorp.clone();
        move || {
            let fields = vorp.read();
            let overall = fields.score();

            let value_score = calculate_field_score(&fields.value, 10);
            let obvious_score = calculate_field_score(&fields.obvious, 5);
            let real_score = calculate_field_score(&fields.real, 10);
            let possible_score = calculate_field_score(&fields.possible, 5);

            QualityScore::new(overall).with_dimensions(vec![
                QualityDimension::new("Value", value_score)
                    .with_issues(get_field_issues(&fields.value, "Value", 10)),
                QualityDimension::new("Obvious", obvious_score)
                    .with_issues(get_field_issues(&fields.obvious, "Obvious", 5)),
                QualityDimension::new("Real", real_score)
                    .with_issues(get_field_issues(&fields.real, "Real", 10)),
                QualityDimension::new("Possible", possible_score)
                    .with_issues(get_field_issues(&fields.possible, "Possible", 5)),
            ])
        }
    });

    rsx! {
        div {
            class: "mt-4",
            QualityScoreBar {
                score: quality_score,
                expanded: props.expanded,
            }
        }
    }
}

/// Calculate score for a VORP field (0-100)
fn calculate_field_score(text: &str, min_words: usize) -> u8 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }

    let word_count = trimmed.split_whitespace().count();
    if word_count < min_words {
        40
    } else if word_count < min_words * 2 {
        70
    } else {
        90
    }
}

/// Get issues for a VORP field
fn get_field_issues(text: &str, field_name: &str, min_words: usize) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return vec![format!("{field_name} is required")];
    }

    let word_count = trimmed.split_whitespace().count();
    if word_count < min_words {
        return vec![format!("{field_name} needs more detail (at least {min_words} words)")];
    }

    Vec::new()
}

/// Props for SolutionConfirm component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct SolutionConfirmProps {
    /// The solution text
    pub solution: Signal<String>,
    /// The VORP justification fields
    pub vorp: Signal<VorpFields>,
    /// Current step number (1-5)
    #[props(default = 3)]
    pub step: u8,
    /// Total steps in the confirmation flow
    #[props(default = 5)]
    pub total_steps: u8,
    /// Callback when Next is clicked
    pub on_next: Option<EventHandler<Event<MouseData>>>,
    /// Callback when Back is clicked
    pub on_back: Option<EventHandler<Event<MouseData>>>,
    /// Whether the Next button should be disabled
    #[props(default = false)]
    pub next_disabled: bool,
    /// Whether the Back button should be disabled
    #[props(default = false)]
    pub back_disabled: bool,
}

/// SolutionConfirm component
///
/// Composes:
/// - SolutionDisplay: Shows the solution text for review/editing
/// - VorpInput: Four VORP justification fields
/// - VorpQuality: Quality score indicator
/// - Navigation: Back/Next buttons
///
/// This is the third confirmation step in the Progressive Discover flow.
#[component]
pub fn SolutionConfirm(props: SolutionConfirmProps) -> Element {
    let mut quality_expanded = use_signal(|| false);

    let toggle_quality = {
        move |_| {
            quality_expanded.toggle();
        }
    };

    rsx! {
        div {
            class: "space-y-6 rounded-lg border border-border/50 bg-card p-6 shadow-sm",

            // Header with step indicator
            div {
                class: "flex items-center justify-between border-b border-border/50 pb-4",
                h2 {
                    class: "text-lg font-semibold text-foreground",
                    "Solution ({props.step}/{props.total_steps})"
                }
                span {
                    class: "text-sm text-muted-foreground",
                    "Confirm your solution and justify VORP"
                }
            }

            // Solution display section
            SolutionDisplay {
                solution: props.solution,
            }

            // VORP input section
            VorpInput {
                vorp: props.vorp,
            }

            // Quality indicator section (expandable)
            div {
                class: "cursor-pointer",
                onclick: toggle_quality,
                VorpQuality {
                    vorp: props.vorp,
                    expanded: *quality_expanded.read(),
                }
            }

            // Navigation buttons
            div {
                class: "flex items-center justify-between border-t border-border/50 pt-4",
                Button {
                    variant: ButtonVariant::Secondary,
                    disabled: props.back_disabled,
                    onclick: {
                        move |e| {
                            if let Some(handler) = &props.on_back {
                                handler.call(e);
                            }
                        }
                    },
                    // Left arrow icon
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        class: "mr-2",
                        path { d: "m15 18-6-6 6-6" }
                    }
                    "Back"
                }

                Button {
                    variant: ButtonVariant::Primary,
                    disabled: props.next_disabled,
                    onclick: {
                        move |e| {
                            if let Some(handler) = &props.on_next {
                                handler.call(e);
                            }
                        }
                    },
                    "Next"
                    // Right arrow icon
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        class: "ml-2",
                        path { d: "m9 18 6-6-6-6" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vorp_fields_default_empty() {
        let vorp = VorpFields::default();
        assert!(vorp.value.is_empty());
        assert!(vorp.obvious.is_empty());
        assert!(vorp.real.is_empty());
        assert!(vorp.possible.is_empty());
        assert!(!vorp.is_complete());
        assert_eq!(vorp.filled_count(), 0);
    }

    #[test]
    fn test_vorp_fields_is_complete() {
        let vorp = VorpFields::with_values(
            "Value text".to_string(),
            "Obvious text".to_string(),
            "Real text".to_string(),
            "Possible text".to_string(),
        );
        assert!(vorp.is_complete());
        assert_eq!(vorp.filled_count(), 4);
    }

    #[test]
    fn test_vorp_fields_partial() {
        let vorp = VorpFields {
            value: "Value".to_string(),
            obvious: String::new(),
            real: "Real".to_string(),
            possible: String::new(),
        };
        assert!(!vorp.is_complete());
        assert_eq!(vorp.filled_count(), 2);
    }

    #[test]
    fn test_vorp_score_empty() {
        let vorp = VorpFields::default();
        assert_eq!(vorp.score(), 0);
    }

    #[test]
    fn test_vorp_score_partial() {
        let vorp = VorpFields {
            value: "Value".to_string(),
            ..Default::default()
        };
        assert_eq!(vorp.score(), 15);
    }

    #[test]
    fn test_vorp_score_complete() {
        let vorp = VorpFields::with_values(
            "This is a long enough value description".to_string(),
            "Obvious benefit".to_string(),
            "This is a real problem statement".to_string(),
            "This is possible to build".to_string(),
        );
        assert!(vorp.score() >= 60);
    }

    #[test]
    fn test_calculate_field_score_empty() {
        let score = calculate_field_score("", 10);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_calculate_field_score_short() {
        let score = calculate_field_score("Short text", 10);
        assert_eq!(score, 40);
    }

    #[test]
    fn test_calculate_field_score_medium() {
        let score = calculate_field_score("This is a medium length text with enough words", 10);
        assert_eq!(score, 70);
    }

    #[test]
    fn test_calculate_field_score_long() {
        let score = calculate_field_score("This is a very long text with many more words than the minimum requirement", 5);
        assert_eq!(score, 90);
    }

    #[test]
    fn test_get_field_issues_empty() {
        let issues = get_field_issues("", "Value", 10);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("required"));
    }

    #[test]
    fn test_get_field_issues_short() {
        let issues = get_field_issues("Short", "Value", 10);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("more detail"));
    }

    #[test]
    fn test_get_field_issues_good() {
        let issues = get_field_issues("This is a good long enough description with many words", "Value", 10);
        assert!(issues.is_empty());
    }
}
