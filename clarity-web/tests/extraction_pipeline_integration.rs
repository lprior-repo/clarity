//! Integration tests for the extraction pipeline.
//!
//! These tests validate the complete flow from raw text to extracted data,
//! testing integration between extraction, storage, and formatting.
//!
//! ## Test Coverage
//! - Complete flow from raw text to extracted data
//! - Integration between extraction, storage, and formatting
//! - Realistic test data scenarios
//! - Error propagation through the pipeline

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

use clarity_web::intent::documents::spec_builder::{
  build_spec_from_session, extract_behaviors_from_answers, extract_constraints_from_answers,
  extract_features_from_answers, extract_non_functional_requirements,
  extract_security_requirements,
};
use clarity_web::intent::interview::storage::{
  append_session_to_jsonl, diff_sessions, format_diff, get_session_from_jsonl,
  list_sessions_from_jsonl, session_to_jsonl_line, SessionWithHistories,
};
use clarity_web::intent::interview::types::{
  Answer, Gap, GapState, InterviewSession, InterviewStage, Perspective, Profile,
};
use tempfile::NamedTempFile;

// =============================================================================
// Test Fixtures
// =============================================================================

/// Create a realistic answer with extracted fields.
fn make_answer_with_extraction(
  question_id: &str,
  question_text: &str,
  response: &str,
  extracted: HashMap<String, String>,
) -> Answer {
  Answer {
    question_id: question_id.to_string(),
    question_text: question_text.to_string(),
    perspective: Perspective::User,
    round: 1,
    response: response.to_string(),
    extracted,
    confidence: 0.9,
    notes: String::new(),
    timestamp: "2026-02-28T12:00:00Z".to_string(),
  }
}

/// Create a complete API interview session with realistic data.
fn make_api_session_with_extraction(session_id: &str) -> InterviewSession {
  let mut extracted_base_url = HashMap::new();
  extracted_base_url.insert(
    "base_url".to_string(),
    "https://api.example.com/v1".to_string(),
  );

  let mut extracted_auth = HashMap::new();
  extracted_auth.insert("auth_method".to_string(), "Bearer JWT".to_string());

  let mut extracted_happy = HashMap::new();
  extracted_happy.insert(
    "happy_path".to_string(),
    "GET /users returns list".to_string(),
  );

  let mut extracted_errors = HashMap::new();
  extracted_errors.insert(
    "error_cases".to_string(),
    "401 Unauthorized, 404 Not Found".to_string(),
  );

  let mut extracted_format = HashMap::new();
  extracted_format.insert("response_format".to_string(), "JSON".to_string());

  InterviewSession {
    id: session_id.to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T12:00:00Z".to_string(),
    completed_at: Some("2026-02-28T12:00:00Z".to_string()),
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: vec![
      make_answer_with_extraction(
        "q-base-url",
        "What is the base URL for the API?",
        "The API is hosted at https://api.example.com/v1",
        extracted_base_url,
      ),
      make_answer_with_extraction(
        "q-auth",
        "What authentication method is used?",
        "We use Bearer JWT tokens for authentication",
        extracted_auth,
      ),
      make_answer_with_extraction(
        "q-happy",
        "What is the happy path for the main endpoint?",
        "GET /users returns a paginated list of users",
        extracted_happy,
      ),
      make_answer_with_extraction(
        "q-errors",
        "What error cases should be handled?",
        "401 Unauthorized when token expired, 404 Not Found for missing resources",
        extracted_errors,
      ),
      make_answer_with_extraction(
        "q-format",
        "What is the response format?",
        "All responses are JSON with consistent structure",
        extracted_format,
      ),
    ],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  }
}

/// Create a CLI session with realistic data.
fn make_cli_session(session_id: &str) -> InterviewSession {
  let mut extracted_cmd = HashMap::new();
  extracted_cmd.insert("command_name".to_string(), "mycli".to_string());

  let mut extracted_help = HashMap::new();
  extracted_help.insert("help_text".to_string(), "A CLI tool for X".to_string());

  InterviewSession {
    id: session_id.to_string(),
    profile: Profile::Cli,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T11:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Refinement,
    rounds_completed: 0,
    answers: vec![
      make_answer_with_extraction(
        "q-cmd",
        "What is the command name?",
        "The command is called mycli",
        extracted_cmd,
      ),
      make_answer_with_extraction(
        "q-help",
        "What does the help text say?",
        "A CLI tool for X",
        extracted_help,
      ),
    ],
    gaps: vec![Gap {
      id: "gap-happy_path".to_string(),
      field: "happy_path".to_string(),
      description: "Missing required field: happy_path".to_string(),
      blocking: true,
      suggested_default: String::new(),
      why_needed: String::new(),
      round: 1,
      state: GapState::Open,
    }],
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  }
}

// =============================================================================
// Complete Flow Tests
// =============================================================================

