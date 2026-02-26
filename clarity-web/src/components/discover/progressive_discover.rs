#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Progressive Discover main component.
//!
//! This is the main orchestration component for the Progressive Discover flow.
//! It renders the appropriate phase component based on the current state
//! and handles phase transitions.

use dioxus::prelude::*;
use std::sync::Arc;

use super::extract_fields_button::ExtractFieldsButton;
use super::extracting_progress::{ExtractionStatus, ExtractingProgress};
use super::preview_summary::PreviewSummary;
use super::problem_confirm::ProblemConfirm;
use super::prompt_textarea::{CharacterCount, PromptTextarea, MIN_PROMPT_LENGTH};
use super::state::{ConfirmSubPhase, ProgressiveDiscoverPhase};
use crate::components::discover::antithesis::AntithesisResponse;
use crate::hooks::{
    use_progressive_discover, use_progressive_discover_actions,
    use_progressive_discover_with_prompt, ProgressiveDiscoverActions, ProgressiveDiscoverState,
};
use crate::providers::ExtractionProvider;
use crate::storage::transcript_store::InterrogationTranscript;
use crate::ui::button::ButtonVariant;
use crate::ui::Button;

// ============================================================================
// Scaffolding Prompts
// ============================================================================

/// Scaffolding prompt templates to help users get started.
///
/// Each tuple contains (label, template) where:
/// - label: A short name for the prompt type
/// - template: The full template text with placeholders
const SCAFFOLDING_PROMPTS: &[(&str, &str)] = &[
    (
        "Problem-first",
        "I'm building [product] because [users] struggle with [problem]. The main pain point is...",
    ),
    (
        "User-first",
        "My target users are [description]. They currently [behavior], but they want [outcome].",
    ),
    (
        "Solution-first",
        "I want to create [solution] that helps [users] achieve [outcome]. The key insight is...",
    ),
];

/// Props for ProgressiveDiscover component
#[derive(Clone, Props)]
pub struct ProgressiveDiscoverProps {
    /// Optional extraction provider for AI field extraction
    pub extraction_provider: Option<Arc<dyn ExtractionProvider>>,
    /// Optional initial prompt to pre-fill
    #[props(default)]
    pub initial_prompt: Option<String>,
    /// Callback when the flow completes (reaches Locked phase)
    pub on_complete: Option<EventHandler<InterrogationTranscript>>,
    /// Callback when the user wants to refine (go back to Prompt from Preview)
    pub on_refine: Option<EventHandler<()>>,
}

impl PartialEq for ProgressiveDiscoverProps {
    fn eq(&self, _other: &Self) -> bool {
        // Props with Arc<dyn Trait> cannot be compared
        // We assume equality based on initial_prompt only
        false
    }
}

