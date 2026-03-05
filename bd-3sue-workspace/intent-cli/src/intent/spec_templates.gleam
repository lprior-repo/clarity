/// Spec template generators for quick project scaffolding
/// Provides pre-built templates for common project types
import gleam/list
import gleam/string

/// Available template types
pub type TemplateType {
  ApiSpec
  CliTool
  DataPipeline
  Workflow
}

/// Template metadata with description
pub type Template {
  Template(
    type_: TemplateType,
    name: String,
    description: String,
    category: String,
  )
}

/// Get all available templates
pub fn list_templates() -> List(Template) {
  [
    Template(
      type_: ApiSpec,
      name: "api-spec",
      description: "REST/GraphQL API with endpoints, authentication, and data models",
      category: "web",
    ),
    Template(
      type_: CliTool,
      name: "cli-tool",
      description: "Command-line tool with commands, arguments, and exit codes",
      category: "tooling",
    ),
    Template(
      type_: DataPipeline,
      name: "data-pipeline",
      description: "Data processing pipeline with sources, transforms, and destinations",
      category: "data",
    ),
    Template(
      type_: Workflow,
      name: "workflow",
      description: "Business workflow with states, transitions, and actors",
      category: "automation",
    ),
  ]
}

/// Find template by name
pub fn find_template(name: String) -> Result(TemplateType, String) {
  let templates = list_templates()
  let found =
    list.find(templates, fn(t) { t.name == string.lowercase(name) })

  case found {
    Ok(template) -> Ok(template.type_)
    Error(_) -> {
      let valid_names =
        templates
        |> list.map(fn(t) { t.name })
        |> string.join(", ")

      Error(
        "Unknown template: '"
        <> name
        <> "'\n\nAvailable templates:\n  "
        <> valid_names,
      )
    }
  }
}

/// Generate spec content from template
pub fn generate_spec(
  template_type: TemplateType,
  spec_name: String,
  package_name: String,
) -> String {
  case template_type {
    ApiSpec -> generate_api_spec(spec_name, package_name)
    CliTool -> generate_cli_spec(spec_name, package_name)
    DataPipeline -> generate_data_pipeline_spec(spec_name, package_name)
    Workflow -> generate_workflow_spec(spec_name, package_name)
  }
}

