//// Tests for BehaviorExecutor abstraction and error handling
//// TDD: Write tests first, then implement
////
//// Tests for:
//// - intent-cli-clm.1: BehaviorExecutor abstraction for runner testability
//// - intent-cli-clm.3: Fix BehaviorError not counted in failed results

import gleam/dict
import gleam/json
import gleeunit/should
import intent/http_client.{ExecutionResult, RequestError}
import intent/interpolate
import intent/runner.{type BehaviorExecutor, BehaviorExecutor}
import intent/types
import test_helpers.{
  make_test_behavior, make_test_config, make_test_feature, make_test_request,
  make_test_spec,
}

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
// BehaviorExecutor Type Tests (intent-cli-clm.1)
// ============================================================================

pub fn behavior_executor_type_exists_test() {
  // Test that BehaviorExecutor type is correctly defined
  let executor = mock_success_executor()
  let config = make_test_config()
  let request = make_test_request("/test")
  let ctx = interpolate.new_context()

  case executor.execute(config, request, ctx) {
    Ok(result) -> {
      result.status
      |> should.equal(200)
    }
    Error(_) -> should.fail()
  }
}

pub fn behavior_executor_mock_success_test() {
  // Test that mock executor returns expected success result
  let executor = mock_success_executor()
  let config = make_test_config()
  let request = make_test_request("/users")
  let ctx = interpolate.new_context()

  let result = executor.execute(config, request, ctx)

  case result {
    Ok(exec_result) -> {
      exec_result.status
      |> should.equal(200)

      exec_result.elapsed_ms
      |> should.equal(10)
    }
    Error(_) -> should.fail()
  }
}

pub fn behavior_executor_mock_error_test() {
  // Test that mock executor can return errors
  let executor = mock_error_executor()
  let config = make_test_config()
  let request = make_test_request("/test")
  let ctx = interpolate.new_context()

  let result = executor.execute(config, request, ctx)

  case result {
    Ok(_) -> should.fail()
    Error(RequestError(msg)) -> {
      msg
      |> should.equal("Connection refused")
    }
    Error(_) -> should.fail()
  }
}

// ============================================================================
// run_spec_with_executor Tests (intent-cli-clm.1)
// ============================================================================

pub fn run_spec_with_executor_success_test() {
  // Test running spec with mock executor that succeeds
  let b = make_test_behavior("test-endpoint", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options = runner.default_options()
  let executor = mock_success_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  result.passed
  |> should.equal(1)

  result.failed
  |> should.equal(0)

  result.pass
  |> should.be_true
}

pub fn run_spec_with_executor_multiple_behaviors_test() {
  // Test running multiple behaviors with mock executor
  let b1 = make_test_behavior("create-user", [])
  let b2 = make_test_behavior("get-user", ["create-user"])
  let b3 = make_test_behavior("delete-user", ["get-user"])
  let spec = make_test_spec([make_test_feature("Users", [b1, b2, b3])])
  let options = runner.default_options()
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

pub fn run_spec_with_executor_status_mismatch_test() {
  // Test that status mismatch causes failure
  let b = make_test_behavior("expects-200", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options = runner.default_options()
  // Return 404 when 200 is expected
  let executor = mock_status_executor(404)

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  result.passed
  |> should.equal(0)

  result.failed
  |> should.equal(1)

  result.pass
  |> should.be_false
}

// ============================================================================
// BehaviorError Counting Tests (intent-cli-clm.3)
// ============================================================================

pub fn behavior_error_counted_as_failure_test() {
  // BUG FIX: BehaviorError (network error) should be counted in failures
  let b = make_test_behavior("test-endpoint", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options = runner.default_options()
  let executor = mock_error_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(1)

  // CRITICAL: Network errors should count as failures
  // Before fix: failed == 0, pass == True (WRONG!)
  // After fix: failed == 1, pass == False (CORRECT!)
  result.failed
  |> should.equal(1)

  result.pass
  |> should.be_false
}

pub fn behavior_error_cascades_to_dependents_test() {
  // When a behavior errors, its dependents should be blocked
  let b1 = make_test_behavior("setup", [])
  let b2 = make_test_behavior("depends-on-setup", ["setup"])
  let spec = make_test_spec([make_test_feature("API", [b1, b2])])
  let options = runner.default_options()
  let executor = mock_error_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(2)

  // First behavior errors (counted as failed)
  // Second behavior should be blocked because first failed
  result.failed
  |> should.equal(1)

  result.blocked
  |> should.equal(1)

  result.pass
  |> should.be_false
}

pub fn behavior_error_summary_includes_errors_test() {
  // Summary should mention errors when BehaviorError occurs
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options = runner.default_options()
  let executor = mock_error_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  // Summary should indicate failure
  result.pass
  |> should.be_false
}

pub fn multiple_errors_all_counted_test() {
  // Multiple network errors should all be counted
  let b1 = make_test_behavior("endpoint-1", [])
  let b2 = make_test_behavior("endpoint-2", [])
  let b3 = make_test_behavior("endpoint-3", [])
  let spec = make_test_spec([make_test_feature("API", [b1, b2, b3])])
  let options = runner.default_options()
  let executor = mock_error_executor()

  let result = runner.run_spec_with_executor(spec, "", options, executor)

  result.total
  |> should.equal(3)

  // All 3 should be counted as failed (network errors)
  result.failed
  |> should.equal(3)

  result.pass
  |> should.be_false
}

// ============================================================================
// Mixed Success/Error Tests
// ============================================================================

pub fn run_spec_preserves_default_behavior_test() {
  // Verify that run_spec still works (uses default http_client executor)
  // This just verifies the function still exists and has same signature
  let spec = make_test_spec([])
  let options = runner.default_options()

  let result = runner.run_spec(spec, "", options)

  // Empty spec should pass
  result.pass
  |> should.be_true

  result.total
  |> should.equal(0)
}

// ============================================================================
// Executor with Target URL Override Tests
// ============================================================================

pub fn executor_receives_target_url_override_test() {
  // When target_url is provided, executor should receive updated config
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])
  let options = runner.default_options()

  // Create executor that checks base_url
  let executor =
    BehaviorExecutor(execute: fn(config, _request, _ctx) {
      // Verify the config has the overridden URL
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
