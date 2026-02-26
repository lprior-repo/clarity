#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod antithesis;
pub mod brutal_truths;
pub mod extract_fields_button;
pub mod extracting_progress;
pub mod field_card;
pub mod locked_phase;
pub mod locked_summary;
pub mod nonpersona_confirm;
pub mod phases;
pub mod persona_confirm;
pub mod preview_summary;
pub mod problem_confirm;
pub mod progressive_discover;
pub mod prompt_textarea;
pub mod quality_score;
pub mod scenario_confirm;
pub mod solution_confirm;
pub mod state;
pub mod straw_man;
pub mod types;

pub use antithesis::AntithesisResponse;
pub use brutal_truths::{
    BrutalTruth, BrutalTruthsChecklist, BrutalTruthsChecklistProps, BrutalTruthsState,
    BrutalTruthsSummary, BrutalTruthsSummaryProps, BrutalTruthItem, BrutalTruthItemProps,
};
pub use extract_fields_button::{
    ExtractFieldsButton, ExtractFieldsButtonProps, ExtractFieldsButtonWithServer,
    ExtractFieldsButtonWithServerProps, ExtractedField, ExtractedFieldsData, MIN_PROMPT_CHARS,
};
pub use extracting_progress::{ExtractionStatus, ExtractingProgress, ExtractingProgressProps};
pub use field_card::{Confidence, FieldCard, FieldCardProps, FieldData};
pub use locked_phase::{LockedPhase, LockedPhaseProps};
pub use locked_summary::{ArtifactStats, LockedSummary, LockedSummaryProps};
pub use nonpersona_confirm::{
    NonpersonaConfirm, NonpersonaConfirmProps, NonpersonaDisplay, NonpersonaDisplayProps,
    NonpersonaGuidance, NonpersonaGuidanceProps, NonpersonaQuality, NonpersonaQualityProps,
};
pub use persona_confirm::{
    PersonaConfirm, PersonaConfirmProps, PersonaDisplay, PersonaDisplayProps,
    PersonaQuality, PersonaQualityProps, StrawManChecklist, StrawManChecklistProps,
};
pub use preview_summary::{sample_transcript, HoleStatusBadge, HoleStatusBadgeProps, PreviewSummary, PreviewSummaryProps};
pub use problem_confirm::{
    AntithesisInput, AntithesisInputProps, AntithesisQuality, AntithesisQualityProps,
    ProblemConfirm, ProblemConfirmProps, ProblemDisplay, ProblemDisplayProps,
};
pub use progressive_discover::{
    BrutalTruthItem as PdBrutalTruthItem, BrutalTruthItemProps as PdBrutalTruthItemProps,
    ConfirmPhase, ConfirmPhaseProps, ExtractingPhase, ExtractingPhaseProps, KirkCompilationPhase,
    KirkCompilationPhaseProps, LockedPhase as PdLockedPhase, LockedPhaseProps as PdLockedPhaseProps,
    PhaseProgress, PhaseProgressProps, PlaceholderConfirmPhase, PlaceholderConfirmPhaseProps,
    PreviewPhase, PreviewPhaseProps, ProgressiveDiscover, ProgressiveDiscoverProps, PromptPhase,
    PromptPhaseProps, ScaffoldingPromptButton, ScaffoldingPromptButtonProps,
};
pub use prompt_textarea::{CharacterCount, CharacterCountProps, MIN_PROMPT_LENGTH, PromptTextarea, PromptTextareaProps, MAX_PROMPT_LENGTH};
pub use quality_score::{QualityDimension, QualityScore, QualityScoreBar, QualityScoreBarProps};
pub use scenario_confirm::{
    HolePunchingChecklist, HolePunchingChecklistProps, ScenarioBulletInput, ScenarioBulletInputProps,
    ScenarioConfirm, ScenarioConfirmProps, ScenarioQuality, ScenarioQualityProps,
};
pub use solution_confirm::{
    SolutionConfirm, SolutionConfirmProps, SolutionDisplay, SolutionDisplayProps,
    VorpFields, VorpInput, VorpInputProps, VorpQuality, VorpQualityProps,
};
pub use state::{ConfirmSubPhase, ProgressiveDiscoverPhase};
pub use straw_man::{StrawManTrap, StrawManValidation};
pub use types::{Hole, HolePunchingResults, HoleType, ScenarioField};
