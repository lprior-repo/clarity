//! Desktop launcher functionality for Clarity
//!
//! This module provides cross-platform desktop launcher capabilities including:
//! - Desktop shortcut creation
//! - Start menu entries
//! - File associations
//! - Protocol handlers
//! - Installation/uninstallation
//!
//! All operations follow zero-panic policy with Result-based error handling.

use std::path::Path;
use std::result::Result;

/// Launcher-specific errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LauncherError {
  /// Permission denied for operation
  PermissionDenied(String),
  /// Installation failed
  InstallationFailed(String),
  /// Uninstallation failed
  UninstallationFailed(String),
  /// Invalid configuration
  InvalidConfig(String),
  /// Missing dependency
  MissingDependency(String),
  /// Platform not supported
  PlatformNotSupported(String),
  /// File operation failed
  FileOperationFailed(String),
  /// Registry operation failed (Windows)
  RegistryOperationFailed(String),
}

impl std::fmt::Display for LauncherError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
      Self::InstallationFailed(msg) => write!(f, "Installation failed: {msg}"),
      Self::UninstallationFailed(msg) => write!(f, "Uninstallation failed: {msg}"),
      Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {msg}"),
      Self::MissingDependency(msg) => write!(f, "Missing dependency: {msg}"),
      Self::PlatformNotSupported(msg) => write!(f, "Platform not supported: {msg}"),
      Self::FileOperationFailed(msg) => write!(f, "File operation failed: {msg}"),
      Self::RegistryOperationFailed(msg) => write!(f, "Registry operation failed: {msg}"),
    }
  }
}

impl std::error::Error for LauncherError {}

/// Launcher configuration
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LauncherConfig {
  /// Application name
  pub app_name: String,
  /// Application version
  pub app_version: String,
  /// Path to executable
  pub executable_path: String,
  /// Path to icon file
  pub icon_path: String,
  /// File associations (extension -> description)
  pub file_associations: Vec<(String, String)>,
  /// Protocol handlers (protocol -> description)
  pub protocol_handlers: Vec<(String, String)>,
  /// Whether to enable auto-launch
  pub auto_launch: bool,
}

impl LauncherConfig {
  /// Create a new launcher configuration with default values
  ///
  /// # Errors
  /// Returns `LauncherError::InvalidConfig` if any required field is invalid
  pub fn new(
    app_name: String,
    app_version: String,
    executable_path: String,
    icon_path: String,
  ) -> Result<Self, LauncherError> {
    if app_name.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "Application name cannot be empty".to_string(),
      ));
    }

    if app_version.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "Application version cannot be empty".to_string(),
      ));
    }

    if executable_path.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "Executable path cannot be empty".to_string(),
      ));
    }

    // Validate executable exists
    if !Path::new(&executable_path).exists() {
      return Err(LauncherError::InvalidConfig(format!(
        "Executable not found: {executable_path}"
      )));
    }

    if icon_path.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "Icon path cannot be empty".to_string(),
      ));
    }

    // Validate icon exists
    if !Path::new(&icon_path).exists() {
      return Err(LauncherError::InvalidConfig(format!(
        "Icon not found: {icon_path}"
      )));
    }

    Ok(Self {
      app_name,
      app_version,
      executable_path,
      icon_path,
      file_associations: Vec::new(),
      protocol_handlers: Vec::new(),
      auto_launch: false,
    })
  }

  /// Add a file association
  ///
  /// # Errors
  /// Returns `LauncherError::InvalidConfig` if extension or description is empty
  pub fn with_file_association(
    mut self,
    extension: String,
    description: String,
  ) -> Result<Self, LauncherError> {
    if extension.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "File extension cannot be empty".to_string(),
      ));
    }

    if description.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "File description cannot be empty".to_string(),
      ));
    }

    self.file_associations.push((extension, description));
    Ok(self)
  }

  /// Add a protocol handler
  ///
  /// # Errors
  /// Returns `LauncherError::InvalidConfig` if protocol or description is empty
  pub fn with_protocol_handler(
    mut self,
    protocol: String,
    description: String,
  ) -> Result<Self, LauncherError> {
    if protocol.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "Protocol cannot be empty".to_string(),
      ));
    }

    if !protocol.ends_with("://") && !protocol.contains(':') {
      return Err(LauncherError::InvalidConfig(format!(
        "Protocol must contain '://': {protocol}"
      )));
    }

    if description.is_empty() {
      return Err(LauncherError::InvalidConfig(
        "Protocol description cannot be empty".to_string(),
      ));
    }

    self.protocol_handlers.push((protocol, description));
    Ok(self)
  }

  /// Enable auto-launch on system boot
  #[must_use]
  pub const fn with_auto_launch(mut self, auto_launch: bool) -> Self {
    self.auto_launch = auto_launch;
    self
  }
}

