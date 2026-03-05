/// Simple benchmarking utilities for Intent CLI performance testing
import gleam/int
import gleam/io
import gleam/list
import gleam/result
import gleam/string

/// Measure execution time of a function
pub fn benchmark(name: String, func: fn() -> Nil) {
  let start = get_current_time()

  func()

  let end = get_current_time()
  let elapsed_ms = end - start

  io.println(name <> ": " <> int.to_string(elapsed_ms) <> "ms")
}

/// Measure execution time of a function that returns a value
pub fn benchmark_return(name: String, func: fn() -> a) -> a {
  let start = get_current_time()

  let result = func()

  let end = get_current_time()
  let elapsed_ms = end - start

  io.println(name <> ": " <> int.to_string(elapsed_ms) <> "ms")

  result
}

/// Run a benchmark multiple times and return average
pub fn benchmark_avg(name: String, iterations: Int, func: fn() -> a) -> a {
  let assert [_, ..] = list.range(1, iterations)
  let results = list.map(list.range(1, iterations), fn(_) { func() })

  let _ =
    benchmark(name, fn() {
      // Already ran above, just report
      Nil
    })

  // Return the last result
  list.last(results)
  |> result.unwrap(panic("benchmark_avg: iterations must be > 0"))
}

/// Format benchmark results for comparison
pub fn format_comparison(name: String, before_ms: Int, after_ms: Int) -> String {
  let improvement = before_ms - after_ms
  let percentage = case before_ms > 0 {
    True ->
      float.to_int(float.round(
        improvement /. float.from_int(before_ms) *. 100.0,
      ))
    False -> 0
  }

  let arrow = case improvement > 0 {
    True -> "↓"
    False -> "↑"
  }

  name
  <> ": "
  <> int.to_string(before_ms)
  <> "ms → "
  <> int.to_string(after_ms)
  <> "ms "
  <> arrow
  <> " "
  <> int.to_string(improvement)
  <> "ms ("
  <> int.to_string(percentage)
  <> "%)"
}

/// Get current time in milliseconds
@external(erlang, "erlang", "monotonic_time")
fn get_current_time() -> Int
