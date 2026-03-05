package user_api

import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "User Management API"

	description: """
		This API manages user accounts. Users can register with an email
		and password, log in to receive a JWT token, and manage their
		profile. Passwords must never appear in any API response.
		"""

	audience: "Mobile and web clients"

	version: "1.0.0"

	success_criteria: [
		"Users can register, login, and manage their profile",
		"Authentication uses JWT tokens",
		"Passwords are never exposed in responses",
		"All errors return structured error objects",
	]

	features: [
		{
			name: "User Registration"

			description: """
				New users register with email and password. The system
				validates the email format and password strength, creates
				the account, and returns the new user (without password).
				"""

			behaviors: [
				{
					name:   "successful-registration"
					intent: "A new user can create an account with valid email and password"

					preconditions: [
						"Email address is not already registered",
						"Password meets security requirements (8+ chars, uppercase, number, special)",
						"User provides name, email, and password",
					]

					postconditions: [
						"User account is created in the system",
						"User is assigned a unique ID with 'usr_' prefix",
						"Email is stored exactly as provided",
						"Password is hashed and stored (never plaintext)",
						"Creation timestamp is recorded",
					]

					verifications: [
						{
							description: "User object returned with correct fields"
							criteria: [
								"ID matches format usr_[a-z0-9]+",
								"Email matches the provided email",
								"Name matches the provided name",
								"Password field is NOT in response",
								"created_at is valid ISO8601 datetime",
							]
							examples: [
								{
									input: {
										email:    "newuser@example.com"
										password: "SecurePass123!"
										name:     "New User"
									}
									output: {
										id:         "usr_abc123xyz"
										email:      "newuser@example.com"
										name:       "New User"
										created_at: "2024-01-15T10:30:00Z"
									}
								}
							]
						}
					]
				},
				{
					name:   "duplicate-email-rejected"
					intent: "Cannot register with an email that's already taken"

					requires: ["successful-registration"]

					preconditions: [
						"User with email newuser@example.com already exists",
						"Different password provided for same email",
					]

					postconditions: [
						"Registration is rejected",
						"No new account is created",
						"Original account remains unchanged",
					]

					verifications: [
						{
							description: "Duplicate email returns specific error"
							criteria: [
								"Error code is EMAIL_EXISTS",
								"Error message is human-readable",
								"HTTP status code indicates conflict",
							]
							examples: [
								{
									input: {
										email:    "newuser@example.com"
										password: "DifferentPass456!"
									}
									output: {
										error: {
											code:    "EMAIL_EXISTS"
											message: "An account with this email already exists"
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "invalid-email-rejected"
					intent: "Email format is validated"

					preconditions: [
						"Email provided is not a valid format",
					]

					postconditions: [
						"Registration is rejected",
						"No account is created",
					]

					verifications: [
						{
							description: "Invalid email returns validation error"
							criteria: [
								"Error code is INVALID_EMAIL",
								"Error message explains the issue",
							]
						}
					]
				},
				{
					name:   "weak-password-rejected"
					intent: "Password must meet strength requirements"

					preconditions: [
						"Email is valid",
						"Password does not meet strength requirements",
					]

					postconditions: [
						"Registration is rejected",
						"No account is created",
						"User is informed of password requirements",
					]

					verifications: [
						{
							description: "Weak password returns specific error"
							criteria: [
								"Error code is WEAK_PASSWORD",
								"Error message may include password requirements",
							]
							examples: [
								{
									input: {
										email:    "another@example.com"
										password: "weak"
									}
									output: {
										error: {
											code: "WEAK_PASSWORD"
										}
									}
								}
							]
						}
					]

					notes: """
						Password requirements:
						- At least 8 characters
						- At least one uppercase letter
						- At least one number
						- At least one special character
						"""
				},
			]
		},
		{
			name: "Authentication"

			description: """
				Users authenticate with email/password and receive a JWT
				token. The token is used for subsequent requests.
				"""

			behaviors: [
				{
					name:   "successful-login"
					intent: "Valid credentials return a JWT token"

					requires: ["successful-registration"]

					preconditions: [
						"User account exists with email newuser@example.com",
						"User knows correct password",
					]

					postconditions: [
						"User is authenticated",
						"JWT token is generated",
						"Token type is Bearer",
						"Token expiration is at least 1 hour",
						"Refresh token is available",
					]

					verifications: [
						{
							description: "Token response contains all required fields"
							criteria: [
								"Token is valid JWT format",
								"Token type is 'Bearer'",
								"Expiration time is >= 3600 seconds",
								"Refresh token is present",
							]
							examples: [
								{
									input: {
										email:    "newuser@example.com"
										password: "SecurePass123!"
									}
									output: {
										token:         "eyJhbGciOiJIUzI1NiIs..."
										token_type:    "Bearer"
										expires_in:    3600
										refresh_token: "dGhpcyBpcyBhIHJlZnJl..."
									}
								}
							]
						}
					]
				},
				{
					name:   "wrong-password-rejected"
					intent: "Invalid password returns authentication error"

					requires: ["successful-registration"]

					preconditions: [
						"User account exists",
						"User provides incorrect password",
					]

					postconditions: [
						"Authentication fails",
						"No token is generated",
						"Error message does not reveal if email exists",
					]

					verifications: [
						{
							description: "Wrong password returns generic error"
							criteria: [
								"Error code is INVALID_CREDENTIALS (not WRONG_PASSWORD)",
								"Error message is generic",
							]
						}
					]

					notes: """
						Error message must NOT reveal whether email exists.
						Always return generic INVALID_CREDENTIALS, never
						EMAIL_NOT_FOUND or WRONG_PASSWORD separately.
						"""
				},
				{
					name:   "unknown-email-rejected"
					intent: "Unknown email returns same error as wrong password"

					preconditions: [
						"No account exists with provided email",
					]

					postconditions: [
						"Authentication fails",
						"No token is generated",
						"Error is identical to wrong password error",
					]

					verifications: [
						{
							description: "Unknown email returns same generic error"
							criteria: [
								"Error code is INVALID_CREDENTIALS",
								"Error matches wrong-password response format",
							]
						}
					]
				},
			]
		},
		{
			name: "Profile Management"

			description: """
				Authenticated users can read and update their profile.
				"""

			behaviors: [
				{
					name:   "get-own-profile"
					intent: "User can retrieve their own profile"

					requires: ["successful-login"]

					preconditions: [
						"User is authenticated with valid token",
						"User ID exists in system",
					]

					postconditions: [
						"User profile data is returned",
						"All profile fields are present",
					]

					verifications: [
						{
							description: "Profile data matches user"
							criteria: [
								"ID matches authenticated user",
								"Email is correct",
								"Name is correct",
							]
						}
					]
				},
				{
					name:   "update-profile"
					intent: "User can update their name"

					requires: ["get-own-profile"]

					preconditions: [
						"User is authenticated",
						"User provides new name value",
					]

					postconditions: [
						"Name is updated in database",
						"Updated timestamp is recorded",
					]

					verifications: [
						{
							description: "Profile update is reflected in response"
							criteria: [
								"Name matches new value",
								"updated_at is recent timestamp",
							]
						}
					]
				},
				{
					name:   "unauthenticated-access-denied"
					intent: "Cannot access profile without token"

					preconditions: [
						"No authentication token provided",
						"User attempts to access profile endpoint",
					]

					postconditions: [
						"Access is denied",
						"No profile data is returned",
					]

					verifications: [
						{
							description: "Missing auth returns unauthorized error"
							criteria: [
								"Error code is UNAUTHORIZED",
								"No profile data is exposed",
							]
						}
					]
				},
			]
		},
	]

	invariants: [
		{
			name: "no-sensitive-data-in-responses"
			description: "Passwords and secrets must never appear in any response"
			criteria: [
				"password field is absent from all user responses",
				"password_hash field is absent from all user responses",
				"secret field is absent from all user responses",
				"api_key field is absent from all user responses",
				"private_key field is absent from all user responses",
			]
		},
		{
			name: "structured-errors"
			description: "All error responses have consistent structure"
			criteria: [
				"Error responses include error.code field",
				"Error responses include error.message field",
				"Error codes are from a predefined set",
			]
		},
		{
			name: "content-type-header"
			description: "All responses declare content type"
			criteria: [
				"Content-Type header is present in all responses",
			]
		},
	]

	anti_patterns: [
		{
			name: "password-in-response"
			description: "NEVER return password in any response"

			bad_example: {
				user: {
					id:       "usr_123"
					email:    "user@example.com"
					password: "secret123"
				}
			}

			good_example: {
				user: {
					id:    "usr_123"
					email: "user@example.com"
				}
			}
		},
		{
			name: "user-enumeration"
			description: "Login errors must not reveal if email exists"

			bad_example: {
				error: {
					code:    "EMAIL_NOT_FOUND"
					message: "No account with this email"
				}
			}

			good_example: {
				error: {
					code:    "INVALID_CREDENTIALS"
					message: "Invalid email or password"
				}
			}
		},
		{
			name: "plain-text-ids"
			description: "IDs should not be sequential integers"

			bad_example: {
				id: 1
			}

			good_example: {
				id: "usr_x7k9m2p4q"
			}

			why: """
				Sequential IDs reveal business metrics and enable enumeration
				attacks. Use prefixed random strings instead.
				"""
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Node.js", "Express", "PostgreSQL"]
		}

		entities: {
			user: {
				fields: {
					id:         "string, prefixed 'usr_', randomly generated"
					email:      "string, unique, validated format"
					password:   "string, hashed with bcrypt, NEVER returned"
					name:       "string, 1-100 chars"
					created_at: "datetime, set on creation"
					updated_at: "datetime, set on every update"
				}
			}
		}

		security: {
			password_hashing: "bcrypt with cost factor >= 10"
			jwt_algorithm:    "HS256 or RS256"
			jwt_expiry:       "1 hour minimum"
			rate_limiting:    "100 requests per minute per IP"
		}

		pitfalls: [
			"Don't return password field even if it's hashed",
			"Don't use sequential integer IDs",
			"Don't reveal whether email exists in login errors",
			"Don't forget to validate email format",
			"Don't allow empty passwords",
		]
	}
}
