// Enhanced Bead Schema - World-Class Ticket Template (v2)
// Incorporates 2025-2026 research on AI agent best practices:
// - Anthropic Claude 4.x Best Practices
// - GitHub Spec-Driven Development (spec-kit)
// - Martin Fowler's SDD Analysis
// - ThoughtWorks Engineering Practices
//
// This schema enforces the complete 16-section structure for one-shot AI implementation

package schema

// ============================================================================
// Core Bead Structure
// ============================================================================

#EnhancedBead: {
	// Identity
	id:              string & =~"^intent-cli-[a-z0-9]+$"
	title:           string & =~"^[A-Za-z-]+: .+"  // Must be "Component: Description"
	type:            #IssueType
	priority:        #Priority
	effort_estimate: #EffortEstimate
	labels:          [...string]

	// The 16 Required Sections (with .5 sections for extended coverage)
	clarifications:       #Clarifications          // Section 0
	ears_requirements:    #EarsRequirements        // Section 1
	contracts:            #KirkContracts           // Section 2
	research_requirements: #ResearchRequirements   // Section 2.5
	inversions:           #InversionAnalysis       // Section 3
	acceptance_tests:     #AcceptanceTests         // Section 4
	e2e_tests:            #E2ETests                // Section 5
	verification_checkpoints: #VerificationCheckpoints  // Section 5.5
	implementation_tasks: #ImplementationTasks     // Section 6
	failure_modes:        #FailureModes            // Section 7
	anti_hallucination:   #AntiHallucination       // Section 7.5
	context_survival:     #ContextSurvival         // Section 7.6
	completion_checklist: #CompletionChecklist     // Section 8
	context:              #Context                 // Section 9
	ai_hints:             #AIHints                 // Section 10
}

#IssueType: "feature" | "bug" | "task" | "epic" | "chore"

#Priority: 0 | 1 | 2 | 3 | 4  // 0=critical, 4=backlog

#EffortEstimate: "15min" | "30min" | "1hr" | "2hr" | "4hr"

// ============================================================================
// Section 0: Clarifications (Anti-Assumption Gate)
// ============================================================================
// From GitHub Spec Kit: Force explicit questions instead of assumptions

#Clarifications: {
	clarification_status: "RESOLVED" | "HAS_OPEN_QUESTIONS"

	resolved_clarifications?: [...{
		question:   string
		answer:     string
		decided_by: string
		date:       string
	}]

	open_clarifications?: [...{
		question:             string & =~"^\\[NEEDS CLARIFICATION: .+\\]$"
		context:              string
		options:              [...string]
		default_if_unresolved: string
	}]

	assumptions?: [...{
		assumption:        string
		validation_method: string
		risk_if_wrong:     string
	}]

	// Validation: No open questions allowed for implementation
	_ready_for_implementation: clarification_status == "RESOLVED"
}

// ============================================================================
// Section 1: EARS Requirements
// ============================================================================

#EarsRequirements: {
	// Pattern 1: Universal truths (always true)
	ubiquitous: [...#UbiquitousRequirement]

	// Pattern 2: Trigger-response pairs
	event_driven: [...#EventDrivenRequirement]

	// Pattern 3: Behavior during states
	state_driven?: [...#StateDrivenRequirement]

	// Pattern 4: Conditional on config/roles
	optional?: [...#OptionalRequirement]

	// Pattern 5: Things that must NEVER happen
	unwanted: [...#UnwantedRequirement]  // Required - inversion thinking

	// Pattern 6: State + Event combinations
	complex?: [...#ComplexRequirement]

	// Validation: Must have at least ubiquitous and event_driven
	_valid: len(ubiquitous) > 0 & len(event_driven) > 0 & len(unwanted) > 0
}

#UbiquitousRequirement: string & =~"^THE SYSTEM SHALL .+"

#EventDrivenRequirement: {
	trigger: string & =~"^WHEN .+"
	shall:   string & =~"^THE SYSTEM SHALL .+"
}

#StateDrivenRequirement: {
	state: string & =~"^WHILE .+"
	shall: string & =~"^THE SYSTEM SHALL .+"
}

#OptionalRequirement: {
	condition: string & =~"^WHERE .+"
	shall:     string & =~"^THE SYSTEM SHALL .+"
}

#UnwantedRequirement: {
	condition: string & =~"^IF .+"
	shall_not: string & =~"^THE SYSTEM SHALL NOT .+"
	because:   string & strings.MinRunes(10)  // Must explain why
}

#ComplexRequirement: {
	state:   string & =~"^WHILE .+"
	trigger: string & =~"^WHEN .+"
	shall:   string & =~"^THE SYSTEM SHALL .+"
}

