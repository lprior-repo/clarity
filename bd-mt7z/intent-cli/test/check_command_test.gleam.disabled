//// Tests for intent check command with mock server
//// Tests end-to-end workflow: load spec -> run check -> validate results
////
//// Exit codes:
//// - 0: all checks passed
//// - 1: check failed (at least one behavior failed)
//// - 2: blocked behaviors detected
//// - 3: invalid spec (loading/parse error)
//// - 4: general error

import gleam/dict
import gleam/json
import gleam/option.{None, Some}
import gleam/string
import gleeunit/should
import intent/http_client.{ExecutionResult, RequestError}
import intent/runner.{
  type BehaviorExecutor, BehaviorExecutor, Normal, Quiet, RunOptions, Verbose,
}
import intent/types
import test_helpers.{make_test_behavior, make_test_feature, make_test_spec}

// ============================================================================
// Mock Executors
// ============================================================================

/// Create a mock executor that returns success with 200 status
fn mock_success_executor() -> BehaviorExecutor {
  BehaviorExecutor(execute: fn(_config, _request, _ctx) {
    Ok(ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 10,
      request_method: types.Get,
      request_path: "/test",
    ))
  })
}

/// Create a mock executor that returns a network error
fn mock_error_executor() -> BehaviorExecutor {
  BehaviorExecutor(execute: fn(_config, _request, _ctx) {
    Error(RequestError("Connection refused"))
  })
}

/// Create a mock executor that returns a specific status code
fn mock_status_executor(status: Int) -> BehaviorExecutor {
  BehaviorExecutor(execute: fn(_config, request, _ctx) {
    Ok(ExecutionResult(
      status: status,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 10,
      request_method: request.method,
      request_path: request.path,
    ))
  })
}

// ============================================================================
// Exit Code Tests
// ============================================================================

