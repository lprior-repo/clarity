// Intent CLI Self-Validation Spec
// Uses KIRK to validate KIRK implementation (dogfooding)
package intent_kirk

import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Intent CLI KIRK Validation"

	description: """
		Self-validation spec for the KIRK (Knowledge-Informed Requirements & Kontract)
		system. This spec validates that KIRK features are correctly implemented
		by testing the Intent CLI against itself.
		"""

	audience: "Intent CLI developers and contributors"
	version:  "1.0.0"

	success_criteria: [
		"All KIRK CLI commands execute without errors",
		"Quality analysis produces valid scores",
		"Inversion analysis identifies missing failure cases",
		"Coverage analysis includes OWASP Top 10",
		"Compact format reduces token usage",
		"Protobuf text output is well-formed",
	]

	config: {
		base_url:   "http://localhost:8080"
		timeout_ms: 5000
	}

	features: [
		{
			name: "Quality Analysis"
			description: """
				The KIRK quality analyzer evaluates specs across 5 dimensions:
				completeness, consistency, testability, clarity, and security.
				"""

			behaviors: [
				{
					name:   "quality-command-runs"
					intent: "The quality command executes successfully on a valid spec"
					notes:  "Tests that quality_analyzer.gleam works correctly"

					request: {
						method: "GET"
						path:   "/quality"
					}

					response: {
						status: 200
						example: {
							completeness: 95.0
							consistency:  100.0
							testability:  100.0
							clarity:      85.0
							security:     70.0
							overall:      90.0
						}

						checks: {
							"completeness": {
								rule: "is number"
								why:  "Completeness is a percentage 0-100"
							}
							"overall": {
								rule: ">= 0"
								why:  "Overall score must be non-negative"
							}
						}
					}
				},
				{
					name:   "quality-json-output"
					intent: "Quality command supports --json flag for machine parsing"

					request: {
						method: "GET"
						path:   "/quality?format=json"
					}

					response: {
						status: 200

						checks: {
							"completeness": {rule: "is number"}
							"issues": {rule: "is array"}
						}
					}
				},
			]
		},
		{
			name: "Inversion Analysis"
			description: """
				The KIRK inversion checker applies Munger's inversion principle
				to identify missing failure cases in API specifications.
				"""

			behaviors: [
				{
					name:   "inversion-identifies-security-gaps"
					intent: "Inversion analysis detects missing security test cases"

					request: {
						method: "GET"
						path:   "/invert"
					}

					response: {
						status: 200
						example: {
							score:         60.0
							security_gaps: [{description: "Missing auth bypass test"}]
						}

						checks: {
							"score": {
								rule: "is number"
								why:  "Score indicates percentage of inversions covered"
							}
							"security_gaps": {
								rule: "is array"
								why:  "Lists specific security inversions missing"
							}
						}
					}
				},
				{
					name:   "inversion-suggests-behaviors"
					intent: "Inversion analysis suggests specific behaviors to add"

					request: {
						method: "GET"
						path:   "/invert"
					}

					response: {
						status: 200

						checks: {
							"suggested_behaviors": {
								rule: "is array"
								why:  "Concrete suggestions for improvement"
							}
						}
					}
				},
			]
		},
		{
			name: "Coverage Analysis"
			description: """
				The KIRK coverage analyzer measures test coverage including
				HTTP methods, status codes, edge cases, and OWASP Top 10.
				"""

			behaviors: [
				{
					name:   "coverage-includes-owasp"
					intent: "Coverage analysis includes OWASP Top 10 security categories"

					request: {
						method: "GET"
						path:   "/coverage"
					}

					response: {
						status: 200
						example: {
							owasp: {
								score:   70.0
								missing: ["A03: Injection"]
							}
						}

						checks: {
							"owasp.score": {
								rule: "is number"
								why:  "OWASP coverage percentage 0-100"
							}
							"owasp.missing": {
								rule: "is array"
								why:  "Lists uncovered OWASP categories"
							}
						}
					}
				},
				{
					name:   "coverage-tracks-methods"
					intent: "Coverage analysis counts HTTP methods used"

					request: {
						method: "GET"
						path:   "/coverage"
					}

					response: {
						status: 200

						checks: {
							"methods": {
								rule: "is object"
								why:  "Map of method -> count"
							}
							"status_codes": {
								rule: "is object"
								why:  "Map of status category -> count"
							}
						}
					}
				},
			]
		},
		{
			name: "Gap Detection"
			description: """
				The KIRK gap detector identifies missing requirements using
				multiple mental models: inversion, second-order thinking, checklists.
				"""

			behaviors: [
				{
					name:   "gaps-uses-mental-models"
					intent: "Gap detection applies multiple mental models"

					request: {
						method: "GET"
						path:   "/gaps"
					}

					response: {
						status: 200
						example: {
							total_gaps: 5
							severity_breakdown: {
								critical: 0
								high:     2
								medium:   2
								low:      1
							}
						}

						checks: {
							"total_gaps": {
								rule: "is integer"
								why:  "Count of all detected gaps"
							}
							"severity_breakdown": {
								rule: "is object"
								why:  "Breakdown by severity level"
							}
						}
					}
				},
				{
					name:   "gaps-provides-suggestions"
					intent: "Each gap includes a concrete suggestion for resolution"

					request: {
						method: "GET"
						path:   "/gaps"
					}

					response: {
						status: 200

						checks: {
							"inversion_gaps": {rule: "is array"}
							"security_gaps": {rule: "is array"}
						}
					}
				},
			]
		},
		{
			name: "Compact Format"
			description: """
				The KIRK compact format (CIN) reduces token usage for AI prompts
				by approximately 50% compared to full JSON/CUE.
				"""

			behaviors: [
				{
					name:   "compact-reduces-tokens"
					intent: "Compact format achieves significant token reduction"

					request: {
						method: "GET"
						path:   "/compact"
					}

					response: {
						status: 200

						checks: {
							"token_savings": {
								rule: ">= 30"
								why:  "Should save at least 30% tokens"
							}
						}
					}
				},
			]
		},
		{
			name: "Protobuf Text Output"
			description: """
				KIRK can export specs to protobuf text format for
				language-agnostic processing and validation.
				"""

			behaviors: [
				{
					name:   "prototext-valid-format"
					intent: "Protobuf text output is syntactically valid"

					request: {
						method: "GET"
						path:   "/prototext"
					}

					response: {
						status: 200

						checks: {
							"body": {
								rule: "non-empty string"
								why:  "Should produce output"
							}
						}
					}
				},
			]
		},
	]

	rules: [
		{
			name:        "all-commands-return-valid-json"
			description: "All KIRK commands with --json flag return valid JSON"

			when: {
				status: ">= 200"
				path:   ".*json.*"
			}

			check: {
				body_must_contain: ["{"]
			}
		},
		{
			name:        "errors-are-structured"
			description: "All error responses have structured format"

			when: {
				status: ">= 400"
			}

			check: {
				fields_must_exist: ["error"]
			}
		},
	]

	anti_patterns: [
		{
			name:        "missing-mental-model"
			description: "Gap suggestions should reference the mental model used"

			bad_example: {
				description: "Missing authentication test"
				suggestion:  "Add auth test"
			}

			good_example: {
				description:  "Missing authentication test"
				suggestion:   "Add auth test"
				mental_model: "Inversion"
			}

			why: "Mental model attribution helps users understand the reasoning"
		},
		{
			name:        "hardcoded-thresholds"
			description: "Quality scores should be configurable, not hardcoded"

			bad_example: {
				score: 80
				pass:  true
			}

			good_example: {
				score:     80
				threshold: 70
				pass:      true
			}

			why: "Different projects have different quality requirements"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Gleam", "Erlang", "CUE"]
		}

		entities: {
			QualityReport: {
				fields: {
					completeness: "float, percentage 0-100"
					consistency:  "float, percentage 0-100"
					testability:  "float, percentage 0-100"
					clarity:      "float, percentage 0-100"
					security:     "float, percentage 0-100"
					overall:      "float, weighted average"
					issues:       "list of QualityIssue"
				}
			}
			Gap: {
				fields: {
					type:         "enum: inversion, second_order, checklist, coverage, security"
					description:  "string, what's missing"
					severity:     "enum: low, medium, high, critical"
					suggestion:   "string, how to fix"
					mental_model: "string, which model identified this"
				}
			}
		}

		security: {
			password_hashing: "Not applicable - no user passwords"
			jwt_algorithm:    "Not applicable - no authentication"
			jwt_expiry:       "Not applicable"
			rate_limiting:    "CLI tool - no rate limiting"
		}

		pitfalls: [
			"Don't assume all specs have all sections filled",
			"Handle empty behaviors lists gracefully",
			"Float comparisons need epsilon tolerance",
			"JSON output must be valid even for empty results",
		]
	}
}