/// ProgressiveDiscover component
///
/// Main orchestration component for the Progressive Discover flow.
///
/// # Phases
///
/// 1. **Prompt** - User enters freeform description
/// 2. **Extracting** - AI extracts structured fields
/// 3. **ConfirmingFields** - Field-by-field confirmation with adversarial coaching
/// 4. **Preview** - Review summary before locking
/// 5. **KirkCompilation** - Compile to KIRK contracts
/// 6. **Locked** - Final state, ready for Bead Factory
///
/// # State Management
///
/// Uses `use_progressive_discover` hook for state management.
/// State is passed down to phase components via props.
#[component]
pub fn ProgressiveDiscover(props: ProgressiveDiscoverProps) -> Element {
    // Initialize state
    let state = match props.initial_prompt.as_ref() {
        Some(prompt) => use_progressive_discover_with_prompt(prompt.clone()),
        None => use_progressive_discover(),
    };

    let actions = use_progressive_discover_actions(state);

    // Read current state
    let current_state = state.read();
    let phase = current_state.phase;
    let sub_phase = current_state.sub_phase;
    let is_loading = current_state.is_loading;
    let error = current_state.error.clone();
    drop(current_state);

    rsx! {
        div {
            class: "flex flex-col gap-6 w-full max-w-4xl mx-auto",

            // Phase progress indicator
            PhaseProgress { phase }

            // Error display
            if let Some(err) = error.as_ref() {
                div {
                    class: "rounded-lg border border-destructive/50 bg-destructive/10 p-4",
                    p {
                        class: "text-sm text-destructive font-medium",
                        "Error: {err}"
                    }
                }
            }

            // Phase content
            div {
                class: "min-h-[400px]",
                match phase {
                    ProgressiveDiscoverPhase::Prompt => {
                        rsx! {
                            PromptPhase {
                                state,
                                actions,
                                extraction_provider: props.extraction_provider.clone(),
                            }
                        }
                    }
                    ProgressiveDiscoverPhase::Extracting => {
                        rsx! {
                            ExtractingPhase {
                                state,
                                actions,
                            }
                        }
                    }
                    ProgressiveDiscoverPhase::ConfirmingFields => {
                        rsx! {
                            ConfirmPhase {
                                state,
                                actions,
                                sub_phase,
                            }
                        }
                    }
                    ProgressiveDiscoverPhase::Preview => {
                        rsx! {
                            PreviewPhase {
                                state,
                                actions,
                                on_refine: props.on_refine.clone(),
                                on_complete: props.on_complete.clone(),
                            }
                        }
                    }
                    ProgressiveDiscoverPhase::KirkCompilation => {
                        rsx! {
                            KirkCompilationPhase {
                                state,
                                actions,
                            }
                        }
                    }
                    ProgressiveDiscoverPhase::Locked => {
                        rsx! {
                            LockedPhase {
                                state,
                                on_complete: props.on_complete.clone(),
                            }
                        }
                    }
                }
            }

            // Loading overlay
            if is_loading {
                div {
                    class: "fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm",
                    div {
                        class: "flex items-center gap-3",
                        svg {
                            class: "h-6 w-6 animate-spin text-primary",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            circle {
                                class: "opacity-25",
                                cx: "12",
                                cy: "12",
                                r: "10",
                                stroke: "currentColor",
                                stroke_width: "4",
                            }
                            path {
                                class: "opacity-75",
                                fill: "currentColor",
                                d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                            }
                        }
                        span {
                            class: "text-sm font-medium text-foreground",
                            "Processing..."
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Phase Progress Component
// ============================================================================

/// Props for PhaseProgress component
#[derive(Clone, Copy, Props, PartialEq)]
pub struct PhaseProgressProps {
    /// Current phase
    pub phase: ProgressiveDiscoverPhase,
}

/// Phase progress indicator component
///
/// Shows the current phase and progress through the flow.
#[component]
pub fn PhaseProgress(props: PhaseProgressProps) -> Element {
    let phases = ProgressiveDiscoverPhase::all();
    let current_ordinal = props.phase.ordinal();

    rsx! {
        div {
            class: "flex items-center justify-center gap-2",

            for (index, phase) in phases.iter().enumerate() {
                // Phase indicator
                div {
                    class: format!(
                        "flex items-center gap-1 {}",
                        if index > 0 { "ml-2" } else { "" }
                    ),
                    // Circle
                    div {
                        class: format!(
                            "flex h-8 w-8 items-center justify-center rounded-full text-xs font-medium {}",
                            if phase.ordinal() < current_ordinal {
                                "bg-primary text-primary-foreground"
                            } else if phase.ordinal() == current_ordinal {
                                "bg-primary text-primary-foreground ring-2 ring-primary ring-offset-2"
                            } else {
                                "bg-muted text-muted-foreground"
                            }
                        ),
                        if phase.ordinal() < current_ordinal {
                            // Checkmark for completed phases
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
                                polyline { points: "20 6 9 17 4 12" }
                            }
                        } else {
                            "{phase.ordinal()}"
                        }
                    }

                    // Label
                    span {
                        class: format!(
                            "text-xs font-medium hidden sm:block {}",
                            if phase.ordinal() == current_ordinal {
                                "text-foreground"
                            } else {
                                "text-muted-foreground"
                            }
                        ),
                        "{phase.display_name()}"
                    }
                }

                // Connector line (except after last)
                if index < phases.len() - 1 {
                    div {
                        class: format!(
                            "h-0.5 w-4 {}",
                            if phase.ordinal() < current_ordinal {
                                "bg-primary"
                            } else {
                                "bg-muted"
                            }
                        ),
                    }
                }
            }
        }
    }
}

// ============================================================================
// Prompt Phase Component
// ============================================================================

/// Props for PromptPhase component
#[derive(Clone, Props)]
pub struct PromptPhaseProps {
    /// State signal
    pub state: Signal<ProgressiveDiscoverState>,
    /// Actions for state manipulation
    pub actions: ProgressiveDiscoverActions,
    /// Optional extraction provider
    pub extraction_provider: Option<Arc<dyn ExtractionProvider>>,
}

// Manual PartialEq implementation since Arc<dyn ExtractionProvider> doesn't implement it
impl PartialEq for PromptPhaseProps {
    fn eq(&self, other: &Self) -> bool {
        // Signals and Arc<dyn Trait> can't be meaningfully compared
        // Return false to avoid incorrect equality assumptions
        false
    }
}

/// Prompt phase component
///
/// Allows user to enter freeform description of their idea.
#[component]
pub fn PromptPhase(props: PromptPhaseProps) -> Element {
    let prompt = use_signal(|| {
        let state = props.state.read();
        state.transcript.original_prompt.clone()
    });

    let is_extracting = use_signal(|| false);

    let on_submit = {
        let mut actions = props.actions.clone();
        let prompt = prompt.clone();
        let mut is_extracting = is_extracting.clone();
        move |_| {
            let prompt_value = prompt.read().clone();
            if prompt_value.trim().len() >= MIN_PROMPT_LENGTH {
                // Set loading state
                is_extracting.set(true);

                // Update transcript with prompt
                let transcript = InterrogationTranscript::from_prompt(prompt_value);
                actions.update_transcript(transcript);
                // Advance to Extracting phase
                actions.advance_phase();
            }
        }
    };

    let on_input = {
        let mut prompt = prompt.clone();
        move |value: String| {
            prompt.set(value);
        }
    };

    let char_count = prompt.read().trim().len();
    let is_ready = char_count >= MIN_PROMPT_LENGTH;

    rsx! {
        div {
            class: "space-y-6 rounded-lg border border-border/50 bg-card p-6 shadow-sm",

            // Header
            div {
                class: "border-b border-border/50 pb-4",
                h2 {
                    class: "text-lg font-semibold text-foreground",
                    "Describe Your Idea"
                }
                p {
                    class: "text-sm text-muted-foreground mt-1",
                    "Tell me what you want to build. Be as detailed as you like."
                }
            }

            // Scaffolding prompts
            div {
                class: "space-y-3",
                p {
                    class: "text-sm font-medium text-muted-foreground",
                    "Need help getting started? Try one of these:"
                }
                div {
                    class: "flex flex-wrap gap-2",
                    for (label, template) in SCAFFOLDING_PROMPTS {
                        ScaffoldingPromptButton {
                            label: (*label).to_string(),
                            template: (*template).to_string(),
                            onclick: {
                                let mut prompt = prompt.clone();
                                move |_| {
                                    prompt.set((*template).to_string());
                                }
                            },
                        }
                    }
                }
            }

            // Input area
            div {
                class: "space-y-4",
                PromptTextarea {
                    value: prompt.read().clone(),
                    placeholder: "Describe your idea...".to_string(),
                    on_change: on_input,
                }

                // Character count with minimum threshold
                CharacterCount {
                    current: char_count,
                    minimum: MIN_PROMPT_LENGTH,
                    maximum: 2000,
                }
            }

            // Submit button
            div {
                class: "flex justify-end border-t border-border/50 pt-4",
                ExtractFieldsButton {
                    prompt: prompt.read().clone(),
                    is_loading: *is_extracting.read(),
                    disabled: !is_ready,
                    on_click: {
                        let mut on_submit = on_submit.clone();
                        move |_prompt: String| {
                            on_submit(());
                        }
                    },
                }
            }
        }
    }
}

/// Props for ScaffoldingPromptButton
#[derive(Clone, Props, PartialEq)]
pub struct ScaffoldingPromptButtonProps {
    /// Label for the button
    pub label: String,
    /// Template text to insert
    pub template: String,
    /// Click handler
    pub onclick: EventHandler<Event<MouseData>>,
}

/// Button for scaffolding prompts
///
/// Displays a styled button that inserts a template prompt when clicked.
#[component]
pub fn ScaffoldingPromptButton(props: ScaffoldingPromptButtonProps) -> Element {
    rsx! {
        button {
            class: "rounded-md border border-border bg-background px-3 py-1.5 text-sm text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors",
            onclick: move |e| {
                props.onclick.call(e);
            },
            "{props.label}"
        }
    }
}

// ============================================================================
// Extracting Phase Component
// ============================================================================

/// Props for ExtractingPhase component
#[derive(Clone, Props, PartialEq)]
pub struct ExtractingPhaseProps {
    /// State signal
    pub state: Signal<ProgressiveDiscoverState>,
    /// Actions for state manipulation
    pub actions: ProgressiveDiscoverActions,
}

/// Extracting phase component
///
/// Shows progress while AI extracts fields from user input.
#[component]
pub fn ExtractingPhase(props: ExtractingPhaseProps) -> Element {
    let progress = use_signal(|| 0u8);
    let status_messages = [
        "Parsing problem statement...",
        "Identifying target users...",
        "Extracting solution details...",
        "Analyzing scenario context...",
        "Validating extraction quality...",
    ];
    let current_message = use_signal(|| 0usize);

    // Simulate extraction progress
    use_effect({
        let mut actions = props.actions.clone();
        let mut progress = progress.clone();
        let mut current_message = current_message.clone();
        move || {
            // For now, auto-advance through the extraction
            // In a real implementation, this would be driven by the AI provider
            let current_progress = *progress.read();
            if current_progress < 100 {
                let new_progress = (current_progress + 20).min(100);
                *progress.write() = new_progress;
                *current_message.write() = (new_progress as usize / 20).min(status_messages.len() - 1);

                if new_progress >= 100 {
                    // Auto-advance to confirming fields after extraction completes
                    actions.advance_phase();
                }
            }
        }
    });

    let message_idx = *current_message.read();
    let message = status_messages.get(message_idx).map_or("Processing...", |s| *s);

    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-12 space-y-6",

            ExtractingProgress {
                status: ExtractionStatus::Extracting,
                progress: *progress.read(),
                message: Some(message.to_string()),
            }

            p {
                class: "text-sm text-muted-foreground text-center max-w-md",
                "Analyzing your input and extracting structured fields. This usually takes a few seconds."
            }
        }
    }
}

// ============================================================================
// Confirm Phase Component
// ============================================================================

/// Props for ConfirmPhase component
#[derive(Clone, Props, PartialEq)]
pub struct ConfirmPhaseProps {
    /// State signal
    pub state: Signal<ProgressiveDiscoverState>,
    /// Actions for state manipulation
    pub actions: ProgressiveDiscoverActions,
    /// Current sub-phase
    pub sub_phase: ConfirmSubPhase,
}

/// Confirm phase component
///
/// Field-by-field confirmation with adversarial coaching.
#[component]
pub fn ConfirmPhase(props: ConfirmPhaseProps) -> Element {
    // Create signals for problem and antithesis
    let problem = use_signal(|| {
        let state = props.state.read();
        state.transcript.problem.content.clone()
    });

    let antithesis = use_signal(|| {
        let state = props.state.read();
        AntithesisResponse::new(state.transcript.antithesis.points.to_vec())
    });

    let on_next = {
        let mut actions = props.actions.clone();
        move |_: Event<MouseData>| {
            // Check if we're at the last sub-phase
            let state = props.state.read();
            if state.sub_phase.is_last() {
                // Advance to Preview phase
                actions.advance_phase();
            } else {
                // Advance to next sub-phase
                actions.advance_sub_phase();
            }
        }
    };

    let on_back = {
        let mut actions = props.actions.clone();
        move |_: Event<MouseData>| {
            let state = props.state.read();
            if state.sub_phase.is_first() {
                // Go back to Extracting phase
                actions.regress_phase();
            } else {
                // Go back to previous sub-phase
                actions.regress_sub_phase();
            }
        }
    };

    // Render the appropriate sub-phase content
    let sub_phase_content = match props.sub_phase {
        ConfirmSubPhase::ConfirmProblem => {
            rsx! {
                ProblemConfirm {
                    problem,
                    antithesis,
                    step: props.sub_phase.ordinal() as u8,
                    total_steps: ConfirmSubPhase::count() as u8,
                    on_next: Some(EventHandler::new(on_next.clone())),
                    on_back: Some(EventHandler::new(on_back.clone())),
                    next_disabled: false,
                    back_disabled: false,
                }
            }
        }
        ConfirmSubPhase::ConfirmPersona => {
            rsx! {
                PlaceholderConfirmPhase {
                    title: "Persona",
                    step: props.sub_phase.ordinal() as u8,
                    total_steps: ConfirmSubPhase::count() as u8,
                    description: "Confirm your target user and validate against straw man traps.",
                    on_next: EventHandler::new(on_next.clone()),
                    on_back: EventHandler::new(on_back.clone()),
                }
            }
        }
        ConfirmSubPhase::ConfirmSolution => {
            rsx! {
                PlaceholderConfirmPhase {
                    title: "Solution",
                    step: props.sub_phase.ordinal() as u8,
                    total_steps: ConfirmSubPhase::count() as u8,
                    description: "Confirm solution and justify VORP (Value Over Replacement).",
                    on_next: EventHandler::new(on_next.clone()),
                    on_back: EventHandler::new(on_back.clone()),
                }
            }
        }
        ConfirmSubPhase::ConfirmNonpersona => {
            rsx! {
                PlaceholderConfirmPhase {
                    title: "Nonpersona",
                    step: props.sub_phase.ordinal() as u8,
                    total_steps: ConfirmSubPhase::count() as u8,
                    description: "Define who you are explicitly NOT building for.",
                    on_next: EventHandler::new(on_next.clone()),
                    on_back: EventHandler::new(on_back.clone()),
                }
            }
        }
        ConfirmSubPhase::ConfirmScenario => {
            rsx! {
                PlaceholderConfirmPhase {
                    title: "Scenario",
                    step: props.sub_phase.ordinal() as u8,
                    total_steps: ConfirmSubPhase::count() as u8,
                    description: "Define trigger, value moment, and outcome with hole punching.",
                    on_next: EventHandler::new(on_next.clone()),
                    on_back: EventHandler::new(on_back.clone()),
                }
            }
        }
    };

    rsx! {
        div {
            class: "space-y-4",
            {sub_phase_content}
        }
    }
}

/// Props for PlaceholderConfirmPhase
#[derive(Clone, Props, PartialEq)]
pub struct PlaceholderConfirmPhaseProps {
    /// Title of the phase
    pub title: String,
    /// Current step
    pub step: u8,
    /// Total steps
    pub total_steps: u8,
    /// Description
    pub description: String,
    /// Next handler
    pub on_next: EventHandler<Event<MouseData>>,
    /// Back handler
    pub on_back: EventHandler<Event<MouseData>>,
}

/// Placeholder confirm phase for sub-phases not yet fully implemented
#[component]
pub fn PlaceholderConfirmPhase(props: PlaceholderConfirmPhaseProps) -> Element {
    rsx! {
        div {
            class: "space-y-6 rounded-lg border border-border/50 bg-card p-6 shadow-sm",

            // Header
            div {
                class: "flex items-center justify-between border-b border-border/50 pb-4",
                h2 {
                    class: "text-lg font-semibold text-foreground",
                    "{props.title} ({props.step}/{props.total_steps})"
                }
                span {
                    class: "text-sm text-muted-foreground",
                    "Confirm your {props.title.to_lowercase()} statement"
                }
            }

            // Description
            p {
                class: "text-sm text-muted-foreground",
                "{props.description}"
            }

            // Placeholder content
            div {
                class: "rounded-lg border border-dashed border-border bg-muted/20 p-8 text-center",
                p {
                    class: "text-sm text-muted-foreground",
                    "This confirmation step will be implemented in a future iteration."
                }
            }

            // Navigation
            div {
                class: "flex items-center justify-between border-t border-border/50 pt-4",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |e| {
                        props.on_back.call(e);
                    },
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
                    onclick: move |e| {
                        props.on_next.call(e);
                    },
                    "Next"
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

// ============================================================================
// Preview Phase Component
// ============================================================================

/// Props for PreviewPhase component
#[derive(Clone, Props, PartialEq)]
pub struct PreviewPhaseProps {
    /// State signal
    pub state: Signal<ProgressiveDiscoverState>,
    /// Actions for state manipulation
    pub actions: ProgressiveDiscoverActions,
    /// Callback when user wants to refine
    pub on_refine: Option<EventHandler<()>>,
    /// Callback when flow completes
    pub on_complete: Option<EventHandler<InterrogationTranscript>>,
}

/// Preview phase component
///
/// Shows summary and allows refining or locking in.
#[component]
pub fn PreviewPhase(props: PreviewPhaseProps) -> Element {
    let transcript_signal = use_signal(|| {
        let state = props.state.read();
        state.transcript.clone()
    });

    let on_refine = {
        let mut actions = props.actions.clone();
        let on_refine = props.on_refine.clone();
        move |_| {
            // Go back to Prompt phase
            actions.regress_phase();
            actions.regress_phase();
            actions.regress_phase();
            if let Some(handler) = on_refine {
                handler.call(());
            }
        }
    };

    let on_lock_in = {
        let mut actions = props.actions.clone();
        let on_complete = props.on_complete.clone();
        let state = props.state.clone();
        move |_| {
            // Advance to KirkCompilation
            actions.advance_phase();
            if let Some(handler) = on_complete {
                let transcript = state.read().transcript.clone();
                handler.call(transcript);
            }
        }
    };

    rsx! {
        div {
            class: "space-y-6",

            // Preview summary
            PreviewSummary {
                transcript: transcript_signal,
                on_change: None,
            }

            // Four Brutal Truths checklist
            div {
                class: "rounded-lg border border-border/50 bg-card p-6 space-y-4",
                h3 {
                    class: "text-sm font-medium text-muted-foreground uppercase tracking-wide",
                    "Four Brutal Truths"
                }
                div {
                    class: "space-y-2",
                    BrutalTruthItem {
                        text: "I have a specific user in mind, not everyone".to_string(),
                    }
                    BrutalTruthItem {
                        text: "I know why they will switch from their current solution".to_string(),
                    }
                    BrutalTruthItem {
                        text: "I can describe a concrete scenario where this delivers value".to_string(),
                    }
                    BrutalTruthItem {
                        text: "I understand why this might fail and am ready to learn".to_string(),
                    }
                }
            }

            // Action buttons
            div {
                class: "flex items-center justify-between border-t border-border/50 pt-4",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: on_refine,
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
                        path { d: "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" }
                        path { d: "M3 3v5h5" }
                    }
                    "Refine"
                }

                Button {
                    variant: ButtonVariant::Primary,
                    onclick: on_lock_in,
                    "Lock In"
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
                        rect {
                            x: "3",
                            y: "11",
                            width: "18",
                            height: "11",
                            rx: "2",
                            ry: "2",
                        }
                        path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
                    }
                }
            }
        }
    }
}

/// Props for BrutalTruthItem
#[derive(Clone, Props, PartialEq)]
pub struct BrutalTruthItemProps {
    /// The truth text
    pub text: String,
}

/// Brutal truth checklist item
#[component]
pub fn BrutalTruthItem(props: BrutalTruthItemProps) -> Element {
    let checked = use_signal(|| false);

    rsx! {
        label {
            class: "flex items-center gap-3 cursor-pointer",
            input {
                r#type: "checkbox",
                checked: *checked.read(),
                onchange: {
                    let mut checked = checked.clone();
                    move |e: Event<FormData>| {
                        *checked.write() = e.checked();
                    }
                },
                class: "h-4 w-4 rounded border-border",
            }
            span {
                class: format!(
                    "text-sm {}",
                    if *checked.read() {
                        "text-foreground"
                    } else {
                        "text-muted-foreground"
                    }
                ),
                "{props.text}"
            }
        }
    }
}

// ============================================================================
// Kirk Compilation Phase Component
// ============================================================================

/// Props for KirkCompilationPhase component
#[derive(Clone, Props, PartialEq)]
pub struct KirkCompilationPhaseProps {
    /// State signal
    pub state: Signal<ProgressiveDiscoverState>,
    /// Actions for state manipulation
    pub actions: ProgressiveDiscoverActions,
}

/// Kirk compilation phase component
///
/// Shows progress while compiling to KIRK contracts.
#[component]
pub fn KirkCompilationPhase(props: KirkCompilationPhaseProps) -> Element {
    let progress = use_signal(|| 0u8);
    let section_index = use_signal(|| 0usize);
    let sections = [
        "Metadata",
        "EARS Requirements",
        "Inversion Controls",
        "Contracts",
        "Acceptance Tests",
    ];

    // Simulate compilation progress
    use_effect({
        let mut actions = props.actions.clone();
        let mut progress = progress.clone();
        let mut section_index = section_index.clone();
        move || {
            let current_progress = *progress.read();
            if current_progress < 100 {
                let new_progress = (current_progress + 20).min(100);
                *progress.write() = new_progress;
                *section_index.write() = (new_progress as usize / 20).min(sections.len() - 1);

                if new_progress >= 100 {
                    // Auto-advance to Locked phase
                    actions.advance_phase();
                }
            }
        }
    });

    let current_section = sections.get(*section_index.read()).map_or("Processing...", |s| *s);

    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-12 space-y-6",

            ExtractingProgress {
                status: ExtractionStatus::Extracting,
                progress: *progress.read(),
                message: Some(format!("Compiling {current_section}...")),
            }

            p {
                class: "text-sm text-muted-foreground text-center max-w-md",
                "Compiling your plan into KIRK contracts. This will create a structured plan ready for implementation."
            }

            // Section list
            div {
                class: "w-full max-w-sm space-y-2",
                for (idx, section) in sections.iter().enumerate() {
                    div {
                        class: format!(
                            "flex items-center gap-2 text-sm {}",
                            if idx <= *section_index.read() {
                                "text-foreground"
                            } else {
                                "text-muted-foreground"
                            }
                        ),
                        if idx < *section_index.read() {
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
                                class: "text-emerald-500",
                                polyline { points: "20 6 9 17 4 12" }
                            }
                        } else if idx == *section_index.read() {
                            svg {
                                class: "h-4 w-4 animate-spin text-primary",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                circle {
                                    class: "opacity-25",
                                    cx: "12",
                                    cy: "12",
                                    r: "10",
                                    stroke: "currentColor",
                                    stroke_width: "4",
                                }
                                path {
                                    class: "opacity-75",
                                    fill: "currentColor",
                                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                                }
                            }
                        } else {
                            div {
                                class: "h-4 w-4 rounded-full border border-border",
                            }
                        }
                        "{section}"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Locked Phase Component
// ============================================================================

/// Props for LockedPhase component
#[derive(Clone, Props, PartialEq)]
pub struct LockedPhaseProps {
    /// State signal
    pub state: Signal<ProgressiveDiscoverState>,
    /// Callback when complete
    pub on_complete: Option<EventHandler<InterrogationTranscript>>,
}

/// Locked phase component
///
/// Final collapsed state showing the locked plan.
#[component]
pub fn LockedPhase(props: LockedPhaseProps) -> Element {
    let transcript = {
        let state = props.state.read();
        state.transcript.clone()
    };

    rsx! {
        div {
            class: "space-y-6 rounded-lg border border-border/50 bg-card p-6 shadow-sm",

            // Success header
            div {
                class: "flex items-center gap-3 border-b border-border/50 pb-4",
                div {
                    class: "flex h-12 w-12 items-center justify-center rounded-full bg-emerald-500/10",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "24",
                        height: "24",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        class: "text-emerald-500",
                        path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
                        polyline { points: "22 4 12 14.01 9 11.01" }
                    }
                }
                div {
                    h2 {
                        class: "text-lg font-semibold text-foreground",
                        "Plan Locked!"
                    }
                    p {
                        class: "text-sm text-muted-foreground",
                        "Your plan is ready for implementation."
                    }
                }
            }

            // Collapsed summary
            div {
                class: "space-y-4",
                div {
                    class: "rounded-lg bg-muted/30 p-4",
                    h3 {
                        class: "text-sm font-medium text-foreground mb-2",
                        "Problem"
                    }
                    p {
                        class: "text-sm text-muted-foreground",
                        {if transcript.problem.content.is_empty() {
                            "No problem defined".to_string()
                        } else {
                            transcript.problem.content.clone()
                        }}
                    }
                }

                div {
                    class: "rounded-lg bg-muted/30 p-4",
                    h3 {
                        class: "text-sm font-medium text-foreground mb-2",
                        "Solution"
                    }
                    p {
                        class: "text-sm text-muted-foreground",
                        {if transcript.solution.content.is_empty() {
                            "No solution defined".to_string()
                        } else {
                            transcript.solution.content.clone()
                        }}
                    }
                }
            }

            // Action buttons
            div {
                class: "flex items-center justify-end gap-3 border-t border-border/50 pt-4",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        // View Plan action (could navigate or open dialog)
                    },
                    "View Plan"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: {
                        let on_complete = props.on_complete.clone();
                        move |_| {
                            if let Some(handler) = on_complete.clone() {
                                let transcript = props.state.read().transcript.clone();
                                handler.call(transcript);
                            }
                        }
                    },
                    "Continue to Bead Factory"
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
    fn test_phase_progress_props_equality() {
        let props1 = PhaseProgressProps {
            phase: ProgressiveDiscoverPhase::Prompt,
        };
        let props2 = PhaseProgressProps {
            phase: ProgressiveDiscoverPhase::Prompt,
        };
        assert_eq!(props1, props2);
    }

    #[test]
    fn test_scaffolding_prompt_button_props_equality() {
        let props1 = ScaffoldingPromptButtonProps {
            text: "Test".to_string(),
            onclick: EventHandler::new(|_| {}),
        };
        let props2 = ScaffoldingPromptButtonProps {
            text: "Test".to_string(),
            onclick: EventHandler::new(|_| {}),
        };
        // Props with EventHandler cannot be compared for equality
        drop(props1);
        drop(props2);
    }

    #[test]
    fn test_brutal_truth_item_props_equality() {
        let props1 = BrutalTruthItemProps {
            text: "Test truth".to_string(),
        };
        let props2 = BrutalTruthItemProps {
            text: "Test truth".to_string(),
        };
        assert_eq!(props1, props2);
    }

    #[test]
    fn test_phase_ordinal_sequence() {
        assert!(ProgressiveDiscoverPhase::Prompt.ordinal() < ProgressiveDiscoverPhase::Extracting.ordinal());
        assert!(ProgressiveDiscoverPhase::Extracting.ordinal() < ProgressiveDiscoverPhase::ConfirmingFields.ordinal());
        assert!(ProgressiveDiscoverPhase::ConfirmingFields.ordinal() < ProgressiveDiscoverPhase::Preview.ordinal());
        assert!(ProgressiveDiscoverPhase::Preview.ordinal() < ProgressiveDiscoverPhase::KirkCompilation.ordinal());
        assert!(ProgressiveDiscoverPhase::KirkCompilation.ordinal() < ProgressiveDiscoverPhase::Locked.ordinal());
    }

    #[test]
    fn test_sub_phase_ordinal_sequence() {
        assert!(ConfirmSubPhase::ConfirmProblem.ordinal() < ConfirmSubPhase::ConfirmPersona.ordinal());
        assert!(ConfirmSubPhase::ConfirmPersona.ordinal() < ConfirmSubPhase::ConfirmSolution.ordinal());
        assert!(ConfirmSubPhase::ConfirmSolution.ordinal() < ConfirmSubPhase::ConfirmNonpersona.ordinal());
        assert!(ConfirmSubPhase::ConfirmNonpersona.ordinal() < ConfirmSubPhase::ConfirmScenario.ordinal());
    }
}
