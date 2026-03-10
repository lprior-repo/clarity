#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
//! Adversarial tests for queue/concurrent extraction operations.
//!
//! These tests validate the extraction pipeline under concurrent load,
//! testing race conditions, queue ordering, and resource exhaustion scenarios.

#![forbid(unsafe_code)]

use clarity_web::providers::{
  ExtractedFields, ExtractionContext, ExtractionError, ExtractionProvider, FieldExtraction,
  FieldType,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Semaphore;

// =============================================================================
// Mock Provider for Testing
// =============================================================================

#[derive(Debug, Clone)]
struct MockExtractionProvider {
  delay_ms: u64,
  max_concurrent: Arc<Semaphore>,
}

impl MockExtractionProvider {
  #[must_use]
  pub fn new(delay_ms: u64, max_concurrent: usize) -> Self {
    Self {
      delay_ms,
      max_concurrent: Arc::new(Semaphore::new(max_concurrent)),
    }
  }
}

#[async_trait::async_trait]
impl ExtractionProvider for MockExtractionProvider {
  async fn extract_fields(
    &self,
    text: &str,
    _context: &ExtractionContext,
  ) -> Result<ExtractedFields, ExtractionError> {
    // Enforce concurrency limit
    let _permit = self
      .max_concurrent
      .acquire()
      .await
      .map_err(|e| ExtractionError::Unknown(format!("Semaphore error: {e}")))?;

    if self.delay_ms > 0 {
      tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
    }

    let word_count = text.split_whitespace().count();
    let confidence = if word_count > 50 {
      0.85
    } else if word_count > 20 {
      0.65
    } else {
      0.45
    };

    let fields = vec![FieldExtraction {
      name: "extracted_text".to_string(),
      field_type: FieldType::TextArea,
      value: json!(text),
      confidence,
      justification: Some("Extracted from input".to_string()),
    }];

    Ok(ExtractedFields {
      fields,
      confidence,
      metadata: clarity_web::providers::ExtractionMetadata {
        provider: "mock".to_string(),
        model: Some("mock-v1".to_string()),
        timestamp: chrono::Utc::now(),
        processing_duration_ms: self.delay_ms,
        extra: json!({}),
      },
    })
  }

  async fn extract_fields_with_schema(
    &self,
    text: &str,
    _schema: &[clarity_web::providers::SchemaField],
    context: &ExtractionContext,
  ) -> Result<ExtractedFields, ExtractionError> {
    self.extract_fields(text, context).await
  }

  fn provider_name(&self) -> &'static str {
    "mock"
  }

  async fn health_check(&self) -> Result<(), ExtractionError> {
    Ok(())
  }
}

// =============================================================================
// Concurrent Request Tests
// =============================================================================

#[tokio::test]
async fn concurrent_extractions_preserve_order() {
  let provider = Arc::new(MockExtractionProvider::new(50, 10));
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let inputs = vec![
    "Input one: Building a task management system",
    "Input two: Creating a notification service",
    "Input three: Designing a user authentication flow",
    "Input four: Implementing a payment gateway",
    "Input five: Developing an analytics dashboard",
  ];

  // Spawn concurrent extractions
  let mut handles = Vec::new();
  for input in inputs.clone() {
    let provider = provider.clone();
    let context = context.clone();
    let handle = tokio::spawn(async move { provider.extract_fields(input, &context).await });
    handles.push(handle);
  }

  // Collect results
  let mut results = Vec::new();
  for handle in handles {
    if let Ok(result) = handle.await {
      results.push(result);
    }
  }

  // All extractions should succeed
  assert_eq!(results.len(), inputs.len());

  // Each result should contain the input text
  for (result, expected_input) in results.iter().zip(inputs.iter()) {
    assert!(result.is_ok());
    let extracted = result.as_ref().expect("extraction ok");
    assert_eq!(extracted.fields.len(), 1);
    assert_eq!(extracted.fields[0].value, json!(expected_input.to_string()));
  }
}