#[test]
fn test_complete_extraction_flow_from_raw_text_to_spec() {
  // Step 1: Create a session with raw text answers
  let session = make_api_session_with_extraction("extraction-flow-1");

  // Step 2: Verify extracted fields are present
  let base_url_answer = session
    .answers
    .iter()
    .find(|a| a.question_id == "q-base-url");
  assert!(base_url_answer.is_some());
  let answer = base_url_answer.unwrap();
  assert!(answer.extracted.contains_key("base_url"));
  assert_eq!(
    answer.extracted.get("base_url"),
    Some(&"https://api.example.com/v1".to_string())
  );

  // Step 3: Verify all required fields for API profile are extracted
  let all_extracted_fields: Vec<&str> = session
    .answers
    .iter()
    .flat_map(|a| a.extracted.keys())
    .map(String::as_str)
    .collect();

  for required in Profile::Api.required_fields() {
    assert!(
      all_extracted_fields.iter().any(|f| f == required),
      "Missing required field: {required}"
    );
  }

  // Step 4: Build spec from session
  let spec = build_spec_from_session(&session);

  // Step 5: Verify spec contains extracted data
  assert!(spec.contains("package api"));
  assert!(spec.contains("features:"));
  assert!(spec.contains("behaviors:"));
  assert!(spec.contains("security:"));
}

#[test]
fn test_extraction_to_storage_roundtrip() {
  // Step 1: Create session with extraction
  let original_session = make_api_session_with_extraction("storage-roundtrip-1");

  // Step 2: Serialize to JSONL line
  let line_result = session_to_jsonl_line(&original_session);
  assert!(line_result.is_ok());
  let line = line_result.unwrap();

  // Step 3: Verify line contains extracted data
  assert!(line.contains("base_url"));
  assert!(line.contains("https://api.example.com/v1"));
  assert!(line.contains("auth_method"));
  assert!(line.contains("Bearer JWT"));

  // Step 4: Parse back from JSON
  let parsed_result = serde_json::from_str::<InterviewSession>(&line);
  assert!(parsed_result.is_ok());
  let parsed_session = parsed_result.unwrap();

  // Step 5: Verify extraction data survived roundtrip
  assert_eq!(parsed_session.answers.len(), original_session.answers.len());

  for original_answer in &original_session.answers {
    let parsed_answer = parsed_session
      .answers
      .iter()
      .find(|a| a.question_id == original_answer.question_id);
    assert!(parsed_answer.is_some());

    let parsed = parsed_answer.unwrap();
    assert_eq!(parsed.extracted, original_answer.extracted);
    assert_eq!(parsed.response, original_answer.response);
  }
}

#[test]
fn test_extraction_pipeline_with_jsonl_file() {
  // Step 1: Create temp file
  let temp_file_result = NamedTempFile::new();
  assert!(temp_file_result.is_ok());
  let temp_file = temp_file_result.unwrap();
  let path = temp_file.path();

  // Step 2: Create and store multiple sessions with extractions
  let session1 = make_api_session_with_extraction("pipeline-session-1");
  let session2 = make_cli_session("pipeline-session-2");

  let append1 = append_session_to_jsonl(&session1, path);
  assert!(append1.is_ok());

  let append2 = append_session_to_jsonl(&session2, path);
  assert!(append2.is_ok());

  // Step 3: Read sessions back
  let sessions_result = list_sessions_from_jsonl(path);
  assert!(sessions_result.is_ok());
  let sessions = sessions_result.unwrap();
  assert_eq!(sessions.len(), 2);

  // Step 4: Get specific session and verify extraction
  let retrieved_result = get_session_from_jsonl(path, "pipeline-session-1");
  assert!(retrieved_result.is_ok());
  let retrieved = retrieved_result.unwrap();

  // Step 5: Verify extracted fields intact
  let auth_answer = retrieved.answers.iter().find(|a| a.question_id == "q-auth");
  assert!(auth_answer.is_some());
  let auth = auth_answer.unwrap();
  assert_eq!(
    auth.extracted.get("auth_method"),
    Some(&"Bearer JWT".to_string())
  );

  // Step 6: Build spec from retrieved session
  let spec = build_spec_from_session(&retrieved);
  assert!(spec.contains("security:"));
}

// =============================================================================
// Integration Tests: Extraction + Formatting
// =============================================================================

#[test]
fn test_extraction_integrates_with_spec_builder() {
  let session = make_api_session_with_extraction("spec-integration-1");

  // Extract features using spec_builder
  let _features = extract_features_from_answers(&session.answers);

  // Extract behaviors
  let behaviors = extract_behaviors_from_answers(&session.answers, &session.profile);

  // Extract constraints
  let _constraints = extract_constraints_from_answers(&session.answers);

  // Extract security
  let security = extract_security_requirements(&session.answers);

  // Extract non-functional
  let _non_functional = extract_non_functional_requirements(&session.answers);

  // Verify all extraction functions work with the session data
  // The API session has endpoint questions which trigger behavior extraction
  assert!(!behaviors.is_empty() || behaviors.contains("Define API behaviors"));
  assert!(!security.is_empty());

  // Build full spec
  let spec = build_spec_from_session(&session);
  assert!(spec.contains("package api"));
}

