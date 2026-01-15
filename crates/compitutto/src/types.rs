use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// A single homework entry
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct HomeworkEntry {
    /// Unique identifier for this entry (UUID-like, changes if entry is recreated)
    pub id: String,

    /// Source identifier for import deduplication.
    /// Based on original (date, subject, task) from the export file.
    /// Used to detect duplicates during re-import even if entry was moved to a different date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,

    /// Type of entry (e.g., "compiti", "nota", "studio")
    #[serde(rename = "type")]
    pub entry_type: String,

    /// Due date in YYYY-MM-DD format
    pub date: String,

    /// Subject name
    pub subject: String,

    /// Task description
    pub task: String,

    /// Whether this entry has been completed
    #[serde(default)]
    pub completed: bool,

    /// Position within the day for ordering
    #[serde(default)]
    pub position: i32,

    /// Parent entry ID (for auto-generated study sessions)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// When this entry was created (RFC 3339 format)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub created_at: String,

    /// When this entry was last updated (RFC 3339 format)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub updated_at: String,
}

impl HomeworkEntry {
    /// Create a new homework entry with auto-generated ID and timestamps.
    /// The source_id is set to match the original (date, subject, task) for import deduplication.
    pub fn new(entry_type: String, date: String, subject: String, task: String) -> Self {
        let source_id = Self::generate_source_id(&date, &subject, &task);
        let id = Self::generate_id();
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            source_id: Some(source_id),
            entry_type,
            date,
            subject,
            task,
            completed: false,
            position: 0,
            parent_id: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Create a new entry with a specific ID (useful for study sessions in tests)
    #[cfg(test)]
    pub fn with_id(
        id: String,
        entry_type: String,
        date: String,
        subject: String,
        task: String,
    ) -> Self {
        let source_id = Self::generate_source_id(&date, &subject, &task);
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            source_id: Some(source_id),
            entry_type,
            date,
            subject,
            task,
            completed: false,
            position: 0,
            parent_id: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Generate a unique ID for this entry (not content-based, just unique)
    fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Generate a source ID based on date, subject, and task.
    /// This is used for import deduplication - entries with the same source_id
    /// are considered duplicates even if the entry has been moved to a different date.
    pub fn generate_source_id(date: &str, subject: &str, task: &str) -> String {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        date.hash(&mut hasher);
        subject.hash(&mut hasher);
        task.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Generate a stable ID for this entry based on its content.
    /// Used for persistent UI state (e.g., checkbox completion in localStorage).
    /// The ID is an 8-character hex string derived from hashing date+subject+task.
    pub fn stable_id(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:08x}", hasher.finish() as u32)
    }

    /// Check if this is an auto-generated study session
    pub fn is_generated(&self) -> bool {
        self.parent_id.is_some()
    }

    /// Check if this is an orphaned study session (was generated but parent deleted)
    pub fn is_orphaned(&self) -> bool {
        self.entry_type == "studio" && self.parent_id.is_none()
    }
}

impl Hash for HomeworkEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.date.hash(state);
        self.subject.hash(state);
        self.task.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homework_entry_new() {
        let entry = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Pag. 100 es. 1-5".to_string(),
        );

        assert_eq!(entry.entry_type, "compiti");
        assert_eq!(entry.date, "2025-01-15");
        assert_eq!(entry.subject, "Matematica");
        assert_eq!(entry.task, "Pag. 100 es. 1-5");
        assert!(!entry.completed);
        assert_eq!(entry.position, 0);
        assert!(entry.parent_id.is_none());
        assert!(!entry.id.is_empty());
        assert!(!entry.created_at.is_empty());
        assert!(!entry.updated_at.is_empty());
    }

    #[test]
    fn test_homework_entry_equality() {
        // Entries with same content should have same source_id but different id
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );

