# Storage Layer Architecture

## Overview

Clarity uses **redb** (a Rust embedded database) for per-project persistence. The storage layer provides immediate, crash-safe writes with full ACID guarantees, ensuring no data loss and complete state recovery on app restart.

### Design Philosophy

- **Immediate Writes**: All state changes are persisted immediately (no batching)
- **Per-Project Databases**: Each project gets its own redb file
- **Schemaless Values**: JSON serialization for flexibility
- **Typed Keys**: String keys for human-readable lookup
- **Cache Invalidation**: Time-based and manual invalidation strategies

## Database Schema

### Table Definitions

| Table Name | Key Type | Value Type | Description |
|------------|----------|------------|-------------|
| `answers` | step_id (String) | AnswerRecord (JSON) | User responses to prompt steps |
| `extractions` | input_hash (String) | ExtractionCache (JSON) | AI field extraction results |
| `project_metadata` | "metadata" (String) | ProjectMetadata (JSON) | Project state and preferences |
| `lattice_cache` | phase (String) | LatticeCache (JSON) | Mental lattice computation results |

### Value Types

#### AnswerRecord

```rust
pub struct AnswerRecord {
    pub step_id: String,           // Unique identifier for the prompt step
    pub value: String,              // The answer value
    pub timestamp: String,          // ISO 8601 timestamp
    pub confidence: Confidence,     // High | Inferred | Uncertain
    pub ai_generated: bool,         // Whether AI suggested this answer
}
```

**Confidence Levels**:
- `High`: Direct user input or verified extraction
- `Inferred`: Derived from context or patterns
- `Uncertain`: Low confidence, requires validation

#### ExtractionCache

```rust
pub struct ExtractionCache {
    pub input_hash: String,         // Hash of input text for lookup
    pub fields: String,             // Extracted fields as JSON
    pub timestamp: String,          // ISO 8601 timestamp
}
```

