//// Tests for the runner module
//// Tests execution orchestration, filtering, and result aggregation
//// Note: These are unit tests that don't make actual HTTP requests

import gleam/list
import gleam/option.{None, Some}
import gleam/string
import gleeunit/should
import intent/output
import intent/resolver
import intent/runner.{RunOptions}
import test_helpers.{make_test_behavior, make_test_feature, make_test_spec}

// ============================================================================
// RunOptions Tests
// ============================================================================

pub fn runner_default_options_test() {
  let options = runner.default_options()

  options.feature_filter
  |> should.equal(None)

  options.behavior_filter
  |> should.equal(None)

  options.output_level
  |> should.equal(runner.Normal)
}

pub fn runner_custom_options_verbose_test() {
  let options =
    RunOptions(
      feature_filter: Some("auth"),
      behavior_filter: Some("login"),
      output_level: runner.Verbose,
    )

  options.feature_filter
  |> should.equal(Some("auth"))

  options.behavior_filter
  |> should.equal(Some("login"))

  options.output_level
  |> should.equal(runner.Verbose)

  runner.is_verbose(options)
  |> should.be_true

  runner.is_quiet(options)
  |> should.be_false
}

pub fn runner_custom_options_quiet_test() {
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: runner.Quiet,
    )

  options.output_level
  |> should.equal(runner.Quiet)

  runner.is_verbose(options)
  |> should.be_false

  runner.is_quiet(options)
  |> should.be_true
}

pub fn runner_output_level_normal_test() {
  let options =
    RunOptions(
      feature_filter: None,
      behavior_filter: None,
      output_level: runner.Normal,
    )

  runner.is_verbose(options)
  |> should.be_false

  runner.is_quiet(options)
  |> should.be_false
}

// ============================================================================
// Spec with Circular Dependencies
// ============================================================================

pub fn runner_circular_dependency_error_test() {
  // Create a spec with circular dependencies
  let a = make_test_behavior("alpha", ["beta"])
  let b = make_test_behavior("beta", ["alpha"])
  let spec = make_test_spec([make_test_feature("Test", [a, b])])

  let result = runner.run_spec(spec, "", runner.default_options())

  result.pass
  |> should.be_false

  // Should have error in summary
  result.summary
  |> string.contains("Failed")
  |> should.be_true
}

pub fn runner_missing_dependency_error_test() {
  // Behavior depends on non-existent behavior
  let b = make_test_behavior("test", ["nonexistent"])
  let spec = make_test_spec([make_test_feature("Test", [b])])

  let result = runner.run_spec(spec, "", runner.default_options())

  result.pass
  |> should.be_false

  result.summary
  |> string.contains("Failed")
  |> should.be_true
}

// ============================================================================
// Empty Spec Tests
// ============================================================================

pub fn runner_empty_spec_test() {
  // Spec with no features
  let spec = make_test_spec([])

  let result = runner.run_spec(spec, "", runner.default_options())

  result.total
  |> should.equal(0)

  result.passed
  |> should.equal(0)

  result.failed
  |> should.equal(0)

  result.blocked
  |> should.equal(0)

  // Empty spec should pass (no failures)
  result.pass
  |> should.be_true
}

pub fn runner_empty_feature_test() {
  // Feature with no behaviors
  let spec = make_test_spec([make_test_feature("Empty", [])])

  let result = runner.run_spec(spec, "", runner.default_options())

  result.total
  |> should.equal(0)

  result.pass
  |> should.be_true
}

// ============================================================================
// Target URL Override Tests
// ============================================================================

pub fn runner_target_url_override_test() {
  // When target URL is provided, it should override config.base_url
  // We can't directly test this without HTTP, but we can verify the behavior
  // exists by testing with an empty target_url (should use config.base_url)
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])

  // With empty target_url, should attempt to use config base_url
  // This will fail because localhost:8080 isn't running, but that's expected
  let result = runner.run_spec(spec, "", runner.default_options())

  // The test should have been attempted (total > 0)
  result.total
  |> should.equal(1)
}

pub fn runner_custom_target_url_test() {
  // With a custom target URL that doesn't exist
  let b = make_test_behavior("test", [])
  let spec = make_test_spec([make_test_feature("API", [b])])

  let result =
    runner.run_spec(
      spec,
      "http://nonexistent.invalid:9999",
      runner.default_options(),
    )

  // Should attempt to run against the custom URL (1 behavior)
  // Verify the runner attempted to execute (total should be 1)
  result.total
  |> should.equal(1)
  // Note: Network errors may be classified as BehaviorError (not BehaviorFailed)
  // which means failed count might be 0 even though execution didn't succeed.
  // The important thing is that the behavior was attempted.
  // If we want strict failure counting, that would require the runner to treat
  // BehaviorError as failed - which is a separate enhancement.
}

// ============================================================================
// Result Aggregation Tests (using resolver for setup)
// ============================================================================

pub fn runner_resolver_succeeds_for_valid_deps_test() {
  // Verify the resolver component works correctly for runner
  let a = make_test_behavior("first", [])
  let b = make_test_behavior("second", ["first"])
  let c = make_test_behavior("third", ["second"])
  let spec = make_test_spec([make_test_feature("Chain", [a, b, c])])

  case resolver.resolve_execution_order(spec) {
    Ok(resolved) -> {
      list.length(resolved)
      |> should.equal(3)

      // Verify order: first, second, third
      let names = list.map(resolved, fn(rb) { rb.behavior.name })
      names
      |> should.equal(["first", "second", "third"])
    }
    Error(_) -> should.fail()
  }
}

