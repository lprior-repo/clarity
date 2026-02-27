use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SpecTemplateError {
  #[error("placeholder not found: {0}")]
  PlaceholderNotFound(String),
  #[error("template is empty")]
  EmptyTemplate,
  #[error("session has no answers")]
  NoAnswers,
  #[error("missing required field: {0}")]
  MissingField(String),
  #[error("JSON serialization failed: {0}")]
  JsonError(String),
  #[error("template rendering failed: {0}")]
  RenderingError(String),
}
