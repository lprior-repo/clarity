// Intent Interview Schema
// Structured interrogation system for discovering and refining specifications
// Supports 5 rounds × 5+ perspectives = comprehensive requirement capture
package intent

// Interview session - persistent state machine
#InterviewSession: {
	// Session metadata
	id:              string            // Unique session ID (e.g., "interview-uuid-v4")
	profile:         #ProfileType      // Type of system being specified
	created_at:      string            // ISO 8601 timestamp
	updated_at:      string            // ISO 8601 timestamp
	completed_at:    string | *null    // When interview finished

	// Interview state
	stage:           #InterviewStage   // Current stage
	rounds_completed: int & >=0 & <=5  // How many rounds completed

	// Accumulated knowledge
	answers:         [...#Answer]       // All answers collected
	extracted_spec:  #PartialSpec      // Evolving spec being built
	gaps:            [...#Gap]          // Missing information
	conflicts:       [...#Conflict]     // Contradictions detected

	// Session metadata for storage
	raw_notes:       string | *""       // Freeform notes during session
}

// Interview stages
#InterviewStage: "discovery" | "refinement" | "validation" | "complete" | "paused"

// System profile types - determines which questions to ask
#ProfileType: "api" | "cli" | "event" | "data" | "workflow" | "ui"

// A single answer with metadata
#Answer: {
	question_id:   string              // Which question was asked
	question_text: string              // For context/audit trail
	perspective:   #Perspective        // Who answered (user/dev/ops/etc)
	round:         int & >=1 & <=5      // Which round (1-5)

	response:      string              // Raw human text answer
	extracted:     {...} | *null       // What we parsed from the answer
	confidence:    float & >=0 & <=1   // How confident are we in extraction (0-1)

	notes:         string | *""        // Follow-up clarifications
	timestamp:     string              // ISO 8601
}

// Perspective/role answering questions
#Perspective: "user" | "developer" | "ops" | "security" | "business"

// Partial spec built progressively
#PartialSpec: {
	// Basic metadata
	name?:           string
	description?:    string
	audience?:       string

	// Functional requirements
	features?:       [...#PartialFeature]
	behaviors?:      [...#PartialBehavior]

	// Constraints and rules
	constraints?:    [...#Constraint]
	rules?:          [...#Invariant]
	anti_patterns?:  [...#AntiPattern]

	// Security and compliance
	auth_method?:    string
	security_rules?: [...#SecurityRule]
	compliance?:     [...#ComplianceRequirement]

	// Non-functional requirements
	performance?:    #PerformanceRequirements
	scale?:          #ScaleRequirements
	availability?:   #AvailabilityRequirements

	// AI hints (filled during interview)
	ai_hints?:       #AIHints
}

#PartialFeature: {
	name?:        string
	description?: string
	behaviors?:   [...#PartialBehavior]
}

#PartialBehavior: {
	intent?:      string
	request?:     #PartialRequest
	response?:    #PartialResponse
	error_cases?: [...string]
}

#PartialRequest: {
	method?:      string
	path?:        string
	body?:        string
	headers?:     {...}
}

#PartialResponse: {
	status?:      int
	format?:      string
	fields?:      [...string]
}

#Constraint: {
	field:       string              // What field/behavior
	constraint:  string              // The constraint (e.g., "min_length=8")
	why:         string              // Explanation
	blocking:    bool | *false       // Blocks spec completion?
	source:      string | *"unknown" // Which question revealed this
}

#SecurityRule: {
	rule:        string
	why:         string
	sensitive_fields?: [...string]
	risk_level:  "critical" | "high" | "medium" | "low"
}

#ComplianceRequirement: {
	requirement: string
	standard:    string | *""        // GDPR, HIPAA, PCI, etc
	why:         string
}

#PerformanceRequirements: {
	target_latency_ms?:  int
	acceptable_delay_ms?: int
	throughput_rps?:     int
	percentile?:         int | *"p95"
}

#ScaleRequirements: {
	concurrent_users?:   int
	requests_per_second?: int
	data_retention_days?: int
	max_payload_size_kb?: int
}

#AvailabilityRequirements: {
	sla?:              string  // "99.9%"
	acceptable_downtime?: string
	geographic?:       string
}

// Gap - missing information blocking spec completion
#Gap: {
	id:           string
	field:        string              // What's missing (e.g., "auth_method")
	description:  string              // What we need
	blocking:     bool                // Prevents spec completion?
	suggested_default?: string         // Fallback suggestion
	why_needed:   string              // Explanation for user
	round:        int | *0            // Which round revealed this
	resolved:     bool | *false       // Was this answered?
	resolution:   string | *""        // How was it resolved
}

// Conflict - contradictions in requirements
#Conflict: {
	id:         string
	between:    [string, string]      // Which two answers/constraints conflict
	description: string               // What's the contradiction
	impact:     string                // Why it matters

	// Suggested resolutions for user to pick from
	options:    [...#ConflictResolution]
	chosen?:    int | *null           // Which option was chosen (index)
}

#ConflictResolution: {
	option:     string
	description: string
	tradeoffs:  string
	recommendation?: string  // "Recommended if..." or "Use when..."
}

// Question - used to drive interview
#Question: {
	id:           string

	// Question routing
	profile:      #ProfileType
	round:        int & >=1 & <=5
	perspective:  #Perspective
	category:     #QuestionCategory
	priority:     #QuestionPriority

	// Question content
	question:     string              // What to ask
	context:      string | *""        // Background/why asking
	example:      string | *""        // Example good answer

	// Processing
	expected_type?: string             // What type of answer (text, number, choice)
	extract_into?: string              // Which field(s) to extract into

	// Dependencies
	depends_on?:  [...string]         // Other question IDs that should come first
	blocks?:      [...string]         // Questions that depend on this answer

	// Follow-ups
	follow_ups?:  [...#FollowUp]
}

#QuestionCategory: "happy_path" | "error_case" | "edge_case" | "constraint" | "dependency" | "nonfunctional"
#QuestionPriority: "critical" | "important" | "nice_to_have"

#FollowUp: {
	condition:    string              // "if answer contains 'X'"
	question:     string
	questions?:   [...#Question]
}

// AI Hints embedded in spec/during interview
#AIHints: {
	implementation?: #ImplementationHints
	entities?:       [string]: #EntityHint
	security?:       #SecurityHints
	pitfalls?:       [...string]
	examples?:       [string]: string
}

#ImplementationHints: {
	suggested_stack?: [...string]
	architecture?:    string
	key_components?:  [...string]
}

#SecurityHints: {
	password_hashing?:  string
	jwt_algorithm?:     string
	jwt_expiry?:        string
	rate_limiting?:     string
	auth_method?:       string
}

#EntityHint: {
	fields?: [string]: string | #FieldHint
}

#FieldHint: {
	description?: string
	type?:        string
	validation?:  string
	example?:     string
	sensitive?:   bool | *false
}
