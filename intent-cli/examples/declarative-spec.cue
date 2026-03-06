package test_declarative

import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Declarative Specification Example"

	description: """
		A simple example using the new declarative schema.
		This demonstrates behaviors with preconditions, postconditions,
		and verifications instead of HTTP-specific fields.
		"""

	audience: "Developers testing the new schema"

	version: "1.0.0"

	success_criteria: [
		"Behaviors use declarative preconditions and postconditions",
		"Verifications specify how to validate behavior",
		"No HTTP-specific fields required",
	]

	features: [
		{
			name: "User Authentication"

			description: "User can authenticate with credentials"

			behaviors: [
				{
					name:   "successful-login"
					intent: "User can log in with valid credentials"

					preconditions: [
						"User account exists with email test@example.com",
						"User password is set to 'SecurePass123!'",
					]

					postconditions: [
						"User is authenticated",
						"Authentication token is generated",
						"Login timestamp is recorded",
					]

					verifications: [
						{
							description: "Authentication token is valid JWT"
							criteria: [
								"Token matches JWT format",
								"Token contains user ID in claims",
								"Token has valid expiration",
							]
						}
					]

					notes: "This is a declarative behavior - no HTTP details"
				},
				{
					name:   "failed-login-invalid-password"

					intent: "Login fails with incorrect password"

					requires: ["successful-login"]

					preconditions: [
						"User account exists",
						"User provides wrong password",
					]

					postconditions: [
						"Authentication fails",
						"Error message is returned",
						"No token is generated",
					]

					verifications: [
						{
							description: "Error response is structured"
							criteria: [
								"Error code is INVALID_CREDENTIALS",
								"Error message is user-friendly",
								"No sensitive information leaked",
							]
						}
					]

					tags: ["auth", "error-case"]
				},
			]
		},
	]

	invariants: [
		{
			name: "passwords-never-exposed"
			description: "Passwords must never appear in any response"
			criteria: [
				"password field is absent from all responses",
				"password_hash field is absent from all responses",
				"plaintext password is never logged",
			]
		},
		{
			name: "consistent-error-format"
			description: "All error responses follow the same structure"
			criteria: [
				"Error responses include error.code field",
				"Error responses include error.message field",
				"Error codes are from a defined set",
			]
		},
	]

	anti_patterns: [
		{
			name: "hardcoded-passwords"

			description: "Never hardcode passwords in configuration or code"

			bad_example: {
				config: {
					admin_password: "admin123"
				}
			}

			good_example: {
				config: {
					password_source: "environment_variable"
					password_env_var: "ADMIN_PASSWORD_HASH"
				}
			}

			why: "Hardcoded passwords in code are a security vulnerability"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Gleam", "Erlang/OTP", "PostgreSQL"]
		}

		entities: {
			"User": {
				fields: {
					id: "Unique user identifier"
					email: "User email address (unique)"
					password_hash: "Bcrypt hash of user password"
					created_at: "Account creation timestamp"
					updated_at: "Last update timestamp"
				}
			}
			"AuthenticationToken": {
				fields: {
					token_id: "Unique token identifier"
					user_id: "Reference to User entity"
					token_string: "JWT token string"
					expires_at: "Token expiration timestamp"
					created_at: "Token issuance timestamp"
				}
			}
		}

		security: {
			password_hashing: "bcrypt with cost factor 12"
			jwt_algorithm: "HS256"
			jwt_expiry: "24 hours"
			rate_limiting: "100 requests per minute per IP"
		}

		pitfalls: [
			"Don't store passwords in plaintext",
			"Don't expose JWT secrets in error messages",
			"Don't accept weak passwords during registration",
		]
	}
}
