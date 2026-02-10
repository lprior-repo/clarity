//! Bead Export/Import UI Components
//!
//! Provides file picker dialogs, export format selection, and import preview.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use clarity_core::export::ExportFormat;
use clarity_core::import::{ConflictResolution, ImportPreview};
use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;
use tracing::instrument;

/// Export modal component properties
#[derive(Clone, Props)]
pub struct ExportModalProps {
  /// Whether the modal is open
  pub is_open: Signal<bool>,
  /// The beads to export
  pub beads: Rc<Vec<clarity_core::db::models::Bead>>,
}

// Manual PartialEq implementation since Signal doesn't implement PartialEq
impl PartialEq for ExportModalProps {
  fn eq(&self, other: &Self) -> bool {
    self.is_open == other.is_open && std::rc::Rc::ptr_eq(&self.beads, &other.beads)
  }
}

impl Eq for ExportModalProps {}

/// Export modal component
///
/// Displays a modal dialog for exporting beads with format selection.
#[component]
pub fn ExportModal(props: ExportModalProps) -> Element {
  let mut is_open = props.is_open;
  let beads = props.beads;
  let mut selected_format = use_signal(|| ExportFormat::Json);
  let is_exporting = use_signal(|| false);
  let mut export_error = use_signal(|| Option::<String>::None);
  let export_success = use_signal(|| false);

  let handle_export = {
    let beads = beads.clone();
    let selected_format = selected_format;
    let mut is_exporting = is_exporting;
    let export_error = export_error;
    let is_open = is_open;

    move |_| {
      let beads = beads.clone();
      let format = *selected_format.read();
      let mut export_error = export_error;
      let mut export_success = export_success;
      let mut is_open = is_open;

      if beads.is_empty() {
        export_error.set(Some("No beads to export".to_string()));
        return;
      }

      is_exporting.set(true);
      export_error.set(None);
      export_success.set(false);

      // Convert Rc<Vec<Bead>> to Vec<Bead> for thread safety
      let beads_vec = (*beads).clone();

      dioxus::prelude::spawn(async move {
        match tokio::task::spawn_blocking(move || {
          clarity_core::export::export_beads(&beads_vec, format)
        })
        .await
        {
          Ok(Ok(content)) => {
            // Trigger file download
            if let Err(e) = save_file_with_dialog(&content, format).await {
              export_error.set(Some(format!("Failed to save file: {e}")));
            } else {
              export_success.set(true);
              tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
              is_exporting.set(false);
              is_open.set(false);
              export_success.set(false);
            }
          }
          Ok(Err(e)) => {
            export_error.set(Some(format!("Export failed: {e}")));
            is_exporting.set(false);
          }
          Err(_) => {
            export_error.set(Some("Export task failed".to_string()));
            is_exporting.set(false);
          }
        }
      });
    }
  };

  rsx! {
      div { class: "modal-overlay",
          div { class: "modal-content",
              div { class: "modal-header",
                  h2 { "Export Beads" }
                  button {
                      class: "modal-close",
                      onclick: move |_| is_open.set(false),
                      "×"
                  }
              }

              div { class: "modal-body",
                  p { class: "help-text",
                      "Export {beads.len()} bead(s) to a file."
                  }

                  div { class: "form-group",
                      label { "Export Format" }
                      select {
                          value: "{selected_format.read().extension()}",
                          onchange: move |evt: Event<FormData>| {
                              let format = match evt.value().as_str() {
                                  "csv" => ExportFormat::Csv,
                                  _ => ExportFormat::Json,
                              };
                              selected_format.set(format);
                          },
                          option { value: "json", "JSON (Full data)" }
                          option { value: "csv", "CSV (Spreadsheet compatible)" }
                      }
                  }

                  div { class: "format-description",
                      {match *selected_format.read() {
                          ExportFormat::Json => rsx! {
                              p { "JSON format includes all bead data with proper structure. Recommended for backups and data transfer." }
                          },
                          ExportFormat::Csv => rsx! {
                              p { "CSV format is compatible with spreadsheets like Excel and Google Sheets. Best for data analysis." }
                          },
                      }}
                  }

                  if let Some(ref error) = *export_error.read() {
                      div { class: "error-message",
                          "{error}"
                          button {
                              class: "btn btn-secondary btn-sm",
                              onclick: move |_| export_error.set(None),
                              "Dismiss"
                          }
                      }
                  }

                  if *export_success.read() {
                      div { class: "success-message",
                          "Export successful!"
                      }
                  }
              }

              div { class: "modal-footer",
                  button {
                      class: "btn btn-secondary",
                      onclick: move |_| is_open.set(false),
                      "Cancel"
                  }
                  button {
                      class: "btn btn-primary",
                      disabled: *is_exporting.read() || beads.is_empty(),
                      onclick: handle_export,
                      {
                          if *is_exporting.read() {
                              "Exporting..."
                          } else {
                              "Export"
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Import modal component properties
#[derive(Clone, Props)]
pub struct ImportModalProps {
  /// Whether the modal is open
  pub is_open: Signal<bool>,
  /// Callback when import is successful (passes the count of imported beads)
  pub on_import_success: Callback<usize>,
}

// Manual PartialEq implementation since Signal and Callback don't implement PartialEq
impl PartialEq for ImportModalProps {
  fn eq(&self, other: &Self) -> bool {
    self.is_open == other.is_open
  }
}

impl Eq for ImportModalProps {}

/// Import modal component
///
/// Displays a modal dialog for importing beads with file picker and preview.
#[component]
pub fn ImportModal(props: ImportModalProps) -> Element {
  let mut is_open = props.is_open;
  let on_import_success = props.on_import_success;

  // Import source: None (not selected), Some("file"), or Some("cli")
  let mut import_source = use_signal(|| Option::<String>::None);

  // File import state
  let mut selected_file = use_signal(|| Option::<String>::None);
  let mut file_content = use_signal(|| Option::<String>::None);
  let mut import_format = use_signal(|| Option::<ExportFormat>::None);
  let mut file_preview = use_signal(|| Option::<ImportPreview>::None);

  // Beads CLI import state
  let mut cli_preview = use_signal(|| Option::<crate::import::BeadsCliImportPreview>::None);

  let mut is_loading = use_signal(|| false);
  let mut import_error = use_signal(|| Option::<String>::None);
  let mut is_importing = use_signal(|| false);

  let handle_file_select = move |_| {
    import_error.set(None);
    file_preview.set(None);

    async move {
      match pick_file().await {
        Ok(Some((path, content))) => {
          // Detect format from extension (case-insensitive)
          let path_lower = path.to_lowercase();
          #[allow(clippy::case_sensitive_file_extension_comparisons)]
          let format = if path_lower.ends_with(".json") {
            Some(ExportFormat::Json)
          } else if path_lower.ends_with(".csv") {
            Some(ExportFormat::Csv)
          } else {
            None
          };

          selected_file.set(Some(path));
          file_content.set(Some(content.clone()));
          import_format.set(format);

          // Generate preview
          if let Some(fmt) = format {
            is_loading.set(true);
            match generate_preview(&content, fmt).await {
              Ok(p) => file_preview.set(Some(p)),
              Err(e) => import_error.set(Some(format!("Failed to parse file: {e}"))),
            }
            is_loading.set(false);
          } else {
            import_error.set(Some(
              "Unknown file format. Please select a .json or .csv file.".to_string(),
            ));
          }
        }
        Ok(None) => {
          // User cancelled
        }
        Err(e) => {
          import_error.set(Some(format!("Failed to open file: {e}")));
        }
      }
    }
  };

  let handle_cli_import = move |_| {
    is_loading.set(true);
    import_error.set(None);
    cli_preview.set(None);

    let mut cli_preview = cli_preview;
    let mut is_loading = is_loading;
    let mut import_error = import_error;

    dioxus::prelude::spawn(async move {
      // Get existing bead titles for duplicate detection
      let existing_titles_result = tokio::task::spawn_blocking(|| match crate::db::DesktopDb::new() {
        Ok(db) => match db.list_beads_sync() {
          Ok(beads) => Ok(beads.into_iter().map(|b: clarity_core::db::models::Bead| b.title).collect::<Vec<String>>()),
          Err(e) => Err(format!("Failed to load existing beads: {e}")),
        },
        Err(e) => Err(format!("Failed to connect to database: {e}")),
      })
      .await;

      let existing_titles = match existing_titles_result {
        Ok(Ok(titles)) => rpds::Vector::from_iter(titles),
        Ok(Err(e)) => {
          import_error.set(Some(e));
          is_loading.set(false);
          return;
        }
        Err(_) => {
          import_error.set(Some("Task failed".to_string()));
          is_loading.set(false);
          return;
        }
      };

      // Import from beads CLI
      match crate::import::import_from_beads_cli(
        &crate::import::BeadsCliConfig::new(),
        &existing_titles,
      ) {
        Ok(p) => {
          cli_preview.set(Some(p));
        }
        Err(e) => {
          import_error.set(Some(format!("Failed to load beads: {e}")));
        }
      }

      is_loading.set(false);
    });
  };

  let handle_import = move |_| {
    is_importing.set(true);
    import_error.set(None);

    let source = import_source.read().clone();
    let on_import_success = on_import_success;
    let mut is_open = is_open;

    async move {
      match source.as_deref() {
        Some("file") => {
          let preview_data = file_preview.read().clone();
          if let Some(p) = preview_data {
            match execute_import(p).await {
              Ok(count) => {
                tracing::info!("Imported {count} beads from file");
                on_import_success.call(count);
                is_open.set(false);
              }
              Err(e) => {
                import_error.set(Some(format!("Import failed: {e}")));
              }
            }
          }
        }
        Some("cli") => {
          let preview_data = cli_preview.read().clone();
          if let Some(p) = preview_data {
            match execute_beads_cli_import(p).await {
              Ok(count) => {
                tracing::info!("Imported {count} beads from Beads CLI");
                on_import_success.call(count);
                is_open.set(false);
              }
              Err(e) => {
                import_error.set(Some(format!("Import failed: {e}")));
              }
            }
          }
        }
        _ => {}
      }
      is_importing.set(false);
    }
  };

  let has_any_preview = file_preview.read().is_some() || cli_preview.read().is_some();

  rsx! {
      div { class: "modal-overlay",
          div { class: "modal-content import-modal",
              div { class: "modal-header",
                  h2 { "Import Beads" }
                  button {
                      class: "modal-close",
                      onclick: move |_| is_open.set(false),
                      "×"
                  }
              }

              div { class: "modal-body",
                  // Source selection screen
                  if import_source.read().is_none() {
                      div { class: "import-source-selection",
                          p { "Choose where to import beads from:" }
                          div { class: "import-options",
                              div { class: "import-option",
                                  h3 { "From File" }
                                  p { "Import beads from a JSON or CSV file." }
                                  button {
                                      class: "btn btn-primary",
                                      onclick: move |_| import_source.set(Some("file".to_string())),
                                      "Select File"
                                  }
                              }
                              div { class: "import-option",
                                  h3 { "From Beads CLI" }
                                  p { "Import beads from .beads/issues.jsonl (beads_rust CLI format)." }
                                  button {
                                      class: "btn btn-primary",
                                      onclick: handle_cli_import,
                                      "Load from Beads CLI"
                                  }
                              }
                          }
                      }
                  }
                  // File import flow
                  else if import_source.read().as_deref() == Some("file") {
                      if selected_file.read().is_none() {
                          div { class: "file-selection",
                              p { "Select a file to import beads from." }
                              button {
                                  class: "btn btn-primary",
                                  onclick: handle_file_select,
                                  "Select File"
                              }
                              button {
                                  class: "btn btn-secondary",
                                  onclick: move |_| import_source.set(None),
                                  "Back"
                              }
                          }
                      } else {
                          div { class: "import-preview",
                              div { class: "file-info",
                                  p { strong { "File: " }
                                      {selected_file.read().as_ref().map_or("", std::string::String::as_str)}
                                  }
                                  p { strong { "Format: " }
                                      {import_format.read().map_or_else(|| "unknown".to_string(), |f| f.extension().to_string())}
                                  }
                              }

                              if *is_loading.read() {
                                  div { class: "loading", "Parsing file..." }
                              }

                              if let Some(ref p) = *file_preview.read() {
                                  div { class: "preview-summary",
                                      h3 { "Preview" }
                                      div { class: "preview-stats",
                                          div { class: "stat",
                                              span { class: "stat-label", "To Add" }
                                              span { class: "stat-value", "{p.to_add.len()}" }
                                          }
                                          div { class: "stat",
                                              span { class: "stat-label", "To Skip" }
                                              span { class: "stat-value", "{p.to_skip.len()}" }
                                          }
                                      }

                                      if p.has_errors() {
                                          details { class: "errors-details",
                                              summary { "Errors ({p.errors.len()})" }
                                              ul { class: "error-list",
                                                  for error in p.errors.iter() {
                                                      li { "{error}" }
                                                  }
                                              }
                                          }
                                      }
                                  }

                                  button {
                                      class: "btn btn-secondary btn-sm",
                                      onclick: move |_| selected_file.set(None),
                                      "Choose Different File"
                                  }
                              }
                          }
                      }

                      if let Some(ref error) = *import_error.read() {
                          div { class: "error-message",
                              "{error}"
                              button {
                                  class: "btn btn-secondary btn-sm",
                                  onclick: move |_| import_error.set(None),
                                  "Dismiss"
                              }
                          }
                      }
                  }
                  // Beads CLI import flow
                  else if import_source.read().as_deref() == Some("cli") {
                      if *is_loading.read() {
                          div { class: "loading",
                              p { "Loading beads from .beads/issues.jsonl..." }
                          }
                      } else if let Some(ref error) = *import_error.read() {
                          div { class: "error-message",
                              "{error}"
                              button {
                                  class: "btn btn-secondary btn-sm",
                                  onclick: move |_| import_error.set(None),
                                  "Dismiss"
                              }
                              button {
                                  class: "btn btn-secondary btn-sm",
                                  onclick: move |_| import_source.set(None),
                                  "Back"
                              }
                          }
                      } else if let Some(ref p) = *cli_preview.read() {
                          div { class: "import-preview",
                              div { class: "preview-summary",
                                  h3 { "Preview" }
                                  div { class: "preview-stats",
                                      div { class: "stat",
                                          span { class: "stat-label", "To Add" }
                                          span { class: "stat-value", "{p.to_add.len()}" }
                                      }
                                      div { class: "stat",
                                          span { class: "stat-label", "To Skip (Duplicates)" }
                                          span { class: "stat-value", "{p.to_skip.len()}" }
                                      }
                                  }

                                  if p.has_errors() {
                                      details { class: "errors-details",
                                          summary { "Errors ({p.errors.len()})" }
                                          ul { class: "error-list",
                                              for error in p.errors.iter() {
                                                  li { "{error}" }
                                              }
                                          }
                                      }
                                  }

                                  if p.to_add.is_empty() && p.to_skip.is_empty() {
                                      p { class: "info-message",
                                          "No new beads to import. All beads from the Beads CLI already exist."
                                      }
                                  }
                              }

                              button {
                                  class: "btn btn-secondary btn-sm",
                                  onclick: move |_| import_source.set(None),
                                  "Back"
                              }
                          }
                      }
                  }
              }

              div { class: "modal-footer",
                  button {
                      class: "btn btn-secondary",
                      onclick: move |_| is_open.set(false),
                      "Cancel"
                  }
                  if has_any_preview {
                      button {
                          class: "btn btn-primary",
                          disabled: *is_importing.read(),
                          onclick: handle_import,
                          {
                              if *is_importing.read() {
                                  "Importing..."
                              } else {
                                  "Import"
                              }
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Export button component properties
#[derive(Clone, Props)]
pub struct ExportButtonProps {
  /// The beads to export
  pub beads: Rc<Vec<clarity_core::db::models::Bead>>,
}

// Manual PartialEq implementation since Rc doesn't implement PartialEq for Vec
impl PartialEq for ExportButtonProps {
  fn eq(&self, other: &Self) -> bool {
    std::rc::Rc::ptr_eq(&self.beads, &other.beads)
  }
}

impl Eq for ExportButtonProps {}

/// Export button component
///
/// A button that opens the export modal
#[component]
pub fn ExportButton(props: ExportButtonProps) -> Element {
  let mut is_open = use_signal(|| false);
  let beads = props.beads;

  rsx! {
      button {
          class: "btn btn-secondary",
          onclick: move |_| is_open.set(true),
          "Export Beads"
      }

      if *is_open.read() {
          ExportModal {
              is_open,
              beads,
          }
      }
  }
}

/// Import button component properties
#[derive(Clone, Props)]
pub struct ImportButtonProps {
  /// Callback when import is successful (passes the count of imported beads)
  pub on_import_success: Callback<usize>,
}

// Manual PartialEq implementation since Callback doesn't implement PartialEq
impl PartialEq for ImportButtonProps {
  fn eq(&self, _other: &Self) -> bool {
    true // Always consider equal since we can't compare Callbacks
  }
}

impl Eq for ImportButtonProps {}

/// Import button component
///
/// A button that opens the import modal
#[component]
pub fn ImportButton(props: ImportButtonProps) -> Element {
  let mut is_open = use_signal(|| false);
  let on_import_success = props.on_import_success;

  rsx! {
      button {
          class: "btn btn-secondary",
          onclick: move |_| is_open.set(true),
          "Import Beads"
      }

      if *is_open.read() {
          ImportModal {
              is_open,
              on_import_success,
          }
      }
  }
}

/// Open a file picker dialog
///
/// # Errors
/// Returns error if file dialog fails
async fn pick_file() -> Result<Option<(String, String)>, String> {
  // Use dioxus-desktop's file dialog API
  let dialog = rfd::AsyncFileDialog::new()
    .add_filter("JSON & CSV", &["json", "csv"])
    .add_filter("JSON", &["json"])
    .add_filter("CSV", &["csv"])
    .set_title("Import Beads");

  let file_handle = dialog.pick_file().await;

  match file_handle {
    Some(handle) => {
      let path = handle.path().to_string_lossy().to_string();
      let content =
        std::fs::read_to_string(handle.path()).map_err(|e| format!("Failed to read file: {e}"))?;
      Ok(Some((path, content)))
    }
    None => Ok(None),
  }
}

/// Save file with a save dialog
///
/// # Errors
/// Returns error if file dialog or write fails
async fn save_file_with_dialog(content: &str, format: ExportFormat) -> Result<(), String> {
  let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
  let default_name = format!("beads_{timestamp}.{}", format.extension());

  let dialog = rfd::AsyncFileDialog::new()
    .set_title("Save Exported Beads")
    .set_file_name(&default_name)
    .save_file();

  let file_handle = dialog.await.ok_or("Save dialog cancelled")?;

  std::fs::write(file_handle.path(), content).map_err(|e| format!("Failed to write file: {e}"))?;

  Ok(())
}

/// Generate import preview
///
/// # Errors
/// Returns error if parsing fails
async fn generate_preview(content: &str, format: ExportFormat) -> Result<ImportPreview, String> {
  // Get existing bead IDs from database
  let existing_ids = tokio::task::spawn_blocking(|| match crate::db::DesktopDb::new() {
    Ok(db) => match db.list_beads_sync() {
      Ok(beads) => beads
        .into_iter()
        .map(|b| b.id.as_str())
        .collect::<HashSet<_>>(),
      Err(_) => HashSet::new(),
    },
    Err(_) => HashSet::new(),
  })
  .await
  .map_err(|e| format!("Failed to load existing beads: {e}"))?;

  let preview =
    clarity_core::import::preview_import(content, format, &existing_ids, ConflictResolution::Skip)
      .map_err(|e| e.to_string())?;

  Ok(preview)
}

/// Execute import with given resolution strategy
///
/// # Errors
/// Returns error if import fails
#[instrument(skip(preview), fields(to_add = preview.to_add.len(), to_replace = preview.to_replace.len(), to_merge = preview.to_merge.len()))]
async fn execute_import(preview: ImportPreview) -> Result<usize, String> {
  use rpds::Vector;
  use tracing::{error, info};

  info!("Starting file import execution");

  // Collect all beads to import based on resolution
  let to_import: Vector<_> = preview
    .to_add
    .iter()
    .chain(preview.to_replace.iter())
    .chain(preview.to_merge.iter())
    .cloned()
    .collect();

  if to_import.is_empty() {
    info!("No beads to import");
    return Ok(0);
  }

  info!(total_to_import = to_import.len(), "Converting beads for import");

  // Convert to NewBeads
  let new_beads =
    clarity_core::import::imported_to_new_beads(&to_import).map_err(|e| {
      error!(error = %e, "Failed to convert imported beads");
      e.to_string()
    })?;

  // Convert to Vec to avoid Rc issues with spawn_blocking
  let beads_vec: Vec<_> = new_beads.iter().cloned().collect();

  // Import to database
  tokio::task::spawn_blocking(move || {
    let db = crate::db::DesktopDb::new().map_err(|e| format!("Database error: {e}"))?;

    let mut imported = 0;
    for new_bead in &beads_vec {
      // Clone the bead to avoid move issues
      let bead_for_db = clarity_core::db::models::NewBead {
        title: new_bead.title.clone(),
        description: new_bead.description.clone(),
        status: new_bead.status,
        priority: new_bead.priority,
        bead_type: new_bead.bead_type,
        created_by: new_bead.created_by,
      };

      match db.create_bead_sync(bead_for_db) {
        Ok(_) => imported += 1,
        Err(e) => {
          error!(error = %e, title = %new_bead.title, "Failed to import bead");
        }
      }
    }

    info!(imported, total_attempted = beads_vec.len(), "File import execution complete");

    Ok(imported)
  })
  .await
  .map_err(|e| {
    error!(error = %e, "Spawn blocking task failed");
    format!("Task failed: {e}")
  })?
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_export_format_selection() {
    let format = ExportFormat::Json;
    assert_eq!(format.extension(), "json");
  }

  #[test]
  fn test_conflict_resolution_display() {
    assert_eq!(format!("{:?}", ConflictResolution::Skip), "Skip");
    assert_eq!(format!("{:?}", ConflictResolution::Replace), "Replace");
    assert_eq!(format!("{:?}", ConflictResolution::Merge), "Merge");
  }
}

/// Execute import from Beads CLI
///
/// # Errors
/// Returns error if import fails
#[instrument(skip(preview), fields(to_add = preview.to_add.len(), to_skip = preview.to_skip.len()))]
async fn execute_beads_cli_import(
  preview: crate::import::BeadsCliImportPreview,
) -> Result<usize, String> {
  use tracing::{error, info};

  info!("Starting Beads CLI import execution");

  // Convert to Vec to avoid Rc issues with spawn_blocking
  let beads_vec: Vec<_> = preview.to_add.iter().cloned().collect();

  if beads_vec.is_empty() {
    info!("No beads to import from Beads CLI");
    return Ok(0);
  }

  info!(total_to_import = beads_vec.len(), "Importing beads from Beads CLI");

  // Import to database
  tokio::task::spawn_blocking(move || {
    let db = crate::db::DesktopDb::new().map_err(|e| format!("Database error: {e}"))?;

    let mut imported = 0;
    for new_bead in &beads_vec {
      // Clone the bead to avoid move issues
      let bead_for_db = clarity_core::db::models::NewBead {
        title: new_bead.title.clone(),
        description: new_bead.description.clone(),
        status: new_bead.status,
        priority: new_bead.priority,
        bead_type: new_bead.bead_type,
        created_by: new_bead.created_by,
      };

      match db.create_bead_sync(bead_for_db) {
        Ok(_) => imported += 1,
        Err(e) => {
          error!(error = %e, title = %new_bead.title, "Failed to import bead from Beads CLI");
        }
      }
    }

    info!(imported, total_attempted = beads_vec.len(), "Beads CLI import execution complete");

    Ok(imported)
  })
  .await
  .map_err(|e| {
    error!(error = %e, "Spawn blocking task failed");
    format!("Task failed: {e}")
  })?
}
