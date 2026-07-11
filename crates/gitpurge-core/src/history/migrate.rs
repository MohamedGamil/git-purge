//! SQLite Schema Migrations (doc 10 §3.2)

use rusqlite::Connection;
use crate::error::Result;

/// Check user_version pragma and run schema creation/migrations.
pub fn migrate(conn: &mut Connection) -> Result<()> {
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| crate::GitPurgeError::Config(format!("Failed to read schema version: {}", e)))?;

    if version == 0 {
        // Run database initialization DDL
        conn.execute_batch(include_str!("schema.sql"))
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to initialize database: {}", e)))?;
        
        // Set version to 1
        conn.execute("PRAGMA user_version = 1;", [])
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to set schema version: {}", e)))?;
    }

    Ok(())
}
