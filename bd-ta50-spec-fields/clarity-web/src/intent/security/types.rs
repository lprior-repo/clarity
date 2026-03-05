#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use thiserror::Error;

pub type SecurityResult<T> = Result<T, SecurityError>;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SecurityError {
  #[error("path traversal detected: {details}")]
  PathTraversal { details: String },
  #[error("URL-encoded path traversal detected: {encoding_type:?}")]
  EncodedPathTraversal { encoding_type: PathEncodingType },
  #[error("shell metacharacter detected: category={category:?}, char='{ch}'")]
  ShellMetacharacter {
    category: MetacharCategory,
    ch: char,
  },
  #[error("ReDoS vulnerability detected: {vulnerability:?}")]
  ReDoSVulnerability { vulnerability: RegexVulnerability },
  #[error("session ID validation failed: {error:?}")]
  SessionIdValidation { error: SessionIdError },
  #[error("null byte detected in input")]
  NullByteDetected,
  #[error("backslash detected in path (potential Windows traversal)")]
  BackslashInPath,
  #[error("empty input provided")]
  EmptyInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PathEncodingType {
  #[error("single URL encoding")]
  SingleEncoded,
  #[error("double URL encoding")]
  DoubleEncoded,
  #[error("mixed encoding")]
  MixedEncoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MetacharCategory {
  #[error("command separator")]
  CommandSeparator,
  #[error("variable expansion")]
  VariableExpansion,
  #[error("grouping")]
  Grouping,
  #[error("redirection")]
  Redirection,
  #[error("escape or quote")]
  EscapeQuote,
  #[error("control character")]
  ControlCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RegexVulnerability {
  #[error("nested quantifiers")]
  NestedQuantifiers,
  #[error("overlapping wildcards")]
  OverlappingWildcards,
  #[error("alternation overlap")]
  AlternationOverlap,
  #[error("exponential backtracking")]
  ExponentialBacktracking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SessionIdError {
  #[error("session ID exceeds maximum length of {max} characters")]
  TooLong { max: usize },
  #[error("session ID contains invalid character: '{ch}'")]
  InvalidCharacter { ch: char },
  #[error("session ID is empty")]
  Empty,
}
