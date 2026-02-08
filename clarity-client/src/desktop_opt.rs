//! Desktop-specific optimizations for Dioxus
//!
//! This module provides performance optimizations and configuration for desktop applications,
//! including render optimization, memory management, and platform-specific tuning.

use std::result::Result;

/// Desktop optimization errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesktopOptError {
  /// Invalid configuration
  InvalidConfig(String),
  /// Optimization failed
  OptimizationFailed(String),
  /// Platform not supported
  PlatformNotSupported(String),
}

impl std::fmt::Display for DesktopOptError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {msg}"),
      Self::OptimizationFailed(msg) => write!(f, "Optimization failed: {msg}"),
      Self::PlatformNotSupported(msg) => write!(f, "Platform not supported: {msg}"),
    }
  }
}

impl std::error::Error for DesktopOptError {}

/// Render optimization strategy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStrategy {
  /// Optimize for battery life (lower performance)
  PowerSaving,
  /// Balanced performance
  Balanced,
  /// Maximum performance (higher power consumption)
  Performance,
}

impl RenderStrategy {
  /// Get frame rate cap for the strategy
  #[must_use]
  pub const fn frame_rate_cap(&self) -> u32 {
    match self {
      Self::PowerSaving => 30,
      Self::Balanced => 60,
      Self::Performance => 120,
    }
  }

  /// Get vsync setting for the strategy
  #[must_use]
  pub const fn vsync_enabled(&self) -> bool {
    match self {
      Self::PowerSaving => true,
      Self::Balanced => true,
      Self::Performance => false,
    }
  }
}

/// Desktop optimization configuration
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopConfig {
  /// Render optimization strategy
  pub render_strategy: RenderStrategy,
  /// Enable hardware acceleration
  pub hardware_acceleration: bool,
  /// Enable GPU rendering
  pub gpu_rendering: bool,
  /// Memory limit in MB (None for unlimited)
  pub memory_limit_mb: Option<usize>,
  /// Enable lazy loading of components
  pub lazy_loading: bool,
  /// Enable render caching
  pub render_caching: bool,
  /// Maximum concurrent render threads (0 for auto)
  pub max_render_threads: usize,
}

impl DesktopConfig {
  /// Create a new desktop configuration with sensible defaults
  #[must_use]
  pub const fn new() -> Self {
    Self {
      render_strategy: RenderStrategy::Balanced,
      hardware_acceleration: true,
      gpu_rendering: true,
      memory_limit_mb: Some(512),
      lazy_loading: true,
      render_caching: true,
      max_render_threads: 0, // Auto-detect
    }
  }

  /// Create a power-saving configuration
  #[must_use]
  pub const fn power_saving() -> Self {
    Self {
      render_strategy: RenderStrategy::PowerSaving,
      hardware_acceleration: true,
      gpu_rendering: false,
      memory_limit_mb: Some(256),
      lazy_loading: true,
      render_caching: true,
      max_render_threads: 1,
    }
  }

  /// Create a performance configuration
  #[must_use]
  pub const fn performance() -> Self {
    Self {
      render_strategy: RenderStrategy::Performance,
      hardware_acceleration: true,
      gpu_rendering: true,
      memory_limit_mb: None, // Unlimited
      lazy_loading: false,
      render_caching: true,
      max_render_threads: 0, // Auto-detect
    }
  }

  /// Set render strategy
  #[must_use]
  pub const fn with_render_strategy(mut self, strategy: RenderStrategy) -> Self {
    self.render_strategy = strategy;
    self
  }

  /// Enable or disable hardware acceleration
  #[must_use]
  pub const fn with_hardware_acceleration(mut self, enabled: bool) -> Self {
    self.hardware_acceleration = enabled;
    self
  }

  /// Enable or disable GPU rendering
  #[must_use]
  pub const fn with_gpu_rendering(mut self, enabled: bool) -> Self {
    self.gpu_rendering = enabled;
    self
  }

  /// Set memory limit
  #[must_use]
  pub const fn with_memory_limit_mb(mut self, limit: Option<usize>) -> Self {
    self.memory_limit_mb = limit;
    self
  }

  /// Enable or disable lazy loading
  #[must_use]
  pub const fn with_lazy_loading(mut self, enabled: bool) -> Self {
    self.lazy_loading = enabled;
    self
  }

  /// Enable or disable render caching
  #[must_use]
  pub const fn with_render_caching(mut self, enabled: bool) -> Self {
    self.render_caching = enabled;
    self
  }

  /// Set maximum render threads
  #[must_use]
  pub const fn with_max_render_threads(mut self, threads: usize) -> Self {
    self.max_render_threads = threads;
    self
  }

