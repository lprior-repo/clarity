# Martin Fowler Test Plan: Context Variables Type Fix

## Happy Path Tests
- test_with_variable_stores_json_value
- test_get_variable_returns_stored_value
- test_store_array_value_and_index
- test_store_object_value_and_navigate
- test_store_null_value
- test_store_boolean_value
- test_store_number_value

## Error Path Tests
- test_get_variable_returns_none_for_missing_key
- test_navigate_nonexistent_path_returns_error
- test_type_mismatch_on_navigation_returns_error

## Edge Case Tests
- test_store_empty_array
- test_store_empty_object
- test_store_deeply_nested_value
- test_overwrite_existing_variable

## Contract Verification Tests
- test_precondition_context_exists
- test_postcondition_variables_is_hashmap_string_value
- test_postcondition_set_variable_accepts_value
- test_postcondition_get_variable_returns_option_value
- test_invariant_type_information_preserved

## Contract Violation Tests
- `test_type_preservation_violation_returns_correct_value`
  Given: Store Value::Array([1, 2, 3]) with key "arr"
  When: Call get_variable("arr")
  Then: Returns Some(&Value::Array([1, 2, 3])), NOT Some(&Value::String("[1,2,3]"))

- `test_array_indexing_violation_returns_correct_value`
  Given: Store Value::Array([1, 2, 3]) with key "arr"
  When: Navigate to arr[0]
  Then: Returns Value::Number(1), NOT error

## Given-When-Then Scenarios
### Scenario 1: Store and retrieve array
Given: A new Context
When: I store Value::Array([1, 2, 3]) with key "numbers"
Then: get_variable("numbers") returns Some(&Value::Array([1, 2, 3]))

### Scenario 2: Navigate into stored object
Given: A Context with {"user": {"name": "Alice"}}
When: I navigate path "user.name"
Then: Returns Value::String("Alice")

### Scenario 3: Type preservation across operations
Given: A Context with stored boolean Value::Bool(true)
When: I retrieve the value
Then: The type is still Bool, not String "true"
