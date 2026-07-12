//! SQLite Schema Migrations (doc 10 §3.2)

use crate::error::Result;
use rusqlite::Connection;

/// Check user_version pragma and run schema creation/migrations.
pub fn migrate(conn: &mut Connection) -> Result<()> {
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to read schema version: {}", e))
        })?;

    let mut current_version = version;

    if current_version == 0 {
        // Run database initialization DDL
        conn.execute_batch(include_str!("schema.sql"))
            .map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to initialize database: {}", e))
            })?;

        // Set version to 2
        conn.execute("PRAGMA user_version = 2;", []).map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to set schema version to 2: {}", e))
        })?;
        current_version = 2;
    }

    if current_version < 2 {
        // Migration to version 2: Add backup_path column to snapshots
        let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN backup_path TEXT;", []);

        conn.execute("PRAGMA user_version = 2;", []).map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to set schema version to 2: {}", e))
        })?;
    }

    Ok(())
}