**Cache Key Generation**:
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_input(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
```

#### ProjectMetadata

```rust
pub struct ProjectMetadata {
    pub mode_preference: String,    // "waterfall" | "agile"
    pub current_phase: String,       // "discover" | "define" | "develop" | "deliver"
    pub created_at: String,          // ISO 8601 timestamp
    pub updated_at: String,          // ISO 8601 timestamp
}
```

#### LatticeCache

```rust
pub struct LatticeCache {
    pub phase: String,               // Phase identifier
    pub output_data: String,         // Serialized lattice output
    pub timestamp: String,          // ISO 8601 timestamp
}
```

## Database Location

### Path Structure

```
~/.local/share/clarity/                # XDG data directory
└── projects/
    └── {project_id}/                  # UUID or user-provided ID
        ├── data.redb                  # Main database file
        ├── data.redb.lock             # File lock (managed by redb)
        └── backup/                    # Automatic backups
            ├── data.redb.2024-02-25   # Daily backup
            └── data.redb.2024-02-24
```

### Path Resolution

```rust
use crate::storage::path_util;

// Get project database path
let project_id = "my-project";
let db_path = path_util::get_project_db_path(project_id)?;

// Result: ~/.local/share/clarity/projects/my-project/data.redb
```

**Cross-Platform Support**:

| Platform | Data Directory |
|----------|----------------|
| Linux | `~/.local/share/clarity` |
| macOS | `~/Library/Application Support/clarity` |
| Windows | `%APPDATA%\clarity` |

## Write Patterns

### Immediate Write Pattern

All state changes trigger immediate writes:

```rust
// User submits answer
let answer = Answer {
    step_id: "problem".to_string(),
    value: "Users forget passwords".to_string(),
    timestamp: Utc::now().to_rfc3339(),
};

// Immediate write (no batching)
store.save_answer(&answer)?;

// Write is guaranteed to be durable after this line
```

### Transaction Pattern

redb uses MVCC for concurrent reads with exclusive writes:

```rust
pub fn save_answer(&self, answer: &Answer) -> StoreResult<()> {
    // Begin write transaction
    let txn = self.db.begin_write()?;

    {
        // Open table
        let table_definition: redb::TableDefinition<&str, &str> =
            redb::TableDefinition::new("answers");
        let mut table = txn.open_table(table_definition)?;

        // Serialize and insert
        let record = AnswerRecord::from_answer(
            answer.step_id.clone(),
            answer.value.clone(),
            answer.timestamp.clone(),
        );
        let json = serde_json::to_string(&record)?;
        table.insert(&answer.step_id, json.as_str())?;
    }

    // Commit transaction (atomic, durable)
    txn.commit()?;
    Ok(())
}
```

### Upsert Pattern

Same key → overwrite (update), new key → insert:

```rust
// Overwrites existing answer if step_id exists
store.save_answer(&answer)?;

// Insert new extraction cache
store.save_extraction_cache(&hash, &cache)?;

// Update project metadata (always overwrites)
store.save_metadata(&metadata)?;
```

### Cache Invalidation Pattern

```rust
// Check cache before expensive operation
if let Some(cached) = store.get_extraction_cache(&input_hash)? {
    // Check age (expire after 24 hours)
    let cached_time = DateTime::parse_from_rfc3339(&cached.timestamp)?;
    let age = Utc::now() - cached_time.with_timezone(&Utc);

    if age.num_hours() < 24 {
        return Ok(cached);
    }
}

// Cache miss or expired - run extraction and cache result
let result = provider.extract_fields(input, context)?;
let cache = ExtractionCache::with_current_timestamp(input_hash, serde_json::to_string(&result)?);
store.save_extraction_cache(&input_hash, &cache)?;
```

## Read Patterns

### Single Record Lookup

```rust
// Get specific answer
if let Some(answer) = store.get_answer("problem")? {
    println!("Problem: {}", answer.value);
} else {
    println!("No problem answer yet");
}
```

### Bulk Read

```rust
// Load all answers for phase restoration
let all_answers = store.get_all_answers()?;

for answer in all_answers {
    println!("{}: {}", answer.step_id, answer.value);
}
```

### Projection Pattern

Read only what you need (redb is lazy):

```rust
// Only reads metadata, not entire database
let metadata = store.get_metadata()?;

// Only reads one extraction cache entry
let cached = store.get_extraction_cache(&specific_hash)?;
```

## Backup and Restore

### Automatic Backups

Backups are created daily before first write:

```rust
pub fn maybe_create_backup(db_path: &Path) -> StoreResult<()> {
    let backup_dir = db_path.parent().unwrap().join("backup");
    std::fs::create_dir_all(&backup_dir)?;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let backup_path = backup_dir.join(format!("data.redb.{}", today));

    // Skip if backup already exists today
    if backup_path.exists() {
        return Ok(());
    }

    // Copy database file
    std::fs::copy(db_path, backup_path)?;
    Ok(())
}
```

### Manual Backup

```rust
use std::fs;

// Export project database
fn export_project(project_id: &str, dest_path: &Path) -> StoreResult<()> {
    let src = path_util::get_project_db_path(project_id)?;
    fs::copy(src, dest_path)?;
    Ok(())
}

// Import project database
fn import_project(src_path: &Path, project_id: &str) -> StoreResult<()> {
    let dest = path_util::get_project_db_path(project_id)?;
    fs::create_dir_all(dest.parent().unwrap())?;
    fs::copy(src_path, dest)?;
    Ok(())
}
```

### Restore from Backup

```rust
pub fn restore_from_backup(project_id: &str, backup_date: &str) -> StoreResult<()> {
    let db_path = path_util::get_project_db_path(project_id)?;
    let backup_path = db_path.parent()
        .unwrap()
        .join("backup")
        .join(format!("data.redb.{}", backup_date));

    // Verify backup exists
    if !backup_path.exists() {
        return Err(StorageError::BackupNotFound(backup_date.to_string()));
    }

    // Close current database if open
    drop(&self.db);

    // Replace with backup
    fs::copy(&backup_path, &db_path)?;

    // Reopen database
    self.db = redb::Database::open(&db_path)?;
    Ok(())
}
```

## Error Handling

### StorageError

```rust
pub enum StorageError {
    // IO errors
    IoError(std::io::Error),

    // Serialization errors
    Serialization(String),

    // Database errors
    Database(String),

    // Backup errors
    BackupNotFound(String),
}
```

### Error Propagation

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization failed: {0}")]
    Serialization(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Backup not found: {0}")]
    BackupNotFound(String),
}
```

## Performance Considerations

### Write Latency

Typical write latencies on local SSD:

| Operation | Latency | Notes |
|-----------|---------|-------|
| Single answer write | < 1ms | Small JSON payload |
| Extraction cache write | 1-2ms | Larger JSON payload |
| Metadata write | < 1ms | Single record |
| Bulk read (all answers) | 5-10ms | Depends on answer count |

### Optimization Strategies

1. **Avoid Unnecessary Writes**
   ```rust
   // Check if value changed before writing
   if let Some(existing) = store.get_answer(&answer.step_id)? {
       if existing.value == answer.value {
           return Ok(()); // Skip write
       }
   }
   store.save_answer(&answer)?;
   ```

2. **Batch Reads**
   ```rust
   // Load all answers once, not per-field
   let all_answers = store.get_all_answers()?;
   let answers_map: HashMap<_, _> = all_answers
       .into_iter()
       .map(|a| (a.step_id.clone(), a))
       .collect();
   ```

3. **Cache Hot Data**
   ```rust
   // Cache metadata in memory (rarely changes)
   let metadata = store.get_metadata()?.unwrap_or_default();
   // Use cached version for reads
   // Only update on explicit save
   ```

### Concurrency

redb supports:
- **Concurrent reads**: Multiple threads can read simultaneously
- **Exclusive writes**: Write transactions block reads
- **MVCC**: Readers see snapshot at transaction start

```rust
// Safe concurrent reads
let reader1 = store.get_all_answers();
let reader2 = store.get_all_answers();
// Both can proceed

// Write blocks new readers
let writer = store.save_answer(&answer)?;
```

## Data Migration

### Schema Versioning

Track schema version in metadata:

```rust
const CURRENT_SCHEMA_VERSION: u32 = 1;

pub struct ProjectMetadata {
    pub schema_version: u32,
    // ... other fields
}
```

### Migration Pattern

```rust
pub fn migrate_database(db_path: &Path) -> StoreResult<()> {
    let db = redb::Database::open(db_path)?;
    let txn = db.begin_read()?;

    // Check current version
    let metadata_table = txn.open_table::<&str, &str>(PROJECT_METADATA)?;
    let version = match metadata_table.get("metadata")? {
        Some(guard) => {
            let metadata: ProjectMetadata = serde_json::from_str(guard.value())?;
            metadata.schema_version
        }
        None => 0, // Pre-versioning database
    };

    // Run migrations
    if version < CURRENT_SCHEMA_VERSION {
        drop(txn);
        run_migrations(&db, version)?;
    }

    Ok(())
}

fn run_migrations(db: &redb::Database, from_version: u32) -> StoreResult<()> {
    match from_version {
        0 => migrate_v0_to_v1(db)?,
        1 => {}, // Already at latest
        v => Err(StorageError::UnknownSchemaVersion(v)),
    }
    Ok(())
}
```

## Testing

### In-Memory Database

Use in-memory backend for tests:

```rust
#[test]
fn test_save_and_get_answer() {
    // Create in-memory database (no file I/O)
    let store = RedbStore::open_in_memory().unwrap();

    let answer = Answer {
        step_id: "test".to_string(),
        value: "Test answer".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    store.save_answer(&answer).unwrap();

    let retrieved = store.get_answer("test").unwrap().unwrap();
    assert_eq!(retrieved.value, "Test answer");
}
```

### Temporary Database

Use temp directory for integration tests:

```rust
#[test]
fn test_persistence_across_reopen() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.redb");

    // Write
    {
        let store = RedbStore::open(&db_path).unwrap();
        store.save_answer(&answer).unwrap();
    } // Database closed

    // Read (new connection)
    {
        let store = RedbStore::open(&db_path).unwrap();
        let retrieved = store.get_answer("test").unwrap().unwrap();
        assert_eq!(retrieved.value, "Test answer");
    }
}
```

## Integration with UI

### Reactive State Loading

```rust
// Load on mount
let answers = use_resource(|| {
    let store = store.clone();
    async move {
        store.load_answers().unwrap_or_default()
    }
});

// Auto-save on change
use_effect(move || {
    let answers = answers.read().clone();
    let store = store.clone();

    async move {
        for answer in &answers {
            if let Err(e) = store.save_answer(answer) {
                eprintln!("Failed to save answer: {}", e);
            }
        }
    }
});
```

### Optimistic UI

```rust
// Update UI immediately
let mut answers = answers.write();
answers.push(new_answer);

// Persist in background
spawn(async move {
    if let Err(e) = store.save_answer(&new_answer) {
        // Revert on error
        show_error(&format!("Failed to save: {}", e));
        answers.retain(|a| a.step_id != new_answer.step_id);
    }
});
```

## Configuration

### Environment Variables

```bash
# Override data directory
CLARITY_DATA_DIR=/custom/path clarity-web

# Enable debug logging
CLARITY_LOG=debug clarity-web

# Disable automatic backups
CLARITY_NO_BACKUP=1 clarity-web
```

### File-Based Configuration

Location: `~/.config/clarity/config.toml`

```toml
[storage]
data_dir = "~/.local/share/clarity"
backup_enabled = true
backup_retention_days = 30
cache_expiry_hours = 24

[database]
max_size_mb = 100
auto_vacuum = true
```

## Monitoring and Debugging

### Database Statistics

```rust
pub fn get_stats(&self) -> StoreResult<DatabaseStats> {
    let txn = self.db.begin_read()?;

    let answers_table = txn.open_table::<&str, &str>(tables::ANSWERS)?;
    let answer_count = answers_table.len()?;

    let extractions_table = txn.open_table::<&str, &str>(tables::EXTRACTIONS)?;
    let extraction_count = extractions_table.len()?;

    Ok(DatabaseStats {
        answer_count,
        extraction_count,
        db_size_bytes: self.db.size()?,
    })
}
```

### Logging

```rust
use tracing::{info, debug, error};

pub fn save_answer(&self, answer: &Answer) -> StoreResult<()> {
    debug!("Saving answer for step: {}", answer.step_id);

    let txn = self.db.begin_write()?;

    // ... write logic ...

    match txn.commit() {
        Ok(_) => {
            info!("Successfully saved answer: {}", answer.step_id);
            Ok(())
        }
        Err(e) => {
            error!("Failed to save answer {}: {:?}", answer.step_id, e);
            Err(StorageError::Database(e.to_string()))
        }
    }
}
```

## Troubleshooting

### Common Issues

**Issue: "Database locked" error**

```
Error: Database locked
```

**Cause**: Another process has the database open

**Solution**:
```bash
# Check for running processes
ps aux | grep clarity-web

# Kill stale processes
killall clarity-web

# Remove lock file (if absolutely sure no other process is running)
rm ~/.local/share/clarity/projects/my-project/data.redb.lock
```

**Issue: "Corruption detected"**

```
Error: Checksum mismatch
```

**Cause**: Database file corrupted

**Solution**:
```bash
# Restore from most recent backup
cp ~/.local/share/clarity/projects/my-project/backup/data.redb.YYYY-MM-DD \
   ~/.local/share/clarity/projects/my-project/data.redb
```

**Issue: Slow writes**

**Cause**: Writing to network drive or slow storage

**Solution**:
1. Move database to local SSD
2. Increase write cache size
3. Disable fsync (not recommended for production)

## Best Practices

### DO

- Write immediately after state changes
- Handle write errors gracefully
- Create backups before major changes
- Use transactions for multi-table operations
- Validate data before serialization

### DON'T

- Batch writes unnecessarily
- Ignore write errors
- Store large binary data (>1MB) in redb
- Modify database file externally while app is running
- Assume writes are synchronous without checking errors

## Future Enhancements

### Planned Features

1. **Compression**: Compress large text fields before storage
2. **Indexing**: Add secondary indexes for faster queries
3. **Replication**: Multi-master replication for collaboration
4. **Cloud Sync**: Optional cloud backup and sync
5. **Encryption**: At-rest encryption for sensitive projects
6. **Sharding**: Distribute large projects across multiple files
