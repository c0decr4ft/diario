//! Database operations module for SQLite storage
//!
//! This module handles all database operations including:
//! - Database initialization and migrations
//! - CRUD operations for homework entries
//! - Study session management
//! - Position management for drag-drop reordering

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use tracing::{debug, info};

use crate::types::HomeworkEntry;

/// Initialize the database at the given path, running any pending migrations
pub fn init_db(db_path: &Path, migrations_dir: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let count = run_migrations(&conn, migrations_dir)?;
    if count > 0 {
        info!(count = count, "Applied migrations");
    }

    Ok(conn)
}

/// Run pending migrations from the migrations directory
pub fn run_migrations(conn: &Connection, migrations_dir: &Path) -> Result<usize> {
    // First, ensure the schema_migrations table exists
    // We need to check if any tables exist first
    let tables_exist: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !tables_exist {
        // No migrations table, so we need to run the initial migration
        // But first check if it exists in the migrations dir
    }

    // Get list of migration files
    let mut migrations: Vec<_> = std::fs::read_dir(migrations_dir)
        .with_context(|| {
            format!(
                "Failed to read migrations directory: {}",
                migrations_dir.display()
            )
        })?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "sql")
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();

    migrations.sort();

    let mut applied = 0;

    for migration_path in migrations {
        let version = migration_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid migration filename"))?
            .to_string();

        // Check if already applied
        let already_applied: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM schema_migrations WHERE version = ?1",
                [&version],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if already_applied {
            continue;
        }

        // Read and execute migration
        let sql = std::fs::read_to_string(&migration_path)
            .with_context(|| format!("Failed to read migration: {}", migration_path.display()))?;

        conn.execute_batch(&sql)
            .with_context(|| format!("Failed to apply migration: {}", version))?;

        // Record migration
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            [&version],
        )?;

        debug!(version = %version, "Applied migration");
        applied += 1;
    }

    Ok(applied)
}

/// Import multiple entries into the database, skipping duplicates based on source_id.
/// Returns the number of entries actually inserted.
pub fn import_entries(conn: &Connection, entries: &[HomeworkEntry]) -> Result<usize> {
    let mut count = 0;
    for entry in entries {
        if insert_entry_if_not_exists(conn, entry)? {
            count += 1;
        }
    }
    Ok(count)
}

