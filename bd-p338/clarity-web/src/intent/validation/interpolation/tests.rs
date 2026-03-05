//! Tests for Context variables storing JSON values
//!
//! These tests verify that Context.variables can store serde_json::Value
//! instead of just String, enabling array indexing and nested object access.

use serde_json::json;

use super::context::Context;
use super::errors::InterpolationError;
use super::resolve::{resolve_path, resolve_variable, value_to_string};
use serde_json::Value;

// ============================================================================
// 1. Context Value Storage Tests
// ============================================================================

mod context_value_storage {
  use super::*;

  /// GIVEN: a new empty context
  /// WHEN: a string variable is added
  /// THEN: the variable is retrievable as a Value::String
  #[test]
  fn given_empty_context_when_add_string_variable_then_retrievable_as_string_value() {
    // GIVEN
    let ctx = Context::new();

    // WHEN
    let ctx = ctx.with_variable("name", "Alice");

    // THEN
    let value = ctx.variables.get("name");
    assert!(value.is_some());
    let value = value.unwrap();
    assert!(matches!(value, Value::String(s) if s == "Alice"));
  }

  /// GIVEN: a new empty context
  /// WHEN: an integer variable is added via Value
  /// THEN: the variable is retrievable as a Value::Number
  #[test]
  fn given_empty_context_when_add_integer_variable_then_retrievable_as_number_value() {
    // GIVEN
    let ctx = Context::new();

    // WHEN
    let ctx = ctx.with_variable("count", json!(42));

    // THEN
    let value = ctx.variables.get("count");
    assert!(value.is_some());
    let value = value.unwrap();
    assert!(matches!(value, Value::Number(n) if n.as_i64() == Some(42)));
  }

  /// GIVEN: a new empty context
  /// WHEN: a boolean variable is added via Value
  /// THEN: the variable is retrievable as a Value::Bool
  #[test]
  fn given_empty_context_when_add_boolean_variable_then_retrievable_as_bool_value() {
    // GIVEN
    let ctx = Context::new();

    // WHEN
    let ctx = ctx.with_variable("active", json!(true));

    // THEN
    let value = ctx.variables.get("active");
    assert!(value.is_some());
    let value = value.unwrap();
    assert!(matches!(value, Value::Bool(true)));
  }

  /// GIVEN: a new empty context
  /// WHEN: a null value is added via Value
  /// THEN: the variable is retrievable as Value::Null
  #[test]
  fn given_empty_context_when_add_null_variable_then_retrievable_as_null_value() {
    // GIVEN
    let ctx = Context::new();

    // WHEN
    let ctx = ctx.with_variable("empty", json!(null));

    // THEN
    let value = ctx.variables.get("empty");
    assert!(value.is_some());
    let value = value.unwrap();
    assert!(matches!(value, Value::Null));
  }

  /// GIVEN: a new empty context
  /// WHEN: an array value is added
  /// THEN: the array is preserved as Value::Array
  #[test]
  fn given_empty_context_when_add_array_variable_then_array_preserved() {
    // GIVEN
    let ctx = Context::new();
    let items = json!([1, 2, 3]);

    // WHEN
    let ctx = ctx.with_variable("items", items.clone());

    // THEN
    let value = ctx.variables.get("items");
    assert!(value.is_some());
    let value = value.unwrap();
    assert!(matches!(value, Value::Array(arr) if arr.len() == 3));
  }

  /// GIVEN: a new empty context
  /// WHEN: an object value is added
  /// THEN: the object is preserved as Value::Object
  #[test]
  fn given_empty_context_when_add_object_variable_then_object_preserved() {
    // GIVEN
    let ctx = Context::new();
    let user = json!({"name": "Alice", "age": 30});

    // WHEN
    let ctx = ctx.with_variable("user", user.clone());

    // THEN
    let value = ctx.variables.get("user");
    assert!(value.is_some());
    let value = value.unwrap();
    assert!(matches!(value, Value::Object(obj) if obj.get("name").is_some()));
  }
}

