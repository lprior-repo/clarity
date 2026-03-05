import gleam/dict
import gleam/json
import gleam/list
import gleam/option.{None, Some}
import gleam/string
import gleeunit
import gleeunit/should
import intent/bead_templates
// import intent/checker  // Removed in v3.0
// import intent/http_client  // Removed in v3.0
import intent/effects_analyzer
import intent/interpolate
import intent/interview
import intent/interview_contract
import intent/interview_questions
// import intent/kirk/effects_analyzer  // Disabled in v3.0, now at intent/effects_analyzer
import intent/question_loader
import intent/question_types.{
  type Question, Critical, Developer, HappyPath, Ops, Question, Security, User,
}
import intent/resolver
// import intent/rules_engine  // Removed in v3.0
import intent/types

pub fn main() -> Nil {
  gleeunit.main()
}

// ============================================================================
// Resolver Tests
// ============================================================================

fn make_behavior(name: String, requires: List(String)) -> types.Behavior {
  types.Behavior(
    name: name,
    intent: "Test intent for " <> name,
    notes: "",
    requires: requires,
    tags: [],
    preconditions: [],
    postconditions: [],
    verifications: [],
  )
}

fn make_feature(name: String, behaviors: List(types.Behavior)) -> types.Feature {
  types.Feature(name: name, description: "Test feature", behaviors: behaviors)
}

fn make_spec(features: List(types.Feature)) -> types.Spec {
  types.Spec(
    name: "Test Spec",
    description: "Test spec",
    audience: "",
    version: "1.0.0",
    success_criteria: [],
    features: features,
    invariants: [],
    anti_patterns: [],
    ai_hints: types.AIHints(
      implementation: types.ImplementationHints(suggested_stack: []),
      entities: dict.new(),
      security: types.SecurityHints(
        password_hashing: "",
        jwt_algorithm: "",
        jwt_expiry: "",
        rate_limiting: "",
      ),
      pitfalls: [],
    ),
  )
}

pub fn effects_analyzer_format_report_completes_test() {
  let behavior = make_behavior("read-user", [])
  let feature = make_feature("users", [behavior])
  let spec = make_spec([feature])

  let report = effects_analyzer.analyze_spec(spec)

  // Check that analysis completed successfully
  report.behavior_effects
  |> list.length
  |> should.equal(1)
}

pub fn resolver_simple_no_deps_test() {
  // Three behaviors with no dependencies - all should be executable
  let b1 = make_behavior("first", [])
  let b2 = make_behavior("second", [])
  let b3 = make_behavior("third", [])

  let spec = make_spec([make_feature("Feature A", [b1, b2, b3])])

  let result = resolver.resolve_execution_order(spec)

  case result {
    Ok(resolved) -> {
      list.length(resolved)
      |> should.equal(3)
    }
    Error(_) -> should.fail()
  }
}