/// Generate API spec template
fn generate_api_spec(name: String, package: String) -> String {
  "package "
  <> package
  <> "

import \"github.com/intent-cli/intent/schema:intent\"

spec: intent.#Spec & {
	name: \""
  <> name
  <> "\"

	description: \"\"\"
		TODO: Describe your API here
		- What problem does it solve?
		- Who are the primary users?
		- What are the main capabilities?
		\"\"\"

	audience: \"API consumers (web, mobile, integrations)\"

	version: \"1.0.0\"

	success_criteria: [
		\"TODO: Add high-level success criteria\",
		\"Example: Users can authenticate and access protected resources\",
		\"Example: All endpoints return appropriate status codes\",
		\"Example: Error responses are structured and actionable\",
	]

	features: [
		{
			name: \"Authentication\"

			description: \"\"\"
				User authentication and authorization. Supports token-based
				authentication with proper error handling for invalid credentials.
				\"\"\"

			behaviors: [
				{
					name:   \"successful-login\"
					intent: \"Registered users can log in with valid credentials\"

					preconditions: [
						\"User exists in the system\",
						\"User has verified their email\",
					]

					postconditions: [
						\"User receives an authentication token\",
						\"Token can be used to access protected endpoints\",
					]

					verifications: [
						{
							description: \"Login with valid credentials returns token\"
							criteria: [
								\"Status code is 200\",
								\"Response contains authentication token\",
								\"Token has valid expiration time\",
							]
						},
					]
				},
				{
					name:   \"invalid-credentials-rejected\"
					intent: \"Login fails with wrong username or password\"

					requires: [\"successful-login\"]

					preconditions: [
						\"User exists but credentials are incorrect\",
					]

					postconditions: [
						\"No authentication token is issued\",
						\"Error message is generic (doesn't reveal user existence)\",
					]

					verifications: [
						{
							description: \"Login with invalid credentials returns error\"
							criteria: [
								\"Status code is 401\",
								\"Error message doesn't reveal if user exists\",
								\"No token is returned\",
							]
						},
					]

					tags: [\"security\", \"auth\"]
				},
			]
		},
		{
			name: \"Core Resources\"

			description: \"\"\"
				Core API resources for the main domain entities. Includes CRUD
				operations with proper validation and error handling.
				\"\"\"

			behaviors: [
				{
					name:   \"list-resources\"
					intent: \"Users can list all available resources\"

					preconditions: [
						\"User is authenticated\",
					]

					postconditions: [
						\"Returns paginated list of resources\",
						\"Resources are sorted by creation date (newest first)\",
					]

					verifications: [
						{
							description: \"Listing returns paginated results\"
							criteria: [
								\"Status code is 200\",
								\"Response contains array of resources\",
								\"Pagination metadata is included\",
							]
						},
					]
				},
				{
					name:   \"create-resource\"
					intent: \"Authenticated users can create new resources\"

					preconditions: [
						\"User is authenticated\",
						\"Request payload is valid\",
					]

					postconditions: [
						\"Resource is created in the system\",
						\"Resource ID is returned in response\",
						\"Resource appears in subsequent list queries\",
					]

					verifications: [
						{
							description: \"Creating a resource returns the new resource\"
							criteria: [
								\"Status code is 201\",
								\"Response contains generated resource ID\",
								\"All fields are preserved\",
							]
						},
					]

					tags: [\"crud\"]
				},
			]
		},
	]

	invariants: [
		{
			name:        \"consistent-error-format\"
			description: \"All errors follow a consistent structure\"
			criteria: [
				\"Error responses contain 'error' field with message\",
				\"4xx errors include a 'code' field for specific error type\",
				\"5xx errors include a 'request_id' for support lookup\",
			]
		},
		{
			name:        \"no-sensitive-data-in-errors\"
			description: \"Error messages never expose sensitive information\"
			criteria: [
				\"Passwords never appear in error messages\",
				\"Internal stack traces are not exposed to clients\",
				\"Database errors are generic, not exposing schema details\",
			]
		},
		{
			name:        \"idempotent-operations\"
			description: \"Safe operations can be repeated without side effects\"
			criteria: [
				\"GET requests never modify server state\",
				\"Multiple identical GET requests return same data\",
				\"HEAD and OPTIONS are also idempotent\",
			]
		},
	]

	anti_patterns: [
		{
			name:        \"hardcoded-auth-tokens\"
			description: \"Don't hardcode authentication tokens in code\"

			bad_example: api_token: \"sk_live_1234567890abcdef\"

			good_example: api_token: #\"\\${API_TOKEN}\"#

			why: \"Hardcoded tokens in version control are a security risk\"
		},
		{
			name:        \"missing-pagination\"
			description: \"Don't return all resources without pagination\"

			bad_example: {
				endpoint: \"/api/users\"
				response: \"Returns all 10,000 users\"
			}

			good_example: {
				endpoint: \"/api/users?page=1&limit=50\"
				response: \"Returns first 50 users with pagination metadata\"
			}

			why: \"Unbounded queries cause performance issues and timeouts\"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: [
				\"REST API with OpenAPI/Swagger documentation\",
				\"JWT or OAuth2 for authentication\",
				\"Rate limiting to prevent abuse\",
				\"Request validation with clear error messages\",
			]
		}

		entities: {
			\"User\": {
				fields: {
					\"id\": \"Unique identifier (UUID or prefixed string)\"
					\"email\": \"Email address (validated format)\"
					\"name\": \"Full name or display name\"
					\"created_at\": \"ISO8601 timestamp of creation\"
					\"updated_at\": \"ISO8601 timestamp of last update\"
				}
			}
		}

		security: {
			password_hashing: \"Use bcrypt, scrypt, or argon2 (never plain SHA256)\"
			jwt_algorithm:    \"RS256 for production, HS256 for development only\"
			jwt_expiry:       \"15 minutes for access tokens, 7 days for refresh tokens\"
			rate_limiting:    \"Implement per-IP and per-user rate limits\"
		}

		pitfalls: [
			\"Don't ignore validation errors - validate early and fail fast\",
			\"Avoid N+1 queries - use eager loading or batch queries\",
			\"Don't leak internal state in error messages\",
			\"Always sanitize user input to prevent injection attacks\",
		]
	}
}
"
}

