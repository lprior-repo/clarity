// Interview Questions Database
// Profile-specific questions for the interview system
// 6 profiles × 5 rounds = comprehensive coverage
package intent

// Question schema
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

// Profile-specific question structure
#ProfileQuestions: {
	round_1: [...#Question]
	round_2: [...#Question]
}

// The exported questions database
questions: {
	// =========================================================================
	// API PROFILE - REST/HTTP API design
	// =========================================================================
	api: #ProfileQuestions & {
		round_1: [
			{
				id:          "r1-user-api-1"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "In one sentence, what should this API do?"
				context:     "We're starting with the core intent. Give us the simplest possible description."
				example:     "Allow users to log in with email and password"
				extract_into: ["name"]
			},
			{
				id:          "r1-user-api-2"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "Who will use this API? What are they trying to accomplish?"
				context:     "Understanding your audience helps us design the right behavior."
				example:     "Mobile app users, web frontend, and third-party integrations"
				extract_into: ["audience", "success_criteria"]
			},
			{
				id:          "r1-user-api-3"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "Walk me through the happy path. What happens step-by-step?"
				context:     "Describe the ideal flow from start to finish. Don't worry about errors yet."
				example:     "Client sends POST /login with email/password → validates → returns JWT token"
				extract_into: ["behaviors"]
			},
			{
				id:          "r1-dev-api-1"
				round:       1
				perspective: "developer"
				category:    "constraint"
				priority:    "important"
				question:    "What data model does this operate on? List the key entities."
				context:     "Understanding the domain helps us catch inconsistencies."
				example:     "Users (id, email, password_hash), Tokens (token, user_id, expires_at)"
				extract_into: ["entities"]
			},
			{
				id:          "r1-security-api-1"
				round:       1
				perspective: "security"
				category:    "constraint"
				priority:    "critical"
				question:    "What kind of authentication does this need?"
				context:     "Auth method cascades through the whole design."
				example:     "JWT for mobile, session cookies for web, API keys for server-to-server"
				extract_into: ["auth_method"]
			},
		]
		round_2: [
			{
				id:          "r2-user-api-1"
				round:       2
				perspective: "user"
				category:    "error_case"
				priority:    "critical"
				question:    "What's the most common error users will hit?"
				context:     "The error that happens 80% of the time."
				example:     "Wrong password, email already exists, invalid token"
				extract_into: ["error_cases"]
			},
			{
				id:          "r2-security-api-1"
				round:       2
				perspective: "security"
				category:    "error_case"
				priority:    "critical"
				question:    "What information should NEVER leak in error messages?"
				context:     "Error responses can expose sensitive information."
				example:     "Don't say 'email exists' - just say 'cannot create account'"
				extract_into: ["anti_patterns"]
			},
			{
				id:          "r2-dev-api-1"
				round:       2
				perspective: "developer"
				category:    "error_case"
				priority:    "important"
				question:    "What are the HTTP status codes you'll return?"
				context:     "200, 400, 401, 403, 404, 409, 500, 503, etc."
				example:     "200 OK, 400 Bad Request, 401 Unauthorized, 409 Conflict"
				extract_into: ["status_codes"]
			},
		]
	}

	// =========================================================================
	// CLI PROFILE - Command-line tool design
	// =========================================================================
	cli: #ProfileQuestions & {
		round_1: [
			{
				id:          "r1-user-cli-1"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "What's the main command users will run?"
				context:     "Start with the primary use case."
				example:     "intent check --file=spec.cue --target=http://api.example.com"
				extract_into: ["name", "command_name"]
			},
			{
				id:          "r1-user-cli-2"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "Who are the users of this CLI?"
				context:     "Developers? DevOps? QA engineers?"
				example:     "API test engineers and DevOps teams testing HTTP endpoints"
				extract_into: ["audience"]
			},
			{
				id:          "r1-dev-cli-1"
				round:       1
				perspective: "developer"
				category:    "constraint"
				priority:    "important"
				question:    "What are the main sub-commands or flags?"
				context:     "List the key operations users will perform."
				example:     "check, validate, generate, run, report, export"
				extract_into: ["behaviors"]
			},
		]
		round_2: [
			{
				id:          "r2-user-cli-1"
				round:       2
				perspective: "user"
				category:    "error_case"
				priority:    "critical"
				question:    "What exit codes should indicate failure?"
				context:     "Scripts need to know if a command succeeded or failed."
				example:     "0=success, 1=generic error, 2=usage error, 3=validation failed"
				extract_into: ["exit_codes"]
			},
			{
				id:          "r2-dev-cli-1"
				round:       2
				perspective: "developer"
				category:    "error_case"
				priority:    "important"
				question:    "How should errors be displayed?"
				context:     "stderr vs stdout, verbosity levels, JSON output, etc."
				example:     "Error message to stderr, JSON output to stdout, optional --debug flag"
				extract_into: ["error_handling"]
			},
		]
	}

	// =========================================================================
	// EVENT PROFILE - Event-driven system design
	// =========================================================================
	event: #ProfileQuestions & {
		round_1: [
			{
				id:          "r1-user-event-1"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "What events will this system emit?"
				context:     "Start with the main event types."
				example:     "user.created, user.deleted, order.placed, payment.confirmed"
				extract_into: ["name", "event_types"]
			},
			{
				id:          "r1-user-event-2"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "Who will consume these events?"
				context:     "What systems care about these events?"
				example:     "Email service, analytics pipeline, notification system"
				extract_into: ["audience"]
			},
			{
				id:          "r1-dev-event-1"
				round:       1
				perspective: "developer"
				category:    "constraint"
				priority:    "important"
				question:    "What fields must every event have?"
				context:     "The common schema across all events."
				example:     "id, timestamp, type, version, source, payload"
				extract_into: ["entities"]
			},
		]
		round_2: [
			{
				id:          "r2-ops-event-1"
				round:       2
				perspective: "ops"
				category:    "error_case"
				priority:    "critical"
				question:    "What happens if an event fails to deliver?"
				context:     "Delivery guarantees: at-most-once, at-least-once, exactly-once?"
				example:     "Retry up to 3 times with exponential backoff, then dead-letter queue"
				extract_into: ["delivery_guarantees"]
			},
		]
	}

	// =========================================================================
	// DATA PROFILE - Data model design
	// =========================================================================
	data: #ProfileQuestions & {
		round_1: [
			{
				id:          "r1-user-data-1"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "What's the primary entity this system manages?"
				context:     "The main thing users care about."
				example:     "Users, Products, Orders, Documents"
				extract_into: ["name", "entities"]
			},
			{
				id:          "r1-ops-data-1"
				round:       1
				perspective: "ops"
				category:    "constraint"
				priority:    "important"
				question:    "How long must data be kept? Any retention policies?"
				context:     "Affects storage, compliance, archival strategy."
				example:     "Keep indefinitely, delete after 90 days, archive after 1 year"
				extract_into: ["retention"]
			},
		]
		round_2: [
			{
				id:          "r2-user-data-1"
				round:       2
				perspective: "user"
				category:    "error_case"
				priority:    "important"
				question:    "Can data be deleted? What happens to related data?"
				context:     "Cascading deletes, soft deletes, audit trails."
				example:     "Delete user → archive their orders, keep for 7 years for tax compliance"
				extract_into: ["deletion_policy"]
			},
		]
	}

	// =========================================================================
	// WORKFLOW PROFILE - State machine / process design
	// =========================================================================
	workflow: #ProfileQuestions & {
		round_1: [
			{
				id:          "r1-user-workflow-1"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "What are the main workflow states?"
				context:     "How does something move from start to finish?"
				example:     "Draft → Submitted → Approved → Completed"
				extract_into: ["states"]
			},
			{
				id:          "r1-user-workflow-2"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "What transitions between states are allowed?"
				context:     "Not all state changes should be valid."
				example:     "Can't go from Approved back to Draft; Draft can skip to Completed if auto-approved"
				extract_into: ["transitions"]
			},
		]
		round_2: [
			{
				id:          "r2-user-workflow-1"
				round:       2
				perspective: "user"
				category:    "error_case"
				priority:    "critical"
				question:    "What happens if a step fails? How does it recover?"
				context:     "Retry, rollback, manual intervention?"
				example:     "Payment fails → send email → user can retry → auto-retry after 1 hour"
				extract_into: ["error_recovery"]
			},
		]
	}

	// =========================================================================
	// UI PROFILE - User interface design
	// =========================================================================
	ui: #ProfileQuestions & {
		round_1: [
			{
				id:          "r1-user-ui-1"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "What's the main screen users see first?"
				context:     "The entry point to your application."
				example:     "Dashboard showing recent activity, login screen, or home page"
				extract_into: ["name", "screens"]
			},
			{
				id:          "r1-user-ui-2"
				round:       1
				perspective: "user"
				category:    "happy_path"
				priority:    "critical"
				question:    "What's the core user flow?"
				context:     "The happy path through your interface."
				example:     "Log in → View dashboard → Create new item → Confirm → See results"
				extract_into: ["user_flows"]
			},
		]
		round_2: [
			{
				id:          "r2-user-ui-1"
				round:       2
				perspective: "user"
				category:    "error_case"
				priority:    "important"
				question:    "What error messages will users see?"
				context:     "Form validation, API errors, permission denied, etc."
				example:     "Email is required, Password must be 8+ characters, Access denied"
				extract_into: ["error_messages"]
			},
		]
	}

	// =========================================================================
	// COMMON ROUNDS - Shared across all profiles
	// =========================================================================
	common: {
		// Round 3: Edge Cases
		round_3: [...#Question] & [
			{
				id:          "r3-dev-1"
				round:       3
				perspective: "developer"
				category:    "edge_case"
				priority:    "important"
				question:    "What's the maximum size of inputs/payloads?"
				context:     "File uploads, API request bodies, database entries."
				example:     "Max file: 100MB, max request body: 10MB, max field length: 255 chars"
				extract_into: ["size_limits"]
			},
			{
				id:          "r3-ops-1"
				round:       3
				perspective: "ops"
				category:    "edge_case"
				priority:    "important"
				question:    "What happens under extreme load?"
				context:     "Rate limiting, queuing, graceful degradation?"
				example:     "Queue requests, return 429 Too Many Requests, fail fast at 10k req/sec"
				extract_into: ["load_handling"]
			},
			{
				id:          "r3-security-1"
				round:       3
				perspective: "security"
				category:    "edge_case"
				priority:    "critical"
				question:    "What if someone tries to do something they shouldn't?"
				context:     "Authorization, privilege escalation, race conditions."
				example:     "User A can't see User B's data, can't modify other users' profiles"
				extract_into: ["security_rules"]
			},
		]

		// Round 4: Security & Compliance
		round_4: [...#Question] & [
			{
				id:          "r4-security-1"
				round:       4
				perspective: "security"
				category:    "constraint"
				priority:    "critical"
				question:    "What data is sensitive and needs encryption?"
				context:     "Passwords, tokens, PII, payment info."
				example:     "Passwords (bcrypt), tokens (in-transit), SSN/credit cards (at-rest)"
				extract_into: ["encryption_requirements"]
			},
			{
				id:          "r4-business-1"
				round:       4
				perspective: "business"
				category:    "constraint"
				priority:    "important"
				question:    "Are there compliance requirements? (GDPR, HIPAA, PCI, SOC2?)"
				context:     "Legal, regulatory, industry standards."
				example:     "GDPR (EU users), PCI DSS (payments), SOC2 (enterprise customers)"
				extract_into: ["compliance"]
			},
		]

		// Round 5: Operations
		round_5: [...#Question] & [
			{
				id:          "r5-ops-1"
				round:       5
				perspective: "ops"
				category:    "constraint"
				priority:    "important"
				question:    "Where will this run? (cloud, on-prem, edge, regions?)"
				context:     "Deployment topology affects everything."
				example:     "AWS (us-east-1, eu-west-1), multi-region for GDPR, CDN for static assets"
				extract_into: ["deployment_target"]
			},
			{
				id:          "r5-ops-2"
				round:       5
				perspective: "ops"
				category:    "nonfunctional"
				priority:    "critical"
				question:    "What's your uptime requirement? (SLA?)"
				context:     "99%, 99.9%, 99.99% availability."
				example:     "99.9% (8.76 hours downtime/year), acceptable during maintenance windows"
				extract_into: ["sla"]
			},
			{
				id:          "r5-ops-3"
				round:       5
				perspective: "ops"
				category:    "nonfunctional"
				priority:    "important"
				question:    "How will you monitor this? (metrics, logs, alerts?)"
				context:     "Observability strategy."
				example:     "Prometheus metrics, ELK logs, PagerDuty alerts on p95 latency > 500ms"
				extract_into: ["monitoring"]
			},
		]
	}
}
