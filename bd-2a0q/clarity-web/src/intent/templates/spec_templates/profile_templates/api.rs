pub(super) fn template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "api",
  "api_spec": {
    "base_url": "{{base_url}}",
    "auth_method": "{{auth_method}}",
    "response_format": "{{response_format}}",
    "versioning": "{{versioning}}"
  },
  "endpoints": [
    {
      "path": "{{endpoint_path}}",
      "method": "{{http_method}}",
      "description": "{{endpoint_description}}"
    }
  ],
  "error_cases": [],
  "happy_path": "{{happy_path}}",
  "invariants": [],
  "ai_hints": {}
}
"#
  .to_string()
}
