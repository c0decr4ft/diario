-- Settings table for user preferences
-- Stores key/value pairs as text (values are JSON-encoded where needed)

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Default: Mon–Fri are work days (1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri)
-- Sat (6) and Sun (0) are always available and not stored here.
INSERT OR IGNORE INTO settings (key, value)
VALUES ('work_days', '[1,2,3,4,5]');
