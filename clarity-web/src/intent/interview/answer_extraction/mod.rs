//! Answer Extraction
//!
//! This module provides functionality to extract structured data from free-text
//! interview answers. It parses various data types from natural language responses,
//! enabling the conversion of unstructured user input into typed, validated values.
//!
//! # Overview
//!
//! The extraction system supports multiple data types:
//! - **Text**: Raw string extraction with whitespace trimming
//! - **Name**: Single-line text (first line only)
//! - **Integer**: Whole numbers extracted from surrounding text
//! - **Float**: Decimal numbers extracted from surrounding text
//! - **Boolean**: True/false values from various formats (yes/no, true/false, 1/0)
//! - **URL**: HTTP/HTTPS URLs extracted from text
//! - **Email**: Email addresses extracted from text
//! - **List**: Items from comma-separated, newline-separated, or numbered lists
//!
//! # Architecture
//!
//! The module is organized into focused submodules:
//! - [`types`] - Error and value types
//! - [`extractors`] - Core extraction functions for each data type
//! - [`parsers`] - List parsing utilities
//! - [`interview`] - Interview-specific extraction functions
//!
//! # Example Usage
//!
//! ```ignore
//! use clarity_web::intent::interview::answer_extraction::{
//!     extract_by_type, ExtractedValue,
//! };
//!
//! // Extract typed values from natural language
//! let response = "The project uses 42 services";
//! let value = extract_by_type(response, "integer");
//! assert_eq!(value, Ok(ExtractedValue::Integer(42)));
//!
//! // Extract URLs from text
//! let response = "Visit https://example.com for more info";
//! let value = extract_by_type(response, "url");
//! assert_eq!(value, Ok(ExtractedValue::Url("https://example.com".to_string())));
//! ```
//!
//! # Interview-Specific Extraction
//!
//! For interview responses, use [`interview::extract_from_answer`] which provides specialized
//! extraction patterns for common fields like `auth_method`, `entities`, and `audience`.
//!
//! # Error Handling
//!
//! All fallible operations return `Result<T, ExtractionError>`. The error type
//! provides detailed information about what went wrong during extraction.

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod types;
pub mod extractors;
pub mod parsers;
pub mod helpers;
pub mod interview;

pub mod tests;

pub use types::{ExtractedValue, ExtractionError};
pub use extractors::{
  extract_by_type, extract_boolean, extract_email, extract_fields, extract_fields_with_types,
  extract_float, extract_integer, extract_list, extract_name, extract_text, extract_url,
};
pub use interview::{calculate_confidence, extract_from_answer};
