/// Pokemon API - Test API for Intent CLI validation
/// A simple REST API built in Gleam using Erlang's inets module
import gleam/dict.{type Dict}
import gleam/dynamic
import gleam/erlang/process
import gleam/int
import gleam/io
import gleam/json
import gleam/list
import gleam/string

// =============================================================================
// Types
// =============================================================================

pub type Pokemon {
  Pokemon(
    id: String,
    name: String,
    pokemon_type: String,
    level: Int,
    hp: Int,
    trainer_id: String,
  )
}

pub type Trainer {
  Trainer(id: String, name: String, pokemon_ids: List(String))
}

pub type Store {
  Store(
    pokemon: Dict(String, Pokemon),
    trainers: Dict(String, Trainer),
    next_id: Int,
  )
}

// =============================================================================
// Main Entry Point
// =============================================================================

pub fn main() {
  let store = init_store()

  io.println("Starting Pokemon API on http://localhost:8080")

  // Start the HTTP server via Erlang FFI
  let _ = start_http_server(8080, store)

  process.sleep_forever()
}

@external(erlang, "pokemon_server", "start")
fn start_http_server(port: Int, store: Store) -> Result(Nil, String)

// =============================================================================
// Store Initialization
// =============================================================================

pub fn init_store() -> Store {
  let pikachu =
    Pokemon(
      id: "pkmn_001",
      name: "Pikachu",
      pokemon_type: "Electric",
      level: 25,
      hp: 100,
      trainer_id: "trainer_ash",
    )

  let charizard =
    Pokemon(
      id: "pkmn_002",
      name: "Charizard",
      pokemon_type: "Fire",
      level: 50,
      hp: 200,
      trainer_id: "trainer_ash",
    )

  let bulbasaur =
    Pokemon(
      id: "pkmn_003",
      name: "Bulbasaur",
      pokemon_type: "Grass",
      level: 15,
      hp: 80,
      trainer_id: "trainer_misty",
    )

  let ash =
    Trainer(id: "trainer_ash", name: "Ash Ketchum", pokemon_ids: [
      "pkmn_001", "pkmn_002",
    ])

  let misty =
    Trainer(id: "trainer_misty", name: "Misty", pokemon_ids: ["pkmn_003"])

  Store(
    pokemon: dict.from_list([
      #("pkmn_001", pikachu),
      #("pkmn_002", charizard),
      #("pkmn_003", bulbasaur),
    ]),
    trainers: dict.from_list([#("trainer_ash", ash), #("trainer_misty", misty)]),
    next_id: 4,
  )
}

// =============================================================================
// Request Handlers (called from Erlang)
// =============================================================================

pub fn handle_request(
  method: String,
  path: String,
  body: String,
  store: Store,
) -> #(Int, String) {
  let path_parts =
    path
    |> string.split("/")
    |> list.filter(fn(s) { s != "" })

  case method, path_parts {
    "GET", ["pokemon"] -> list_pokemon(store)
    "GET", ["pokemon", id] -> get_pokemon(store, id)
    "POST", ["pokemon"] -> create_pokemon(body, store)
    "GET", ["trainers"] -> list_trainers(store)
    "GET", ["trainers", id] -> get_trainer(store, id)
    "GET", ["trainers", id, "pokemon"] -> get_trainer_pokemon(store, id)
    _, _ -> not_found()
  }
}

// =============================================================================
// Pokemon Endpoints
// =============================================================================

fn list_pokemon(store: Store) -> #(Int, String) {
  let pokemon_list =
    store.pokemon
    |> dict.values
    |> list.map(pokemon_to_json)

  let response =
    json.object([#("pokemon", json.array(pokemon_list, fn(x) { x }))])
  #(200, json.to_string(response))
}

fn get_pokemon(store: Store, id: String) -> #(Int, String) {
  case dict.get(store.pokemon, id) {
    Ok(pokemon) -> #(200, json.to_string(pokemon_to_json(pokemon)))
    Error(_) ->
      error_response(
        404,
        "POKEMON_NOT_FOUND",
        "Pokemon with id '" <> id <> "' not found",
      )
  }
}

fn create_pokemon(body: String, store: Store) -> #(Int, String) {
  case parse_pokemon_create(body) {
    Ok(#(name, pokemon_type, level, trainer_id)) -> {
      case dict.get(store.trainers, trainer_id) {
        Ok(_) -> {
          case level >= 1 && level <= 100 {
            True -> {
              let id = "pkmn_" <> pad_id(store.next_id)
              let pokemon =
                Pokemon(
                  id: id,
                  name: name,
                  pokemon_type: pokemon_type,
                  level: level,
                  hp: level * 4,
                  trainer_id: trainer_id,
                )
              #(201, json.to_string(pokemon_to_json(pokemon)))
            }
            False ->
              error_response(
                400,
                "INVALID_LEVEL",
                "Level must be between 1 and 100",
              )
          }
        }
        Error(_) ->
          error_response(400, "TRAINER_NOT_FOUND", "Trainer not found")
      }
    }
    Error(msg) -> error_response(400, "INVALID_REQUEST", msg)
  }
}

