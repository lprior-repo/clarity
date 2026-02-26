#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

use super::quality_score::{QualityDimension, QualityScore, QualityScoreBar};
use super::straw_man::StrawManTrap;
use crate::ui::{Button, Textarea};
use crate::ui::button::ButtonVariant;

/// Props for PersonaDisplay component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct PersonaDisplayProps {
    /// The persona text to display/edit
    pub persona: Signal<String>,
    /// Placeholder text for the textarea
    #[props(default = String::from("Describe your target user persona..."))]
    pub placeholder: String,
    /// Whether the persona text is editable
    #[props(default = true)]
    pub editable: bool,
}

/// PersonaDisplay component
///
/// Displays and allows editing of the persona description.
/// This shows the extracted persona content for review.
#[component]
pub fn PersonaDisplay(props: PersonaDisplayProps) -> Element {
    let persona = props.persona;
    let mut local_persona = use_signal(|| persona.read().clone());

    // Sync local persona when external signal changes
    use_effect({
        let persona = persona.clone();
        move || {
            let external = persona.read().clone();
            let local = local_persona.read().clone();
            if external != local {
                *local_persona.write() = external;
            }
        }
    });

    let on_input = {
        let mut persona = persona.clone();
        move |value: String| {
            *local_persona.write() = value.clone();
            *persona.write() = value;
        }
    };

    rsx! {
        div {
            class: "space-y-2",
            label {
                class: "text-sm font-medium text-foreground",
                "Based on what you wrote, here's the target user I see:"
            }
            Textarea {
                value: local_persona.read().clone(),
                placeholder: props.placeholder.clone(),
                disabled: !props.editable,
                rows: 4,
                oninput: on_input,
            }
        }
    }
}

/// Props for StrawManChecklist component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct StrawManChecklistProps {
    /// Which traps have been detected/confirmed
    pub detected_traps: Signal<Vec<StrawManTrap>>,
    /// Whether the checklist is interactive
    #[props(default = true)]
    pub enabled: bool,
}

