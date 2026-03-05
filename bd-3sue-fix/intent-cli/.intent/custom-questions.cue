// Custom Interview Questions for Intent CLI development
// This file demonstrates how to add/override interview questions
package intent

// Import the Question schema
#Question: {
	id:            string
	round:         1 | 2 | 3 | 4 | 5
	perspective:   "user" | "developer" | "ops" | "security" | "business"
	category:      "happy_path" | "error_case" | "edge_case" | "constraint" | "dependency" | "nonfunctional"
	priority:      "critical" | "important" | "nice_to_have"
	question:      string
	context:       string
	example:       string
	expected_type: string | *"text"
	extract_into:  [...string] | *[]
	depends_on:    [...string] | *[]
	blocks:        [...string] | *[]
}

custom_questions: {
	// Add a custom question for API profile
	api: {
		round_1: [
			{
				id:          "custom-api-deps"
				round:       1
				perspective: "developer"
				category:    "dependency"
				priority:    "important"
				question:    "What external services or APIs does this API depend on?"
				context:     "Understanding dependencies helps plan for failure modes"
				example:     "Auth service, payment gateway, database, cache"
				extract_into: ["dependencies"]
			},
		]
	}

	// Add custom questions for round 3 (common)
	common: {
		round_3: [
			{
				id:          "custom-scalability"
				round:       3
				perspective: "ops"
				category:    "nonfunctional"
				priority:    "important"
				question:    "What's your expected traffic pattern and scale requirements?"
				context:     "Custom question for our specific scaling needs"
				example:     "100 req/sec baseline, 10x spike during sales"
			},
		]
	}
}
