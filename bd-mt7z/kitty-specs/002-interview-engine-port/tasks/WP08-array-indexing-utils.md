---
work_package_id: WP08
title: Array Indexing Utilities
lane: planned
dependencies: []
subtasks: [T036, T037, T038, T039, T040]
---

# WP08: Array Indexing Utilities

## Objective

Port `array_indexing.gleam` (287 lines) for JSON path navigation with array indexing support.

## Context

- **Source**: `/tmp/intent-cli/src/intent/array_indexing.gleam` (287 lines)
- **Target**: `clarity-web/src/intent/util/array_indexing.rs`
- **Priority**: P1 (High)

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement |
|----|--------------|-------------|
| P1 | JSON value is valid | Runtime (serde_json) |
| P2 | Index is non-negative for positive indexing | Runtime |
| P3 | Negative index is within bounds | Runtime |

### Postconditions

| ID | Postcondition | Verification |
|----|---------------|--------------|
| Q1 | Returns Err for out-of-bounds | Runtime |
| Q2 | -1 returns last element | Test |
| Q3 | Path "items[0].id" navigates correctly | Test |

### Error Taxonomy

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum IndexError {
    #[error("Path not found: {path}")]
    PathNotFound { path: String, available_keys: Vec<String> },

    #[error("Index out of bounds: {index} (array length: {len})")]
    OutOfBounds { index: isize, len: usize, path: String },

    #[error("Not an array: {path}")]
    NotAnArray { path: String, actual_type: String },

    #[error("Invalid path syntax: {path}")]
    InvalidPathSyntax { path: String, reason: String },
}
```

---

## Subtasks

### T036: Create `clarity-web/src/intent/util/array_indexing.rs`

### T037: Implement `ArrayIndexing` enum

```rust
/// Array indexing specification
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayIndexing {
    /// No array indexing
    NoArray,
    /// Positive index: `[0]`, `[1]`, ...
    Index(usize),
    /// Negative index from end: `[-1]` = last, `[-2]` = second to last
    LastN(usize),
    /// Wildcard: `[*]` (iterate all)
    All,
}
```

### T038: Implement `parse_path_component()` for `[index]` syntax

```rust
/// Parse path component like "items[0]" or "items[-1]"
///
/// # Examples
/// ```
/// let (field, index) = parse_path_component("items[0]")?;
/// assert_eq!(field, "items");
/// assert_eq!(index, ArrayIndexing::Index(0));
///
/// let (field, index) = parse_path_component("items[-1]")?;
/// assert_eq!(index, ArrayIndexing::LastN(1));
/// ```
pub fn parse_path_component(component: &str) -> Result<(String, ArrayIndexing), IndexError>
```

### T039: Implement `split_path()` for dot/bracket navigation

```rust
/// Split path like "response.body.items[0].id" into components
///
/// # Examples
/// ```
/// let parts = split_path("items[0].id")?;
/// assert_eq!(parts, vec!["items[0]", "id"]);
/// ```
pub fn split_path(path: &str) -> Result<Vec<String>, IndexError>
```

### T040: Implement `navigate_path()` for JSON traversal

```rust
/// Navigate JSON value using path with array indexing
///
/// # Examples
/// ```
/// let json = serde_json::json!({"items": [{"id": 1}, {"id": 2}]});
/// let result = navigate_path(&json, &["items[0]", "id"])?;
/// assert_eq!(result, serde_json::json!(1));
///
/// let last = navigate_path(&json, &["items[-1]", "id"])?;
/// assert_eq!(last, serde_json::json!(2));
/// ```
pub fn navigate_path(value: &serde_json::Value, path: &[String])
    -> Result<serde_json::Value, IndexError>
```

---

## Test Strategy

```rust
#[test]
fn test_positive_index() {
    let json = serde_json::json!([10, 20, 30]);
    let result = get_array_element(&json, 0).unwrap();
    assert_eq!(result, serde_json::json!(10));
}

#[test]
fn test_negative_index() {
    let json = serde_json::json!([10, 20, 30]);
    let result = get_array_element_last(&json, 1).unwrap(); // -1 = last
    assert_eq!(result, serde_json::json!(30));
}

#[test]
fn test_out_of_bounds() {
    let json = serde_json::json!([10, 20]);
    let result = get_array_element(&json, 5);
    assert!(matches!(result, Err(IndexError::OutOfBounds { .. })));
}

#[test]
fn test_nested_path() {
    let json = serde_json::json!({
        "response": {
            "body": {
                "items": [{"id": 1}, {"id": 2}]
            }
        }
    });
    let result = navigate_path(&json, &["response", "body", "items[0]", "id"]).unwrap();
    assert_eq!(result, serde_json::json!(1));
}
```

---

## Definition of Done

- [ ] ArrayIndexing enum defined
- [ ] parse_path_component() implemented
- [ ] split_path() implemented
- [ ] navigate_path() implemented
- [ ] All tests pass
