pub(super) fn template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "data",
  "data_spec": {
    "data_model": {{data_model}},
    "access_patterns": "{{access_patterns}}",
    "retention": "{{retention}}"
  },
  "entities": [
    {
      "name": "{{entity_name}}",
      "description": "{{entity_description}}"
    }
  ],
  "queries": [
    {
      "name": "{{query_name}}",
      "description": "{{query_description}}"
    }
  ],
  "invariants": [],
  "ai_hints": {}
}
"#
  .to_string()
}
