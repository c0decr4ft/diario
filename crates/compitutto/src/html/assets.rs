//! Static CSS and JavaScript assets embedded in the HTML pages.

pub const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;700;900&display=swap');

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

html, body {
    height: 100%;
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
    width: 100%;
    min-height: 100%;
    padding: 30px 40px 60px;
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
}

/* Header styles */
.header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 40px;
    flex-wrap: wrap;
    gap: 20px;
}

.header-left {
    flex: 1;
}

h1 {
    color: #fff;
    font-weight: 900;
    font-size: 3.5em;
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
    padding-top: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
}

/* View toggle */
.view-toggle {
    display: flex;
    gap: 4px;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
}

.view-btn {
    padding: 10px 20px;
    border: none;
    background: transparent;
    color: #888;
    font-weight: 600;
    font-size: 0.9em;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.2s;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    text-decoration: none;
    display: inline-flex;
    align-items: center;
}

.view-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.05);
}

.view-btn.active {
    background: linear-gradient(135deg, #ff0096, #00ffff);
    color: #000;
    box-shadow: 0 0 15px rgba(255, 0, 150, 0.4);
}

/* List view */
.list-view {
    display: grid;
    gap: 50px;
}

.list-view.hidden,
.calendar-view.hidden {
    display: none;
}

.date-group {
    position: relative;
}

.date-items {
    padding-left: 28px;
    position: relative;
}

.date-header {
    color: #fff;
    font-weight: 900;
    font-size: 1.1em;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    margin-bottom: 28px;
    margin-left: -28px;
    padding: 12px 28px;
    background: linear-gradient(90deg, rgba(255, 0, 150, 0.15) 0%, rgba(0, 255, 255, 0.1) 50%, transparent 100%);
    border-left: 4px solid;
    border-image: linear-gradient(180deg, #ff0096, #00ffff) 1;
    text-shadow: 0 0 8px rgba(0,255,255,0.6);
    cursor: pointer;
    user-select: none;
    display: flex;
    align-items: center;
    gap: 12px;
    transition: background 0.2s;
}

.date-header:hover {
    background: linear-gradient(90deg, rgba(255, 0, 150, 0.25) 0%, rgba(0, 255, 255, 0.15) 50%, transparent 100%);
}

.date-header .collapse-indicator {
    font-size: 0.8em;
    transition: transform 0.3s ease;
    display: inline-block;
}

.date-group.collapsed .date-header .collapse-indicator {
    transform: rotate(-90deg);
}

.date-group .date-items {
    overflow: hidden;
    transition: max-height 0.3s ease, opacity 0.3s ease;
    max-height: 2000px;
    opacity: 1;
}

.date-group.collapsed .date-items {
    max-height: 0;
    opacity: 0;
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

.homework-item:hover {
    background: rgba(255,255,255,0.05);
    border-color: rgba(255,0,150,0.4);
    transform: translateX(4px);
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

/* Compiti - pink to purple */
.homework-type[data-type="compiti"] {
    background: linear-gradient(135deg, #ff0096, #9933ff);
    box-shadow: 0 0 8px rgba(255,0,150,0.5);
}

/* Nota - blue to cyan */
.homework-type[data-type="nota"] {
    background: linear-gradient(135deg, #3366ff, #00ffff);
    box-shadow: 0 0 8px rgba(0,255,255,0.5);
}

/* Verifica - orange to red (important!) */
.homework-type[data-type="verifica"] {
    background: linear-gradient(135deg, #ff6600, #ff0033);
    box-shadow: 0 0 8px rgba(255,102,0,0.5);
}

/* Interrogazione - red to pink */
.homework-type[data-type="interrogazione"] {
    background: linear-gradient(135deg, #ff3366, #ff0096);
    box-shadow: 0 0 8px rgba(255,51,102,0.5);
}

/* Studio - cyan to green */
.homework-type[data-type="studio"] {
    background: linear-gradient(135deg, #00ffff, #33ff99);
    box-shadow: 0 0 8px rgba(0,255,255,0.5);
}

/* Lavoro (do-it reminder) - amber to orange */
.homework-type[data-type="lavoro"] {
    background: linear-gradient(135deg, #ffaa00, #ff6600);
    box-shadow: 0 0 8px rgba(255,170,0,0.6);
    color: #000;
}

/* Lavoro item row gets a left border accent */
.lavoro-item {
    border-left: 3px solid #ffaa00;
}

/* Compiti-due item gets a stronger red left border */
.compiti-due-item {
    border-left: 3px solid #ff3366;
}

/* Link to the due date shown under a lavoro task */
.due-link {
    font-size: 0.8em;
    margin-top: 6px;
    color: #aaa;
}
.due-link a {
    color: #ffaa00;
    text-decoration: none;
    font-weight: 700;
}
.due-link a:hover {
    text-decoration: underline;
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

/* Delete button */
.delete-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s;
    font-size: 14px;
    padding: 4px 8px;
    border-radius: 4px;
}

.homework-item:hover .delete-btn {
    opacity: 0.6;
}

.delete-btn:hover {
    opacity: 1 !important;
    background: rgba(255, 0, 0, 0.2);
}

/* Study session (generated) styling */
.homework-item[data-generated="true"] {
    background: rgba(0, 255, 255, 0.03);
}

/* Orphaned study session */
.homework-item[data-orphaned="true"] {
    background: rgba(255, 153, 0, 0.03);
}

.auto-badge, .orphan-badge {
    font-size: 0.55em;
    padding: 2px 6px;
    border-radius: 3px;
    margin-left: 8px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.auto-badge {
    background: rgba(0, 255, 255, 0.2);
    color: #00ffff;
}

.orphan-badge {
    background: rgba(255, 153, 0, 0.2);
    color: #ff9900;
}

/* Drag states */
.homework-item.dragging {
    opacity: 0.4;
}

.homework-item[draggable="true"] {
    cursor: grab;
}

.homework-item[draggable="true"]:active {
    cursor: grabbing;
}

.date-group.drag-over {
    background: rgba(255, 0, 150, 0.05);
    box-shadow: inset 0 0 20px rgba(255, 0, 150, 0.2);
}

/* Add button */
.add-entry-btn {
    position: fixed;
    bottom: 30px;
    right: 30px;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: linear-gradient(135deg, #ff0096, #00ffff);
    border: none;
    color: #000;
    font-size: 28px;
    font-weight: bold;
    cursor: pointer;
    box-shadow: 0 4px 20px rgba(255, 0, 150, 0.4);
    z-index: 100;
    transition: transform 0.2s, box-shadow 0.2s;
}

.add-entry-btn:hover {
    transform: scale(1.1);
    box-shadow: 0 6px 30px rgba(255, 0, 150, 0.6);
}

/* Dialogs */
dialog {
    background: #1a1a1a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    padding: 24px;
    max-width: 400px;
    width: 90%;
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    margin: 0;
}

dialog::backdrop {
    background: rgba(0, 0, 0, 0.7);
}

dialog h3 {
    margin-bottom: 16px;
    font-size: 1.2em;
}

dialog p {
    margin-bottom: 12px;
    color: #ccc;
}

.dialog-note {
    background: rgba(255, 153, 0, 0.1);
    border: 1px solid rgba(255, 153, 0, 0.3);
    border-radius: 4px;
    padding: 12px;
    margin: 16px 0;
}

.dialog-note input {
    width: 100%;
    margin-top: 8px;
    padding: 8px;
    background: #0a0a0a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
}

.dialog-buttons {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 20px;
}

.dialog-buttons button {
    padding: 10px 20px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-weight: 600;
    transition: all 0.2s;
}

.btn-cancel {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.3) !important;
    color: #fff;
}

.btn-cancel:hover {
    background: rgba(255, 255, 255, 0.1);
}

.btn-primary {
    background: linear-gradient(135deg, #ff0096, #00ffff);
    color: #000;
}

.btn-primary:hover {
    box-shadow: 0 0 15px rgba(255, 0, 150, 0.5);
}

.btn-danger {
    background: #ff3333;
    color: #fff;
}

.btn-danger:hover {
    background: #ff5555;
}

/* Form styles */
.form-group {
    margin-bottom: 16px;
}

.form-group label {
    display: block;
    margin-bottom: 6px;
    font-weight: 600;
    color: #ccc;
    font-size: 0.9em;
}

.form-group input,
.form-group select,
.form-group textarea {
    width: 100%;
    padding: 10px;
    background: #0a0a0a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 1em;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
    outline: none;
    border-color: #ff0096;
    box-shadow: 0 0 0 2px rgba(255, 0, 150, 0.2);
}

.form-group select {
    cursor: pointer;
}

.form-group input[type="date"]::-webkit-calendar-picker-indicator {
    filter: invert(1);
    cursor: pointer;
}

.form-group textarea {
    resize: vertical;
    min-height: 80px;
}

/* Calendar view */
.calendar-view {
    width: 100%;
    flex: 1;
    display: flex;
    flex-direction: column;
}

.calendar-layout {
    display: flex;
    gap: 24px;
    flex: 1;
    min-height: 0;
}

.calendar-main {
    flex: 1;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.calendar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
}

.cal-month-year {
    font-size: 1.5em;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.cal-nav-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #fff;
    width: 40px;
    height: 40px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1.2em;
    transition: all 0.2s;
}

.cal-nav-btn:hover {
    background: rgba(255, 0, 150, 0.2);
    border-color: #ff0096;
}

.calendar-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
    margin-bottom: 8px;
}

.cal-day-header {
    padding: 12px 8px;
    text-align: center;
    font-weight: 700;
    font-size: 0.75em;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #888;
}

.calendar-days {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-auto-rows: 1fr;
    gap: 8px;
    flex: 1;
    min-height: 0;
}

.cal-day {
    min-height: 80px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 8px;
    transition: all 0.2s;
    cursor: pointer;
    overflow: hidden;
}

.cal-day:hover {
    border-color: rgba(255, 0, 150, 0.3);
    background: rgba(255, 255, 255, 0.04);
}

.cal-day.other-month { opacity: 0.3; }
.cal-day.today { border-color: #ff0096; box-shadow: 0 0 10px rgba(255, 0, 150, 0.3); }
.cal-day.has-entries { border-color: rgba(0, 255, 255, 0.4); }
.cal-day.selected { border-color: #ff0096; background: rgba(255, 0, 150, 0.1); box-shadow: 0 0 15px rgba(255, 0, 150, 0.3); }

.cal-day-number { font-weight: 700; font-size: 0.9em; margin-bottom: 6px; color: #888; }
.cal-day.today .cal-day-number { color: #ff0096; }
.cal-day.has-entries .cal-day-number { color: #00ffff; }
.cal-day.selected .cal-day-number { color: #ff0096; }

.cal-entry {
    background: rgba(255, 0, 150, 0.15);
    border-left: 3px solid #ff0096;
    padding: 3px 6px;
    margin-bottom: 3px;
    border-radius: 4px;
    font-size: 0.7em;
    transition: all 0.2s;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.cal-entry[data-type="compiti"] { background: rgba(255, 0, 150, 0.15); border-left-color: #ff0096; }
.cal-entry[data-type="nota"] { background: rgba(51, 102, 255, 0.15); border-left-color: #3366ff; }
.cal-entry[data-type="verifica"] { background: rgba(255, 102, 0, 0.15); border-left-color: #ff6600; }
.cal-entry[data-type="interrogazione"] { background: rgba(255, 51, 102, 0.15); border-left-color: #ff3366; }
.cal-entry[data-type="studio"] { background: rgba(0, 255, 255, 0.15); border-left-color: #00ffff; }
.cal-entry.completed { opacity: 0.4; text-decoration: line-through; }

.cal-entry-subject { font-weight: 600; color: #fff; }
.cal-entry-more { font-size: 0.65em; color: #00ffff; text-align: center; padding: 2px; cursor: pointer; }
.cal-entry-more:hover { color: #ff0096; }

/* Calendar Sidebar */
.calendar-sidebar {
    width: 350px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 0, 150, 0.05);
}

.sidebar-date { font-size: 1.1em; font-weight: 700; color: #fff; margin: 0; }
.sidebar-close { background: transparent; border: none; color: #888; font-size: 1.5em; cursor: pointer; padding: 0; line-height: 1; transition: color 0.2s; }
.sidebar-close:hover { color: #ff0096; }
.sidebar-content { flex: 1; overflow-y: auto; padding: 16px; }
.sidebar-empty { color: #666; text-align: center; padding: 40px 20px; font-size: 0.9em; }

.sidebar-entry {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-left: 4px solid #ff0096;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 12px;
    transition: all 0.2s;
}

.sidebar-entry[data-type="compiti"] { border-left-color: #ff0096; }
.sidebar-entry[data-type="nota"] { border-left-color: #3366ff; }
.sidebar-entry[data-type="verifica"] { border-left-color: #ff6600; }
.sidebar-entry[data-type="interrogazione"] { border-left-color: #ff3366; }
.sidebar-entry[data-type="studio"] { border-left-color: #00ffff; }
.sidebar-entry:hover { background: rgba(255, 255, 255, 0.05); border-color: rgba(255, 0, 150, 0.3); }
.sidebar-entry.completed { opacity: 0.5; }

.sidebar-entry-header { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
.sidebar-entry-checkbox { width: 20px; height: 20px; cursor: pointer; accent-color: #ff0096; }
.sidebar-entry-subject { font-weight: 700; font-size: 0.95em; color: #fff; text-transform: uppercase; letter-spacing: 0.03em; }

.sidebar-entry-type {
    background: linear-gradient(135deg, #ff0096, #00ffff);
    color: #000;
    font-size: 0.6em;
    padding: 2px 8px;
    border-radius: 3px;
    text-transform: uppercase;
    font-weight: 700;
    margin-left: auto;
}

.sidebar-entry-type[data-type="compiti"] { background: linear-gradient(135deg, #ff0096, #9933ff); }
.sidebar-entry-type[data-type="nota"] { background: linear-gradient(135deg, #3366ff, #00ffff); }
.sidebar-entry-type[data-type="verifica"] { background: linear-gradient(135deg, #ff6600, #ff0033); }
.sidebar-entry-type[data-type="interrogazione"] { background: linear-gradient(135deg, #ff3366, #ff0096); }
.sidebar-entry-type[data-type="studio"] { background: linear-gradient(135deg, #00ffff, #33ff99); }

.sidebar-entry-task { color: #ccc; font-size: 0.85em; line-height: 1.5; margin-left: 32px; }
.sidebar-entry.completed .sidebar-entry-task { text-decoration: line-through; }

@media (max-width: 1200px) {
    .calendar-layout { flex-direction: column; }
    .calendar-sidebar { width: 100%; max-height: 400px; }
}

@media (max-width: 768px) {
    h1 { font-size: 2.5em; }
    .container { padding: 20px 16px 40px; }
    .header { flex-direction: column; align-items: flex-start; }
    .view-toggle { width: 100%; }
    .view-btn { flex: 1; text-align: center; }
    .calendar-days { gap: 4px; }
    .cal-day { min-height: 70px; padding: 4px; }
    .cal-day-number { font-size: 0.8em; }
    .cal-entry { font-size: 0.6em; padding: 2px 4px; }
    .add-entry-btn { bottom: 20px; right: 20px; width: 48px; height: 48px; font-size: 24px; }
    .calendar-sidebar { max-height: 350px; }
    .sidebar-entry { padding: 12px; }
}
"#;

pub const JAVASCRIPT: &str = r#"
// ========== Helper Functions ==========

function updateCompletedCount(delta) {
    const el = document.getElementById('completed-count');
    if (el) {
        const current = parseInt(el.textContent) || 0;
        el.textContent = current + delta;
    }
}

// ========== Collapsible Date Sections ==========

function checkAndCollapseIfAllCompleted(dateGroup) {
    const items = dateGroup.querySelectorAll('.homework-item');
    const allCompleted = Array.from(items).every(item => item.classList.contains('completed'));
    if (allCompleted && items.length > 0) {
        dateGroup.classList.add('collapsed');
    }
}

document.querySelectorAll('.date-header').forEach(header => {
    header.addEventListener('click', function(e) {
        const dateGroup = this.closest('.date-group');
        dateGroup.classList.toggle('collapsed');
    });
});

// ========== Checkbox Completion (API-backed) ==========

/// Sync a linked entry's visual state and persist it to the API.
/// Used to keep lavoro ↔ compiti completion in sync.
async function syncLinkedEntry(linkedId, isChecked) {
    const linkedItem = document.querySelector(`[data-entry-id="${linkedId}"]`);
    if (!linkedItem) return; // may be off-screen / different date, just persist
    const linkedCheckbox = linkedItem.querySelector('.homework-checkbox');
    if (linkedCheckbox) linkedCheckbox.checked = isChecked;
    if (isChecked) {
        linkedItem.classList.add('completed');
        updateCompletedCount(1);
        checkAndCollapseIfAllCompleted(linkedItem.closest('.date-group'));
    } else {
        linkedItem.classList.remove('completed');
        updateCompletedCount(-1);
        linkedItem.closest('.date-group')?.classList.remove('collapsed');
    }
    // Fire-and-forget — best effort, no revert on error for the linked entry
    await fetch(`/api/entries/${linkedId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ completed: isChecked })
    });
}

document.querySelectorAll('.homework-checkbox').forEach(checkbox => {
    checkbox.addEventListener('change', async function() {
        const entryId = this.getAttribute('data-entry-id');
        const item = document.querySelector(`[data-entry-id="${entryId}"]`);
        const isChecked = this.checked;
        const dateGroup = item.closest('.date-group');

        // Optimistic UI update for the clicked item
        if (isChecked) {
            item.classList.add('completed');
            updateCompletedCount(1);
        } else {
            item.classList.remove('completed');
            updateCompletedCount(-1);
            dateGroup.classList.remove('collapsed');
        }
        if (isChecked) {
            checkAndCollapseIfAllCompleted(dateGroup);
        }

        // Sync the linked entry (lavoro ↔ compiti)
        const parentId = item.dataset.parentId;   // set on lavoro items
        const lavoroId = item.dataset.lavoroId;   // set on compiti items
        const linkedId = parentId || lavoroId;
        if (linkedId) {
            syncLinkedEntry(linkedId, isChecked);
        }

        // Persist the primary entry
        try {
            const response = await fetch(`/api/entries/${entryId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ completed: isChecked })
            });
            if (!response.ok) {
                // Revert primary
                this.checked = !isChecked;
                item.classList.toggle('completed');
                updateCompletedCount(isChecked ? -1 : 1);
                if (isChecked) dateGroup.classList.remove('collapsed');
                console.error('Failed to update completion state');
            }
        } catch (error) {
            this.checked = !isChecked;
            item.classList.toggle('completed');
            updateCompletedCount(isChecked ? -1 : 1);
            if (isChecked) dateGroup.classList.remove('collapsed');
            console.error('Error updating completion:', error);
        }
    });
});

// ========== Delete Functionality ==========

const deleteDialog = document.getElementById('delete-dialog');
const deleteMessage = document.getElementById('delete-message');
const deleteChildrenNote = document.getElementById('delete-children-note');
const deleteConfirmInput = document.getElementById('delete-confirm-input');
const deleteConfirmBtn = document.getElementById('delete-confirm');
const deleteCancelBtn = document.getElementById('delete-cancel');

let pendingDeleteId = null;
let pendingDeleteHasChildren = false;

document.querySelectorAll('.delete-btn').forEach(btn => {
    btn.addEventListener('click', async function(e) {
        e.stopPropagation();
        pendingDeleteId = this.getAttribute('data-entry-id');
        try {
            const response = await fetch(`/api/entries/${pendingDeleteId}/children`);
            const children = await response.json();
            pendingDeleteHasChildren = children.length > 0;
            if (pendingDeleteHasChildren) {
                deleteMessage.textContent = `This entry has ${children.length} study session(s) linked to it.`;
                deleteChildrenNote.style.display = 'block';
                deleteConfirmInput.value = '';
            } else {
                deleteMessage.textContent = 'Are you sure you want to delete this entry?';
                deleteChildrenNote.style.display = 'none';
            }
            deleteDialog.showModal();
        } catch (error) {
            console.error('Error checking children:', error);
        }
    });
});

deleteCancelBtn.addEventListener('click', () => {
    deleteDialog.close();
    pendingDeleteId = null;
    pendingDeleteHasChildren = false;
});

deleteConfirmBtn.addEventListener('click', async () => {
    if (!pendingDeleteId) return;
    if (pendingDeleteHasChildren) {
        const input = deleteConfirmInput.value.toLowerCase().trim();
        if (input !== 'delete all' && input !== 'keep') {
            deleteConfirmInput.focus();
            return;
        }
        try {
            if (input === 'delete all') {
                await fetch(`/api/entries/${pendingDeleteId}/cascade`, { method: 'DELETE' });
            } else {
                await fetch(`/api/entries/${pendingDeleteId}`, { method: 'DELETE' });
            }
            location.reload();
        } catch (error) {
            console.error('Delete error:', error);
        }
    } else {
        try {
            await fetch(`/api/entries/${pendingDeleteId}`, { method: 'DELETE' });
            location.reload();
        } catch (error) {
            console.error('Delete error:', error);
        }
    }
    deleteDialog.close();
});

deleteDialog.addEventListener('click', (e) => {
    if (e.target === deleteDialog) deleteDialog.close();
});

// ========== Drag and Drop ==========

const positionDialog = document.getElementById('position-dialog');
const positionTopBtn = document.getElementById('position-top');
const positionBottomBtn = document.getElementById('position-bottom');
const positionCancelBtn = document.getElementById('position-cancel');

let draggedItem = null;
let draggedEntryId = null;
let targetDate = null;

document.querySelectorAll('.homework-item').forEach(item => {
    item.addEventListener('dragstart', function(e) {
        draggedItem = this;
        draggedEntryId = this.getAttribute('data-entry-id');
        this.classList.add('dragging');
        e.dataTransfer.effectAllowed = 'move';
    });
    item.addEventListener('dragend', function() {
        this.classList.remove('dragging');
        document.querySelectorAll('.date-group').forEach(g => g.classList.remove('drag-over'));
    });
});

document.querySelectorAll('.date-group').forEach(group => {
    group.addEventListener('dragover', function(e) {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        this.classList.add('drag-over');
    });
    group.addEventListener('dragleave', function(e) {
        if (!this.contains(e.relatedTarget)) this.classList.remove('drag-over');
    });
    group.addEventListener('drop', function(e) {
        e.preventDefault();
        this.classList.remove('drag-over');
        if (!draggedItem) return;
        targetDate = this.getAttribute('data-date');
        const sourceDate = draggedItem.closest('.date-group').getAttribute('data-date');
        if (targetDate === sourceDate) { draggedItem = null; return; }
        positionDialog.showModal();
    });
});

async function moveEntry(position) {
    if (!draggedEntryId || !targetDate) return;
    try {
        const entriesResponse = await fetch('/api/entries');
        const entries = await entriesResponse.json();
        const targetEntries = entries.filter(e => e.date === targetDate);
        let newPosition;
        if (position === 'top') {
            newPosition = 0;
            for (const entry of targetEntries) {
                await fetch(`/api/entries/${entry.id}`, {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ position: entry.position + 1 })
                });
            }
        } else {
            newPosition = targetEntries.length > 0
                ? Math.max(...targetEntries.map(e => e.position)) + 1
                : 0;
        }
        await fetch(`/api/entries/${draggedEntryId}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ date: targetDate, position: newPosition })
        });
        location.reload();
    } catch (error) {
        console.error('Error moving entry:', error);
    }
}

positionTopBtn.addEventListener('click', () => { positionDialog.close(); moveEntry('top'); });
positionBottomBtn.addEventListener('click', () => { positionDialog.close(); moveEntry('bottom'); });
positionCancelBtn.addEventListener('click', () => {
    positionDialog.close();
    draggedItem = null; draggedEntryId = null; targetDate = null;
});
positionDialog.addEventListener('click', (e) => {
    if (e.target === positionDialog) {
        positionDialog.close();
        draggedItem = null; draggedEntryId = null; targetDate = null;
    }
});

// ========== Add Entry ==========

const addEntryBtn = document.getElementById('add-entry-btn');
const addEntryDialog = document.getElementById('add-entry-dialog');
const addEntryForm = document.getElementById('add-entry-form');
const addEntryCancelBtn = document.getElementById('add-entry-cancel');

addEntryBtn.addEventListener('click', () => {
    const today = new Date().toISOString().split('T')[0];
    document.getElementById('new-entry-date').value = today;
    document.getElementById('new-entry-subject').value = '';
    document.getElementById('new-entry-type').value = 'compiti';
    document.getElementById('new-entry-task').value = '';
    addEntryDialog.showModal();
});

addEntryCancelBtn.addEventListener('click', () => { addEntryDialog.close(); });

addEntryForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const entry = {
        date: document.getElementById('new-entry-date').value,
        subject: document.getElementById('new-entry-subject').value,
        entry_type: document.getElementById('new-entry-type').value,
        task: document.getElementById('new-entry-task').value
    };
    try {
        const response = await fetch('/api/entries', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(entry)
        });
        if (response.ok) { addEntryDialog.close(); location.reload(); }
        else { console.error('Failed to create entry'); }
    } catch (error) {
        console.error('Error creating entry:', error);
    }
});

addEntryDialog.addEventListener('click', (e) => {
    if (e.target === addEntryDialog) addEntryDialog.close();
});

// ========== View Toggle ==========

const listViewBtn = document.getElementById('list-view-btn');
const calendarViewBtn = document.getElementById('calendar-view-btn');
const listView = document.getElementById('list-view');
const calendarView = document.getElementById('calendar-view');

function showListView() {
    listView.classList.remove('hidden');
    calendarView.classList.add('hidden');
    listViewBtn.classList.add('active');
    calendarViewBtn.classList.remove('active');
    localStorage.setItem('preferredView', 'list');
}

function showCalendarView() {
    listView.classList.add('hidden');
    calendarView.classList.remove('hidden');
    listViewBtn.classList.remove('active');
    calendarViewBtn.classList.add('active');
    localStorage.setItem('preferredView', 'calendar');
    renderCalendar();
}

listViewBtn.addEventListener('click', showListView);
calendarViewBtn.addEventListener('click', showCalendarView);

// ========== Calendar ==========

const calendarDays = document.getElementById('calendar-days');
const calMonthYear = document.getElementById('cal-month-year');
const calPrev = document.getElementById('cal-prev');
const calNext = document.getElementById('cal-next');
const calendarSidebar = document.getElementById('calendar-sidebar');
const sidebarDate = document.getElementById('sidebar-date');
const sidebarContent = document.getElementById('sidebar-content');
const sidebarClose = document.getElementById('sidebar-close');

let currentYear = parseInt(calMonthYear.dataset.year);
let currentMonth = parseInt(calMonthYear.dataset.month);
let selectedDate = null;
let entriesByDate = {};

try {
    entriesByDate = JSON.parse(calendarDays.dataset.entries || '{}');
} catch (e) {
    console.error('Failed to parse entries:', e);
}

const monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
];
const dayNames = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

function formatDateForSidebar(dateStr) {
    const date = new Date(dateStr + 'T00:00:00');
    return `${dayNames[date.getDay()]}, ${monthNames[date.getMonth()]} ${date.getDate()}`;
}

function selectDay(dateStr) {
    document.querySelectorAll('.cal-day.selected').forEach(el => el.classList.remove('selected'));
    const dayEl = document.querySelector(`.cal-day[data-date="${dateStr}"]`);
    if (dayEl) dayEl.classList.add('selected');
    selectedDate = dateStr;
    renderSidebar(dateStr);
}

function renderSidebar(dateStr) {
    const entries = entriesByDate[dateStr] || [];
    sidebarDate.textContent = formatDateForSidebar(dateStr);
    if (entries.length === 0) {
        sidebarContent.innerHTML = '<p class="sidebar-empty">No entries for this day</p>';
        return;
    }
    let html = '';
    entries.forEach(entry => {
        const completedClass = entry.completed ? ' completed' : '';
        const checkedAttr = entry.completed ? ' checked' : '';
        const typeLower = entry.entry_type ? entry.entry_type.toLowerCase() : '';
        const typeAttr = typeLower ? ` data-type="${typeLower}"` : '';
        const typeHtml = entry.entry_type ? `<span class="sidebar-entry-type" data-type="${typeLower}">${escapeHtml(entry.entry_type)}</span>` : '';
        html += `
            <div class="sidebar-entry${completedClass}" data-entry-id="${entry.id}"${typeAttr}>
                <div class="sidebar-entry-header">
                    <input type="checkbox" class="sidebar-entry-checkbox" data-entry-id="${entry.id}"${checkedAttr}>
                    <span class="sidebar-entry-subject">${escapeHtml(entry.subject)}</span>
                    ${typeHtml}
                </div>
                <div class="sidebar-entry-task">${escapeHtml(entry.task)}</div>
            </div>
        `;
    });
    sidebarContent.innerHTML = html;
    sidebarContent.querySelectorAll('.sidebar-entry-checkbox').forEach(checkbox => {
        checkbox.addEventListener('change', handleSidebarCheckbox);
    });
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

async function handleSidebarCheckbox(e) {
    const entryId = e.target.dataset.entryId;
    const isChecked = e.target.checked;
    const entryEl = e.target.closest('.sidebar-entry');
    if (isChecked) { entryEl.classList.add('completed'); updateCompletedCount(1); }
    else { entryEl.classList.remove('completed'); updateCompletedCount(-1); }
    if (selectedDate && entriesByDate[selectedDate]) {
        const entry = entriesByDate[selectedDate].find(e => e.id === entryId);
        if (entry) entry.completed = isChecked;
    }
    renderCalendar();
    if (selectedDate) {
        const dayEl = document.querySelector(`.cal-day[data-date="${selectedDate}"]`);
        if (dayEl) dayEl.classList.add('selected');
    }
    try {
        const response = await fetch(`/api/entries/${entryId}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ completed: isChecked })
        });
        if (!response.ok) {
            e.target.checked = !isChecked;
            entryEl.classList.toggle('completed');
            updateCompletedCount(isChecked ? -1 : 1);
            if (selectedDate && entriesByDate[selectedDate]) {
                const entry = entriesByDate[selectedDate].find(e => e.id === entryId);
                if (entry) entry.completed = !isChecked;
            }
            console.error('Failed to update completion state');
        }
    } catch (error) {
        e.target.checked = !isChecked;
        entryEl.classList.toggle('completed');
        updateCompletedCount(isChecked ? -1 : 1);
        console.error('Error updating completion:', error);
    }
}

function calculateMaxEntries() {
    const calendarDaysRect = calendarDays.getBoundingClientRect();
    if (calendarDaysRect.height === 0) return 2;
    const firstDay = new Date(currentYear, currentMonth - 1, 1);
    const lastDay = new Date(currentYear, currentMonth, 0);
    const daysInMonth = lastDay.getDate();
    let startDayOfWeek = firstDay.getDay();
    startDayOfWeek = startDayOfWeek === 0 ? 6 : startDayOfWeek - 1;
    const numRows = Math.ceil((startDayOfWeek + daysInMonth) / 7);
    const gap = 8;
    const availableHeight = calendarDaysRect.height - (gap * (numRows - 1));
    const cellHeight = availableHeight / numRows;
    const availableForEntries = cellHeight - 24 - 16 - 18;
    return Math.max(1, Math.floor(availableForEntries / 22));
}

function renderCalendar() {
    const year = currentYear;
    const month = currentMonth;
    calMonthYear.textContent = `${monthNames[month - 1]} ${year}`;
    const firstDay = new Date(year, month - 1, 1);
    const lastDay = new Date(year, month, 0);
    const daysInMonth = lastDay.getDate();
    let startDayOfWeek = firstDay.getDay();
    startDayOfWeek = startDayOfWeek === 0 ? 6 : startDayOfWeek - 1;
    const today = new Date();
    const todayStr = today.toISOString().split('T')[0];
    const maxEntries = calculateMaxEntries();
    let html = '';
    const prevMonth = month === 1 ? 12 : month - 1;
    const prevYear = month === 1 ? year - 1 : year;
    const daysInPrevMonth = new Date(prevYear, prevMonth, 0).getDate();
    for (let i = startDayOfWeek - 1; i >= 0; i--) {
        const day = daysInPrevMonth - i;
        const dateStr = `${prevYear}-${String(prevMonth).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
        html += renderCalendarDay(day, dateStr, true, false, false, maxEntries);
    }
    for (let day = 1; day <= daysInMonth; day++) {
        const dateStr = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
        html += renderCalendarDay(day, dateStr, false, dateStr === todayStr, dateStr === selectedDate, maxEntries);
    }
    const totalCells = Math.ceil((startDayOfWeek + daysInMonth) / 7) * 7;
    const nextMonth = month === 12 ? 1 : month + 1;
    const nextYear = month === 12 ? year + 1 : year;
    for (let day = 1; day <= totalCells - startDayOfWeek - daysInMonth; day++) {
        const dateStr = `${nextYear}-${String(nextMonth).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
        html += renderCalendarDay(day, dateStr, true, false, false, maxEntries);
    }
    calendarDays.innerHTML = html;
    calendarDays.querySelectorAll('.cal-day').forEach(dayEl => {
        dayEl.addEventListener('click', () => selectDay(dayEl.dataset.date));
    });
}

function renderCalendarDay(day, dateStr, isOtherMonth, isToday = false, isSelected = false, maxEntries = 2) {
    const entries = entriesByDate[dateStr] || [];
    const hasEntries = entries.length > 0;
    let classes = 'cal-day';
    if (isOtherMonth) classes += ' other-month';
    if (isToday) classes += ' today';
    if (hasEntries) classes += ' has-entries';
    if (isSelected) classes += ' selected';
    let html = `<div class="${classes}" data-date="${dateStr}">`;
    html += `<div class="cal-day-number">${day}</div>`;
    entries.slice(0, maxEntries).forEach(entry => {
        const completedClass = entry.completed ? ' completed' : '';
        const typeAttr = entry.entry_type ? ` data-type="${entry.entry_type.toLowerCase()}"` : '';
        html += `<div class="cal-entry${completedClass}"${typeAttr}>`;
        html += `<span class="cal-entry-subject">${escapeHtml(entry.subject)}</span>`;
        html += '</div>';
    });
    if (entries.length > maxEntries) {
        html += `<div class="cal-entry-more">+${entries.length - maxEntries} more</div>`;
    }
    html += '</div>';
    return html;
}

sidebarClose.addEventListener('click', () => {
    selectedDate = null;
    document.querySelectorAll('.cal-day.selected').forEach(el => el.classList.remove('selected'));
    sidebarDate.textContent = 'Select a day';
    sidebarContent.innerHTML = '<p class="sidebar-empty">Click on a day to see its entries</p>';
});

calPrev.addEventListener('click', () => {
    currentMonth--;
    if (currentMonth < 1) { currentMonth = 12; currentYear--; }
    renderCalendar();
});

calNext.addEventListener('click', () => {
    currentMonth++;
    if (currentMonth > 12) { currentMonth = 1; currentYear++; }
    renderCalendar();
});

let resizeTimeout;
window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
        if (!calendarView.classList.contains('hidden')) renderCalendar();
    }, 150);
});

if (localStorage.getItem('preferredView') === 'calendar') {
    showCalendarView();
} else if (!calendarView.classList.contains('hidden')) {
    renderCalendar();
}

// ========== Due-link scrolling ==========

document.addEventListener('click', function(e) {
    const link = e.target.closest('a[data-scroll-to]');
    if (!link) return;
    e.preventDefault();
    const targetId = link.dataset.scrollTo;
    const targetDate = link.getAttribute('href').replace('#entry-group-', '');
    const group = document.getElementById('entry-group-' + targetDate);
    if (group && group.classList.contains('collapsed')) group.classList.remove('collapsed');
    const entry = document.querySelector('[data-entry-id="' + targetId + '"]');
    if (entry) {
        entry.scrollIntoView({ behavior: 'smooth', block: 'center' });
        entry.style.outline = '2px solid #ffaa00';
        setTimeout(() => { entry.style.outline = ''; }, 2000);
    } else if (group) {
        group.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
});
"#;
