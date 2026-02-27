pub(super) fn template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "ui",
  "ui_spec": {
    "user_flows": {{user_flows}},
    "happy_path": "{{happy_path}}",
    "states": "{{states}}"
  },
  "components": [
    {
      "name": "{{component_name}}",
      "description": "{{component_description}}"
    }
  ],
  "screens": [{"name": "{{screen_name}}", "route": "{{route}}"}],
  "accessibility": {},
  "invariants": [],
  "ai_hints": {}
}
"#
  .to_string()
}
