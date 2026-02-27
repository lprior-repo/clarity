/// Behavior dependency resolver
/// Topologically sorts behaviors based on their `requires` dependencies
import gleam/dict.{type Dict}
import gleam/list
import gleam/result
import gleam/set.{type Set}
import intent/types.{type Behavior, type Spec}

/// A resolved behavior with its feature context
pub type ResolvedBehavior {
  ResolvedBehavior(feature_name: String, behavior: Behavior)
}

/// Error types for resolution
pub type ResolveError {
  CyclicDependency(behaviors: List(String))
  MissingDependency(behavior: String, missing: String)
  DuplicateBehaviorName(name: String)
}

/// Resolve all behaviors in a spec into execution order
pub fn resolve_execution_order(
  spec: Spec,
) -> Result(List(ResolvedBehavior), ResolveError) {
  // First, collect all behaviors with their feature context
  let all_behaviors = collect_all_behaviors(spec)

  // Check for duplicate names
  use _ <- result.try(check_duplicates(all_behaviors))

  // Build dependency graph
  use graph <- result.try(build_dependency_graph(all_behaviors))

  // Topological sort
  topological_sort(all_behaviors, graph)
}

/// Filter behaviors by a specific feature name
pub fn filter_by_feature(
  behaviors: List(ResolvedBehavior),
  feature_name: String,
) -> List(ResolvedBehavior) {
  list.filter(behaviors, fn(rb) { rb.feature_name == feature_name })
}

/// Filter behaviors by a specific behavior name
pub fn filter_by_name(
  behaviors: List(ResolvedBehavior),
  name: String,
) -> List(ResolvedBehavior) {
  list.filter(behaviors, fn(rb) { rb.behavior.name == name })
}

/// Get behaviors that depend on a given behavior
pub fn get_dependents(
  behaviors: List(ResolvedBehavior),
  name: String,
) -> List(ResolvedBehavior) {
  list.filter(behaviors, fn(rb) { list.contains(rb.behavior.requires, name) })
}

fn collect_all_behaviors(spec: Spec) -> List(ResolvedBehavior) {
  spec.features
  |> list.flat_map(fn(feature) {
    feature.behaviors
    |> list.map(fn(behavior) {
      ResolvedBehavior(feature_name: feature.name, behavior: behavior)
    })
  })
}

fn check_duplicates(
  behaviors: List(ResolvedBehavior),
) -> Result(Nil, ResolveError) {
  let names = list.map(behaviors, fn(rb) { rb.behavior.name })
  case check_duplicates_loop(names, set.new(), []) {
    Ok(_) -> Ok(Nil)
    Error(e) -> Error(e)
  }
}

fn check_duplicates_loop(
  names: List(String),
  seen: Set(String),
  acc: List(String),
) -> Result(List(String), ResolveError) {
  case names, acc {
    [], _ -> Ok(list.reverse(acc))
    [name, ..rest], _ ->
      case set.contains(seen, name) {
        True -> Error(DuplicateBehaviorName(name))
        False ->
          check_duplicates_loop(rest, set.insert(seen, name), [name, ..acc])
      }
  }
}

fn build_dependency_graph(
  behaviors: List(ResolvedBehavior),
) -> Result(Dict(String, List(String)), ResolveError) {
  let behavior_names =
    behaviors
    |> list.map(fn(rb) { rb.behavior.name })
    |> set.from_list

  behaviors
  |> list.try_fold(dict.new(), fn(graph, rb) {
    // Verify all dependencies exist
    use _ <- result.try(
      list.try_each(rb.behavior.requires, fn(dep) {
        case set.contains(behavior_names, dep) {
          True -> Ok(Nil)
          False -> Error(MissingDependency(rb.behavior.name, dep))
        }
      }),
    )

    Ok(dict.insert(graph, rb.behavior.name, rb.behavior.requires))
  })
}

