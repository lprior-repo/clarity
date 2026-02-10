#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Test embedded SQLite database initialization for desktop app

use anyhow::Result;
use clarity_core::db::sqlite_pool::{create_sqlite_pool, SqliteDbConfig};
use clarity_core::db::migrate::run_sqlite_migrations;
use std::path::PathBuf;

/// Desktop database configuration
#[derive(Debug, Clone)]
struct DesktopDbConfig {
    db_path: PathBuf,
}

impl DesktopDbConfig {
    fn new() -> Result<Self> {
        Self::with_path("test_clarity.db")
    }

    fn with_path(db_filename: &str) -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine local data directory"))?;

        let app_dir = data_dir.join("clarity");
        std::fs::create_dir_all(&app_dir)
            .context(format!("Failed to create data directory at: {}", app_dir.display()))?;

        let db_path = app_dir.join(db_filename);
        Ok(Self { db_path })
    }

    fn to_sqlite_config(&self) -> SqliteDbConfig {
        SqliteDbConfig::new(format!("sqlite:{}", self.db_path.display()))
    }

    pub fn db_path_str(&self) -> String {
        self.db_path.display().to_string()
    }
}

async fn initialize_database() -> Result<SqliteDbConfig> {
    let db_config = DesktopDbConfig::new()
        .context("Failed to create desktop database configuration")?;

    eprintln!("Clarity test database: {}", db_config.db_path_str());

    let sqlite_config = db_config.to_sqlite_config();

    let pool = create_sqlite_pool(&sqlite_config)
        .await
        .context("Failed to create SQLite connection pool")?;

    run_sqlite_migrations(&pool)
        .await
        .context("Failed to run database migrations")?;

    pool.close().await;

    Ok(sqlite_config)
}

#[tokio::test]
async fn test_database_initialization() -> Result<()> {
    let db_config = initialize_database().await?;
    eprintln!("Database initialized successfully at: {}", db_config.database_url);

    // Clean up test database
    let test_path = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get data dir"))?
        .join("clarity")
        .join("test_clarity.db");

    if test_path.exists() {
        std::fs::remove_file(&test_path)
            .context("Failed to remove test database")?;
        eprintln!("Cleaned up test database");
    }

    Ok(())
}

#[test]
fn test_desktop_db_config_new() {
    let result = DesktopDbConfig::new();
    assert!(result.is_ok(), "Should create desktop DB config");

    let config = result.unwrap();
    assert!(config.db_path.ends_with("test_clarity.db"));
    eprintln!("Database path: {}", config.db_path.display());
}

#[test]
fn test_directory_creation() {
    let temp_config = DesktopDbConfig::with_path("test_creation.db")
        .expect("Should create temp config");

    assert!(
        temp_config.db_path.parent().map_or(false, |p| p.exists()),
        "Parent directory should be created"
    );

    if temp_config.db_path.exists() {
        std::fs::remove_file(&temp_config.db_path)
            .expect("Should remove test database file");
    }
}
