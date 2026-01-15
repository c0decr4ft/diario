use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::parser;
use crate::types::HomeworkEntry;

/// Keywords that indicate a test/quiz (case-insensitive)
const TEST_KEYWORDS: &[&str] = &["verifica", "prova", "test", "interrogazione"];

/// Check if an entry is a test or quiz based on keywords in the task
pub fn is_test_or_quiz(entry: &HomeworkEntry) -> bool {
    let task_lower = entry.task.to_lowercase();
    TEST_KEYWORDS.iter().any(|kw| task_lower.contains(kw))
}

/// Generate study sessions for a test entry
///
/// Creates up to 4 study session entries on the days leading up to the test.
/// Each study session links back to its parent test via `parent_id`.
pub fn generate_study_sessions(test: &HomeworkEntry, today: NaiveDate) -> Vec<HomeworkEntry> {
    let test_date = match NaiveDate::parse_from_str(&test.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let days_until = (test_date - today).num_days();

    // Only generate for future tests (at least 2 days away to have study time)
    if days_until < 2 {
        return Vec::new();
    }

    // Generate up to 4 days before, but only for future dates
    let days_to_generate = std::cmp::min(4, days_until - 1) as usize;

    // Truncate task to 100 chars for study session text
    let truncated_task = if test.task.len() > 100 {
        format!("{}...", &test.task[..100])
    } else {
        test.task.clone()
    };

    let now = chrono::Utc::now().to_rfc3339();

    (1..=days_to_generate)
        .map(|days_before| {
            let study_date = test_date - chrono::Duration::days(days_before as i64);
            let date_str = study_date.format("%Y-%m-%d").to_string();
            let task_str = format!("Study for: {}", truncated_task);
            let id = compute_study_session_id(&test.id, days_before);
            let source_id = HomeworkEntry::generate_source_id(&date_str, &test.subject, &task_str);
            HomeworkEntry {
                id,
                source_id: Some(source_id),
                entry_type: "studio".to_string(),
                date: date_str,
                subject: test.subject.clone(),
                task: task_str,
                completed: false,
                position: 0,
                parent_id: Some(test.id.clone()),
                created_at: now.clone(),
                updated_at: now.clone(),
            }
        })
        .collect()
}

/// Compute a deterministic ID for a study session based on parent ID and days before
fn compute_study_session_id(parent_id: &str, days_before: usize) -> String {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    parent_id.hash(&mut hasher);
    days_before.hash(&mut hasher);
    "study".hash(&mut hasher);
    format!("study_{:016x}", hasher.finish())
}

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
        debug!("No export files found, using existing data");
        return Ok(existing_entries);
    }

    let mut new_entries: Vec<HomeworkEntry> = Vec::new();
    for file in &files {
        debug!(file = %file.display(), "Processing export file");
        match parser::parse_excel_xml(file) {
            Ok(entries) => {
                debug!(count = entries.len(), "Found entries");
                new_entries.extend(entries);
            }
            Err(e) => {
                warn!(file = %file.display(), error = %e, "Failed to parse export file");
            }
        }
    }

    // Merge and deduplicate
    let all_entries = merge_and_deduplicate(existing_entries, new_entries);
    let new_count = all_entries.len().saturating_sub(existing_count);

    info!(
        total = all_entries.len(),
        new = new_count,
        "Entries processed"
    );

    // Save updated JSON
    save_json(&all_entries, &json_path)?;
    debug!(path = %json_path.display(), "Data saved");

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

    debug!(count = entries.len(), "Loaded existing entries");
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
    use chrono::NaiveDate;
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

    // ========== is_test_or_quiz tests ==========

    #[test]
    fn test_is_test_verifica() {
        let entry = make_entry("compiti", "2025-01-20", "MATEMATICA", "Verifica sui limiti");
        assert!(is_test_or_quiz(&entry));
    }

    #[test]
    fn test_is_test_prova() {
        let entry = make_entry("nota", "2025-01-20", "ITALIANO", "Prova di italiano");
        assert!(is_test_or_quiz(&entry));
    }

    #[test]
    fn test_is_test_interrogazione() {
        let entry = make_entry("compiti", "2025-01-20", "STORIA", "Interrogazione cap. 5");
        assert!(is_test_or_quiz(&entry));
    }

    #[test]
    fn test_is_test_english_test() {
        let entry = make_entry("compiti", "2025-01-20", "INGLESE", "Test unit 3");
        assert!(is_test_or_quiz(&entry));
    }

    #[test]
    fn test_is_test_case_insensitive() {
        let entry = make_entry("compiti", "2025-01-20", "MATEMATICA", "VERIFICA sui limiti");
        assert!(is_test_or_quiz(&entry));
    }

    #[test]
    fn test_is_not_test_regular_homework() {
        let entry = make_entry("compiti", "2025-01-20", "MATEMATICA", "Esercizi pag. 50");
        assert!(!is_test_or_quiz(&entry));
    }

    // ========== generate_study_sessions tests ==========

    #[test]
    fn test_generate_study_sessions_future_test() {
        let test = make_entry("compiti", "2025-01-20", "MATEMATICA", "Verifica sui limiti");
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions = generate_study_sessions(&test, today);

        // 5 days away, should generate 4 study sessions
        assert_eq!(sessions.len(), 4);

        // Check dates are correct (1, 2, 3, 4 days before the test)
        assert_eq!(sessions[0].date, "2025-01-19");
        assert_eq!(sessions[1].date, "2025-01-18");
        assert_eq!(sessions[2].date, "2025-01-17");
        assert_eq!(sessions[3].date, "2025-01-16");

        // Check all have correct parent_id
        for session in &sessions {
            assert_eq!(session.parent_id, Some(test.id.clone()));
            assert_eq!(session.entry_type, "studio");
            assert_eq!(session.subject, "MATEMATICA");
            assert!(session.task.starts_with("Study for: "));
        }
    }

    #[test]
    fn test_generate_study_sessions_close_test() {
        let test = make_entry("compiti", "2025-01-17", "MATEMATICA", "Verifica");
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions = generate_study_sessions(&test, today);

        // 2 days away, should generate 1 study session (day before)
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].date, "2025-01-16");
    }

    #[test]
    fn test_generate_study_sessions_tomorrow_test() {
        let test = make_entry("compiti", "2025-01-16", "MATEMATICA", "Verifica");
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions = generate_study_sessions(&test, today);

        // Only 1 day away, no time for study sessions
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_generate_study_sessions_past_test() {
        let test = make_entry("compiti", "2025-01-10", "MATEMATICA", "Verifica");
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions = generate_study_sessions(&test, today);

        // Test is in the past
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_generate_study_sessions_long_task_truncated() {
        let long_task = format!("Verifica su: {}", "a".repeat(200));
        let test = make_entry("compiti", "2025-01-20", "MATEMATICA", &long_task);
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions = generate_study_sessions(&test, today);

        // Task should be truncated with "..."
        assert!(sessions[0].task.len() < 150);
        assert!(sessions[0].task.ends_with("..."));
    }

    #[test]
    fn test_generate_study_sessions_deterministic_ids() {
        let test = make_entry("compiti", "2025-01-20", "MATEMATICA", "Verifica");
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions1 = generate_study_sessions(&test, today);
        let sessions2 = generate_study_sessions(&test, today);

        // IDs should be the same for the same test
        for (s1, s2) in sessions1.iter().zip(sessions2.iter()) {
            assert_eq!(s1.id, s2.id);
        }
    }

    #[test]
    fn test_generate_study_sessions_invalid_date() {
        let test = make_entry("compiti", "invalid-date", "MATEMATICA", "Verifica");
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions = generate_study_sessions(&test, today);

        // Should return empty for invalid date
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_study_session_is_generated() {
        let test = make_entry("compiti", "2025-01-20", "MATEMATICA", "Verifica");
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let sessions = generate_study_sessions(&test, today);

        // All study sessions should be marked as generated
        for session in &sessions {
            assert!(session.is_generated());
        }

        // Original test is not generated
        assert!(!test.is_generated());
    }
}