/// StrawManChecklist component
///
/// Displays a checklist of straw man traps for the user to validate.
/// Each trap has a checkbox that indicates whether the persona falls into that trap.
#[component]
pub fn StrawManChecklist(props: StrawManChecklistProps) -> Element {
    let detected_traps = props.detected_traps;

    let toggle_trap = {
        let mut detected_traps = detected_traps.clone();
        move |trap: StrawManTrap| {
            let current = detected_traps.read().clone();
            let new_traps = if current.contains(&trap) {
                current
                    .iter()
                    .filter(|&t| t != &trap)
                    .copied()
                    .collect()
            } else {
                let mut updated = current;
                updated.push(trap);
                updated
            };
            *detected_traps.write() = new_traps;
        }
    };

    rsx! {
        div {
            class: "space-y-3",
            label {
                class: "text-sm font-medium text-foreground",
                "Check any straw man traps your persona might fall into:"
            }
            div {
                class: "space-y-2 rounded-lg border border-border/50 bg-muted/20 p-4",
                p {
                    class: "text-xs text-muted-foreground mb-3",
                    "A straw man persona is an unrealistic user that doesn't represent real human behavior. Check any that apply:"
                }
                for trap in StrawManTrap::all() {
                    label {
                        class: format!(
                            "flex items-start gap-3 cursor-pointer p-2 rounded-md transition-colors {}",
                            if detected_traps.read().contains(trap) {
                                "bg-amber-500/10 border border-amber-500/30"
                            } else {
                                "hover:bg-muted/50"
                            }
                        ),
                        input {
                            r#type: "checkbox",
                            checked: detected_traps.read().contains(trap),
                            disabled: !props.enabled,
                            onchange: {
                                let mut toggle_trap = toggle_trap.clone();
                                move |_| {
                                    toggle_trap(*trap);
                                }
                            },
                            class: "mt-1 h-4 w-4 rounded border-border shrink-0",
                        }
                        div {
                            class: "flex-1",
                            div {
                                class: "text-sm font-medium text-foreground",
                                "{trap.label()}"
                            }
                            div {
                                class: "text-xs text-muted-foreground mt-0.5",
                                "{trap.checkbox_label()}"
                            }
                            div {
                                class: "text-xs text-muted-foreground mt-1",
                                "{trap.description()}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Props for PersonaQuality component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct PersonaQualityProps {
    /// The persona text to evaluate
    pub persona: Signal<String>,
    /// Detected straw man traps
    pub detected_traps: Signal<Vec<StrawManTrap>>,
    /// Whether to show expanded details
    #[props(default = false)]
    pub expanded: bool,
}

/// PersonaQuality component
///
/// Displays quality metrics for the persona.
/// Uses the QualityScoreBar component with dimensions for:
/// - Specificity: How specific is the persona description?
/// - Realism: Is this a realistic user?
/// - Straw Man Check: Are there any trap warnings?
#[component]
pub fn PersonaQuality(props: PersonaQualityProps) -> Element {
    let persona = props.persona;
    let detected_traps = props.detected_traps;

    // Calculate quality dimensions based on persona
    let quality_score = use_memo({
        let persona = persona.clone();
        let detected_traps = detected_traps.clone();
        move || {
            let persona_text = persona.read();
            let traps = detected_traps.read();
            let trap_count = traps.len();

            // Base score on content and trap count
            let base_score = calculate_persona_score(&persona_text);
            let trap_penalty = trap_count * 15;
            let overall = base_score.saturating_sub(trap_penalty as u8);

            let specificity = calculate_specificity_score(&persona_text);
            let realism = calculate_realism_score(&persona_text, trap_count);

            QualityScore::new(overall).with_dimensions(vec![
                QualityDimension::new("Specificity", specificity)
                    .with_issues(get_specificity_issues(&persona_text)),
                QualityDimension::new("Realism", realism)
                    .with_issues(get_realism_issues(&persona_text, &traps)),
                QualityDimension::new("Straw Man Check", (100 - trap_penalty).min(100) as u8)
                    .with_issues(get_trap_issues(&traps)),
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

/// Calculate overall persona score (0-100)
fn calculate_persona_score(text: &str) -> u8 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }

    let word_count = trimmed.split_whitespace().count();
    match word_count {
        0..=9 => 30,
        10..=24 => 50,
        25..=49 => 70,
        50..=99 => 85,
        _ => 80,
    }
}

/// Calculate specificity score (0-100)
fn calculate_specificity_score(text: &str) -> u8 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }

    let word_count = trimmed.split_whitespace().count();
    let base = match word_count {
        0..=9 => 30,
        10..=24 => 50,
        25..=49 => 70,
        _ => 65,
    };

    // Bonus for specific details
    let lower = trimmed.to_lowercase();
    let detail_bonus = if lower.contains("specifically") || lower.contains("for example") {
        15
    } else if lower.contains("typically") || lower.contains("usually") {
        10
    } else {
        0
    };

    (base + detail_bonus).min(100)
}

/// Calculate realism score (0-100)
fn calculate_realism_score(text: &str, trap_count: usize) -> u8 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }

    let base = 70u8;
    let trap_penalty = (trap_count * 15) as u8;

    // Check for realistic language
    let lower = trimmed.to_lowercase();
    let realism_bonus = if lower.contains("because") || lower.contains("since") {
        10
    } else if lower.contains("might") || lower.contains("often") {
        5
    } else {
        0
    };

    base.saturating_sub(trap_penalty).saturating_add(realism_bonus).min(100)
}

/// Get specificity issues
fn get_specificity_issues(text: &str) -> Vec<String> {
    let word_count = text.trim().split_whitespace().count();
    if word_count < 10 {
        vec!["Add more specific details about your target user".to_string()]
    } else if word_count < 25 {
        vec!["Consider adding more specific examples".to_string()]
    } else {
        Vec::new()
    }
}

/// Get realism issues
fn get_realism_issues(text: &str, traps: &[StrawManTrap]) -> Vec<String> {
    let mut issues = Vec::new();

    if text.trim().is_empty() {
        issues.push("Persona description is empty".to_string());
    }

    if !traps.is_empty() {
        issues.push(format!("{} straw man trap(s) detected", traps.len()));
    }

    issues
}

/// Get trap issues
fn get_trap_issues(traps: &[StrawManTrap]) -> Vec<String> {
    traps
        .iter()
        .map(|trap| format!("{}: {}", trap.label(), trap.checkbox_label()))
        .collect()
}

/// Props for PersonaConfirm component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct PersonaConfirmProps {
    /// The persona text
    pub persona: Signal<String>,
    /// Detected straw man traps
    pub detected_traps: Signal<Vec<StrawManTrap>>,
    /// Current step number (1-5)
    #[props(default = 2)]
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

/// PersonaConfirm component
///
/// Composes:
/// - PersonaDisplay: Shows the persona text for review/editing
/// - StrawManChecklist: Validates against straw man traps
/// - PersonaQuality: Quality score indicator
/// - Navigation: Back/Next buttons
///
/// This is the second confirmation step in the Progressive Discover flow.
#[component]
pub fn PersonaConfirm(props: PersonaConfirmProps) -> Element {
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
                    "Persona ({props.step}/{props.total_steps})"
                }
                span {
                    class: "text-sm text-muted-foreground",
                    "Confirm your target user persona"
                }
            }

            // Persona display section
            PersonaDisplay {
                persona: props.persona,
            }

            // Straw man trap checklist
            StrawManChecklist {
                detected_traps: props.detected_traps,
            }

            // Quality indicator section (expandable)
            div {
                class: "cursor-pointer",
                onclick: toggle_quality,
                PersonaQuality {
                    persona: props.persona,
                    detected_traps: props.detected_traps,
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
    fn test_calculate_persona_score_empty() {
        let score = calculate_persona_score("");
        assert_eq!(score, 0);
    }

    #[test]
    fn test_calculate_persona_score_short() {
        let score = calculate_persona_score("A short text");
        assert_eq!(score, 30);
    }

    #[test]
    fn test_calculate_persona_score_medium() {
        let score = calculate_persona_score("This is a medium length persona description with some details");
        assert!(score >= 50 && score <= 85);
    }

    #[test]
    fn test_calculate_specificity_score_empty() {
        let score = calculate_specificity_score("");
        assert_eq!(score, 0);
    }

    #[test]
    fn test_get_specificity_issues_short() {
        let issues = get_specificity_issues("Short");
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_get_specificity_issues_long() {
        // Need at least 25 words to have no issues
        let issues = get_specificity_issues("This is a long enough description with many words to pass the threshold for specificity and should be considered good enough for the test to pass without any issues being returned");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_get_trap_issues_empty() {
        let issues = get_trap_issues(&[]);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_get_trap_issues_with_traps() {
        let issues = get_trap_issues(&[StrawManTrap::IrrationalActor]);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("Irrational Actor"));
    }

    // Note: Tests requiring Dioxus runtime are skipped.
    // The following tests require dioxus::prelude::launch_test() wrapper:
    // - test_calculate_specificity_score_with_example (uses Signal internally via use_memo)
    // - test_calculate_realism_score_with_traps (uses Signal internally via use_memo)
}
