package interview_workflow

import "github.com/intent-cli/intent/schema:intent"

// Example: Interview-Driven API Design
// Demonstrates how the interview system helps discover requirements
// This spec shows the OUTPUT of an interview session - a complete specification

spec: intent.#Spec & {
	name: "Order Processing API"

	description: """
		Order processing system designed through Intent's interview workflow.
		This example shows how different perspectives (user, developer, ops,
		security, business) contribute to a complete specification.

		Interview Profile: API
		Rounds Completed: 5
		Total Questions Answered: 23
		Gaps Identified: 3 (all resolved)
		Conflicts Found: 2 (all resolved)
		"""

	audience: """
		Primary: E-commerce checkout flows (web and mobile)
		Secondary: Order management dashboard (internal)
		Tertiary: Third-party fulfillment integrations
		"""

	version: "1.2.0"

	success_criteria: [
		// From Round 1 - Happy Path Discovery
		"Customers can place orders with multiple items",
		"Orders transition through defined states correctly",
		"Payment processing integrates with payment gateway",

		// From Round 2 - Error Cases
		"Invalid orders rejected with clear error codes",
		"Payment failures handled gracefully with retry option",
		"Out-of-stock items prevented at checkout",

		// From Round 3 - Edge Cases
		"Concurrent order placement handled correctly",
		"Large orders (100+ items) supported",
		"International orders with currency conversion",

		// From Round 4 - Security & Compliance
		"PCI DSS compliant payment handling",
		"No credit card data stored in logs",

		// From Round 5 - Operations
		"99.9% uptime during peak hours",
		"Graceful degradation under load",
	]

	features: [
		{
			name: "Order Creation"

			description: """
				Create and validate orders. Discovered through User perspective
				questions about the happy path and error cases.
				"""

			behaviors: [
				{
					name:   "create-order-happy-path"
					intent: "Customer places a valid order with in-stock items"

					preconditions: [
						"User is authenticated with valid token",
						"All items in order are in stock",
						"Shipping address is valid",
						"Payment method is valid",
					]

					postconditions: [
						"Order is created with unique ID",
						"Order status is pending_payment",
						"Inventory is reserved for items",
						"Payment URL is generated",
						"Expiration timestamp is set (30 minutes)",
					]

					verifications: [
						{
							description: "Order created successfully"
							criteria: [
								"ID matches format ord_[0-9]{8}_[0-9]{3}",
								"Status is pending_payment",
								"Items array is non-empty",
								"Item quantities are >= 1",
								"Total is between 0.01 and 100000.00",
								"Payment URL is valid",
								"expires_at is valid ISO8601 datetime",
							]
							examples: [
								{
									input: {
										items: [
											{product_id: "prod_abc", quantity: 2},
											{product_id: "prod_xyz", quantity: 1},
										]
										shipping_address: {
											street:  "123 Main St"
											city:    "Seattle"
											state:   "WA"
											zip:     "98101"
											country: "US"
										}
										payment_method_id: "pm_card_visa"
									}
									output: {
										id:     "ord_20240115_001"
										status: "pending_payment"
										items: [
											{
												product_id:   "prod_abc"
												product_name: "Widget Pro"
												quantity:     2
												unit_price:   49.99
												subtotal:     99.98
											},
											{
												product_id:   "prod_xyz"
												product_name: "Gadget Basic"
												quantity:     1
												unit_price:   29.99
												subtotal:     29.99
											},
										]
										totals: {
											subtotal: 129.97
											tax:      11.70
											shipping: 9.99
											total:    151.66
										}
										payment_url: "https://payments.example.com/checkout/ord_20240115_001"
										created_at:  "2024-01-15T10:30:00Z"
										expires_at:  "2024-01-15T11:00:00Z"
									}
								}
							]
						}
					]

					notes: """
						From R1 Question: "Walk me through the happy path"
						Answer: Customer adds items -> validates stock -> creates order
						       -> reserves inventory -> initiates payment -> confirms
						"""
				},
				{
					name:   "create-order-out-of-stock"
					intent: "Order rejected when item is out of stock"

					preconditions: [
						"User is authenticated",
						"At least one item is out of stock",
					]

					postconditions: [
						"Order creation is rejected",
						"No inventory is reserved",
						"Error indicates which items are unavailable",
					]

					verifications: [
						{
							description: "Out of stock returns specific error"
							criteria: [
								"Error code is OUT_OF_STOCK",
								"Error includes unavailable items list",
								"Error shows requested vs available quantities",
							]
							examples: [
								{
									input: {
										items: [
											{product_id: "prod_sold_out", quantity: 1}
										]
										shipping_address: {
											street: "123 Main St"
											city:   "Seattle"
											state:  "WA"
											zip:    "98101"
										}
									}
									output: {
										error: {
											code:    "OUT_OF_STOCK"
											message: "One or more items are out of stock"
											details: {
												unavailable: [
													{
														product_id: "prod_sold_out"
														requested:  1
														available:  0
													}
												]
											}
										}
									}
								}
							]
						}
					]

					notes: """
						From R2 Question: "What's the most common error users will hit?"
						Answer: "Out of stock" - prevents frustration at payment stage

						GAP RESOLVED: Initially undefined behavior. Interview revealed
						need to check stock BEFORE accepting order, not after payment.
						"""
				},
				{
					name:   "create-order-exceeds-limit"
					intent: "Order rejected when total exceeds maximum"

					preconditions: [
						"User is authenticated",
						"Order total exceeds $100,000",
						"Or order has more than 100 items",
					]

					postconditions: [
						"Order creation is rejected",
						"Error shows current limits",
						"Error shows what was provided",
					]

					verifications: [
						{
							description: "Order limit exceeded returns specific error"
							criteria: [
								"Error code is ORDER_LIMIT_EXCEEDED",
								"Error includes max_total and your_total",
								"Error includes max_items and your_items",
							]
							examples: [
								{
									input: {
										items: [
											{product_id: "prod_expensive", quantity: 1000}
										]
									}
									output: {
										error: {
											code:    "ORDER_LIMIT_EXCEEDED"
											message: "Order exceeds maximum allowed value"
											details: {
												max_total:  100000.00
												your_total: 150000.00
												max_items:  100
												your_items: 1000
											}
										}
									}
								}
							]
						}
					]

					notes: """
						From R3 Question: "What's the maximum size of inputs/payloads?"
						Answer: Max order total $100,000, max items 100

						CONFLICT RESOLVED: Business wanted $50k limit, Fraud team
						wanted $10k. Compromise: $100k with enhanced fraud checks
						above $25k.
						"""
				},
			]
		},
		{
			name: "Order State Transitions"

			description: """
				Order workflow states discovered through interview process.
				State machine defined from User perspective questions about
				"what happens next" and error recovery.
				"""

			behaviors: [
				{
					name:   "order-payment-confirmed"
					intent: "Order transitions to confirmed after payment"

					requires: ["create-order-happy-path"]

					preconditions: [
						"Order exists in pending_payment state",
						"Payment is successfully completed",
					]

					postconditions: [
						"Order status changes to confirmed",
						"Payment status is recorded",
						"Transaction ID is stored",
						"Timeline is updated",
					]

					verifications: [
						{
							description: "Payment confirmed successfully"
							criteria: [
								"Status is confirmed",
								"Payment status is completed",
								"Transaction ID is present",
								"Timeline has at least 1 entry",
							]
							examples: [
								{
									output: {
										id:     "ord_20240115_001"
										status: "confirmed"
										payment: {
											status:         "completed"
											transaction_id: "txn_abc123"
											completed_at:   "2024-01-15T10:35:00Z"
										}
										timeline: [
											{status: "pending_payment", at: "2024-01-15T10:30:00Z"},
											{status: "confirmed", at: "2024-01-15T10:35:00Z"},
										]
									}
								}
							]
						}
					]

					notes: """
						From R1 Question: "What are the main workflow states?"
						Answer: pending_payment -> confirmed -> processing -> shipped -> delivered

						This is the happy path state transition.
						"""
				},
				{
					name:   "order-payment-failed"
					intent: "Failed payment returns order to pending state"

					preconditions: [
						"Order exists in pending_payment state",
						"Payment is declined",
					]

					postconditions: [
						"Order stays in pending_payment state",
						"Error indicates payment was declined",
						"Retry is allowed",
						"Expiration timestamp is still valid",
					]

					verifications: [
						{
							description: "Payment failed but retry allowed"
							criteria: [
								"Status is pending_payment",
								"Error retry flag is true",
								"Error expires_at is valid ISO8601 datetime",
							]
							examples: [
								{
									output: {
										order_id:     "ord_20240115_001"
										status:       "pending_payment"
										error: {
											code:       "PAYMENT_DECLINED"
											message:    "Your card was declined"
											retry:      true
											expires_at: "2024-01-15T11:00:00Z"
										}
									}
								}
							]
						}
					]

					notes: """
						From R2 Question: "What happens if a step fails? How does it recover?"
						Answer: Payment failures return to pending_payment, user can retry
						within expiration window.

						GAP RESOLVED: Initially unclear if order should be cancelled
						or allow retry. Interview revealed users prefer retry option.
						"""
				},
				{
					name:   "order-cancel-before-ship"
					intent: "Customer can cancel before shipping"

					preconditions: [
						"Order exists",
						"Order status is pending_payment, confirmed, or processing",
						"User is authenticated as order owner",
					]

					postconditions: [
						"Order status changes to cancelled",
						"Refund is initiated",
						"Refund amount is calculated",
						"Cancellation timestamp is recorded",
					]

					verifications: [
						{
							description: "Order cancelled successfully"
							criteria: [
								"Status is cancelled",
								"Refund status is pending, processing, or completed",
								"Refund amount is between 0.0 and 100000.0",
							]
							examples: [
								{
									input: {
										reason: "Changed my mind"
									}
									output: {
										id:              "ord_20240115_001"
										status:          "cancelled"
										cancelled_at:    "2024-01-15T12:00:00Z"
										cancelled_by:    "customer"
										refund_status:   "pending"
										refund_amount:   151.66
										refund_eta_days: 5
									}
								}
							]
						}
					]

					notes: """
						From R2 Question: "What transitions between states are allowed?"
						Answer: Can cancel from pending_payment, confirmed, processing
						Cannot cancel once shipped (must use returns process)

						CONFLICT RESOLVED: Support wanted to allow cancel anytime,
						Warehouse said impossible after picking. Compromise: cancel
						until picked, then requires manager approval.
						"""
				},
			]
		},
		{
			name: "Order History & Tracking"

			description: """
				Order lookup endpoints with perspective-specific features:
				User wants simple status, Ops needs detailed timeline,
				Security needs audit trail without sensitive data.
				"""

			behaviors: [
				{
					name:   "list-customer-orders"
					intent: "Customer views their order history"

					preconditions: [
						"User is authenticated",
						"User has placed at least one order",
					]

					postconditions: [
						"List of customer's orders is returned",
						"Pagination metadata is included",
						"Orders belong to authenticated user only",
					]

					verifications: [
						{
							description: "Customer orders listed successfully"
							criteria: [
								"Orders array has max 10 items",
								"Order IDs match format ord_[0-9]{8}_[0-9]{3}",
								"Page number is >= 1",
								"Pagination metadata is complete",
							]
							examples: [
								{
									output: {
										orders: [
											{
												id:         "ord_20240115_001"
												status:     "confirmed"
												total:      151.66
												item_count: 3
												created_at: "2024-01-15T10:30:00Z"
											}
										]
										pagination: {
											page:        1
											per_page:    10
											total_pages: 3
											total_items: 25
										}
									}
								}
							]
						}
					]

					notes: """
						From R5 Question: "What's your uptime requirement?"
						Answer: 99.9% - this endpoint is critical for customer experience
						Added caching layer to ensure high availability.
						"""
				},
				{
					name:   "get-order-tracking"
					intent: "Customer tracks shipment status"

					preconditions: [
						"User is authenticated",
						"Order exists and belongs to user",
						"Order has been shipped",
					]

					postconditions: [
						"Tracking information is returned",
						"Tracking events are ordered by timestamp",
						"Carrier information is included",
					]

					verifications: [
						{
							description: "Tracking information returned"
							criteria: [
								"Tracking number is non-empty string",
								"Events array is non-empty",
								"Event timestamps are valid ISO8601 datetime",
								"Estimated delivery matches YYYY-MM-DD format",
							]
							examples: [
								{
									output: {
										order_id:        "ord_20240115_001"
										carrier:         "UPS"
										tracking_number: "1Z999AA10123456784"
										tracking_url:    "https://ups.com/track/1Z999AA10123456784"
										estimated_delivery: "2024-01-18"
										events: [
											{
												timestamp:   "2024-01-16T08:00:00Z"
												location:    "Seattle, WA"
												status:      "In Transit"
												description: "Package departed facility"
											},
											{
												timestamp:   "2024-01-15T18:00:00Z"
												location:    "Seattle, WA"
												status:      "Shipped"
												description: "Package picked up"
											},
										]
									}
								}
							]
						}
					]
				}
			]
		}
	]

	invariants: [
		{
			name: "no-payment-data-in-responses"
			description: "Credit card numbers must never appear in API responses"
			criteria: [
				"Responses do not contain card_number field",
				"Responses do not contain cvv field",
				"Responses do not contain expiry field",
				"Responses do not contain test card numbers like 4111",
			]
		},
		{
			name: "structured-errors"
			description: "All errors have code and message (from Developer interview)"
			criteria: [
				"Error responses (status >= 400) include error.code field",
				"Error responses include error.message field",
			]
		},
		{
			name: "audit-timestamps"
			description: "Orders must have creation timestamps (from Security interview)"
			criteria: [
				"Order responses include created_at field",
				"created_at is valid ISO8601 datetime",
			]
		},
	]

	anti_patterns: [
		{
			name: "exposing-internal-ids"
			description: "Don't expose database IDs (from Security interview R4)"

			bad_example: {
				id:          123
				internal_id: "db_row_456"
			}

			good_example: {
				id: "ord_20240115_001"
			}

			why: """
				Internal IDs reveal system details. Use prefixed opaque IDs
				that include enough context for debugging but no internals.
				"""
		},
		{
			name: "vague-errors"
			description: "Don't return generic errors (from User interview R2)"

			bad_example: {
				error: "Something went wrong"
			}

			good_example: {
				error: {
					code:    "PAYMENT_DECLINED"
					message: "Your card was declined. Please try a different payment method."
					retry:   true
				}
			}

			why: """
				Users need actionable error messages. Specific codes let
				clients handle errors programmatically.
				"""
		},
		{
			name: "inconsistent-states"
			description: "Don't use inconsistent status values (from Developer interview)"

			bad_example: {
				status: "PENDING" // Uppercase
			}

			good_example: {
				status: "pending_payment" // snake_case
			}

			why: """
				Consistent status naming prevents client bugs. Use snake_case
				for all enum values.
				"""
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Go", "PostgreSQL", "Redis"]
		}

		entities: {
			order: {
				fields: {
					id:         "string, 'ord_YYYYMMDD_NNN' format"
					status:     "enum: pending_payment, confirmed, processing, shipped, delivered, cancelled"
					items:      "array of line items with product_id, quantity, unit_price"
					totals:     "object with subtotal, tax, shipping, total"
					created_at: "ISO8601 datetime"
					expires_at: "ISO8601 datetime, 30 min from creation if unpaid"
				}
			}
		}

		security: {
			password_hashing: "Not applicable (no passwords stored)"
			jwt_algorithm:    "RS256"
			jwt_expiry:       "1 hour"
			rate_limiting:    "100 req/min per customer, 1000 req/min per IP"
		}

		pitfalls: [
			"Don't process payment before validating stock",
			"Don't allow order modification after payment",
			"Don't expose payment card data anywhere",
			"Don't allow cancellation after shipping",
			"Don't forget inventory release on order expiry",
		]
	}
}
