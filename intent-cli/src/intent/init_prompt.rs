//! Interactive prompts for spec initialization
//!
//! Provides guided prompts for project setup with sensible defaults.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use std::path::Path;

/// Project configuration collected from interactive prompts
#[derive(Debug, Clone)]
pub struct ProjectConfig {
  /// Project name
  pub name: String,
  /// Project description
  pub description: String,
  /// Target audience
  pub audience: String,
  /// Template profile (api, cli, data, event, workflow, ui)
  pub profile: String,
  /// Output filename
  pub output_file: String,
  /// Initial version
  pub version: String,
}

impl ProjectConfig {
  /// Returns the default filename based on the project name
  #[must_use]
  pub fn default_filename(&self) -> String {
    let sanitized: String = self
      .name
      .to_lowercase()
      .chars()
      .map(|c| if c.is_alphanumeric() { c } else { '-' })
      .collect();

    let compressed: String = sanitized
      .split('-')
      .filter(|s| !s.is_empty())
      .collect::<Vec<_>>()
      .join("-");

    format!("{compressed}.cue")
  }
}

/// Template information for display
#[derive(Debug, Clone)]
pub struct Template {
  pub id: String,
  pub name: String,
  pub description: String,
}

/// Get all available templates
#[must_use]
pub fn get_templates() -> Vec<Template> {
  vec![
    Template {
      id: "api".to_string(),
      name: "API Service".to_string(),
      description: "REST or GraphQL API service with endpoints, auth, and data models".to_string(),
    },
    Template {
      id: "cli".to_string(),
      name: "CLI Tool".to_string(),
      description: "Command-line interface with commands, flags, and exit codes".to_string(),
    },
    Template {
      id: "data".to_string(),
      name: "Data Pipeline".to_string(),
      description: "Data processing with sources, transformations, and destinations".to_string(),
    },
    Template {
      id: "event".to_string(),
      name: "Event Processor".to_string(),
      description: "Event-driven system with producers, consumers, and handlers".to_string(),
    },
    Template {
      id: "workflow".to_string(),
      name: "Workflow Engine".to_string(),
      description: "Multi-step workflows with states, transitions, and error recovery".to_string(),
    },
    Template {
      id: "ui".to_string(),
      name: "UI Application".to_string(),
      description: "User interface with screens, interactions, and state management".to_string(),
    },
  ]
}

/// Run the interactive project configuration wizard
///
/// # Errors
/// Returns an error if user input cannot be read or validation fails.
pub fn run_interactive_setup(default_name: Option<&str>) -> Result<ProjectConfig> {
  println!();
  println!("===================================================================");
  println!("                   Intent Spec Initialization");
  println!("===================================================================");
  println!();
  println!("This wizard will guide you through creating a new Intent specification.");
  println!("Press Enter to accept defaults, or type your own values.");
  println!();

  // Step 1: Project name
  let name = prompt_project_name(default_name)?;
  println!();

  // Step 2: Project description
  let description = prompt_description(&name)?;
  println!();

  // Step 3: Target audience
  let audience = prompt_audience()?;
  println!();

  // Step 4: Template selection
  let profile = prompt_template()?;
  println!();

  // Step 5: Version
  let version = prompt_version()?;
  println!();

  // Build initial config for confirmation
  let default_filename = generate_filename(&name);
  let config = ProjectConfig {
    name: name.clone(),
    description,
    audience,
    profile,
    output_file: default_filename.clone(),
    version,
  };

  // Step 6: Confirmation and summary
  let final_config = confirm_and_finalize(&config)?;

  Ok(final_config)
}

/// Prompt for project name
fn prompt_project_name(default: Option<&str>) -> Result<String> {
  let prompt_text = "Project name";
  let default_val = default.unwrap_or("My Project");

  let input: String = Input::new()
    .with_prompt(prompt_text)
    .default(default_val.to_string())
    .interact_text()
    .context("Failed to read project name")?;

  let trimmed = input.trim();
  if trimmed.is_empty() {
    return Err(anyhow::anyhow!("Project name cannot be empty"));
  }

  Ok(trimmed.to_string())
}

/// Prompt for project description with smart default
fn prompt_description(name: &str) -> Result<String> {
  let default_desc = format!("{name} specification");

  let input: String = Input::new()
    .with_prompt("Description")
    .default(default_desc)
    .interact_text()
    .context("Failed to read description")?;

  Ok(input.trim().to_string())
}

/// Prompt for target audience
fn prompt_audience() -> Result<String> {
  println!("Who is the primary audience for this project?");
  println!("  Examples: API developers, end users, system administrators");

  let input: String = Input::new()
    .with_prompt("Target audience")
    .default("developers".to_string())
    .interact_text()
    .context("Failed to read target audience")?;

  Ok(input.trim().to_string())
}

/// Prompt for template selection
fn prompt_template() -> Result<String> {
  let templates = get_templates();
  let items: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();

  println!("Select a template profile:");
  let selection = Select::new()
    .with_prompt("Template")
    .items(&items)
    .default(0)
    .interact()
    .context("Failed to select template")?;

  Ok(templates[selection].id.clone())
}

/// Prompt for initial version
fn prompt_version() -> Result<String> {
  let input: String = Input::new()
    .with_prompt("Initial version")
    .default("0.1.0".to_string())
    .interact_text()
    .context("Failed to read version")?;

  let trimmed = input.trim();

  // Basic validation - version should look like semver
  let parts: Vec<&str> = trimmed.split('.').collect();
  if parts.len() < 2 {
    println!("  Note: Version should ideally follow semver (e.g., 0.1.0)");
  }

  Ok(trimmed.to_string())
}

