package pokemon_api

import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Pokemon API"

	description: """
		A simple API for managing Pokemon and their trainers.
		Trainers can have multiple Pokemon, and each Pokemon belongs
		to exactly one trainer.
		"""

	audience: "Game clients and admin tools"

	version: "1.0.0"

	success_criteria: [
		"Pokemon can be listed and retrieved by ID",
		"Trainers can be listed and retrieved by ID",
		"New Pokemon can be created with valid data",
		"All errors return structured error objects",
	]

	features: [
		{
			name: "Pokemon Retrieval"

			description: """
				Users can list all Pokemon or retrieve a specific Pokemon
				by its ID. Pokemon IDs are prefixed with 'pkmn_'.
				"""

			behaviors: [
				{
					name:   "list-all-pokemon"
					intent: "Get a list of all Pokemon in the system"

					preconditions: [
						"Pokemon data exists in the system",
					]

					postconditions: [
						"All Pokemon are returned",
						"Each Pokemon has required fields populated",
					]

					verifications: [
						{
							description: "Pokemon list is non-empty"
							criteria: [
								"Pokemon array is not empty",
								"Each Pokemon has valid ID",
								"Each Pokemon has a name",
								"Each Pokemon has a type",
								"Each Pokemon has level >= 1",
								"Each Pokemon has HP >= 1",
								"Each Pokemon belongs to a trainer",
							]
							examples: [
								{
									input: {}
									output: {
										pokemon: [
											{
												id:         "pkmn_001"
												name:       "Pikachu"
												type:       "Electric"
												level:      25
												hp:         100
												trainer_id: "trainer_ash"
											},
										]
									}
								}
							]
						}
					]
				},
				{
					name:   "get-existing-pokemon"
					intent: "Retrieve a specific Pokemon by ID"

					preconditions: [
						"Pokemon with ID pkmn_001 exists",
					]

					postconditions: [
						"Pokemon data is returned",
						"All Pokemon fields are present",
					]

					verifications: [
						{
							description: "Pokemon data matches expected structure"
							criteria: [
								"ID matches format pkmn_[0-9]+",
								"Name is Pikachu",
								"Type is from predefined list",
								"Level is >= 1",
								"HP is >= 1",
								"Trainer ID matches format trainer_[a-z]+",
							]
							examples: [
								{
									input: {
										pokemon_id: "pkmn_001"
									}
									output: {
										id:         "pkmn_001"
										name:       "Pikachu"
										type:       "Electric"
										level:      25
										hp:         100
										trainer_id: "trainer_ash"
									}
								}
							]
						}
					]
				},
				{
					name:   "get-nonexistent-pokemon"
					intent: "Requesting a non-existent Pokemon returns error"

					preconditions: [
						"Pokemon with ID pkmn_999 does not exist",
					]

					postconditions: [
						"Request fails",
						"Error indicates Pokemon not found",
					]

					verifications: [
						{
							description: "Non-existent Pokemon returns 404 error"
							criteria: [
								"Error code is POKEMON_NOT_FOUND",
								"Error message is human-readable",
							]
							examples: [
								{
									input: {
										pokemon_id: "pkmn_999"
									}
									output: {
										error: {
											code:    "POKEMON_NOT_FOUND"
											message: "Pokemon with id 'pkmn_999' not found"
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
			name: "Trainer Retrieval"

			description: """
				Users can list all trainers or retrieve a specific trainer
				by ID. Trainer responses include a pokemon_count field.
				"""

			behaviors: [
				{
					name:   "list-all-trainers"
					intent: "Get a list of all trainers"

					preconditions: [
						"Trainer data exists in the system",
					]

					postconditions: [
						"All trainers are returned",
						"Each trainer has pokemon_count",
					]

					verifications: [
						{
							description: "Trainer list is non-empty"
							criteria: [
								"Trainers array is not empty",
								"Each trainer has valid ID",
								"Each trainer has a name",
								"Each trainer has pokemon_count >= 0",
							]
							examples: [
								{
									input: {}
									output: {
										trainers: [
											{
												id:            "trainer_ash"
												name:          "Ash Ketchum"
												pokemon_count: 2
											},
										]
									}
								}
							]
						}
					]
				},
				{
					name:   "get-existing-trainer"
					intent: "Retrieve a specific trainer by ID"

					preconditions: [
						"Trainer with ID trainer_ash exists",
					]

					postconditions: [
						"Trainer data is returned",
					]

					verifications: [
						{
							description: "Trainer data is complete"
							criteria: [
								"ID is trainer_ash",
								"Name is non-empty string",
								"Pokemon count is >= 0",
							]
							examples: [
								{
									input: {
										trainer_id: "trainer_ash"
									}
									output: {
										id:            "trainer_ash"
										name:          "Ash Ketchum"
										pokemon_count: 2
									}
								}
							]
						}
					]
				},
				{
					name:   "get-trainer-pokemon"
					intent: "Get all Pokemon belonging to a trainer"

					requires: ["get-existing-trainer"]

					preconditions: [
						"Trainer trainer_ash exists",
						"Trainer has at least one Pokemon",
					]

					postconditions: [
						"All trainer's Pokemon are returned",
						"All Pokemon belong to this trainer",
					]

					verifications: [
						{
							description: "Trainer's Pokemon list is complete"
							criteria: [
								"Pokemon array is not empty",
								"All Pokemon belong to trainer_ash",
							]
						}
					]
				},
				{
					name:   "get-nonexistent-trainer"
					intent: "Requesting a non-existent trainer returns error"

					preconditions: [
						"Trainer with ID trainer_unknown does not exist",
					]

					postconditions: [
						"Request fails",
						"Error indicates trainer not found",
					]

					verifications: [
						{
							description: "Non-existent trainer returns error"
							criteria: [
								"Error code is TRAINER_NOT_FOUND",
							]
						}
					]
				},
			]
		},
		{
			name: "Pokemon Creation"

			description: """
				New Pokemon can be created with a name, type, level,
				and trainer_id. The system validates all fields.
				"""

			behaviors: [
				{
					name:   "create-pokemon-success"
					intent: "Create a new Pokemon with valid data"

					requires: ["get-existing-trainer"]

					preconditions: [
						"Trainer trainer_ash exists",
						"Valid Pokemon data provided",
					]

					postconditions: [
						"Pokemon is created",
						"Pokemon is assigned unique ID",
						"Pokemon is assigned to trainer",
						"HP is calculated based on level",
					]

					verifications: [
						{
							description: "New Pokemon created successfully"
							criteria: [
								"ID matches format pkmn_[0-9]+",
								"Name matches provided name",
								"Type matches provided type",
								"Level matches provided level",
								"HP equals level * 4",
								"Trainer ID matches provided trainer",
							]
							examples: [
								{
									input: {
										name:       "Squirtle"
										type:       "Water"
										level:      10
										trainer_id: "trainer_ash"
									}
									output: {
										id:         "pkmn_004"
										name:       "Squirtle"
										type:       "Water"
										level:      10
										hp:         40
										trainer_id: "trainer_ash"
									}
								}
							]
						}
					]

					notes: """
						HP is calculated server-side as level * 4.
						The ID is auto-generated with pkmn_ prefix.
						"""
				},
				{
					name:   "create-pokemon-invalid-level"
					intent: "Creating Pokemon with invalid level fails"

					preconditions: [
						"Pokemon data has level outside valid range",
					]

					postconditions: [
						"Creation is rejected",
						"No Pokemon is created",
					]

					verifications: [
						{
							description: "Invalid level returns validation error"
							criteria: [
								"Error code is INVALID_LEVEL",
							]
						}
					]

					notes: "Level must be between 1 and 100"
				},
				{
					name:   "create-pokemon-invalid-trainer"
					intent: "Creating Pokemon for non-existent trainer fails"

					preconditions: [
						"Trainer with provided ID does not exist",
					]

					postconditions: [
						"Creation is rejected",
						"No Pokemon is created",
					]

					verifications: [
						{
							description: "Invalid trainer returns validation error"
							criteria: [
								"Error code is TRAINER_NOT_FOUND",
							]
						}
					]
				},
			]
		},
	]

	invariants: [
		{
			name: "structured-errors"
			description: "All error responses have consistent structure"
			criteria: [
				"Error responses include error.code field",
				"Error responses include error.message field",
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
			name: "sequential-ids"
			description: "IDs should not be plain sequential integers"

			bad_example: {
				id: 1
			}

			good_example: {
				id: "pkmn_001"
			}

			why: """
				Sequential IDs reveal business metrics and enable enumeration
				attacks. Use prefixed identifiers instead.
				"""
		},
	]

	ai_hints: {
		implementation: {
			suggested_stack: ["Python", "Flask", "SQLite"]
		}

		entities: {
			pokemon: {
				fields: {
					id:         "string, prefixed 'pkmn_', auto-generated"
					name:       "string, 1-50 chars"
					type:       "string, one of predefined types"
					level:      "integer, 1-100"
					hp:         "integer, calculated as level * 4"
					trainer_id: "string, foreign key to trainer"
				}
			}
			trainer: {
				fields: {
					id:   "string, prefixed 'trainer_'"
					name: "string, 1-100 chars"
				}
			}
		}

		pitfalls: [
			"Don't use sequential integer IDs",
			"Don't forget to validate trainer exists before creating Pokemon",
			"Don't allow levels outside 1-100 range",
		]
	}
}