fn parse_pokemon_create(
  body_str: String,
) -> Result(#(String, String, Int, String), String) {
  case json.decode(body_str, dynamic.dynamic) {
    Ok(data) -> {
      let decoder =
        dynamic.decode4(
          fn(name, pokemon_type, level, trainer_id) {
            #(name, pokemon_type, level, trainer_id)
          },
          dynamic.field("name", dynamic.string),
          dynamic.field("type", dynamic.string),
          dynamic.field("level", dynamic.int),
          dynamic.field("trainer_id", dynamic.string),
        )
      case decoder(data) {
        Ok(res) -> Ok(res)
        Error(_) ->
          Error("Missing required fields: name, type, level, trainer_id")
      }
    }
    Error(_) -> Error("Invalid JSON")
  }
}

// =============================================================================
// Trainer Endpoints
// =============================================================================

fn list_trainers(store: Store) -> #(Int, String) {
  let trainer_list =
    store.trainers
    |> dict.values
    |> list.map(trainer_to_json)

  let response =
    json.object([#("trainers", json.array(trainer_list, fn(x) { x }))])
  #(200, json.to_string(response))
}

fn get_trainer(store: Store, id: String) -> #(Int, String) {
  case dict.get(store.trainers, id) {
    Ok(trainer) -> #(200, json.to_string(trainer_to_json(trainer)))
    Error(_) ->
      error_response(
        404,
        "TRAINER_NOT_FOUND",
        "Trainer with id '" <> id <> "' not found",
      )
  }
}

fn get_trainer_pokemon(store: Store, id: String) -> #(Int, String) {
  case dict.get(store.trainers, id) {
    Ok(trainer) -> {
      let pokemon_list =
        trainer.pokemon_ids
        |> list.filter_map(fn(pid) {
          case dict.get(store.pokemon, pid) {
            Ok(p) -> Ok(pokemon_to_json(p))
            Error(_) -> Error(Nil)
          }
        })
      let response =
        json.object([#("pokemon", json.array(pokemon_list, fn(x) { x }))])
      #(200, json.to_string(response))
    }
    Error(_) ->
      error_response(
        404,
        "TRAINER_NOT_FOUND",
        "Trainer with id '" <> id <> "' not found",
      )
  }
}

// =============================================================================
// Response Helpers
// =============================================================================

fn not_found() -> #(Int, String) {
  error_response(404, "NOT_FOUND", "Endpoint not found")
}

fn error_response(status: Int, code: String, message: String) -> #(Int, String) {
  let response =
    json.object([
      #(
        "error",
        json.object([
          #("code", json.string(code)),
          #("message", json.string(message)),
        ]),
      ),
    ])
  #(status, json.to_string(response))
}

// =============================================================================
// JSON Serialization
// =============================================================================

fn pokemon_to_json(pokemon: Pokemon) -> json.Json {
  json.object([
    #("id", json.string(pokemon.id)),
    #("name", json.string(pokemon.name)),
    #("type", json.string(pokemon.pokemon_type)),
    #("level", json.int(pokemon.level)),
    #("hp", json.int(pokemon.hp)),
    #("trainer_id", json.string(pokemon.trainer_id)),
  ])
}

fn trainer_to_json(trainer: Trainer) -> json.Json {
  json.object([
    #("id", json.string(trainer.id)),
    #("name", json.string(trainer.name)),
    #("pokemon_count", json.int(list.length(trainer.pokemon_ids))),
  ])
}

// =============================================================================
// Utilities
// =============================================================================

fn pad_id(n: Int) -> String {
  let s = int.to_string(n)
  case string.length(s) {
    1 -> "00" <> s
    2 -> "0" <> s
    _ -> s
  }
}
