use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::parser;
use crate::types::HomeworkEntry;

/// Process all export files and merge with existing data
pub fn process_all_exports(output_dir: &Path) -> Result<Vec<HomeworkEntry>> {
    let json_path = output_dir.join("homework.json");

    // Load existing entries
    let existing_entries = load_existing_entries(&json_path).unwrap_or_default();
    let existing_count = existing_entries.len();

    // Find and process all export files
    let files = find_all_exports()?;

    if files.is_empty() {
        if existing_entries.is_empty() {
            anyhow::bail!("No export files found in data/ and no existing data.");
        }
        println!("No export files found, using existing data.");
        return Ok(existing_entries);
    }

    let mut new_entries: Vec<HomeworkEntry> = Vec::new();
    for file in &files {
        println!("Processing: {}", file.display());
        match parser::parse_excel_xml(file) {
            Ok(entries) => {
                println!("  Found {} entries", entries.len());
                new_entries.extend(entries);
            }
            Err(e) => {
                eprintln!("  Warning: Failed to parse {}: {}", file.display(), e);
            }
        }
    }

    // Merge and deduplicate
    let all_entries = merge_and_deduplicate(existing_entries, new_entries);
    let new_count = all_entries.len().saturating_sub(existing_count);

    println!("Total entries: {} ({} new)", all_entries.len(), new_count);

    // Save updated JSON
    save_json(&all_entries, &json_path)?;
    println!("Data saved: {}", json_path.display());

    Ok(all_entries)
}

/// Load existing entries from JSON file
fn load_existing_entries(path: &PathBuf) -> Result<Vec<HomeworkEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path).context("Failed to read existing JSON")?;
    let entries: Vec<HomeworkEntry> =
        serde_json::from_str(&content).context("Failed to parse existing JSON")?;

    println!("Loaded {} existing entries", entries.len());
    Ok(entries)
}

/// Find all export files in data/ directory
fn find_all_exports() -> Result<Vec<PathBuf>> {
    let data_dir = PathBuf::from("data");

    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files: Vec<_> = std::fs::read_dir(&data_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("export_") && n.contains(".xls"))
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();

    files.sort();
    Ok(files)
}

/// Merge new entries with existing, removing duplicates
fn merge_and_deduplicate(
    existing: Vec<HomeworkEntry>,
    new: Vec<HomeworkEntry>,
) -> Vec<HomeworkEntry> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<HomeworkEntry> = Vec::new();

    // Add existing entries first
    for entry in existing {
        let key = entry.dedup_key();
        if seen.insert(key) {
            result.push(entry);
        }
    }

    // Add new entries if not duplicates
    for entry in new {
        let key = entry.dedup_key();
        if seen.insert(key) {
            result.push(entry);
        }
    }

    // Sort by date
    result.sort_by(|a, b| a.date.cmp(&b.date));

    result
}