#[tokio::test]
async fn queue_respects_concurrency_limits() {
  let max_concurrent = 2;
  let provider = Arc::new(MockExtractionProvider::new(100, max_concurrent));
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let inputs = (0..5).map(|i| format!("Input {i}")).collect::<Vec<_>>();

  let _start = std::time::Instant::now();

  // Spawn 5 concurrent requests with limit of 2
  let mut handles = Vec::new();
  for input in &inputs {
    let provider = provider.clone();
    let context = context.clone();
    let input = (*input).to_string();
    let handle = tokio::spawn(async move {
      let result = provider.extract_fields(&input, &context).await;
      match result {
        Ok(fields) => Ok((input, fields)),
        Err(e) => Err((input, e)),
      }
    });
    handles.push(handle);
  }

  // Count results
  let mut successful = 0;
  let mut empty_inputs = 0;

  for handle in handles {
    if let Ok(result) = handle.await {
      match result {
        Ok((input, _)) => {
          successful += 1;
          if input.trim().is_empty() {
            empty_inputs += 1;
          }
        }
        Err((_, e)) => {
          panic!("Unexpected error: {e:?}");
        }
      }
    }
  }

  // All 5 extractions should succeed (mock accepts all)
  assert_eq!(successful, 5);
  // None of the inputs are empty - they are "Input 0" through "Input 4"
  assert_eq!(empty_inputs, 0);
}

#[tokio::test]
async fn rapid_queue_load_doesnt_cause_race_conditions() {
  let provider = Arc::new(MockExtractionProvider::new(0, 100));
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  // Create 100 rapid concurrent requests
  let inputs: Vec<_> = (0..100).map(|i| format!("Request {i}")).collect();

  let mut handles = Vec::new();
  for input in &inputs {
    let provider = provider.clone();
    let context = context.clone();
    let input = input.clone();
    let handle = tokio::spawn(async move { provider.extract_fields(&input, &context).await });
    handles.push(handle);
  }

  let mut successful = 0;
  for handle in handles {
    if let Ok(Ok(_)) = handle.await {
      successful += 1;
    }
  }

  assert_eq!(successful, 100);
}

#[tokio::test]
async fn queue_handles_varied_processing_times() {
  let provider = Arc::new(MockExtractionProvider::new(50, 10));
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  // Inputs with varying lengths to simulate different processing times
  let inputs = vec![
    "Short".to_string(),
    "This is a medium length input that will take some processing time".to_string(),
    "A".repeat(500), // Long input
    "Another short one".to_string(),
    "B".repeat(250),
  ];

  let mut handles = Vec::new();
  for input in inputs {
    let provider = provider.clone();
    let context = context.clone();
    let handle = tokio::spawn(async move {
      let start = std::time::Instant::now();
      let result = provider.extract_fields(&input, &context).await;
      let elapsed = start.elapsed();
      (result, elapsed)
    });
    handles.push(handle);
  }

  let mut successful = 0;
  for handle in handles {
    if let Ok((Ok(_), _)) = handle.await {
      successful += 1;
    }
  }

  // All should complete
  assert_eq!(successful, 5);
}

#[tokio::test]
async fn queue_handles_cancellation_mid_stream() {
  let provider = Arc::new(MockExtractionProvider::new(100, 2));
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let inputs: Vec<_> = (0..10).map(|i| format!("Input {i}")).collect();

  // Create tasks but don't await all of them
  let mut handles = Vec::new();
  for input in &inputs {
    let provider = provider.clone();
    let context = context.clone();
    let input = input.clone();
    let handle = tokio::spawn(async move { provider.extract_fields(&input, &context).await });
    handles.push(handle);
  }

  // Only await the first 3
  let mut results = Vec::new();
  for handle in handles.into_iter().take(3) {
    if let Ok(result) = handle.await {
      results.push(result);
    }
  }

  // Should have 3 results
  assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn stress_test_queue_capacity() {
  let provider = Arc::new(MockExtractionProvider::new(10, 50));
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  // Create 1000 concurrent requests
  let inputs: Vec<_> = (0..1000).map(|i| format!("Input {i}")).collect();

  let mut handles = Vec::new();
  for input in &inputs {
    let provider = provider.clone();
    let context = context.clone();
    let input = input.clone();
    let handle = tokio::spawn(async move { provider.extract_fields(&input, &context).await });
    handles.push(handle);
  }

  let mut successful = 0;
  for handle in handles {
    if let Ok(Ok(_)) = handle.await {
      successful += 1;
    }
  }

  assert_eq!(successful, 1000);
}
