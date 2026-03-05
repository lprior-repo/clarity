package nested_paths

import "github.com/intent-cli/intent/schema:intent"

// Example: Nested JSON Path Validation
// Demonstrates checking deeply nested fields like "user.profile.address.city"

spec: intent.#Spec & {
	name: "Customer Profile API"

	description: """
		A customer profile API with deeply nested data structures.
		Demonstrates validating nested paths like user.profile.address.city
		and array indexing like orders[0].items[0].product.
		"""

	audience: "CRM and support applications"

	version: "1.0.0"

	success_criteria: [
		"User profiles contain complete nested data",
		"Addresses are validated at all nesting levels",
		"Order history with nested items is accessible",
	]

	features: [
		{
			name: "Profile Management"

			description: """
				User profiles with nested personal info, addresses, and preferences.
				"""

			behaviors: [
				{
					name:   "get-full-profile"
					intent: "Retrieve complete user profile with all nested data"

					preconditions: [
						"User with ID usr_12345 exists in system",
						"Requester has valid authentication token",
					]

					postconditions: [
						"Complete user profile returned with all nested fields",
						"All nested validation rules pass",
					]

					verifications: [
						{
							description: "Verify complete profile structure with nested fields"
							criteria: [
								"User ID follows format usr_[a-z0-9]+",
								"Email is valid format",
								"First name and last name are non-empty strings",
								"Address contains city, state (2 uppercase), zip (US format), country",
								"Preferences include newsletter (boolean), language (2 char code), timezone",
							]
							examples: [
								{
									input: {
										user_id: "usr_12345"
										auth_token: "valid_token_abc"
									}
									output: {
										user: {
											id:    "usr_12345"
											email: "customer@example.com"
											profile: {
												first_name: "Jane"
												last_name:  "Smith"
												phone:      "+1-555-123-4567"
												address: {
													street:  "123 Main St"
													city:    "Seattle"
													state:   "WA"
													zip:     "98101"
													country: "US"
												}
												preferences: {
													newsletter: true
													language:   "en"
													timezone:   "America/Los_Angeles"
												}
											}
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "update-nested-address"
					intent: "Update specific nested address fields"

					requires: ["get-full-profile"]

					preconditions: [
						"User profile exists",
						"New address data is valid",
					]

					postconditions: [
						"Address fields updated with new values",
						"Other profile fields remain unchanged",
						"Country persists if not specified",
					]

					verifications: [
						{
							description: "Verify nested address update"
							criteria: [
								"Street updated to new value",
								"City updated to new value",
								"State updated to new value",
								"Country retains previous value when not specified",
							]
							examples: [
								{
									input: {
										user_id: "usr_12345"
										address: {
											street: "456 Oak Ave"
											city:   "Portland"
											state:  "OR"
											zip:    "97201"
										}
									}
									output: {
										user: {
											id: "usr_12345"
											profile: {
												address: {
													street:  "456 Oak Ave"
													city:    "Portland"
													state:   "OR"
													zip:     "97201"
													country: "US"
												}
											}
										}
									}
								}
							]
						}
					]
				},
			]
		},
		{
			name: "Order History"

			description: """
				Order history with nested items, products, and shipping info.
				"""

			behaviors: [
				{
					name:   "get-order-with-nested-items"
					intent: "Retrieve order with deeply nested product information"

					preconditions: [
						"Order ord_xyz789 exists",
						"Requester is authenticated",
					]

					postconditions: [
						"Complete order structure returned",
						"All nested relationships populated",
						"Arrays properly indexed",
					]

					verifications: [
						{
							description: "Verify deeply nested order structure"
							criteria: [
								"Order ID follows ord_[a-z0-9]+ format",
								"Status is valid enum value",
								"Items array is non-empty",
								"Item quantity is positive integer",
								"Product has nested category with parent",
								"Product pricing includes base, discount, final",
								"Shipping has nested tracking with events array",
								"Totals include subtotal, tax, shipping, total",
							]
							examples: [
								{
									input: {
										order_id: "ord_xyz789"
									}
									output: {
										order: {
											id:     "ord_xyz789"
											status: "shipped"
											items: [
												{
													quantity: 2
													product: {
														id:   "prod_abc"
														name: "Widget Pro"
														category: {
															id:   "cat_electronics"
															name: "Electronics"
															parent: {
																id:   "cat_all"
																name: "All Products"
															}
														}
														pricing: {
															base:     99.99
															discount: 10.00
															final:    89.99
														}
													}
												}
											]
											shipping: {
												carrier: "UPS"
												tracking: {
													number: "1Z999AA10123456784"
													url:    "https://ups.com/track/1Z999AA10123456784"
													events: [
														{
															timestamp: "2024-01-15T10:30:00Z"
															location:  "Seattle, WA"
															status:    "In Transit"
														}
													]
												}
												address: {
													recipient: "Jane Smith"
													street:    "123 Main St"
													city:      "Seattle"
													state:     "WA"
													zip:       "98101"
												}
											}
											totals: {
												subtotal: 179.98
												tax:      16.20
												shipping: 9.99
												total:    206.17
											}
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "list-orders-summary"
					intent: "List orders with minimal nested data"

					preconditions: [
						"User has existing orders",
						"Requester is authenticated",
					]

					postconditions: [
						"Paginated list of orders returned",
						"Each order has summary fields",
						"Pagination metadata included",
					]

					verifications: [
						{
							description: "Verify paginated order list"
							criteria: [
								"Orders is an array (may be empty)",
								"Order IDs follow standard format",
								"Order totals are reasonable values",
								"Pagination page >= 1",
								"Total pages >= 0",
							]
							examples: [
								{
									input: {
										user_id: "usr_12345"
										page: 1
										per_page: 20
									}
									output: {
										orders: [
											{
												id:         "ord_abc123"
												status:     "delivered"
												item_count: 3
												total:      150.00
												created_at: "2024-01-10T08:00:00Z"
											}
										]
										pagination: {
											page:        1
											per_page:    20
											total_pages: 5
											total_items: 87
										}
									}
								}
							]
						}
					]
				},
			]
		},
		{
			name: "Organization Hierarchy"

			description: """
				Organizational data with deep nesting for company structures.
				"""

			behaviors: [
				{
					name:   "get-org-structure"
					intent: "Retrieve deeply nested organizational hierarchy"

					preconditions: [
						"Organization org_acme exists",
						"Requester has permission to view org structure",
					]

					postconditions: [
						"Complete org hierarchy returned",
						"Departments contain teams",
						"Teams contain leads",
						"All nested levels properly populated",
					]

					verifications: [
						{
							description: "Verify deeply nested organization structure"
							criteria: [
								"Organization ID starts with org_",
								"Headquarters address country is ISO code",
								"Departments array is non-empty",
								"Department IDs start with dept_",
								"Department heads have employee IDs starting with emp_",
								"Team IDs start with team_",
								"Team leads have names",
							]
							examples: [
								{
									input: {
										org_id: "org_acme"
									}
									output: {
										organization: {
											id:   "org_acme"
											name: "Acme Corp"
											headquarters: {
												address: {
													city:    "San Francisco"
													country: "US"
												}
											}
											departments: [
												{
													id:   "dept_eng"
													name: "Engineering"
													head: {
														id:    "emp_001"
														name:  "Alice Johnson"
														title: "VP Engineering"
													}
													teams: [
														{
															id:   "team_backend"
															name: "Backend"
															lead: {
																id:   "emp_010"
																name: "Bob Wilson"
															}
														}
													]
												}
											]
										}
									}
								}
							]
						}
					]
				},
			]
		},
	]

	invariants: [
		{
			name:        "nested-id-consistency"
			description: "All IDs at any nesting level must be prefixed strings"

			criteria: [
								"Every id field is a string (not numeric)",
								"ID strings follow entity-specific prefixes",
							]
		},
	]

	anti_patterns: [
		{
			name:        "flat-data-model"
			description: "Don't flatten nested relationships into the parent"

			bad_example: {
				user_id:                 "usr_123"
				user_profile_first_name: "Jane"
				user_profile_addr_city:  "Seattle"
			}

			good_example: {
				user: {
					id: "usr_123"
					profile: {
						first_name: "Jane"
						address: {
							city: "Seattle"
						}
					}
				}
			}

			why: """
				Nested structure is more intuitive and allows for proper
				typing. Flat keys with underscores are hard to parse.
				"""
		},
		{
			name:        "deep-nesting-without-purpose"
			description: "Don't nest more than 4-5 levels without good reason"

			bad_example: {
				a: {b: {c: {d: {e: {f: {value: 1}}}}}}
			}

			good_example: {
				entity: {
					metadata: {
						nested_value: 1
					}
				}
			}

			why: """
				Excessive nesting makes APIs hard to consume and
				validation rules difficult to write. Keep it practical.
				"""
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Python", "FastAPI", "MongoDB"]
		}

		entities: {
			user: {
				fields: {
					id:      "string, prefixed 'usr_'"
					email:   "string, valid email"
					profile: "nested object with name, address, preferences"
				}
			}
			order: {
				fields: {
					id:       "string, prefixed 'ord_'"
					items:    "array of line items with nested products"
					shipping: "nested object with tracking and address"
					totals:   "nested object with subtotal, tax, shipping, total"
				}
			}
		}

		pitfalls: [
			"Validate at every nesting level, not just top-level",
			"Handle missing intermediate paths gracefully",
			"Don't assume array indexes exist",
			"Consider partial updates for deeply nested data",
		]
	}
}
