# Contract Specification: CLI UI Terminal Formatting Module

## Context
- Feature: Port cli_ui.gleam to Rust with 9 terminal output functions
- Domain terms: ANSI codes, colors, terminal output
- Assumptions: termcolor crate is acceptable
- Open questions: None

## Preconditions
- [P1] Terminal supports ANSI or NO_COLOR env is set
- [P2] stdout is available for writing

## Postconditions
- [Q1] print_header outputs bold text with separator lines
- [Q2] print_success outputs green text with checkmark
- [Q3] print_warning outputs yellow text with warning symbol
- [Q4] print_error outputs red text with X symbol
- [Q5] print_info outputs blue text with info symbol
- [Q6] print_labeled outputs "label: value" format
- [Q7] print_list_item outputs indented bullet point
- [Q8] print_colored applies custom color function
- [Q9] badge formats count with label and color
- [Q10] NO_COLOR=1 disables all ANSI output

## Invariants
- [I1] No function panics on I/O error
- [I2] All output is valid UTF-8
- [I3] Broken pipe is handled gracefully

## Error Taxonomy
- UiError::IoError - when stdout write fails
- UiError::BrokenPipe - when pipe is closed

## Contract Signatures
```rust
pub fn print_header(text: &str) -> Result<(), UiError>;
pub fn print_success(text: &str) -> Result<(), UiError>;
pub fn print_warning(text: &str) -> Result<(), UiError>;
pub fn print_error(text: &str) -> Result<(), UiError>;
pub fn print_info(text: &str) -> Result<(), UiError>;
pub fn print_labeled(label: &str, value: &str) -> Result<(), UiError>;
pub fn print_list_item(text: &str, indent: usize) -> Result<(), UiError>;
pub fn print_colored<F: Fn(&str) -> String>(text: &str, color_fn: F) -> Result<(), UiError>;
pub fn badge(label: &str, count: usize, color_fn: impl Fn(&str) -> String) -> String;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Terminal/NO_COLOR | Runtime | check NO_COLOR env |
| P2: stdout available | Result | std::io::Result |

## Violation Examples (REQUIRED)
- VIOLATES [I1]: Broken pipe causes panic -- WRONG, should return Err(UiError::BrokenPipe)
- VIOLATES [Q10]: NO_COLOR=1 still outputs ANSI -- WRONG, should output plain text

## Ownership Contracts
- All functions borrow &str, no ownership transfer
- badge returns owned String, caller owns result

## Non-goals
- [ ] Windows console API support (termcolor handles this)
- [ ] Custom color schemes beyond standard ANSI