// ============================================================================
// 2. Value Type Resolution Tests
// ============================================================================

mod value_type_resolution {
  use super::*;

  /// GIVEN: context with string variable
  /// WHEN: resolving the string variable
  /// THEN: returns the string value as-is
  #[test]
  fn given_context_with_string_when_resolve_then_returns_string() {
    // GIVEN
    let ctx = Context::new().with_variable("name", json!("Alice"));

    // WHEN
    let result = resolve_variable("name", &ctx);

    // THEN
    assert_eq!(result, Ok("Alice".to_string()));
  }

  /// GIVEN: context with number variable
  /// WHEN: resolving the number variable
  /// THEN: returns number as string representation
  #[test]
  fn given_context_with_number_when_resolve_then_returns_number_as_string() {
    // GIVEN
    let ctx = Context::new().with_variable("count", json!(42));

    // WHEN
    let result = resolve_variable("count", &ctx);

    // THEN
    assert_eq!(result, Ok("42".to_string()));
  }

  /// GIVEN: context with boolean variable
  /// WHEN: resolving the boolean variable
  /// THEN: returns "true" or "false"
  #[test]
  fn given_context_with_boolean_when_resolve_then_returns_true_or_false_string() {
    // GIVEN
    let ctx_true = Context::new().with_variable("flag", json!(true));
    let ctx_false = Context::new().with_variable("flag", json!(false));

    // WHEN
    let result_true = resolve_variable("flag", &ctx_true);
    let result_false = resolve_variable("flag", &ctx_false);

    // THEN
    assert_eq!(result_true, Ok("true".to_string()));
    assert_eq!(result_false, Ok("false".to_string()));
  }

  /// GIVEN: context with null variable
  /// WHEN: resolving the null variable
  /// THEN: returns empty string
  #[test]
  fn given_context_with_null_when_resolve_then_returns_empty_string() {
    // GIVEN
    let ctx = Context::new().with_variable("empty", json!(null));

    // WHEN
    let result = resolve_variable("empty", &ctx);

    // THEN
    assert_eq!(result, Ok(String::new()));
  }

  /// GIVEN: context with array variable
  /// WHEN: resolving the array variable without index
  /// THEN: returns JSON string of array
  #[test]
  fn given_context_with_array_when_resolve_without_index_then_returns_json_string() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!([1, 2, 3]));

    // WHEN
    let result = resolve_variable("items", &ctx);

    // THEN
    assert_eq!(result, Ok("[1,2,3]".to_string()));
  }

  /// GIVEN: context with object variable
  /// WHEN: resolving the object variable without path
  /// THEN: returns JSON string of object
  #[test]
  fn given_context_with_object_when_resolve_without_path_then_returns_json_string() {
    // GIVEN
    let ctx = Context::new().with_variable("user", json!({"name": "Alice"}));

    // WHEN
    let result = resolve_variable("user", &ctx);

    // THEN
    assert_eq!(result, Ok("{\"name\":\"Alice\"}".to_string()));
  }
}

// ============================================================================
// 3. Array Indexing Tests
// ============================================================================

mod array_indexing {
  use super::*;