/// Topological sort using Kahn's algorithm
fn topological_sort(
  behaviors: List(ResolvedBehavior),
  graph: Dict(String, List(String)),
) -> Result(List(ResolvedBehavior), ResolveError) {
  // Build a map from name to ResolvedBehavior
  let by_name =
    behaviors
    |> list.map(fn(rb) { #(rb.behavior.name, rb) })
    |> dict.from_list

  // Calculate in-degrees
  let in_degrees = calculate_in_degrees(behaviors, graph)

  // Find all nodes with in-degree 0
  let initial_queue =
    in_degrees
    |> dict.filter(fn(_, degree) { degree == 0 })
    |> dict.keys

  kahn_loop(initial_queue, in_degrees, graph, by_name, [])
}

fn calculate_in_degrees(
  behaviors: List(ResolvedBehavior),
  graph: Dict(String, List(String)),
) -> Dict(String, Int) {
  // Initialize all to 0
  let initial =
    behaviors
    |> list.map(fn(rb) { #(rb.behavior.name, 0) })
    |> dict.from_list

  // Count incoming edges (how many dependencies a node has)
  // A node's in-degree is the number of things it depends on
  dict.fold(graph, initial, fn(degrees, node, deps) {
    // The node's in-degree is the number of dependencies it has
    dict.insert(degrees, node, list.length(deps))
  })
}

fn kahn_loop(
  queue: List(String),
  in_degrees: Dict(String, Int),
  graph: Dict(String, List(String)),
  by_name: Dict(String, ResolvedBehavior),
  result: List(ResolvedBehavior),
) -> Result(List(ResolvedBehavior), ResolveError) {
  case queue {
    [] -> {
      // Check if all nodes were processed
      let remaining =
        in_degrees
        |> dict.filter(fn(_, degree) { degree > 0 })
        |> dict.keys

      case remaining {
        [] -> Ok(list.reverse(result))
        _ -> Error(CyclicDependency(remaining))
      }
    }
    [node, ..rest_queue] -> {
      // Get the behavior for this node
      case dict.get(by_name, node) {
        Ok(rb) -> {
          // Update in-degrees for dependents
          let #(new_degrees, new_ready) =
            update_in_degrees_for_dependents(node, graph, in_degrees)

          // Add newly ready nodes to queue
          let updated_queue = list.append(rest_queue, new_ready)

          kahn_loop(updated_queue, new_degrees, graph, by_name, [rb, ..result])
        }
        Error(_) -> {
          // This shouldn't happen, skip
          kahn_loop(rest_queue, in_degrees, graph, by_name, result)
        }
      }
    }
  }
}

fn update_in_degrees_for_dependents(
  completed: String,
  graph: Dict(String, List(String)),
  in_degrees: Dict(String, Int),
) -> #(Dict(String, Int), List(String)) {
  // Find all nodes that depend on the completed node
  let dependents =
    graph
    |> dict.filter(fn(_, deps) { list.contains(deps, completed) })
    |> dict.keys

  // Decrement their in-degrees
  let #(new_degrees, newly_ready) =
    list.fold(dependents, #(in_degrees, []), fn(acc, dep) {
      let #(degrees, ready) = acc
      case dict.get(degrees, dep) {
        Ok(count) -> {
          let new_count = count - 1
          let new_degrees = dict.insert(degrees, dep, new_count)
          case new_count == 0 {
            True -> #(new_degrees, [dep, ..ready])
            False -> #(new_degrees, ready)
          }
        }
        Error(_) -> acc
      }
    })

  #(new_degrees, newly_ready)
}

/// Format a resolve error as a human-readable string
pub fn format_error(error: ResolveError) -> String {
  case error {
    CyclicDependency(behaviors) ->
      "Cyclic dependency detected involving: "
      <> list_to_string(behaviors, ", ")
    MissingDependency(behavior, missing) ->
      "Behavior '"
      <> behavior
      <> "' requires '"
      <> missing
      <> "' which does not exist"
    DuplicateBehaviorName(name) -> "Duplicate behavior name: " <> name
  }
}

fn list_to_string(items: List(String), sep: String) -> String {
  case items {
    [] -> ""
    [item] -> item
    [item, ..rest] -> item <> sep <> list_to_string(rest, sep)
  }
}
