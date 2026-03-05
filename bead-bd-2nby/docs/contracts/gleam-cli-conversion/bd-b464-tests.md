# Martin Fowler Test Plan: CLI UI Terminal Formatting

## Happy Path Tests
- test_print_header_outputs_bold_with_separators
- test_print_success_outputs_green_checkmark
- test_print_warning_outputs_yellow_warning
- test_print_error_outputs_red_x
- test_print_info_outputs_blue_info
- test_print_labeled_outputs_label_value_format
- test_print_list_item_outputs_indented_bullet
- test_badge_formats_count_with_color

## Error Path Tests
- test_broken_pipe_returns_error_not_panic
- test_io_error_returns_ui_error

## Edge Case Tests
- test_no_color_env_disables_ansi
- test_empty_string_handled
- test_multiline_text_handled
- test_unicode_text_handled

## Contract Verification Tests
- test_precondition_stdout_available
- test_postcondition_header_format
- test_postcondition_success_format
- test_postcondition_warning_format
- test_postcondition_error_format
- test_invariant_no_panic_on_io_error
- test_invariant_utf8_output

## Contract Violation Tests
- `test_broken_pipe_violation_returns_error`
  Given: stdout pipe is closed
  When: print_error("test") is called
  Then: Returns Err(UiError::BrokenPipe), NOT panic

- `test_no_color_violation_outputs_plain`
  Given: NO_COLOR=1 environment variable
  When: print_success("test") is called
  Then: Outputs "test" without ANSI codes, NOT green text

## Given-When-Then Scenarios
### Scenario 1: Success message
Given: A terminal supporting ANSI
When: I call print_success("Operation complete")
Then: Green text with checkmark is output

### Scenario 2: NO_COLOR mode
Given: NO_COLOR=1 environment variable set
When: I call print_error("Failed")
Then: Plain text "✗ Failed" without ANSI codes

### Scenario 3: List items
Given: A list of 3 items
When: I call print_list_item for each with indent 2
Then: Each item is indented with bullet point
