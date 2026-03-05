pub(super) fn template() -> String {
  r#"{
  "name": "{{spec_name}}",
  "description": "{{description}}",
  "profile": "cli",
  "cli_spec": {
    "command_name": "{{command_name}}",
    "version": "{{version}}",
    "help_text": "{{help_text}}"
  },
  "commands": [
    {
      "name": "{{command}}",
      "description": "{{command_description}}"
    }
  ],
  "exit_codes": [],
  "happy_path": "{{happy_path}}",
  "invariants": [],
  "ai_hints": {}
}
"#
  .to_string()
}