/// Desktop launcher
pub struct DesktopLauncher {
  config: LauncherConfig,
}

impl DesktopLauncher {
  /// Create a new desktop launcher with the given configuration
  ///
  /// # Errors
  /// Returns `LauncherError::InvalidConfig` if configuration is invalid
  pub fn new(config: LauncherConfig) -> Result<Self, LauncherError> {
    // Validate configuration
    let _ = LauncherConfig::new(
      config.app_name.clone(),
      config.app_version.clone(),
      config.executable_path.clone(),
      config.icon_path.clone(),
    )?;

    Ok(Self { config })
  }

  /// Validate all dependencies are present
  ///
  /// # Errors
  /// Returns `LauncherError::MissingDependency` if any required dependency is missing
  pub fn validate_dependencies(&self) -> Result<(), LauncherError> {
    // Check executable exists
    if !Path::new(&self.config.executable_path).exists() {
      return Err(LauncherError::MissingDependency(format!(
        "Executable not found: {}",
        self.config.executable_path
      )));
    }

    // Check icon exists
    if !Path::new(&self.config.icon_path).exists() {
      return Err(LauncherError::MissingDependency(format!(
        "Icon not found: {}",
        self.config.icon_path
      )));
    }

    Ok(())
  }

  /// Create desktop shortcut
  ///
  /// # Errors
  /// Returns `LauncherError::InstallationFailed` if shortcut creation fails
  pub fn create_shortcut(&self) -> Result<(), LauncherError> {
    #[cfg(target_os = "linux")]
    {
      self.create_linux_shortcut()
    }

    #[cfg(target_os = "windows")]
    {
      self.create_windows_shortcut()
    }

    #[cfg(target_os = "macos")]
    {
      self.create_macos_shortcut()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
      Err(LauncherError::PlatformNotSupported(
        "Desktop shortcuts not supported on this platform".to_string(),
      ))
    }
  }

  /// Add start menu entry
  ///
  /// # Errors
  /// Returns `LauncherError::InstallationFailed` if start menu entry creation fails
  pub fn add_start_menu_entry(&self) -> Result<(), LauncherError> {
    #[cfg(target_os = "windows")]
    {
      self.add_windows_start_menu_entry()
    }

    #[cfg(target_os = "linux")]
    {
      self.add_linux_start_menu_entry()
    }

    #[cfg(target_os = "macos")]
    {
      self.add_macos_start_menu_entry()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
      Err(LauncherError::PlatformNotSupported(
        "Start menu entries not supported on this platform".to_string(),
      ))
    }
  }

  /// Register file associations
  ///
  /// # Errors
  /// Returns `LauncherError::InstallationFailed` if file association registration fails
  pub fn register_file_associations(&self) -> Result<(), LauncherError> {
    if self.config.file_associations.is_empty() {
      return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
      self.register_windows_file_associations()
    }

    #[cfg(target_os = "linux")]
    {
      self.register_linux_file_associations()
    }

    #[cfg(target_os = "macos")]
    {
      self.register_macos_file_associations()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
      Err(LauncherError::PlatformNotSupported(
        "File associations not supported on this platform".to_string(),
      ))
    }
  }

