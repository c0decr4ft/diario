//! Settings page rendering.

use maud::{html, Markup, PreEscaped, DOCTYPE};

use super::assets::CSS;

/// Render the settings page as a full HTML string.
pub fn render_settings_page(work_days: &[u32], days_ahead: u32, study_days: u32) -> String {
    let weekdays: &[(u32, &str)] = &[
        (1u32, "Monday"),
        (2u32, "Tuesday"),
        (3u32, "Wednesday"),
        (4u32, "Thursday"),
        (5u32, "Friday"),
    ];

    let markup: Markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Compitutto — Settings" }
                style { (PreEscaped(CSS)) (PreEscaped(SETTINGS_CSS)) }
            }
            body {
                div.container {
                    header.header {
                        div.header-left {
                            h1 { "Compitutto" }
                        }
                        div.header-right {
                            a.nav-link href="/" { "← Back" }
                        }
                    }
                    div.settings-page {
                        h2 { "Settings" }

                        // ── Work days ──────────────────────────────────────
                        section.settings-section {
                            h3 { "Work days" }
                            p.settings-desc {
                                "Select the days you can work on homework. "
                                "Weekends are always available and can't be disabled."
                            }
                            div.work-days-grid {
                                @for (num, name) in weekdays {
                                    @let checked = work_days.contains(num);
                                    label class={"day-toggle" @if checked { " checked" }} {
                                        input
                                            type="checkbox"
                                            name="work_day"
                                            value=(num)
                                            checked[checked]
                                            data-day=(num);
                                        span { (name) }
                                    }
                                }
                                label.day-toggle.always-on {
                                    input type="checkbox" checked disabled;
                                    span { "Saturday" }
                                    span.always-badge { "always" }
                                }
                                label.day-toggle.always-on {
                                    input type="checkbox" checked disabled;
                                    span { "Sunday" }
                                    span.always-badge { "always" }
                                }
                            }
                        }

                        // ── Homework reminder timing ───────────────────────
                        section.settings-section {
                            h3 { "Homework reminder timing" }
                            p.settings-desc {
                                "How many days before the due date should the "
                                "\"Do it\" reminder appear?"
                            }
                            div.radio-group {
                                @for (val, label) in &[(1u32, "1 day before"), (2u32, "2 days before")] {
                                    label class={"radio-option" @if days_ahead == *val { " checked" }} {
                                        input
                                            type="radio"
                                            name="days_ahead"
                                            value=(val)
                                            checked[days_ahead == *val];
                                        span { (label) }
                                    }
                                }
                            }
                        }

                        // ── Study session days ─────────────────────────────
                        section.settings-section {
                            h3 { "Study days before a verifica" }
                            p.settings-desc {
                                "How many study-session reminders to generate before a test. "
                                "Minimum is 3."
                            }
                            div.stepper-row {
                                button #"study-days-dec" type="button" { "−" }
                                span #"study-days-value" data-value=(study_days) { (study_days) }
                                button #"study-days-inc" type="button" { "+" }
                                span.stepper-hint { "(min 3)" }
                            }
                        }

                        // ── Save ───────────────────────────────────────────
                        div.settings-actions {
                            button #"save-settings" type="button" { "Save all settings" }
                            span #"save-status" {}
                        }

                        // ── Reprocess (hidden until settings are saved) ────
                        section.settings-section #"reprocess-section" style="display:none;margin-top:40px" {
                            h3 { "Reprocess future events" }
                            p.settings-desc {
                                "Your settings were saved. Click below to delete and regenerate all "
                                "future \"Do it\" reminders and study sessions using your new settings. "
                                "Past entries and their completed state are never affected."
                            }
                            div.settings-actions {
                                button #"reprocess-btn" type="button" { "Reprocess future events" }
                                span #"reprocess-status" {}
                            }
                        }
                    }
                }
                script { (PreEscaped(SETTINGS_JS)) }
            }
        }
    };
    markup.into_string()
}

