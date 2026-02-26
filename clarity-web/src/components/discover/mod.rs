#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod antithesis;
pub mod brutal_truths;
// TODO: Delete discover_flow.rs after progressive_discover.rs main container is implemented
// pub mod discover_flow;
pub mod extracting_progress;
pub mod field_card;
pub mod locked_phase;
pub mod locked_summary;
pub mod preview_summary;
pub mod problem_confirm;
pub mod progressive_discover;
pub mod quality_score;
pub mod state;
pub mod straw_man;
pub mod types;

pub use antithesis::AntithesisResponse;
pub use brutal_truths::{
    BrutalTruth, BrutalTruthsChecklist, BrutalTruthsChecklistProps, BrutalTruthsState,
    BrutalTruthsSummary, BrutalTruthsSummaryProps, BrutalTruthItem, BrutalTruthItemProps,
};
// TODO: Restore after progressive_discover.rs main container is implemented
// pub use discover_flow::{DiscoverFlow, DiscoverFlowProps};
pub use extracting_progress::{ExtractionStatus, ExtractingProgress, ExtractingProgressProps};
pub use field_card::{Confidence, FieldCard, FieldCardProps, FieldData};
pub use locked_phase::{LockedPhase, LockedPhaseProps};
pub use locked_summary::{ArtifactStats, LockedSummary, LockedSummaryProps};
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
pub use quality_score::{QualityDimension, QualityScore, QualityScoreBar, QualityScoreBarProps};
pub use state::{ConfirmSubPhase, ProgressiveDiscoverPhase};
pub use straw_man::{StrawManTrap, StrawManValidation};
pub use types::{Hole, HolePunchingResults, HoleType, ScenarioField};
