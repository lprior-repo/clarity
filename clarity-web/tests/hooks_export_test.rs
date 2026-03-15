#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

use clarity_web::hooks::{has_recoverable_session, use_progressive_discover_full};

#[test]
fn test_use_progressive_discover_full_is_exported() {
    let _fn_ptr: fn() -> _ = use_progressive_discover_full;
}

#[test]
fn test_has_recoverable_session_is_exported() {
    // This test verifies has_recoverable_session is exported and callable
    // On non-wasm32 targets, it should return false
    let result = has_recoverable_session();
    assert!(!result, "On non-wasm32 targets, has_recoverable_session should return false");
}