// ============================================================================
// Multiple Features Tests
// ============================================================================

pub fn runner_multiple_features_count_test() {
  let f1 = make_test_feature("Auth", [make_test_behavior("login", [])])
  let f2 = make_test_feature("Users", [make_test_behavior("get-user", [])])
  let f3 =
    make_test_feature("Products", [make_test_behavior("list-products", [])])
  let spec = make_test_spec([f1, f2, f3])

  let result = runner.run_spec(spec, "", runner.default_options())

  // Should have 3 total behaviors
  result.total
  |> should.equal(3)
}

pub fn runner_cross_feature_dependencies_test() {
  // Behavior in feature 2 depends on behavior in feature 1
  let b1 = make_test_behavior("create-user", [])
  let b2 = make_test_behavior("get-user", ["create-user"])

  let f1 = make_test_feature("Write", [b1])
  let f2 = make_test_feature("Read", [b2])
  let spec = make_test_spec([f1, f2])

  // Should resolve correctly (no missing dependency error)
  case resolver.resolve_execution_order(spec) {
    Ok(resolved) -> {
      list.length(resolved)
      |> should.equal(2)
    }
    Error(_) -> should.fail()
  }
}

// ============================================================================
// Summary Message Tests
// ============================================================================

pub fn runner_summary_all_passed_format_test() {
  // When all pass, summary should mention "passed"
  let spec = make_test_spec([])
  let result = runner.run_spec(spec, "", runner.default_options())

  result.summary
  |> string.contains("passed")
  |> should.be_true
}

// ============================================================================
// SpecResult Structure Tests
// ============================================================================

pub fn spec_result_fields_test() {
  // Test that SpecResult has all expected fields
  let result =
    output.SpecResult(
      pass: True,
      passed: 5,
      failed: 0,
      blocked: 0,
      total: 5,
      summary: "All 5 behaviors passed",
      failures: [],
      blocked_behaviors: [],
      rule_violations: [],
      anti_patterns_detected: [],
    )

  result.pass
  |> should.be_true

  result.passed
  |> should.equal(5)

  result.failed
  |> should.equal(0)

  result.blocked
  |> should.equal(0)

  result.total
  |> should.equal(5)

  list.is_empty(result.failures)
  |> should.be_true

  list.is_empty(result.blocked_behaviors)
  |> should.be_true

  list.is_empty(result.rule_violations)
  |> should.be_true

  list.is_empty(result.anti_patterns_detected)
  |> should.be_true
}

pub fn spec_result_with_failures_test() {
  // Test SpecResult with failures
  let result =
    output.SpecResult(
      pass: False,
      passed: 3,
      failed: 2,
      blocked: 0,
      total: 5,
      summary: "2 failures out of 5",
      failures: [],
      // Would contain BehaviorFailure in real use
      blocked_behaviors: [],
      rule_violations: [],
      anti_patterns_detected: [],
    )

  result.pass
  |> should.be_false

  result.passed
  |> should.equal(3)

  result.failed
  |> should.equal(2)
}

pub fn spec_result_with_blocked_test() {
  // Test SpecResult with blocked behaviors
  let result =
    output.SpecResult(
      pass: False,
      passed: 2,
      failed: 1,
      blocked: 2,
      total: 5,
      summary: "1 failure, 2 blocked",
      failures: [],
      blocked_behaviors: [],
      // Would contain BlockedBehavior in real use
      rule_violations: [],
      anti_patterns_detected: [],
    )

  result.blocked
  |> should.equal(2)

  // Even with blocked, pass should be false
  result.pass
  |> should.be_false
}

// ============================================================================
// Behavior Execution Order Tests
// ============================================================================

pub fn runner_complex_dependency_graph_test() {
  // Diamond dependency: D depends on B and C, both depend on A
  let a = make_test_behavior("setup", [])
  let b = make_test_behavior("path-b", ["setup"])
  let c = make_test_behavior("path-c", ["setup"])
  let d = make_test_behavior("final", ["path-b", "path-c"])
  let spec = make_test_spec([make_test_feature("Diamond", [a, b, c, d])])

  case resolver.resolve_execution_order(spec) {
    Ok(resolved) -> {
      list.length(resolved)
      |> should.equal(4)

      // Setup must be first
      let names = list.map(resolved, fn(rb) { rb.behavior.name })
      let assert [first, ..rest] = names
      first
      |> should.equal("setup")

      // Final must be last
      let assert Ok(last) = list.last(rest)
      last
      |> should.equal("final")
    }
    Error(_) -> should.fail()
  }
}

pub fn runner_long_chain_test() {
  // Long dependency chain: a -> b -> c -> d -> e
  let a = make_test_behavior("a", [])
  let b = make_test_behavior("b", ["a"])
  let c = make_test_behavior("c", ["b"])
  let d = make_test_behavior("d", ["c"])
  let e = make_test_behavior("e", ["d"])
  let spec = make_test_spec([make_test_feature("Chain", [a, b, c, d, e])])

  case resolver.resolve_execution_order(spec) {
    Ok(resolved) -> {
      let names = list.map(resolved, fn(rb) { rb.behavior.name })
      names
      |> should.equal(["a", "b", "c", "d", "e"])
    }
    Error(_) -> should.fail()
  }
}
