package conflicts_gaps

import "github.com/intent-cli/intent/schema:intent"

// Example: Conflict and Gap Detection
// Demonstrates common requirement conflicts and specification gaps
// that Intent's interview system helps identify and resolve

spec: intent.#Spec & {
	name: "Multi-Tenant SaaS API"

	description: """
		A multi-tenant SaaS platform API that demonstrates common conflicts
		between different stakeholder perspectives and gaps in requirements.

		This spec documents RESOLVED conflicts and gaps as a reference for
		how Intent helps discover and address these issues during interviews.

		# Conflicts Found and Resolved

		1. TENANT ISOLATION (Security vs Performance)
		   - Security: Complete data isolation, separate databases
		   - Performance: Shared database with row-level security
		   - Resolution: Shared DB with encryption per tenant

		2. API RATE LIMITS (Business vs Engineering)
		   - Business: Unlimited for enterprise tier
		   - Engineering: Must have limits to protect infrastructure
		   - Resolution: High limits (10k/min) for enterprise, with burst

		3. DATA RETENTION (Legal vs Cost)
		   - Legal: Keep everything for 7 years
		   - Cost: Storage is expensive at scale
		   - Resolution: 7 years cold storage, 90 days hot

		4. ERROR VERBOSITY (Security vs Developer Experience)
		   - Security: Minimal error details to prevent info leaks
		   - DX: Detailed errors help debugging
		   - Resolution: Verbose in dev/staging, minimal in production

		# Gaps Identified and Filled

		1. Tenant onboarding flow (who creates first admin?)
		2. Cross-tenant data sharing (is it ever allowed?)
		3. Tenant deletion and data export requirements
		4. Audit log access (who can see what?)
		"""

	audience: """
		Primary: Enterprise customers managing their own tenant data
		Secondary: Tenant administrators configuring their organization
		Tertiary: Platform operators managing the multi-tenant system
		"""

	version: "2.0.0"

	success_criteria: [
		"Tenants are completely isolated from each other",
		"Each tenant can customize their configuration",
		"Platform admins can manage all tenants",
		"Audit logs track all sensitive operations",
		"Data export available for compliance",
	]

	features: [
		{
			name: "Tenant Isolation"

			description: """
				Core tenant isolation demonstrating the RESOLVED conflict
				between complete isolation (security) and shared resources
				(performance/cost).

				CONFLICT: Security wanted separate databases per tenant.
				          Operations said this doesn't scale past 100 tenants.

				RESOLUTION: Shared database with:
				- Row-level security (RLS) policies
				- Encrypted tenant data with per-tenant keys
				- Query-level tenant ID enforcement
				"""

			behaviors: [
				{
					name:   "tenant-data-isolation"
					intent: "User can only access data from their own tenant"

					preconditions: [
						"User is authenticated",
						"User belongs to a specific tenant",
						"Tenant ID is in JWT token",
					]

					postconditions: [
						"Only data from user's tenant is returned",
						"Response includes tenant context for verification",
						"All returned items belong to requesting tenant",
					]

					verifications: [
						{
							description: "Tenant isolation is enforced"
							criteria: [
								"Response meta.tenant_id matches requesting tenant",
								"All returned items belong to requesting tenant",
							]
							examples: [
								{
									output: {
										users: [
											{
												id:        "usr_abc123"
												tenant_id: "tenant_acme"
												email:     "alice@acme.com"
												role:      "admin"
											},
											{
												id:        "usr_def456"
												tenant_id: "tenant_acme"
												email:     "bob@acme.com"
												role:      "member"
											},
										]
										meta: {
											tenant_id: "tenant_acme"
											total:     2
										}
									}
								}
							]
						}
					]

					notes: """
						CONFLICT RESOLUTION:
						- Every query includes tenant_id filter (auto-added by middleware)
						- RLS policies prevent cross-tenant access at DB level
						- Tenant ID in JWT is source of truth, not request headers
						"""
				},
				{
					name:   "cross-tenant-access-blocked"
					intent: "Attempting to access another tenant's data returns 404"

					preconditions: [
						"User is authenticated",
						"User attempts to access resource from different tenant",
					]

					postconditions: [
						"Request is rejected with 404",
						"Error message does not reveal resource exists",
						"No tenant information is leaked",
					]

					verifications: [
						{
							description: "Cross-tenant access returns 404"
							criteria: [
								"Error code is NOT_FOUND",
								"Error message is generic",
								"No tenant information in error",
							]
							examples: [
								{
									output: {
										error: {
											code:    "NOT_FOUND"
											message: "User not found"
										}
									}
								}
							]
						}
					]

					notes: """
						GAP IDENTIFIED: Original spec didn't specify what happens when
						a user tries to access another tenant's resources.

						Options considered:
						1. Return 403 Forbidden - reveals resource exists
						2. Return 404 Not Found - no information leakage
						3. Return 400 Bad Request - confusing

						RESOLUTION: Return 404 to prevent tenant enumeration attacks.
						"""
				},
			]
		},
		{
			name: "Rate Limiting"

			description: """
				API rate limiting demonstrating the RESOLVED conflict between
				Business (wanting unlimited for enterprise) and Engineering
				(needing infrastructure protection).

				CONFLICT: Sales promised "unlimited API calls" to enterprise.
				          Platform team said unlimited will cause outages.

				RESOLUTION:
				- Free tier: 100 requests/minute
				- Pro tier: 1,000 requests/minute
				- Enterprise: 10,000 requests/minute with burst to 15,000
				- All tiers get clear headers showing usage
				"""

			behaviors: [
				{
					name:   "rate-limit-headers"
					intent: "Every response includes rate limit information"

					preconditions: [
						"User makes any API request",
					]

					postconditions: [
						"Response includes rate limit headers",
						"Headers show limit, remaining, and reset time",
					]

					verifications: [
						{
							description: "Rate limit headers present"
							criteria: [
								"X-RateLimit-Limit header present",
								"X-RateLimit-Remaining header present",
								"X-RateLimit-Reset header present",
							]
							examples: [
								{
									headers: {
										"X-RateLimit-Limit":     "10000"
										"X-RateLimit-Remaining": "9995"
										"X-RateLimit-Reset":     "1705326000"
									}
									output: {
										status: "ok"
									}
								}
							]
						}
					]

					notes: """
						CONFLICT RESOLUTION: Instead of "unlimited", enterprise gets
						very high limits plus burst capacity. All tiers get transparent
						headers so clients can self-throttle.
						"""
				},
				{
					name:   "rate-limit-exceeded"
					intent: "Exceeding rate limit returns 429 with retry info"

					preconditions: [
						"User exceeds rate limit for their tier",
					]

					postconditions: [
						"Request is rejected with 429 status",
						"Response includes retry-after information",
						"Error shows current tier and limits",
					]

					verifications: [
						{
							description: "Rate limit exceeded returns helpful error"
							criteria: [
								"Error code is RATE_LIMITED",
								"Error includes retry_after (integer >= 1)",
								"Error shows current tier",
								"Retry-After header present",
							]
							examples: [
								{
									headers: {
										"Retry-After":           "30"
										"X-RateLimit-Limit":     "10000"
										"X-RateLimit-Remaining": "0"
									}
									output: {
										error: {
											code:        "RATE_LIMITED"
											message:     "Rate limit exceeded. Please retry after 30 seconds."
											retry_after: 30
											limit:       10000
											tier:        "enterprise"
											upgrade_url: null
										}
									}
								}
							]
						}
					]

					notes: """
						GAP IDENTIFIED: Original spec didn't define what happens
						when rate limit is exceeded.

						RESOLUTION: Return 429 with Retry-After header and helpful
						message. Log for abuse detection but don't immediately block.
						"""
				},
			]
		},
		{
			name: "Data Retention"

			description: """
				Data retention policies demonstrating RESOLVED conflict between
				Legal (keep everything) and Cost (minimize storage).

				CONFLICT: Legal required 7-year retention for audit.
				          Finance said storing 7 years of data is too expensive.

				RESOLUTION: Tiered storage
				- Hot: Last 90 days, fast queries
				- Warm: 90 days to 1 year, slower queries
				- Cold: 1-7 years, archived, retrieval takes hours
				- Deleted: After 7 years, permanently removed
				"""

			behaviors: [
				{
					name:   "query-recent-data"
					intent: "Recent data (hot tier) returns immediately"

					preconditions: [
						"User requests data from last 90 days",
						"User is authenticated admin",
					]

					postconditions: [
						"Data is returned immediately",
						"Storage tier is indicated as 'hot'",
						"Query time is under 100ms",
					]

					verifications: [
						{
							description: "Recent data query succeeds"
							criteria: [
								"Storage tier is 'hot'",
								"Logs array is non-empty",
								"All logs belong to requesting tenant",
								"Query time is reasonable",
							]
							examples: [
								{
									output: {
										logs: [
											{
												id:         "log_abc123"
												timestamp:  "2024-01-15T10:30:00Z"
												actor_id:   "usr_admin"
												action:     "user.created"
												resource:   "usr_new123"
												tenant_id:  "tenant_acme"
												ip_address: "192.168.1.1"
											}
										]
										meta: {
											storage_tier: "hot"
											query_time:   "45ms"
											total:        1250
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "query-archived-data"
					intent: "Old data (cold tier) requires async retrieval"

					preconditions: [
						"User requests data older than 1 year",
						"User is authenticated admin",
					]

					postconditions: [
						"Async retrieval job is created",
						"Job ID is returned for polling",
						"Data will be available for 24 hours",
					]

					verifications: [
						{
							description: "Archived data query creates job"
							criteria: [
								"Job ID starts with 'job_'",
								"Status is 'pending'",
								"Storage tier is 'cold'",
								"Estimated time is provided",
								"Expiration timestamp is provided",
							]
							examples: [
								{
									output: {
										job_id:              "job_archive_xyz789"
										status:              "pending"
										estimated_time:      "2-4 hours"
										storage_tier:        "cold"
										notification_email:  "admin@acme.com"
										expires_at:          "2024-01-17T10:30:00Z"
									}
								}
							]
						}
					]

					notes: """
						GAP IDENTIFIED: How do users access 5-year-old audit data?

						RESOLUTION: Async retrieval with job tracking
						1. User requests data range
						2. System returns job ID
						3. User polls for completion
						4. Data available for 24 hours once retrieved
						"""
				},
			]
		},
		{
			name: "Error Handling"

			description: """
				Error response verbosity demonstrating RESOLVED conflict between
				Security (minimal info) and Developer Experience (detailed errors).

				CONFLICT: Security team wanted errors like "An error occurred"
				          Developers complained they can't debug integrations

				RESOLUTION: Environment-aware error responses
				- Production: Minimal errors with request ID for support
				- Staging/Dev: Detailed errors with stack traces
				- All environments: Structured error codes for programmatic handling
				"""

			behaviors: [
				{
					name:   "production-error-minimal"
					intent: "Production errors are minimal but trackable"

					preconditions: [
						"Environment is production",
						"Request contains invalid data",
					]

					postconditions: [
						"Error is returned with structured code",
						"Request ID is provided for support",
						"No stack traces or internal details exposed",
					]

					verifications: [
						{
							description: "Production error is minimal"
							criteria: [
								"Error code is non-empty string",
								"Error message is generic but helpful",
								"Request ID starts with 'req_'",
								"No stack trace in response",
								"No SQL queries in response",
							]
							examples: [
								{
									headers: {
										"X-Environment": "production"
									}
									input: {
										email: "invalid-email"
									}
									output: {
										error: {
											code:       "VALIDATION_ERROR"
											message:    "The request contains invalid data"
											request_id: "req_abc123xyz"
											docs_url:   "https://docs.example.com/errors/VALIDATION_ERROR"
										}
									}
								}
							]
						}
					]

					notes: """
						CONFLICT RESOLUTION: Production errors include:
						- Structured error code (for programmatic handling)
						- Human message (generic but helpful)
						- Request ID (for support ticket correlation)
						- NO stack traces, internal details, or query info
						"""
				},
				{
					name:   "development-error-verbose"
					intent: "Development errors include debugging details"

					preconditions: [
						"Environment is development or staging",
						"Request contains invalid data",
					]

					postconditions: [
						"Error includes field-level details",
						"Error includes internal context",
						"Error may include suggestions",
					]

					verifications: [
						{
							description: "Development error is verbose"
							criteria: [
								"Error code matches production",
								"Error details are present",
								"Field-level errors included",
								"Suggestions may be present",
							]
							examples: [
								{
									headers: {
										"X-Environment": "development"
									}
									input: {
										email: "invalid-email"
									}
									output: {
										error: {
											code:       "VALIDATION_ERROR"
											message:    "The request contains invalid data"
											request_id: "req_dev123xyz"
											details: {
												fields: {
													email: {
														value:      "invalid-email"
														constraint: "Must be a valid email address"
														pattern:    "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
													}
												}
											}
											suggestion: "Check the email field format"
										}
									}
								}
							]
						}
					]

					notes: """
						For non-production environments, include:
						- Field-level validation errors
						- Internal error context
						- Suggested fixes
						Still no stack traces in API responses (use logs)
						"""
				},
			]
		},
		{
			name: "Tenant Lifecycle"

			description: """
				Tenant onboarding and offboarding - major GAPS identified
				during interview process.

				GAPS IDENTIFIED:
				1. Who creates the first admin user for a new tenant?
				2. What happens to data when a tenant is deleted?
				3. How does data export work for compliance?
				4. Can a deleted tenant be restored?
				"""

			behaviors: [
				{
					name:   "create-tenant"
					intent: "Platform admin creates new tenant with initial admin"

					preconditions: [
						"User is authenticated as platform admin",
						"Tenant name and slug are provided",
						"Admin email is provided",
					]

					postconditions: [
						"Tenant is created with unique ID",
						"Tenant status is 'provisioning'",
						"Invitation email is sent to admin",
						"Invitation expires in 7 days",
					]

					verifications: [
						{
							description: "Tenant created successfully"
							criteria: [
								"Tenant ID starts with 'tenant_'",
								"Status is 'provisioning'",
								"Admin invitation_sent is true",
								"Invitation expires_at is valid ISO8601 datetime",
							]
							examples: [
								{
									input: {
										name:        "Acme Corp"
										slug:        "acme"
										tier:        "enterprise"
										admin_email: "admin@acme.com"
									}
									output: {
										id:          "tenant_acme"
										name:        "Acme Corp"
										slug:        "acme"
										tier:        "enterprise"
										status:      "provisioning"
										admin: {
											email:             "admin@acme.com"
											invitation_sent:   true
											invitation_expires: "2024-01-22T10:30:00Z"
										}
										created_at: "2024-01-15T10:30:00Z"
									}
								}
							]
						}
					]

					notes: """
						GAP RESOLVED: "Who creates the first admin?"

						Answer: Platform admin creates tenant with initial admin email.
						System sends invitation to that email to set password.
						This solves chicken-and-egg of no users in empty tenant.
						"""
				},
				{
					name:   "delete-tenant-request"
					intent: "Tenant deletion is a controlled process with data export"

					preconditions: [
						"User is authenticated as tenant admin",
						"Tenant exists",
					]

					postconditions: [
						"Tenant status changes to 'pending_deletion'",
						"30-day grace period starts",
						"Data export job is created automatically",
						"Grace period allows cancellation",
					]

					verifications: [
						{
							description: "Tenant deletion initiated"
							criteria: [
								"Status is 'pending_deletion'",
								"Grace period is >= 30 days",
								"Can cancel is true",
								"Export job is created",
								"Export status is generating, ready, or failed",
							]
							examples: [
								{
									input: {
										confirm:     true
										reason:      "Switching to competitor"
										export_data: true
									}
									output: {
										tenant_id:      "tenant_acme"
										status:         "pending_deletion"
										deletion_date:  "2024-02-15T10:30:00Z"
										grace_period:   30
										can_cancel:     true
										export: {
											job_id:     "export_xyz789"
											status:     "generating"
											format:     "zip"
											includes:   ["users", "data", "audit_logs", "config"]
											expires_at: "2024-01-22T10:30:00Z"
										}
									}
								}
							]
						}
					]

					notes: """
						GAP RESOLVED: "What happens when a tenant is deleted?"

						Answer: Multi-step process:
						1. Request deletion (starts 30-day countdown)
						2. Data export generated automatically
						3. Tenant marked as "pending_deletion"
						4. Can cancel during 30-day grace period
						5. After 30 days, data permanently deleted
						"""
				},
				{
					name:   "cancel-deletion"
					intent: "Tenant can cancel deletion during grace period"

					preconditions: [
						"User is authenticated as tenant admin",
						"Tenant is in 'pending_deletion' status",
						"Grace period has not expired",
					]

					postconditions: [
						"Tenant status changes back to 'active'",
						"Deletion is cancelled",
						"Data is preserved",
					]

					verifications: [
						{
							description: "Deletion cancelled successfully"
							criteria: [
								"Status is 'active'",
							]
							examples: [
								{
									output: {
										tenant_id: "tenant_acme"
										status:    "active"
										message:   "Deletion cancelled. Tenant restored to active status."
									}
								}
							]
						}
					]

					notes: """
						GAP RESOLVED: "Can a deleted tenant be restored?"

						Answer: Yes, during the 30-day grace period.
						After that, data is permanently gone per retention policy.
						"""
				},
			]
		},
		{
			name: "Cross-Tenant Sharing"

			description: """
				Data sharing between tenants - GAP that revealed complex
				requirements during interview.

				GAP IDENTIFIED: "Is cross-tenant data sharing ever allowed?"

				Answer: Yes, in specific controlled scenarios:
				1. Tenant A explicitly shares a resource with Tenant B
				2. Sharing is read-only by default
				3. Both tenant admins must approve
				4. Audit log tracks all cross-tenant access
				5. Sharing can be revoked anytime
				"""

			behaviors: [
				{
					name:   "create-share-link"
					intent: "Tenant admin creates a sharing link for a resource"

					preconditions: [
						"User is authenticated as tenant admin",
						"Resource exists and belongs to user's tenant",
						"Target tenant exists",
					]

					postconditions: [
						"Share link is created",
						"Share is in 'pending_acceptance' status",
						"Share has expiration date",
						"Share URL is generated",
					]

					verifications: [
						{
							description: "Share link created successfully"
							criteria: [
								"Share ID is present",
								"Status is 'pending_acceptance'",
								"Permission is 'read' or 'read_write'",
								"expires_at is valid ISO8601 datetime",
								"Share URL is present",
							]
							examples: [
								{
									input: {
										target_tenant: "tenant_partner"
										permission:    "read"
										expires_in:    "7d"
									}
									output: {
										share_id:       "share_xyz789"
										resource_id:    "res_abc123"
										source_tenant:  "tenant_acme"
										target_tenant:  "tenant_partner"
										permission:     "read"
										status:         "pending_acceptance"
										expires_at:     "2024-01-22T10:30:00Z"
										share_url:      "https://app.example.com/shared/share_xyz789"
									}
								}
							]
						}
					]
				},
				{
					name:   "access-shared-resource"
					intent: "Target tenant accesses shared resource"

					preconditions: [
						"User is authenticated",
						"User belongs to target tenant",
						"Share exists and is accepted",
						"Share has not expired",
					]

					postconditions: [
						"Resource data is returned",
						"Share metadata is included",
						"Access is logged in both tenants' audit logs",
					]

					verifications: [
						{
							description: "Shared resource accessed successfully"
							criteria: [
								"Resource data is present",
								"Share metadata shows source tenant",
								"Share metadata shows permission level",
								"Share metadata includes expiration",
							]
							examples: [
								{
									output: {
										resource: {
											id:   "res_abc123"
											name: "Shared Document"
											data: {}
										}
										share_meta: {
											source_tenant: "tenant_acme"
											permission:    "read"
											expires_at:    "2024-01-22T10:30:00Z"
											accessed_via:  "share_xyz789"
										}
									}
								}
							]
						}
					]

					notes: """
						Cross-tenant access is:
						- Logged in both tenants' audit logs
						- Subject to target tenant's rate limits
						- Revocable by source tenant at any time
						"""
				},
			]
		},
	]

	invariants: [
		{
			name: "tenant-context-required"
			description: "All data responses must include tenant context"
			criteria: [
				"Data responses include tenant_id field",
				"Tenant ID matches authenticated user's tenant",
			]
		},
		{
			name: "no-cross-tenant-data"
			description: "Regular API calls cannot access other tenants"
			criteria: [
				"Responses do not contain data from other tenants",
				"Responses do not expose cross-tenant access capabilities",
			]
		},
	]

	anti_patterns: [
		{
			name: "tenant-in-url-only"
			description: "Don't rely on URL for tenant identification"

			bad_example: {
				url: "/tenants/acme/users"
			}

			good_example: {
				url:     "/users"
				headers: {"X-Tenant-ID": "from_jwt"}
			}

			why: """
				Tenant ID in URL can be manipulated. Extract tenant from
				authenticated JWT token instead.
				"""
		},
		{
			name: "global-admin-bypass"
			description: "Don't create god-mode admin that bypasses isolation"

			bad_example: {
				role: "super_admin"
				can_access: "all_tenants"
			}

			good_example: {
				role: "platform_admin"
				must_impersonate: true
				audit_logged: true
			}

			why: """
				Even platform admins should impersonate specific tenants
				with full audit logging, not have global access.
				"""
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Rust", "PostgreSQL with RLS", "Redis"]
		}

		entities: {
			tenant: {
				fields: {
					id:             "string, 'tenant_' + slug"
					name:           "string, display name"
					slug:           "string, URL-safe identifier"
					tier:           "enum: free, pro, enterprise"
					status:         "enum: provisioning, active, suspended, pending_deletion"
					encryption_key: "per-tenant encryption key, never exposed"
				}
			}
		}

		security: {
			password_hashing: "bcrypt with cost 12"
			jwt_algorithm:    "RS256"
			jwt_expiry:       "1 hour"
			rate_limiting:    "Tiered per plan: 100/1000/10000 req/min"
		}

		pitfalls: [
			"Never trust tenant ID from request headers alone",
			"Always validate JWT tenant claim matches request",
			"Log all cross-tenant access attempts",
			"Don't cache data without tenant context",
			"Rate limits must be per-tenant, not global",
			"Encryption keys must be per-tenant for true isolation",
		]
	}
}