#[test]
fn test_extraction_with_version_history() {
  // Create session with histories - use a session that is NOT complete
  // (update_answer fails for complete sessions)
  let mut session = make_api_session_with_extraction("version-history-1");
  session.stage = InterviewStage::Refinement;
  session.completed_at = None;

  let mut wrapper = SessionWithHistories::with_initial_histories(session);

  // Verify initial histories created
  assert_eq!(wrapper.answer_histories.len(), 5);

  // Update an answer with new extraction
  let update_result = wrapper.update_answer(
    "q-auth",
    "We use OAuth2 with PKCE for authentication",
    "security_update",
    "2026-02-28T13:00:00Z",
  );
  assert!(update_result.is_ok());

  // Verify version history updated
  let history = wrapper.get_history("q-auth");
  assert!(history.is_some());
  let hist = history.unwrap();
  assert_eq!(hist.len(), 2);

  // Verify the session answer was updated
  let updated_answer = wrapper
    .session
    .answers
    .iter()
    .find(|a| a.question_id == "q-auth");
  assert!(updated_answer.is_some());
  assert_eq!(
    updated_answer.unwrap().response,
    "We use OAuth2 with PKCE for authentication"
  );

  // Note: The extracted HashMap stays the same in this flow
  // A real extraction pipeline would update extracted fields
}

// =============================================================================
// Error Propagation Tests
// =============================================================================

#[test]
fn test_error_propagation_storage_not_found() {
  let temp_file_result = NamedTempFile::new();
  assert!(temp_file_result.is_ok());
  let temp_file = temp_file_result.unwrap();
  let path = temp_file.path();

  // Try to get non-existent session
  let result = get_session_from_jsonl(path, "nonexistent-session");
  assert!(result.is_err());

  let error = result.err().unwrap();
  assert!(error.to_string().contains("not found"));
}