/// Get all entries from the database, sorted by date and position
pub fn get_all_entries(conn: &Connection) -> Result<Vec<HomeworkEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, entry_type, date, subject, task, completed, position, parent_id, created_at, updated_at
         FROM entries
         ORDER BY date ASC, position ASC"
    )?;

    let entries = stmt
        .query_map([], |row| {
            Ok(HomeworkEntry {
                id: row.get(0)?,
                source_id: row.get(1)?,
                entry_type: row.get(2)?,
                date: row.get(3)?,
                subject: row.get(4)?,
                task: row.get(5)?,
                completed: row.get::<_, i32>(6)? != 0,
                position: row.get(7)?,
                parent_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

/// Get a single entry by ID
pub fn get_entry(conn: &Connection, id: &str) -> Result<Option<HomeworkEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, entry_type, date, subject, task, completed, position, parent_id, created_at, updated_at
         FROM entries
         WHERE id = ?1"
    )?;

    let entry = stmt
        .query_row([id], |row| {
            Ok(HomeworkEntry {
                id: row.get(0)?,
                source_id: row.get(1)?,
                entry_type: row.get(2)?,
                date: row.get(3)?,
                subject: row.get(4)?,
                task: row.get(5)?,
                completed: row.get::<_, i32>(6)? != 0,
                position: row.get(7)?,
                parent_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .optional()?;

    Ok(entry)
}

/// Insert a new entry into the database
pub fn insert_entry(conn: &Connection, entry: &HomeworkEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO entries (id, source_id, entry_type, date, subject, task, completed, position, parent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            entry.id,
            entry.source_id,
            entry.entry_type,
            entry.date,
            entry.subject,
            entry.task,
            entry.completed as i32,
            entry.position,
            entry.parent_id,
            entry.created_at,
            entry.updated_at,
        ],
    )?;
    Ok(())
}

/// Insert an entry only if no entry with the same source_id already exists.
/// This allows entries to be moved to different dates while still being
/// recognized as duplicates during future imports.
pub fn insert_entry_if_not_exists(conn: &Connection, entry: &HomeworkEntry) -> Result<bool> {
    // Check if an entry with this source_id already exists
    if let Some(ref source_id) = entry.source_id {
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM entries WHERE source_id = ?1",
            [source_id],
            |row| row.get(0),
        )?;
        if exists {
            return Ok(false);
        }
    }

    // No duplicate found, insert the entry
    conn.execute(
        "INSERT INTO entries (id, source_id, entry_type, date, subject, task, completed, position, parent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            entry.id,
            entry.source_id,
            entry.entry_type,
            entry.date,
            entry.subject,
            entry.task,
            entry.completed as i32,
            entry.position,
            entry.parent_id,
            entry.created_at,
            entry.updated_at,
        ],
    )?;
    Ok(true)
}

/// Helper struct for partial entry updates
#[derive(Default)]
pub struct EntryUpdate {
    pub date: Option<String>,
    pub completed: Option<bool>,
    pub position: Option<i32>,
    pub task: Option<String>,
}

/// Update an existing entry
pub fn update_entry(conn: &Connection, id: &str, updates: &EntryUpdate) -> Result<bool> {
    let mut set_clauses = vec!["updated_at = datetime('now')"];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref date) = updates.date {
        set_clauses.push("date = ?");
        params_vec.push(Box::new(date.clone()));
    }
    if let Some(completed) = updates.completed {
        set_clauses.push("completed = ?");
        params_vec.push(Box::new(completed as i32));
    }
    if let Some(position) = updates.position {
        set_clauses.push("position = ?");
        params_vec.push(Box::new(position));
    }
    if let Some(ref task) = updates.task {
        set_clauses.push("task = ?");
        params_vec.push(Box::new(task.clone()));
    }

    params_vec.push(Box::new(id.to_string()));

    let sql = format!("UPDATE entries SET {} WHERE id = ?", set_clauses.join(", "));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let affected = conn.execute(&sql, params_refs.as_slice())?;
    Ok(affected > 0)
}

/// Delete an entry by ID (orphans children by setting their parent_id to NULL)
pub fn delete_entry(conn: &Connection, id: &str) -> Result<bool> {
    let affected = conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;
    Ok(affected > 0)
}

/// Get all child entries (study sessions) for a parent entry
pub fn get_children(conn: &Connection, parent_id: &str) -> Result<Vec<HomeworkEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, entry_type, date, subject, task, completed, position, parent_id, created_at, updated_at
         FROM entries
         WHERE parent_id = ?1
         ORDER BY date ASC"
    )?;

    let entries = stmt
        .query_map([parent_id], |row| {
            Ok(HomeworkEntry {
                id: row.get(0)?,
                source_id: row.get(1)?,
                entry_type: row.get(2)?,
                date: row.get(3)?,
                subject: row.get(4)?,
                task: row.get(5)?,
                completed: row.get::<_, i32>(6)? != 0,
                position: row.get(7)?,
                parent_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

/// Delete an entry and all its children (cascade delete)
pub fn delete_with_children(conn: &Connection, id: &str) -> Result<usize> {
    // First delete children
    let children_deleted = conn.execute("DELETE FROM entries WHERE parent_id = ?1", [id])?;

    // Then delete the entry itself
    let entry_deleted = conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;

    Ok(children_deleted + entry_deleted)
}

/// Get the maximum position for entries on a specific date
pub fn get_max_position_for_date(conn: &Connection, date: &str) -> Result<i32> {
    let max: Option<i32> = conn.query_row(
        "SELECT MAX(position) FROM entries WHERE date = ?1",
        [date],
        |row| row.get(0),
    )?;
    Ok(max.unwrap_or(-1))
}

/// Reorder entries for a specific date based on the provided ID order
#[cfg(test)]
pub fn reorder_entries(conn: &Connection, date: &str, entry_ids: &[&str]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    for (position, id) in entry_ids.iter().enumerate() {
        tx.execute(
            "UPDATE entries SET position = ?1, updated_at = datetime('now') WHERE id = ?2 AND date = ?3",
            params![position as i32, id, date],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Check if an entry with the given ID exists
#[cfg(test)]
pub fn entry_exists(conn: &Connection, id: &str) -> Result<bool> {
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM entries WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(exists)
}

/// Count all entries in the database
pub fn count_entries(conn: &Connection) -> Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))?;
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();

        // Create the initial migration
        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        let conn = init_db(&db_path, &migrations_dir).unwrap();
        (temp_dir, conn)
    }

    fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry {
        HomeworkEntry::new(
            entry_type.to_string(),
            date.to_string(),
            subject.to_string(),
            task.to_string(),
        )
    }

    // ========== init_db tests ==========

    #[test]
    fn test_init_db_creates_tables() {
        let (_temp_dir, conn) = setup_test_db();

        // Check that entries table exists
        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='entries'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists);
    }

    #[test]
    fn test_init_db_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();

        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        // Initialize twice
        let _conn1 = init_db(&db_path, &migrations_dir).unwrap();
        drop(_conn1);
        let conn2 = init_db(&db_path, &migrations_dir).unwrap();

        // Should still work
        let count = count_entries(&conn2).unwrap();
        assert_eq!(count, 0);
    }

    // ========== CRUD tests ==========

    #[test]
    fn test_insert_and_get_entry() {
        let (_temp_dir, conn) = setup_test_db();
        let entry = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");

        insert_entry(&conn, &entry).unwrap();

        let retrieved = get_entry(&conn, &entry.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, entry.id);
        assert_eq!(retrieved.entry_type, "compiti");
        assert_eq!(retrieved.date, "2025-01-15");
        assert_eq!(retrieved.subject, "MATEMATICA");
        assert_eq!(retrieved.task, "Task 1");
    }

    #[test]
    fn test_get_nonexistent_entry() {
        let (_temp_dir, conn) = setup_test_db();
        let retrieved = get_entry(&conn, "nonexistent").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_get_all_entries_sorted() {
        let (_temp_dir, conn) = setup_test_db();

        let entry1 = make_entry("compiti", "2025-01-20", "MATEMATICA", "Task 3");
        let entry2 = make_entry("nota", "2025-01-10", "ITALIANO", "Task 1");
        let entry3 = make_entry("compiti", "2025-01-15", "INGLESE", "Task 2");

        insert_entry(&conn, &entry1).unwrap();
        insert_entry(&conn, &entry2).unwrap();
        insert_entry(&conn, &entry3).unwrap();

        let entries = get_all_entries(&conn).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].date, "2025-01-10");
        assert_eq!(entries[1].date, "2025-01-15");
        assert_eq!(entries[2].date, "2025-01-20");
    }

    #[test]
    fn test_insert_entry_if_not_exists() {
        let (_temp_dir, conn) = setup_test_db();
        let entry = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");

        // First insert should succeed
        let inserted = insert_entry_if_not_exists(&conn, &entry).unwrap();
        assert!(inserted);

        // Second insert should be ignored
        let inserted = insert_entry_if_not_exists(&conn, &entry).unwrap();
        assert!(!inserted);

        // Only one entry should exist
        assert_eq!(count_entries(&conn).unwrap(), 1);
    }

    #[test]
    fn test_update_entry() {
        let (_temp_dir, conn) = setup_test_db();
        let entry = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");
        insert_entry(&conn, &entry).unwrap();

        let updates = EntryUpdate {
            completed: Some(true),
            position: Some(5),
            ..Default::default()
        };

        let updated = update_entry(&conn, &entry.id, &updates).unwrap();
        assert!(updated);

        let retrieved = get_entry(&conn, &entry.id).unwrap().unwrap();
        assert!(retrieved.completed);
        assert_eq!(retrieved.position, 5);
    }

    #[test]
    fn test_update_nonexistent_entry() {
        let (_temp_dir, conn) = setup_test_db();
        let updates = EntryUpdate {
            completed: Some(true),
            ..Default::default()
        };

        let updated = update_entry(&conn, "nonexistent", &updates).unwrap();
        assert!(!updated);
    }

    #[test]
    fn test_delete_entry() {
        let (_temp_dir, conn) = setup_test_db();
        let entry = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");
        insert_entry(&conn, &entry).unwrap();

        let deleted = delete_entry(&conn, &entry.id).unwrap();
        assert!(deleted);

        let retrieved = get_entry(&conn, &entry.id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_delete_nonexistent_entry() {
        let (_temp_dir, conn) = setup_test_db();
        let deleted = delete_entry(&conn, "nonexistent").unwrap();
        assert!(!deleted);
    }

    // ========== Parent/child relationship tests ==========

    #[test]
    fn test_get_children() {
        let (_temp_dir, conn) = setup_test_db();

        let parent = make_entry("compiti", "2025-01-20", "MATEMATICA", "Test");
        insert_entry(&conn, &parent).unwrap();

        let mut child1 = HomeworkEntry::with_id(
            "child1".to_string(),
            "studio".to_string(),
            "2025-01-18".to_string(),
            "MATEMATICA".to_string(),
            "Study for: Test".to_string(),
        );
        child1.parent_id = Some(parent.id.clone());
        insert_entry(&conn, &child1).unwrap();

        let mut child2 = HomeworkEntry::with_id(
            "child2".to_string(),
            "studio".to_string(),
            "2025-01-19".to_string(),
            "MATEMATICA".to_string(),
            "Study for: Test".to_string(),
        );
        child2.parent_id = Some(parent.id.clone());
        insert_entry(&conn, &child2).unwrap();

        let children = get_children(&conn, &parent.id).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].date, "2025-01-18");
        assert_eq!(children[1].date, "2025-01-19");
    }

    #[test]
    fn test_delete_with_children() {
        let (_temp_dir, conn) = setup_test_db();

        let parent = make_entry("compiti", "2025-01-20", "MATEMATICA", "Test");
        insert_entry(&conn, &parent).unwrap();

        let mut child = HomeworkEntry::with_id(
            "child1".to_string(),
            "studio".to_string(),
            "2025-01-18".to_string(),
            "MATEMATICA".to_string(),
            "Study for: Test".to_string(),
        );
        child.parent_id = Some(parent.id.clone());
        insert_entry(&conn, &child).unwrap();

        let deleted = delete_with_children(&conn, &parent.id).unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(count_entries(&conn).unwrap(), 0);
    }

    #[test]
    fn test_delete_parent_orphans_children() {
        let (_temp_dir, conn) = setup_test_db();

        let parent = make_entry("compiti", "2025-01-20", "MATEMATICA", "Test");
        insert_entry(&conn, &parent).unwrap();

        let mut child = HomeworkEntry::with_id(
            "child1".to_string(),
            "studio".to_string(),
            "2025-01-18".to_string(),
            "MATEMATICA".to_string(),
            "Study for: Test".to_string(),
        );
        child.parent_id = Some(parent.id.clone());
        insert_entry(&conn, &child).unwrap();

        // Delete only the parent
        delete_entry(&conn, &parent.id).unwrap();

        // Child should still exist with NULL parent (orphaned)
        let orphan = get_entry(&conn, "child1").unwrap().unwrap();
        assert!(orphan.parent_id.is_none()); // Foreign key ON DELETE SET NULL
    }

    // ========== Position management tests ==========

    #[test]
    fn test_get_max_position_for_date() {
        let (_temp_dir, conn) = setup_test_db();

        let mut entry1 = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");
        entry1.position = 0;
        insert_entry(&conn, &entry1).unwrap();

        let mut entry2 = make_entry("nota", "2025-01-15", "ITALIANO", "Task 2");
        entry2.position = 5;
        insert_entry(&conn, &entry2).unwrap();

        let max = get_max_position_for_date(&conn, "2025-01-15").unwrap();
        assert_eq!(max, 5);
    }

    #[test]
    fn test_get_max_position_for_empty_date() {
        let (_temp_dir, conn) = setup_test_db();
        let max = get_max_position_for_date(&conn, "2025-01-15").unwrap();
        assert_eq!(max, -1);
    }

    #[test]
    fn test_reorder_entries() {
        let (_temp_dir, conn) = setup_test_db();

        let entry1 = HomeworkEntry::with_id(
            "id1".to_string(),
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Task 1".to_string(),
        );
        let entry2 = HomeworkEntry::with_id(
            "id2".to_string(),
            "nota".to_string(),
            "2025-01-15".to_string(),
            "ITALIANO".to_string(),
            "Task 2".to_string(),
        );
        let entry3 = HomeworkEntry::with_id(
            "id3".to_string(),
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "INGLESE".to_string(),
            "Task 3".to_string(),
        );

        insert_entry(&conn, &entry1).unwrap();
        insert_entry(&conn, &entry2).unwrap();
        insert_entry(&conn, &entry3).unwrap();

        // Reorder to: id3, id1, id2
        reorder_entries(&conn, "2025-01-15", &["id3", "id1", "id2"]).unwrap();

        let e1 = get_entry(&conn, "id1").unwrap().unwrap();
        let e2 = get_entry(&conn, "id2").unwrap().unwrap();
        let e3 = get_entry(&conn, "id3").unwrap().unwrap();

        assert_eq!(e3.position, 0);
        assert_eq!(e1.position, 1);
        assert_eq!(e2.position, 2);
    }

    // ========== Entry exists test ==========

    #[test]
    fn test_entry_exists() {
        let (_temp_dir, conn) = setup_test_db();
        let entry = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");

        assert!(!entry_exists(&conn, &entry.id).unwrap());

        insert_entry(&conn, &entry).unwrap();

        assert!(entry_exists(&conn, &entry.id).unwrap());
    }

    // ========== import_entries tests ==========

    #[test]
    fn test_import_entries() {
        let (_temp_dir, conn) = setup_test_db();

        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];

        let count = import_entries(&conn, &entries).unwrap();
        assert_eq!(count, 2);

        let all = get_all_entries(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_import_entries_empty() {
        let (_temp_dir, conn) = setup_test_db();

        let count = import_entries(&conn, &[]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_import_entries_skips_duplicates() {
        let (_temp_dir, conn) = setup_test_db();

        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        // Import once
        let count1 = import_entries(&conn, &entries).unwrap();
        assert_eq!(count1, 1);

        // Import same entries again
        let count2 = import_entries(&conn, &entries).unwrap();
        assert_eq!(count2, 0); // Should skip duplicates

        assert_eq!(count_entries(&conn).unwrap(), 1);
    }

    #[test]
    fn test_import_entries_partial_duplicates() {
        let (_temp_dir, conn) = setup_test_db();

        let entries1 = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        import_entries(&conn, &entries1).unwrap();

        // Import with one existing and one new
        let entries2 = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"), // duplicate
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),      // new
        ];

        let count = import_entries(&conn, &entries2).unwrap();
        assert_eq!(count, 1); // Only the new one

        assert_eq!(count_entries(&conn).unwrap(), 2);
    }

    #[test]
    fn test_moved_entry_not_reimported() {
        // This tests the key behavior: when an entry is moved to a different date,
        // re-importing the original entry (same source_id) doesn't create a duplicate.
        let (_temp_dir, conn) = setup_test_db();

        // Import an entry originally on 2025-01-15
        let original = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");
        let original_source_id = original.source_id.clone();
        import_entries(&conn, &[original.clone()]).unwrap();
        assert_eq!(count_entries(&conn).unwrap(), 1);

        // Simulate user moving the entry to 2025-01-20 via the UI
        let updates = EntryUpdate {
            date: Some("2025-01-20".to_string()),
            ..Default::default()
        };
        update_entry(&conn, &original.id, &updates).unwrap();

        // Verify the entry was moved
        let moved = get_entry(&conn, &original.id).unwrap().unwrap();
        assert_eq!(moved.date, "2025-01-20");
        // source_id should remain unchanged (still references original date)
        assert_eq!(moved.source_id, original_source_id);

        // Now re-import the same entry (as if processing exports again)
        // This creates a new HomeworkEntry with the same content as the original
        let reimport = make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1");
        // The reimport should have the same source_id
        assert_eq!(reimport.source_id, original_source_id);

        let count = import_entries(&conn, &[reimport]).unwrap();
        // Should NOT insert - the source_id already exists
        assert_eq!(count, 0);

        // Should still have only 1 entry
        assert_eq!(count_entries(&conn).unwrap(), 1);

        // And it should still be on the moved date
        let entries = get_all_entries(&conn).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].date, "2025-01-20");
    }
}
