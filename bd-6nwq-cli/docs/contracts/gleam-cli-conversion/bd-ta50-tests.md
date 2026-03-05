# Martin Fowler Test Plan: Add Missing Spec Fields

## Happy Path Tests
- test_parse_spec_with_all_new_fields
- test_parse_spec_with_audience_only
- test_parse_spec_with_version_only
- test_parse_spec_with_success_criteria_only
- test_serialize_spec_includes_new_fields

## Error Path Tests
- (No error paths - fields have defaults)

## Edge Case Tests
- test_parse_spec_without_new_fields_uses_defaults
- test_empty_success_criteria_allowed
- test_empty_audience_allowed
- test_empty_version_allowed

## Contract Verification Tests
- test_precondition_spec_struct_exists
- test_postcondition_audience_field_exists
- test_postcondition_version_field_exists
- test_postcondition_success_criteria_field_exists
- test_postcondition_defaults_are_empty
- test_invariant_existing_specs_parse

## Contract Violation Tests
- `test_missing_fields_violation_succeeds`
  Given: JSON spec without audience, version, success_criteria
  When: Parsed into Spec struct
  Then: Succeeds with empty string/vec defaults, NOT error

- `test_default_values_violation_empty`
  Given: Parsed spec without new fields
  When: Access audience, version, success_criteria
  Then: Returns "" and [], NOT "unknown" or other values

## Given-When-Then Scenarios
### Scenario 1: Full spec parsing
Given: JSON with all fields including new ones
When: Parsed into Spec
Then: All fields including audience, version, success_criteria are populated

### Scenario 2: Backward compatibility
Given: JSON without new fields (old format)
When: Parsed into Spec
Then: Parsing succeeds with defaults

### Scenario 3: Round-trip serialization
Given: A Spec with new fields set
When: Serialized to JSON and parsed back
Then: All fields including new ones are preserved
