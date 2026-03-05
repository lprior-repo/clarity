/// Batch processing for multiple specs
/// Processes multiple spec files and generates summary reports
import gleam/int
import gleam/io
import gleam/list
import gleam/result
import gleam/string
import gleam_community/ansi
import intent/cli_ui
import intent/loader
import intent/quality_analyzer
import intent/security
import simplifile

/// Batch processing configuration
pub type BatchConfig {
  BatchConfig(
    output_dir: String,
    parallel: Bool,
    verbose: Bool,
    continue_on_error: Bool,
  )
}

/// Result of processing a single spec
pub type SpecResult {
  SpecResult(
    file: String,
    status: BatchStatus,
    behaviors_count: Int,
    quality_score: Int,
    error: String,
  )
}

/// Batch processing status
pub type BatchStatus {
  Success
  Failed
  Skipped
}

/// Summary report for batch processing
pub type BatchSummary {
  BatchSummary(
    total_files: Int,
    successful: Int,
    failed: Int,
    skipped: Int,
    total_behaviors: Int,
    average_quality: Int,
    results: List(SpecResult),
  )
}

/// Process multiple spec files sequentially
pub fn process_specs(files: List(String), config: BatchConfig) -> BatchSummary {
  case files {
    [] -> {
      cli_ui.print_error("No spec files provided")
      BatchSummary(
        total_files: 0,
        successful: 0,
        failed: 0,
        skipped: 0,
        total_behaviors: 0,
        average_quality: 0,
        results: [],
      )
    }
    _ -> {
      cli_ui.print_header("Batch Processing")
      io.println(
        "Processing " <> int.to_string(list.length(files)) <> " file(s)...",
      )
      io.println("")

      // Process with index tracking
      let results =
        list.index_map(files, fn(file, index) {
          let position = index + 1
          process_single_spec(file, position, list.length(files), config)
        })

      let summary = generate_summary(results)
      display_summary(summary, config)

      summary
    }
  }
}

/// Process a single spec file
fn process_single_spec(
  file: String,
  position: Int,
  total: Int,
  config: BatchConfig,
) -> SpecResult {
  // Display progress
  let progress =
    "[" <> int.to_string(position) <> "/" <> int.to_string(total) <> "]"

  io.println(ansi.cyan(progress) <> " Processing: " <> file <> "...")

  // Validate file path for security
  case security.validate_file_path(file) {
    Error(_) -> {
      cli_ui.print_error("  ✗ Invalid file path")
      SpecResult(
        file: file,
        status: Failed,
        behaviors_count: 0,
        quality_score: 0,
        error: "Invalid file path",
      )
    }
    Ok(validated_path) -> {
      // Check if file exists
      case simplifile.verify_is_file(validated_path) {
        Ok(False) -> {
          cli_ui.print_error("  ✗ File not found")
          SpecResult(
            file: file,
            status: Skipped,
            behaviors_count: 0,
            quality_score: 0,
            error: "File not found",
          )
        }
        Error(_) -> {
          cli_ui.print_error("  ✗ Cannot access file")
          SpecResult(
            file: file,
            status: Failed,
            behaviors_count: 0,
            quality_score: 0,
            error: "Cannot access file",
          )
        }
        Ok(True) -> {
          // Load and analyze spec
          case loader.load_spec_quiet(validated_path) {
            Ok(spec) -> {
              // Count behaviors
              let behaviors_count =
                spec.features
                |> list.flat_map(fn(f) { f.behaviors })
                |> list.length

              // Calculate quality score
              let quality_report = quality_analyzer.analyze_spec(spec)

              let quality_score = quality_report.overall_score

              // Display result
              let behaviors_str =
                int.to_string(behaviors_count) <> " behavior(s)"
              let quality_str =
                "Quality: " <> int.to_string(quality_score) <> "/100"

              case config.verbose {
                True -> {
                  io.println(
                    "  "
                    <> ansi.green("✓")
                    <> " Success - "
                    <> behaviors_str
                    <> ", "
                    <> quality_str,
                  )

                  // Show quality breakdown
                  case quality_report.issues {
                    [] -> Nil
                    issues -> {
                      io.println(
                        "    Issues: " <> int.to_string(list.length(issues)),
                      )
                    }
                  }
                }
                False -> {
                  io.println(
                    "  "
                    <> ansi.green("✓")
                    <> " "
                    <> behaviors_str
                    <> " ("
                    <> quality_str
                    <> ")",
                  )
                }
              }

              SpecResult(
                file: file,
                status: Success,
                behaviors_count: behaviors_count,
                quality_score: quality_score,
                error: "",
              )
            }
            Error(error) -> {
              let error_msg = loader.format_error(error)

              case config.continue_on_error {
                True -> {
                  cli_ui.print_warning("  ⚠ Failed (continuing)")
                  case config.verbose {
                    True -> {
                      io.println(
                        "    Error: "
                        <> string.split(error_msg, "\n")
                        |> list.first
                        |> result.unwrap("Unknown error"),
                      )
                    }
                    False -> Nil
                  }
                }
                False -> {
                  cli_ui.print_error("  ✗ Failed")
                  case config.verbose {
                    True -> {
                      io.println(
                        "    Error: "
                        <> string.split(error_msg, "\n")
                        |> list.first
                        |> result.unwrap("Unknown error"),
                      )
                    }
                    False -> Nil
                  }
                }
              }

              SpecResult(
                file: file,
                status: case config.continue_on_error {
                  True -> Skipped
                  False -> Failed
                },
                behaviors_count: 0,
                quality_score: 0,
                error: error_msg,
              )
            }
          }
        }
      }
    }
  }
}