/// Generate default filename from project name
fn generate_filename(name: &str) -> String {
  let sanitized: String = name
    .to_lowercase()
    .chars()
    .map(|c| if c.is_alphanumeric() { c } else { '-' })
    .collect();

  let compressed: String = sanitized
    .split('-')
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("-");

  format!("{compressed}.cue")
}

/// Display summary and confirm, allow edits
fn confirm_and_finalize(config: &ProjectConfig) -> Result<ProjectConfig> {
  loop {
    println!();
    println!("-------------------------------------------------------------------");
    println!("                      Configuration Summary");
    println!("-------------------------------------------------------------------");
    println!();
    println!("  Project Name:     {}", config.name);
    println!("  Description:      {}", config.description);
    println!("  Target Audience:  {}", config.audience);
    println!(
      "  Template:         {} ({})",
      get_template_name(&config.profile),
      config.profile
    );
    println!("  Version:          {}", config.version);
    println!("  Output File:      {}", config.output_file);
    println!();

    let options = [
      "Confirm - Create spec",
      "Edit project name",
      "Edit description",
      "Edit audience",
      "Edit template",
      "Edit version",
      "Edit output filename",
      "Cancel",
    ];

    let selection = Select::new()
      .with_prompt("What would you like to do?")
      .items(&options)
      .default(0)
      .interact()
      .context("Failed to select option")?;

    match selection {
      0 => {
        // Confirm - verify output file doesn't exist (optional warning)
        if Path::new(&config.output_file).exists() {
          let overwrite = Confirm::new()
            .with_prompt(format!(
              "File '{}' already exists. Overwrite?",
              config.output_file
            ))
            .default(false)
            .interact()
            .context("Failed to read confirmation")?;

          if !overwrite {
            continue;
          }
        }
        return Ok(config.clone());
      }
      1 => {
        // Edit name
        let new_name = prompt_project_name(Some(&config.name))?;
        let new_filename = generate_filename(&new_name);
        return Ok(ProjectConfig {
          name: new_name,
          output_file: new_filename,
          ..config.clone()
        });
      }
      2 => {
        // Edit description
        let new_desc = prompt_description(&config.name)?;
        return Ok(ProjectConfig {
          description: new_desc,
          ..config.clone()
        });
      }
      3 => {
        // Edit audience
        let new_audience = prompt_audience()?;
        return Ok(ProjectConfig {
          audience: new_audience,
          ..config.clone()
        });
      }
      4 => {
        // Edit template
        let new_profile = prompt_template()?;
        return Ok(ProjectConfig {
          profile: new_profile,
          ..config.clone()
        });
      }
      5 => {
        // Edit version
        let new_version = prompt_version()?;
        return Ok(ProjectConfig {
          version: new_version,
          ..config.clone()
        });
      }
      6 => {
        // Edit output filename
        let new_filename = prompt_output_filename(&config.output_file)?;
        return Ok(ProjectConfig {
          output_file: new_filename,
          ..config.clone()
        });
      }
      7 => {
        // Cancel
        return Err(anyhow::anyhow!("Initialization cancelled by user"));
      }
      _ => continue,
    }
  }
}

/// Prompt for output filename
fn prompt_output_filename(default: &str) -> Result<String> {
  let input: String = Input::new()
    .with_prompt("Output filename")
    .default(default.to_string())
    .interact_text()
    .context("Failed to read output filename")?;

  let trimmed = input.trim();

  // Add .cue extension if not present
  if trimmed.to_lowercase().ends_with(".cue") {
    Ok(trimmed.to_string())
  } else {
    Ok(format!("{trimmed}.cue"))
  }
}

/// Get human-readable template name from ID
fn get_template_name(id: &str) -> String {
  get_templates()
    .iter()
    .find(|t| t.id == id)
    .map(|t| t.name.clone())
    .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_project_config_default_filename() {
    let config = ProjectConfig {
      name: "My API Project".to_string(),
      description: "Test".to_string(),
      audience: "developers".to_string(),
      profile: "api".to_string(),
      output_file: "test.cue".to_string(),
      version: "0.1.0".to_string(),
    };

    assert_eq!(config.default_filename(), "my-api-project.cue");
  }

  #[test]
  fn test_project_config_default_filename_special_chars() {
    let config = ProjectConfig {
      name: "Test@#$Project!!!".to_string(),
      description: "Test".to_string(),
      audience: "developers".to_string(),
      profile: "api".to_string(),
      output_file: "test.cue".to_string(),
      version: "0.1.0".to_string(),
    };

    assert_eq!(config.default_filename(), "test-project.cue");
  }

  #[test]
  fn test_get_templates() {
    let templates = get_templates();
    assert_eq!(templates.len(), 6);
    assert!(templates.iter().any(|t| t.id == "api"));
    assert!(templates.iter().any(|t| t.id == "cli"));
  }

  #[test]
  fn test_generate_filename() {
    assert_eq!(generate_filename("My API"), "my-api.cue");
    assert_eq!(generate_filename("Test Project"), "test-project.cue");
    assert_eq!(
      generate_filename("  Multiple   Spaces  "),
      "multiple-spaces.cue"
    );
  }

  #[test]
  fn test_get_template_name() {
    assert_eq!(get_template_name("api"), "API Service");
    assert_eq!(get_template_name("cli"), "CLI Tool");
    assert_eq!(get_template_name("unknown"), "Unknown");
  }
}
