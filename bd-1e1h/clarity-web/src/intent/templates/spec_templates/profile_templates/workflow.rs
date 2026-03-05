pub(super) fn template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "workflow",
  "workflow_spec": {
    "steps": {{steps}},
    "happy_path": "{{happy_path}}",
    "error_recovery": "{{error_recovery}}"
  },
  "states": [],
  "transitions": [],
  "invariants": [],
  "ai_hints": {}
}
"#
  .to_string()
}