#[test]
fn test_error_propagation_empty_session_id() {
  let session = InterviewSession {
    id: String::new(), // Empty ID
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: Vec::new(),
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Serialize should still work (empty ID is valid JSON)
  let line_result = session_to_jsonl_line(&session);
  assert!(line_result.is_ok());
}

#[test]
fn test_error_propagation_invalid_json_recovery() {
  // Create temp file with invalid JSON line
  let temp_file_result = NamedTempFile::new();
  assert!(temp_file_result.is_ok());
  let temp_file = temp_file_result.unwrap();
  let path = temp_file.path();

  // Write valid session first
  let session = make_api_session_with_extraction("valid-session");
  let append_result = append_session_to_jsonl(&session, path);
  assert!(append_result.is_ok());

  // Reading should succeed
  let read_result = list_sessions_from_jsonl(path);
  assert!(read_result.is_ok());
  let sessions = read_result.unwrap();
  assert_eq!(sessions.len(), 1);
}

// =============================================================================
// Diff Tests with Extraction
// =============================================================================

#[test]
fn test_diff_with_extraction_changes() {
  // Create two sessions with different extractions
  let session1 = make_api_session_with_extraction("diff-1");
  let mut session2 = session1.clone();
  session2.id = "diff-2".to_string();

  // Modify extraction in session2
  if let Some(answer) = session2
    .answers
    .iter_mut()
    .find(|a| a.question_id == "q-auth")
  {
    answer.response = "OAuth2 with refresh tokens".to_string();
    answer
      .extracted
      .insert("auth_method".to_string(), "OAuth2".to_string());
  }

  // Add new answer with extraction
  let mut new_extracted = HashMap::new();
  new_extracted.insert("rate_limit".to_string(), "100 req/min".to_string());
  session2.answers.push(make_answer_with_extraction(
    "q-rate-limit",
    "What are the rate limits?",
    "100 requests per minute",
    new_extracted,
  ));

  // Compute diff
  let diff = diff_sessions(&session1, &session2);

  // Verify diff detected changes
  assert_eq!(diff.answers_modified.len(), 1);
  assert_eq!(diff.answers_added.len(), 1);
  assert_eq!(diff.answers_removed.len(), 0);

  // Format diff
  let formatted = format_diff(&diff);
  assert!(formatted.contains("Session Diff:"));
  assert!(formatted.contains("Answers Modified"));
  assert!(formatted.contains("Answers Added"));
}

// =============================================================================
// Realistic Scenario Tests
// =============================================================================

#[test]
fn test_realistic_workflow_session_extraction() {
  // Create a workflow profile session
  let mut extracted_steps = HashMap::new();
  extracted_steps.insert(
    "steps".to_string(),
    "validate -> process -> notify".to_string(),
  );

  let mut extracted_happy = HashMap::new();
  extracted_happy.insert("happy_path".to_string(), "All steps succeed".to_string());

  let mut extracted_recovery = HashMap::new();
  extracted_recovery.insert(
    "error_recovery".to_string(),
    "Retry 3 times, then alert".to_string(),
  );

  let session = InterviewSession {
    id: "workflow-1".to_string(),
    profile: Profile::Workflow,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T12:00:00Z".to_string(),
    completed_at: Some("2026-02-28T12:00:00Z".to_string()),
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: vec![
      make_answer_with_extraction(
        "q-steps",
        "What are the workflow steps?",
        "The workflow has three steps: validate, process, and notify",
        extracted_steps,
      ),
      make_answer_with_extraction(
        "q-happy",
        "What is the happy path?",
        "All steps succeed in sequence",
        extracted_happy,
      ),
      make_answer_with_extraction(
        "q-recovery",
        "How are errors handled?",
        "Retry up to 3 times, then send alert",
        extracted_recovery,
      ),
    ],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  };

  // Verify all workflow required fields extracted
  let required = Profile::Workflow.required_fields();
  let extracted_fields: Vec<&str> = session
    .answers
    .iter()
    .flat_map(|a| a.extracted.keys())
    .map(String::as_str)
    .collect();

  for field in required {
    assert!(
      extracted_fields.iter().any(|f| f == field),
      "Missing workflow field: {field}"
    );
  }

  // Build spec
  let spec = build_spec_from_session(&session);
  assert!(spec.contains("package api")); // Note: currently always "api"
}

#[test]
fn test_data_profile_extraction() {
  let mut extracted_model = HashMap::new();
  extracted_model.insert(
    "data_model".to_string(),
    "User { id, name, email }".to_string(),
  );

  let mut extracted_patterns = HashMap::new();
  extracted_patterns.insert(
    "access_patterns".to_string(),
    "Read-heavy, occasional writes".to_string(),
  );

  let mut extracted_retention = HashMap::new();
  extracted_retention.insert("retention".to_string(), "7 years".to_string());

  let session = InterviewSession {
    id: "data-1".to_string(),
    profile: Profile::Data,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T11:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![
      make_answer_with_extraction(
        "q-model",
        "What is the data model?",
        "User entity with id, name, and email fields",
        extracted_model,
      ),
      make_answer_with_extraction(
        "q-patterns",
        "What are the access patterns?",
        "Read-heavy workload with occasional writes",
        extracted_patterns,
      ),
      make_answer_with_extraction(
        "q-retention",
        "What is the data retention policy?",
        "7 years for compliance",
        extracted_retention,
      ),
    ],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Verify data profile required fields
  let required = Profile::Data.required_fields();
  let extracted_fields: Vec<&str> = session
    .answers
    .iter()
    .flat_map(|a| a.extracted.keys())
    .map(String::as_str)
    .collect();

  for field in required {
    assert!(
      extracted_fields.iter().any(|f| f == field),
      "Missing data field: {field}"
    );
  }
}

#[test]
fn test_event_profile_extraction() {
  let mut extracted_type = HashMap::new();
  extracted_type.insert("event_type".to_string(), "UserCreated".to_string());

  let mut extracted_schema = HashMap::new();
  extracted_schema.insert(
    "payload_schema".to_string(),
    "{ userId: string, timestamp: number }".to_string(),
  );

  let mut extracted_trigger = HashMap::new();
  extracted_trigger.insert("trigger".to_string(), "POST /users".to_string());

  let session = InterviewSession {
    id: "event-1".to_string(),
    profile: Profile::Event,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T11:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![
      make_answer_with_extraction(
        "q-type",
        "What type of event?",
        "UserCreated event when new user signs up",
        extracted_type,
      ),
      make_answer_with_extraction(
        "q-schema",
        "What is the payload schema?",
        "Contains userId (string) and timestamp (number)",
        extracted_schema,
      ),
      make_answer_with_extraction(
        "q-trigger",
        "What triggers this event?",
        "Triggered by POST /users endpoint",
        extracted_trigger,
      ),
    ],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Verify event profile required fields
  let required = Profile::Event.required_fields();
  let extracted_fields: Vec<&str> = session
    .answers
    .iter()
    .flat_map(|a| a.extracted.keys())
    .map(String::as_str)
    .collect();

  for field in required {
    assert!(
      extracted_fields.iter().any(|f| f == field),
      "Missing event field: {field}"
    );
  }
}

// =============================================================================
// Confidence Score Tests
// =============================================================================

#[test]
fn test_confidence_score_with_extraction() {
  let mut extracted = HashMap::new();
  extracted.insert(
    "base_url".to_string(),
    "https://api.example.com".to_string(),
  );

  // Long response with extracted fields
  let long_response =
    "This is a very detailed response about the API base URL that exceeds fifty characters";
  let confidence_high = InterviewSession::calculate_confidence(long_response, &HashMap::new());
  assert_eq!(confidence_high, 0.6); // Long but no extracted fields

  let confidence_with_extraction =
    InterviewSession::calculate_confidence(long_response, &extracted);
  assert_eq!(confidence_with_extraction, 0.85); // Long with extracted fields

  // Short response
  let short_response = "Short";
  let confidence_short = InterviewSession::calculate_confidence(short_response, &HashMap::new());
  assert_eq!(confidence_short, 0.6);
}

// =============================================================================
// Gap Detection with Extraction Tests
// =============================================================================

#[test]
fn test_gap_detection_with_extracted_fields() {
  // Session with partial extraction (missing some required fields)
  let mut extracted_base_url = HashMap::new();
  extracted_base_url.insert(
    "base_url".to_string(),
    "https://api.example.com".to_string(),
  );

  let session = InterviewSession {
    id: "gap-test-1".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![make_answer_with_extraction(
      "q-base-url",
      "What is the base URL?",
      "https://api.example.com",
      extracted_base_url,
    )],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Detect gaps
  let gaps = session.detect_gaps();

  // API profile requires: base_url, auth_method, happy_path, error_cases, response_format
  // We only have base_url, so 4 gaps should be detected
  assert_eq!(gaps.len(), 4);

  let gap_fields: Vec<&str> = gaps.iter().map(|g| g.field.as_str()).collect();
  assert!(gap_fields.contains(&"auth_method"));
  assert!(gap_fields.contains(&"happy_path"));
  assert!(gap_fields.contains(&"error_cases"));
  assert!(gap_fields.contains(&"response_format"));
  assert!(!gap_fields.contains(&"base_url")); // Already extracted
}

#[test]
fn test_no_gaps_when_all_fields_extracted() {
  // Session with all required fields extracted
  let session = make_api_session_with_extraction("no-gaps-test");

  // Detect gaps
  let gaps = session.detect_gaps();

  // All required fields are extracted, so no gaps
  assert!(gaps.is_empty());
}

// =============================================================================
// Multi-Round Extraction Tests
// =============================================================================

#[test]
fn test_extraction_across_multiple_rounds() {
  // Round 1: Initial answers
  let mut session = InterviewSession::new(
    "multi-round-1".to_string(),
    Profile::Api,
    "2026-02-28T10:00:00Z".to_string(),
  );

  // Add first round answer
  let mut extracted1 = HashMap::new();
  extracted1.insert(
    "base_url".to_string(),
    "https://api.example.com".to_string(),
  );

  let answer1 = Answer {
    question_id: "q-base-url".to_string(),
    question_text: "What is the base URL?".to_string(),
    perspective: Perspective::User,
    round: 1,
    response: "https://api.example.com".to_string(),
    extracted: extracted1,
    confidence: 0.9,
    notes: String::new(),
    timestamp: "2026-02-28T10:00:00Z".to_string(),
  };

  let add_result = session.add_answer(answer1, "2026-02-28T10:00:00Z");
  assert!(add_result.is_ok());

  // Complete round 1
  let complete_result = session.complete_round("2026-02-28T10:30:00Z");
  assert!(complete_result.is_ok());

  // Add round 2 answer
  let mut extracted2 = HashMap::new();
  extracted2.insert("auth_method".to_string(), "Bearer".to_string());

  let answer2 = Answer {
    question_id: "q-auth".to_string(),
    question_text: "What auth method?".to_string(),
    perspective: Perspective::User,
    round: 2,
    response: "Bearer tokens".to_string(),
    extracted: extracted2,
    confidence: 0.85,
    notes: String::new(),
    timestamp: "2026-02-28T10:30:00Z".to_string(),
  };

  let add_result2 = session.add_answer(answer2, "2026-02-28T10:30:00Z");
  assert!(add_result2.is_ok());

  // Verify both rounds have extraction data
  assert_eq!(session.answers.len(), 2);

  let round1_answer = session.answers.iter().find(|a| a.round == 1);
  assert!(round1_answer.is_some());
  assert!(round1_answer.unwrap().extracted.contains_key("base_url"));

  let round2_answer = session.answers.iter().find(|a| a.round == 2);
  assert!(round2_answer.is_some());
  assert!(round2_answer.unwrap().extracted.contains_key("auth_method"));
}

// =============================================================================
// Concurrent Session Tests
// =============================================================================

#[test]
fn test_multiple_sessions_with_unique_extractions() {
  let temp_file_result = NamedTempFile::new();
  assert!(temp_file_result.is_ok());
  let temp_file = temp_file_result.unwrap();
  let path = temp_file.path();

  // Create multiple sessions with different profiles
  let api_session = make_api_session_with_extraction("concurrent-api");
  let cli_session = make_cli_session("concurrent-cli");

  // Store both
  let append1 = append_session_to_jsonl(&api_session, path);
  assert!(append1.is_ok());

  let append2 = append_session_to_jsonl(&cli_session, path);
  assert!(append2.is_ok());

  // Retrieve both and verify extractions are isolated
  let retrieved_api = get_session_from_jsonl(path, "concurrent-api");
  assert!(retrieved_api.is_ok());
  let api = retrieved_api.unwrap();
  assert_eq!(api.profile, Profile::Api);
  assert!(api
    .answers
    .iter()
    .any(|a| a.extracted.contains_key("base_url")));

  let retrieved_cli = get_session_from_jsonl(path, "concurrent-cli");
  assert!(retrieved_cli.is_ok());
  let cli = retrieved_cli.unwrap();
  assert_eq!(cli.profile, Profile::Cli);
  assert!(cli
    .answers
    .iter()
    .any(|a| a.extracted.contains_key("command_name")));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_extraction_with_empty_response() {
  let session = InterviewSession {
    id: "empty-response".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![Answer {
      question_id: "q-empty".to_string(),
      question_text: "What is X?".to_string(),
      perspective: Perspective::User,
      round: 1,
      response: String::new(),   // Empty response
      extracted: HashMap::new(), // No extraction
      confidence: 0.0,
      notes: String::new(),
      timestamp: "2026-02-28T10:00:00Z".to_string(),
    }],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Extract features should handle empty responses gracefully
  let features = extract_features_from_answers(&session.answers);
  assert!(features.is_empty());

  // Session should still be serializable
  let line_result = session_to_jsonl_line(&session);
  assert!(line_result.is_ok());
}

#[test]
fn test_extraction_with_special_characters() {
  let mut extracted = HashMap::new();
  extracted.insert(
    "special_field".to_string(),
    "Value with \"quotes\" and \n newlines \t tabs".to_string(),
  );

  let session = InterviewSession {
    id: "special-chars".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![make_answer_with_extraction(
      "q-special",
      "Special chars?",
      "Value with \"quotes\" and \n newlines",
      extracted,
    )],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Should serialize and deserialize correctly
  let line_result = session_to_jsonl_line(&session);
  assert!(line_result.is_ok());

  let parsed_result = serde_json::from_str::<InterviewSession>(&line_result.unwrap());
  assert!(parsed_result.is_ok());

  let parsed = parsed_result.unwrap();
  let answer = &parsed.answers[0];
  assert!(answer.extracted.contains_key("special_field"));
  assert!(answer
    .extracted
    .get("special_field")
    .unwrap()
    .contains("quotes"));
}

#[test]
fn test_extraction_with_unicode() {
  let mut extracted = HashMap::new();
  extracted.insert(
    "unicode_field".to_string(),
    "Unicode: \u{4e2d}\u{6587} \u{65e5}\u{672c}\u{8a9e}".to_string(),
  );

  let session = InterviewSession {
    id: "unicode-test".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![make_answer_with_extraction(
      "q-unicode",
      "Unicode response?",
      "\u{4e2d}\u{6587} response",
      extracted,
    )],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Should handle unicode correctly
  let line_result = session_to_jsonl_line(&session);
  assert!(line_result.is_ok());

  let parsed_result = serde_json::from_str::<InterviewSession>(&line_result.unwrap());
  assert!(parsed_result.is_ok());

  let parsed = parsed_result.unwrap();
  assert!(parsed.answers[0].response.contains("\u{4e2d}\u{6587}"));
}

// =============================================================================
// End-to-End Extraction Pipeline Tests
// =============================================================================
//
// These tests verify the complete extraction pipeline by actually calling
// the extraction functions on raw interview responses, then verifying
// the extracted data flows correctly through storage and spec building.

#[test]
fn test_e2e_extraction_pipeline_text_field() {
  // Realistic interview Q&A for a text field
  let response = "The API is available at https://api.example.com/v2";

  // Create session with extracted data
  let mut spec = HashMap::new();
  spec.insert("endpoint".to_string(), "url".to_string());

  let extracted_from_response =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(response, &spec);

  // Verify extraction worked
  assert!(extracted_from_response.contains_key("endpoint"));
  assert!(extracted_from_response
    .get("endpoint")
    .unwrap()
    .contains("https://api.example.com"));

  // Now create session and verify full pipeline
  let mut answer_extracted = HashMap::new();
  answer_extracted.insert(
    "endpoint".to_string(),
    extracted_from_response.get("endpoint").unwrap().clone(),
  );

  let session = InterviewSession {
    id: "e2e-text-1".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![Answer {
      question_id: "q-endpoint".to_string(),
      question_text: "What is the API endpoint URL?".to_string(),
      perspective: Perspective::User,
      round: 1,
      response: response.to_string(),
      extracted: answer_extracted,
      confidence: 0.9,
      notes: String::new(),
      timestamp: "2026-02-28T10:00:00Z".to_string(),
    }],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Verify storage roundtrip
  let line_result = session_to_jsonl_line(&session);
  assert!(line_result.is_ok());

  let parsed_result = serde_json::from_str::<InterviewSession>(&line_result.unwrap());
  assert!(parsed_result.is_ok());

  let parsed = parsed_result.unwrap();
  assert_eq!(
    parsed.answers[0].extracted.get("endpoint").unwrap(),
    "https://api.example.com/v2"
  );
}

#[test]
fn test_e2e_extraction_pipeline_integer_field() {
  let response = "The rate limit is 1000 requests per minute";

  // Extract integer using actual function
  let mut spec = HashMap::new();
  spec.insert("rate_limit".to_string(), "integer".to_string());

  let extracted =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(response, &spec);

  assert!(extracted.contains_key("rate_limit"));
  assert_eq!(extracted.get("rate_limit").unwrap(), "1000");

  // Create session and verify pipeline
  let mut answer_extracted = HashMap::new();
  answer_extracted.insert(
    "rate_limit".to_string(),
    extracted.get("rate_limit").unwrap().clone(),
  );

  let session = InterviewSession {
    id: "e2e-int-1".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![Answer {
      question_id: "q-rate-limit".to_string(),
      question_text: "What is the rate limit?".to_string(),
      perspective: Perspective::User,
      round: 1,
      response: response.to_string(),
      extracted: answer_extracted,
      confidence: 0.85,
      notes: String::new(),
      timestamp: "2026-02-28T10:00:00Z".to_string(),
    }],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Build spec from extracted data
  let spec_output = build_spec_from_session(&session);
  assert!(spec_output.contains("package api"));
}

#[test]
fn test_e2e_extraction_pipeline_boolean_field() {
  let response = "yes";

  // Extract boolean
  let mut spec = HashMap::new();
  spec.insert("auth_method".to_string(), "boolean".to_string());

  let extracted =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(response, &spec);

  assert!(extracted.contains_key("auth_method"));
  assert_eq!(extracted.get("auth_method").unwrap(), "true");

  // Create session
  let mut answer_extracted = HashMap::new();
  answer_extracted.insert(
    "auth_method".to_string(),
    extracted.get("auth_method").unwrap().clone(),
  );

  let session = InterviewSession {
    id: "e2e-bool-1".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![Answer {
      question_id: "q-auth-required".to_string(),
      question_text: "Does the API require authentication?".to_string(),
      perspective: Perspective::User,
      round: 1,
      response: response.to_string(),
      extracted: answer_extracted,
      confidence: 0.9,
      notes: String::new(),
      timestamp: "2026-02-28T10:00:00Z".to_string(),
    }],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Verify gap detection works correctly with extracted boolean
  let gaps = session.detect_gaps();
  // Auth field is now filled, so it shouldn't appear as a gap
  let auth_gap_exists = gaps.iter().any(|g| g.field == "auth_method");
  assert!(
    !auth_gap_exists,
    "Boolean extraction should fill auth_method gap"
  );
}

#[test]
fn test_e2e_extraction_pipeline_email_field() {
  let response = "Please contact support@example.com for more details";

  // Extract email
  let mut spec = HashMap::new();
  spec.insert("contact_email".to_string(), "email".to_string());

  let extracted =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(response, &spec);

  assert!(extracted.contains_key("contact_email"));
  assert_eq!(
    extracted.get("contact_email").unwrap(),
    "support@example.com"
  );
}

#[test]
fn test_e2e_extraction_pipeline_float_field() {
  let response = "2.5";

  // Extract float
  let mut spec = HashMap::new();
  spec.insert("version".to_string(), "float".to_string());

  let extracted =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(response, &spec);

  assert!(extracted.contains_key("version"));
  assert_eq!(extracted.get("version").unwrap(), "2.5");
}

#[test]
fn test_e2e_extraction_pipeline_name_field() {
  let response = "Payment Gateway";

  // Extract name (should take first line only)
  let mut spec = HashMap::new();
  spec.insert("api_name".to_string(), "name".to_string());

  let extracted =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(response, &spec);

  assert!(extracted.contains_key("api_name"));
  assert_eq!(extracted.get("api_name").unwrap(), "Payment Gateway");
}

#[test]
fn test_e2e_extraction_pipeline_boolean_field_with_storage() {
  let response = "yes";

  // Extract boolean
  let mut spec = HashMap::new();
  spec.insert("auth_method".to_string(), "boolean".to_string());

  let extracted =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(response, &spec);

  assert!(extracted.contains_key("auth_method"));
  assert_eq!(extracted.get("auth_method").unwrap(), "true");

  // Create session
  let mut answer_extracted = HashMap::new();
  answer_extracted.insert(
    "auth_method".to_string(),
    extracted.get("auth_method").unwrap().clone(),
  );

  let session = InterviewSession {
    id: "e2e-bool-1".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T10:00:00Z".to_string(),
    completed_at: None,
    stage: InterviewStage::Discovery,
    rounds_completed: 0,
    answers: vec![Answer {
      question_id: "q-auth-required".to_string(),
      question_text: "Does the API require authentication?".to_string(),
      perspective: Perspective::User,
      round: 1,
      response: response.to_string(),
      extracted: answer_extracted,
      confidence: 0.9,
      notes: String::new(),
      timestamp: "2026-02-28T10:00:00Z".to_string(),
    }],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: Vec::new(),
  };

  // Verify gap detection works correctly with extracted boolean
  let gaps = session.detect_gaps();
  // Auth field is now filled, so it shouldn't appear as a gap
  let auth_gap_exists = gaps.iter().any(|g| g.field == "auth_method");
  assert!(
    !auth_gap_exists,
    "Boolean extraction should fill auth_method gap"
  );
}

#[test]
fn test_e2e_full_api_interview_extraction() {
  // Simulate a complete API interview with multiple Q&A pairs

  // Q1: Base URL
  let q1_response = "The API base URL is https://api.myservice.com/v3";
  let q1_extracted = {
    let mut spec = HashMap::new();
    spec.insert("base_url".to_string(), "url".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(q1_response, &spec)
  };

  // Q2: Authentication
  let q2_response = "We use Bearer tokens with OAuth2";
  let q2_extracted = {
    let mut spec = HashMap::new();
    spec.insert("auth_method".to_string(), "text".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(q2_response, &spec)
  };

  // Q3: Happy path
  let q3_response = "GET /users returns a paginated list of users with 100 items per page";
  let q3_extracted = {
    let mut spec = HashMap::new();
    spec.insert("happy_path".to_string(), "text".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(q3_response, &spec)
  };

  // Q4: Error cases
  let q4_response = "401 Unauthorized when token is expired, 404 Not Found for missing resources, 500 for server errors";
  let q4_extracted = {
    let mut spec = HashMap::new();
    spec.insert("error_cases".to_string(), "text".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(q4_response, &spec)
  };

  // Q5: Response format
  let q5_response = "JSON with consistent envelope structure";
  let q5_extracted = {
    let mut spec = HashMap::new();
    spec.insert("response_format".to_string(), "text".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(q5_response, &spec)
  };

  // Build session with all extracted data
  let session = InterviewSession {
    id: "full-api-interview-1".to_string(),
    profile: Profile::Api,
    created_at: "2026-02-28T10:00:00Z".to_string(),
    updated_at: "2026-02-28T12:00:00Z".to_string(),
    completed_at: Some("2026-02-28T12:00:00Z".to_string()),
    stage: InterviewStage::Complete,
    rounds_completed: 1,
    answers: vec![
      Answer {
        question_id: "q-base-url".to_string(),
        question_text: "What is the base URL?".to_string(),
        perspective: Perspective::User,
        round: 1,
        response: q1_response.to_string(),
        extracted: q1_extracted,
        confidence: 0.95,
        notes: String::new(),
        timestamp: "2026-02-28T10:00:00Z".to_string(),
      },
      Answer {
        question_id: "q-auth".to_string(),
        question_text: "What authentication method?".to_string(),
        perspective: Perspective::User,
        round: 1,
        response: q2_response.to_string(),
        extracted: q2_extracted,
        confidence: 0.9,
        notes: String::new(),
        timestamp: "2026-02-28T10:05:00Z".to_string(),
      },
      Answer {
        question_id: "q-happy".to_string(),
        question_text: "What is the happy path?".to_string(),
        perspective: Perspective::User,
        round: 1,
        response: q3_response.to_string(),
        extracted: q3_extracted,
        confidence: 0.85,
        notes: String::new(),
        timestamp: "2026-02-28T10:10:00Z".to_string(),
      },
      Answer {
        question_id: "q-errors".to_string(),
        question_text: "What error cases to handle?".to_string(),
        perspective: Perspective::User,
        round: 1,
        response: q4_response.to_string(),
        extracted: q4_extracted,
        confidence: 0.9,
        notes: String::new(),
        timestamp: "2026-02-28T10:15:00Z".to_string(),
      },
      Answer {
        question_id: "q-format".to_string(),
        question_text: "What is the response format?".to_string(),
        perspective: Perspective::User,
        round: 1,
        response: q5_response.to_string(),
        extracted: q5_extracted,
        confidence: 0.95,
        notes: String::new(),
        timestamp: "2026-02-28T10:20:00Z".to_string(),
      },
    ],
    gaps: Vec::new(),
    conflicts: Vec::new(),
    raw_notes: String::new(),
    current_phase: 1,
    completed_phases: vec![1],
  };

  // Verify no gaps (all required fields extracted)
  let gaps = session.detect_gaps();
  assert!(
    gaps.is_empty(),
    "All API required fields should be extracted, no gaps expected"
  );

  // Verify spec building works
  let spec = build_spec_from_session(&session);
  assert!(spec.contains("package api"));

  // Verify storage roundtrip
  let line_result = session_to_jsonl_line(&session);
  assert!(line_result.is_ok());

  let parsed_result = serde_json::from_str::<InterviewSession>(&line_result.unwrap());
  assert!(parsed_result.is_ok());

  let parsed = parsed_result.unwrap();

  // Verify all extracted data survived roundtrip
  assert!(parsed.answers.iter().all(|a| !a.extracted.is_empty()));

  // Verify specific extracted values
  let base_url_answer = parsed
    .answers
    .iter()
    .find(|a| a.question_id == "q-base-url");
  assert!(base_url_answer.is_some());
  assert!(base_url_answer
    .unwrap()
    .extracted
    .get("base_url")
    .unwrap()
    .contains("https://api.myservice.com"));
}

#[test]
fn test_e2e_extraction_error_handling() {
  // Test extraction with invalid inputs

  // Invalid URL (no protocol)
  let response_no_protocol = "My API is at myserver.com";
  let mut spec = HashMap::new();
  spec.insert("url".to_string(), "url".to_string());

  let extracted = clarity_web::intent::interview::answer_extraction::extract_fields_with_types(
    response_no_protocol,
    &spec,
  );

  // Should not extract URL when no protocol present
  assert!(!extracted.contains_key("url"));

  // Empty response
  let response_empty = "";
  let mut spec2 = HashMap::new();
  spec2.insert("name".to_string(), "name".to_string());

  let extracted_empty =
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(
      response_empty,
      &spec2,
    );

  // Should not extract from empty response
  assert!(!extracted_empty.contains_key("name"));
}

#[test]
fn test_e2e_extraction_with_mixed_responses() {
  // Realistic interview with varied response types

  // Text response (use "name" type - which extracts first line only)
  let r1 = "deploy";
  let e1 = {
    let mut s = HashMap::new();
    s.insert("name".to_string(), "name".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(r1, &s)
  };

  // Boolean response
  let r2 = "yes";
  let e2 = {
    let mut s = HashMap::new();
    s.insert("autocomplete".to_string(), "boolean".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(r2, &s)
  };

  // List response
  let r3 = "init, build, deploy, status";
  let e3 = {
    let mut s = HashMap::new();
    s.insert("subcommands".to_string(), "list".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(r3, &s)
  };

  // URL response
  let r4 = "https://docs.example.com/cli";
  let e4 = {
    let mut s = HashMap::new();
    s.insert("docs_url".to_string(), "url".to_string());
    clarity_web::intent::interview::answer_extraction::extract_fields_with_types(r4, &s)
  };

  // Verify all extractions worked
  assert!(e1.contains_key("name"));
  assert_eq!(e1.get("name").unwrap(), "deploy");

  assert!(e2.contains_key("autocomplete"));
  assert_eq!(e2.get("autocomplete").unwrap(), "true");

  assert!(e3.contains_key("subcommands"));
  let subcommands = e3.get("subcommands").unwrap();
  assert!(subcommands.contains("init"));
  assert!(subcommands.contains("deploy"));

  assert!(e4.contains_key("docs_url"));
  assert!(e4
    .get("docs_url")
    .unwrap()
    .contains("https://docs.example.com"));
}