  /// GIVEN: context with array variable
  /// WHEN: resolving with positive index [0]
  /// THEN: returns first element as string
  #[test]
  fn given_context_with_array_when_resolve_index_zero_then_returns_first_element() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!(["a", "b", "c"]));

    // WHEN
    let result = resolve_variable("items[0]", &ctx);

    // THEN
    assert_eq!(result, Ok("a".to_string()));
  }

  /// GIVEN: context with array variable
  /// WHEN: resolving with positive index [1]
  /// THEN: returns second element as string
  #[test]
  fn given_context_with_array_when_resolve_index_one_then_returns_second_element() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!(["a", "b", "c"]));

    // WHEN
    let result = resolve_variable("items[1]", &ctx);

    // THEN
    assert_eq!(result, Ok("b".to_string()));
  }

  /// GIVEN: context with array variable
  /// WHEN: resolving with positive index [2] on 3-element array
  /// THEN: returns last element as string
  #[test]
  fn given_context_with_array_when_resolve_last_index_then_returns_last_element() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!(["a", "b", "c"]));

    // WHEN
    let result = resolve_variable("items[2]", &ctx);

    // THEN
    assert_eq!(result, Ok("c".to_string()));
  }

  /// GIVEN: context with array variable
  /// WHEN: resolving with negative index [-1]
  /// THEN: returns last element as string
  #[test]
  fn given_context_with_array_when_resolve_negative_one_then_returns_last_element() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!(["a", "b", "c"]));

    // WHEN
    let result = resolve_variable("items[-1]", &ctx);

    // THEN
    assert_eq!(result, Ok("c".to_string()));
  }

  /// GIVEN: context with array variable
  /// WHEN: resolving with negative index [-2]
  /// THEN: returns second-to-last element
  #[test]
  fn given_context_with_array_when_resolve_negative_two_then_returns_second_to_last() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!(["a", "b", "c"]));

    // WHEN
    let result = resolve_variable("items[-2]", &ctx);

    // THEN
    assert_eq!(result, Ok("b".to_string()));
  }

  /// GIVEN: context with 3-element array
  /// WHEN: resolving with index [100]
  /// THEN: returns IndexOutOfBounds error
  #[test]
  fn given_context_with_small_array_when_resolve_large_index_then_returns_out_of_bounds() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!([1, 2, 3]));

    // WHEN
    let result = resolve_variable("items[100]", &ctx);

    // THEN
    assert!(matches!(
      result,
      Err(InterpolationError::IndexOutOfBounds {
        index: 100,
        length: 3
      })
    ));
  }

  /// GIVEN: context with empty array
  /// WHEN: resolving with index [0]
  /// THEN: returns IndexOutOfBounds error
  #[test]
  fn given_context_with_empty_array_when_resolve_any_index_then_returns_out_of_bounds() {
    // GIVEN
    let ctx = Context::new().with_variable("empty", json!([]));

    // WHEN
    let result = resolve_variable("empty[0]", &ctx);

    // THEN
    assert!(matches!(
      result,
      Err(InterpolationError::IndexOutOfBounds { length: 0, .. })
    ));
  }

  /// GIVEN: context with string variable
  /// WHEN: resolving with array index [0]
  /// THEN: returns NotAnArray error
  #[test]
  fn given_context_with_string_when_resolve_with_index_then_returns_not_an_array() {
    // GIVEN
    let ctx = Context::new().with_variable("name", json!("Alice"));

    // WHEN
    let result = resolve_variable("name[0]", &ctx);

    // THEN
    assert!(matches!(result, Err(InterpolationError::NotAnArray(_))));
  }
}

// ============================================================================
// 4. Nested Path Navigation Tests
// ============================================================================

mod nested_path_navigation {
  use super::*;

  /// GIVEN: context with object variable
  /// WHEN: resolving nested field path user.name
  /// THEN: returns nested field value
  #[test]
  fn given_context_with_object_when_resolve_field_path_then_returns_nested_value() {
    // GIVEN
    let ctx = Context::new().with_variable("user", json!({"name": "Alice", "age": 30}));

    // WHEN
    let result = resolve_variable("user.name", &ctx);

    // THEN
    assert_eq!(result, Ok("Alice".to_string()));
  }

  /// GIVEN: context with deeply nested object
  /// WHEN: resolving 3-level path data.user.profile
  /// THEN: returns deeply nested value
  #[test]
  fn given_context_with_nested_object_when_resolve_deep_path_then_returns_deep_value() {
    // GIVEN
    let ctx = Context::new().with_variable(
      "data",
      json!({"user": {"profile": {"email": "test@example.com"}}}),
    );

    // WHEN
    let result = resolve_variable("data.user.profile", &ctx);

    // THEN
    assert_eq!(result, Ok("{\"email\":\"test@example.com\"}".to_string()));
  }

