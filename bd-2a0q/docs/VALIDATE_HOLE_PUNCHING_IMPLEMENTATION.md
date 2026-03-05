# Implementation: validate_hole_punching_server

## Overview
Successfully implemented `validate_hole_punching_server` function as per bead bd-13yb.

## Location
- **File**: `/home/lewis/src/clarity/clarity-web/src/server.rs`
- **Function**: `validate_hole_punching_server` (lines 786-990)
- **Tests**: Lines 1315-1410

## Function Signature

```rust
#[server]
pub async fn validate_hole_punching_server(
    scenario: ScenarioField,
    session_id: Option<String>,
) -> Result<HolePunchingResults, ServerFnError>
```

## Implementation Details

### 1. Input Parameters
- `scenario: ScenarioField` - Contains trigger, value_moment, feeling, and hole_punching state
- `session_id: Option<String>` - Optional session ID for rate limiting

### 2. Return Type
- `Result<HolePunchingResults, ServerFnError>`
- `HolePunchingResults` contains:
  - `discovery_hole: Option<String>` - How did they find the feature?
  - `edge_case_hole: Option<String>` - What if internet drops, typos, etc?
  - `motivation_dropoff: Option<String>` - Why continue at high-friction steps?

### 3. Features Implemented

#### Rate Limiting
- Uses existing `RATE_LIMITER` singleton (10 requests/minute)
- Returns error with retry-after duration when rate limited
- Logs rate limit events

#### Input Validation
- Validates all three bullet fields are complete (trigger, value_moment, feeling)
- Returns clear error message if validation fails

#### AI-Powered Hole Detection
- Defines schema with 5 fields:
  1. `discovery_hole_addressed` (boolean)
  2. `edge_case_hole_addressed` (boolean)
  3. `motivation_dropoff_addressed` (boolean)
  4. `identified_holes` (text, optional)
  5. `suggestions` (text, optional)

- Builds detailed analysis prompt including:
  - All three scenario fields
  - Current hole punching status
  - Clear description of each hole type
  - Instructions for evaluation

- Calls `AI_PROVIDER.extract_fields_with_schema()` for analysis

#### Result Processing
- Parses AI response for all 3 hole types
- Marks holes as addressed if AI returns `true`
- Preserves existing hole explanations if already set
- Defaults to "Addressed in scenario" for newly addressed holes

#### Logging
- Logs API calls with input lengths
- Logs rate limit events
- Logs completion with:
  - `is_complete` status
  - `addressed_count`
  - `unaddressed` holes list

### 4. Tests Added

All tests added to `integration_tests` module:

1. **test_hole_punching_results_serialization**
   - Verifies `HolePunchingResults` serializes/deserializes correctly

2. **test_scenario_field_serialization**
   - Verifies `ScenarioField` serializes/deserializes correctly

3. **test_hole_punching_results_is_complete**
   - Tests `is_complete()` method
   - Tests `addressed_count()` method
   - Verifies empty string normalization

4. **test_hole_punching_results_unaddressed_holes**
   - Tests `unaddressed_holes()` returns correct hole types

5. **test_hole_punching_results_from_strings**
   - Tests `from_strings()` normalizes empty strings to None

6. **test_scenario_field_validation_helpers**
   - Tests `is_bullets_complete()`
   - Tests individual field empty checks
   - Verifies whitespace handling

## Integration Points

### Imports Added
```rust
use crate::components::discover::types::{HolePunchingResults, ScenarioField};
```

### Type Dependencies
- `HolePunchingResults` from `clarity-web/src/components/discover/types.rs`
- `ScenarioField` from `clarity-web/src/components/discover/types.rs`
- `ServerFnError` from `dioxus_fullstack`
- `ExtractionContext` and `FieldType` from `crate::providers`
- `SchemaField` from `crate::providers`

### Infrastructure Used
- `RATE_LIMITER` - Global rate limiter singleton
- `AI_PROVIDER` - Global OpenCode AI provider singleton
- `tracing::info` and `tracing::warn` - Structured logging

## Verification

Run the verification script:
```bash
cd /home/lewis/src/clarity
rustc test_hole_punching.rs -o test_hole_punching
./test_hole_punching
```

All checks pass:
- ✅ Function signature correct
- ✅ Parameters correct
- ✅ Return type correct
- ✅ Rate limiting implemented
- ✅ Input validation implemented
- ✅ AI schema defined for all 3 hole types
- ✅ AI provider called
- ✅ Results parsed correctly
- ✅ Logging present
- ✅ All 6 unit tests added

## Usage Example

```rust
use crate::components::discover::types::ScenarioField;
use crate::server::validate_hole_punching_server;

let scenario = ScenarioField {
    trigger: "User encounters error message".to_string(),
    value_moment: "Instant problem resolution".to_string(),
    feeling: "Relieved and confident".to_string(),
    hole_punching: HolePunchingResults::default(),
};

let results = validate_hole_punching_server(
    scenario,
    Some("session-123".to_string())
).await?;

if !results.is_complete() {
    println!("Missing holes: {:?}", results.unaddressed_holes());
}
```

## Error Handling

Function returns `ServerFnError` for:
- Rate limit exceeded (with retry-after duration)
- Empty or incomplete scenario input
- AI provider errors (rate limited, auth, invalid input, generic)
- Extraction errors

All error paths are covered with specific error messages.

## Documentation

Function includes comprehensive documentation:
- Overview of the three hole types
- Detailed parameter descriptions
- Return type documentation
- Usage example
- Error handling notes

## Next Steps

The implementation is complete and ready for:
1. Client-side integration via Dioxus fullstack
2. E2E testing with real AI provider
3. UI integration in the discover flow component
