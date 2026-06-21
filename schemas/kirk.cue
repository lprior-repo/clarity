// KIRK: Knowledge-Informed Requirements & Kontract
// Extended CUE Schema - Source of Truth
// Adds mental models, quality metrics, and AI optimization
package intent

// =============================================================================
// KIRK EXTENSIONS TO SPEC
// =============================================================================

// Minimal legacy anchors retained so old KIRK extensions evaluate while the
// normative #KirkContract16 below becomes the Clarity source-of-truth artifact.
#Spec: {
	name?: string
	version?: string
	behaviors?: [...#Behavior]
	ai_hints?: #AIHints
	...
}

#Behavior: {
	name?: string
	intent?: string
	...
}

#AIHints: {
	do?: [...string]
	do_not?: [...string]
	...
}

// Extended Spec with KIRK mental model fields
#KirkSpec: #Spec & {
	// Mental Model: Inversion
	// "What would make this fail?"
	inversions?: #Inversions

	// Mental Model: Pre-Mortem
	// "Imagine it failed - why?"
	pre_mortem?: #PreMortem

	// Quality Metrics (computed)
	quality_score?: #QualityScore

	// Coverage Analysis (computed)
	coverage?: #Coverage

	// Detected Gaps (computed)
	gaps?: [...#Gap]

	// Detected Conflicts (computed)
	conflicts?: [...#Conflict]
}

// =============================================================================
// CLARITY KIRK16 CONTRACT - MASTER_DOC.md SECTION 16
// =============================================================================

#KirkContract16: {
	schema_version: string
	schema_hash: string & =~"^sha256:[a-f0-9]{64}$"
	session_id: string
	profile: "rust-cli" | "rust-library" | "rust-web-service" | "rust-async-service" | "rust-storage" | "rust-ui" | "rust-refactor"
	sections: [
		#KirkSection0,
		#KirkSection1,
		#KirkSection2,
		#KirkSection3,
		#KirkSection4,
		#KirkSection5,
		#KirkSection6,
		#KirkSection7,
		#KirkSection8,
		#KirkSection9,
		#KirkSection10,
		#KirkSection11,
		#KirkSection12,
		#KirkSection13,
		#KirkSection14,
		#KirkSection15,
	]
}

#KirkSection: {
	index: int & >=0 & <=15
	title: string
	content: string
	evidence_event_ids: [...string] & [_, ...]
	...
}

#KirkSection0: #KirkSection & {index: 0, title: "Original Prompt"}
#KirkSection1: #KirkSection & {index: 1, title: "Problem Statement"}
#KirkSection2: #KirkSection & {index: 2, title: "Antithesis Points"}
#KirkSection3: #KirkSection & {index: 3, title: "Target Persona"}
#KirkSection4: #KirkSection & {index: 4, title: "Straw Man Validation"}
#KirkSection5: #KirkSection & {index: 5, title: "Solution Description"}
#KirkSection6: #KirkSection & {index: 6, title: "VORP Justification"}
#KirkSection7: #KirkSection & {index: 7, title: "Non-Persona"}
#KirkSection8: #KirkSection & {index: 8, title: "Scenario Trigger"}
#KirkSection9: #KirkSection & {index: 9, title: "Scenario Value Moment"}
#KirkSection10: #KirkSection & {index: 10, title: "Scenario Feeling"}
#KirkSection11: #KirkSection & {index: 11, title: "Discovery Hole"}
#KirkSection12: #KirkSection & {index: 12, title: "Edge Case Hole"}
#KirkSection13: #KirkSection & {index: 13, title: "Motivation Drop-off"}
#KirkSection14: #KirkSection & {index: 14, title: "EARS Requirements"}
#KirkSection15: #KirkSection & {
	index: 15
	title: "Compilation Metadata"
	metadata: #KirkMetadata
}

#KirkMetadata: {
	generated_at: string
	event_log_last_seq: int & >=0
	event_log_hash: string & =~"^sha256:[a-f0-9]{64}$"
	reviewer_panel_results: [...{
		reviewer: "sre" | "architect" | "wardley" | "munger" | "security" | "test"
		verdict: "Pass" | "Fail"
		evidence_event_ids: [...string] & [_, ...]
	}]
	gate_results: [...{
		gate_id: string
		passed: bool
		evidence_event_ids: [...string]
	}]
}