// ============================================================================
// Section 2: KIRK Contracts (Design by Contract)
// ============================================================================

#KirkContracts: {
	preconditions:  #Preconditions
	postconditions: #Postconditions
	invariants:     [...string] & [_, ...]  // At least one invariant required
}

#Preconditions: {
	auth_required:   bool
	required_inputs: [...#InputSpec]
	system_state?:   [...string]
}

#InputSpec: {
	field:           string
	type:            "String" | "Int" | "Bool" | "List" | "Object"
	constraints:     string
	example_valid:   string | number | bool
	example_invalid: string | number | bool
}

#Postconditions: {
	state_changes:      [...string]
	return_guarantees:  [...#ReturnGuarantee]
	side_effects?:      [...string]
}

#ReturnGuarantee: {
	field:     string
	guarantee: string
}

// ============================================================================
// Section 2.5: Research Requirements (Investigate Before Implementing)
// ============================================================================
// From Anthropic: "ALWAYS read and understand relevant files before proposing code edits"

#ResearchRequirements: {
	files_to_read: [...{
		path:            string
		what_to_extract: string
		document_in:     string | *"research_notes.md"
	}]

	patterns_to_find?: [...{
		pattern:            string
		purpose:            string
		expected_locations: string
	}]

	prior_art?: [...{
		feature:       string
		location:      string
		what_to_learn: string
	}]

	external_docs?: [...{
		url:     string
		section: string
		extract: string
	}]

	research_questions: [...{
		question: string
		answered: bool
		answer?:  string
	}]

	research_complete_when: [...string] & [_, ...]  // At least one criterion
}

// ============================================================================
// Section 3: Inversion Analysis
// ============================================================================

#InversionAnalysis: {
	security_failures?:       [...#FailureMode]
	usability_failures?:      [...#FailureMode]
	data_integrity_failures?: [...#FailureMode]
	integration_failures?:    [...#FailureMode]

	// Must have at least one category of failures analyzed
	_valid: len(security_failures) > 0 | len(usability_failures) > 0 | len(integration_failures) > 0
}

#FailureMode: {
	failure:     string
	prevention:  string
	test_for_it: string  // Name of test that verifies prevention
}

// ============================================================================
// Section 4: ATDD Acceptance Tests
// ============================================================================

#AcceptanceTests: {
	happy_paths:     [...#AcceptanceTest] & [_, ...]  // At least one required
	error_paths:     [...#AcceptanceTest] & [_, ...]  // At least one required
	edge_cases?:     [...#EdgeCaseTest]
	contract_tests?: [...#ContractTest]
}

#AcceptanceTest: {
	name:  string & =~"^test_.+"
	given: string
	when:  string
	then:  [...string] & [_, ...]  // At least one assertion

	// Real data - no placeholders allowed
	real_input:      string
	expected_output: string | null  // null for error path tests
	expected_error?: string
}

#EdgeCaseTest: {
	name:     string & =~"^test_.+"
	scenario: string
	input:    string
	expected: string
}

#ContractTest: {
	name:     string & =~"^test_(precondition|postcondition|invariant)_.+"
	verifies: string
	test:     string
}

// ============================================================================
// Section 5: End-to-End Tests
// ============================================================================

#E2ETests: {
	pipeline_test: #PipelineTest  // The "soup to nuts" test
	e2e_scenarios?: [...#E2EScenario]
}

#PipelineTest: {
	name:        string & =~"^test_full_.+"
	description: string

	setup: {
		files_to_create?: [...{
			path:    string
			content: string
		}]
		environment?: [...string]
		precondition_commands?: [...string]
	}

	execute: {
		command:     string
		stdin?:      string
		timeout_ms?: number | *10000
	}

	verify: {
		exit_code:           number
		stdout_contains?:    [...string]
		stdout_matches_json?: [...{
			path:        string
			value?:      _
			type?:       string
			pattern?:    string
			min_length?: number
		}]
		files_created?: [...{
			path:      string
			contains?: string
		}]
		files_not_modified?: [...string]
		side_effects?:       [...string]
	}

	cleanup?: {
		commands?:        [...string]
		files_to_delete?: [...string]
	}
}

#E2EScenario: {
	name:        string
	description: string
	steps:       [...{
		action: string
		verify: string
	}]
}