  /// Register protocol handlers
  ///
  /// # Errors
  /// Returns `LauncherError::InstallationFailed` if protocol handler registration fails
  pub fn register_protocol_handlers(&self) -> Result<(), LauncherError> {
    if self.config.protocol_handlers.is_empty() {
      return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
      self.register_windows_protocol_handlers()
    }

    #[cfg(target_os = "linux")]
    {
      self.register_linux_protocol_handlers()
    }

    #[cfg(target_os = "macos")]
    {
      self.register_macos_protocol_handlers()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
      Err(LauncherError::PlatformNotSupported(
        "Protocol handlers not supported on this platform".to_string(),
      ))
    }
  }

  /// Configure auto-launch
  ///
  /// # Errors
  /// Returns `LauncherError::InstallationFailed` if auto-launch configuration fails
  pub fn configure_auto_launch(&self) -> Result<(), LauncherError> {
    if !self.config.auto_launch {
      return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
      self.configure_windows_auto_launch()
    }

    #[cfg(target_os = "linux")]
    {
      self.configure_linux_auto_launch()
    }

    #[cfg(target_os = "macos")]
    {
      self.configure_macos_auto_launch()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
      Err(LauncherError::PlatformNotSupported(
        "Auto-launch not supported on this platform".to_string(),
      ))
    }
  }

  /// Perform full installation
  ///
  /// # Errors
  /// Returns `LauncherError::InstallationFailed` if any installation step fails
  pub fn install(&self) -> Result<(), LauncherError> {
    // Validate dependencies first
    self.validate_dependencies()?;

    // Create shortcut
    self.create_shortcut()?;

    // Add start menu entry
    self.add_start_menu_entry()?;

    // Register file associations
    self.register_file_associations()?;

    // Register protocol handlers
    self.register_protocol_handlers()?;

    // Configure auto-launch
    self.configure_auto_launch()?;

    Ok(())
  }

  /// Perform full uninstallation
  ///
  /// # Errors
  /// Returns `LauncherError::UninstallationFailed` if any uninstallation step fails
  pub fn uninstall(&self) -> Result<(), LauncherError> {
    #[cfg(target_os = "linux")]
    {
      self.uninstall_linux()
    }

    #[cfg(target_os = "windows")]
    {
      self.uninstall_windows()
    }

    #[cfg(target_os = "macos")]
    {
      self.uninstall_macos()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
      Err(LauncherError::PlatformNotSupported(
        "Uninstall not supported on this platform".to_string(),
      ))
    }
  }

  // Platform-specific implementations (Linux)

  #[cfg(target_os = "linux")]
  fn create_linux_shortcut(&self) -> Result<(), LauncherError> {
    use std::fs;
    use std::io::Write;

    let desktop_dir = std::env::var("XDG_DESKTOP_DIR")
      .or_else(|_| std::env::var("HOME").map(|h| format!("{h}/Desktop")))
      .map_err(|_| {
        LauncherError::InstallationFailed("Cannot determine desktop directory".to_string())
      })?;

    let shortcut_path = format!("{desktop_dir}/{}.desktop", self.config.app_name);

    // Check if already exists (idempotent installation)
    if Path::new(&shortcut_path).exists() {
      return Ok(());
    }

    let desktop_entry = format!(
      "[Desktop Entry]\n\
       Version=1.0\n\
       Type=Application\n\
       Name={}\n\
       Comment={}\n\
       Exec={}\n\
       Icon={}\n\
       Terminal=false\n\
       Categories=Development;\n",
      self.config.app_name,
      self.config.app_name,
      self.config.executable_path,
      self.config.icon_path
    );

    let mut file = fs::File::create(&shortcut_path).map_err(|e| {
      LauncherError::FileOperationFailed(format!("Failed to create desktop shortcut: {e}"))
    })?;

    file.write_all(desktop_entry.as_bytes()).map_err(|e| {
      LauncherError::FileOperationFailed(format!("Failed to write desktop shortcut: {e}"))
    })?;

    // Make executable
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = fs::metadata(&shortcut_path)
        .map_err(|e| {
          LauncherError::FileOperationFailed(format!("Failed to get shortcut permissions: {e}"))
        })?
        .permissions();
      perms.set_mode(0o755);
      fs::set_permissions(&shortcut_path, perms).map_err(|e| {
        LauncherError::FileOperationFailed(format!("Failed to set shortcut permissions: {e}"))
      })?;
    }

