/// Configuration file support for Intent CLI
/// Loads and parses .intentrc.yaml configuration file
import glaml.{type Node, NodeInt, NodeMap, NodeStr, document_root, parse_string}
import gleam/int
import gleam/list
import gleam/result
import simplifile

/// Configuration file path
const config_file_path = ".intentrc.yaml"

/// ============================================================================
/// CONFIG TYPES
/// ============================================================================
/// Intent CLI configuration from .intentrc.yaml
pub type Config {
  Config(
    default_profile: String,
    default_output_format: String,
    default_strategy: String,
    watch_debounce_ms: Int,
    max_cache_entries: Int,
  )
}

/// Default configuration values
fn default_config() -> Config {
  Config(
    default_profile: "api",
    default_output_format: "json",
    default_strategy: "page_rank",
    watch_debounce_ms: 500,
    max_cache_entries: 50,
  )
}

/// ============================================================================
/// CONFIG LOADING
/// ============================================================================
/// Load configuration from .intentrc.yaml in current directory
/// Returns Ok(Config) if file exists and is valid, Ok(default_config()) if not found
/// Returns Error(String) if file exists but is invalid
pub fn load_config() -> Result(Config, String) {
  case simplifile.verify_is_file(config_file_path) {
    Ok(True) -> {
      simplifile.read(config_file_path)
      |> result.map_error(fn(_err) { "Failed to read config file" })
      |> result.then(parse_yaml_config)
    }
    Ok(False) -> Ok(default_config())
    Error(_err) -> {
      // File doesn't exist or can't be accessed - use defaults silently
      Ok(default_config())
    }
  }
}

/// ============================================================================
/// YAML PARSING
/// ============================================================================
/// Parse YAML configuration content
fn parse_yaml_config(yaml_content: String) -> Result(Config, String) {
  case parse_string(yaml_content) {
    Error(_err) -> {
      Error("Failed to parse YAML configuration file")
    }
    Ok(docs) -> {
      // Take the first document
      case docs {
        [] -> Error("Empty YAML configuration file")
        [doc, ..] -> {
          let root_node = document_root(doc)
          decode_config(root_node)
        }
      }
    }
  }
}

/// Decode configuration from parsed YAML
fn decode_config(yaml: Node) -> Result(Config, String) {
  case yaml {
    NodeMap(map) -> {
      // Helper function to find a string key in the map and extract its string value
      let find_string = fn(key: String, default: String) {
        list.find(map, fn(pair) {
          case pair.0 {
            NodeStr(k) -> k == key
            _ -> False
          }
        })
        |> result.map(fn(pair) {
          case pair.1 {
            NodeStr(s) -> s
            _ -> default
          }
        })
        |> result.unwrap(default)
      }

      // Helper function to find a string key in the map and extract its int value
      let find_int = fn(key: String, default: Int) {
        list.find(map, fn(pair) {
          case pair.0 {
            NodeStr(k) -> k == key
            _ -> False
          }
        })
        |> result.map(fn(pair) {
          case pair.1 {
            NodeInt(i) -> i
            _ -> default
          }
        })
        |> result.unwrap(default)
      }

      let default_profile = find_string("default_profile", "api")
      let default_output_format = find_string("default_output_format", "json")
      let default_strategy = find_string("default_strategy", "page_rank")
      let watch_debounce_ms = find_int("watch_debounce_ms", 500)
      let max_cache_entries = find_int("max_cache_entries", 50)

      let config =
        Config(
          default_profile: default_profile,
          default_output_format: default_output_format,
          default_strategy: default_strategy,
          watch_debounce_ms: watch_debounce_ms,
          max_cache_entries: max_cache_entries,
        )

      validate_config(config)
    }
    _ -> Error("YAML configuration must be a map/object")
  }
}

/// ============================================================================
/// CONFIG VALIDATION
/// ============================================================================
/// Validate configuration values
fn validate_config(config: Config) -> Result(Config, String) {
  // Validate default_profile
  let valid_profiles = ["api", "cli", "event", "data", "workflow", "ui"]
  let profile_valid =
    list.contains(valid_profiles, config.default_profile)

  let profile_error =
    case profile_valid {
      True -> Ok(Nil)
      False ->
        Error(
          "Invalid default_profile: "
            <> config.default_profile
            <> ". Valid options: api, cli, event, data, workflow, ui",
        )
    }

  // Validate default_output_format
  let valid_formats = ["json", "text", "markdown"]
  let format_valid =
    list.contains(valid_formats, config.default_output_format)

  let format_error =
    case format_valid {
      True -> Ok(Nil)
      False ->
        Error(
          "Invalid default_output_format: "
            <> config.default_output_format
            <> ". Valid options: json, text, markdown",
        )
    }

  // Validate default_strategy
  let valid_strategies = ["page_rank", "effort_ease", "dependency_order"]
  let strategy_valid =
    list.contains(valid_strategies, config.default_strategy)

  let strategy_error =
    case strategy_valid {
      True -> Ok(Nil)
      False ->
        Error(
          "Invalid default_strategy: "
            <> config.default_strategy
            <> ". Valid options: page_rank, effort_ease, dependency_order",
        )
    }

  // Validate watch_debounce_ms
  let debounce_error =
    case config.watch_debounce_ms {
      n if n >= 0 && n <= 60000 -> Ok(Nil)
      _ ->
        Error(
          "Invalid watch_debounce_ms: "
            <> int.to_string(config.watch_debounce_ms)
            <> ". Must be between 0 and 60000",
        )
    }

  // Validate max_cache_entries
  let cache_error =
    case config.max_cache_entries {
      n if n >= 0 && n <= 10000 -> Ok(Nil)
      _ ->
        Error(
          "Invalid max_cache_entries: "
            <> int.to_string(config.max_cache_entries)
            <> ". Must be between 0 and 10000",
        )
    }

  // Return first error or Ok(config)
  case profile_error {
    Error(e) -> Error(e)
    Ok(_) ->
      case format_error {
        Error(e) -> Error(e)
        Ok(_) ->
          case strategy_error {
            Error(e) -> Error(e)
            Ok(_) ->
              case debounce_error {
                Error(e) -> Error(e)
                Ok(_) ->
                  case cache_error {
                    Error(e) -> Error(e)
                    Ok(_) -> Ok(config)
                  }
              }
          }
      }
  }
}

/// ============================================================================
/// CONFIG HELPERS
/// ============================================================================
/// Get default profile from config, falling back to CLI argument
pub fn get_profile(config: Config, cli_arg: String) -> String {
  case cli_arg {
    "" -> config.default_profile
    _ -> cli_arg
  }
}

/// Get default output format from config, falling back to CLI argument
pub fn get_output_format(config: Config, cli_arg: String) -> String {
  case cli_arg {
    "" -> config.default_output_format
    _ -> cli_arg
  }
}

/// Get default strategy from config, falling back to CLI argument
pub fn get_strategy(config: Config, cli_arg: String) -> String {
  case cli_arg {
    "" -> config.default_strategy
    _ -> cli_arg
  }
}

/// Check if config file exists
pub fn config_file_exists() -> Bool {
  case simplifile.verify_is_file(config_file_path) {
    Ok(True) -> True
    _ -> False
  }
}