// ============================================================================
// Section 5.5: Verification Checkpoints (Quality Gates)
// ============================================================================
// From Anthropic: "Have the model write tests in structured format (tests.json)"

#VerificationCheckpoints: {
	gate_0_research: #Gate
	gate_1_tests:    #Gate
	gate_2_implementation: #Gate
	gate_3_integration: #Gate

	tests_json?: {
		format:   string
		location: string
	}
}

#Gate: {
	name:              string
	must_pass_before:  string
	checks:            [...string] & [_, ...]  // At least one check
	evidence_required: [...string] & [_, ...]  // At least one evidence
}

// ============================================================================
// Section 6: Implementation Tasks
// ============================================================================

#ImplementationTasks: {
	// Phase 0: Research (must complete before code)
	phase_0_research: #TaskPhase

	// Phase 1: Write failing tests first (TDD Red)
	phase_1_tests_first: #TaskPhase

	// Phase 2: Implement to make tests pass (TDD Green)
	phase_2_implementation: #TaskPhase

	// Phase 3: Wire up to system (Integration)
	phase_3_integration?: #TaskPhase

	// Phase 4: Verification
	phase_4_verification: #TaskPhase

	// Parallelization rules for AI
	parallelization_rules?: [...string]
}

#TaskPhase: {
	parallelizable: bool
	gate_required?: string
	tasks:          [...#Task] & [_, ...]  // At least one task
}

#Task: {
	task:      string
	file?:     string
	what?:     string
	done_when: string

	// Parallelization markers
	parallel_group?: string
	depends_on?:     string | null

	patterns_to_use?: [...string]
	commands?: [...string]
	expected?: string
}

// ============================================================================
// Section 7: Failure Modes & Debugging
// ============================================================================

#FailureModes: {
	failure_modes: [...#FailureModeDebug]
	debugging_commands?: [...#DebuggingCommand]
}

#FailureModeDebug: {
	symptom:      string
	likely_cause: string
	where_to_look: [...{
		file:          string
		line_range?:   string
		function?:     string
		what_to_check: string
	}]
	fix_pattern: string
}

#DebuggingCommand: {
	scenario: string
	run:      string
	look_for: string
}

// ============================================================================
// Section 7.5: Anti-Hallucination Rules (Ground Truth Enforcement)
// ============================================================================
// From Anthropic: "Never speculate about code you have not opened"

#AntiHallucination: {
	read_before_write: [...{
		file:                     string
		must_read_first:          bool | *true
		key_sections_to_understand: [...string]
	}]

	verify_before_reference?: [...{
		type:              string
		expected_location: string
		verify_command:    string
	}]

	apis_that_exist: [...{
		api:         string
		signature:   string
		import_from: string
	}]

	apis_that_do_not_exist?: [...string]

	no_placeholder_values: [...string] & [_, ...]  // At least one rule

	git_verification: {
		before_claiming_done: string
	}
}