  /// GIVEN: context with array of objects
  /// WHEN: resolving items[0].name
  /// THEN: returns field from first array element
  #[test]
  fn given_context_with_object_array_when_resolve_indexed_field_then_returns_field_value() {
    // GIVEN
    let ctx = Context::new().with_variable("users", json!([{"name": "Alice"}, {"name": "Bob"}]));

    // WHEN
    let result = resolve_variable("users[0].name", &ctx);

    // THEN
    assert_eq!(result, Ok("Alice".to_string()));
  }

  /// GIVEN: context with object containing array field
  /// WHEN: resolving user.emails[0]
  /// THEN: returns first element of nested array
  #[test]
  fn given_context_with_array_field_when_resolve_nested_index_then_returns_element() {
    // GIVEN
    let ctx = Context::new().with_variable(
      "user",
      json!({"name": "Alice", "emails": ["alice@work.com", "alice@home.com"]}),
    );

    // WHEN
    let result = resolve_variable("user.emails[0]", &ctx);

    // THEN
    assert_eq!(result, Ok("alice@work.com".to_string()));
  }

  /// GIVEN: context with object without requested field
  /// WHEN: resolving user.missing
  /// THEN: returns FieldNotFound error
  #[test]
  fn given_context_with_object_when_resolve_missing_field_then_returns_not_found() {
    // GIVEN
    let ctx = Context::new().with_variable("user", json!({"name": "Alice"}));

    // WHEN
    let result = resolve_variable("user.missing", &ctx);

    // THEN
    assert!(matches!(
      result,
      Err(InterpolationError::VariableNotFound(_))
    ));
  }
}

// ============================================================================
// 5. Error Handling Tests
// ============================================================================

mod error_handling {
  use super::*;

  /// GIVEN: empty context
  /// WHEN: resolving unknown variable
  /// THEN: returns VariableNotFound error
  #[test]
  fn given_empty_context_when_resolve_unknown_then_returns_variable_not_found() {
    // GIVEN
    let ctx = Context::new();

    // WHEN
    let result = resolve_variable("unknown", &ctx);

    // THEN
    assert!(matches!(result, Err(InterpolationError::VariableNotFound(name)) if name == "unknown"));
  }

  /// GIVEN: any context
  /// WHEN: resolving empty path
  /// THEN: returns InvalidPath error
  #[test]
  fn given_any_context_when_resolve_empty_path_then_returns_invalid_path() {
    // GIVEN
    let ctx = Context::new();

    // WHEN
    let result = resolve_path("", &ctx);

    // THEN
    assert!(matches!(result, Err(InterpolationError::InvalidPath(msg)) if msg.contains("empty")));
  }

  /// GIVEN: any context
  /// WHEN: resolving whitespace-only path
  /// THEN: returns InvalidPath error
  #[test]
  fn given_any_context_when_resolve_whitespace_path_then_returns_invalid_path() {
    // GIVEN
    let ctx = Context::new();

    // WHEN
    let result = resolve_path("   ", &ctx);

    // THEN
    assert!(matches!(result, Err(InterpolationError::InvalidPath(_))));
  }

  /// GIVEN: any context
  /// WHEN: resolving path with double dots user..name
  /// THEN: returns InvalidPath error
  #[test]
  fn given_any_context_when_resolve_double_dot_path_then_returns_invalid_path() {
    // GIVEN
    let ctx = Context::new().with_variable("user", json!({"name": "Alice"}));

    // WHEN
    let result = resolve_variable("user..name", &ctx);

    // THEN
    assert!(matches!(result, Err(InterpolationError::InvalidPath(_))));
  }

