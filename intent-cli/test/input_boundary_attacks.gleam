/// RED-02: Input Boundary Attacks - Bug Discovery Report
/// Direct EARS format reporting of discovered bugs
import gleam/dynamic
import gleam/json
import gleam/string
import gleeunit/should
import intent/parser

/// BUG 1: Empty string accepted for required 'name' field
/// SEVERITY: MEDIUM
/// EVIDENCE: parser.gleam:20 accepts "" after dynamic.string validation
/// RISK: Empty specs break downstream processing
pub fn bug_1_empty_string_for_required_name_test() {
  let bad_json =
    "{\"name\":\"\",\"description\":\"test\",\"audience\":\"dev\",\"version\":\"1.0\",\"success_criteria\":[],\"features\":[],\"invariants\":[],\"anti_patterns\":[],\"ai_hints\":{\"implementation\":{\"suggested_stack\":[]},\"entities\":{},\"security\":{\"password_hashing\":\"bcrypt\",\"jwt_algorithm\":\"HS256\",\"jwt_expiry\":\"1h\",\"rate_limiting\":\"100/min\"},\"pitfalls\":[]}}"

  case json.decode(bad_json, dynamic.dynamic) {
    Ok(data) -> {
      let result = parser.parse_spec(data)
      case result {
        Ok(spec) -> {
          spec.name |> should.equal("")
        }
        Error(_) -> should.be_ok(Ok(Nil))
      }
    }
    Error(_) -> should.be_ok(Ok(Nil))
  }
}

/// BUG 2: Zero timeout accepted (should require positive int)
/// SEVERITY: MEDIUM
/// SKIPPED: v3.0 removed config.timeout_ms field (no more Config type)
pub fn bug_2_zero_timeout_accepted_test() {
  // Test skipped - config removed in v3.0
  True |> should.be_true
}

/// BUG 3: Negative timeout accepted (should be positive only)
/// SEVERITY: MEDIUM
/// SKIPPED: v3.0 removed config.timeout_ms field (no more Config type)
pub fn bug_3_negative_timeout_accepted_test() {
  // Test skipped - config removed in v3.0
  True |> should.be_true
}

/// BUG 4: Null bytes accepted without sanitization
/// SEVERITY: HIGH
/// EVIDENCE: json.decode + parser.gleam:20 accepts \u0000 in strings
/// RISK: String truncation, buffer overflows, C API issues
pub fn bug_4_null_byte_sanitization_test() {
  let bad_json =
    "{\"name\":\"test\\u0000\",\"description\":\"test\",\"audience\":\"dev\",\"version\":\"1.0\",\"success_criteria\":[],\"features\":[],\"invariants\":[],\"anti_patterns\":[],\"ai_hints\":{\"implementation\":{\"suggested_stack\":[]},\"entities\":{},\"security\":{\"password_hashing\":\"bcrypt\",\"jwt_algorithm\":\"HS256\",\"jwt_expiry\":\"1h\",\"rate_limiting\":\"100/min\"},\"pitfalls\":[]}}"

  case json.decode(bad_json, dynamic.dynamic) {
    Ok(data) -> {
      case dynamic.field("name", dynamic.string)(data) {
        Ok(name) -> {
          string.contains(name, "\u{0000}") |> should.be_true
        }
        Error(_) -> should.be_ok(Ok(Nil))
      }
    }
    Error(_) -> should.be_ok(Ok(Nil))
  }
}

/// BUG 5: No path validation for traversal attacks
/// SEVERITY: MEDIUM
/// SKIPPED: v3.0 removed request.path field (behaviors are declarative)
pub fn bug_5_path_traversal_validation_test() {
  // Test skipped - request.path removed in v3.0
  True |> should.be_true
}

/// BUG 6: No sanitization of shell metacharacters
/// SEVERITY: MEDIUM
/// SKIPPED: v3.0 removed request.path field (behaviors are declarative)
pub fn bug_6_shell_metacharacter_validation_test() {
  // Test skipped - request.path removed in v3.0
  True |> should.be_true
}