pub fn resolver_linear_dependency_chain_test() {
  // b3 -> b2 -> b1 (b3 depends on b2, b2 depends on b1)
  let b1 = make_behavior("first", [])
  let b2 = make_behavior("second", ["first"])
  let b3 = make_behavior("third", ["second"])

  let spec = make_spec([make_feature("Feature A", [b1, b2, b3])])

  let result = resolver.resolve_execution_order(spec)

  case result {
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

pub fn resolver_multiple_deps_on_one_test() {
  // b2 and b3 both depend on b1
  let b1 = make_behavior("base", [])
  let b2 = make_behavior("child-a", ["base"])
  let b3 = make_behavior("child-b", ["base"])

  let spec = make_spec([make_feature("Feature A", [b1, b2, b3])])

  let result = resolver.resolve_execution_order(spec)

  case result {
    Ok(resolved) -> {
      list.length(resolved)
      |> should.equal(3)

      // First should be base
      let names = list.map(resolved, fn(rb) { rb.behavior.name })
      let assert [first, ..] = names
      first
      |> should.equal("base")
    }
    Error(_) -> should.fail()
  }
}

pub fn resolver_missing_dependency_test() {
  // b1 depends on "nonexistent" which doesn't exist
  let b1 = make_behavior("first", ["nonexistent"])

  let spec = make_spec([make_feature("Feature A", [b1])])

  let result = resolver.resolve_execution_order(spec)

  case result {
    Ok(_) -> should.fail()
    Error(resolver.MissingDependency(behavior, missing)) -> {
      behavior
      |> should.equal("first")
      missing
      |> should.equal("nonexistent")
    }
    Error(_) -> should.fail()
  }
}

pub fn resolver_cyclic_dependency_test() {
  // b1 -> b2 -> b1 (cycle)
  let b1 = make_behavior("first", ["second"])
  let b2 = make_behavior("second", ["first"])

  let spec = make_spec([make_feature("Feature A", [b1, b2])])

  let result = resolver.resolve_execution_order(spec)

  case result {
    Ok(_) -> should.fail()
    Error(resolver.CyclicDependency(_)) -> should.be_ok(Ok(Nil))
    Error(_) -> should.fail()
  }
}

pub fn resolver_duplicate_name_test() {
  // Two behaviors with the same name
  let b1 = make_behavior("same-name", [])
  let b2 = make_behavior("same-name", [])

  let spec = make_spec([make_feature("Feature A", [b1, b2])])

  let result = resolver.resolve_execution_order(spec)

  case result {
    Ok(_) -> should.fail()
    Error(resolver.DuplicateBehaviorName(name)) -> {
      name
      |> should.equal("same-name")
    }
    Error(_) -> should.fail()
  }
}

pub fn resolver_cross_feature_deps_test() {
  // b2 in Feature B depends on b1 in Feature A
  let b1 = make_behavior("base", [])
  let b2 = make_behavior("dependent", ["base"])

  let spec =
    make_spec([make_feature("Feature A", [b1]), make_feature("Feature B", [b2])])

  let result = resolver.resolve_execution_order(spec)

  case result {
    Ok(resolved) -> {
      list.length(resolved)
      |> should.equal(2)

      let names = list.map(resolved, fn(rb) { rb.behavior.name })
      names
      |> should.equal(["base", "dependent"])
    }
    Error(_) -> should.fail()
  }
}

// ============================================================================
// Interpolation Tests
// ============================================================================

pub fn interpolate_simple_variable_test() {
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable("user_id", json_string("12345"))

  let result = interpolate.interpolate_string(ctx, "/users/${user_id}")

  result
  |> should.be_ok

  case result {
    Ok(s) ->
      s
      |> should.equal("/users/12345")
    Error(_) -> should.fail()
  }
}

pub fn interpolate_multiple_variables_test() {
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable("org", json_string("acme"))
    |> interpolate.set_variable("team", json_string("dev"))

  let result = interpolate.interpolate_string(ctx, "/orgs/${org}/teams/${team}")

  case result {
    Ok(s) ->
      s
      |> should.equal("/orgs/acme/teams/dev")
    Error(_) -> should.fail()
  }
}

pub fn interpolate_missing_variable_test() {
  let ctx = interpolate.new_context()

  let result = interpolate.interpolate_string(ctx, "/users/${unknown}")

  result
  |> should.be_error
}

pub fn interpolate_no_variables_test() {
  let ctx = interpolate.new_context()

  let result = interpolate.interpolate_string(ctx, "/users/static")

  case result {
    Ok(s) ->
      s
      |> should.equal("/users/static")
    Error(_) -> should.fail()
  }
}

// Helper to create a JSON string value
fn json_string(s: String) -> json.Json {
  json.string(s)
}

// ============================================================================
// Interview Engine Tests
// ============================================================================

pub fn interview_get_questions_api_round_1_test() {
  let questions = interview_questions.get_questions_for_round("api", 1)
  let has_questions = questions != []
  has_questions |> should.be_true()
}

pub fn interview_get_questions_cli_round_1_test() {
  let questions = interview_questions.get_questions_for_round("cli", 1)
  let has_questions = questions != []
  has_questions |> should.be_true()
}

pub fn interview_create_session_test() {
  let extracted =
    interview.extract_from_answer("q1", "We use JWT tokens for authentication", [
      "auth_method",
    ])
  let auth_method = dict.get(extracted, "auth_method")
  auth_method |> should.equal(Ok("jwt"))
}

pub fn interview_extract_auth_method_oauth_test() {
  let extracted =
    interview.extract_from_answer("q1", "OAuth 2.0 is our auth standard", [
      "auth_method",
    ])
  let auth_method = dict.get(extracted, "auth_method")
  auth_method |> should.equal(Ok("oauth"))
}

pub fn interview_extract_entities_test() {
  let extracted =
    interview.extract_from_answer("q1", "Users, Orders, Products, Payments", [
      "entities",
    ])
  let entities = dict.get(extracted, "entities")
  entities
  |> should.equal(Ok("Users, Orders, Products, Payments"))
}

pub fn interview_extract_audience_mobile_test() {
  let extracted =
    interview.extract_from_answer("q1", "Mainly mobile app users", ["audience"])
  let audience = dict.get(extracted, "audience")
  audience |> should.equal(Ok("mobile"))
}

pub fn interview_detect_gaps_empty_answers_test() {
  let answers = []
  let gaps = interview.detect_gaps(interview.Api, answers)
  let has_gaps = gaps != []
  has_gaps |> should.be_true()
}

pub fn interview_detect_gaps_with_answers_test() {
  let answers = [
    interview.Answer(
      question_id: "q1",
      question_text: "What auth?",
      perspective: Security,
      round: 1,
      response: "JWT",
      extracted: dict.from_list([#("auth_method", "jwt")]),
      confidence: 0.9,
      notes: "",
      timestamp: "2024-01-01T00:00:00Z",
    ),
    interview.Answer(
      question_id: "q2",
      question_text: "What entities?",
      perspective: Developer,
      round: 1,
      response: "Users, Tokens",
      extracted: dict.from_list([
        #("entities", "Users, Tokens"),
        #("base_url", "http://localhost:8080"),
      ]),
      confidence: 0.85,
      notes: "",
      timestamp: "2024-01-01T00:00:00Z",
    ),
    interview.Answer(
      question_id: "q3",
      question_text: "Happy path?",
      perspective: User,
      round: 1,
      response: "Login and get token",
      extracted: dict.from_list([#("happy_path", "Login and get token")]),
      confidence: 0.8,
      notes: "",
      timestamp: "2024-01-01T00:00:00Z",
    ),
    interview.Answer(
      question_id: "q4",
      question_text: "Errors?",
      perspective: User,
      round: 2,
      response: "Wrong password, user not found",
      extracted: dict.from_list([#("error_cases", "Wrong password")]),
      confidence: 0.75,
      notes: "",
      timestamp: "2024-01-01T00:00:00Z",
    ),
    interview.Answer(
      question_id: "q5",
      question_text: "Format?",
      perspective: Developer,
      round: 1,
      response: "JSON response format",
      extracted: dict.from_list([#("response_format", "json")]),
      confidence: 0.9,
      notes: "",
      timestamp: "2024-01-01T00:00:00Z",
    ),
  ]

  let gaps = interview.detect_gaps(interview.Api, answers)
  gaps |> list.length() |> should.equal(0)
}

pub fn interview_detect_conflicts_cap_theorem_test() {
  let answers = [
    interview.Answer(
      question_id: "q1",
      question_text: "Performance?",
      perspective: Ops,
      round: 3,
      response: "We need fast latency, under 50ms",
      extracted: dict.from_list([]),
      confidence: 0.8,
      notes: "",
      timestamp: "2024-01-01T00:00:00Z",
    ),
    interview.Answer(
      question_id: "q2",
      question_text: "Consistency?",
      perspective: Developer,
      round: 3,
      response: "All data must be strongly consistent",
      extracted: dict.from_list([]),
      confidence: 0.85,
      notes: "",
      timestamp: "2024-01-01T00:00:00Z",
    ),
  ]

  let conflicts = interview.detect_conflicts(answers)
  conflicts
  |> list.any(fn(c) { c.id == "conflict-cap" })
  |> should.be_true()
}

pub fn interview_calculate_confidence_high_test() {
  let extracted =
    dict.from_list([#("auth_method", "jwt"), #("audience", "mobile")])
  let confidence =
    interview.calculate_confidence(
      "q1",
      "This is a very detailed response about JWT authentication and mobile users with specific requirements",
      extracted,
    )
  let is_high = confidence >. 0.8
  is_high |> should.be_true()
}

pub fn interview_add_answer_test() {
  let question =
    Question(
      id: "q1",
      round: 1,
      perspective: User,
      category: HappyPath,
      priority: Critical,
      question: "What should this do?",
      context: "Start simple",
      example: "Example here",
      expected_type: "text",
      extract_into: [],
      depends_on: [],
      blocks: [],
    )

  let formatted = interview.format_question(question)
  formatted |> string.contains("[CRITICAL]") |> should.be_true()
  formatted |> string.contains("What should this do?") |> should.be_true()
}

pub fn interview_contract_accepts_ask_question_payload_test() {
  let payload =
    "{\"action\":\"ask_question\",\"session\":{\"id\":\"interview-1\",\"profile\":\"api\",\"created_at\":\"2026-02-06T00:00:00Z\",\"updated_at\":\"2026-02-06T00:00:00Z\",\"stage\":\"discovery\"},\"progress\":{\"current_round\":1,\"total_rounds\":5,\"questions_asked\":0,\"questions_remaining\":18,\"percent_complete\":0},\"agent_protocol\":{\"target\":\"claude_code\",\"contract_version\":\"v1\",\"goal\":\"turn user intent into plan-ready requirements\"},\"question\":{\"id\":\"r1-user-api-1\",\"round\":1,\"text\":\"In one sentence, what should this API do?\",\"pattern\":\"ubiquitous\",\"context\":\"Start simple\",\"examples\":[\"Allow users to log in\"],\"priority\":\"critical\",\"perspective\":\"user\",\"extract_into\":[\"name\"]},\"guidance\":{\"ask_exactly\":true,\"next_command\":\"intent interview --session interview-1 --answer \\\"<human answer>\\\"\",\"planning_focus\":\"Define the core user outcome and canonical happy path\"}}"

  interview_contract.validate_ai_directive_json(payload)
  |> should.be_ok()
}

pub fn interview_contract_accepts_generate_beads_payload_test() {
  let payload =
    "{\"action\":\"generate_beads\",\"session\":{\"id\":\"interview-1\",\"profile\":\"api\",\"created_at\":\"2026-02-06T00:00:00Z\",\"updated_at\":\"2026-02-06T00:00:00Z\",\"stage\":\"complete\"},\"progress\":{\"current_round\":5,\"total_rounds\":5,\"questions_asked\":18,\"questions_remaining\":0,\"percent_complete\":100},\"agent_protocol\":{\"target\":\"claude_code\",\"contract_version\":\"v1\",\"goal\":\"turn user intent into plan-ready requirements\"},\"guidance\":{\"next_command\":\"intent beads interview-1\",\"planning_focus\":\"Turn this captured intent into atomic, dependency-aware work items\"}}"

  interview_contract.validate_ai_directive_json(payload)
  |> should.be_ok()
}

pub fn interview_contract_rejects_missing_protocol_test() {
  let payload =
    "{\"action\":\"ask_question\",\"session\":{\"id\":\"interview-1\",\"profile\":\"api\",\"created_at\":\"2026-02-06T00:00:00Z\",\"updated_at\":\"2026-02-06T00:00:00Z\",\"stage\":\"discovery\"},\"progress\":{\"current_round\":1,\"total_rounds\":5,\"questions_asked\":0,\"questions_remaining\":18,\"percent_complete\":0},\"question\":{\"id\":\"r1-user-api-1\",\"round\":1,\"text\":\"In one sentence, what should this API do?\",\"pattern\":\"ubiquitous\",\"context\":\"Start simple\",\"examples\":[\"Allow users to log in\"],\"priority\":\"critical\",\"perspective\":\"user\",\"extract_into\":[\"name\"]},\"guidance\":{\"ask_exactly\":true,\"next_command\":\"intent interview --session interview-1 --answer \\\"<human answer>\\\"\",\"planning_focus\":\"Define the core user outcome and canonical happy path\"}}"

  interview_contract.validate_ai_directive_json(payload)
  |> should.be_error()
}

// ============================================================================
// HTTP Client Tests
// ============================================================================

// pub fn http_client_url_construction_simple_test() {
//   // Test simple URL construction without interpolation
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/users/123",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   case result {
//     Error(_) -> {
//       // Expected to fail without mocking HTTP - we're testing URL construction logic
//       should.be_ok(Ok(Nil))
//     }
//     Ok(_) -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_path_interpolation_test() {
//   // Test path interpolation with variables
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/users/${user_id}",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx =
//     interpolate.new_context()
//     |> interpolate.set_variable("user_id", json.string("123"))
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Path interpolation should work - URL construction should proceed
//   // Even if HTTP request fails, interpolation error should not occur
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_missing_variable_interpolation_test() {
//   // Test that missing variables in path cause interpolation errors
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/users/${unknown_var}",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// pub fn http_client_header_interpolation_test() {
//   // Test header interpolation with variables
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.from_list([#("X-Default", "default-value")]),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/users",
//       headers: dict.from_list([#("X-Token", "${auth_token}")]),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx =
//     interpolate.new_context()
//     |> interpolate.set_variable("auth_token", json.string("secret123"))
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Header interpolation should work
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_header_merge_test() {
//   // Test that request headers override config headers
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.from_list([
//         #("X-Default", "config-value"),
//         #("X-Config-Only", "config"),
//       ]),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/users",
//       headers: dict.from_list([#("X-Default", "request-value")]),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Header merge should work without interpolation errors
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_body_json_interpolation_test() {
//   // Test body interpolation with JSON content
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let body_json =
//     json.object([
//       #("username", json.string("${username}")),
//       #("email", json.string("user@example.com")),
//     ])
// 
//   let request =
//     types.Request(
//       method: types.Post,
//       path: "/users",
//       headers: dict.new(),
//       query: dict.new(),
//       body: body_json,
//     )
// 
//   let ctx =
//     interpolate.new_context()
//     |> interpolate.set_variable("username", json.string("john_doe"))
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Body interpolation should work
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_invalid_url_test() {
//   // Test invalid URL handling
//   let config =
//     types.Config(
//       base_url: "not a valid url at all",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/users",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   case result {
//     Error(http_client.UrlParseError(_)) -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// pub fn http_client_https_url_test() {
//   // Test HTTPS URL handling
//   let config =
//     types.Config(
//       base_url: "https://api.example.com",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/secure-endpoint",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // HTTPS URLs should be valid and not cause UrlParseError
//   case result {
//     Error(http_client.UrlParseError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_custom_port_test() {
//   // Test URL with custom port
//   let config =
//     types.Config(
//       base_url: "http://localhost:3000",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/health",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Custom port should be parsed correctly
//   case result {
//     Error(http_client.UrlParseError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_path_leading_slash_test() {
//   // Test that paths are normalized with leading slash
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   // Path without leading slash
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "users/123",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Should handle path without leading slash (not a URL parse error)
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_method_conversion_get_test() {
//   // Test that GET method is handled correctly
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/users",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // GET request should not cause method conversion errors
//   case result {
//     Error(http_client.UrlParseError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_method_conversion_post_test() {
//   // Test that POST method with body is handled correctly
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.new(),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Post,
//       path: "/users",
//       headers: dict.new(),
//       query: dict.new(),
//       body: json.object([#("name", json.string("John"))]),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // POST request should not cause method conversion errors
//   case result {
//     Error(http_client.UrlParseError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// pub fn http_client_multiple_header_merge_test() {
//   // Test merging multiple headers from both config and request
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.from_list([
//         #("X-API-Version", "v1"),
//         #("User-Agent", "intent-cli"),
//       ]),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/data",
//       headers: dict.from_list([
//         #("Authorization", "Bearer token"),
//         #("X-Request-ID", "123"),
//       ]),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Multiple headers should merge without errors
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }
// 
// // // ============================================================================
// // // Rules Engine Tests
// // // ============================================================================
// // 
// // fn make_execution_result(
// //   status: Int,
// //   body_str: String,
// //   method: types.Method,
// //   path: String,
// // ) -> http_client.ExecutionResult {
// //   http_client.ExecutionResult(
// //     status: status,
// //     headers: dict.new(),
// //     body: json.object([#("test", json.string(body_str))]),
// //     raw_body: body_str,
// //     elapsed_ms: 100,
// //     request_method: method,
// //     request_path: path,
// //   )
// // }
// // 
// // pub fn rules_engine_check_when_status_equals_test() {
// //   // Test status condition with exact match (== 200)
// //   let rule =
// //     types.Rule(
// //       name: "Check 200 OK",
// //       description: "Verify 200 response",
// //       when: types.When(status: "== 200", method: types.Get, path: "/users"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response = make_execution_result(200, "ok", types.Get, "/users")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   list.length(results)
// //   |> should.equal(1)
// // 
// //   case results {
// //     [rules_engine.RulePassed(name)] -> name |> should.equal("Check 200 OK")
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_when_status_greater_than_test() {
// //   // Test status condition with > operator
// //   let rule =
// //     types.Rule(
// //       name: "Check 4xx error",
// //       description: "Verify error status",
// //       when: types.When(status: "> 399", method: types.Post, path: "/create"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response =
// //     make_execution_result(400, "bad request", types.Post, "/create")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   list.length(results)
// //   |> should.equal(1)
// // 
// //   case results {
// //     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_when_status_less_than_test() {
// //   // Test status condition with < operator
// //   let rule =
// //     types.Rule(
// //       name: "Check success range",
// //       description: "Verify 2xx status",
// //       when: types.When(status: "< 300", method: types.Get, path: "/data"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response = make_execution_result(201, "created", types.Get, "/data")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   case results {
// //     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_when_method_mismatch_test() {
// //   // Test that rule doesn't apply when method doesn't match
// //   let rule =
// //     types.Rule(
// //       name: "POST rule",
// //       description: "Only for POST",
// //       when: types.When(status: "== 200", method: types.Post, path: "/create"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response = make_execution_result(200, "ok", types.Get, "/create")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   // Rule should not apply because method is GET, not POST
// //   list.length(results)
// //   |> should.equal(0)
// // }
// // 
// // pub fn rules_engine_check_when_path_exact_match_test() {
// //   // Test exact path matching
// //   let rule =
// //     types.Rule(
// //       name: "Exact path rule",
// //       description: "Check exact path",
// //       when: types.When(status: "== 200", method: types.Get, path: "/exact/path"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response = make_execution_result(200, "ok", types.Get, "/exact/path")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   case results {
// //     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_when_path_regex_match_test() {
// //   // Test regex path matching
// //   let rule =
// //     types.Rule(
// //       name: "Regex path rule",
// //       description: "Check regex path",
// //       when: types.When(status: "== 200", method: types.Get, path: "^/users/.*"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response = make_execution_result(200, "ok", types.Get, "/users/123")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   case results {
// //     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_body_must_contain_test() {
// //   // Test body_must_contain check
// //   let rule =
// //     types.Rule(
// //       name: "Body content rule",
// //       description: "Verify body contains text",
// //       when: types.When(status: "== 200", method: types.Get, path: "/test"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: ["success"],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response =
// //     make_execution_result(200, "Operation was a success", types.Get, "/test")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   case results {
// //     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_body_must_not_contain_test() {
// //   // Test body_must_not_contain check
// //   let rule =
// //     types.Rule(
// //       name: "No error rule",
// //       description: "Verify no error in body",
// //       when: types.When(status: "== 200", method: types.Get, path: "/test"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: ["error"],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response =
// //     make_execution_result(200, "This is clean data", types.Get, "/test")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   case results {
// //     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_body_must_not_contain_violation_test() {
// //   // Test body_must_not_contain violation
// //   let rule =
// //     types.Rule(
// //       name: "No error rule",
// //       description: "Verify no error in body",
// //       when: types.When(status: "== 200", method: types.Get, path: "/test"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: ["error"],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response =
// //     make_execution_result(200, "This has an error in it", types.Get, "/test")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   case results {
// //     [rules_engine.RuleFailed(name, _, violations)] -> {
// //       name |> should.equal("No error rule")
// //       list.length(violations) |> should.equal(1)
// //     }
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_body_must_contain_violation_test() {
// //   // Test body_must_contain violation
// //   let rule =
// //     types.Rule(
// //       name: "Required text rule",
// //       description: "Verify required text",
// //       when: types.When(status: "== 200", method: types.Get, path: "/test"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: ["required"],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response =
// //     make_execution_result(200, "This is missing it", types.Get, "/test")
// //   let results = rules_engine.check_rules([rule], response, "test_behavior")
// // 
// //   case results {
// //     [rules_engine.RuleFailed(_, _, violations)] -> {
// //       list.length(violations) |> should.equal(1)
// //     }
// //     _ -> should.fail()
// //   }
// // }
// // 
// // pub fn rules_engine_check_multiple_rules_test() {
// //   // Test multiple rules applied in sequence
// //   let rule1 =
// //     types.Rule(
// //       name: "Rule 1",
// //       description: "First rule",
// //       when: types.When(status: "== 200", method: types.Get, path: "/test"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let rule2 =
// //     types.Rule(
// //       name: "Rule 2",
// //       description: "Second rule",
// //       when: types.When(status: "== 200", method: types.Get, path: "/test"),
// //       check: types.RuleCheck(
// //         body_must_not_contain: [],
// //         body_must_contain: [],
// //         fields_must_exist: [],
// //         fields_must_not_exist: [],
// //         header_must_exist: "",
// //         header_must_not_exist: "",
// //       ),
// //       example: json.null(),
// //     )
// // 
// //   let response = make_execution_result(200, "ok", types.Get, "/test")
// //   let results =
// //     rules_engine.check_rules([rule1, rule2], response, "test_behavior")
// // 
// //   list.length(results) |> should.equal(2)
// // }
// // 
// // pub fn rules_engine_format_violation_body_contains_test() {
// //   let violation = rules_engine.BodyContains("forbidden", "response body")
// //   let formatted = rules_engine.format_violation(violation)
// //   formatted
// //   |> string.contains("forbidden")
// //   |> should.be_true()
// // }
// // 
// // pub fn rules_engine_format_violation_body_missing_test() {
// //   let violation = rules_engine.BodyMissing("required")
// //   let formatted = rules_engine.format_violation(violation)
// //   formatted
// //   |> string.contains("required")
// //   |> should.be_true()
// // }
// // 
// // pub fn rules_engine_format_violation_field_missing_test() {
// //   let violation = rules_engine.FieldMissing("user.id")
// //   let formatted = rules_engine.format_violation(violation)
// //   formatted
// //   |> string.contains("user.id")
// //   |> should.be_true()
// // }
// // 
// // pub fn rules_engine_format_violation_header_missing_test() {
// //   let violation = rules_engine.HeaderMissing("X-Custom")
// //   let formatted = rules_engine.format_violation(violation)
// //   formatted
// //   |> string.contains("X-Custom")
// //   |> should.be_true()
// // }
// // 
// // // ============================================================================
// // Resolver Advanced Tests
// // ============================================================================
// 
// pub fn resolver_complex_diamond_dependency_test() {
//   // Diamond pattern: b3 and b4 both depend on b1, b5 depends on both
//   let b1 = make_behavior("base", [])
//   let b3 = make_behavior("left", ["base"])
//   let b4 = make_behavior("right", ["base"])
//   let b5 = make_behavior("merge", ["left", "right"])
// 
//   let spec = make_spec([make_feature("Feature A", [b1, b3, b4, b5])])
//   let result = resolver.resolve_execution_order(spec)
// 
//   case result {
//     Ok(resolved) -> {
//       list.length(resolved) |> should.equal(4)
//       let names = list.map(resolved, fn(rb) { rb.behavior.name })
//       // base should come first
//       let assert [first, ..] = names
//       first |> should.equal("base")
//       // merge should come last (it has two dependencies)
//       case list.last(names) {
//         Ok(last) -> last |> should.equal("merge")
//         Error(_) -> should.fail()
//       }
//     }
//     Error(_) -> should.fail()
//   }
// }
// 
// pub fn resolver_multiple_branches_test() {
//   // Multiple independent branches
//   let b1 = make_behavior("root", [])
//   let b2 = make_behavior("branch-a-1", ["root"])
//   let b3 = make_behavior("branch-a-2", ["branch-a-1"])
//   let b4 = make_behavior("branch-b-1", ["root"])
//   let b5 = make_behavior("branch-b-2", ["branch-b-1"])
// 
//   let spec = make_spec([make_feature("Feature", [b1, b2, b3, b4, b5])])
//   let result = resolver.resolve_execution_order(spec)
// 
//   case result {
//     Ok(resolved) -> {
//       list.length(resolved) |> should.equal(5)
//       let names = list.map(resolved, fn(rb) { rb.behavior.name })
//       // Check that all expected behaviors are present
//       list.any(names, fn(n) { n == "root" }) |> should.be_true()
//       list.any(names, fn(n) { n == "branch-a-1" }) |> should.be_true()
//       list.any(names, fn(n) { n == "branch-a-2" }) |> should.be_true()
//       list.any(names, fn(n) { n == "branch-b-1" }) |> should.be_true()
//       list.any(names, fn(n) { n == "branch-b-2" }) |> should.be_true()
//     }
//     Error(_) -> should.fail()
//   }
// }
// 
// pub fn resolver_deep_chain_test() {
//   // Long dependency chain: b5 -> b4 -> b3 -> b2 -> b1
//   let b1 = make_behavior("step1", [])
//   let b2 = make_behavior("step2", ["step1"])
//   let b3 = make_behavior("step3", ["step2"])
//   let b4 = make_behavior("step4", ["step3"])
//   let b5 = make_behavior("step5", ["step4"])
// 
//   let spec = make_spec([make_feature("Feature", [b1, b2, b3, b4, b5])])
//   let result = resolver.resolve_execution_order(spec)
// 
//   case result {
//     Ok(resolved) -> {
//       list.length(resolved) |> should.equal(5)
//       let names = list.map(resolved, fn(rb) { rb.behavior.name })
//       names |> should.equal(["step1", "step2", "step3", "step4", "step5"])
//     }
//     Error(_) -> should.fail()
//   }
// }
// 
// // ============================================================================
// // Empty/Null Response Handling Tests
// // ============================================================================
// 
// pub fn rules_engine_empty_body_test() {
//   // Test rule application with empty response body
//   let rule =
//     types.Rule(
//       name: "Empty body rule",
//       description: "Handle empty response",
//       when: types.When(
//         status: "== 204",
//         method: types.Delete,
//         path: "/resource",
//       ),
//       check: types.RuleCheck(
//         body_must_not_contain: [],
//         body_must_contain: [],
//         fields_must_exist: [],
//         fields_must_not_exist: [],
//         header_must_exist: "",
//         header_must_not_exist: "",
//       ),
//       example: json.null(),
//     )
// 
//   let response =
//     http_client.ExecutionResult(
//       status: 204,
//       headers: dict.new(),
//       body: json.null(),
//       raw_body: "",
//       elapsed_ms: 50,
//       request_method: types.Delete,
//       request_path: "/resource",
//     )
// 
//   let results = rules_engine.check_rules([rule], response, "test")
//   case results {
//     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// pub fn rules_engine_null_json_value_test() {
//   // Test handling of null JSON values
//   let rule =
//     types.Rule(
//       name: "Null handling rule",
//       description: "Handle null values",
//       when: types.When(status: "== 200", method: types.Get, path: "/nullable"),
//       check: types.RuleCheck(
//         body_must_not_contain: [],
//         body_must_contain: [],
//         fields_must_exist: [],
//         fields_must_not_exist: [],
//         header_must_exist: "",
//         header_must_not_exist: "",
//       ),
//       example: json.null(),
//     )
// 
//   let response =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.object([#("value", json.null())]),
//       raw_body: "{\"value\":null}",
//       elapsed_ms: 60,
//       request_method: types.Get,
//       request_path: "/nullable",
//     )
// 
//   let results = rules_engine.check_rules([rule], response, "test")
//   case results {
//     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// pub fn rules_engine_whitespace_body_test() {
//   // Test handling of whitespace-only body
//   let rule =
//     types.Rule(
//       name: "Whitespace rule",
//       description: "Handle whitespace body",
//       when: types.When(status: "== 200", method: types.Get, path: "/test"),
//       check: types.RuleCheck(
//         body_must_not_contain: ["error"],
//         body_must_contain: [],
//         fields_must_exist: [],
//         fields_must_not_exist: [],
//         header_must_exist: "",
//         header_must_not_exist: "",
//       ),
//       example: json.null(),
//     )
// 
//   let response =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.null(),
//       raw_body: "   \n\t  ",
//       elapsed_ms: 40,
//       request_method: types.Get,
//       request_path: "/test",
//     )
// 
//   let results = rules_engine.check_rules([rule], response, "test")
//   case results {
//     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// pub fn rules_engine_nested_null_field_test() {
//   // Test checking for null in nested fields
//   let rule =
//     types.Rule(
//       name: "Nested null rule",
//       description: "Check nested fields",
//       when: types.When(status: "== 200", method: types.Get, path: "/nested"),
//       check: types.RuleCheck(
//         body_must_not_contain: [],
//         body_must_contain: [],
//         fields_must_exist: ["user"],
//         fields_must_not_exist: [],
//         header_must_exist: "",
//         header_must_not_exist: "",
//       ),
//       example: json.null(),
//     )
// 
//   let response =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.object([#("user", json.null())]),
//       raw_body: "{\"user\":null}",
//       elapsed_ms: 55,
//       request_method: types.Get,
//       request_path: "/nested",
//     )
// 
//   let results = rules_engine.check_rules([rule], response, "test")
//   case results {
//     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// pub fn rules_engine_empty_object_test() {
//   // Test handling of empty objects
//   let rule =
//     types.Rule(
//       name: "Empty object rule",
//       description: "Handle empty objects",
//       when: types.When(status: "== 200", method: types.Get, path: "/data"),
//       check: types.RuleCheck(
//         body_must_not_contain: [],
//         body_must_contain: [],
//         fields_must_exist: ["data"],
//         fields_must_not_exist: [],
//         header_must_exist: "",
//         header_must_not_exist: "",
//       ),
//       example: json.null(),
//     )
// 
//   let response =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.object([#("data", json.object([]))]),
//       raw_body: "{\"data\":{}}",
//       elapsed_ms: 65,
//       request_method: types.Get,
//       request_path: "/data",
//     )
// 
//   let results = rules_engine.check_rules([rule], response, "test")
//   case results {
//     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// // ============================================================================
// // Unicode and Special Character Support Tests
// // ============================================================================
// 
// pub fn interpolate_unicode_variable_test() {
//   let ctx =
//     interpolate.new_context()
//     |> interpolate.set_variable("emoji", json_string("🎉"))
// 
//   let result = interpolate.interpolate_string(ctx, "status: ${emoji}")
// 
//   case result {
//     Ok(s) -> s |> should.equal("status: 🎉")
//     Error(_) -> should.fail()
//   }
// }
// 
// pub fn interpolate_unicode_in_path_test() {
//   let ctx =
//     interpolate.new_context()
//     |> interpolate.set_variable("category", json_string("réclame"))
// 
//   let result = interpolate.interpolate_string(ctx, "/search/${category}")
// 
//   case result {
//     Ok(s) -> s |> should.equal("/search/réclame")
//     Error(_) -> should.fail()
//   }
// }
// 
// pub fn rules_engine_unicode_body_content_test() {
//   // Test body checks with Unicode characters
//   let rule =
//     types.Rule(
//       name: "Unicode content rule",
//       description: "Check for Unicode in response",
//       when: types.When(status: "== 200", method: types.Get, path: "/message"),
//       check: types.RuleCheck(
//         body_must_not_contain: [],
//         body_must_contain: ["✓"],
//         fields_must_exist: [],
//         fields_must_not_exist: [],
//         header_must_exist: "",
//         header_must_not_exist: "",
//       ),
//       example: json.null(),
//     )
// 
//   let response =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.null(),
//       raw_body: "Status: ✓ All systems operational",
//       elapsed_ms: 50,
//       request_method: types.Get,
//       request_path: "/message",
//     )
// 
//   let results = rules_engine.check_rules([rule], response, "test")
//   case results {
//     [rules_engine.RulePassed(_)] -> should.be_ok(Ok(Nil))
//     _ -> should.fail()
//   }
// }
// 
// pub fn rules_engine_emoji_in_description_test() {
//   // Test emoji in rule descriptions
//   let rule =
//     types.Rule(
//       name: "emoji_test",
//       description: "Check emoji support 🚀 in descriptions",
//       when: types.When(status: "== 200", method: types.Get, path: "/status"),
//       check: types.RuleCheck(
//         body_must_not_contain: [],
//         body_must_contain: [],
//         fields_must_exist: [],
//         fields_must_not_exist: [],
//         header_must_exist: "",
//         header_must_not_exist: "",
//       ),
//       example: json.null(),
//     )
// 
//   let response =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.null(),
//       raw_body: "ok",
//       elapsed_ms: 50,
//       request_method: types.Get,
//       request_path: "/status",
//     )
// 
//   let results = rules_engine.check_rules([rule], response, "test")
//   // Description should contain emoji but not affect rule execution
//   case results {
//     [rules_engine.RulePassed(name)] -> name |> should.equal("emoji_test")
//     _ -> should.fail()
//   }
// }
// 
// pub fn interpolate_special_characters_test() {
//   let ctx =
//     interpolate.new_context()
//     |> interpolate.set_variable("special", json_string("@#$%^&*()"))
// 
//   let result = interpolate.interpolate_string(ctx, "chars: ${special}")
// 
//   case result {
//     Ok(s) -> s |> should.equal("chars: @#$%^&*()")
//     Error(_) -> should.fail()
//   }
// }
// 
// pub fn http_client_unicode_header_test() {
//   // Test Unicode in HTTP headers
//   let config =
//     types.Config(
//       base_url: "http://localhost:8080",
//       timeout_ms: 5000,
//       headers: dict.from_list([#("X-Custom", "café")]),
//     )
// 
//   let request =
//     types.Request(
//       method: types.Get,
//       path: "/test",
//       headers: dict.from_list([#("X-Greeting", "こんにちは")]),
//       query: dict.new(),
//       body: json.null(),
//     )
// 
//   let ctx = interpolate.new_context()
// 
//   let result = http_client.execute_request(config, request, ctx)
// 
//   // Should handle Unicode headers without interpolation errors
//   case result {
//     Error(http_client.InterpolationError(_)) -> should.fail()
//     _ -> should.be_ok(Ok(Nil))
//   }
// }

// ============================================================================
// Output Formatting Tests
// ============================================================================

pub fn json_encoding_test() {
  // Test JSON encoding of various values
  let value =
    json.object([
      #("name", json.string("Test")),
      #("count", json.int(42)),
      #("enabled", json.bool(True)),
    ])

  let json_str = json.to_string(value)

  // Verify JSON is properly formatted
  json_str
  |> string.contains("Test")
  |> should.be_true()

  json_str
  |> string.contains("42")
  |> should.be_true()
}

pub fn summary_calculation_test() {
  // Test that summary calculations work correctly
  let passed = 10
  let failed = 3
  let blocked = 1
  let total = passed + failed + blocked

  total |> should.equal(14)

  let percentage = passed * 100 / total
  percentage |> should.equal(71)
}

pub fn string_formatting_test() {
  // Test human-readable string formatting
  let behavior_name = "get-user-by-id"
  let feature_name = "User Management"

  let formatted = feature_name <> ": " <> behavior_name

  formatted
  |> should.equal("User Management: get-user-by-id")
}

pub fn error_message_formatting_test() {
  // Test error message formatting
  let field = "status"
  let expected = "200"
  let actual = "404"

  let message =
    "Field '"
    <> field
    <> "' expected '"
    <> expected
    <> "' but got '"
    <> actual
    <> "'"

  message
  |> string.contains("status")
  |> should.be_true()

  message
  |> string.contains("200")
  |> should.be_true()

  message
  |> string.contains("404")
  |> should.be_true()
}

pub fn list_to_string_formatting_test() {
  // Test formatting lists into readable strings
  let items = ["first", "second", "third"]
  let formatted = string.join(items, ", ")

  formatted |> should.equal("first, second, third")
}

pub fn boolean_to_status_test() {
  // Test converting boolean results to status strings
  let passed = True
  let status = case passed {
    True -> "PASS"
    False -> "FAIL"
  }

  status |> should.equal("PASS")
}

pub fn json_null_handling_test() {
  // Test JSON null handling in output
  let value = json.null()
  let json_str = json.to_string(value)

  json_str |> should.equal("null")
}

// ============================================================================
// Beads Generation Tests
// ============================================================================

pub fn bead_generation_api_profile_test() {
  // Test generating beads from API profile session

  let answers = [
    interview.Answer(
      question_id: "q1",
      question_text: "What API endpoints do we need?",
      perspective: Security,
      round: 1,
      response: "We need GET /users, POST /users, PUT /users/:id, DELETE /users/:id",
      extracted: dict.from_list([#("endpoints", "users")]),
      confidence: 0.9,
      notes: "",
      timestamp: "2026-02-06T00:00:00Z",
    ),
  ]

  let session =
    make_test_interview_session(
      "api-test-session",
      answers,
      [],
      [],
      interview.Discovery,
    )

  let beads = bead_templates.generate_beads_from_session(session)

  // Verify beads were generated
  list.is_empty(beads) |> should.equal(False)

  // Verify first bead structure
  case list.first(beads) {
    Ok(first_bead) -> {
      first_bead.profile_type |> should.equal("api")
      first_bead.issue_type |> should.equal("api_endpoint")
      first_bead.priority |> should.equal(3)
    }
    Error(_) -> should.fail()
  }
}

pub fn beads_to_jsonl_multiple_test() {
  // Test converting multiple beads to JSONL format
  let beads = [
    bead_templates.BeadRecord(
      title: "First Bead",
      description: "First task",
      profile_type: "api",
      priority: 3,
      issue_type: "feature",
      labels: ["high"],
      ai_hints: "Do this first",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "Second Bead",
      description: "Second task",
      profile_type: "data",
      priority: 2,
      issue_type: "schema",
      labels: ["medium"],
      ai_hints: "Then do this",
      acceptance_criteria: [],
      dependencies: ["First Bead"],
    ),
  ]

  let jsonl = bead_templates.beads_to_jsonl(beads)

  // Verify multiple lines
  let lines = string.split(jsonl, "\n")
  list.length(lines) |> should.equal(2)

  // Verify both beads are present
  jsonl |> string.contains("First Bead") |> should.be_true()
  jsonl |> string.contains("Second Bead") |> should.be_true()
  jsonl |> string.contains("First task") |> should.be_true()
  jsonl |> string.contains("Second task") |> should.be_true()
}

pub fn beads_to_enhanced_cue_contains_schema_entries_test() {
  let beads = [
    bead_templates.BeadRecord(
      title: "Implement CLI command",
      description: "Add a command for release checks",
      profile_type: "cli",
      priority: 2,
      issue_type: "cli_command",
      labels: ["cli"],
      ai_hints: "Follow existing command patterns",
      acceptance_criteria: ["Command runs"],
      dependencies: [],
    ),
  ]

  let cue_payload = bead_templates.beads_to_enhanced_cue(beads)

  cue_payload |> string.contains("package schema") |> should.be_true()
  cue_payload |> string.contains("#EnhancedBead") |> should.be_true()
  cue_payload
  |> string.contains("CLI: Implement CLI command")
  |> should.be_true()
}

pub fn beads_to_enhanced_cue_normalizes_out_of_range_priority_test() {
  let beads = [
    bead_templates.BeadRecord(
      title: "Out of range priority",
      description: "Should clamp priority to schema range",
      profile_type: "api",
      priority: 99,
      issue_type: "api_endpoint",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
  ]

  let cue_payload = bead_templates.beads_to_enhanced_cue(beads)

  cue_payload |> string.contains("priority: 4") |> should.be_true()
}

pub fn bead_validation_header_includes_cue_vet_instructions_test() {
  let bead =
    bead_templates.BeadRecord(
      title: "CLI smoke",
      description: "Run command matrix",
      profile_type: "cli",
      priority: 1,
      issue_type: "task",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    )

  let updated =
    bead_templates.with_validation_header(
      bead,
      ".beads/schemas/intent-cli-cli1.cue",
    )

  updated.description
  |> string.contains(
    "cue vet .beads/schemas/intent-cli-cli1.cue implementation.cue",
  )
  |> should.be_true()
}

pub fn bead_stats_calculation_test() {
  // Test bead statistics calculation
  let beads = [
    bead_templates.BeadRecord(
      title: "API 1",
      description: "desc",
      profile_type: "api",
      priority: 3,
      issue_type: "endpoint",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "API 2",
      description: "desc",
      profile_type: "api",
      priority: 3,
      issue_type: "endpoint",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "Data 1",
      description: "desc",
      profile_type: "data",
      priority: 2,
      issue_type: "schema",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
  ]

  let stats = bead_templates.bead_stats(beads)

  stats.total |> should.equal(3)

  // Verify by_type counts
  dict.get(stats.by_type, "endpoint")
  |> should.equal(Ok(2))

  dict.get(stats.by_type, "schema")
  |> should.equal(Ok(1))

  // Verify by_priority counts
  dict.get(stats.by_priority, 3)
  |> should.equal(Ok(2))

  dict.get(stats.by_priority, 2)
  |> should.equal(Ok(1))
}

pub fn filter_beads_by_type_test() {
  // Test filtering beads by issue type
  let beads = [
    bead_templates.BeadRecord(
      title: "Endpoint 1",
      description: "desc",
      profile_type: "api",
      priority: 1,
      issue_type: "endpoint",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "Schema 1",
      description: "desc",
      profile_type: "data",
      priority: 1,
      issue_type: "schema",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "Endpoint 2",
      description: "desc",
      profile_type: "api",
      priority: 1,
      issue_type: "endpoint",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
  ]

  let endpoints = bead_templates.filter_beads_by_type(beads, "endpoint")
  list.length(endpoints) |> should.equal(2)

  let schemas = bead_templates.filter_beads_by_type(beads, "schema")
  list.length(schemas) |> should.equal(1)
}

pub fn sort_beads_by_priority_test() {
  // Test sorting beads by priority (higher number = higher priority)
  let beads = [
    bead_templates.BeadRecord(
      title: "Low Priority",
      description: "desc",
      profile_type: "api",
      priority: 1,
      issue_type: "task",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "High Priority",
      description: "desc",
      profile_type: "api",
      priority: 5,
      issue_type: "task",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "Medium Priority",
      description: "desc",
      profile_type: "api",
      priority: 3,
      issue_type: "task",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
  ]

  let sorted = bead_templates.sort_beads_by_priority(beads)

  // First should be highest priority
  case list.first(sorted) {
    Ok(first) -> first.title |> should.equal("High Priority")
    Error(_) -> should.fail()
  }

  // Last should be lowest priority
  case list.last(sorted) {
    Ok(last) -> last.title |> should.equal("Low Priority")
    Error(_) -> should.fail()
  }
}

pub fn add_bead_dependency_test() {
  // Test adding dependencies between beads
  let beads = [
    bead_templates.BeadRecord(
      title: "Schema Design",
      description: "desc",
      profile_type: "data",
      priority: 1,
      issue_type: "schema",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
    bead_templates.BeadRecord(
      title: "API Endpoint",
      description: "desc",
      profile_type: "api",
      priority: 1,
      issue_type: "endpoint",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
  ]

  let updated =
    bead_templates.add_dependency(beads, "API Endpoint", "Schema Design")

  case list.last(updated) {
    Ok(endpoint_bead) -> {
      endpoint_bead.title |> should.equal("API Endpoint")
      list.contains(endpoint_bead.dependencies, "Schema Design")
      |> should.be_true()
    }
    Error(_) -> should.fail()
  }
}

pub fn empty_session_beads_test() {
  // Test generating beads from session with no answers

  let session =
    make_test_interview_session("test-session", [], [], [], interview.Discovery)

  let beads = bead_templates.generate_beads_from_session(session)

  // Verify no beads were generated when there are no answers
  list.is_empty(beads) |> should.be_true()
}

pub fn bead_stats_empty_list_test() {
  // Test stats calculation with empty bead list
  let beads: List(bead_templates.BeadRecord) = []
  let stats = bead_templates.bead_stats(beads)

  stats.total |> should.equal(0)
  dict.is_empty(stats.by_type) |> should.be_true()
  dict.is_empty(stats.by_priority) |> should.be_true()
}

pub fn bead_multiple_dependencies_test() {
  // Test adding multiple dependencies to a bead
  let beads = [
    bead_templates.BeadRecord(
      title: "Implementation",
      description: "desc",
      profile_type: "api",
      priority: 1,
      issue_type: "endpoint",
      labels: [],
      ai_hints: "",
      acceptance_criteria: [],
      dependencies: [],
    ),
  ]

  let step1 = bead_templates.add_dependency(beads, "Implementation", "Schema")
  let step2 = bead_templates.add_dependency(step1, "Implementation", "Auth")

  case list.first(step2) {
    Ok(bead) -> {
      list.length(bead.dependencies) |> should.equal(2)
      list.contains(bead.dependencies, "Schema") |> should.be_true()
      list.contains(bead.dependencies, "Auth") |> should.be_true()
    }
    Error(_) -> should.fail()
  }
}

// --- Status Code Tests ---

// pub fn checker_status_code_match_test() {
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: dict.new(),
//       headers: dict.new(),
//     )
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.null(),
//       raw_body: json.to_string(json.null()),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   result.status_ok |> should.be_true()
//   result.status_expected |> should.equal(200)
//   result.status_actual |> should.equal(200)
// }
// 
// pub fn checker_status_code_mismatch_test() {
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: dict.new(),
//       headers: dict.new(),
//     )
//   let actual =
//     http_client.ExecutionResult(
//       status: 404,
//       headers: dict.new(),
//       body: json.null(),
//       raw_body: json.to_string(json.null()),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   result.status_ok |> should.be_false()
//   result.status_expected |> should.equal(200)
//   result.status_actual |> should.equal(404)
// }
// 
// // --- Field Check Tests ---
// 
// pub fn checker_field_equals_string_pass_test() {
//   let checks =
//     dict.from_list([
//       #("name", types.Check(rule: "equals John", why: "Name must match")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.string("John"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
//   list.length(result.failed) |> should.equal(0)
// }
// 
// pub fn checker_field_equals_string_fail_test() {
//   let checks =
//     dict.from_list([
//       #("name", types.Check(rule: "equals John", why: "Name must match")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.string("Jane"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_equals_int_pass_test() {
//   let checks =
//     dict.from_list([
//       #("age", types.Check(rule: "equals 25", why: "Age must match")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("age", json.int(25))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
//   list.length(result.failed) |> should.equal(0)
// }
// 
// pub fn checker_field_is_string_pass_test() {
//   let checks =
//     dict.from_list([
//       #("name", types.Check(rule: "string", why: "Must be string")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.string("test"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
//   list.length(result.failed) |> should.equal(0)
// }
// 
// pub fn checker_field_is_string_fail_test() {
//   let checks =
//     dict.from_list([
//       #("name", types.Check(rule: "string", why: "Must be string")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.int(123))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_integer_pass_test() {
//   let checks =
//     dict.from_list([
//       #("count", types.Check(rule: "integer", why: "Must be integer")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("count", json.int(42))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_boolean_pass_test() {
//   let checks =
//     dict.from_list([
//       #("active", types.Check(rule: "boolean", why: "Must be boolean")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("active", json.bool(True))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_array_pass_test() {
//   let checks =
//     dict.from_list([
//       #("items", types.Check(rule: "array", why: "Must be array")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([
//       #("items", json.array([json.int(1), json.int(2)], fn(x) { x })),
//     ])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_object_pass_test() {
//   let checks =
//     dict.from_list([
//       #("data", types.Check(rule: "object", why: "Must be object")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([#("data", json.object([#("key", json.string("value"))]))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_present_pass_test() {
//   let checks =
//     dict.from_list([
//       #("id", types.Check(rule: "present", why: "ID must be present")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("id", json.string("abc-123"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
//   list.length(result.failed) |> should.equal(0)
// }
// 
// pub fn checker_field_present_fail_test() {
//   let checks =
//     dict.from_list([
//       #("id", types.Check(rule: "present", why: "ID must be present")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.string("test"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_absent_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "password",
//         types.Check(rule: "absent", why: "Password should not be returned"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.string("test"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
//   list.length(result.failed) |> should.equal(0)
// }
// 
// pub fn checker_field_absent_fail_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "password",
//         types.Check(rule: "absent", why: "Password should not be returned"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("password", json.string("secret"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_non_empty_string_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "name",
//         types.Check(rule: "non-empty string", why: "Name must not be empty"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.string("John"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_non_empty_string_fail_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "name",
//         types.Check(rule: "non-empty string", why: "Name must not be empty"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("name", json.string(""))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_email_pass_test() {
//   let checks =
//     dict.from_list([
//       #("email", types.Check(rule: "email", why: "Must be valid email")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("email", json.string("user@example.com"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_email_fail_test() {
//   let checks =
//     dict.from_list([
//       #("email", types.Check(rule: "email", why: "Must be valid email")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("email", json.string("not-an-email"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_uuid_pass_test() {
//   let checks =
//     dict.from_list([
//       #("id", types.Check(rule: "uuid", why: "Must be valid UUID")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([#("id", json.string("550e8400-e29b-41d4-a716-446655440000"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_uuid_fail_test() {
//   let checks =
//     dict.from_list([
//       #("id", types.Check(rule: "uuid", why: "Must be valid UUID")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("id", json.string("not-a-uuid"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_is_iso8601_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "created_at",
//         types.Check(rule: "iso8601 datetime", why: "Must be valid datetime"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("created_at", json.string("2024-01-15T10:30:00Z"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// // --- Numeric Comparison Tests ---
// 
// pub fn checker_field_integer_gte_pass_test() {
//   let checks =
//     dict.from_list([
//       #("count", types.Check(rule: "integer >= 5", why: "Must be at least 5")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("count", json.int(10))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_integer_gte_fail_test() {
//   let checks =
//     dict.from_list([
//       #("count", types.Check(rule: "integer >= 5", why: "Must be at least 5")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("count", json.int(3))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_field_integer_lte_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "count",
//         types.Check(rule: "integer <= 100", why: "Must not exceed 100"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("count", json.int(50))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_number_between_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "age",
//         types.Check(
//           rule: "number between 18.0 and 65.0",
//           why: "Age must be in range",
//         ),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("age", json.int(30))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_field_number_between_fail_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "age",
//         types.Check(
//           rule: "number between 18.0 and 65.0",
//           why: "Age must be in range",
//         ),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("age", json.int(17))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.failed) |> should.equal(1)
// }
// 
// // --- String Pattern Tests ---
// 
// pub fn checker_string_starts_with_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "code",
//         types.Check(rule: "string starting with ERR-", why: "Error code format"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("code", json.string("ERR-001"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_string_ends_with_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "file",
//         types.Check(rule: "string ending with .json", why: "Must be JSON file"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("file", json.string("config.json"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_string_containing_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "message",
//         types.Check(
//           rule: "string containing success",
//           why: "Should mention success",
//         ),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([#("message", json.string("Operation success complete"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// // --- Array Tests ---
// 
// pub fn checker_non_empty_array_pass_test() {
//   let checks =
//     dict.from_list([
//       #("items", types.Check(rule: "non-empty array", why: "Must have items")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("items", json.array([json.int(1)], fn(x) { x }))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_non_empty_array_fail_test() {
//   let checks =
//     dict.from_list([
//       #("items", types.Check(rule: "non-empty array", why: "Must have items")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("items", json.array([], fn(x) { x }))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_array_of_length_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "coords",
//         types.Check(rule: "array of length 3", why: "Must have 3 elements"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([
//       #(
//         "coords",
//         json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x }),
//       ),
//     ])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_array_min_items_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "tags",
//         types.Check(rule: "array with min 2 items", why: "Need at least 2 tags"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([
//       #(
//         "tags",
//         json.array(
//           [json.string("a"), json.string("b"), json.string("c")],
//           fn(x) { x },
//         ),
//       ),
//     ])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// // --- One Of Tests ---
// 
// pub fn checker_one_of_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "status",
//         types.Check(
//           rule: "one of [active, inactive, pending]",
//           why: "Valid status",
//         ),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("status", json.string("active"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_one_of_fail_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "status",
//         types.Check(
//           rule: "one of [active, inactive, pending]",
//           why: "Valid status",
//         ),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body = json.object([#("status", json.string("unknown"))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.failed) |> should.equal(1)
// }
// 
// // --- Header Check Tests ---
// 
// pub fn checker_header_present_pass_test() {
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.null(),
//       checks: dict.new(),
//       headers: dict.from_list([#("Content-Type", "application/json")]),
//     )
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.from_list([#("Content-Type", "application/json")]),
//       body: json.null(),
//       raw_body: json.to_string(json.null()),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
//   list.length(result.failed) |> should.equal(0)
// }
// 
// pub fn checker_header_value_mismatch_test() {
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.null(),
//       checks: dict.new(),
//       headers: dict.from_list([#("Content-Type", "application/json")]),
//     )
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.from_list([#("Content-Type", "text/plain")]),
//       body: json.null(),
//       raw_body: json.to_string(json.null()),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }

// pub fn checker_header_missing_test() {
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.null(),
//       checks: dict.new(),
//       headers: dict.from_list([#("X-Request-Id", "abc-123")]),
//     )
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: json.null(),
//       raw_body: json.to_string(json.null()),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(0)
//   list.length(result.failed) |> should.equal(1)
// }
// 
// pub fn checker_header_case_insensitive_test() {
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.null(),
//       checks: dict.new(),
//       headers: dict.from_list([#("content-type", "application/json")]),
//     )
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.from_list([#("Content-Type", "application/json")]),
//       body: json.null(),
//       raw_body: json.to_string(json.null()),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// // --- Nested Field Tests ---
// 
// pub fn checker_nested_field_pass_test() {
//   let checks =
//     dict.from_list([
//       #(
//         "user.name",
//         types.Check(rule: "equals John", why: "User name must match"),
//       ),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([
//         #("user", json.object([#("name", json.string("John"))])),
//       ]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([#("user", json.object([#("name", json.string("John"))]))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(1)
// }
// 
// pub fn checker_nested_field_missing_test() {
//   let checks =
//     dict.from_list([
//       #("user.email", types.Check(rule: "is email", why: "Must have email")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([
//         #("user", json.object([#("name", json.string("John"))])),
//       ]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([#("user", json.object([#("name", json.string("John"))]))])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.failed) |> should.equal(1)
// }
// 
// // --- Multiple Checks Test ---
// 
// pub fn checker_multiple_checks_test() {
//   let checks =
//     dict.from_list([
//       #("id", types.Check(rule: "uuid", why: "ID must be UUID")),
//       #("name", types.Check(rule: "non-empty string", why: "Name required")),
//       #("email", types.Check(rule: "email", why: "Email required")),
//       #("age", types.Check(rule: "integer >= 0", why: "Age must be positive")),
//     ])
//   let expected =
//     types.Response(
//       status: 200,
//       example: json.object([
//         #("id", json.string("550e8400-e29b-41d4-a716-446655440000")),
//         #("name", json.string("John")),
//         #("email", json.string("john@example.com")),
//         #("age", json.int(30)),
//       ]),
//       checks: checks,
//       headers: dict.new(),
//     )
//   let body =
//     json.object([
//       #("id", json.string("550e8400-e29b-41d4-a716-446655440000")),
//       #("name", json.string("John")),
//       #("email", json.string("john@example.com")),
//       #("age", json.int(30)),
//     ])
//   let actual =
//     http_client.ExecutionResult(
//       status: 200,
//       headers: dict.new(),
//       body: body,
//       raw_body: json.to_string(body),
//       elapsed_ms: 100,
//       request_method: types.Get,
//       request_path: "/test",
//     )
//   let result = checker.check_response(expected, actual, empty_context())
// 
//   list.length(result.passed) |> should.equal(4)
//   list.length(result.failed) |> should.equal(0)
// }
// 
// ============================================================================
// Custom Question Loading Tests
// ============================================================================

// Helper to create a test question
fn make_test_question(id: String, round: Int, question_text: String) -> Question {
  Question(
    id,
    round,
    User,
    HappyPath,
    Critical,
    question_text,
    "Test context",
    "Test example",
    "text",
    [],
    [],
    [],
  )
}

pub fn question_loader_merge_empty_custom_returns_base_test() {
  // When custom questions are empty (None), base should be returned unchanged
  let base =
    question_loader.ProfileQuestions(
      round_1: [make_test_question("q1", 1, "Question 1")],
      round_2: [make_test_question("q2", 2, "Question 2")],
    )
  let custom = None

  let result = merge_profile_test(base, custom)

  list.length(result.round_1) |> should.equal(1)
  list.length(result.round_2) |> should.equal(1)
}

pub fn question_loader_merge_adds_new_questions_test() {
  // Custom questions with new IDs should be added
  let base =
    question_loader.ProfileQuestions(
      round_1: [make_test_question("q1", 1, "Question 1")],
      round_2: [],
    )
  let custom =
    Some(question_loader.CustomProfileQuestions(
      round_1: Some([make_test_question("q-new", 1, "New Question")]),
      round_2: None,
    ))

  let result = merge_profile_test(base, custom)

  // Should have both original and new question
  list.length(result.round_1) |> should.equal(2)
}

pub fn question_loader_merge_overrides_by_id_test() {
  // Custom question with same ID should override the base
  let base =
    question_loader.ProfileQuestions(
      round_1: [make_test_question("q1", 1, "Original Question")],
      round_2: [],
    )
  let custom =
    Some(question_loader.CustomProfileQuestions(
      round_1: Some([make_test_question("q1", 1, "Overridden Question")]),
      round_2: None,
    ))

  let result = merge_profile_test(base, custom)

  // Should still have just one question (the override)
  list.length(result.round_1) |> should.equal(1)

  // The question should be the overridden one
  case result.round_1 {
    [q] -> q.question |> should.equal("Overridden Question")
    _ -> should.fail()
  }
}

pub fn question_loader_merge_preserves_non_overridden_test() {
  // Questions not overridden should remain
  let base =
    question_loader.ProfileQuestions(
      round_1: [
        make_test_question("q1", 1, "Question 1"),
        make_test_question("q2", 1, "Question 2"),
        make_test_question("q3", 1, "Question 3"),
      ],
      round_2: [],
    )
  let custom =
    Some(question_loader.CustomProfileQuestions(
      round_1: Some([make_test_question("q2", 1, "Overridden Q2")]),
      round_2: None,
    ))

  let result = merge_profile_test(base, custom)

  // Should have 3 questions (2 preserved + 1 override)
  list.length(result.round_1) |> should.equal(3)

  // Find the overridden question
  let overridden = list.filter(result.round_1, fn(q) { q.id == "q2" })
  case overridden {
    [q] -> q.question |> should.equal("Overridden Q2")
    _ -> should.fail()
  }
}

pub fn question_loader_merge_common_rounds_test() {
  // Test merging common questions (rounds 3-5)
  let base =
    question_loader.CommonQuestions(
      round_3: [make_test_question("r3-q1", 3, "Round 3 Q1")],
      round_4: [make_test_question("r4-q1", 4, "Round 4 Q1")],
      round_5: [],
    )
  let custom =
    Some(question_loader.CustomCommonQuestions(
      round_3: None,
      round_4: Some([make_test_question("r4-q1", 4, "Overridden R4 Q1")]),
      round_5: Some([make_test_question("r5-new", 5, "New Round 5 Q")]),
    ))

  let result = merge_common_test(base, custom)

  // Round 3 unchanged
  list.length(result.round_3) |> should.equal(1)

  // Round 4 has override
  list.length(result.round_4) |> should.equal(1)
  case result.round_4 {
    [q] -> q.question |> should.equal("Overridden R4 Q1")
    _ -> should.fail()
  }

  // Round 5 has new question
  list.length(result.round_5) |> should.equal(1)
}

pub fn question_loader_file_not_found_test() {
  // Loading from non-existent file should return FileNotFound error
  let result = question_loader.load_custom_questions("/nonexistent/path.cue")

  case result {
    Error(question_loader.FileNotFound(_)) -> should.be_true(True)
    _ -> should.fail()
  }
}

// Helper to test merge_profile
// Replicates the merge logic since merge_custom_questions is private
fn merge_profile_test(
  base: question_loader.ProfileQuestions,
  custom: option.Option(question_loader.CustomProfileQuestions),
) -> question_loader.ProfileQuestions {
  case custom {
    None -> base
    Some(c) ->
      question_loader.ProfileQuestions(
        round_1: merge_question_list_test(base.round_1, c.round_1),
        round_2: merge_question_list_test(base.round_2, c.round_2),
      )
  }
}

fn merge_common_test(
  base: question_loader.CommonQuestions,
  custom: option.Option(question_loader.CustomCommonQuestions),
) -> question_loader.CommonQuestions {
  case custom {
    None -> base
    Some(c) ->
      question_loader.CommonQuestions(
        round_3: merge_question_list_test(base.round_3, c.round_3),
        round_4: merge_question_list_test(base.round_4, c.round_4),
        round_5: merge_question_list_test(base.round_5, c.round_5),
      )
  }
}

fn merge_question_list_test(
  base: List(Question),
  custom: option.Option(List(Question)),
) -> List(Question) {
  case custom {
    None -> base
    Some(custom_questions) -> {
      let custom_ids = list.map(custom_questions, fn(q) { q.id })
      let filtered_base =
        list.filter(base, fn(q) { !list.contains(custom_ids, q.id) })
      list.append(filtered_base, custom_questions)
    }
  }
}

// ============================================================================
// Session Diff Tests
// ============================================================================
fn make_test_interview_session(
  id: String,
  answers: List(interview.Answer),
  gaps: List(interview.Gap),
  conflicts: List(interview.Conflict),
  stage: interview.InterviewStage,
) -> interview.InterviewSession {
  interview.InterviewSession(
    id: id,
    profile: interview.Api,
    created_at: "2026-01-05T00:00:00Z",
    updated_at: "2026-01-05T00:00:00Z",
    completed_at: "",
    stage: stage,
    rounds_completed: 0,
    answers: answers,
    gaps: gaps,
    conflicts: conflicts,
    raw_notes: "",
    current_phase: 1,
    completed_phases: [],
  )
}
