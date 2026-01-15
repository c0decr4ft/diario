use anyhow::Result;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::types::HomeworkEntry;

/// Generate HTML file from homework entries
pub fn generate_html(entries: &[HomeworkEntry], path: &Path) -> Result<()> {
    let html = render_page(entries);
    fs::write(path, html.into_string())?;
    Ok(())
}

pub fn render_page(entries: &[HomeworkEntry]) -> Markup {
    // Group entries by date
    let mut by_date: BTreeMap<&str, Vec<&HomeworkEntry>> = BTreeMap::new();
    for entry in entries {
        by_date.entry(&entry.date).or_default().push(entry);
    }

    let total_count = entries.len();

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
                    h1 { "Compitutto" }
                    div.stats {
                        span #"total-count" { (total_count) }
                        " entries"
                    }
                    div.homework-list #"homework-list" {
                        @if entries.is_empty() {
                            div.empty-state {
                                p { "No homework entries found." }
                            }
                        } @else {
                            @for (idx, (date, items)) in by_date.iter().enumerate() {
                                (render_date_group(date, items, idx))
                            }
                        }
                    }
                }
                script { (PreEscaped(JAVASCRIPT)) }
            }
        }
    }
}

fn render_date_group(date: &str, items: &[&HomeworkEntry], group_idx: usize) -> Markup {
    html! {
        div.date-group {
            div.date-header { "📅 " (date) }
            @for (item_idx, item) in items.iter().enumerate() {
                @let entry_id = group_idx * 100 + item_idx + 1;
                div.homework-item data-entry-id=(entry_id) {
                    input.homework-checkbox type="checkbox" id={"entry-" (entry_id)} data-entry-id=(entry_id);
                    div.homework-content {
                        div.homework-subject {
                            (item.subject)
                            @if !item.entry_type.is_empty() {
                                span.homework-type { (item.entry_type) }
                            }
                        }
                        div.homework-task { (item.task) }
                    }
                }
            }
        }
    }
}

const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;700;900&display=swap');

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #0a0a0a;
    color: #fff;
    min-height: 100vh;
    padding: 0;
    line-height: 1.4;
    overflow-x: hidden;
}

body::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: 
        repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(255,255,255,0.03) 2px, rgba(255,255,255,0.03) 4px),
        radial-gradient(circle at 20% 50%, rgba(255,0,150,0.1) 0%, transparent 50%),
        radial-gradient(circle at 80% 80%, rgba(0,255,255,0.1) 0%, transparent 50%);
    pointer-events: none;
    z-index: 0;
}

.container {
    max-width: 1000px;
    margin: 0 auto;
    padding: 40px 24px 60px;
    position: relative;
    z-index: 1;
}

h1 {
    color: #fff;
    font-weight: 900;
    font-size: 4.5em;
    letter-spacing: -0.03em;
    margin-bottom: 4px;
    text-transform: uppercase;
    text-shadow: 
        0 0 10px rgba(255,0,150,0.5),
        0 0 20px rgba(0,255,255,0.3),
        4px 4px 0 #ff0096,
        -2px -2px 0 #00ffff;
    transform: rotate(-1deg);
    animation: glitch 3s infinite;
}

@keyframes glitch {
    0%, 100% { transform: rotate(-1deg) translate(0, 0); }
    25% { transform: rotate(-0.5deg) translate(-1px, 1px); }
    50% { transform: rotate(-1.5deg) translate(1px, -1px); }
    75% { transform: rotate(-0.8deg) translate(-1px, -1px); }
}

.stats {
    color: #888;
    font-size: 0.85em;
    font-weight: 700;
    margin-bottom: 50px;
    padding-top: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
}

.homework-list {
    display: grid;
    gap: 50px;
}