pub fn check_exit_code_all_passed_test() {
  // Test that check returns exit code 0 when all behaviors pass
  let b = make_test_behavior("test-endpoint", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  result.passed
  |> should.equal(1)

  result.failed
  |> should.equal(0)

  result.blocked
  |> should.equal(0)

  result.pass
  |> should.be_true
}

pub fn check_exit_code_failure_test() {
  // Test that check returns exit code 1 when behaviors fail
  let b = make_test_behavior("failing-endpoint", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  // Return 500 when 200 is expected
  let executor = mock_status_executor(500)

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  result.passed
  |> should.equal(0)

  result.failed
  |> should.equal(1)

  result.blocked
  |> should.equal(0)

  result.pass
  |> should.be_false
}

pub fn check_exit_code_blocked_test() {
  // Test that check returns exit code 2 when behaviors are blocked
  let b1 = make_test_behavior("setup", [])
  let b2 = make_test_behavior("depends-on-setup", ["setup"])
  let spec = make_test_spec([make_test_feature("API", [b1, b2])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  // First behavior errors, second should be blocked
  let executor = mock_error_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(2)

  result.failed
  |> should.equal(1)

  result.blocked
  |> should.equal(1)

  result.pass
  |> should.be_false
}

pub fn check_exit_code_invalid_spec_test() {
  // Test that check returns exit code 3 for invalid spec
  // Spec with circular dependency should fail to resolve
  let a = make_test_behavior("alpha", ["beta"])
  let b = make_test_behavior("beta", ["alpha"])
  let spec = make_test_spec([make_test_feature("Test", [a, b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.pass
  |> should.be_false

  result.summary
  |> string.contains("Failed")
  |> should.be_true
}

// ============================================================================
// Feature Filter Tests
// ============================================================================

pub fn check_feature_filter_test() {
  // Test that --feature flag filters behaviors
  let b1 = make_test_behavior("auth-login", [])
  let b2 = make_test_behavior("users-list", [])
  let b3 = make_test_behavior("users-create", [])

  let f1 = make_test_feature("Authentication", [b1])
  let f2 = make_test_feature("Users", [b2, b3])
  let spec = make_test_spec([f1, f2])

  let options =
    RunOptions(
      feature_filter: Some("Users"),
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  // Should only run behaviors from Users feature
  result.total
  |> should.equal(2)

  result.passed
  |> should.equal(2)
}

pub fn check_feature_filter_nonexistent_test() {
  // Test filtering to non-existent feature returns 0 behaviors
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])

  let options =
    RunOptions(
      feature_filter: Some("NonExistent"),
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(0)

  result.pass
  |> should.be_true
}

// ============================================================================
// Behavior Filter Tests
// ============================================================================

pub fn check_behavior_filter_test() {
  // Test that --only flag runs specific behavior
  let b1 = make_test_behavior("behavior-1", [])
  let b2 = make_test_behavior("behavior-2", [])
  let b3 = make_test_behavior("behavior-3", [])

  let spec = make_test_spec([make_test_feature("Test", [b1, b2, b3])])

  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: Some("behavior-2"),
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  // Should only run behavior-2
  result.total
  |> should.equal(1)

  result.passed
  |> should.equal(1)
}

pub fn check_behavior_filter_with_dependencies_test() {
  // Test that --only runs specific behavior (dependencies are resolved at spec level)
  let b1 = make_test_behavior("setup", [])
  let b2 = make_test_behavior("main", ["setup"])
  let b3 = make_test_behavior("cleanup", [])

  let spec = make_test_spec([make_test_feature("Test", [b1, b2, b3])])

  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: Some("main"),
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  // Should run only the filtered behavior (dependencies are resolved but filter applies)
  result.total
  |> should.equal(1)

  result.passed
  |> should.equal(1)
}

// ============================================================================
// Target URL Override Tests
// ============================================================================

pub fn check_target_url_override_test() {
  // Test that --target overrides config.base_url
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )

  // Create executor that verifies base_url override
  let executor =
    BehaviorExecutor(execute: fn(config, _request, _ctx) {
      case config.base_url {
        "http://override.test:9999" -> {
          Ok(ExecutionResult(
            status: 200,
            headers: dict.new(),
            body: json.object([]),
            raw_body: "{}",
            elapsed_ms: 5,
            request_method: types.Get,
            request_path: "/test",
          ))
        }
        _ -> Error(RequestError("Wrong base_url: " <> config.base_url))
      }
    })

  let result =
    runner.run_spec_with_executor(
      spec,
      "http://override.test:9999",
      options,
      executor,
    )

  result.passed
  |> should.equal(1)

  result.pass
  |> should.be_true
}

pub fn check_target_url_empty_uses_config_test() {
  // Test that empty target_url uses config.base_url
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )

  // Create executor that verifies default config base_url
  let executor =
    BehaviorExecutor(execute: fn(config, _request, _ctx) {
      case config.base_url {
        "http://localhost:8080" -> {
          Ok(ExecutionResult(
            status: 200,
            headers: dict.new(),
            body: json.object([]),
            raw_body: "{}",
            elapsed_ms: 5,
            request_method: types.Get,
            request_path: "/test",
          ))
        }
        _ -> Error(RequestError("Expected default base_url"))
      }
    })

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.passed
  |> should.equal(1)

  result.pass
  |> should.be_true
}

// ============================================================================
// Output Level Tests
// ============================================================================

pub fn check_output_level_normal_test() {
  // Test Normal output level
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])

  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )

  runner.is_verbose(options)
  |> should.be_false

  runner.is_quiet(options)
  |> should.be_false

  let executor = mock_success_executor()
  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.pass
  |> should.be_true
}

pub fn check_output_level_verbose_test() {
  // Test Verbose output level
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])

  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Verbose,
    )

  runner.is_verbose(options)
  |> should.be_true

  runner.is_quiet(options)
  |> should.be_false

  let executor = mock_success_executor()
  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.pass
  |> should.be_true
}

pub fn check_output_level_quiet_test() {
  // Test Quiet output level
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])

  let options =
    RunOptions(feature_filter: None, behavior_filter: None, output_level: Quiet)

  runner.is_verbose(options)
  |> should.be_false

  runner.is_quiet(options)
  |> should.be_true

  let executor = mock_success_executor()
  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.pass
  |> should.be_true
}

// ============================================================================
// Complex Workflow Tests
// ============================================================================

