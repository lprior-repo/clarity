/// RED-02: Input Boundary Attacks on intent-cli
/// Critical bugs only for rapid testing
import gleam/dynamic
import gleam/json
import gleam/string
import gleeunit/should
import intent/parser

pub fn empty_string_required_field_name_test() {
  let bad_json =
    "{\"name\":\"\",\"description\":\"test\",\"audience\":\"dev\",\"version\":\"1.0\",\"success_criteria\":[],\"config\":{\"base_url\":\"http://localhost\",\"timeout_ms\":5000,\"headers\":{}},\"features\":[],\"rules\":[],\"anti_patterns\":[],\"ai_hints\":{\"implementation\":{\"suggested_stack\":[]},\"entities\":{},\"security\":{\"password_hashing\":\"bcrypt\",\"jwt_algorithm\":\"HS256\",\"jwt_expiry\":\"1h\",\"rate_limiting\":\"100/min\"},\"pitfalls\":[]}}"

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

pub fn zero_timeout_accepted_test() {
  // Test skipped - v3.0 removed config.timeout_ms field (no more Config type)
  True |> should.be_true
}

pub fn negative_timeout_accepted_test() {
  // Test skipped - v3.0 removed config.timeout_ms field (no more Config type)
  True |> should.be_true
}

pub fn null_byte_in_string_test() {
  let bad_json =
    "{\"name\":\"test\\u0000\",\"description\":\"test\",\"audience\":\"dev\",\"version\":\"1.0\",\"success_criteria\":[],\"config\":{\"base_url\":\"http://localhost\",\"timeout_ms\":5000,\"headers\":{}},\"features\":[],\"rules\":[],\"anti_patterns\":[],\"ai_hints\":{\"implementation\":{\"suggested_stack\":[]},\"entities\":{},\"security\":{\"password_hashing\":\"bcrypt\",\"jwt_algorithm\":\"HS256\",\"jwt_expiry\":\"1h\",\"rate_limiting\":\"100/min\"},\"pitfalls\":[]}}"

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

pub fn path_traversal_accepted_test() {
  // Test skipped - v3.0 removed request.path field (behaviors are declarative)
  True |> should.be_true
}

pub fn shell_metacharacters_accepted_test() {
  // Test skipped - v3.0 removed request.path field (behaviors are declarative)
  True |> should.be_true
}