const SETTINGS_CSS: &str = r#"
.header-right { display: flex; align-items: center; }
.nav-link {
    color: #fff;
    text-decoration: none;
    font-weight: 700;
    font-size: 0.9em;
    padding: 8px 16px;
    border: 1px solid rgba(255,255,255,0.2);
    border-radius: 4px;
}
.nav-link:hover { background: rgba(255,255,255,0.1); }
.settings-page { max-width: 600px; padding-top: 40px; }
.settings-page h2 { font-size: 1.8em; font-weight: 900; margin-bottom: 30px; }
.settings-section { margin-bottom: 40px; border-bottom: 1px solid rgba(255,255,255,0.07); padding-bottom: 32px; }
.settings-section h3 { font-size: 1.1em; font-weight: 700; margin-bottom: 10px; color: #fff; }
.settings-desc { color: #aaa; font-size: 0.9em; line-height: 1.6; margin-bottom: 20px; }

.work-days-grid { display: flex; flex-wrap: wrap; gap: 12px; }
.day-toggle {
    display: flex; align-items: center; gap: 8px;
    padding: 10px 18px;
    border: 1px solid rgba(255,255,255,0.15);
    border-radius: 6px;
    cursor: pointer; user-select: none;
    transition: all 0.15s;
    background: rgba(255,255,255,0.04);
}
.day-toggle input[type="checkbox"] { display: none; }
.day-toggle:not(.always-on):hover { border-color: rgba(255,170,0,0.5); background: rgba(255,170,0,0.08); }
.day-toggle.checked { border-color: #ffaa00; background: rgba(255,170,0,0.15); box-shadow: 0 0 8px rgba(255,170,0,0.3); }
.day-toggle.always-on { opacity: 0.45; cursor: default; border-color: rgba(255,255,255,0.08); }
.always-badge { font-size: 0.65em; text-transform: uppercase; letter-spacing: 0.08em; color: #888; margin-left: 4px; }

.radio-group { display: flex; gap: 12px; flex-wrap: wrap; }
.radio-option {
    display: flex; align-items: center; gap: 8px;
    padding: 10px 20px;
    border: 1px solid rgba(255,255,255,0.15);
    border-radius: 6px;
    cursor: pointer; user-select: none;
    transition: all 0.15s;
    background: rgba(255,255,255,0.04);
}
.radio-option input[type="radio"] { display: none; }
.radio-option:hover { border-color: rgba(255,170,0,0.5); background: rgba(255,170,0,0.08); }
.radio-option.checked { border-color: #ffaa00; background: rgba(255,170,0,0.15); box-shadow: 0 0 8px rgba(255,170,0,0.3); }

.stepper-row { display: flex; align-items: center; gap: 16px; }
.stepper-row button {
    width: 36px; height: 36px;
    background: rgba(255,255,255,0.08);
    border: 1px solid rgba(255,255,255,0.2);
    border-radius: 4px;
    color: #fff; font-size: 1.2em; font-weight: 700;
    cursor: pointer; transition: all 0.15s;
}
.stepper-row button:hover { background: rgba(255,170,0,0.2); border-color: #ffaa00; }
#study-days-value { font-size: 1.4em; font-weight: 900; min-width: 2ch; text-align: center; }
.stepper-hint { font-size: 0.8em; color: #666; }

.settings-actions { display: flex; align-items: center; gap: 16px; margin-top: 32px; }
#save-settings {
    padding: 12px 32px;
    background: linear-gradient(135deg, #ffaa00, #ff6600);
    color: #000; font-weight: 900; border: none; border-radius: 4px;
    cursor: pointer; font-size: 0.95em; letter-spacing: 0.05em; text-transform: uppercase;
}
#save-settings:hover { opacity: 0.85; }
#save-status { font-size: 0.85em; color: #33ff99; }
#reprocess-btn {
    padding: 12px 32px;
    background: linear-gradient(135deg, #3366ff, #00ffff);
    color: #000; font-weight: 900; border: none; border-radius: 4px;
    cursor: pointer; font-size: 0.95em; letter-spacing: 0.05em; text-transform: uppercase;
}
#reprocess-btn:hover { opacity: 0.85; }
#reprocess-btn:disabled { opacity: 0.5; cursor: default; }
#reprocess-status { font-size: 0.85em; color: #00ffff; }
"#;

const SETTINGS_JS: &str = r#"
document.querySelectorAll('.day-toggle:not(.always-on)').forEach(label => {
    label.addEventListener('click', (e) => {
        e.preventDefault();
        label.classList.toggle('checked');
    });
});

document.querySelectorAll('.radio-option').forEach(label => {
    label.addEventListener('click', () => {
        document.querySelectorAll('.radio-option').forEach(l => l.classList.remove('checked'));
        label.classList.add('checked');
        label.querySelector('input').checked = true;
    });
});

const studyDaysEl = document.getElementById('study-days-value');
const MIN_STUDY_DAYS = 3;
document.getElementById('study-days-dec').addEventListener('click', () => {
    const v = parseInt(studyDaysEl.dataset.value);
    if (v > MIN_STUDY_DAYS) { studyDaysEl.dataset.value = v - 1; studyDaysEl.textContent = v - 1; }
});
document.getElementById('study-days-inc').addEventListener('click', () => {
    const v = parseInt(studyDaysEl.dataset.value);
    studyDaysEl.dataset.value = v + 1; studyDaysEl.textContent = v + 1;
});

const reprocessSection = document.getElementById('reprocess-section');

document.getElementById('reprocess-btn').addEventListener('click', async () => {
    const status = document.getElementById('reprocess-status');
    const btn = document.getElementById('reprocess-btn');
    status.textContent = 'Reprocessing…';
    btn.disabled = true;
    try {
        const res = await fetch('/api/reprocess', { method: 'POST' });
        if (res.ok) {
            const data = await res.json();
            status.textContent = `✓ Done — removed ${data.deleted}, created ${data.created}`;
            // Hide the section again once reprocessed
            setTimeout(() => {
                status.textContent = '';
                reprocessSection.style.display = 'none';
            }, 4000);
        } else {
            status.textContent = '✗ Error reprocessing';
        }
    } catch (e) {
        status.textContent = '✗ Network error';
    } finally {
        btn.disabled = false;
    }
});

document.getElementById('save-settings').addEventListener('click', async () => {
    const status = document.getElementById('save-status');
    status.textContent = '';

    const workDays = Array.from(document.querySelectorAll('input[name="work_day"]'))
        .filter(cb => cb.closest('.day-toggle').classList.contains('checked'))
        .map(cb => parseInt(cb.dataset.day));

    const daysAhead = parseInt(
        document.querySelector('input[name="days_ahead"]:checked')?.value ?? '2'
    );

    const studyDays = parseInt(studyDaysEl.dataset.value);

    try {
        const results = await Promise.all([
            fetch('/api/settings/work-days', {
                method: 'PUT', headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ days: workDays }),
            }),
            fetch('/api/settings/homework-days-ahead', {
                method: 'PUT', headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ value: daysAhead }),
            }),
            fetch('/api/settings/study-days-before', {
                method: 'PUT', headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ value: studyDays }),
            }),
        ]);

        if (results.every(r => r.ok)) {
            status.textContent = '✓ Saved';
            setTimeout(() => { status.textContent = ''; }, 3000);
            // Reveal the reprocess section now that settings changed
            reprocessSection.style.display = 'block';
            reprocessSection.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        } else {
            status.textContent = '✗ Error saving one or more settings';
        }
    } catch (e) {
        status.textContent = '✗ Network error';
    }
});
"#;
