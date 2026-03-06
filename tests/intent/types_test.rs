//! Integration tests for intent types

#[test]
fn test_intent_module_exists() {
    // Verify the intent module is accessible
    use clarity_web::intent;
    let _ = intent::types::Spec::default();
}
