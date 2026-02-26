#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Extracting Phase component for the Progressive Discover flow.
//!
//! This module implements the Extracting phase (WP03) which displays
//! an animated progress bar and status messages while the AI extracts
//! the 5 fields (Problem, Persona, Solution, Nonpersona, Scenario)
//! from the user's prompt.
//!
//! # Beads
//!
//! - **bd-23qy**: Progress animation with status messages
//! - **bd-xz68**: ExtractingPhase component with auto-transition

use dioxus::prelude::*;

use crate::components::discover::extracting_progress::{ExtractionStatus, ExtractingProgress};
use crate::hooks::{ProgressiveDiscoverActions, ProgressiveDiscoverState};
use crate::ui::button::ButtonVariant;
use crate::ui::Button;

// ============================================================================
// Extraction Status Messages (bd-23qy)
// ============================================================================

/// Status messages shown during extraction progress.
///
/// Each message corresponds to roughly 20% of the extraction process.
const EXTRACTION_STATUS_MESSAGES: &[&str] = &[
    "Parsing problem statement...",
    "Identifying target users...",
    "Extracting solution details...",
    "Analyzing scenario context...",
    "Validating extraction quality...",
];

// ============================================================================
// ExtractingPhase Component (bd-xz68)
// ============================================================================

/// Props for ExtractingPhase component
#[derive(Clone, Props, PartialEq)]
pub struct ExtractingPhaseProps {
    /// State signal
    pub state: Signal<ProgressiveDiscoverState>,
    /// Actions for state manipulation
    pub actions: ProgressiveDiscoverActions,
}

/// Extracting phase component (bd-xz68)
///
/// Shows progress while AI extracts fields from user input.
/// Displays an animated progress bar and status messages.
/// Auto-transitions to ConfirmingFields phase when extraction completes.
///
/// # Features
///
/// - Animated progress bar (0-100%)
/// - Status messages that update as extraction progresses
/// - Error handling with retry capability
/// - Guard against multiple phase transitions
///
/// # State Machine
///
/// ```text
/// Extracting (0%) -> ... -> Extracting (100%) -> ConfirmingFields
/// ```
#[component]
pub fn ExtractingPhase(props: ExtractingPhaseProps) -> Element {
    let progress = use_signal(|| 0u8);
    let current_message = use_signal(|| 0usize);

    // Guard flag to prevent multiple phase transitions (Issue C2)
    let has_transitioned = use_signal(|| false);

    // Error state for failed extractions (Issue C3)
    let error = use_signal(|| None::<String>);

    // Simulate extraction progress
    use_effect({
        let mut actions = props.actions.clone();
        let mut progress = progress.clone();
        let mut current_message = current_message.clone();
        let mut has_transitioned = has_transitioned.clone();
        let error = error.clone();
        move || {
            // Check for existing error - don't continue if failed
            if error.read().is_some() {
                return;
            }

            // For now, auto-advance through the extraction
            // In a real implementation, this would be driven by the AI provider
            let current_progress = *progress.read();
            if current_progress < 100 {
                let new_progress = (current_progress + 20).min(100);
                *progress.write() = new_progress;
                *current_message.write() =
                    (new_progress as usize / 20).min(EXTRACTION_STATUS_MESSAGES.len() - 1);

                // Guard against multiple transitions (Issue C2)
                if new_progress >= 100 && !*has_transitioned.read() {
                    *has_transitioned.write() = true;
                    // Auto-advance to confirming fields after extraction completes
                    actions.advance_phase();
                }
            }
        }
    });

    let message_idx = *current_message.read();
    let message = EXTRACTION_STATUS_MESSAGES
        .get(message_idx)
        .map_or("Processing...", |s| *s);

    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-12 space-y-6",

            // Error display (Issue C3)
            if let Some(err) = error.read().as_ref() {
                div {
                    class: "w-full max-w-md p-4 bg-destructive/10 text-destructive rounded-lg border border-destructive/50",
                    p {
                        class: "text-sm font-medium mb-2",
                        "Extraction failed: {err}"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: {
                            let mut error = error.clone();
                            let mut progress = progress.clone();
                            let mut current_message = current_message.clone();
                            let mut has_transitioned = has_transitioned.clone();
                            move |_| {
                                // Reset state for retry
                                *error.write() = None;
                                *progress.write() = 0;
                                *current_message.write() = 0;
                                *has_transitioned.write() = false;
                            }
                        },
                        "Retry"
                    }
                }
            } else {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_status_messages_count() {
        // Should have 5 messages for 20% increments
        assert_eq!(EXTRACTION_STATUS_MESSAGES.len(), 5);
    }

    #[test]
    fn test_extraction_status_messages_content() {
        // Verify all messages are non-empty
        for message in EXTRACTION_STATUS_MESSAGES {
            assert!(!message.is_empty());
        }
    }

    #[test]
    fn test_extracting_phase_props_creation() {
        // Note: Signal and Actions cannot be meaningfully compared,
        // but we can create them
        let state = Signal::new(ProgressiveDiscoverState::default());
        let actions = ProgressiveDiscoverActions::new(state);
        let props = ExtractingPhaseProps {
            state,
            actions,
        };
        // Just verify it can be created and dropped
        drop(props);
    }
}
