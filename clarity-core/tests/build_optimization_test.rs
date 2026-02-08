//! Build Optimization Tests (LTO + PGO)
//!
//! These tests verify that release builds are properly optimized.
//! See bead bd-2m1 and docs/BUILD_OPTIMIZATION.md for details.

use std::path::PathBuf;

/// Test 1: LTO should be enabled in release profile
#[test]
fn test_lto_enabled_in_release() -> Result<(), Box<dyn std::error::Error>> {
  // Try workspace Cargo.toml first, then local
  let cargo_toml =
    std::fs::read_to_string("../Cargo.toml").or_else(|_| std::fs::read_to_string("Cargo.toml"))?;

  // Verify LTO is enabled
  assert!(
    cargo_toml.contains("lto = true")
      || cargo_toml.contains("lto = \"thin\"")
      || cargo_toml.contains("lto = \"fat\""),
    "Cargo.toml should enable LTO in release profile"
  );
  Ok(())
}

/// Test 2: Codegen units should be optimized for release
#[test]
fn test_codegen_units_optimized() -> Result<(), Box<dyn std::error::Error>> {
  let cargo_toml = std::fs::read_to_string("../Cargo.toml")?;

  // Check if codegen-units is set to 1 for release (max optimization)
  assert!(
    cargo_toml.contains("codegen-units = 1"),
    "Release profile should use codegen-units = 1 for maximum optimization.\n\
         This provides the best runtime performance at the cost of longer compilation time."
  );
  Ok(())
}

/// Test 3: Strip symbols should be enabled
#[test]
fn test_strip_symbols_enabled() -> Result<(), Box<dyn std::error::Error>> {
  let cargo_toml = std::fs::read_to_string("../Cargo.toml")?;

  // Verify strip is enabled
  assert!(
    cargo_toml.contains("strip = true"),
    "Cargo.toml should enable strip in release profile to reduce binary size"
  );
  Ok(())
}

/// Test 4: Opt-level should be 3 for release
#[test]
fn test_opt_level_maximized() -> Result<(), Box<dyn std::error::Error>> {
  let cargo_toml = std::fs::read_to_string("../Cargo.toml")?;

  // Verify opt-level is 3
  assert!(
    cargo_toml.contains("opt-level = 3"),
    "Release profile should use opt-level = 3 for maximum optimization"
  );
  Ok(())
}

/// Test 5: CI profile should exist with thin LTO
#[test]
fn test_ci_profile_exists() -> Result<(), Box<dyn std::error::Error>> {
  let cargo_toml = std::fs::read_to_string("../Cargo.toml")?;

  // Verify CI profile exists
  assert!(
    cargo_toml.contains("[profile.ci]"),
    "Cargo.toml should define a [profile.ci] for faster CI builds with thin LTO"
  );

  // Verify CI profile uses thin LTO
  assert!(
    cargo_toml.contains("lto = \"thin\""),
    "CI profile should use thin LTO for faster compilation"
  );
  Ok(())
}

/// Test 6: PGO profiles should exist
#[test]
fn test_pgo_profiles_exist() -> Result<(), Box<dyn std::error::Error>> {
  let cargo_toml = std::fs::read_to_string("../Cargo.toml")?;

  // Verify PGO instrumentation profile exists
  assert!(
    cargo_toml.contains("[profile.pgo-instrument]"),
    "Cargo.toml should define a [profile.pgo-instrument] for PGO workflow"
  );

  // Verify PGO optimized profile exists
  assert!(
    cargo_toml.contains("[profile.pgo-optimized]"),
    "Cargo.toml should define a [profile.pgo-optimized] for PGO workflow"
  );
  Ok(())
}

/// Test 7: Cargo config should exist for target-specific optimizations
#[test]
fn test_cargo_config_exists() -> Result<(), Box<dyn std::error::Error>> {
  let config_path = PathBuf::from("../.cargo/config.toml");
  assert!(
    config_path.exists(),
    ".cargo/config.toml should exist for target-specific optimizations and PGO configuration"
  );

  let config = std::fs::read_to_string(&config_path)?;

  // Verify it has documentation about PGO
  assert!(
    config.contains("PGO") || config.contains("profile-generate") || config.contains("profile-use"),
    ".cargo/config.toml should document PGO workflow"
  );
  Ok(())
}

/// Test 8: Moon tasks should include PGO workflow
#[test]
fn test_moon_pgo_tasks_exist() -> Result<(), Box<dyn std::error::Error>> {
  let moon_tasks = std::fs::read_to_string("../.moon/tasks.yml")?;

  // Verify PGO instrumentation task exists
  assert!(
    moon_tasks.contains("pgo-instrument"),
    ".moon/tasks.yml should define a pgo-instrument task"
  );

  // Verify PGO build task exists
  assert!(
    moon_tasks.contains("pgo-build"),
    ".moon/tasks.yml should define a pgo-build task"
  );

  // Verify PGO optimize task exists
  assert!(
    moon_tasks.contains("pgo-optimize"),
    ".moon/tasks.yml should define a pgo-optimize task"
  );
  Ok(())
}

/// Test 9: PGO workload script should exist
#[test]
fn test_pgo_workload_script_exists() -> Result<(), Box<dyn std::error::Error>> {
  let script_path = PathBuf::from("../.moon/scripts/pgo-workload.sh");
  assert!(
    script_path.exists(),
    ".moon/scripts/pgo-workload.sh should exist to run representative workload for PGO"
  );

  // Verify script is executable
  let metadata = std::fs::metadata(&script_path)?;
  let permissions = metadata.permissions();
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mode = permissions.mode();
    assert!(
      mode & 0o111 != 0,
      "PGO workload script should be executable (chmod +x)"
    );
  }
  Ok(())
}

/// Test 10: Documentation should exist
#[test]
fn test_optimization_documentation_exists() -> Result<(), Box<dyn std::error::Error>> {
  let doc_path = PathBuf::from("../docs/BUILD_OPTIMIZATION.md");
  assert!(
    doc_path.exists(),
    "docs/BUILD_OPTIMIZATION.md should exist to document LTO and PGO workflow"
  );

  let docs = std::fs::read_to_string(&doc_path)?;

  // Verify documentation covers key topics
  assert!(
    docs.contains("LTO") && docs.contains("PGO"),
    "Documentation should cover both LTO and PGO"
  );

  assert!(
    docs.contains("bd-2m1"),
    "Documentation should reference the implementation bead"
  );
  Ok(())
}