// ============================================================================
// Section 7.6: Context Window Survival (Long-Running Task Support)
// ============================================================================
// From Anthropic: "For tasks spanning multiple context windows, use structured state files"

#ContextSurvival: {
	progress_file: {
		path:   string
		format: string
	}

	tests_status_file?: {
		path:             string
		update_frequency: string
	}

	research_notes_file?: {
		path:     string
		contains: [...string]
	}

	git_checkpoints?: {
		frequency:      string
		message_format: string
		purpose:        string
	}

	recovery_instructions: string
}

// ============================================================================
// Section 8: Completion Checklist
// ============================================================================

#CompletionChecklist: {
	tests: [...string] & [
			"[ ] All acceptance tests written and passing",
			"[ ] All error path tests written and passing",
			"[ ] E2E pipeline test passing with real data",
			"[ ] No mocks or fake data in any test",
			...,
	]

	code: [...string] & [
			"[ ] Implementation uses Result<T, Error> throughout",
			"[ ] Zero unwrap() or expect() calls",
			...,
	]

	ci: [...string] & [
			"[ ] moon run :ci passes",
			...,
	]

	documentation?: [...string]
}

// ============================================================================
// Section 9: Context & References
// ============================================================================

#Context: {
	related_files: [...{
		path:      string
		relevance: string
	}]

	similar_implementations?: [...string]
	external_references?:     [...string]

	codebase_patterns?: [...{
		pattern:         string
		example_location: string
		how_to_apply:    string
	}]
}

// ============================================================================
// Section 10: AI Implementation Hints (Claude 4.x Optimized)
// ============================================================================

#AIHints: {
	do: [...string] & [_, ...]      // At least one "do"
	do_not: [...string] & [_, ...]  // At least one "do not"

	// Claude 4.x specific: Avoid "think" in prompts
	language_guidance?: {
		avoid:       [...string]
		use_instead: [...string]
	}

	// Claude 4.x specific: Action vs suggestion guidance
	action_guidance?: string

	// Claude 4.x specific: Parallel execution guidance
	parallel_execution?: string

	// Claude 4.x specific: Incremental progress
	incremental_progress?: string

	code_patterns?: [...{
		name:     string
		use_when: string
		example:  string
	}]

	// Constitutional principles (project invariants)
	constitution: [...string] & [_, ...]  // At least one principle
}

// ============================================================================
// Validation: Quality Gates (Extended for 16-Section Template)
// ============================================================================

// A bead is only valid if it passes ALL quality gates
#ValidBead: #EnhancedBead & {
	// Gate 0: Clarifications resolved
	clarifications: _ready_for_implementation: true

	// Gate 1: EARS coverage
	ears_requirements: _valid: true

	// Gate 2: Research requirements defined
	research_requirements: files_to_read: [_, ...]

	// Gate 3: Inversion analysis done
	inversions: _valid: true

	// Gate 4: Happy and error path tests defined
	acceptance_tests: {
		happy_paths: [_, ...]
		error_paths: [_, ...]
	}

	// Gate 5: Pipeline test defined
	e2e_tests: pipeline_test: name: =~"^test_full_.+"

	// Gate 6: Verification checkpoints defined
	verification_checkpoints: {
		gate_0_research: checks: [_, ...]
		gate_1_tests: checks: [_, ...]
	}

	// Gate 7: Test-first tasks defined
	implementation_tasks: phase_0_research: tasks: [_, ...]

	// Gate 8: Anti-hallucination rules
	anti_hallucination: read_before_write: [_, ...]

	// Gate 9: Context survival configured
	context_survival: progress_file: path: =~".+"

	// Gate 10: Constitution defined
	ai_hints: constitution: [_, ...]
}

// Lightweight validation for in-progress beads
#DraftBead: #EnhancedBead & {
	// Only require clarifications status to be set
	clarifications: clarification_status: "RESOLVED" | "HAS_OPEN_QUESTIONS"
}
