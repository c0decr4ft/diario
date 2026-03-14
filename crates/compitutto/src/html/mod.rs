//! HTML rendering for Compitutto.
//!
//! Organised into submodules:
//!   - `assets`   — CSS and JavaScript constants
//!   - `calendar` — Calendar view (month grid + sidebar)
//!   - `settings` — Settings page

pub mod assets;
pub mod calendar;
pub mod settings;

pub use settings::render_settings_page;

use anyhow::Result;
use chrono::NaiveDate;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::types::HomeworkEntry;

use assets::{CSS, JAVASCRIPT};
use calendar::render_calendar;

/// Write a full HTML page to disk.
pub fn generate_html(entries: &[HomeworkEntry], path: &Path) -> Result<()> {
    let html = render_page(entries);
    fs::write(path, html.into_string())?;
    Ok(())
}

/// Render the main homework list page.
pub fn render_page(entries: &[HomeworkEntry]) -> Markup {
    // Group entries by date
    let mut by_date: BTreeMap<&str, Vec<&HomeworkEntry>> = BTreeMap::new();
    for entry in entries {
        by_date.entry(&entry.date).or_default().push(entry);
    }

    // Build an id -> entry lookup for linking lavoro items to their parent compiti
    let entry_by_id: std::collections::HashMap<&str, &HomeworkEntry> =
        entries.iter().map(|e| (e.id.as_str(), e)).collect();

    let total_count = entries.len();
    let completed_count = entries.iter().filter(|e| e.completed).count();

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Compitutto" }
                style { (PreEscaped(CSS)) }
            }
            body {
                div.container {
                    header.header {
                        div.header-left {
                            h1 { "Compitutto" }
                            div.stats {
                                span #"completed-count" { (completed_count) }
                                " / "
                                span #"total-count" { (total_count) }
                                " completed"
                            }
                        }
                        div.view-toggle {
                            button.view-btn.active #"list-view-btn" type="button" { "List" }
                            button.view-btn #"calendar-view-btn" type="button" { "Calendar" }
                            a.view-btn href="/settings" { "⚙ Settings" }
                        }
                    }
                    div.list-view #"list-view" {
                        @if entries.is_empty() {
                            div.empty-state {
                                p { "No homework entries found." }
                            }
                        } @else {
                            @for (date, items) in by_date.iter().rev() {
                                (render_date_group(date, items, &entry_by_id))
                            }
                        }
                    }
                    div.calendar-view.hidden #"calendar-view" {
                        (render_calendar(entries, &by_date))
                    }
                }

                // Floating add button
                button.add-entry-btn #"add-entry-btn" type="button" title="Add new entry" { "+" }

                // Delete confirmation dialog
                dialog #"delete-dialog" {
                    h3 { "Delete Entry" }
                    p #"delete-message" { "Are you sure you want to delete this entry?" }
                    div.dialog-note #"delete-children-note" style="display:none" {
                        p { "This entry has study sessions linked to it." }
                        p { "Type " strong { "delete all" } " to delete everything, or " strong { "keep" } " to delete only this entry:" }
                        input #"delete-confirm-input" type="text" placeholder="Type here...";
                    }
                    div.dialog-buttons {
                        button.btn-cancel #"delete-cancel" type="button" { "Cancel" }
                        button.btn-danger #"delete-confirm" type="button" { "Delete" }
                    }
                }

                // Position dialog for drag-drop
                dialog #"position-dialog" {
                    h3 { "Position" }
                    p { "Where should this entry be placed?" }
                    div.dialog-buttons {
                        button.btn-primary #"position-top" type="button" { "Add to Top" }
                        button.btn-primary #"position-bottom" type="button" { "Add to Bottom" }
                        button.btn-cancel #"position-cancel" type="button" { "Cancel" }
                    }
                }

                // Add entry dialog
                dialog #"add-entry-dialog" {
                    h3 { "Add New Entry" }
                    form #"add-entry-form" {
                        div.form-group {
                            label for="new-entry-date" { "Date" }
                            input #"new-entry-date" type="date" required;
                        }
                        div.form-group {
                            label for="new-entry-subject" { "Subject" }
                            select #"new-entry-subject" required {
                                option value="" disabled selected { "Select a subject..." }
                                option value="Arte e Immagine" { "Arte e Immagine" }
                                option value="Educazione Civica" { "Educazione Civica" }
                                option value="Geografia" { "Geografia" }
                                option value="Italiano" { "Italiano" }
                                option value="Lingua Inglese" { "Lingua Inglese" }
                                option value="Matematica" { "Matematica" }
                                option value="Musica" { "Musica" }
                                option value="Religione" { "Religione" }
                                option value="Scienze" { "Scienze" }
                                option value="Scienze Motorie" { "Scienze Motorie" }
                                option value="Storia" { "Storia" }
                                option value="Tecnologia" { "Tecnologia" }
                                option value="Tedesco" { "Tedesco" }
                            }
                        }
                        div.form-group {
                            label for="new-entry-type" { "Type" }
                            select #"new-entry-type" {
                                option value="compiti" { "Compiti" }
                                option value="nota" { "Nota" }
                                option value="verifica" { "Verifica" }
                                option value="interrogazione" { "Interrogazione" }
                                option value="studio" { "Studio" }
                            }
                        }
                        div.form-group {
                            label for="new-entry-task" { "Task" }
                            textarea #"new-entry-task" rows="3" placeholder="Task description..." required {}
                        }
                        div.dialog-buttons {
                            button.btn-cancel #"add-entry-cancel" type="button" { "Cancel" }
                            button.btn-primary type="submit" { "Add Entry" }
                        }
                    }
                }

                script { (PreEscaped(JAVASCRIPT)) }
            }
        }
    }
}

