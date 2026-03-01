#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Product Management Engine - Define Phase (Double Diamond Phase 2)
//!
//! This module implements the Define phase components of the Product Management Engine,
//! converting raw discovery outputs into structured, graph-based requirements.
//!
//! # Components
//!
//! - **Great Reindexing Engine**: Converts time-based stories into graph-based requirements
//! - **Brutal Truths Prioritizer**: Applies Four Brutal Truths framework with VORP scoring

pub mod brutal_truths;
pub mod great_reindexing;

pub use brutal_truths::{
  BrutalTruth, BrutalTruthsOutput, PrioritizedItem, PrioritizerError, VorpCalculator, VorpScore,
};
pub use great_reindexing::{
  GraphRequirement, JobToBeDone, ReindexingError, ReindexingOutput, RequirementEdge,
  RequirementGraph, RequirementNode, StoryInput, UserStory,
};