  /// GIVEN: context with array
  /// WHEN: resolving path with unclosed bracket items[0
  /// THEN: returns InvalidPath error
  #[test]
  fn given_context_with_array_when_resolve_unclosed_bracket_then_returns_invalid_path() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!([1, 2, 3]));

    // WHEN
    let result = resolve_variable("items[0", &ctx);

    // THEN
    assert!(
      matches!(result, Err(InterpolationError::InvalidPath(msg)) if msg.contains("unclosed"))
    );
  }

  /// GIVEN: context with array
  /// WHEN: resolving with non-numeric index items[abc]
  /// THEN: returns InvalidPath error
  #[test]
  fn given_context_with_array_when_resolve_non_numeric_index_then_returns_invalid_path() {
    // GIVEN
    let ctx = Context::new().with_variable("items", json!([1, 2, 3]));

    // WHEN
    let result = resolve_variable("items[abc]", &ctx);

    // THEN
    assert!(matches!(result, Err(InterpolationError::InvalidPath(msg)) if msg.contains("invalid")));
  }
}

// ============================================================================
// 6. Backward Compatibility Tests
// ============================================================================

mod backward_compatibility {
  use super::*;

  /// GIVEN: existing code using with_variable with &str
  /// WHEN: context is used for resolution
  /// THEN: string is stored and resolved correctly
  #[test]
  fn given_legacy_code_when_add_string_variable_then_still_works() {
    // GIVEN (legacy code pattern)
    let ctx = Context::new().with_variable("name", "Alice");

    // WHEN
    let result = resolve_variable("name", &ctx);

    // THEN (same behavior as before)
    assert_eq!(result, Ok("Alice".to_string()));
  }

  /// GIVEN: existing code using from_variables with String pairs
  /// WHEN: context is used for resolution
  /// THEN: all variables are stored and resolved correctly
  #[test]
  fn given_legacy_code_when_use_from_variables_then_still_works() {
    // GIVEN (legacy code pattern)
    let vars = vec![
      ("name".to_string(), "Alice".to_string()),
      ("city".to_string(), "London".to_string()),
    ];
    let ctx = Context::from_variables(vars);

    // WHEN
    let name = resolve_variable("name", &ctx);
    let city = resolve_variable("city", &ctx);

    // THEN (same behavior as before)
    assert_eq!(name, Ok("Alice".to_string()));
    assert_eq!(city, Ok("London".to_string()));
  }

  /// GIVEN: existing code using chained with_variable
  /// WHEN: context is used for resolution
  /// THEN: all variables are accessible
  #[test]
  fn given_legacy_code_when_chain_with_variable_then_still_works() {
    // GIVEN (legacy code pattern)
    let ctx = Context::new()
      .with_variable("a", "1")
      .with_variable("b", "2")
      .with_variable("c", "3");

    // WHEN
    let a = resolve_variable("a", &ctx);
    let b = resolve_variable("b", &ctx);
    let c = resolve_variable("c", &ctx);

    // THEN
    assert_eq!(a, Ok("1".to_string()));
    assert_eq!(b, Ok("2".to_string()));
    assert_eq!(c, Ok("3".to_string()));
  }
}

// ============================================================================
// 7. Edge Case Tests
// ============================================================================

mod edge_cases {
  use super::*;

  /// GIVEN: context with unicode string
  /// WHEN: resolving the variable
  /// THEN: unicode is preserved correctly
  #[test]
  fn given_context_with_unicode_when_resolve_then_unicode_preserved() {
    // GIVEN
    let ctx = Context::new().with_variable("greeting", "こんにちは");

    // WHEN
    let result = resolve_variable("greeting", &ctx);

    // THEN
    assert_eq!(result, Ok("こんにちは".to_string()));
  }

  /// GIVEN: context with empty string variable
  /// WHEN: resolving the variable
  /// THEN: returns empty string
  #[test]
  fn given_context_with_empty_string_when_resolve_then_returns_empty() {
    // GIVEN
    let ctx = Context::new().with_variable("empty", "");

    // WHEN
    let result = resolve_variable("empty", &ctx);

    // THEN
    assert_eq!(result, Ok(String::new()));
  }

