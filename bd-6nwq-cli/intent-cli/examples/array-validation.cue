package array_validation

import "github.com/intent-cli/intent/schema:intent"

// Example: Array Validation Rules
// Demonstrates various array checks like length, min/max items, and element validation

spec: intent.#Spec & {
	name: "Product Catalog API"

	description: """
		A product catalog API demonstrating array validation patterns.
		Shows how to validate list endpoints, pagination, and array elements.
		"""

	audience: "E-commerce applications"

	version: "1.0.0"

	success_criteria: [
		"Products are returned as arrays with predictable structure",
		"Pagination limits are respected",
		"Array elements follow consistent schema",
	]

	features: [
		{
			name: "Product Listing"

			description: """
				List products with filtering and pagination. Demonstrates
				various array validation rules.
				"""

			behaviors: [
				{
					name:   "list-all-products"
					intent: "Retrieve all products as a non-empty array"

					preconditions: [
						"Product catalog contains at least one product",
					]

					postconditions: [
						"All products are returned",
						"Total count matches array length",
					]

					verifications: [
						{
							description: "Product list is non-empty"
							criteria: [
								"Products array is not empty",
								"Total count is >= 1",
								"Total count matches array length",
							]
							examples: [
								{
									input: {}
									output: {
										products: [
											{id: "prod_abc123", name: "Widget", price: 9.99},
											{id: "prod_def456", name: "Gadget", price: 19.99},
										]
										total: 2
									}
								}
							]
						}
					]
				},
				{
					name:   "list-with-pagination"
					intent: "Pagination respects limit parameter"

					preconditions: [
						"User requests limit of 5 products",
						"Catalog has more than 5 products",
					]

					postconditions: [
						"Returned array has exactly 5 items",
						"Limit parameter is echoed in response",
					]

					verifications: [
						{
							description: "Pagination limits are enforced"
							criteria: [
								"Products array has max 5 items",
								"Limit equals 5",
								"Offset is present",
							]
							examples: [
								{
									input: {limit: 5}
									output: {
										products: [
											{id: "prod_001", name: "Item 1", price: 10.00},
											{id: "prod_002", name: "Item 2", price: 20.00},
											{id: "prod_003", name: "Item 3", price: 30.00},
											{id: "prod_004", name: "Item 4", price: 40.00},
											{id: "prod_005", name: "Item 5", price: 50.00},
										]
										limit:  5
										offset: 0
										total:  100
									}
								}
							]
						}
					]
				},
				{
					name:   "list-exact-length"
					intent: "Featured products returns exactly 3 items"

					preconditions: [
						"Featured products section exists",
					]

					postconditions: [
						"Exactly 3 featured products are returned",
					]

					verifications: [
						{
							description: "Featured section has fixed size"
							criteria: [
								"Featured array has exactly 3 items",
								"All items have valid product IDs",
							]
							examples: [
								{
									input: {}
									output: {
										featured: [
											{id: "prod_feat1", name: "Top Seller", price: 99.99},
											{id: "prod_feat2", name: "New Arrival", price: 49.99},
											{id: "prod_feat3", name: "Staff Pick", price: 29.99},
										]
									}
								}
							]
						}
					]
				},
				{
					name:   "list-minimum-items"
					intent: "Search results have at least 1 result when query matches"

					preconditions: [
						"Search query 'widget' matches products in catalog",
					]

					postconditions: [
						"At least 1 result is returned",
						"Query is echoed in response",
					]

					verifications: [
						{
							description: "Search returns matching results"
							criteria: [
								"Results array has min 1 item",
								"Query equals 'widget'",
							]
							examples: [
								{
									input: {q: "widget"}
									output: {
										results: [
											{id: "prod_widget1", name: "Blue Widget", price: 12.99},
											{id: "prod_widget2", name: "Red Widget", price: 14.99},
										]
										query: "widget"
									}
								}
							]
						}
					]
				},
				{
					name:   "list-with-element-validation"
					intent: "Each tag in product follows naming convention"

					preconditions: [
						"Product prod_abc123 exists",
						"Product has tags assigned",
					]

					postconditions: [
						"Product data is returned",
						"All tags follow lowercase kebab-case format",
					]

					verifications: [
						{
							description: "Tags follow naming convention"
							criteria: [
								"ID matches format prod_[a-z0-9]+",
								"Each tag matches [a-z][a-z0-9-]*",
								"Tags are lowercase with hyphens",
							]
							examples: [
								{
									input: {product_id: "prod_abc123"}
									output: {
										id:   "prod_abc123"
										name: "Widget Pro"
										tags: ["electronics", "new-arrival", "sale"]
									}
								}
							]
						}
					]
				},
				{
					name:   "empty-search-results"
					intent: "Search with no matches returns empty array (not null)"

					preconditions: [
						"Search query matches no products",
					]

					postconditions: [
						"Empty array is returned (not null)",
						"Total is 0",
					]

					verifications: [
						{
							description: "Empty results return empty array"
							criteria: [
								"Results is an array (not null)",
								"Results array has 0 items",
								"Total equals 0",
							]
							examples: [
								{
									input: {q: "nonexistent_xyz_123"}
									output: {
										results: []
										query:   "nonexistent_xyz_123"
										total:   0
									}
								}
							]
						}
					]
				},
			]
		},
		{
			name: "Categories"

			description: """
				Category endpoints demonstrating nested array validation.
				"""

			behaviors: [
				{
					name:   "list-categories-with-products"
					intent: "Categories include product counts and nested arrays"

					preconditions: [
						"Categories exist in the system",
						"At least one category has subcategories",
					]

					postconditions: [
						"All categories are returned",
						"Each category has product count",
						"Subcategories are present as arrays",
					]

					verifications: [
						{
							description: "Categories include nested arrays"
							criteria: [
								"Categories array is not empty",
								"First category has non-empty subcategories array",
								"Product count is >= 0",
							]
							examples: [
								{
									input: {}
									output: {
										categories: [
											{
												id:            "cat_electronics"
												name:          "Electronics"
												product_count: 150
												subcategories: ["phones", "laptops", "accessories"]
											},
											{
												id:            "cat_clothing"
												name:          "Clothing"
												product_count: 300
												subcategories: ["shirts", "pants", "shoes"]
											},
										]
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
			name: "array-responses"
			description: "List endpoints should return arrays, not objects"
			criteria: [
								"List endpoints return array types",
								"Empty collections are empty arrays not null",
			]
		},
	]

	anti_patterns: [
		{
			name: "null-for-empty"
			description: "Never return null for empty collections"

			bad_example: {
				products: null
			}

			good_example: {
				products: []
			}

			why: """
				Null requires special handling in clients. Empty arrays
				are more predictable and avoid null pointer exceptions.
				"""
		},
		{
			name: "array-without-wrapper"
			description: "Don't return bare arrays, wrap in object"

			bad_example: {
				_comment: "Response is just a bare array"
				response: "[ {id: 1, name: Product} ]"
			}

			good_example: {
				products: [{id: "1", name: "Product"}]
				total:    1
			}

			why: """
				Bare arrays can't be extended with metadata. Object
				wrappers allow adding pagination, totals, and links.
				"""
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Go", "PostgreSQL"]
		}

		entities: {
			product: {
				fields: {
					id:    "string, prefixed 'prod_', randomly generated"
					name:  "string, 1-200 chars"
					price: "decimal, >= 0"
					tags:  "array of strings, lowercase kebab-case"
				}
			}
		}

		pitfalls: [
			"Don't return null for empty arrays",
			"Don't exceed pagination limits",
			"Ensure total count matches when not paginated",
			"Validate array element structure consistently",
		]
	}
}
