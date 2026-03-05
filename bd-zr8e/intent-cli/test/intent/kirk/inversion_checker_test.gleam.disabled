import gleam/dict
import gleam/json
import gleam/list
import gleam/string
import gleeunit/should
import intent/kirk/inversion_checker
import intent/types

// =============================================================================
// Test Data Helpers
// =============================================================================

fn empty_ai_hints() -> types.AIHints {
  types.AIHints(
    implementation: types.ImplementationHints(suggested_stack: []),
    entities: dict.new(),
    security: types.SecurityHints(
      password_hashing: "",
      jwt_algorithm: "",
      jwt_expiry: "",
      rate_limiting: "",
    ),
    pitfalls: [],
  )
}

fn empty_spec() -> types.Spec {
  types.Spec(
    name: "Test API",
    version: "1.0.0",
    description: "Test spec",
    audience: "",
    success_criteria: [],
    features: [],
    config: types.Config(base_url: "", timeout_ms: 5000, headers: dict.new()),
    rules: [],
    anti_patterns: [],
    ai_hints: empty_ai_hints(),
  )
}

fn make_behavior(name: String, intent: String, status: Int) -> types.Behavior {
  types.Behavior(
    name: name,
    intent: intent,
    notes: "",
    requires: [],
    tags: [],
    request: types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    ),
    response: types.Response(
      status: status,
      example: json.null(),
      checks: dict.new(),
      headers: dict.new(),
    ),
    captures: dict.new(),
  )
}

fn spec_with_behavior(name: String, intent: String, status: Int) -> types.Spec {
  let behavior = make_behavior(name, intent, status)

  let feature =
    types.Feature(name: "Test Feature", description: "Test", behaviors: [
      behavior,
    ])

  types.Spec(
    name: "Test API",
    version: "1.0.0",
    description: "Test spec",
    audience: "",
    success_criteria: [],
    features: [feature],
    config: types.Config(base_url: "", timeout_ms: 5000, headers: dict.new()),
    rules: [],
    anti_patterns: [],
    ai_hints: empty_ai_hints(),
  )
}

// =============================================================================
// Score Calculation Tests
// =============================================================================

pub fn analyze_empty_spec_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)

  // Empty spec should have 0% coverage
  should.equal(report.score, 0.0)

  // Should have gaps in all categories
  case report.security_gaps {
    [] -> should.fail()
    _ -> Nil
  }
}

pub fn analyze_spec_with_security_tests_test() {
  let spec =
    spec_with_behavior("auth-bypass-test", "Test auth bypass returns 401", 401)
  let report = inversion_checker.analyze_inversions(spec)

  // Should have better score than empty spec
  should.be_true(report.score >. 0.0)
}

pub fn analyze_spec_with_multiple_tests_test() {
  let behavior1 = make_behavior("valid-login", "Valid login returns 200", 200)
  let behavior2 =
    make_behavior("invalid-credentials", "Invalid credentials return 401", 401)

  let feature =
    types.Feature(
      name: "Auth Feature",
      description: "Authentication",
      behaviors: [behavior1, behavior2],
    )

  let spec =
    types.Spec(
      name: "Auth API",
      version: "1.0.0",
      description: "Auth spec",
      audience: "",
      success_criteria: [],
      features: [feature],
      config: types.Config(base_url: "", timeout_ms: 5000, headers: dict.new()),
      rules: [],
      anti_patterns: [],
      ai_hints: empty_ai_hints(),
    )

  let report = inversion_checker.analyze_inversions(spec)

  // Score should be better than single test
  should.be_true(report.score >. 0.0)

  // Total gaps should be reasonable
  let total_gaps =
    list.length(report.security_gaps)
    + list.length(report.usability_gaps)
    + list.length(report.integration_gaps)

  should.be_true(total_gaps < 30)
}

// =============================================================================
// Security Gap Tests
// =============================================================================

pub fn security_gap_detection_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)

  // Should detect security gaps
  case report.security_gaps {
    [] -> should.fail()
    [gap, ..] -> {
      should.equal(gap.category, "security")
      should.not_equal(gap.description, "")
      should.not_equal(gap.what_could_fail, "")
    }
  }
}

pub fn security_gap_covered_by_anti_pattern_test() {
  let anti_pattern =
    types.AntiPattern(
      name: "SQL Injection Prevention",
      description: "Test SQL injection protection",
      bad_example: json.null(),
      good_example: json.null(),
      why: "",
    )

  let spec =
    types.Spec(
      name: "Test API",
      version: "1.0.0",
      description: "Test",
      audience: "",
      success_criteria: [],
      features: [],
      config: types.Config(base_url: "", timeout_ms: 5000, headers: dict.new()),
      rules: [],
      anti_patterns: [anti_pattern],
      ai_hints: empty_ai_hints(),
    )

  let report = inversion_checker.analyze_inversions(spec)

  // Anti-patterns should reduce gaps
  should.be_true(report.score >. 0.0)
}

// =============================================================================
// Usability Gap Tests
// =============================================================================

pub fn usability_gap_detection_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)

  // Should detect usability gaps
  case report.usability_gaps {
    [] -> should.fail()
    [gap, ..] -> {
      should.equal(gap.category, "usability")
      should.not_equal(gap.description, "")
    }
  }
}

pub fn usability_not_found_covered_test() {
  let spec =
    spec_with_behavior("not-found-test", "Test not found returns 404", 404)
  let report = inversion_checker.analyze_inversions(spec)

  // Should not have not-found gap since we test 404
  let has_not_found_gap =
    list.any(report.usability_gaps, fn(gap) {
      string.contains(gap.description, "non-existent")
    })

  should.be_false(has_not_found_gap)
}