  /// Validate configuration
  ///
  /// # Errors
  /// Returns `DesktopOptError::InvalidConfig` if configuration is invalid
  pub fn validate(&self) -> Result<(), DesktopOptError> {
    if let Some(limit) = self.memory_limit_mb {
      if limit < 64 {
        return Err(DesktopOptError::InvalidConfig(format!(
          "Memory limit too low: {} MB (minimum: 64 MB)",
          limit
        )));
      }
      if limit > 8192 {
        return Err(DesktopOptError::InvalidConfig(format!(
          "Memory limit too high: {} MB (maximum: 8192 MB)",
          limit
        )));
      }
    }

    if self.max_render_threads > 16 {
      return Err(DesktopOptError::InvalidConfig(format!(
        "Max render threads too high: {} (maximum: 16)",
        self.max_render_threads
      )));
    }

    Ok(())
  }
}

impl Default for DesktopConfig {
  fn default() -> Self {
    Self::new()
  }
}

/// Desktop optimization manager
pub struct DesktopOptimizer {
  /// Current configuration
  config: DesktopConfig,
}

impl DesktopOptimizer {
  /// Create a new desktop optimizer with the given configuration
  ///
  /// # Errors
  /// Returns `DesktopOptError::InvalidConfig` if configuration is invalid
  pub fn new(config: DesktopConfig) -> Result<Self, DesktopOptError> {
    config.validate()?;
    Ok(Self { config })
  }

  /// Get the current configuration
  #[must_use]
  pub const fn config(&self) -> &DesktopConfig {
    &self.config
  }

  /// Update the configuration
  ///
  /// # Errors
  /// Returns `DesktopOptError::InvalidConfig` if new configuration is invalid
  pub fn update_config(&mut self, config: DesktopConfig) -> Result<(), DesktopOptError> {
    config.validate()?;
    self.config = config;
    Ok(())
  }

  /// Apply optimizations to the runtime
  ///
  /// # Errors
  /// Returns `DesktopOptError::OptimizationFailed` if optimization fails
  pub fn apply_optimizations(&self) -> Result<(), DesktopOptError> {
    // Apply render strategy
    self.apply_render_strategy()?;

    // Apply memory limits
    self.apply_memory_limits()?;

    Ok(())
  }

  /// Apply render strategy
  ///
  /// # Errors
  /// Returns `DesktopOptError::OptimizationFailed` if strategy application fails
  fn apply_render_strategy(&self) -> Result<(), DesktopOptError> {
    // In a real implementation, this would configure the Dioxus renderer
    // For now, we just validate the strategy
    let _fps = self.config.render_strategy.frame_rate_cap();
    let _vsync = self.config.render_strategy.vsync_enabled();

    Ok(())
  }

  /// Apply memory limits
  ///
  /// # Errors
  /// Returns `DesktopOptError::OptimizationFailed` if memory limit application fails
  fn apply_memory_limits(&self) -> Result<(), DesktopOptError> {
    if let Some(limit_mb) = self.config.memory_limit_mb {
      // In a real implementation, this would set allocator limits
      let _limit_bytes = limit_mb * 1024 * 1024;
    }

    Ok(())
  }

  /// Get recommended configuration for the current system
  ///
  /// # Errors
  /// Returns `DesktopOptError::PlatformNotSupported` if platform detection fails
  pub fn detect_system_config() -> Result<DesktopConfig, DesktopOptError> {
    let cpu_count = Self::get_cpu_count();

    // Detect if we're on a laptop (assume battery-powered)
    let is_laptop = Self::detect_laptop();

    let config = if is_laptop {
      // Use power-saving config for laptops
      DesktopConfig::power_saving().with_max_render_threads(cpu_count)
    } else {
      // Use balanced config for desktops
      DesktopConfig::new().with_max_render_threads(cpu_count)
    };

    // Validate the config before returning
    config.validate()?;

    Ok(config)
  }

  /// Detect if running on a laptop (platform-specific)
  #[must_use]
  fn detect_laptop() -> bool {
    // Simple heuristic: assume mobile platforms are laptops
    // In a real implementation, this would use platform APIs
    cfg!(target_os = "macos") || cfg!(target_os = "windows")
  }

  /// Get CPU count for render threads
  #[must_use]
  fn get_cpu_count() -> usize {
    num_cpus::get().max(1).min(16) // Clamp between 1 and 16
  }

  /// Get performance metrics (placeholder)
  #[must_use]
  pub fn performance_metrics(&self) -> PerformanceMetrics {
    PerformanceMetrics {
      frame_rate: self.config.render_strategy.frame_rate_cap(),
      vsync_enabled: self.config.render_strategy.vsync_enabled(),
      hardware_acceleration: self.config.hardware_acceleration,
      gpu_rendering: self.config.gpu_rendering,
      memory_limit_mb: self.config.memory_limit_mb,
    }
  }
}