pub fn check_workflow_registration_flow_test() {
  // Test a realistic user registration flow with dependencies
  let b1 = make_test_behavior("register-user", [])
  let b2 = make_test_behavior("login-user", ["register-user"])
  let b3 = make_test_behavior("get-profile", ["login-user"])

  let spec = make_test_spec([make_test_feature("Auth", [b1, b2, b3])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(3)

  result.passed
  |> should.equal(3)

  result.failed
  |> should.equal(0)

  result.blocked
  |> should.equal(0)

  result.pass
  |> should.be_true
}

pub fn check_workflow_with_failure_cascade_test() {
  // Test that failure cascades to dependent behaviors
  let b1 = make_test_behavior("setup", [])
  let b2 = make_test_behavior("process", ["setup"])
  let b3 = make_test_behavior("cleanup", ["process"])

  let spec = make_test_spec([make_test_feature("Pipeline", [b1, b2, b3])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )

  // First behavior succeeds, second fails, third blocked
  let executor =
    BehaviorExecutor(execute: fn(_config, request, _ctx) {
      case request.path {
        "/setup" -> {
          Ok(ExecutionResult(
            status: 200,
            headers: dict.new(),
            body: json.object([]),
            raw_body: "{}",
            elapsed_ms: 10,
            request_method: types.Get,
            request_path: "/setup",
          ))
        }
        _ -> Error(RequestError("Service unavailable"))
      }
    })

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(3)

  result.passed
  |> should.equal(1)

  result.failed
  |> should.equal(1)

  result.blocked
  |> should.equal(1)

  result.pass
  |> should.be_false
}

pub fn check_workflow_multiple_features_test() {
  // Test running multiple features
  let auth_b = make_test_behavior("login", [])
  let user_b = make_test_behavior("list-users", [])
  let product_b = make_test_behavior("list-products", [])

  let f1 = make_test_feature("Auth", [auth_b])
  let f2 = make_test_feature("Users", [user_b])
  let f3 = make_test_feature("Products", [product_b])

  let spec = make_test_spec([f1, f2, f3])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(3)

  result.passed
  |> should.equal(3)

  result.pass
  |> should.be_true
}

// ============================================================================
// Error Handling Tests
// ============================================================================

pub fn check_network_error_test() {
  // Test that network errors are handled gracefully
  let b = make_test_behavior("endpoint", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_error_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  // Network errors should count as failures
  result.failed
  |> should.equal(1)

  result.pass
  |> should.be_false
}

pub fn check_timeout_error_test() {
  // Test that timeout errors are handled
  let b = make_test_behavior("endpoint", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )

  let executor =
    BehaviorExecutor(execute: fn(_config, _request, _ctx) {
      Error(RequestError("Request timeout after 5000ms"))
    })

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  result.failed
  |> should.equal(1)

  result.pass
  |> should.be_false
}

pub fn check_dns_error_test() {
  // Test that DNS errors are handled
  let b = make_test_behavior("endpoint", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )

  let executor =
    BehaviorExecutor(execute: fn(_config, _request, _ctx) {
      Error(RequestError("DNS resolution failed"))
    })

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  result.failed
  |> should.equal(1)

  result.pass
  |> should.be_false
}

// ============================================================================
// Empty Spec Tests
// ============================================================================

pub fn check_empty_spec_passes_test() {
  // Test that empty spec passes (no behaviors to fail)
  let spec = make_test_spec([])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(0)

  result.passed
  |> should.equal(0)

  result.failed
  |> should.equal(0)

  result.blocked
  |> should.equal(0)

  result.pass
  |> should.be_true
}

pub fn check_empty_feature_passes_test() {
  // Test that feature with no behaviors passes
  let spec = make_test_spec([make_test_feature("Empty", [])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(0)

  result.pass
  |> should.be_true
}

// ============================================================================
// Summary Tests
// ============================================================================

pub fn check_summary_all_passed_format_test() {
  // Test summary message when all pass
  let spec = make_test_spec([])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()
  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.summary
  |> string.contains("passed")
  |> should.be_true
}

pub fn check_summary_failure_format_test() {
  // Test summary message when behaviors fail
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_error_executor()
  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.summary
  |> string.contains("failures")
  |> should.be_true
}

pub fn check_summary_blocked_format_test() {
  // Test summary message when behaviors are blocked
  let b1 = make_test_behavior("setup", [])
  let b2 = make_test_behavior("depends", ["setup"])
  let spec = make_test_spec([make_test_feature("API", [b1, b2])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_error_executor()
  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.summary
  |> string.contains("blocked")
  |> should.be_true
}

pub fn check_summary_all_three_test() {
  // Test summary with passed, failed, and blocked
  let b1 = make_test_behavior("setup", [])
  let b2 = make_test_behavior("success", ["setup"])
  let b3 = make_test_behavior("failure", ["setup"])
  let b4 = make_test_behavior("blocked", ["failure"])

  let spec = make_test_spec([make_test_feature("Test", [b1, b2, b3, b4])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )

  // Setup succeeds, success succeeds, failure fails, blocked blocked
  let executor =
    BehaviorExecutor(execute: fn(_config, request, _ctx) {
      case request.path {
        "/setup" | "/success" -> {
          Ok(ExecutionResult(
            status: 200,
            headers: dict.new(),
            body: json.object([]),
            raw_body: "{}",
            elapsed_ms: 10,
            request_method: types.Get,
            request_path: request.path,
          ))
        }
        _ -> Error(RequestError("Service error"))
      }
    })

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  let summary = result.summary
  summary
  |> string.contains("1 failures")
  |> should.be_true

  summary
  |> string.contains("1 blocked")
  |> should.be_true
}

// ============================================================================
// Dependency Resolution Tests
// ============================================================================

pub fn check_cross_feature_dependencies_test() {
  // Test dependencies across features
  let b1 = make_test_behavior("create-user", [])
  let b2 = make_test_behavior("get-user", ["create-user"])

  let f1 = make_test_feature("Users", [b1])
  let f2 = make_test_feature("Profile", [b2])

  let spec = make_test_spec([f1, f2])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(2)

  result.passed
  |> should.equal(2)

  result.pass
  |> should.be_true
}

pub fn check_complex_dependency_graph_test() {
  // Test diamond dependency: D depends on B and C, both depend on A
  let a = make_test_behavior("A", [])
  let b = make_test_behavior("B", ["A"])
  let c = make_test_behavior("C", ["A"])
  let d = make_test_behavior("D", ["B", "C"])

  let spec = make_test_spec([make_test_feature("Diamond", [a, b, c, d])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(4)

  result.passed
  |> should.equal(4)

  result.pass
  |> should.be_true
}

pub fn check_long_dependency_chain_test() {
  // Test long chain of dependencies
  let b1 = make_test_behavior("step-1", [])
  let b2 = make_test_behavior("step-2", ["step-1"])
  let b3 = make_test_behavior("step-3", ["step-2"])
  let b4 = make_test_behavior("step-4", ["step-3"])
  let b5 = make_test_behavior("step-5", ["step-4"])

  let spec = make_test_spec([make_test_feature("Chain", [b1, b2, b3, b4, b5])])
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(5)

  result.passed
  |> should.equal(5)

  result.pass
  |> should.be_true
}

// ============================================================================
// Combination Filter Tests
// ============================================================================

pub fn check_feature_and_behavior_filter_test() {
  // Test combining --feature and --only flags
  let f1_b1 = make_test_behavior("auth-login", [])
  let f1_b2 = make_test_behavior("auth-logout", [])
  let f2_b1 = make_test_behavior("users-list", [])
  let f2_b2 = make_test_behavior("users-create", [])

  let f1 = make_test_feature("Auth", [f1_b1, f1_b2])
  let f2 = make_test_feature("Users", [f2_b1, f2_b2])

  let spec = make_test_spec([f1, f2])
  let options =
    RunOptions(
      feature_filter: Some("Users"),
      behavior_filter: Some("users-list"),
      output_level: Normal,
    )
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  // Should only run users-list (from Users feature)
  result.total
  |> should.equal(1)

  result.passed
  |> should.equal(1)
}
