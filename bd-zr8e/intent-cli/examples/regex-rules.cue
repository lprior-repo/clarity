package regex_rules

import "github.com/intent-cli/intent/schema:intent"

// Example: Regex Pattern Validation
// Demonstrates string matching, starting with, ending with, and containing rules

spec: intent.#Spec & {
	name: "Document Management API"

	description: """
		A document management API demonstrating various regex validation patterns.
		Shows how to validate IDs, file names, slugs, codes, and formatted strings.
		"""

	audience: "Content management systems"

	version: "1.0.0"

	success_criteria: [
		"All identifiers follow predictable patterns",
		"File names and slugs are sanitized",
		"Codes and references match expected formats",
	]

	features: [
		{
			name: "Document Operations"

			description: """
				Create and manage documents with validated naming patterns.
				"""

			behaviors: [
				{
					name:   "create-document"
					intent: "Create document with validated ID and slug"

					preconditions: [
						"User is authenticated",
						"Document title is provided",
					]

					postconditions: [
						"Document created with prefixed ID",
						"Slug generated from title in kebab-case",
						"Version follows semantic versioning",
						"Timestamp in ISO8601 format",
					]

					verifications: [
						{
							description: "Verify document creation with validated fields"
							criteria: [
								"Document ID format: doc_[a-z0-9]{10}",
								"Slug is lowercase kebab-case: [a-z0-9]+(-[a-z0-9]+)*",
								"Version format: X.Y.Z",
								"Timestamp is valid ISO8601",
							]
							examples: [
								{
									input: {
										title: "Getting Started Guide"
										content: "Welcome to our platform..."
									}
									output: {
										id:         "doc_a1b2c3d4e5"
										slug:       "getting-started-guide"
										title:      "Getting Started Guide"
										version:    "1.0.0"
										created_at: "2024-01-15T10:30:00Z"
									}
								}
							]
						}
					]
				},
				{
					name:   "get-document-by-slug"
					intent: "Retrieve document using URL-safe slug"

					requires: ["create-document"]

					preconditions: [
						"Document with given slug exists",
					]

					postconditions: [
						"Correct document returned",
						"Slug matches request",
					]

					verifications: [
						{
							description: "Verify slug-based document retrieval"
							criteria: [
								"Returned document ID matches the slug's document",
								"Slug only contains lowercase, numbers, hyphens",
							]
							examples: [
								{
									input: {
										slug: "getting-started-guide"
									}
									output: {
										id:   "doc_a1b2c3d4e5"
										slug: "getting-started-guide"
										title: "Getting Started Guide"
									}
								}
							]
						}
					]
				},
			]
		},
		{
			name: "File Uploads"

			description: """
				File upload endpoints with extension and naming validation.
				"""

			behaviors: [
				{
					name:   "upload-image"
					intent: "Upload image with validated filename"

					preconditions: [
						"User is authenticated",
						"Valid image file provided",
					]

					postconditions: [
						"File stored with validated ID",
						"Original filename preserved",
						"Extension validated against allowed types",
						"MIME type is image/*",
						"CDN URL uses HTTPS",
					]

					verifications: [
						{
							description: "Verify image upload validation"
							criteria: [
								"File ID format: file_img_[a-z0-9]+",
								"Original filename ends with valid extension",
								"Stored name matches pattern with extension",
								"MIME type starts with image/",
								"URL starts with https://",
							]
							examples: [
								{
									input: {
										filename: "my-photo.jpg"
										content: "<binary data>"
									}
									output: {
										file_id:       "file_img_abc123"
										original_name: "my-photo.jpg"
										stored_name:   "file_img_abc123.jpg"
										mime_type:     "image/jpeg"
										size_bytes:    245678
										url:           "https://cdn.example.com/images/file_img_abc123.jpg"
									}
								}
							]
						}
					]
				},
				{
					name:   "upload-document-file"
					intent: "Upload document with specific extension validation"

					preconditions: [
						"User is authenticated",
						"Document file provided",
					]

					postconditions: [
						"File validated against allowed document types",
						"MIME type matches file type",
					]

					verifications: [
						{
							description: "Verify document file validation"
							criteria: [
								"File ID format: file_doc_[a-z0-9]+",
								"Stored name has allowed document extension",
								"MIME type is one of: application/pdf, application/msword, text/plain, text/markdown",
							]
							examples: [
								{
									input: {
										filename: "annual-report-2024.pdf"
										content: "<binary data>"
									}
									output: {
										file_id:       "file_doc_xyz789"
										original_name: "annual-report-2024.pdf"
										stored_name:   "file_doc_xyz789.pdf"
										mime_type:     "application/pdf"
										size_bytes:    1234567
									}
								}
							]
						}
					]
				},
			]
		},
		{
			name: "Reference Codes"

			description: """
				Various business reference codes with strict formatting.
				"""

			behaviors: [
				{
					name:   "create-invoice"
					intent: "Create invoice with formatted reference number"

					preconditions: [
						"Customer exists",
						"Amount is valid",
					]

					postconditions: [
						"Invoice ID includes date and sequence",
						"Reference is human-readable format",
						"Due date in YYYY-MM-DD format",
						"No payment ID on new invoice",
					]

					verifications: [
						{
							description: "Verify invoice code formats"
							criteria: [
								"ID format: inv_[0-9]{8}_[0-9]{4}",
								"Reference format: INV-[0-9]{4}-[0-9]{5}",
								"Customer ID starts with cust_",
								"Due date: YYYY-MM-DD",
								"Payment ID is null/absent",
							]
							examples: [
								{
									input: {
										customer_id: "cust_123"
										amount:      150.00
									}
									output: {
										id:         "inv_20240115_0001"
										reference:  "INV-2024-00001"
										customer:   "cust_123"
										amount:     150.00
										status:     "pending"
										issued_at:  "2024-01-15T10:30:00Z"
										due_date:   "2024-02-15"
										payment_id: null
									}
								}
							]
						}
					]
				},
				{
					name:   "record-payment"
					intent: "Record payment with transaction reference"

					requires: ["create-invoice"]

					preconditions: [
						"Invoice exists",
						"Payment method valid",
					]

					postconditions: [
						"Payment recorded with method-specific ID",
						"Transaction ID follows timestamp format",
						"Receipt URL contains /receipts/",
					]

					verifications: [
						{
							description: "Verify payment code formats"
							criteria: [
								"Payment ID format: pay_(cc|bank|wire|check)_[a-z0-9]+",
								"Transaction ID format: TXN[0-9]{14}[A-Z]{3}",
								"Method is valid enum value",
								"Receipt URL contains /receipts/",
							]
							examples: [
								{
									input: {
										amount: 150.00
										method: "credit_card"
									}
									output: {
										payment_id:     "pay_cc_abc123xyz"
										transaction_id: "TXN20240115103045ABC"
										amount:         150.00
										method:         "credit_card"
										status:         "completed"
										receipt_url:    "https://payments.example.com/receipts/pay_cc_abc123xyz"
									}
								}
							]
						}
					]
				},
				{
					name:   "create-shipping-label"
					intent: "Create shipping label with carrier-specific tracking"

					preconditions: [
						"Carrier is supported",
						"Order ID valid",
					]

					postconditions: [
						"Label ID includes carrier and date",
						"Tracking number matches carrier format",
						"Service code in CARRIER-SERVICE format",
						"Label URL ends with .pdf",
					]

					verifications: [
						{
							description: "Verify shipping label formats"
							criteria: [
								"Label ID format: lbl_(ups|fedex|usps|dhl)_[0-9]{8}_[0-9]{3}",
								"UPS tracking: 1Z[A-Z0-9]{16}",
								"Service code: (UPS|FEDEX|USPS|DHL)-[A-Z]+",
								"Label URL ends with .pdf",
							]
							examples: [
								{
									input: {
										carrier:  "ups"
										order_id: "ord_abc123"
									}
									output: {
										label_id:        "lbl_ups_20240115_001"
										carrier:         "ups"
										tracking_number: "1Z999AA10123456784"
										service_code:    "UPS-GROUND"
										label_url:       "https://labels.example.com/lbl_ups_20240115_001.pdf"
									}
								}
							]
						}
					]
				},
			]
		},
		{
			name: "User Identifiers"

			description: """
				Various user identifier formats with specific patterns.
				"""

			behaviors: [
				{
					name:   "create-api-key"
					intent: "Generate API key with specific format"

					preconditions: [
						"User is authenticated",
						"Key name provided",
					]

					postconditions: [
						"Key ID indicates environment (live/test)",
						"API key follows specific format",
						"Full key only shown once",
					]

					notes: """
						The full api_key is only shown once at creation time.
						Store it securely as it cannot be retrieved later.
						"""

					verifications: [
						{
							description: "Verify API key format"
							criteria: [
								"Key ID format: key_(live|test)_[a-z0-9]+",
								"API key format: example_key_[x]{32}",
								"Prefix matches example_",
							]
							examples: [
								{
									input: {
										name: "Production Key"
									}
									output: {
										key_id:     "key_live_abc123"
										api_key:    "example_key_xxxxxxxxxxxxxxxxxxxxxxxxxxxx"
										name:       "Production Key"
										prefix:     "example_"
										created_at: "2024-01-15T10:30:00Z"
									}
								}
							]
						}
					]
				},
				{
					name:   "validate-phone-number"
					intent: "Validate and format phone number"

					preconditions: [
						"Phone number provided",
					]

					postconditions: [
						"E.164 format validated",
						"Formatted for display",
						"Country code extracted",
						"Phone type classified",
					]

					verifications: [
						{
							description: "Verify phone number validation"
							criteria: [
								"E.164 format: +[1-9][0-9]{6,14}",
								"US formatted: +[0-9]+ \\([0-9]{3}\\) [0-9]{3}-[0-9]{4}",
								"Country code: [1-9][0-9]{0,2}",
								"Type is one of: mobile, landline, voip, unknown",
							]
							examples: [
								{
									input: {
										phone: "+15551234567"
									}
									output: {
										valid:         true
										original:      "+15551234567"
										formatted:     "+1 (555) 123-4567"
										country_code:  "1"
										national:      "(555) 123-4567"
										e164:          "+15551234567"
										type:          "mobile"
										carrier:       "Example Wireless"
									}
								}
							]
						}
					]
				},
				{
					name:   "validate-credit-card"
					intent: "Validate credit card with masked display"

					preconditions: [
						"Card number provided",
					]

					postconditions: [
						"Card number masked showing only last 4",
						"Brand identified",
						"BIN extracted (first 6 digits)",
					]

					verifications: [
						{
							description: "Verify credit card validation and masking"
							criteria: [
								"Masked format: **** **** **** [0-9]{4}",
								"Last 4 digits: [0-9]{4}",
								"Brand is one of: visa, mastercard, amex, discover",
								"BIN: [0-9]{6}",
							]
							examples: [
								{
									input: {
										number: "4111111111111111"
									}
									output: {
										valid:   true
										masked:  "**** **** **** 1111"
										last4:   "1111"
										brand:   "visa"
										type:    "credit"
										bin:     "411111"
										country: "US"
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
			name:        "id-format-consistency"
			description: "All IDs should follow prefix_identifier pattern"

			criteria: [
				"No numeric IDs like {id: 1}",
				"No bare integer IDs like {id: \"1\"}",
				"All IDs use prefixed string format",
			]
		},
	]

	anti_patterns: [
		{
			name:        "numeric-ids"
			description: "Don't use numeric IDs, use prefixed strings"

			bad_example: {
				id: 12345
			}

			good_example: {
				id: "doc_abc123xyz"
			}

			why: """
				Prefixed string IDs are self-documenting (you know it's a
				document) and don't leak information about record counts.
				"""
		},
		{
			name:        "inconsistent-id-patterns"
			description: "Don't mix ID formats across the API"

			bad_example: {
				user_id:    "USR-123"
				order_id:   "ord_456"
				product_id: "PROD:789"
			}

			good_example: {
				user_id:    "usr_abc123"
				order_id:   "ord_def456"
				product_id: "prod_ghi789"
			}

			why: """
				Consistent ID patterns make parsing predictable and
				help with debugging and log analysis.
				"""
		},
		{
			name:        "unvalidated-slugs"
			description: "Don't accept any string as a slug"

			bad_example: {
				slug: "My Document Title!!!"
			}

			good_example: {
				slug: "my-document-title"
			}

			why: """
				Slugs should be URL-safe: lowercase, alphanumeric, and
				hyphens only. Special characters cause encoding issues.
				"""
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["TypeScript", "Express", "PostgreSQL"]
		}

		entities: {
			document: {
				fields: {
					id:      "string, 'doc_' + 10 random alphanumeric"
					slug:    "string, generated from title, lowercase kebab-case"
					version: "string, semantic version X.Y.Z"
				}
			}
			invoice: {
				fields: {
					id:        "string, 'inv_YYYYMMDD_NNNN' format"
					reference: "string, 'INV-YYYY-NNNNN' for humans"
				}
			}
		}

		pitfalls: [
			"Validate regex patterns on input, not just output",
			"Be careful with regex escaping in different languages",
			"Consider Unicode in regex patterns",
			"Test edge cases like empty strings",
			"Compile regex once, not on every request",
		]
	}
}
