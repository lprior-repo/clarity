pub(super) fn template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "event",
  "event_spec": {
    "event_type": "{{event_type}}",
    "payload_schema": {{payload_schema}},
    "trigger": "{{trigger}}"
  },
  "events": [
    {
      "type": "{{event_type}}",
      "description": "{{event_description}}",
      "source": "{{source}}"
    }
  ],
  "subscriptions": [{"topic": "{{topic}}", "handler": "{{handler}}"}],
  "invariants": [],
  "ai_hints": {}
}
"#
  .to_string()
}
