package meal_planner_api

import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Meal Planner API"

	description: """
		A meal planning API that allows users to scrape recipes from websites,
		store them in a structured format, create meal plans, and export
		everything to JSON. Recipes are scraped from external URLs and
		normalized into a consistent schema.
		"""

	audience: "Health-conscious individuals and families planning weekly meals"

	version: "1.0.0"

	success_criteria: [
		"Users can scrape recipes from popular recipe websites",
		"Scraped recipes are normalized to a consistent JSON schema",
		"Users can create and manage meal plans",
		"All data can be exported to JSON files",
		"Recipes include nutritional information when available",
	]

	features: [
		{
			name: "Recipe Management"

			description: """
				Core recipe operations: scraping from URLs, listing, retrieving,
				and deleting recipes. Scraped recipes are normalized to include
				title, ingredients, instructions, prep time, cook time, servings,
				and optional nutritional information.
				"""

			behaviors: [
				{
					name:   "create-recipe"
					intent: "Create a new recipe directly via API"

					preconditions: [
						"Recipe data is provided in structured format",
						"Required fields (title, ingredients, instructions) are present",
						"Servings count is positive",
					]

					postconditions: [
						"Recipe is created with unique ID prefixed 'rcp_'",
						"All recipe fields are stored",
						"Creation timestamp is recorded",
						"Source URL is preserved if provided",
					]

					verifications: [
						{
							description: "Recipe object returned with correct fields"
							criteria: [
								"ID matches format rcp_[a-z0-9]+",
								"Title matches the provided title",
								"Ingredients array is non-empty",
								"Instructions array is non-empty",
								"Serving count matches input",
								"created_at is valid ISO8601 datetime",
								"Source URL is valid if provided",
							]
							examples: [
								{
									input: {
										title:      "Famous Butter Chicken"
										source_url: "https://example.com/butter-chicken"
										ingredients: [
											"1 lb chicken breast",
											"2 tbsp butter",
											"1 cup tomato sauce",
											"1/2 cup heavy cream",
											"2 tsp garam masala",
										]
										instructions: [
											"Marinate the chicken in yogurt and spices for 2 hours",
											"Grill or bake the chicken until cooked through",
											"Prepare the butter sauce with tomatoes and cream",
											"Add the chicken to the sauce and simmer for 10 minutes",
										]
										prep_time_minutes:  30
										cook_time_minutes:  45
										servings:           4
										tags: ["indian", "chicken", "dinner"]
									}
									output: {
										id:          "rcp_abc123xyz"
										title:       "Famous Butter Chicken"
										source_url:  "https://example.com/butter-chicken"
										ingredients: [
											"1 lb chicken breast",
											"2 tbsp butter",
											"1 cup tomato sauce",
										]
										instructions: [
											"Marinate the chicken in yogurt and spices for 2 hours",
											"Grill or bake the chicken until cooked through",
										]
										prep_time_minutes:  30
										cook_time_minutes:  45
										servings:           4
										tags:       ["indian", "chicken", "dinner"]
										created_at: "2024-01-15T10:30:00Z"
									}
								}
							]
						}
					]

					notes: """
						Direct recipe creation for programmatic use. The scrape
						endpoint is for extracting recipes from URLs, but this
						endpoint allows creating recipes from structured data.
						"""
				},
				{
					name:   "scrape-recipe-invalid-url"
					intent: "Reject invalid or unreachable URLs"

					preconditions: [
						"URL provided is not a valid format",
					]

					postconditions: [
						"Request is rejected",
						"No recipe is created",
						"Clear error message is returned",
					]

					verifications: [
						{
							description: "Invalid URL returns specific error"
							criteria: [
								"Error code is INVALID_URL",
								"Error message explains the issue",
							]
							examples: [
								{
									input: {
										url: "not-a-valid-url"
									}
									output: {
										error: {
											code:    "INVALID_URL"
											message: "Invalid URL format"
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "scrape-recipe-unsupported-site"
					intent: "Handle websites that cannot be scraped"

					preconditions: [
						"URL is valid but points to unsupported site",
						"Site does not contain recognizable recipe data",
					]

					postconditions: [
						"Request is rejected",
						"Helpful error message includes suggestions",
						"No recipe is created",
					]

					verifications: [
						{
							description: "Unsupported site returns helpful error"
							criteria: [
								"Error code is RECIPE_NOT_FOUND or UNSUPPORTED_SITE",
								"Error message explains the issue",
								"Hint suggests supported recipe sites",
							]
							examples: [
								{
									input: {
										url: "https://example.com/some-random-page"
									}
									output: {
										error: {
											code:    "RECIPE_NOT_FOUND"
											message: "Could not extract recipe data from this URL"
											hint:    "Try a URL from AllRecipes, Food Network, or BBC Good Food"
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "list-all-recipes"
					intent: "Get all saved recipes"

					requires: ["create-recipe"]

					preconditions: [
						"At least one recipe exists in the system",
					]

					postconditions: [
						"All recipes are returned",
						"Total count matches array length",
					]

					verifications: [
						{
							description: "Recipes list is returned"
							criteria: [
								"Recipes array is non-empty",
								"Total count is >= 1",
								"Each recipe has required fields",
							]
						}
					]
				},
				{
					name:   "get-recipe-by-id"
					intent: "Retrieve a specific recipe by ID"

					requires: ["create-recipe"]

					preconditions: [
						"Recipe with specified ID exists",
					]

					postconditions: [
						"Requested recipe is returned",
						"Recipe ID matches requested ID",
					]

					verifications: [
						{
							description: "Single recipe is returned"
							criteria: [
								"ID matches the requested ID",
								"Recipe has a title",
								"All fields are present",
							]
						}
					]
				},
				{
					name:   "get-recipe-not-found"
					intent: "Return 404 for non-existent recipe"

					preconditions: [
						"Recipe with specified ID does not exist",
					]

					postconditions: [
						"Request is rejected",
						"Error indicates resource not found",
					]

					verifications: [
						{
							description: "Non-existent recipe returns 404"
							criteria: [
								"Error code is NOT_FOUND",
							]
							examples: [
								{
									input: {
										id: "rcp_nonexistent999"
									}
									output: {
										error: {
											code: "NOT_FOUND"
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "delete-recipe"
					intent: "Remove a recipe from the collection"

					requires: ["export-meal-plan"]

					preconditions: [
						"Recipe with specified ID exists",
					]

					postconditions: [
						"Recipe is deleted from the system",
						"No trace of recipe remains",
					]

					verifications: [
						{
							description: "Recipe is deleted successfully"
							criteria: [
								"Deletion succeeds",
								"Subsequent get returns 404",
							]
						}
					]

					notes: """
						Run last since it removes the recipe used by other tests
						"""
				},
			]
		},
		{
			name: "Meal Planning"

			description: """
				Create and manage meal plans. A meal plan assigns recipes to
				specific days and meal types (breakfast, lunch, dinner, snack).
				Plans can span any date range and include shopping lists.
				"""

			behaviors: [
				{
					name:   "create-meal-plan"
					intent: "Create a new meal plan for a week"

					preconditions: [
						"Plan name is provided",
						"Start and end dates are valid",
						"End date is after start date",
					]

					postconditions: [
						"Meal plan is created with unique ID prefixed 'plan_'",
						"Plan starts with empty meals list",
						"Creation timestamp is recorded",
					]

					verifications: [
						{
							description: "Meal plan created successfully"
							criteria: [
								"ID matches format plan_[a-z0-9]+",
								"Name matches input",
								"Start date matches input",
								"End date matches input",
								"Meals array is initially empty",
								"created_at is valid ISO8601 datetime",
							]
							examples: [
								{
									input: {
										name:       "Healthy Week"
										start_date: "2024-01-15"
										end_date:   "2024-01-21"
									}
									output: {
										id:         "plan_xyz789"
										name:       "Healthy Week"
										start_date: "2024-01-15"
										end_date:   "2024-01-21"
										meals:      []
										created_at: "2024-01-15T10:30:00Z"
									}
								}
							]
						}
					]
				},
				{
					name:   "create-meal-plan-invalid-dates"
					intent: "Reject meal plan with end date before start date"

					preconditions: [
						"End date is before start date",
					]

					postconditions: [
						"Plan creation is rejected",
						"No plan is created",
						"Clear error about date range is returned",
					]

					verifications: [
						{
							description: "Invalid date range returns error"
							criteria: [
								"Error code is INVALID_DATE_RANGE",
							]
							examples: [
								{
									input: {
										name:       "Bad Plan"
										start_date: "2024-01-21"
										end_date:   "2024-01-15"
									}
									output: {
										error: {
											code: "INVALID_DATE_RANGE"
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "add-meal-to-plan"
					intent: "Schedule a recipe for a specific meal"

					requires: ["create-meal-plan", "create-recipe"]

					preconditions: [
						"Meal plan exists",
						"Recipe exists",
						"Meal type is valid (breakfast, lunch, dinner, snack)",
						"Servings is positive",
					]

					postconditions: [
						"Meal is added to plan",
						"Meal has unique ID prefixed 'meal_'",
						"Recipe details are included",
					]

					verifications: [
						{
							description: "Meal added successfully"
							criteria: [
								"ID matches format meal_[a-z0-9]+",
								"Meal type is one of: breakfast, lunch, dinner, snack",
								"Servings is >= 1",
								"Recipe title is included",
							]
							examples: [
								{
									input: {
										recipe_id: "rcp_abc123xyz"
										date:      "2024-01-15"
										meal_type: "dinner"
										servings:  4
									}
									output: {
										id:        "meal_abc123"
										recipe_id: "rcp_abc123xyz"
										date:      "2024-01-15"
										meal_type: "dinner"
										servings:  4
										recipe: {
											id:    "rcp_abc123xyz"
											title: "Famous Butter Chicken"
										}
									}
								}
							]
						}
					]
				},
				{
					name:   "get-meal-plan"
					intent: "Retrieve a meal plan with all scheduled meals"

					requires: ["add-meal-to-plan"]

					preconditions: [
						"Meal plan exists",
						"At least one meal is scheduled",
					]

					postconditions: [
						"Plan is returned with all meals",
						"Meals array includes scheduled meals",
					]

					verifications: [
						{
							description: "Meal plan returned with meals"
							criteria: [
								"ID matches requested plan",
								"Meals array is non-empty",
							]
						}
					]
				},
				{
					name:   "generate-shopping-list"
					intent: "Generate aggregated shopping list from meal plan"

					requires: ["add-meal-to-plan"]

					preconditions: [
						"Meal plan exists",
						"At least one meal is scheduled with ingredients",
					]

					postconditions: [
						"Shopping list is generated",
						"Ingredients are aggregated by recipe",
						"Generation timestamp is recorded",
					]

					verifications: [
						{
							description: "Shopping list generated"
							criteria: [
								"Plan ID matches requested plan",
								"Items array is non-empty",
								"Items include ingredient and quantity",
								"Items reference which recipes use them",
								"generated_at is valid ISO8601 datetime",
							]
							examples: [
								{
									output: {
										plan_id: "plan_xyz789"
										items: [
											{
												ingredient: "chicken breast"
												quantity:   "1 lb"
												recipes:    ["Famous Butter Chicken"]
											},
											{
												ingredient: "butter"
												quantity:   "2 tbsp"
												recipes:    ["Famous Butter Chicken"]
											},
										]
										generated_at: "2024-01-15T10:35:00Z"
									}
								}
							]
						}
					]
				},
			]
		},
		{
			name: "Data Export"

			description: """
				Export recipes and meal plans to JSON files for backup,
				sharing, or use in other applications.
				"""

			behaviors: [
				{
					name:   "export-all-recipes"
					intent: "Export all recipes to a JSON file"

					requires: ["create-recipe"]

					preconditions: [
						"At least one recipe exists",
					]

					postconditions: [
						"All recipes are exported",
						"Export includes schema version",
						"Export timestamp is recorded",
					]

					verifications: [
						{
							description: "All recipes exported successfully"
							criteria: [
								"Recipes array is non-empty",
								"exported_at is valid ISO8601 datetime",
								"Version matches format ^1\\.[0-9]+$",
							]
						}
					]
				},
				{
					name:   "export-meal-plan"
					intent: "Export a specific meal plan to JSON"

					requires: ["add-meal-to-plan"]

					preconditions: [
						"Meal plan exists",
						"At least one meal is scheduled",
					]

					postconditions: [
						"Plan is exported with meals",
						"Full recipe data is included",
					]

					verifications: [
						{
							description: "Meal plan exported successfully"
							criteria: [
								"Meal plan ID matches requested plan",
								"Meals array is non-empty",
								"Recipes array is non-empty",
							]
						}
					]
				},
			]
		},
	]

	invariants: [
		{
			name: "consistent-error-format"
			description: "All errors return structured error objects"
			criteria: [
				"Error responses (status >= 400) include error.code field",
				"Error responses include error.message field",
			]
		},
		{
			name: "no-internal-errors-exposed"
			description: "Internal implementation details not leaked"
			criteria: [
				"Error responses (status >= 500) do not contain stack traces",
				"Error responses do not contain panic messages",
				"Error responses do not contain runtime error details",
				"Error responses do not contain SQL queries or database errors",
			]
		},
	]

	anti_patterns: [
		{
			name: "sequential-ids"
			description: "IDs should not be sequential integers"

			bad_example: {
				id: 1
			}

			good_example: {
				id: "rcp_x7k9m2p4q"
			}

			why: """
				Sequential IDs reveal business metrics and enable enumeration
				attacks. Use prefixed random strings instead.
				"""
		},
		{
			name: "null-instead-of-empty-array"
			description: "Empty collections should be empty arrays, not null"

			bad_example: {
				ingredients: null
			}

			good_example: {
				ingredients: []
			}

			why: "Null vs empty array causes client-side null checks and crashes"
		},
		{
			name: "inconsistent-time-format"
			description: "All timestamps must use ISO8601 format"

			bad_example: {
				created_at: "Jan 15, 2024 10:30 AM"
			}

			good_example: {
				created_at: "2024-01-15T10:30:00Z"
			}

			why: "ISO8601 is machine-parseable and timezone-aware"
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Go", "net/http", "SQLite", "colly (web scraping)"]
		}

		entities: {
			recipe: {
				fields: {
					id:                 "string, prefixed 'rcp_', randomly generated"
					title:              "string, extracted from webpage"
					source_url:         "string, original URL"
					ingredients:        "[]string, list of ingredient lines"
					instructions:       "[]string, list of steps"
					prep_time_minutes:  "int, optional"
					cook_time_minutes:  "int, optional"
					total_time_minutes: "int, calculated or extracted"
					servings:           "int, number of servings"
					nutrition:          "object, optional nutritional info"
					tags:               "[]string, categorization"
					created_at:         "datetime, when scraped"
				}
			}
			meal_plan: {
				fields: {
					id:         "string, prefixed 'plan_'"
					name:       "string, user-provided name"
					start_date: "date, YYYY-MM-DD format"
					end_date:   "date, YYYY-MM-DD format"
					meals:      "[]meal, scheduled meals"
					created_at: "datetime"
				}
			}
			meal: {
				fields: {
					id:        "string, prefixed 'meal_'"
					recipe_id: "string, reference to recipe"
					date:      "date, when to prepare"
					meal_type: "enum: breakfast, lunch, dinner, snack"
					servings:  "int, portions for this meal"
				}
			}
		}

		security: {
			rate_limiting: "100 requests per minute per IP for scraping"
		}

		pitfalls: [
			"Don't scrape sites that block bots - respect robots.txt",
			"Don't assume all recipe sites have the same structure",
			"Don't forget to handle network timeouts when scraping",
			"Don't store raw HTML - extract and normalize to JSON",
			"Don't forget ingredient quantity parsing is hard - keep original strings",
		]
	}
}
