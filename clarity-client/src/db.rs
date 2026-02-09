#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Desktop database module for client-side SQLite access

use anyhow::Result;
use clarity_core::db::models::{Bead, BeadFilters, BeadId, NewBead};
use rusqlite::{params, Connection};

/// Desktop database wrapper
#[derive(Debug)]
pub struct DesktopDb {
    conn: Connection,
}

impl DesktopDb {
    /// Create a new DesktopDb with default path
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine local data directory"))?;

        let app_dir = data_dir.join("clarity");
        std::fs::create_dir_all(&app_dir)?;

        let db_path = app_dir.join("clarity.db");
        let conn = Connection::open(&db_path)?;

        // Run migrations
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS beads (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                priority INTEGER NOT NULL,
                bead_type TEXT NOT NULL,
                created_by TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_beads_status ON beads(status);
            CREATE INDEX IF NOT EXISTS idx_beads_type ON beads(bead_type);
            CREATE INDEX IF NOT EXISTS idx_beads_priority ON beads(priority);
            ",
        )?;

        Ok(Self { conn })
    }

    /// List all beads without filtering
    pub fn list_beads(&self) -> Result<Vec<Bead>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
             FROM beads ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>("id")?,
                row.get::<_, String>("title")?,
                row.get::<_, Option<String>>("description")?,
                row.get::<_, String>("status")?,
                row.get::<_, i16>("priority")?,
                row.get::<_, String>("bead_type")?,
                row.get::<_, Option<String>>("created_by")?,
                row.get::<_, String>("created_at")?,
                row.get::<_, String>("updated_at")?,
            ))
        })?;

        let mut beads = Vec::new();
        for row_result in rows {
            let (id_str, title, description, status_str, priority_val, type_str, created_by_str, created_at_str, updated_at_str) = row_result?;

            let id = BeadId::from_str(&id_str)?;
            let status = status_str.parse()?;
            let bead_type = type_str.parse()?;
            let priority = clarity_core::db::models::BeadPriority::new(priority_val)?;

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)?
                .with_timezone(&chrono::Utc);

            let created_by = created_by_str
                .map(|s| Ok::<_, anyhow::Error>(clarity_core::db::models::UserId::from_str(&s)?))
                .transpose()?;

            beads.push(Bead {
                id, title, description, status, priority, bead_type, created_by, created_at, updated_at,
            });
        }

        Ok(beads)
    }

    /// List beads with filtering
    pub fn list_beads_filtered(&self, filters: &BeadFilters) -> Result<Vec<Bead>> {
        let mut query = String::from(
            "SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
             FROM beads WHERE 1=1"
        );

        if let Some(ref status) = filters.status {
            query.push_str(&format!(" AND status = '{}'", status.replace('\'', "''")));
        }
        if let Some(ref bead_type) = filters.bead_type {
            query.push_str(&format!(" AND bead_type = '{}'", bead_type.replace('\'', "''")));
        }
        if let Some(priority) = filters.priority {
            query.push_str(&format!(" AND priority = {}", priority));
        }
        if let Some(ref search) = filters.search {
            query.push_str(&format!(" AND (title LIKE '%{}%' OR description LIKE '%{}%')", search, search));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut stmt = self.conn.prepare(&query)?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>("id")?,
                row.get::<_, String>("title")?,
                row.get::<_, Option<String>>("description")?,
                row.get::<_, String>("status")?,
                row.get::<_, i16>("priority")?,
                row.get::<_, String>("bead_type")?,
                row.get::<_, Option<String>>("created_by")?,
                row.get::<_, String>("created_at")?,
                row.get::<_, String>("updated_at")?,
            ))
        })?;

        let mut beads = Vec::new();
        for row_result in rows {
            let (id_str, title, description, status_str, priority_val, type_str, created_by_str, created_at_str, updated_at_str) = row_result?;

            let id = BeadId::from_str(&id_str)?;
            let status = status_str.parse()?;
            let bead_type = type_str.parse()?;
            let priority = clarity_core::db::models::BeadPriority::new(priority_val)?;

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)?
                .with_timezone(&chrono::Utc);

            let created_by = created_by_str
                .map(|s| Ok::<_, anyhow::Error>(clarity_core::db::models::UserId::from_str(&s)?))
                .transpose()?;

            beads.push(Bead {
                id, title, description, status, priority, bead_type, created_by, created_at, updated_at,
            });
        }

        Ok(beads)
    }

    /// Get a single bead by ID
    pub fn get_bead(&self, id: BeadId) -> Result<Bead> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
             FROM beads WHERE id = ?",
        )?;

        let (id_str, title, description, status_str, priority_val, type_str, created_by_str, created_at_str, updated_at_str) =
            stmt.query_row(params![id.to_string()], |row| {
                Ok((
                    row.get::<_, String>("id")?,
                    row.get::<_, String>("title")?,
                    row.get::<_, Option<String>>("description")?,
                    row.get::<_, String>("status")?,
                    row.get::<_, i16>("priority")?,
                    row.get::<_, String>("bead_type")?,
                    row.get::<_, Option<String>>("created_by")?,
                    row.get::<_, String>("created_at")?,
                    row.get::<_, String>("updated_at")?,
                ))
            })?;

        let id = BeadId::from_str(&id_str)?;
        let status = status_str.parse()?;
        let bead_type = type_str.parse()?;
        let priority = clarity_core::db::models::BeadPriority::new(priority_val)?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
            .with_timezone(&chrono::Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)?
            .with_timezone(&chrono::Utc);

        let created_by = created_by_str
            .map(|s| Ok::<_, anyhow::Error>(clarity_core::db::models::UserId::from_str(&s)?))
            .transpose()?;

        Ok(Bead {
            id, title, description, status, priority, bead_type, created_by, created_at, updated_at,
        })
    }

    /// Create a new bead
    pub fn create_bead(&self, bead: NewBead) -> Result<Bead> {
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();

        self.conn.execute(
            "INSERT INTO beads (id, title, description, status, priority, bead_type, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id.to_string(),
                bead.title,
                bead.description,
                bead.status.as_str(),
                bead.priority.0,
                bead.bead_type.as_str(),
                bead.created_by.map(|u| u.to_string()),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        self.get_bead(BeadId::from(id))
    }

    /// Update an existing bead
    pub fn update_bead(&self, id: BeadId, bead: NewBead) -> Result<Bead> {
        let now = chrono::Utc::now();

        let rows_affected = self.conn.execute(
            "UPDATE beads
             SET title = ?, description = ?, status = ?, priority = ?, bead_type = ?, created_by = ?, updated_at = ?
             WHERE id = ?",
            params![
                bead.title,
                bead.description,
                bead.status.as_str(),
                bead.priority.0,
                bead.bead_type.as_str(),
                bead.created_by.map(|u| u.to_string()),
                now.to_rfc3339(),
                id.to_string(),
            ],
        )?;

        if rows_affected == 0 {
            anyhow::bail!("Bead not found: {id}");
        }

        self.get_bead(id)
    }

    /// Delete a bead
    pub fn delete_bead(&self, id: BeadId) -> Result<()> {
        let rows_affected = self.conn.execute("DELETE FROM beads WHERE id = ?", params![id.to_string()])?;

        if rows_affected == 0 {
            anyhow::bail!("Bead not found: {id}");
        }

        Ok(())
    }
}