/// Performance metrics
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerformanceMetrics {
  /// Current frame rate cap
  pub frame_rate: u32,
  /// Whether vsync is enabled
  pub vsync_enabled: bool,
  /// Whether hardware acceleration is enabled
  pub hardware_acceleration: bool,
  /// Whether GPU rendering is enabled
  pub gpu_rendering: bool,
  /// Memory limit in MB
  pub memory_limit_mb: Option<usize>,
}

/// Create a desktop launcher configuration optimized for the current platform
///
/// # Errors
/// Returns `DesktopOptError::PlatformNotSupported` if platform is not supported
pub fn create_optimized_launcher() -> Result<DesktopOptimizer, DesktopOptError> {
  let config = DesktopOptimizer::detect_system_config()?;
  DesktopOptimizer::new(config)
}

#[cfg(test)]
mod tests {
  use super::*;

  // Martin Fowler Test Suite: Desktop Optimization

  #[test]
  fn test_render_strategy_frame_rate_cap() {
    // GIVEN: different render strategies
    let power_saving = RenderStrategy::PowerSaving;
    let balanced = RenderStrategy::Balanced;
    let performance = RenderStrategy::Performance;

    // WHEN: getting frame rate caps
    let fps_power = power_saving.frame_rate_cap();
    let fps_balanced = balanced.frame_rate_cap();
    let fps_perf = performance.frame_rate_cap();

    // THEN: frame rates should match expectations
    assert_eq!(fps_power, 30);
    assert_eq!(fps_balanced, 60);
    assert_eq!(fps_perf, 120);
  }

  #[test]
  fn test_render_strategy_vsync() {
    // GIVEN: different render strategies
    let power_saving = RenderStrategy::PowerSaving;
    let balanced = RenderStrategy::Balanced;
    let performance = RenderStrategy::Performance;

    // WHEN: getting vsync settings
    let vsync_power = power_saving.vsync_enabled();
    let vsync_balanced = balanced.vsync_enabled();
    let vsync_perf = performance.vsync_enabled();

    // THEN: vsync should be enabled for power-saving and balanced
    assert!(vsync_power);
    assert!(vsync_balanced);
    // Performance mode disables vsync for maximum FPS
    assert!(!vsync_perf);
  }

  #[test]
  fn test_desktop_config_new() {
    // GIVEN: no parameters
    // WHEN: creating new desktop config
    let config = DesktopConfig::new();

    // THEN: should have balanced defaults
    assert_eq!(config.render_strategy, RenderStrategy::Balanced);
    assert!(config.hardware_acceleration);
    assert!(config.gpu_rendering);
    assert_eq!(config.memory_limit_mb, Some(512));
    assert!(config.lazy_loading);
    assert!(config.render_caching);
    assert_eq!(config.max_render_threads, 0);
  }

  #[test]
  fn test_desktop_config_power_saving() {
    // GIVEN: no parameters
    // WHEN: creating power-saving config
    let config = DesktopConfig::power_saving();

    // THEN: should prioritize power efficiency
    assert_eq!(config.render_strategy, RenderStrategy::PowerSaving);
    assert!(config.hardware_acceleration);
    assert!(!config.gpu_rendering); // GPU disabled to save power
    assert_eq!(config.memory_limit_mb, Some(256));
    assert!(config.lazy_loading);
    assert!(config.render_caching);
    assert_eq!(config.max_render_threads, 1); // Single thread
  }

  #[test]
  fn test_desktop_config_performance() {
    // GIVEN: no parameters
    // WHEN: creating performance config
    let config = DesktopConfig::performance();

    // THEN: should prioritize performance
    assert_eq!(config.render_strategy, RenderStrategy::Performance);
    assert!(config.hardware_acceleration);
    assert!(config.gpu_rendering);
    assert_eq!(config.memory_limit_mb, None); // No memory limit
    assert!(!config.lazy_loading); // Load everything immediately
    assert!(config.render_caching);
    assert_eq!(config.max_render_threads, 0); // Auto-detect
  }

  #[test]
  fn test_desktop_config_validate_valid() {
    // GIVEN: valid desktop config
    let config = DesktopConfig::new();

    // WHEN: validating config
    let result = config.validate();

    // THEN: validation should succeed
    assert!(result.is_ok());
  }

  #[test]
  fn test_desktop_config_validate_memory_too_low() {
    // GIVEN: config with memory limit too low
    let config = DesktopConfig::new().with_memory_limit_mb(Some(32));

    // WHEN: validating config
    let result = config.validate();

    // THEN: validation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(DesktopOptError::InvalidConfig(_))));
  }

  #[test]
  fn test_desktop_config_validate_memory_too_high() {
    // GIVEN: config with memory limit too high
    let config = DesktopConfig::new().with_memory_limit_mb(Some(10000));

    // WHEN: validating config
    let result = config.validate();

    // THEN: validation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(DesktopOptError::InvalidConfig(_))));
  }

