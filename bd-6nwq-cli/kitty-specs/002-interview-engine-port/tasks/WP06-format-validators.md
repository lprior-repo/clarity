---
work_package_id: WP06
title: Format Validators
lane: planned
dependencies: []
subtasks: [T026, T027, T028, T029, T030, T031]
---

# WP06: Format Validators

## Objective

Port `formats.gleam` (569 lines) with RFC-compliant email, UUID, URI, and ISO8601 validators.

## Context

- **Source**: `/tmp/intent-cli/src/intent/formats.gleam` (569 lines)
- **Target**: `clarity-web/src/intent/formats.rs`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement |
|----|--------------|-------------|
| P1 | Input is valid UTF-8 | Compile-time |
| P2 | Email contains exactly one @ | Runtime |
| P3 | UUID format matches 8-4-4-4-12 | Runtime |

### Postconditions

| ID | Postcondition | Verification |
|----|---------------|--------------|
| Q1 | Email validates RFC 5322 structure | Test vectors |
| Q2 | UUID version/variant bits correct | Bit checks |
| Q3 | URI has valid scheme + authority | Test vectors |
| Q4 | ISO8601 calendar date is valid | 2024-02-30 rejected |
| Q5 | Validators never panic | Fuzz testing |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | Empty string always fails validation |
| I2 | Validators return Result, never panic |
| I3 | Validation is deterministic |

### Error Taxonomy

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum FormatError {
    #[error("Invalid email: {reason}")]
    InvalidEmail { input: String, reason: String },

    #[error("Invalid UUID: {reason}")]
    InvalidUuid { input: String, reason: String },

    #[error("Invalid URI: {reason}")]
    InvalidUri { input: String, reason: String },

    #[error("Invalid ISO8601 datetime: {reason}")]
    InvalidIso8601 { input: String, reason: String },
}
```

### Violation Examples (REQUIRED)

```
VIOLATES P2: Email missing @ symbol
  -> FormatError::InvalidEmail { input: "no-at-sign", reason: "missing @" }

VIOLATES Q4: Invalid date 2024-02-30
  -> FormatError::InvalidIso8601 { input: "2024-02-30", reason: "February has max 29 days" }

VIOLATES Q2: UUID wrong format "12345"
  -> FormatError::InvalidUuid { input: "12345", reason: "expected 8-4-4-4-12 format" }
```

---

## Subtasks

### T026: Create `clarity-web/src/intent/formats.rs` module

### T027: Implement `validate_email()` with RFC 5322 compliant parsing

- Split on @ exactly once
- Validate local part (no consecutive dots, valid chars)
- Validate domain (has dot, valid labels)
- No regex - use pure parsing

### T028: Implement `validate_uuid()` with version/variant checking

- Format: 8-4-4-4-12 hex chars
- Version bits: char at position 14 must be 1-5
- Variant bits: char at position 19 must be 8,9,a,b

### T029: Implement `validate_uri()` with RFC 3986 scheme validation

- Must have scheme://authority
- Scheme: starts with letter, alphanumeric + - .
- Authority: non-empty after scheme

### T030: Implement `validate_iso8601()` with calendar validation

- Date: YYYY-MM-DD with valid month/day
- Time: HH:MM:SS with valid ranges
- Leap year calculation for Feb 29

### T031: Add helper functions: `is_valid_hex`, `is_leap_year`, `get_days_in_month`

---

## Test Strategy

### Test Vectors

```rust
#[test]
fn test_email_vectors() {
    let valid = vec!["user@example.com", "user+tag@example.co.uk"];
    let invalid = vec!["", "@", "user@", "@example.com", "user..dot@example.com"];

    for email in valid {
        assert!(validate_email(email).is_ok(), "Should accept: {}", email);
    }
    for email in invalid {
        assert!(validate_email(email).is_err(), "Should reject: {}", email);
    }
}

#[test]
fn test_uuid_vectors() {
    let valid = "550e8400-e29b-41d4-a716-446655440000";
    let invalid = vec!["", "not-a-uuid", "550e8400-e29b-41d4-a716"];

    assert!(validate_uuid(valid).is_ok());
    for uuid in invalid {
        assert!(validate_uuid(uuid).is_err(), "Should reject: {}", uuid);
    }
}

#[test]
fn test_iso8601_calendar() {
    // Leap year
    assert!(validate_iso8601("2024-02-29").is_ok());  // 2024 is leap year
    assert!(validate_iso8601("2023-02-29").is_err()); // 2023 is not

    // Invalid dates
    assert!(validate_iso8601("2024-02-30").is_err());
    assert!(validate_iso8601("2024-13-01").is_err());
    assert!(validate_iso8601("2024-00-15").is_err());
}
```

---

## Definition of Done

- [ ] All 4 validators implemented
- [ ] Test vectors pass
- [ ] No panics on any input
- [ ] Calendar edge cases handled
