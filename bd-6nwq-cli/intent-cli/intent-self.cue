package api

spec: {
	name:        "Intent CLI"
	description: "Contract-driven API testing CLI - pushes spec-driven development via structured CUE schemas and interview integrations"
	audience:    "Developers building APIs who want contract-driven testing to ensure implementations match specifications"
	version:     "2.0.0"

	success_criteria: [
		"All CLI commands execute without errors",
		"Specs validate correctly against schema",
		"Interview system guides users through all 5 rounds",
		"Generated specs are syntactically valid CUE",
		"Check command executes behaviors in correct dependency order",
		"Response validation catches mismatches",
		"Exit codes match specification (0=pass, 1=fail, 2=blocked, 3=invalid, 4=error)",
	]

	config: {
		base_url:   ""
		timeout_ms: 5000
		headers:    {}
	}

	features: [
		{
			name:        "Schema Validation"
			description: "Validate CUE specs against Intent schema"
			behaviors: [
				{
					name:   "validate_valid_spec"
					intent: "Validate a syntactically correct spec file"
					request: {
						method: "GET"
						path:   "/validate/valid"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks:  {}
					}
					notes:    "Returns 200 when spec is valid"
					requires: []
					tags:     ["validation", "happy-path"]
					captures: {}
				},
				{
					name:   "validate_invalid_spec"
					intent: "Reject specs with schema violations"
					request: {
						method: "GET"
						path:   "/validate/invalid"
						headers: {}
						query:   {}
					}
					response: {
						status:  400
						example: null
						checks:  {}
					}
					notes:    "Returns 400 when spec has validation errors"
					requires: []
					tags:     ["validation", "error-case"]
					captures: {}
				},
			]
		},
		{
			name:        "Specification Parsing"
			description: "Parse and transform CUE specifications"
			behaviors: [
				{
					name:   "parse_spec_to_json"
					intent: "Convert CUE spec to JSON for processing"
					request: {
						method: "GET"
						path:   "/spec/parse"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks: {
							"format": {
								rule: "is valid JSON"
								why:  "Specs must be parseable for validation and execution"
							}
						}
					}
					notes:    "Exports spec to JSON representation"
					requires: []
					tags:     ["parsing", "export"]
					captures: {}
				},
			]
		},
		{
			name:        "Spec Execution"
			description: "Execute behaviors defined in specs against target APIs"
			behaviors: [
				{
					name:   "execute_behaviors"
					intent: "Run all behaviors from a valid spec against target API"
					request: {
						method: "POST"
						path:   "/execute"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks: {
							"results": {
								rule: "not empty"
								why:  "Should execute and return results for each behavior"
							}
						}
					}
					notes:    "Runs behaviors in dependency order"
					requires: []
					tags:     ["execution", "happy-path"]
					captures: {}
				},
				{
					name:   "execute_with_unreachable_target"
					intent: "Handle gracefully when target API is unreachable"
					request: {
						method: "POST"
						path:   "/execute/unreachable"
						headers: {}
						query:   {}
					}
					response: {
						status:  503
						example: null
						checks: {
							"error": {
								rule: "contains network error"
								why:  "Should report connection failures clearly"
							}
						}
					}
					notes:    "Timeout or connection errors should fail gracefully"
					requires: []
					tags:     ["execution", "error-handling"]
					captures: {}
				},
				{
					name:   "execute_with_filtering"
					intent: "Execute only specified features or behaviors"
					request: {
						method: "POST"
						path:   "/execute/filtered"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks:  {}
					}
					notes:    "Supports feature and behavior filtering"
					requires: ["execute_behaviors"]
					tags:     ["execution", "filtering"]
					captures: {}
				},
			]
		},
		{
			name:        "Output Formatting"
			description: "Display and export results in multiple formats"
			behaviors: [
				{
					name:   "output_human_readable"
					intent: "Display results in human-friendly format"
					request: {
						method: "GET"
						path:   "/output/text"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks:  {}
					}
					notes:    "Pretty-printed text output for CLI"
					requires: []
					tags:     ["output", "formatting"]
					captures: {}
				},
				{
					name:   "output_json"
					intent: "Export results as JSON for tooling"
					request: {
						method: "GET"
						path:   "/output/json"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks: {
							"content_type": {
								rule: "is application/json"
								why:  "JSON output enables integration with other tools"
							}
						}
					}
					notes:    "Structured JSON for programmatic access"
					requires: ["output_human_readable"]
					tags:     ["output", "json"]
					captures: {}
				},
			]
		},
		{
			name:        "Quality Analysis"
			description: "Assess and improve specification quality"
			behaviors: [
				{
					name:   "quality_score"
					intent: "Calculate comprehensive quality score for a spec"
					request: {
						method: "GET"
						path:   "/quality/score"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks: {
							"score": {
								rule: "is between 0 and 100"
								why:  "Quality metrics must be meaningful and quantifiable"
							}
						}
					}
					notes:    "Scores coverage, clarity, testability, AI readiness"
					requires: []
					tags:     ["quality", "metrics"]
					captures: {}
				},
				{
					name:   "quality_lint"
					intent: "Identify quality issues and anti-patterns"
					request: {
						method: "GET"
						path:   "/quality/lint"
						headers: {}
						query:   {}
					}
					response: {
						status:  200
						example: null
						checks:  {}
					}
					notes:    "Detects naming, documentation, and structure issues"
					requires: []
					tags:     ["quality", "linting"]
					captures: {}
				},
			]
		},
	]

	rules: [
		{
			name:        "Valid Exit Codes"
			description: "All commands must return appropriate exit codes"
			when: null
			check: {
				body_must_not_contain: [
					"panic",
					"segmentation fault",
				]
			}
			example: null
		},
		{
			name:        "Error Message Quality"
			description: "Error messages should be user-friendly without exposing internals"
			when: null
			check: {
				body_must_not_contain: [
					"stack trace",
					"memory address",
				]
			}
			example: null
		},
		{
			name:        "Spec Completeness"
			description: "All specs must have required fields populated"
			when: null
			check: {
				fields_must_exist: [
					"name",
					"description",
					"version",
					"features",
				]
			}
			example: null
		},
	]

	anti_patterns: [
		{
			name:        "Vague Behavior Intent"
			description: "Intent should clearly describe what is being tested"
			bad_example: {
				intent: "test the endpoint"
			}
			good_example: {
				intent: "Verify POST /users returns 201 with valid user ID when given valid email and password"
			}
			why: "Vague intents make AI implementation and human review harder"
		},
		{
			name:        "Missing Error Cases"
			description: "Specs must include error scenarios, not just happy paths"
			bad_example: {
				behaviors: [
					{
						name:   "create_user"
						intent: "Create a user"
					},
				]
			}
			good_example: {
				behaviors: [
					{
						name:   "create_user_valid"
						intent: "Create user with valid input"
					},
					{
						name:   "create_user_duplicate"
						intent: "Reject when email already exists"
					},
					{
						name:   "create_user_invalid_format"
						intent: "Reject invalid email format"
					},
				]
			}
			why: "Comprehensive error handling is essential for production APIs"
		},
		{
			name:        "Unexplained Checks"
			description: "Every check must explain why it matters"
			bad_example: {
				checks: {
					"user.id": {
						rule: "equals uuid"
						why:  ""
					}
				}
			}
			good_example: {
				checks: {
					"user.id": {
						rule: "equals uuid"
						why:  "UUIDs prevent enumeration attacks and ensure global uniqueness"
					}
				}
			}
			why: "The 'why' helps AI understand intent and helps reviewers validate correctness"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: [
				"Gleam (primary language)",
				"Erlang/BEAM VM (runtime)",
				"CUE (specification language)",
			]
		}

		entities: {
			spec: {
				fields: {
					name:           "string, unique spec identifier"
					description:    "string, purpose and scope"
					version:        "semver string for tracking changes"
					features:       "list of feature groups"
					behaviors:      "list of executable test cases"
					rules:          "global validation rules"
					ai_hints:       "implementation guidance"
				}
			}

			behavior: {
				fields: {
					name:     "string, behavior identifier"
					intent:   "string, what this behavior validates"
					request:  "HTTP request definition"
					response: "expected response"
					requires: "dependencies on other behaviors"
					captures: "output values to extract"
				}
			}

			check: {
				fields: {
					rule: "string, validation rule expression"
					why:  "string, importance explanation"
				}
			}
		}

		security: {
			password_hashing:  "N/A - CLI development tool"
			jwt_algorithm:     "N/A - no authentication"
			jwt_expiry:        "N/A - no sessions"
			rate_limiting:     "N/A - local single-user tool"
		}

		pitfalls: [
			"Don't assume specs are always valid - validate thoroughly",
			"Don't expose CUE internals to users - provide clear errors",
			"Don't auto-correct specs - report issues and let users fix them",
			"Don't forget file handling edge cases",
			"Don't assume APIs are reachable - handle network failures",
			"Don't implement unbounded HTTP timeouts",
			"Don't leak CLI internals in error output",
			"Don't forget to document exit codes for scripting",
		]
	}
}