  #[test]
  fn test_desktop_config_validate_threads_too_high() {
    // GIVEN: config with too many render threads
    let config = DesktopConfig::new().with_max_render_threads(32);

    // WHEN: validating config
    let result = config.validate();

    // THEN: validation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(DesktopOptError::InvalidConfig(_))));
  }

  #[test]
  fn test_desktop_optimizer_new_valid() {
    // GIVEN: valid config
    let config = DesktopConfig::new();

    // WHEN: creating optimizer
    let result = DesktopOptimizer::new(config);

    // THEN: optimizer should be created successfully
    assert!(result.is_ok());
    let optimizer = result.unwrap();
    assert_eq!(optimizer.config().render_strategy, RenderStrategy::Balanced);
  }

  #[test]
  fn test_desktop_optimizer_new_invalid() {
    // GIVEN: invalid config (memory limit too low)
    let config = DesktopConfig::new().with_memory_limit_mb(Some(32));

    // WHEN: creating optimizer
    let result = DesktopOptimizer::new(config);

    // THEN: optimizer creation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(DesktopOptError::InvalidConfig(_))));
  }

  #[test]
  fn test_desktop_optimizer_update_config() {
    // GIVEN: optimizer with initial config
    let config = DesktopConfig::new();
    let mut optimizer = DesktopOptimizer::new(config).unwrap();

    // WHEN: updating to power-saving config
    let new_config = DesktopConfig::power_saving();
    let result = optimizer.update_config(new_config);

    // THEN: config should be updated successfully
    assert!(result.is_ok());
    assert_eq!(
      optimizer.config().render_strategy,
      RenderStrategy::PowerSaving
    );
  }

  #[test]
  fn test_desktop_optimizer_apply_optimizations() {
    // GIVEN: optimizer with valid config
    let config = DesktopConfig::new();
    let optimizer = DesktopOptimizer::new(config).unwrap();

    // WHEN: applying optimizations
    let result = optimizer.apply_optimizations();

    // THEN: optimizations should be applied successfully
    assert!(result.is_ok());
  }

  #[test]
  fn test_desktop_optimizer_performance_metrics() {
    // GIVEN: optimizer with balanced config
    let config = DesktopConfig::new();
    let optimizer = DesktopOptimizer::new(config).unwrap();

    // WHEN: getting performance metrics
    let metrics = optimizer.performance_metrics();

    // THEN: metrics should match config
    assert_eq!(metrics.frame_rate, 60);
    assert!(metrics.vsync_enabled);
    assert!(metrics.hardware_acceleration);
    assert!(metrics.gpu_rendering);
    assert_eq!(metrics.memory_limit_mb, Some(512));
  }

  #[test]
  fn test_desktop_optimizer_detect_system_config() {
    // WHEN: detecting system config
    let result = DesktopOptimizer::detect_system_config();

    // THEN: config should be detected successfully
    assert!(result.is_ok());
    let config = result.unwrap();
    // Config should be valid
    assert!(config.validate().is_ok());
  }

  #[test]
  fn test_create_optimized_launcher() {
    // WHEN: creating optimized launcher
    let result = create_optimized_launcher();

    // THEN: launcher should be created successfully
    assert!(result.is_ok());
    let optimizer = result.unwrap();
    // Config should be valid
    assert!(optimizer.config().validate().is_ok());
  }

  #[test]
  fn test_desktop_opt_error_display() {
    // GIVEN: various desktop opt errors
    let err1 = DesktopOptError::InvalidConfig("Invalid config".to_string());
    let err2 = DesktopOptError::OptimizationFailed("Optimization failed".to_string());
    let err3 = DesktopOptError::PlatformNotSupported("Unsupported platform".to_string());

    // WHEN: converting errors to string
    let msg1 = err1.to_string();
    let msg2 = err2.to_string();
    let msg3 = err3.to_string();

    // THEN: error messages should be descriptive
    assert!(msg1.contains("Invalid configuration"));
    assert!(msg2.contains("Optimization failed"));
    assert!(msg3.contains("Platform not supported"));
  }

  #[test]
  fn test_performance_metrics() {
    // GIVEN: a performance metrics instance
    let metrics = PerformanceMetrics {
      frame_rate: 60,
      vsync_enabled: true,
      hardware_acceleration: true,
      gpu_rendering: true,
      memory_limit_mb: Some(512),
    };

    // THEN: fields should be accessible
    assert_eq!(metrics.frame_rate, 60);
    assert!(metrics.vsync_enabled);
    assert!(metrics.hardware_acceleration);
    assert!(metrics.gpu_rendering);
    assert_eq!(metrics.memory_limit_mb, Some(512));
  }
}
