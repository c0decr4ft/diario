//! Calendar view rendering (Rust-side HTML structure only).
//! The actual day-cell rendering is done client-side in JavaScript.

use maud::{html, Markup};
use std::collections::BTreeMap;

use crate::types::HomeworkEntry;

/// Render the calendar layout shell: header with prev/next, the day-name grid,
/// the empty days container (populated by JS), and the sidebar.
pub fn render_calendar(
    entries: &[HomeworkEntry],
    by_date: &BTreeMap<&str, Vec<&HomeworkEntry>>,
) -> Markup {
    // Determine which month to show initially — the most recent entry's month.
    let reference_date = entries
        .iter()
        .map(|e| &e.date)
        .max()
        .map(|s| s.as_str())
        .unwrap_or("2025-01-15");

    let parts: Vec<&str> = reference_date.split('-').collect();
    let year: i32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(2025);
    let month: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

    html! {
        div.calendar-layout {
            div.calendar-main {
                div.calendar-header {
                    button.cal-nav-btn #"cal-prev" type="button" { "<" }
                    span.cal-month-year #"cal-month-year" data-year=(year) data-month=(month) {
                        (month_name(month)) " " (year)
                    }
                    button.cal-nav-btn #"cal-next" type="button" { ">" }
                }
                div.calendar-grid {
                    @for day in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                        div.cal-day-header { (day) }
                    }
                }
                div.calendar-days #"calendar-days" data-entries=(entries_to_json(by_date)) {}
            }
            aside.calendar-sidebar #"calendar-sidebar" {
                div.sidebar-header {
                    h3.sidebar-date #"sidebar-date" { "Select a day" }
                    button.sidebar-close #"sidebar-close" type="button" { "×" }
                }
                div.sidebar-content #"sidebar-content" {
                    p.sidebar-empty { "Click on a day to see its entries" }
                }
            }
        }
    }
}

/// Map month number (1-based) to English name.
pub fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

/// Serialize entries grouped by date into a JSON string for the JS calendar renderer.
pub fn entries_to_json(by_date: &BTreeMap<&str, Vec<&HomeworkEntry>>) -> String {
    use std::collections::HashMap;

    let map: HashMap<&str, Vec<_>> = by_date
        .iter()
        .map(|(date, items)| {
            let entries: Vec<_> = items
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "subject": e.subject,
                        "task": e.task,
                        "entry_type": e.entry_type,
                        "completed": e.completed
                    })
                })
                .collect();
            (*date, entries)
        })
        .collect();

    serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
}