fn save_json(entries: &[HomeworkEntry], path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(entries)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to prevent concurrent directory changes in tests
    static DIR_LOCK: Mutex<()> = Mutex::new(());

    /// Helper to create a HomeworkEntry
    fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry {
        HomeworkEntry::new(
            entry_type.to_string(),
            date.to_string(),
            subject.to_string(),
            task.to_string(),
        )
    }

    /// Helper to run a test with a changed directory, ensuring cleanup
    fn with_temp_dir<F, T>(temp_dir: &TempDir, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _lock = DIR_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let result = f();
        std::env::set_current_dir(original_dir).unwrap();
        result
    }

    // ========== merge_and_deduplicate tests ==========

    #[test]
    fn test_merge_empty_lists() {
        let result = merge_and_deduplicate(vec![], vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_existing_only() {
        let existing = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];
        let result = merge_and_deduplicate(existing, vec![]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_new_only() {
        let new = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];
        let result = merge_and_deduplicate(vec![], new);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_no_duplicates() {
        let existing = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let new = vec![make_entry("nota", "2025-01-16", "ITALIANO", "Task 2")];
        let result = merge_and_deduplicate(existing, new);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_removes_duplicates() {
        let existing = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let new = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];
        let result = merge_and_deduplicate(existing, new);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_keeps_existing_over_new() {
        let existing = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let new = vec![make_entry("nota", "2025-01-15", "MATEMATICA", "Task 1")];
        let result = merge_and_deduplicate(existing, new);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entry_type, "compiti");
    }

    #[test]
    fn test_merge_sorts_by_date() {
        let existing = vec![make_entry("compiti", "2025-01-20", "MATEMATICA", "Task 3")];
        let new = vec![
            make_entry("nota", "2025-01-10", "ITALIANO", "Task 1"),
            make_entry("compiti", "2025-01-15", "INGLESE", "Task 2"),
        ];
        let result = merge_and_deduplicate(existing, new);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].date, "2025-01-10");
        assert_eq!(result[1].date, "2025-01-15");
        assert_eq!(result[2].date, "2025-01-20");
    }

    #[test]
    fn test_merge_deduplicates_within_new() {
        let new = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
        ];
        let result = merge_and_deduplicate(vec![], new);
        assert_eq!(result.len(), 1);
    }

    // ========== load_existing_entries tests ==========

    #[test]
    fn test_load_existing_entries_file_not_exists() {
        let path = PathBuf::from("/nonexistent/path/homework.json");
        let result = load_existing_entries(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_existing_entries_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("homework.json");
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let json = serde_json::to_string_pretty(&entries).unwrap();
        std::fs::write(&json_path, json).unwrap();

        let loaded = load_existing_entries(&json_path).unwrap();
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn test_load_existing_entries_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("homework.json");
        std::fs::write(&json_path, "not valid json").unwrap();

        let result = load_existing_entries(&json_path);
        assert!(result.is_err());
    }

    // ========== save_json tests ==========

    #[test]
    fn test_save_json_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("homework.json");
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        save_json(&entries, &json_path).unwrap();
        assert!(json_path.exists());
    }

    #[test]
    fn test_save_json_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("homework.json");
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        save_json(&entries, &json_path).unwrap();
        let loaded = load_existing_entries(&json_path).unwrap();
        assert_eq!(entries, loaded);
    }

    // ========== find_all_exports tests ==========

    #[test]
    fn test_find_all_exports_no_data_dir() {
        let result = find_all_exports();
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_all_exports_with_export_files() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        std::fs::write(data_dir.join("export_20250115.xls"), "content1").unwrap();
        std::fs::write(data_dir.join("export_20250116.xlsx"), "content2").unwrap();
        std::fs::write(data_dir.join("other_file.xls"), "ignored").unwrap();

        let files = with_temp_dir(&temp_dir, || find_all_exports().unwrap());

        assert_eq!(files.len(), 2);
        assert!(files[0].to_string_lossy().contains("export_20250115"));
        assert!(files[1].to_string_lossy().contains("export_20250116"));
    }

    #[test]
    fn test_find_all_exports_empty_data_dir() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        let files = with_temp_dir(&temp_dir, || find_all_exports().unwrap());
        assert!(files.is_empty());
    }

    // ========== process_all_exports tests ==========

    fn create_test_excel_xml(path: &std::path::Path, entries: &[(&str, &str, &str, &str)]) {
        let mut rows = String::from(
            r#"<Row><Cell><Data ss:Type="String">tipo</Data></Cell><Cell><Data ss:Type="String">data_inizio</Data></Cell><Cell><Data ss:Type="String">materia</Data></Cell><Cell><Data ss:Type="String">nota</Data></Cell></Row>"#,
        );
        for (tipo, date, subject, task) in entries {
            rows.push_str(&format!(
                r#"<Row><Cell><Data ss:Type="String">{}</Data></Cell><Cell><Data ss:Type="String">{}</Data></Cell><Cell><Data ss:Type="String">{}</Data></Cell><Cell><Data ss:Type="String">{}</Data></Cell></Row>"#,
                tipo, date, subject, task
            ));
        }
        let xml = format!(
            r#"<?xml version="1.0"?><Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet"><Worksheet ss:Name="Table1"><Table>{}</Table></Worksheet></Workbook>"#,
            rows
        );
        std::fs::write(path, xml).unwrap();
    }

    #[test]
    fn test_process_all_exports_with_new_files() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        create_test_excel_xml(
            &data_dir.join("export_20250115.xls"),
            &[
                ("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
                ("nota", "2025-01-16", "ITALIANO", "Task 2"),
            ],
        );

        let output_path = temp_dir.path().to_path_buf();
        let result = with_temp_dir(&temp_dir, || process_all_exports(&output_path));

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(temp_dir.path().join("homework.json").exists());
    }

    #[test]
    fn test_process_all_exports_merges_with_existing() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        let existing = vec![make_entry("compiti", "2025-01-10", "INGLESE", "Existing")];
        save_json(&existing, &temp_dir.path().join("homework.json")).unwrap();

        create_test_excel_xml(
            &data_dir.join("export_20250115.xls"),
            &[("nota", "2025-01-15", "MATEMATICA", "New task")],
        );

        let output_path = temp_dir.path().to_path_buf();
        let result = with_temp_dir(&temp_dir, || process_all_exports(&output_path));

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].date, "2025-01-10");
        assert_eq!(entries[1].date, "2025-01-15");
    }

    #[test]
    fn test_process_all_exports_no_files_no_existing_data() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        let output_path = temp_dir.path().to_path_buf();
        let result = with_temp_dir(&temp_dir, || process_all_exports(&output_path));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No export files"));
    }

    #[test]
    fn test_process_all_exports_no_files_with_existing_data() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        let existing = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        save_json(&existing, &temp_dir.path().join("homework.json")).unwrap();

        let output_path = temp_dir.path().to_path_buf();
        let result = with_temp_dir(&temp_dir, || process_all_exports(&output_path));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_process_all_exports_handles_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        create_test_excel_xml(
            &data_dir.join("export_20250115.xls"),
            &[("compiti", "2025-01-15", "MATEMATICA", "Valid")],
        );
        std::fs::write(data_dir.join("export_20250116.xls"), "invalid xml").unwrap();

        let output_path = temp_dir.path().to_path_buf();
        let result = with_temp_dir(&temp_dir, || process_all_exports(&output_path));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