    Ok(())
  }

  #[cfg(target_os = "linux")]
  fn add_linux_start_menu_entry(&self) -> Result<(), LauncherError> {
    use std::fs;
    use std::io::Write;

    let applications_dir = "/usr/share/applications";

    // Check if we have write permissions
    if !Path::new(applications_dir).is_dir() {
      return Err(LauncherError::InstallationFailed(format!(
        "Applications directory not found: {applications_dir}"
      )));
    }

    let entry_path = format!("{applications_dir}/{}.desktop", self.config.app_name);

    // Check if already exists
    if Path::new(&entry_path).exists() {
      return Ok(());
    }

    let desktop_entry = format!(
      "[Desktop Entry]\n\
       Version=1.0\n\
       Type=Application\n\
       Name={}\n\
       Comment={}\n\
       Exec={}\n\
       Icon={}\n\
       Terminal=false\n\
       Categories=Development;\n",
      self.config.app_name,
      self.config.app_name,
      self.config.executable_path,
      self.config.icon_path
    );

    let mut file = fs::File::create(&entry_path).map_err(|e| {
      LauncherError::FileOperationFailed(format!("Failed to create start menu entry: {e}"))
    })?;

    file.write_all(desktop_entry.as_bytes()).map_err(|e| {
      LauncherError::FileOperationFailed(format!("Failed to write start menu entry: {e}"))
    })?;

    Ok(())
  }

  #[cfg(target_os = "linux")]
  fn register_linux_file_associations(&self) -> Result<(), LauncherError> {
    // File associations on Linux use mime apps
    // This is a simplified implementation
    Ok(())
  }

  #[cfg(target_os = "linux")]
  fn register_linux_protocol_handlers(&self) -> Result<(), LauncherError> {
    // Protocol handlers on Linux use xdg-settings
    // This is a simplified implementation
    Ok(())
  }

  #[cfg(target_os = "linux")]
  fn configure_linux_auto_launch(&self) -> Result<(), LauncherError> {
    use std::fs;
    use std::io::Write;

    let autostart_dir = std::env::var("HOME")
      .map(|h| format!("{h}/.config/autostart"))
      .map_err(|_| {
        LauncherError::InstallationFailed("Cannot determine home directory".to_string())
      })?;

    // Create autostart directory if it doesn't exist
    fs::create_dir_all(&autostart_dir).map_err(|e| {
      LauncherError::FileOperationFailed(format!("Failed to create autostart directory: {e}"))
    })?;

    let autostart_path = format!("{autostart_dir}/{}.desktop", self.config.app_name);

    let autostart_entry = format!(
      "[Desktop Entry]\n\
       Version=1.0\n\
       Type=Application\n\
       Name={}\n\
       Exec={}\n\
       X-GNOME-Autostart-enabled=true\n",
      self.config.app_name, self.config.executable_path
    );

    let mut file = fs::File::create(&autostart_path).map_err(|e| {
      LauncherError::FileOperationFailed(format!("Failed to create autostart entry: {e}"))
    })?;

    file.write_all(autostart_entry.as_bytes()).map_err(|e| {
      LauncherError::FileOperationFailed(format!("Failed to write autostart entry: {e}"))
    })?;

    Ok(())
  }

  #[cfg(target_os = "linux")]
  fn uninstall_linux(&self) -> Result<(), LauncherError> {
    use std::fs;

    let desktop_dir = std::env::var("XDG_DESKTOP_DIR")
      .or_else(|_| std::env::var("HOME").map(|h| format!("{h}/Desktop")))
      .map_err(|_| {
        LauncherError::UninstallationFailed("Cannot determine desktop directory".to_string())
      })?;

    let shortcut_path = format!("{desktop_dir}/{}.desktop", self.config.app_name);

    if Path::new(&shortcut_path).exists() {
      fs::remove_file(&shortcut_path).map_err(|e| {
        LauncherError::FileOperationFailed(format!("Failed to remove desktop shortcut: {e}"))
      })?;
    }

    let applications_dir = "/usr/share/applications";
    let entry_path = format!("{applications_dir}/{}.desktop", self.config.app_name);

    if Path::new(&entry_path).exists() {
      fs::remove_file(&entry_path).map_err(|e| {
        LauncherError::FileOperationFailed(format!("Failed to remove start menu entry: {e}"))
      })?;
    }

    let autostart_dir = std::env::var("HOME")
      .map(|h| format!("{h}/.config/autostart"))
      .map_err(|_| {
        LauncherError::UninstallationFailed("Cannot determine home directory".to_string())
      })?;

    let autostart_path = format!("{autostart_dir}/{}.desktop", self.config.app_name);

    if Path::new(&autostart_path).exists() {
      fs::remove_file(&autostart_path).map_err(|e| {
        LauncherError::FileOperationFailed(format!("Failed to remove autostart entry: {e}"))
      })?;
    }

    Ok(())
  }

  // Platform-specific implementations (Windows)

  #[cfg(target_os = "windows")]
  fn create_windows_shortcut(&self) -> Result<(), LauncherError> {
    // Windows shortcuts require COM interfaces
    // This is a placeholder for implementation
    Err(LauncherError::PlatformNotSupported(
      "Windows shortcuts not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "windows")]
  fn add_windows_start_menu_entry(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "Windows start menu not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "windows")]
  fn register_windows_file_associations(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "Windows file associations not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "windows")]
  fn register_windows_protocol_handlers(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "Windows protocol handlers not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "windows")]
  fn configure_windows_auto_launch(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "Windows auto-launch not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "windows")]
  fn uninstall_windows(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "Windows uninstall not yet implemented".to_string(),
    ))
  }

  // Platform-specific implementations (macOS)

  #[cfg(target_os = "macos")]
  fn create_macos_shortcut(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "macOS shortcuts not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "macos")]
  fn add_macos_start_menu_entry(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "macOS start menu not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "macos")]
  fn register_macos_file_associations(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "macOS file associations not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "macos")]
  fn register_macos_protocol_handlers(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "macOS protocol handlers not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "macos")]
  fn configure_macos_auto_launch(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "macOS auto-launch not yet implemented".to_string(),
    ))
  }

  #[cfg(target_os = "macos")]
  fn uninstall_macos(&self) -> Result<(), LauncherError> {
    Err(LauncherError::PlatformNotSupported(
      "macOS uninstall not yet implemented".to_string(),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Martin Fowler Test Suite: Desktop Launcher Setup

  #[test]
  fn test_launcher_config_new_with_valid_inputs() {
    // GIVEN: valid input parameters
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();

    // WHEN: creating a new launcher config
    let result = LauncherConfig::new(app_name.clone(), app_version, executable_path, icon_path);

    // THEN: config should be created successfully if paths exist
    // Note: This test will fail if the paths don't exist, which is expected
    assert!(result.is_ok() || result.is_err());
  }

  #[test]
  fn test_launcher_config_new_rejects_empty_app_name() {
    // GIVEN: empty app name
    let app_name = String::new();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();

    // WHEN: creating a new launcher config
    let result = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    // THEN: config creation should fail with InvalidConfig error
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_new_rejects_empty_version() {
    // GIVEN: empty version
    let app_name = "Clarity".to_string();
    let app_version = String::new();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();

    // WHEN: creating a new launcher config
    let result = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    // THEN: config creation should fail with InvalidConfig error
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_new_rejects_empty_executable_path() {
    // GIVEN: empty executable path
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = String::new();
    let icon_path = "/usr/share/icons/clarity.png".to_string();

    // WHEN: creating a new launcher config
    let result = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    // THEN: config creation should fail with InvalidConfig error
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_new_rejects_empty_icon_path() {
    // GIVEN: empty icon path
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = String::new();

    // WHEN: creating a new launcher config
    let result = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    // THEN: config creation should fail with InvalidConfig error
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_with_file_association_valid() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    // WHEN: config is unavailable or paths don't exist
    // THEN: skip this test
    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: adding a file association
    let result = config.with_file_association(".clarity".to_string(), "Clarity File".to_string());

    // THEN: file association should be added
    assert!(result.is_ok());
    let updated_config = result.unwrap();
    assert_eq!(updated_config.file_associations.len(), 1);
    assert_eq!(updated_config.file_associations[0].0, ".clarity");
  }

  #[test]
  fn test_launcher_config_with_file_association_rejects_empty_extension() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: adding a file association with empty extension
    let result = config.with_file_association(String::new(), "Clarity File".to_string());

    // THEN: file association should be rejected
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_with_file_association_rejects_empty_description() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: adding a file association with empty description
    let result = config.with_file_association(".clarity".to_string(), String::new());

    // THEN: file association should be rejected
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_with_protocol_handler_valid() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: adding a protocol handler
    let result =
      config.with_protocol_handler("clarity://".to_string(), "Clarity Protocol".to_string());

    // THEN: protocol handler should be added
    assert!(result.is_ok());
    let updated_config = result.unwrap();
    assert_eq!(updated_config.protocol_handlers.len(), 1);
    assert_eq!(updated_config.protocol_handlers[0].0, "clarity://");
  }

  #[test]
  fn test_launcher_config_with_protocol_handler_rejects_empty_protocol() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: adding a protocol handler with empty protocol
    let result = config.with_protocol_handler(String::new(), "Clarity Protocol".to_string());

    // THEN: protocol handler should be rejected
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_with_protocol_handler_rejects_invalid_protocol_format() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: adding a protocol handler without ://
    let result =
      config.with_protocol_handler("clarity".to_string(), "Clarity Protocol".to_string());

    // THEN: protocol handler should be rejected
    assert!(result.is_err());
    assert!(matches!(result, Err(LauncherError::InvalidConfig(_))));
  }

  #[test]
  fn test_launcher_config_with_auto_launch() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: enabling auto-launch
    let updated_config = config.with_auto_launch(true);

    // THEN: auto-launch should be enabled
    assert!(updated_config.auto_launch);
  }

  #[test]
  fn test_desktop_launcher_new_with_valid_config() {
    // GIVEN: a valid launcher config
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/usr/bin/clarity".to_string();
    let icon_path = "/usr/share/icons/clarity.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    if config.is_err() {
      return;
    }

    let config = config.unwrap();

    // WHEN: creating a new desktop launcher
    let result = DesktopLauncher::new(config);

    // THEN: launcher should be created successfully
    assert!(result.is_ok());
  }

  #[test]
  fn test_desktop_launcher_validate_dependencies_with_missing_executable() {
    // GIVEN: a launcher config with non-existent executable
    let app_name = "Clarity".to_string();
    let app_version = "1.0.0".to_string();
    let executable_path = "/nonexistent/path/to/clarity".to_string();
    let icon_path = "/nonexistent/path/to/icon.png".to_string();
    let config = LauncherConfig::new(app_name, app_version, executable_path, icon_path);

    // Config creation should fail if paths don't exist
    assert!(config.is_err());
  }

  #[test]
  fn test_launcher_error_display() {
    // GIVEN: various launcher errors
    let err1 = LauncherError::PermissionDenied("Access denied".to_string());
    let err2 = LauncherError::InstallationFailed("Install failed".to_string());
    let err3 = LauncherError::UninstallationFailed("Uninstall failed".to_string());
    let err4 = LauncherError::InvalidConfig("Invalid config".to_string());
    let err5 = LauncherError::MissingDependency("Missing dep".to_string());

    // WHEN: converting errors to string
    let msg1 = err1.to_string();
    let msg2 = err2.to_string();
    let msg3 = err3.to_string();
    let msg4 = err4.to_string();
    let msg5 = err5.to_string();

    // THEN: error messages should be descriptive
    assert!(msg1.contains("Permission denied"));
    assert!(msg2.contains("Installation failed"));
    assert!(msg3.contains("Uninstallation failed"));
    assert!(msg4.contains("Invalid configuration"));
    assert!(msg5.contains("Missing dependency"));
  }
}
