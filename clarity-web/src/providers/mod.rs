#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Extraction provider trait and implementations
//!
//! This module defines the interface for extracting structured fields from
//! unstructured text using various AI providers (`OpenAI`, `Claude`, local models, etc.).

#[cfg(not(target_arch = "wasm32"))]
mod opencode;
mod r#trait;

#[cfg(not(target_arch = "wasm32"))]
pub use opencode::{OpenCodeProvider, OpenCodeProviderOptions};
pub use r#trait::{
  ExtractedFields, ExtractionContext, ExtractionError, ExtractionMetadata, ExtractionProvider,
  FieldExtraction, FieldType, SchemaField,
};