  /// GIVEN: context with large number
  /// WHEN: resolving the variable
  /// THEN: number is converted to string correctly
  #[test]
  fn given_context_with_large_number_when_resolve_then_converts_correctly() {
    // GIVEN
    let ctx = Context::new().with_variable("big", json!(i64::MAX));

    // WHEN
    let result = resolve_variable("big", &ctx);

    // THEN
    assert_eq!(result, Ok(i64::MAX.to_string()));
  }

  /// GIVEN: context with float number
  /// WHEN: resolving the variable
  /// THEN: float is converted to string correctly
  #[test]
  fn given_context_with_float_when_resolve_then_converts_correctly() {
    // GIVEN
    let ctx = Context::new().with_variable("pi", json!(3.14159));

    // WHEN
    let result = resolve_variable("pi", &ctx);

    // THEN
    assert!(result.unwrap().starts_with("3.14"));
  }

  /// GIVEN: context with array of mixed types
  /// WHEN: resolving specific index
  /// THEN: correct type is returned
  #[test]
  fn given_context_with_mixed_array_when_resolve_index_then_returns_correct_type() {
    // GIVEN
    let ctx = Context::new().with_variable("mixed", json!([1, "two", true, null]));

    // WHEN
    let first = resolve_variable("mixed[0]", &ctx);
    let second = resolve_variable("mixed[1]", &ctx);
    let third = resolve_variable("mixed[2]", &ctx);
    let fourth = resolve_variable("mixed[3]", &ctx);

    // THEN
    assert_eq!(first, Ok("1".to_string()));
    assert_eq!(second, Ok("two".to_string()));
    assert_eq!(third, Ok("true".to_string()));
    assert_eq!(fourth, Ok(String::new()));
  }
}

// ============================================================================
// 8. Integration Tests
// ============================================================================

mod integration {
  use super::*;

  /// GIVEN: context with variables, request_body, and response_body
  /// WHEN: resolving from different sources
  /// THEN: correct source is used in priority order
  #[test]
  fn given_full_context_when_resolve_then_uses_priority_order() {
    // GIVEN
    let ctx = Context::new()
      .with_variable("id", json!(123))
      .with_request_body(json!({"data": "request"}))
      .with_response_body(json!({"data": "response"}));

    // WHEN
    let id = resolve_variable("id", &ctx);
    let req = resolve_variable("request.data", &ctx);
    let resp = resolve_variable("response.data", &ctx);

    // THEN
    assert_eq!(id, Ok("123".to_string()));
    assert_eq!(req, Ok("request".to_string()));
    assert_eq!(resp, Ok("response".to_string()));
  }
}

// ============================================================================
// 9. Value to String Unit Tests
// ============================================================================

mod value_to_string_tests {
  use super::*;

  #[test]
  fn given_null_value_when_convert_to_string_then_returns_empty() {
    let result = value_to_string(&Value::Null);
    assert_eq!(result, Ok(String::new()));
  }

  #[test]
  fn given_bool_true_when_convert_to_string_then_returns_true() {
    let result = value_to_string(&Value::Bool(true));
    assert_eq!(result, Ok("true".to_string()));
  }

  #[test]
  fn given_bool_false_when_convert_to_string_then_returns_false() {
    let result = value_to_string(&Value::Bool(false));
    assert_eq!(result, Ok("false".to_string()));
  }

  #[test]
  fn given_number_when_convert_to_string_then_returns_string_representation() {
    let result = value_to_string(&json!(42));
    assert_eq!(result, Ok("42".to_string()));
  }

  #[test]
  fn given_string_when_convert_to_string_then_returns_same_string() {
    let result = value_to_string(&json!("hello"));
    assert_eq!(result, Ok("hello".to_string()));
  }

  #[test]
  fn given_array_when_convert_to_string_then_returns_json_string() {
    let result = value_to_string(&json!([1, 2, 3]));
    assert_eq!(result, Ok("[1,2,3]".to_string()));
  }

  #[test]
  fn given_object_when_convert_to_string_then_returns_json_string() {
    let result = value_to_string(&json!({"key": "value"}));
    assert_eq!(result, Ok("{\"key\":\"value\"}".to_string()));
  }
}