/// Generate CLI tool spec template
fn generate_cli_spec(name: String, package: String) -> String {
  "package "
  <> package
  <> "

import \"github.com/intent-cli/intent/schema:intent\"

spec: intent.#Spec & {
	name: \""
  <> name
  <> "\"

	description: \"\"\"
		TODO: Describe your CLI tool here
		- What problem does it solve?
		- Who are the primary users?
		- What are the main commands?
		\"\"\"

	audience: \"Developers and system administrators\"

	version: \"1.0.0\"

	success_criteria: [
		\"TODO: Add high-level success criteria\",
		\"Example: All commands work with --help flag\",
		\"Example: Exit codes follow POSIX conventions\",
		\"Example: Errors provide clear, actionable messages\",
	]

	features: [
		{
			name: \"Command Structure\"

			description: \"\"\"
				Main command structure with subcommands, flags, and arguments.
				Follows CLI conventions with --help, --version, and proper exit codes.
				\"\"\"

			behaviors: [
				{
					name:   \"shows-help\"
					intent: \"Running with --help shows usage information\"

					verifications: [
						{
							description: \"Help text is comprehensive\"
							criteria: [
								\"Shows all available commands\",
								\"Lists all flags with descriptions\",
								\"Includes usage examples\",
								\"Exits with code 0\",
							]
						},
					]
				},
				{
					name:   \"shows-version\"
					intent: \"--version flag displays version information\"

					verifications: [
						{
							description: \"Version output is correct\"
							criteria: [
								\"Displays semantic version (e.g., 1.0.0)\",
								\"Exits with code 0\",
								\"Format matches standard conventions\",
							]
						},
					]
				},
			]
		},
		{
			name: \"Core Commands\"

			description: \"\"\"
				Primary commands that implement the tool's main functionality.
				Each command should do one thing well and fail gracefully.
				\"\"\"

			behaviors: [
				{
					name:   \"successful-command-execution\"
					intent: \"Valid command execution produces expected output\"

					preconditions: [
						\"All required arguments are provided\",
						\"Input files or resources are accessible\",
					]

					postconditions: [
						\"Command produces output in expected format\",
						\"Exit code is 0 (success)\",
						\"Side effects are applied as expected\",
					]

					verifications: [
						{
							description: \"Command executes successfully\"
							criteria: [
								\"Exit code is 0\",
								\"Output is written to stdout (or file as specified)\",
								\"No errors are written to stderr\",
							]
						},
					]

					tags: [\"happy-path\"]
				},
				{
					name:   \"missing-required-arguments\"
					intent: \"Command fails clearly when required arguments are missing\"

					verifications: [
						{
							description: \"Missing arguments show helpful error\"
							criteria: [
								\"Exit code is non-zero (typically 1 or 2)\",
								\"Error message lists missing required arguments\",
								\"Suggests using --help for usage information\",
							]
						},
					]

					tags: [\"error-handling\"]
				},
				{
					name:   \"invalid-input-handling\"
					intent: \"Invalid input is rejected with clear error messages\"

					verifications: [
						{
							description: \"Invalid input produces actionable error\"
							criteria: [
								\"Exit code is non-zero\",
								\"Error message describes what was invalid\",
								\"Error message suggests how to fix the input\",
							]
						},
					]

					tags: [\"validation\"]
				},
			]
		},
	]

	invariants: [
		{
			name:        \"posix-exit-codes\"
			description: \"Exit codes follow POSIX conventions\"
			criteria: [
				\"0 = success\",
				\"1-64 = application errors (1 = general error, 2 = misuse)\",
				\"65+ = system errors (not typically used by apps)\",
			]
		},
		{
			name:        \"separate-output-streams\"
			description: \"Normal output and errors go to correct streams\"
			criteria: [
				\"Normal results written to stdout\",
				\"Errors and diagnostics written to stderr\",
				\"Progress information written to stderr (allows stdout piping)\"
			]
		},
		{
			name:        \"fail-fast\"
			description: \"Errors stop execution immediately\"
			criteria: [
				\"Validation errors prevent any work from starting\",
				\"Runtime errors stop current operation\",
				\"Exit codes accurately reflect failure\",
			]
		},
	]

	anti_patterns: [
		{
			name:        \"silent-failures\"
			description: \"Don't fail silently without error messages\"

			bad_example: {
				error: \"Command exits with code 1 but prints nothing\"
			}

			good_example: {
				error: \"Error: Unable to read file 'data.txt': Permission denied\"
			}

			why: \"Silent failures frustrate users and make debugging impossible\"
		},
		{
			name:        \"inconsistent-flags\"
			description: \"Don't use inconsistent flag naming\"

			bad_example: {
				flags: [\"--verbose\", \"-q\", \"--output-file\", \"-d\"]
			}

			good_example: {
				flags: [
					\"--verbose, -v\",
					\"--quiet, -q\",
					\"--output, -o\",
					\"--debug, -d\",
				]
			}

			why: \"Consistent naming follows user expectations and reduces confusion\"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: [
				\"CLI framework (e.g., Commander, Clap, argparse)\",
				\"Structured logging with levels (debug, info, warn, error)\",
				\"Configuration file support (config.yaml/.toolrc)\",
				\"Shell completion scripts (bash, zsh, fish)\",
			]
		}

		entities: {
			\"Command\": {
				fields: {
					\"name\": \"Command name (kebab-case)\"
					\"arguments\": \"List of positional arguments with types\"
					\"flags\": \"Optional flags with short and long forms\"
					\"description\": \"Short description for help text\"
				}
			}
		}

		security: {
			password_hashing: \"N/A for CLI (consider keyring for stored credentials)\"
			jwt_algorithm:    \"N/A for CLI\"
			jwt_expiry:       \"N/A for CLI\"
			rate_limiting:    \"N/A for CLI\"
		}

		pitfalls: [
			\"Don't ignore exit codes in scripts - always check $? after execution\",
			\"Avoid parsing command output - use dedicated output formats (JSON)\",
			\"Don't assume current directory - always use absolute paths when needed\",
			\"Never log sensitive data (passwords, tokens, personal info)\",
		]
	}
}
"
}

/// Generate data pipeline spec template
fn generate_data_pipeline_spec(name: String, package: String) -> String {
  "package "
  <> package
  <> "

import \"github.com/intent-cli/intent/schema:intent\"

spec: intent.#Spec & {
	name: \""
  <> name
  <> "\"

	description: \"\"\"
		TODO: Describe your data pipeline here
		- What data does it process?
		- Where does data come from?
		- Where does the data go?
		\"\"\"

	audience: \"Data engineers and analysts\"

	version: \"1.0.0\"

	success_criteria: [
		\"TODO: Add high-level success criteria\",
		\"Example: Pipeline processes data correctly end-to-end\",
		\"Example: Failed batches can be recovered and reprocessed\",
		\"Example: Data quality issues are detected and reported\",
	]

	features: [
		{
			name: \"Data Ingestion\"

			description: \"\"\"
				Data sources are read and ingested into the pipeline. Handles
				connection failures, rate limits, and schema validation.
				\"\"\"

			behaviors: [
				{
					name:   \"read-from-source\"
					intent: \"Pipeline reads data from configured source\"

					preconditions: [
						\"Source is accessible and authenticated\",
						\"Source schema is known\",
					]

					postconditions: [
						\"Data is read into staging area\",
						\"Metadata (row count, timestamp) is recorded\",
					]

					verifications: [
						{
							description: \"Data is successfully read\"
							criteria: [
								\"All expected records are read\",
								\"Schema validation passes\",
								\"Read operations are idempotent\",
							]
						},
					]

					tags: [\"ingestion\"]
				},
				{
					name:   \"handle-unavailable-source\"
					intent: \"Pipeline fails gracefully when source is unavailable\"

					verifications: [
						{
							description: \"Source unavailability is handled\"
							criteria: [
								\"Error is logged with source details\",
								\"Pipeline exits with non-zero status\",
								\"No partial data is written to destination\",
							]
						},
					]

					tags: [\"error-handling\"]
				},
			]
		},
		{
			name: \"Data Transformation\"

			description: \"\"\"
				Data is transformed according to business rules. Includes
				cleaning, enrichment, validation, and format conversion.
				\"\"\"

			behaviors: [
				{
					name:   \"apply-transformations\"
					intent: \"Data is transformed according to configuration\"

					preconditions: [
						\"Data exists in staging area\",
						\"Transformation rules are configured\",
					]

					postconditions: [
						\"Transformed data is written to output staging\",
						\"Transformation metrics are recorded\",
					]

					verifications: [
						{
							description: \"Transformations are applied correctly\"
							criteria: [
								\"All business rules are applied\",
								\"Data quality checks pass\",
								\"Output schema matches destination requirements\",
							]
						},
					]

					tags: [\"transform\"]
				},
			]
		},
		{
			name: \"Data Loading\"

			description: \"\"\"
				Transformed data is loaded into the destination. Supports
				batch loading, deduplication, and transaction safety.
				\"\"\"

			behaviors: [
				{
					name:   \"write-to-destination\"
					intent: \"Processed data is written to destination\"

					preconditions: [
						\"Transformed data is ready\",
						\"Destination is accessible\",
					]

					postconditions: [
						\"Data is committed to destination\",
						\"Load metadata is recorded\",
					]

					verifications: [
						{
							description: \"Data is successfully loaded\"
							criteria: [
								\"All records are written\",
								\"No duplicate records are created\",
								\"Write operation is atomic (all or nothing)\"
							]
						},
					]

					tags: [\"loading\"]
				},
			]
		},
	]

	invariants: [
		{
			name:        \"data-quality-validation\"
			description: \"Data quality is validated at each stage\"
			criteria: [
				\"Schema validation checks data types\",
				\"Null checks ensure required fields are present\",
				\"Range validation catches invalid values\",
				\"Referential integrity validates foreign keys\",
			]
		},
		{
			name:        \"audit-trail\"
			description: \"All data operations are logged for audit\"
			criteria: [
				\"Each record records source system and timestamp\",
				\"Transformations are logged with parameters\",
				\"Load operations track record counts and status\",
			]
		},
		{
			name:        \"idempotent-batches\"
			description: \"Re-running a batch doesn't create duplicates\"
			criteria: [
				\"Deduplication uses natural keys or hashes\",
				\"Upsert operations prefer updates over inserts\",
				\"Duplicate detection works across batch runs\",
			]
		},
	]

	anti_patterns: [
		{
			name:        \"silent-data-loss\"
			description: \"Don't lose data silently in transformations\"

			bad_example: {
				transform: \"Rows failing validation are dropped\"
			}

			good_example: {
				transform: \"Rows failing validation are logged and sent to error table\"
			}

			why: \"Silent data loss corrupts analytics and business decisions\"
		},
		{
			name:        \"tightly-coupled-stages\"
			description: \"Don't tightly couple pipeline stages\"

			bad_example: {
				pipeline: \"Ingestion, transform, and load in one monolithic transaction\"
			}

			good_example: {
				pipeline: \"Separate stages with staging tables between them\"
			}

			why: \"Loose coupling allows partial recovery and easier debugging\"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: [
				\"Workflow orchestrator (Airflow, Prefect, Dagster)\",
				\"Data processing framework (Spark, Pandas, dbt)\",
				\"Data warehouse (Snowflake, BigQuery, Redshift)\",
				\"Monitoring and alerting (Prometheus, Grafana)\",
			]
		}

		entities: {
			\"PipelineRun\": {
				fields: {
					\"run_id\": \"Unique identifier for this pipeline execution\"
					\"start_time\": \"Pipeline start timestamp\"
					\"end_time\": \"Pipeline completion timestamp\"
					\"status\": \"Current status (running, success, failed)\"
					\"records_read\": \"Number of records read from source\"
					\"records_written\": \"Number of records written to destination\"
				}
			}
		}

		security: {
			password_hashing: \"N/A (use vault for credentials)\"
			jwt_algorithm:    \"N/A\"
			jwt_expiry:       \"N/A\"
			rate_limiting:    \"Apply API rate limits to source systems\"
		}

		pitfalls: [
			\"Don't assume data quality - always validate\",
			\"Avoid processing all data in memory - use streaming/chunking\",
			\"Never hardcode credentials - use secret management\",
			\"Don't ignore timezone differences - use UTC consistently\",
		]
	}
}
"
}

/// Generate workflow spec template
fn generate_workflow_spec(name: String, package: String) -> String {
  "package "
  <> package
  <> "

import \"github.com/intent-cli/intent/schema:intent\"

spec: intent.#Spec & {
	name: \""
  <> name
  <> "\"

	description: \"\"\"
		TODO: Describe your workflow here
		- What business process does it automate?
		- Who are the participants/actors?
		- What are the key states and transitions?
		\"\"\"

	audience: \"Business users and system integrators\"

	version: \"1.0.0\"

	success_criteria: [
		\"TODO: Add high-level success criteria\",
		\"Example: Workflow completes all required steps\",
		\"Example: Users can track workflow status at any time\",
		\"Example: Failed steps can be retried independently\",
	]

	features: [
		{
			name: \"Workflow Initialization\"

			description: \"\"\"
				Workflow instances are created and initialized with initial state.
				Validates required inputs and assigns initial actors.
				\"\"\"

			behaviors: [
				{
					name:   \"create-workflow-instance\"
					intent: \"New workflow instance is created with valid inputs\"

					preconditions: [
						\"User is authenticated and authorized\",
						\"Required inputs are provided\",
					]

					postconditions: [
						\"Workflow instance exists in initial state\",
						\"Initial state is assigned to correct actor\",
						\"Workflow ID is returned to caller\",
					]

					verifications: [
						{
							description: \"Workflow is created successfully\"
							criteria: [
								\"Workflow ID is generated and returned\",
								\"Workflow status is 'pending' or 'in_progress'\",
								\"Initial state is correct\",
							]
						},
					]

					tags: [\"lifecycle\"]
				},
			]
		},
		{
			name: \"State Transitions\"

			description: \"\"\"
				Workflows move through defined states based on actions and conditions.
				Each transition is validated and recorded for audit.
				\"\"\"

			behaviors: [
				{
					name:   \"valid-state-transition\"
					intent: \"Workflow moves to next state when conditions are met\"

					preconditions: [
						\"Current actor is authorized for this action\",
						\"Transition conditions are satisfied\",
					]

					postconditions: [
						\"Workflow state is updated\",
						\"Transition is logged with timestamp and actor\",
						\"Next actor is notified (if applicable)\",
					]

					verifications: [
						{
							description: \"State transition succeeds\"
							criteria: [
								\"New state is valid for current workflow\",
								\"Transition history is recorded\",
								\"Authorization check passed\",
							]
						},
					]

					tags: [\"transitions\"]
				},
				{
					name:   \"invalid-transition-rejected\"
					intent: \"Invalid state transitions are rejected\"

					verifications: [
						{
							description: \"Invalid transitions fail\"
							criteria: [
								\"Error message explains why transition is invalid\",
								\"Workflow state remains unchanged\",
								\"Event is logged for security auditing\",
							]
						},
					]

					tags: [\"validation\"]
				},
			]
		},
		{
			name: \"Actor Assignment\"

			description: \"\"\"
				Workflow steps are assigned to appropriate actors (users or systems).
				Assignments can be manual (by user) or automatic (by rules).
				\"\"\"

			behaviors: [
				{
					name:   \"assign-to-actor\"
					intent: \"Workflow step is assigned to a specific actor\"

					preconditions: [
						\"Assigning actor has permission to assign\",
						\"Target actor is valid for this step\",
					]

					postconditions: [
						\"Step is marked as assigned to target actor\",
						\"Target actor receives notification\",
					]

					verifications: [
						{
							description: \"Assignment succeeds\"
							criteria: [
								\"Assignment is recorded\",
								\"Notification is sent\",
								\"Actor can see assigned step in their queue\",
							]
						},
					]

					tags: [\"assignment\"]
				},
			]
		},
	]

	invariants: [
		{
			name:        \"audit-log\"
			description: \"All workflow changes are logged with full context\"
			criteria: [
				\"State transitions record who, when, and why\",
				\"Assignments record assigner and assignee\",
				\"Data changes record old and new values\",
			]
		},
		{
			name:        \"authorization\"
			description: \"Only authorized actors can perform actions\"
			criteria: [
				\"Each state transition defines allowed roles\",
				\"Authorization checks happen before state changes\",
				\"Cross-tenant isolation is enforced\",
			]
		},
		{
			name:        \"no-orphaned-states\"
			description: \"Every state can reach a terminal state\"
			criteria: [
				\"All states have defined transitions\",
				\"No state is a dead end (except terminal states)\",
				\"Workflow can complete or be cancelled from any state\",
			]
		},
	]

	anti_patterns: [
		{
			name:        \"missing-timeouts\"
			description: \"Don't allow workflow steps to hang indefinitely\"

			bad_example: {
				assignment: \"Step assigned with no timeout\"
			}

			good_example: {
				assignment: \"Step assigned with 48-hour timeout to escalation\"
			}

			why: \"Without timeouts, workflows stall when people are unavailable\"
		},
		{
			name:        \"implicit-transitions\"
			description: \"Don't allow implicit or side-effect transitions\"

			bad_example: {
				transition: \"State changes when unrelated data is modified\"
			}

			good_example: {
				transition: \"State changes only via explicit transition API call\"
			}

			why: \"Implicit transitions make workflows unpredictable and hard to debug\"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: [
				\"Workflow engine (Temporal, Camunda, Airflow)\",
				\"State machine with clear transition rules\",
				\"Message queue for notifications (SQS, Kafka)\",
				\"Audit log storage (database table with append-only writes)\",
			]
		}

		entities: {
			\"WorkflowInstance\": {
				fields: {
					\"id\": \"Unique workflow instance identifier\"
					\"current_state\": \"Current state in the workflow\"
					\"started_at\": \"Workflow creation timestamp\"
					\"completed_at\": \"Workflow completion timestamp (null if active)\"
					\"initiated_by\": \"User or system that started the workflow\"
				}
			}
			\"StateTransition\": {
				fields: {
					\"from_state\": \"Previous state\"
					\"to_state\": \"New state\"
					\"actor\": \"User or system that performed the transition\"
					\"timestamp\": \"When the transition occurred\"
					\"reason\": \"Optional reason or comment\"
				}
			}
		}

		security: {
			password_hashing: \"N/A (use central auth service)\"
			jwt_algorithm:    \"N/A\"
			jwt_expiry:       \"N/A\"
			rate_limiting:    \"N/A\"
		}

		pitfalls: [
			\"Don't allow direct database updates - use transition API\",
			\"Avoid sending notifications during database transactions\",
			\"Never skip authorization checks for 'system' users\",
			\"Don't assume actors are still active - check permissions at runtime\",
		]
	}
}
"
}

/// Format template type for display
pub fn format_template_type(template: TemplateType) -> String {
  case template {
    ApiSpec -> "api-spec"
    CliTool -> "cli-tool"
    DataPipeline -> "data-pipeline"
    Workflow -> "workflow"
  }
}

/// Get template description
pub fn get_template_description(template: TemplateType) -> String {
  case template {
    ApiSpec ->
      "REST/GraphQL API with endpoints, authentication, and data models"
    CliTool ->
      "Command-line tool with commands, arguments, and exit codes"
    DataPipeline ->
      "Data processing pipeline with sources, transforms, and destinations"
    Workflow ->
      "Business workflow with states, transitions, and actors"
  }
}

/// Validate and sanitize spec name
pub fn validate_spec_name(name: String) -> Result(String, String) {
  let trimmed = string.trim(name)

  case string.length(trimmed) {
    0 -> Error("Spec name cannot be empty")
    _ -> {
      // Check for valid characters (letters, numbers, spaces, hyphens, apostrophes)
      let has_invalid_chars =
        string.contains(trimmed, "\"")
        || string.contains(trimmed, "\\n")
        || string.contains(trimmed, "\\t")

      case has_invalid_chars {
        True ->
          Error(
            "Spec name contains invalid characters. Use letters, numbers, spaces, hyphens, and apostrophes.",
          )
        False -> Ok(trimmed)
      }
    }
  }
}

/// Generate package name from spec name
pub fn generate_package_name(spec_name: String) -> String {
  spec_name
  |> string.lowercase
  |> string.replace(" ", "_")
  |> string.replace("-", "_")
  |> string.replace("'", "")
}