/// Render a single date group (header + all homework items for that date).
fn render_date_group(
    date: &str,
    items: &[&HomeworkEntry],
    entry_by_id: &std::collections::HashMap<&str, &HomeworkEntry>,
) -> Markup {
    let all_completed = items.iter().all(|item| item.completed);
    let group_class = if all_completed {
        "date-group collapsed"
    } else {
        "date-group"
    };
    html! {
        div class=(group_class) data-date=(date) id={"entry-group-" (date)} {
            div.date-header {
                span.collapse-indicator { "▼" }
                "📅 "
                (NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .map(|d| format!("{} {}", d.format("%A"), date))
                    .unwrap_or_else(|_| date.to_string()))
            }
            div.date-items {
                @for item in items.iter() {
                    @let entry_id = &item.id;
                    @let stable_id = item.stable_id();
                    @let is_generated = item.is_generated();
                    @let is_orphaned = item.is_orphaned();
                    @let is_completed = item.completed;
                    @let is_lavoro = item.entry_type == "lavoro";
                    @let is_compiti = item.entry_type == "compiti";
                    @let parent_info = if is_lavoro {
                        item.parent_id.as_deref()
                            .and_then(|pid| entry_by_id.get(pid))
                            .map(|p| (p.id.clone(), p.date.clone()))
                    } else {
                        None
                    };
                    // For compiti: find the lavoro child so we can sync completion
                    @let lavoro_child_id = if is_compiti {
                        entry_by_id.values()
                            .find(|e| e.entry_type == "lavoro"
                                && e.parent_id.as_deref() == Some(entry_id.as_str()))
                            .map(|e| e.id.clone())
                    } else {
                        None
                    };
                    @let item_class = {
                        let mut cls = "homework-item".to_string();
                        if is_completed { cls.push_str(" completed"); }
                        if is_lavoro   { cls.push_str(" lavoro-item"); }
                        if is_compiti  { cls.push_str(" compiti-due-item"); }
                        cls
                    };
                    div
                        class=(item_class)
                        data-entry-id=(entry_id)
                        data-stable-id=(stable_id)
                        data-generated=[is_generated.then_some("true")]
                        data-orphaned=[is_orphaned.then_some("true")]
                        data-parent-id=[parent_info.as_ref().map(|(id, _)| id.as_str())]
                        data-lavoro-id=[lavoro_child_id.as_deref()]
                        draggable="true"
                    {
                        input.homework-checkbox
                            type="checkbox"
                            id={"entry-" (stable_id)}
                            data-entry-id=(entry_id)
                            checked[is_completed];
                        div.homework-content {
                            div.homework-subject {
                                (item.subject)
                                @if !item.entry_type.is_empty() {
                                    @let type_lower = item.entry_type.to_lowercase();
                                    span.homework-type data-type=(type_lower) {
                                        @if is_lavoro { "✏️ Do it" }
                                        @else if is_compiti { "📋 Due" }
                                        @else { (item.entry_type) }
                                    }
                                }
                                @if is_generated {
                                    span.auto-badge { "auto" }
                                }
                                @if is_orphaned {
                                    span.orphan-badge { "orphaned" }
                                }
                            }
                            div.homework-task { (item.task) }
                            @if let Some((parent_id, parent_date)) = parent_info {
                                div.due-link {
                                    "📅 Due: "
                                    a href={"#entry-group-" (parent_date)} data-scroll-to=(parent_id) {
                                        (NaiveDate::parse_from_str(&parent_date, "%Y-%m-%d")
                                            .map(|d| format!("{} {}", d.format("%A"), parent_date))
                                            .unwrap_or(parent_date))
                                    }
                                }
                            }
                        }
                        button.delete-btn type="button" data-entry-id=(entry_id) title="Delete entry" { "🗑" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::calendar::{entries_to_json, month_name, render_calendar};
    use super::*;
    use tempfile::TempDir;

    fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry {
        HomeworkEntry::new(
            entry_type.to_string(),
            date.to_string(),
            subject.to_string(),
            task.to_string(),
        )
    }

    // ========== render_page tests ==========

    #[test]
    fn test_render_page_empty_entries() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("Compitutto"));
        assert!(html.contains("No homework entries found"));
        assert!(html.contains("0"));
    }

    #[test]
    fn test_render_page_single_entry() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "Matematica",
            "Pag. 100 es. 1-5",
        )];
        let html = render_page(&entries).into_string();
        assert!(html.contains("Matematica"));
        assert!(html.contains("Pag. 100 es. 1-5"));
        assert!(html.contains("2025-01-15"));
        assert!(html.contains("compiti"));
        assert!(html.contains(">1<"));
    }

    #[test]
    fn test_render_page_multiple_entries_same_date() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-15", "Italiano", "Task 2"),
        ];
        let html = render_page(&entries).into_string();
        assert!(html.contains("Matematica"));
        assert!(html.contains("Italiano"));
        assert_eq!(html.matches(r#"class="date-group""#).count(), 1);
    }

    #[test]
    fn test_render_page_multiple_dates() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-16", "Italiano", "Task 2"),
            make_entry("compiti", "2025-01-17", "INGLESE", "Task 3"),
        ];
        let html = render_page(&entries).into_string();
        assert!(html.contains("2025-01-15"));
        assert!(html.contains("2025-01-16"));
        assert!(html.contains("2025-01-17"));
        assert!(html.contains(">3<"));
    }

    #[test]
    fn test_render_page_has_required_elements() {
        let entries = vec![make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let html = render_page(&entries).into_string();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\""));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("charset=\"UTF-8\""));
        assert!(html.contains("viewport"));
        assert!(html.contains("<title>Compitutto</title>"));
        assert!(html.contains("<style>"));
        assert!(html.contains("<script>"));
    }

    #[test]
    fn test_render_page_has_checkboxes() {
        let entries = vec![make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let html = render_page(&entries).into_string();
        assert!(html.contains("homework-checkbox"));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("data-entry-id"));
    }

    #[test]
    fn test_render_page_escapes_html_in_task() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "Matematica",
            "<script>alert('xss')</script>",
        )];
        let html = render_page(&entries).into_string();
        assert!(!html.contains("<script>alert('xss')</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_render_page_handles_special_characters() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "Matematica",
            "Esercizi con àèìòù & simboli",
        )];
        let html = render_page(&entries).into_string();
        assert!(html.contains("Esercizi con àèìòù &amp; simboli"));
    }

    #[test]
    fn test_render_page_empty_entry_type() {
        let entries = vec![make_entry("", "2025-01-15", "Matematica", "Task 1")];
        let html = render_page(&entries).into_string();
        assert!(html.contains("Matematica"));
        assert!(html.contains("Task 1"));
        assert!(!html.contains("<span class=\"homework-type\">"));
    }

    #[test]
    fn test_render_page_entry_type_badge() {
        let entries = vec![make_entry("nota", "2025-01-15", "Matematica", "Task 1")];
        let html = render_page(&entries).into_string();
        assert!(html.contains("homework-type"));
        assert!(html.contains("nota"));
    }

    #[test]
    fn test_render_page_groups_entries_by_date() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-15", "Italiano", "Task 2"),
            make_entry("compiti", "2025-01-16", "INGLESE", "Task 3"),
        ];
        let html = render_page(&entries).into_string();
        assert!(html.matches("date-group").count() >= 2);
    }

    #[test]
    fn test_render_page_css_included() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("font-family"));
        assert!(html.contains("background"));
        assert!(html.contains(".homework-item"));
        assert!(html.contains(".date-header"));
    }

    #[test]
    fn test_render_page_javascript_included() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("homework-checkbox"));
        assert!(html.contains("/api/entries"));
        assert!(html.contains("dragstart"));
        assert!(html.contains("delete-dialog"));
    }

    // ========== render_date_group tests ==========

    #[test]
    fn test_render_date_group_basic() {
        let entries = [
            make_entry("compiti", "2025-01-15", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-15", "Italiano", "Task 2"),
        ];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        assert!(html.contains("date-group"));
        assert!(html.contains("2025-01-15"));
        assert!(html.contains("Matematica"));
        assert!(html.contains("Italiano"));
    }

    #[test]
    fn test_render_date_group_entry_ids_are_stable() {
        let entries = [
            make_entry("compiti", "2025-01-15", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-15", "Italiano", "Task 2"),
        ];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        let entry1_id = entries[0].stable_id();
        let entry2_id = entries[1].stable_id();
        assert!(html.contains(&format!("entry-{}", entry1_id)));
        assert!(html.contains(&format!("entry-{}", entry2_id)));
        assert_eq!(entry1_id.len(), 8);
        assert_eq!(entry2_id.len(), 8);
    }

    #[test]
    fn test_render_date_group_ids_independent_of_position() {
        let entry1 = make_entry("compiti", "2025-01-15", "Matematica", "Task 1");
        let entry2 = make_entry("nota", "2025-01-16", "Italiano", "Task 2");
        let refs1: Vec<&HomeworkEntry> = vec![&entry1, &entry2];
        let html1 = render_date_group("2025-01-15", &refs1, &Default::default()).into_string();
        let refs2: Vec<&HomeworkEntry> = vec![&entry2, &entry1];
        let html2 = render_date_group("2025-01-15", &refs2, &Default::default()).into_string();
        let entry1_id = entry1.stable_id();
        assert!(html1.contains(&format!("entry-{}", entry1_id)));
        assert!(html2.contains(&format!("entry-{}", entry1_id)));
    }

    // ========== generate_html tests ==========

    #[test]
    fn test_generate_html_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");
        let entries = vec![make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        generate_html(&entries, &html_path).unwrap();
        assert!(html_path.exists());
    }

    #[test]
    fn test_generate_html_content() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");
        let entries = vec![make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        generate_html(&entries, &html_path).unwrap();
        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("Matematica"));
        assert!(content.contains("Task 1"));
    }

    #[test]
    fn test_generate_html_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");
        std::fs::write(&html_path, "old content").unwrap();
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "Matematica",
            "New task",
        )];
        generate_html(&entries, &html_path).unwrap();
        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(!content.contains("old content"));
        assert!(content.contains("New task"));
    }

    #[test]
    fn test_generate_html_empty_entries() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");
        generate_html(&[], &html_path).unwrap();
        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("No homework entries found"));
    }

    // ========== Edge cases ==========

    #[test]
    fn test_render_page_long_task_text() {
        let long_task = "A".repeat(1000);
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "Matematica",
            &long_task,
        )];
        let html = render_page(&entries).into_string();
        assert!(html.contains(&long_task));
    }

    #[test]
    fn test_render_page_unicode_content() {
        let entries = vec![make_entry("compiti", "2025-01-15", "日本語", "任务描述 🎉")];
        let html = render_page(&entries).into_string();
        assert!(html.contains("日本語"));
        assert!(html.contains("任务描述"));
    }

    #[test]
    fn test_render_page_many_entries() {
        let entries: Vec<HomeworkEntry> = (0..100)
            .map(|i| {
                make_entry(
                    "compiti",
                    &format!("2025-01-{:02}", (i % 28) + 1),
                    &format!("SUBJECT_{}", i),
                    &format!("Task {}", i),
                )
            })
            .collect();
        let html = render_page(&entries).into_string();
        assert!(html.contains(">100<"));
        assert!(html.contains("SUBJECT_0"));
        assert!(html.contains("SUBJECT_99"));
    }

    // ========== Dialog/UI element tests ==========

    #[test]
    fn test_render_page_has_add_button() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("add-entry-btn"));
        assert!(html.contains(r#"id="add-entry-btn""#));
    }

    #[test]
    fn test_render_page_has_delete_dialog() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("delete-dialog"));
        assert!(html.contains("delete-confirm"));
        assert!(html.contains("delete-cancel"));
    }

    #[test]
    fn test_render_page_has_position_dialog() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("position-dialog"));
        assert!(html.contains("position-top"));
        assert!(html.contains("position-bottom"));
    }

    #[test]
    fn test_render_page_has_add_entry_dialog() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("add-entry-dialog"));
        assert!(html.contains("add-entry-form"));
        assert!(html.contains("new-entry-date"));
        assert!(html.contains("new-entry-subject"));
        assert!(html.contains("new-entry-type"));
        assert!(html.contains("new-entry-task"));
    }

    #[test]
    fn test_render_date_group_has_delete_buttons() {
        let entries = [make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        assert!(html.contains("delete-btn"));
        assert!(html.contains(r#"title="Delete entry""#));
    }

    #[test]
    fn test_render_date_group_draggable() {
        let entries = [make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        assert!(html.contains(r#"draggable="true""#));
    }

    #[test]
    fn test_render_date_group_data_date() {
        let entries = [make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        assert!(html.contains(r#"data-date="2025-01-15""#));
    }

    #[test]
    fn test_render_date_group_generated_entry() {
        let mut entry = make_entry("studio", "2025-01-15", "Matematica", "Study for: Test");
        entry.parent_id = Some("parent123".to_string());
        let refs: Vec<&HomeworkEntry> = vec![&entry];
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        assert!(html.contains(r#"data-generated="true""#));
        assert!(html.contains("auto-badge"));
        assert!(html.contains("auto"));
    }

    #[test]
    fn test_render_date_group_orphaned_entry() {
        let entry = make_entry("studio", "2025-01-15", "Matematica", "Study for: Test");
        let refs: Vec<&HomeworkEntry> = vec![&entry];
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        assert!(html.contains(r#"data-orphaned="true""#));
        assert!(html.contains("orphan-badge"));
        assert!(html.contains("orphaned"));
    }

    #[test]
    fn test_render_date_group_completed_entry() {
        let mut entry = make_entry("compiti", "2025-01-15", "Matematica", "Task 1");
        entry.completed = true;
        let refs: Vec<&HomeworkEntry> = vec![&entry];
        let html = render_date_group("2025-01-15", &refs, &Default::default()).into_string();
        assert!(html.contains("homework-item") && html.contains("completed"));
        assert!(html.contains("checked"));
    }

    // ========== CSS/JS content tests ==========

    #[test]
    fn test_render_page_css_has_generated_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("[data-generated=\"true\"]"));
        assert!(html.contains("auto-badge"));
    }

    #[test]
    fn test_render_page_css_has_orphaned_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("[data-orphaned=\"true\"]"));
        assert!(html.contains("orphan-badge"));
    }

    #[test]
    fn test_render_page_css_has_drag_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains(".dragging"));
        assert!(html.contains(".drag-over"));
    }

    #[test]
    fn test_render_page_css_has_delete_button_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains(".delete-btn"));
    }

    // ========== View toggle tests ==========

    #[test]
    fn test_render_page_has_view_toggle() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("view-toggle"));
        assert!(html.contains("list-view-btn"));
        assert!(html.contains("calendar-view-btn"));
    }

    #[test]
    fn test_render_page_has_list_view() {
        let entries = vec![make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let html = render_page(&entries).into_string();
        assert!(html.contains(r#"id="list-view""#));
        assert!(html.contains(r#"class="list-view""#));
    }

    #[test]
    fn test_render_page_has_calendar_view() {
        let entries = vec![make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let html = render_page(&entries).into_string();
        assert!(html.contains(r#"id="calendar-view""#));
        assert!(html.contains("calendar-layout"));
        assert!(html.contains("calendar-main"));
        assert!(html.contains("calendar-sidebar"));
    }

    #[test]
    fn test_render_page_calendar_view_hidden_by_default() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains(r#"class="calendar-view hidden""#));
    }

    #[test]
    fn test_render_page_has_calendar_navigation() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("cal-prev"));
        assert!(html.contains("cal-next"));
        assert!(html.contains("cal-month-year"));
    }

    #[test]
    fn test_render_page_has_calendar_day_headers() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("Mon"));
        assert!(html.contains("Tue"));
        assert!(html.contains("Sat"));
        assert!(html.contains("Sun"));
    }

    #[test]
    fn test_render_page_calendar_contains_entries_data() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-16", "Italiano", "Task 2"),
        ];
        let html = render_page(&entries).into_string();
        assert!(html.contains("data-entries="));
        assert!(html.contains("Matematica"));
        assert!(html.contains("Italiano"));
    }

    #[test]
    fn test_render_page_css_has_view_toggle_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains(".view-toggle"));
        assert!(html.contains(".view-btn"));
        assert!(html.contains(".view-btn.active"));
    }

    #[test]
    fn test_render_page_css_has_calendar_styling() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains(".calendar-view"));
        assert!(html.contains(".calendar-main"));
        assert!(html.contains(".cal-day"));
        assert!(html.contains(".cal-entry"));
    }

    #[test]
    fn test_render_page_javascript_has_view_toggle() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("showListView"));
        assert!(html.contains("showCalendarView"));
        assert!(html.contains("localStorage"));
    }

    #[test]
    fn test_render_page_javascript_has_calendar_rendering() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("renderCalendar"));
        assert!(html.contains("renderCalendarDay"));
        assert!(html.contains("calPrev"));
        assert!(html.contains("calNext"));
    }

    // ========== Reverse chronological order ==========

    #[test]
    fn test_render_page_dates_in_reverse_chronological_order() {
        let entries = vec![
            make_entry("compiti", "2025-01-10", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-15", "Italiano", "Task 2"),
            make_entry("compiti", "2025-01-20", "INGLESE", "Task 3"),
        ];
        let html = render_page(&entries).into_string();
        let pos_10 = html.find("2025-01-10").unwrap();
        let pos_15 = html.find("2025-01-15").unwrap();
        let pos_20 = html.find("2025-01-20").unwrap();
        assert!(
            pos_20 < pos_15,
            "2025-01-20 should appear before 2025-01-15"
        );
        assert!(
            pos_15 < pos_10,
            "2025-01-15 should appear before 2025-01-10"
        );
    }

    // ========== Calendar helper tests ==========

    #[test]
    fn test_month_name() {
        assert_eq!(month_name(1), "January");
        assert_eq!(month_name(6), "June");
        assert_eq!(month_name(12), "December");
        assert_eq!(month_name(0), "Unknown");
        assert_eq!(month_name(13), "Unknown");
    }

    #[test]
    fn test_entries_to_json() {
        let entries = [
            make_entry("compiti", "2025-01-15", "Matematica", "Task 1"),
            make_entry("nota", "2025-01-15", "Italiano", "Task 2"),
        ];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let mut by_date: BTreeMap<&str, Vec<&HomeworkEntry>> = BTreeMap::new();
        by_date.insert("2025-01-15", refs);
        let json = entries_to_json(&by_date);
        assert!(json.contains("2025-01-15"));
        assert!(json.contains("Matematica"));
        assert!(json.contains("Italiano"));
    }

    #[test]
    fn test_entries_to_json_empty() {
        let by_date: BTreeMap<&str, Vec<&HomeworkEntry>> = BTreeMap::new();
        assert_eq!(entries_to_json(&by_date), "{}");
    }

    #[test]
    fn test_render_calendar_basic() {
        let entries = vec![make_entry("compiti", "2025-01-15", "Matematica", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let mut by_date: BTreeMap<&str, Vec<&HomeworkEntry>> = BTreeMap::new();
        by_date.insert("2025-01-15", refs);
        let html = render_calendar(&entries, &by_date).into_string();
        assert!(html.contains("calendar-layout"));
        assert!(html.contains("calendar-main"));
        assert!(html.contains("calendar-header"));
        assert!(html.contains("calendar-grid"));
        assert!(html.contains("calendar-days"));
        assert!(html.contains("calendar-sidebar"));
    }

    #[test]
    fn test_render_calendar_month_from_entries() {
        let entries = vec![make_entry("compiti", "2025-03-15", "Matematica", "Task 1")];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let mut by_date: BTreeMap<&str, Vec<&HomeworkEntry>> = BTreeMap::new();
        by_date.insert("2025-03-15", refs);
        let html = render_calendar(&entries, &by_date).into_string();
        assert!(html.contains("March"));
        assert!(html.contains("2025"));
    }

    // ========== Layout tests ==========

    #[test]
    fn test_render_page_has_header_structure() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains("header"));
        assert!(html.contains("header-left"));
    }

    #[test]
    fn test_render_page_css_has_full_width_container() {
        let entries: Vec<HomeworkEntry> = vec![];
        let html = render_page(&entries).into_string();
        assert!(html.contains(".container"));
        assert!(html.contains("width: 100%"));
    }
}