.date-group {
    border-left: 4px solid;
    border-image: linear-gradient(180deg, #ff0096, #00ffff) 1;
    padding-left: 28px;
    margin-left: 4px;
    position: relative;
}

.date-group::before {
    content: '';
    position: absolute;
    left: -2px;
    top: 0;
    width: 8px;
    height: 8px;
    background: #00ffff;
    box-shadow: 0 0 10px #00ffff;
    border-radius: 50%;
}

.date-header {
    color: #fff;
    font-weight: 900;
    font-size: 1.1em;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    margin-bottom: 28px;
    padding-top: 4px;
    text-shadow: 0 0 8px rgba(0,255,255,0.6);
}

.homework-item {
    display: flex;
    align-items: flex-start;
    gap: 20px;
    padding: 20px;
    margin-bottom: 16px;
    background: rgba(255,255,255,0.03);
    border: 1px solid rgba(255,255,255,0.1);
    transition: all 0.2s;
    position: relative;
}

.homework-item::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    width: 3px;
    height: 100%;
    background: linear-gradient(180deg, #ff0096, #00ffff);
    opacity: 0;
    transition: opacity 0.2s;
}

.homework-item:hover {
    background: rgba(255,255,255,0.05);
    border-color: rgba(255,0,150,0.4);
    transform: translateX(4px);
}

.homework-item:hover::before {
    opacity: 1;
}

.homework-item:last-child {
    margin-bottom: 0;
}

.homework-item.completed {
    opacity: 0.3;
    filter: grayscale(1);
}

.homework-item.completed .homework-task {
    text-decoration: line-through;
}

.homework-checkbox {
    width: 24px;
    height: 24px;
    min-width: 24px;
    cursor: pointer;
    margin-top: 2px;
    accent-color: #ff0096;
    filter: drop-shadow(0 0 4px rgba(255,0,150,0.6));
}

.homework-content {
    flex: 1;
}

.homework-subject {
    color: #fff;
    font-weight: 700;
    font-size: 1.1em;
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.homework-type {
    display: inline-block;
    background: linear-gradient(135deg, #ff0096, #00ffff);
    color: #000;
    font-size: 0.65em;
    padding: 4px 10px;
    margin-left: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    font-weight: 900;
    border: 1px solid #fff;
    box-shadow: 0 0 8px rgba(255,0,150,0.5);
}

.homework-task {
    color: #ccc;
    line-height: 1.6;
    font-size: 0.95em;
    margin-top: 4px;
}

.empty-state {
    padding: 60px 20px;
    text-align: center;
    color: #666;
    font-size: 0.9em;
}

@media (max-width: 768px) {
    h1 {
        font-size: 3em;
    }
    
    .container {
        padding: 30px 16px 40px;
    }
}
"#;

const JAVASCRIPT: &str = r#"
// Load saved checkbox states from localStorage
function loadCheckboxStates() {
    const saved = localStorage.getItem('homework-checkboxes');
    if (saved) {
        const states = JSON.parse(saved);
        Object.keys(states).forEach(entryId => {
            const checkbox = document.getElementById(`entry-${entryId}`);
            const item = document.querySelector(`[data-entry-id="${entryId}"]`);
            if (checkbox && states[entryId]) {
                checkbox.checked = true;
                if (item) item.classList.add('completed');
            }
        });
    }
}

// Save checkbox states to localStorage
function saveCheckboxState(entryId, checked) {
    const saved = localStorage.getItem('homework-checkboxes') || '{}';
    const states = JSON.parse(saved);
    states[entryId] = checked;
    localStorage.setItem('homework-checkboxes', JSON.stringify(states));
}

// Add event listeners to all checkboxes
document.querySelectorAll('.homework-checkbox').forEach(checkbox => {
    checkbox.addEventListener('change', function() {
        const entryId = this.getAttribute('data-entry-id');
        const item = document.querySelector(`[data-entry-id="${entryId}"]`);
        
        if (this.checked) {
            item.classList.add('completed');
        } else {
            item.classList.remove('completed');
        }
        
        saveCheckboxState(entryId, this.checked);
    });
});

// Load states on page load
loadCheckboxStates();
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a HomeworkEntry
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
        assert!(html.contains("0")); // Total count
    }

    #[test]
    fn test_render_page_single_entry() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            "Pag. 100 es. 1-5",
        )];
        let html = render_page(&entries).into_string();

        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("Pag. 100 es. 1-5"));
        assert!(html.contains("2025-01-15"));
        assert!(html.contains("compiti"));
        assert!(html.contains(">1<")); // Total count: 1
    }

    #[test]
    fn test_render_page_multiple_entries_same_date() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
        ];
        let html = render_page(&entries).into_string();

        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("ITALIANO"));
        assert!(html.contains("Task 1"));
        assert!(html.contains("Task 2"));
        // Should only have one date header for 2025-01-15
        assert_eq!(html.matches("2025-01-15").count(), 1);
    }

    #[test]
    fn test_render_page_multiple_dates() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
            make_entry("compiti", "2025-01-17", "INGLESE", "Task 3"),
        ];
        let html = render_page(&entries).into_string();

        assert!(html.contains("2025-01-15"));
        assert!(html.contains("2025-01-16"));
        assert!(html.contains("2025-01-17"));
        assert!(html.contains(">3<")); // Total count: 3
    }

    #[test]
    fn test_render_page_has_required_elements() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let html = render_page(&entries).into_string();

        // Check document structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\""));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));

        // Check meta tags
        assert!(html.contains("charset=\"UTF-8\""));
        assert!(html.contains("viewport"));

        // Check title
        assert!(html.contains("<title>Compitutto</title>"));

        // Check CSS and JS
        assert!(html.contains("<style>"));
        assert!(html.contains("<script>"));
    }

    #[test]
    fn test_render_page_has_checkboxes() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
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
            "MATEMATICA",
            "<script>alert('xss')</script>",
        )];
        let html = render_page(&entries).into_string();

        // Should be escaped, not rendered as actual script tag
        assert!(!html.contains("<script>alert('xss')</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_render_page_handles_special_characters() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            "Esercizi con àèìòù & simboli",
        )];
        let html = render_page(&entries).into_string();

        // Ampersand should be escaped
        assert!(html.contains("Esercizi con àèìòù &amp; simboli"));
    }

    #[test]
    fn test_render_page_empty_entry_type() {
        let entries = vec![make_entry(
            "", // Empty type
            "2025-01-15",
            "MATEMATICA",
            "Task 1",
        )];
        let html = render_page(&entries).into_string();

        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("Task 1"));
        // Should not have a type badge element (span with homework-type class)
        // The CSS class definition will still be there, but no <span class="homework-type">
        assert!(!html.contains("<span class=\"homework-type\">"));
    }

    #[test]
    fn test_render_page_entry_type_badge() {
        let entries = vec![make_entry("nota", "2025-01-15", "MATEMATICA", "Task 1")];
        let html = render_page(&entries).into_string();

        assert!(html.contains("homework-type"));
        assert!(html.contains("nota"));
    }

    #[test]
    fn test_render_page_groups_entries_by_date() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
            make_entry("compiti", "2025-01-16", "INGLESE", "Task 3"),
        ];
        let html = render_page(&entries).into_string();

        // Count date-group divs
        let date_groups = html.matches("date-group").count();
        // 2 groups (2 unique dates), each appearing in class name
        assert!(date_groups >= 2);
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

        assert!(html.contains("localStorage"));
        assert!(html.contains("loadCheckboxStates"));
        assert!(html.contains("saveCheckboxState"));
    }

    // ========== render_date_group tests ==========

    #[test]
    fn test_render_date_group_basic() {
        let entries = [
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
        ];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();
        let html = render_date_group("2025-01-15", &refs, 0).into_string();

        assert!(html.contains("date-group"));
        assert!(html.contains("2025-01-15"));
        assert!(html.contains("MATEMATICA"));
        assert!(html.contains("ITALIANO"));
    }

    #[test]
    fn test_render_date_group_entry_ids() {
        let entries = [
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-15", "ITALIANO", "Task 2"),
        ];
        let refs: Vec<&HomeworkEntry> = entries.iter().collect();

        // Group index 0 -> entry IDs start at 1 (0*100+0+1, 0*100+1+1)
        let html = render_date_group("2025-01-15", &refs, 0).into_string();
        assert!(html.contains("entry-1"));
        assert!(html.contains("entry-2"));

        // Group index 1 -> entry IDs start at 101 (1*100+0+1, 1*100+1+1)
        let html2 = render_date_group("2025-01-16", &refs, 1).into_string();
        assert!(html2.contains("entry-101"));
        assert!(html2.contains("entry-102"));
    }

    // ========== generate_html tests ==========

    #[test]
    fn test_generate_html_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");

        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        generate_html(&entries, &html_path).unwrap();

        assert!(html_path.exists());
    }

    #[test]
    fn test_generate_html_content() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("index.html");

        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        generate_html(&entries, &html_path).unwrap();

        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("MATEMATICA"));
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
            "MATEMATICA",
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
            "MATEMATICA",
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

        assert!(html.contains(">100<")); // Total count
        assert!(html.contains("SUBJECT_0"));
        assert!(html.contains("SUBJECT_99"));
    }
}
