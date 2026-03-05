# Contract Specification: Context Variables Type Fix

## Context
- Feature: Fix Context.variables to store Json values instead of strings
- Domain terms: Context, Variable, Value (serde_json::Value)
- Assumptions: Breaking change is acceptable for correctness
- Open questions: None

## Preconditions
- [P1] Context struct exists with variables field
- [P2] serde_json::Value type is available

## Postconditions
- [Q1] Context.variables is HashMap<String, Value>
- [Q2] set_variable accepts Value not String
- [Q3] get_variable returns Option<Value>
- [Q4] Array values can be stored and indexed
- [Q5] Object values can be stored and navigated

## Invariants
- [I1] Type information is never lost during storage
- [I2] All existing usages updated to handle Value type

## Error Taxonomy
- InterpolationError::InvalidPath - when path navigation fails
- InterpolationError::TypeMismatch - when expected type differs from actual
- InterpolationError::VariableNotFound - when key doesn't exist

## Contract Signatures
```rust
pub fn with_variable(mut self, key: impl Into<String>, value: Value) -> Self;
pub fn get_variable(&self, key: &str) -> Option<&Value>;
pub fn with_request_body(mut self, body: Value) -> Self;
pub fn with_response_body(mut self, body: Value) -> Self;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Context exists | Compile-time | struct definition |
| P2: Value available | Compile-time | use serde_json::Value |

## Violation Examples (REQUIRED)
- VIOLATES [I1]: Store array `[1,2,3]`, retrieve string "[1,2,3]" -- WRONG, should preserve Value::Array
- VIOLATES [Q4]: Store `Value::Array([1,2,3])`, call `ctx.get_variable("arr").unwrap()[0]` -- should work, not fail

## Ownership Contracts
- with_variable: Takes ownership of Value, caller gives up ownership
- get_variable: Returns reference, no ownership transfer
- Context is mutated in place via builder pattern

## Non-goals
- [ ] Backward compatibility with string-based API (breaking change acceptable)