// =============================================================================
// EXTENDED BEHAVIOR
// =============================================================================

#KirkBehavior: #Behavior & {
	// Design by Contract: Preconditions
	preconditions?: #Preconditions

	// Design by Contract: Postconditions
	postconditions?: #Postconditions

	// Second-Order Thinking
	// "What happens after this action?"
	second_order_effects?: [...string]

	// INVEST Validation (computed)
	invest_valid?: bool
}

// =============================================================================
// MENTAL MODEL: INVERSION
// =============================================================================

#Inversions: {
	// What security failures could occur?
	security_failures?: [...string]

	// What usability failures could occur?
	usability_failures?: [...string]

	// What integration failures could occur?
	integration_failures?: [...string]
}

// =============================================================================
// MENTAL MODEL: PRE-MORTEM
// =============================================================================

#PreMortem: {
	// "The project failed. What happened?"
	assumed_failure: string

	// List of likely causes
	likely_causes: [...#LikelyCause]
}

#LikelyCause: {
	cause:       string
	probability: "low" | "medium" | "high"
	mitigation:  string
}

// =============================================================================
// DESIGN BY CONTRACT
// =============================================================================

#Preconditions: {
	// Is authentication required?
	auth_required?: bool

	// Which fields must be provided?
	required_fields?: [...string]

	// Field-level constraints
	field_constraints?: [string]: string
}

#Postconditions: {
	// What state changes occur?
	state_changes?: [...string]

	// What response guarantees exist?
	response_guarantees?: [string]: string
}

// =============================================================================
// QUALITY METRICS
// =============================================================================

#QualityScore: {
	completeness: float & >=0 & <=100
	consistency:  float & >=0 & <=100
	testability:  float & >=0 & <=100
	clarity:      float & >=0 & <=100
	security:     float & >=0 & <=100
	overall:      float & >=0 & <=100
	issues?: [...#QualityIssue]
}

#QualityIssue: {
	field:    string
	issue:    string
	severity: #Severity
}

#Severity: "info" | "warning" | "error" | "critical"

// =============================================================================
// COVERAGE ANALYSIS
// =============================================================================

#Coverage: {
	methods: [string]:      int
	status_codes: [string]: int
	paths: [string]:        [...string]
	edge_cases?:            #EdgeCaseCoverage
	owasp?:                 #OWASPCoverage
}

#EdgeCaseCoverage: {
	tested:    [...string]
	suggested: [...string]
}

#OWASPCoverage: {
	categories: [string]: bool
	score: float & >=0 & <=100
	missing?: [...string]
}

// =============================================================================
// GAP DETECTION
// =============================================================================

#Gap: {
	type:         #GapType
	description:  string
	severity:     #Severity
	suggestion:   string
	mental_model: string
}

#GapType: "inversion" | "second_order" | "checklist" | "coverage" | "security"

// =============================================================================
// CONFLICT DETECTION
// =============================================================================

#Conflict: {
	type:               #ConflictType
	first:              string
	second:             string
	resolution_options: [...string]
	chosen_resolution?: int
}

#ConflictType: "cap_theorem" | "scope_paradox" | "security_usability" | "performance_consistency"

// =============================================================================
// ENHANCED AI HINTS
// =============================================================================

#KirkAIHints: #AIHints & {
	// Explicit code patterns
	code_patterns?: {
		error_handling?:  string
		auth_middleware?: string
		validation?:      string
	}

	// Type definitions by language
	type_definitions?: {
		typescript?: string
		golang?:     string
		python?:     string
	}

	// Database schema by engine
	database_schema?: {
		postgresql?: string
		mysql?:      string
		sqlite?:     string
	}
}

// =============================================================================
// COMPACT INTENT NOTATION (CIN)
// Token-efficient format for AI prompts
// =============================================================================

#CompactSpec: {
	header:   string // "SPEC name version"
	features: [...#CompactFeature]
}

#CompactFeature: {
	name:      string
	behaviors: [...#CompactBehavior]
}

#CompactBehavior: {
	name:     string
	intent:   string
	requires: [...string] // Prefixed with "<-"
	request:  string      // "METHOD /path {body}"
	status:   int
	checks: [...#CompactCheck]
	captures: [...string] // ">> var: path"
}

#CompactCheck: {
	field: string
	rule:  string
	why?:  string
}