// =============================================================================
// Integration Gap Tests
// =============================================================================

pub fn integration_gap_detection_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)

  // Should detect integration gaps
  case report.integration_gaps {
    [] -> should.fail()
    [gap, ..] -> {
      should.equal(gap.category, "integration")
      should.not_equal(gap.description, "")
    }
  }
}

// =============================================================================
// Suggestion Tests
// =============================================================================

pub fn suggestions_generated_for_gaps_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)

  // Should suggest behaviors to fill gaps
  case report.suggested_behaviors {
    [] -> should.fail()
    [suggestion, ..] -> {
      should.not_equal(suggestion.name, "")
      should.not_equal(suggestion.intent, "")
      should.not_equal(suggestion.category, "")
    }
  }
}

pub fn suggestions_limited_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)

  // Should limit suggestions to reasonable number
  should.be_true(list.length(report.suggested_behaviors) < 20)
}

// =============================================================================
// Severity Tests
// =============================================================================

pub fn severity_to_string_test() {
  should.equal(
    inversion_checker.severity_to_string(inversion_checker.Low),
    "low",
  )
  should.equal(
    inversion_checker.severity_to_string(inversion_checker.Medium),
    "medium",
  )
  should.equal(
    inversion_checker.severity_to_string(inversion_checker.High),
    "high",
  )
  should.equal(
    inversion_checker.severity_to_string(inversion_checker.Critical),
    "critical",
  )
}

// =============================================================================
// Report Formatting Tests
// =============================================================================

pub fn format_report_returns_string_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)
  let formatted = inversion_checker.format_report(report)

  // Should return a non-empty string
  should.not_equal(formatted, "")

  // Should contain key sections
  should.be_true(string.contains(formatted, "Inversion Coverage"))
  should.be_true(string.contains(formatted, "Security Gaps"))
  should.be_true(string.contains(formatted, "Usability Gaps"))
  should.be_true(string.contains(formatted, "Integration Gaps"))
}

pub fn format_report_includes_score_test() {
  let spec = empty_spec()
  let report = inversion_checker.analyze_inversions(spec)
  let formatted = inversion_checker.format_report(report)

  // Should include score percentage
  should.be_true(string.contains(formatted, "0%"))
}

pub fn format_report_with_covered_categories_test() {
  let spec =
    spec_with_behavior(
      "invalid-credentials",
      "Invalid credentials return 401",
      401,
    )
  let report = inversion_checker.analyze_inversions(spec)
  let formatted = inversion_checker.format_report(report)

  // Should show coverage
  should.be_true(string.contains(formatted, "Inversion Coverage"))
}

// =============================================================================
// Edge Cases
// =============================================================================

pub fn spec_with_very_long_paths_test() {
  let behavior =
    types.Behavior(
      name: "test",
      intent: "Test",
      notes: "",
      requires: [],
      tags: [],
      request: types.Request(
        method: types.Get,
        path: "/api/v1/users/12345/posts/67890/comments",
        headers: dict.new(),
        query: dict.new(),
        body: json.null(),
      ),
      response: types.Response(
        status: 200,
        example: json.null(),
        checks: dict.new(),
        headers: dict.new(),
      ),
      captures: dict.new(),
    )

  let feature =
    types.Feature(name: "Test Feature", description: "Test", behaviors: [
      behavior,
    ])

  let spec =
    types.Spec(
      name: "Test API",
      version: "1.0.0",
      description: "Test",
      audience: "",
      success_criteria: [],
      features: [feature],
      config: types.Config(base_url: "", timeout_ms: 5000, headers: dict.new()),
      rules: [],
      anti_patterns: [],
      ai_hints: empty_ai_hints(),
    )

  let report = inversion_checker.analyze_inversions(spec)

  // Should handle long paths gracefully
  should.be_true(report.score >=. 0.0)
}

pub fn spec_with_multiple_features_test() {
  let feature1 =
    types.Feature(name: "Feature 1", description: "First feature", behaviors: [
      types.Behavior(
        name: "test1",
        intent: "Test 1",
        notes: "",
        requires: [],
        tags: [],
        request: types.Request(
          method: types.Get,
          path: "/path1",
          headers: dict.new(),
          query: dict.new(),
          body: json.null(),
        ),
        response: types.Response(
          status: 200,
          example: json.null(),
          checks: dict.new(),
          headers: dict.new(),
        ),
        captures: dict.new(),
      ),
    ])

  let feature2 =
    types.Feature(name: "Feature 2", description: "Second feature", behaviors: [
      types.Behavior(
        name: "test2",
        intent: "Test 2",
        notes: "",
        requires: [],
        tags: [],
        request: types.Request(
          method: types.Post,
          path: "/path2",
          headers: dict.new(),
          query: dict.new(),
          body: json.null(),
        ),
        response: types.Response(
          status: 201,
          example: json.null(),
          checks: dict.new(),
          headers: dict.new(),
        ),
        captures: dict.new(),
      ),
    ])

  let spec =
    types.Spec(
      name: "Test API",
      version: "1.0.0",
      description: "Test",
      audience: "",
      success_criteria: [],
      features: [feature1, feature2],
      config: types.Config(base_url: "", timeout_ms: 5000, headers: dict.new()),
      rules: [],
      anti_patterns: [],
      ai_hints: empty_ai_hints(),
    )

  let report = inversion_checker.analyze_inversions(spec)

  // Should handle multiple features
  should.be_true(report.score >=. 0.0)
}