        // Different id (each entry gets a unique id)
        assert_ne!(entry1.id, entry2.id);
        // Same source_id (content-based, used for deduplication)
        assert_eq!(entry1.source_id, entry2.source_id);
    }

    #[test]
    fn test_homework_entry_hash_consistency() {
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "nota".to_string(), // Different type but same date/subject/task
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );

        // Verify hash is based on date/subject/task only
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn calc_hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        // Same content (date/subject/task) = same hash regardless of entry_type
        assert_eq!(calc_hash(&entry1), calc_hash(&entry2));
    }

    #[test]
    fn test_homework_entry_clone() {
        let entry = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );
        let cloned = entry.clone();

        assert_eq!(entry.id, cloned.id);
        assert_eq!(entry.entry_type, cloned.entry_type);
        assert_eq!(entry.date, cloned.date);
        assert_eq!(entry.subject, cloned.subject);
        assert_eq!(entry.task, cloned.task);
    }

    #[test]
    fn test_homework_entry_serialization() {
        let entry = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Pag. 100".to_string(),
        );

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"id\":"));
        assert!(json.contains("\"type\":\"compiti\""));
        assert!(json.contains("\"date\":\"2025-01-15\""));
        assert!(json.contains("\"subject\":\"Matematica\""));
        assert!(json.contains("\"task\":\"Pag. 100\""));
        assert!(json.contains("\"completed\":false"));
        assert!(json.contains("\"position\":0"));
        assert!(json.contains("\"created_at\":"));
        assert!(json.contains("\"updated_at\":"));
    }

    #[test]
    fn test_homework_entry_deserialization() {
        // Test with minimal JSON (using defaults for new fields)
        let json = r#"{"id":"abc123","type":"nota","date":"2025-01-20","subject":"Italiano","task":"Studiare"}"#;
        let entry: HomeworkEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.id, "abc123");
        assert_eq!(entry.entry_type, "nota");
        assert_eq!(entry.date, "2025-01-20");
        assert_eq!(entry.subject, "Italiano");
        assert_eq!(entry.task, "Studiare");
        assert!(!entry.completed); // default
        assert_eq!(entry.position, 0); // default
        assert!(entry.parent_id.is_none()); // default
    }

    #[test]
    fn test_homework_entry_roundtrip_serialization() {
        let original = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi con caratteri speciali: àèìòù & <test>".to_string(),
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: HomeworkEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.entry_type, deserialized.entry_type);
        assert_eq!(original.date, deserialized.date);
        assert_eq!(original.subject, deserialized.subject);
        assert_eq!(original.task, deserialized.task);
        assert_eq!(original.completed, deserialized.completed);
        assert_eq!(original.position, deserialized.position);
        assert_eq!(original.parent_id, deserialized.parent_id);
    }

    #[test]
    fn test_homework_entry_empty_fields() {
        let entry = HomeworkEntry::new(String::new(), String::new(), String::new(), String::new());

        assert!(entry.entry_type.is_empty());
        assert!(entry.date.is_empty());
        assert!(entry.subject.is_empty());
        assert!(entry.task.is_empty());
    }

    #[test]
    fn test_stable_id_consistency() {
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "nota".to_string(), // Different type
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );

        // Same content (date/subject/task) = same stable_id
        assert_eq!(entry1.stable_id(), entry2.stable_id());

        // ID is 8 hex characters
        assert_eq!(entry1.stable_id().len(), 8);
        assert!(entry1.stable_id().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_stable_id_different_content() {
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Task A".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Task B".to_string(),
        );

        // Different content = different stable_id
        assert_ne!(entry1.stable_id(), entry2.stable_id());
    }

    #[test]
    fn test_stable_id_deterministic() {
        let entry = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );

        // Same entry should always produce the same ID
        let id1 = entry.stable_id();
        let id2 = entry.stable_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_is_generated() {
        let mut entry = HomeworkEntry::new(
            "studio".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Study for: Test".to_string(),
        );

        // Not generated initially
        assert!(!entry.is_generated());

        // Set parent_id to make it generated
        entry.parent_id = Some("parent123".to_string());
        assert!(entry.is_generated());
    }

    #[test]
    fn test_is_orphaned() {
        let mut entry = HomeworkEntry::new(
            "studio".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Study for: Test".to_string(),
        );

        // Studio entry without parent is orphaned
        assert!(entry.is_orphaned());

        // With parent, not orphaned
        entry.parent_id = Some("parent123".to_string());
        assert!(!entry.is_orphaned());

        // Non-studio entry without parent is not orphaned
        let regular = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );
        assert!(!regular.is_orphaned());
    }

    #[test]
    fn test_with_id() {
        let entry = HomeworkEntry::with_id(
            "custom-id-123".to_string(),
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );

        assert_eq!(entry.id, "custom-id-123");
        assert_eq!(entry.entry_type, "compiti");
        assert!(!entry.completed);
        assert_eq!(entry.position, 0);
    }

    #[test]
    fn test_source_id_deterministic() {
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "nota".to_string(), // Different type
            "2025-01-15".to_string(),
            "Matematica".to_string(),
            "Esercizi".to_string(),
        );

        // Same date/subject/task = same source_id (used for deduplication)
        assert_eq!(entry1.source_id, entry2.source_id);
        assert!(entry1.source_id.as_ref().unwrap().len() == 16); // 16 hex chars

        // But different id (unique per entry, UUID v4 format)
        assert_ne!(entry1.id, entry2.id);
        assert_eq!(entry1.id.len(), 36); // UUID format: 8-4-4-4-12
    }

    #[test]
    fn test_rapid_id_generation_uniqueness() {
        // Create many entries rapidly to ensure IDs are unique
        let mut ids = std::collections::HashSet::new();
        for i in 0..100 {
            let entry = HomeworkEntry::new(
                "compiti".to_string(),
                format!("2025-01-{:02}", i % 28 + 1),
                format!("SUBJECT_{}", i),
                format!("Task {}", i),
            );
            assert!(
                ids.insert(entry.id.clone()),
                "Duplicate ID generated: {}",
                entry.id
            );
        }
        assert_eq!(ids.len(), 100);
    }
}