/// Generate summary from results
fn generate_summary(results: List(SpecResult)) -> BatchSummary {
  let successful =
    results
    |> list.filter(fn(r) { r.status == Success })
    |> list.length

  let failed =
    results
    |> list.filter(fn(r) { r.status == Failed })
    |> list.length

  let skipped =
    results
    |> list.filter(fn(r) { r.status == Skipped })
    |> list.length

  let total_behaviors =
    results
    |> list.filter(fn(r) { r.status == Success })
    |> list.map(fn(r) { r.behaviors_count })
    |> list.fold(from: 0, with: int.add)

  let successful_results =
    results
    |> list.filter(fn(r) { r.status == Success })

  let average_quality = case list.length(successful_results) {
    0 -> 0
    count -> {
      let sum =
        successful_results
        |> list.map(fn(r) { r.quality_score })
        |> list.fold(from: 0, with: int.add)

      sum / count
    }
  }

  BatchSummary(
    total_files: list.length(results),
    successful: successful,
    failed: failed,
    skipped: skipped,
    total_behaviors: total_behaviors,
    average_quality: average_quality,
    results: results,
  )
}

/// Display summary report
fn display_summary(summary: BatchSummary, config: BatchConfig) {
  io.println("")
  cli_ui.print_header("Batch Summary")

  // Overall stats
  io.println("Total files: " <> int.to_string(summary.total_files))
  io.println(
    "  "
    <> ansi.green("✓")
    <> " Successful: "
    <> int.to_string(summary.successful),
  )

  case summary.failed > 0 {
    True -> {
      io.println(
        "  " <> ansi.red("✗") <> " Failed: " <> int.to_string(summary.failed),
      )
    }
    False -> Nil
  }

  case summary.skipped > 0 {
    True -> {
      io.println(
        "  "
        <> ansi.yellow("⚠")
        <> " Skipped: "
        <> int.to_string(summary.skipped),
      )
    }
    False -> Nil
  }

  // Behaviors and quality
  case summary.successful > 0 {
    True -> {
      io.println("")
      io.println("Total behaviors: " <> int.to_string(summary.total_behaviors))
      io.println(
        "Average quality: " <> int.to_string(summary.average_quality) <> "/100",
      )

      // Quality rating
      let rating = case summary.average_quality {
        score if score >= 90 -> ansi.green("Excellent")
        score if score >= 75 -> ansi.green("Good")
        score if score >= 60 -> ansi.yellow("Fair")
        _ -> ansi.red("Poor")
      }
      io.println("Quality rating: " <> rating)
    }
    False -> Nil
  }

  // Detailed results if verbose
  case config.verbose {
    True -> {
      io.println("")
      io.println(ansi.bold("Detailed Results:"))
      io.println("")

      list.each(summary.results, fn(result) {
        let status_icon = case result.status {
          Success -> ansi.green("✓")
          Failed -> ansi.red("✗")
          Skipped -> ansi.yellow("⊘")
        }

        let status_str = case result.status {
          Success -> "Success"
          Failed -> "Failed"
          Skipped -> "Skipped"
        }

        io.println(status_icon <> " " <> result.file <> " - " <> status_str)

        case result.status == Success {
          True -> {
            io.println(
              "    Behaviors: "
              <> int.to_string(result.behaviors_count)
              <> ", Quality: "
              <> int.to_string(result.quality_score)
              <> "/100",
            )
          }
          False -> {
            case result.error {
              "" -> Nil
              err -> {
                let first_line =
                  string.split(err, "\n")
                  |> list.first
                  |> result.unwrap("Unknown error")

                io.println("    Error: " <> first_line)
              }
            }
          }
        }

        io.println("")
      })
    }
    False -> Nil
  }

  // Failed files summary
  let failed_results =
    summary.results
    |> list.filter(fn(r) { r.status == Failed || r.status == Skipped })

  case list.length(failed_results) > 0 && !config.verbose {
    True -> {
      io.println("")
      io.println(ansi.bold("Failed/Skipped Files:"))
      list.each(failed_results, fn(result) {
        io.println("  • " <> result.file)
      })
    }
    False -> Nil
  }

  // Success/failure message
  io.println("")
  case summary.failed {
    0 -> {
      cli_ui.print_success("All files processed successfully!")
    }
    _ -> {
      cli_ui.print_warning(
        "Processing complete with "
        <> int.to_string(summary.failed)
        <> " error(s)",
      )
    }
  }
}

/// Get spec files from directory
pub fn get_specs_from_dir(dir: String) -> Result(List(String), String) {
  // First check if it's a valid directory
  case simplifile.verify_is_directory(dir) {
    Ok(False) -> Error("Not a directory: " <> dir)
    Ok(True) -> {
      case simplifile.read_directory(dir) {
        Ok(files) -> {
          let cue_files =
            files
            |> list.filter(fn(f) {
              string.ends_with(f, ".cue") && !string.starts_with(f, ".")
            })
            |> list.map(fn(f) {
              // Ensure we return absolute paths
              case string.starts_with(f, "/") {
                True -> f
                False -> dir <> "/" <> f
              }
            })

          case list.is_empty(cue_files) {
            True -> Error("No .cue files found in: " <> dir)
            False -> Ok(cue_files)
          }
        }
        Error(_) -> Error("Failed to read directory: " <> dir)
  }
      }
    Error(_) -> Error("Cannot access directory: " <> dir)
  }
}

/// Format batch status as string
pub fn status_to_string(status: BatchStatus) -> String {
  case status {
    Success -> "success"
    Failed -> "failed"
    Skipped -> "skipped"
  }
}
