use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// A single homework entry
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct HomeworkEntry {
    /// Type of entry (e.g., "compiti", "nota")
    #[serde(rename = "type")]
    pub entry_type: String,

    /// Due date in YYYY-MM-DD format
    pub date: String,

    /// Subject name
    pub subject: String,

    /// Task description
    pub task: String,
}

impl HomeworkEntry {
    pub fn new(entry_type: String, date: String, subject: String, task: String) -> Self {
        Self {
            entry_type,
            date,
            subject,
            task,
        }
    }

    /// Create a deduplication key for this entry
    /// Two entries are considered duplicates if they have the same date, subject, and task
    pub fn dedup_key(&self) -> String {
        format!("{}|{}|{}", self.date, self.subject, self.task)
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
    use std::collections::HashSet;

    #[test]
    fn test_homework_entry_new() {
        let entry = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Pag. 100 es. 1-5".to_string(),
        );

        assert_eq!(entry.entry_type, "compiti");
        assert_eq!(entry.date, "2025-01-15");
        assert_eq!(entry.subject, "MATEMATICA");
        assert_eq!(entry.task, "Pag. 100 es. 1-5");
    }

    #[test]
    fn test_dedup_key_format() {
        let entry = HomeworkEntry::new(
            "nota".to_string(),
            "2025-01-15".to_string(),
            "ITALIANO".to_string(),
            "Leggere capitolo 3".to_string(),
        );

        assert_eq!(entry.dedup_key(), "2025-01-15|ITALIANO|Leggere capitolo 3");
    }

    #[test]
    fn test_dedup_key_ignores_entry_type() {
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "ITALIANO".to_string(),
            "Leggere capitolo 3".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "nota".to_string(),
            "2025-01-15".to_string(),
            "ITALIANO".to_string(),
            "Leggere capitolo 3".to_string(),
        );

        // Same dedup key even with different entry_type
        assert_eq!(entry1.dedup_key(), entry2.dedup_key());
    }

    #[test]
    fn test_homework_entry_equality() {
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Esercizi".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Esercizi".to_string(),
        );
        let entry3 = HomeworkEntry::new(
            "nota".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Esercizi".to_string(),
        );

        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3); // Different entry_type
    }

    #[test]
    fn test_homework_entry_hash_consistency() {
        let entry1 = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Esercizi".to_string(),
        );
        let entry2 = HomeworkEntry::new(
            "nota".to_string(), // Different type but same date/subject/task
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Esercizi".to_string(),
        );

        let mut set = HashSet::new();
        set.insert(entry1);

        // entry2 has same hash (date/subject/task) so it should be found
        // Note: HashSet uses both hash AND equality, so this tests hash
        let entry2_clone = entry2.clone();
        set.insert(entry2);

        // Both should be in set since PartialEq considers entry_type
        assert_eq!(set.len(), 2);

        // Verify hash is based on date/subject/task only
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn calc_hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        assert_eq!(
            calc_hash(&entry2_clone),
            calc_hash(&HomeworkEntry::new(
                "different_type".to_string(),
                "2025-01-15".to_string(),
                "MATEMATICA".to_string(),
                "Esercizi".to_string(),
            ))
        );
    }

    #[test]
    fn test_homework_entry_clone() {
        let entry = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Esercizi".to_string(),
        );
        let cloned = entry.clone();

        assert_eq!(entry, cloned);
    }

    #[test]
    fn test_homework_entry_serialization() {
        let entry = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Pag. 100".to_string(),
        );

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"type\":\"compiti\""));
        assert!(json.contains("\"date\":\"2025-01-15\""));
        assert!(json.contains("\"subject\":\"MATEMATICA\""));
        assert!(json.contains("\"task\":\"Pag. 100\""));
    }

    #[test]
    fn test_homework_entry_deserialization() {
        let json = r#"{"type":"nota","date":"2025-01-20","subject":"ITALIANO","task":"Studiare"}"#;
        let entry: HomeworkEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.entry_type, "nota");
        assert_eq!(entry.date, "2025-01-20");
        assert_eq!(entry.subject, "ITALIANO");
        assert_eq!(entry.task, "Studiare");
    }

    #[test]
    fn test_homework_entry_roundtrip_serialization() {
        let original = HomeworkEntry::new(
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Esercizi con caratteri speciali: àèìòù & <test>".to_string(),
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: HomeworkEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_homework_entry_empty_fields() {
        let entry = HomeworkEntry::new(String::new(), String::new(), String::new(), String::new());

        assert_eq!(entry.dedup_key(), "||");
        assert!(entry.entry_type.is_empty());
        assert!(entry.date.is_empty());
        assert!(entry.subject.is_empty());
        assert!(entry.task.is_empty());
    }
}
