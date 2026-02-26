#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Extraction provider trait and implementations
//!
//! This module defines the interface for extracting structured fields from
//! unstructured text using various AI providers (OpenAI, Claude, local models, etc.).

mod opencode;
mod r#trait;

pub use opencode::OpenCodeProvider;
pub use r#trait::{
    ExtractionContext, ExtractionError, ExtractionMetadata, ExtractionProvider,
    ExtractedFields, FieldExtraction, FieldType, SchemaField,
};
