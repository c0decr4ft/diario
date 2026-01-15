use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::HomeworkEntry;

/// Parse an Excel XML (SpreadsheetML) file and extract homework entries
pub fn parse_excel_xml(path: &Path) -> Result<Vec<HomeworkEntry>> {
    let content = fs::read_to_string(path).context("Failed to read file")?;

    // Check if it's XML format
    if !content.starts_with("<?xml") && !content.contains("<Workbook") {
        anyhow::bail!("File does not appear to be Excel XML format");
    }

    let rows = parse_spreadsheet_rows(&content)?;

    if rows.is_empty() {
        anyhow::bail!("No data rows found in file");
    }

    // First row is headers
    let headers = &rows[0];
    println!("Found columns: {:?}", headers);

    // Map column indices
    let col_indices = map_columns(headers);

    // Parse data rows into entries
    let mut entries = Vec::new();

    for row in rows.iter().skip(1) {
        if let Some(entry) = parse_row(row, &col_indices) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Parse SpreadsheetML XML into rows of cell values
fn parse_spreadsheet_rows(xml: &str) -> Result<Vec<Vec<String>>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_row = false;
    let mut in_cell = false;
    let mut in_data = false;
    let mut current_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"Row" => {
                        in_row = true;
                        current_row = Vec::new();
                    }
                    b"Cell" => {
                        if in_row {
                            in_cell = true;
                        }
                    }
                    b"Data" => {
                        if in_cell {
                            in_data = true;
                            current_text.clear();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"Row" => {
                        if in_row && !current_row.is_empty() {
                            rows.push(current_row.clone());
                        }
                        in_row = false;
                    }
                    b"Cell" => {
                        if in_cell && !in_data {
                            // Empty cell
                            current_row.push(String::new());
                        }
                        in_cell = false;
                    }
                    b"Data" => {
                        if in_data {
                            current_row.push(current_text.trim().to_string());
                            current_text.clear();
                        }
                        in_data = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_data {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("XML parse error: {}", e),
            _ => {}
        }
    }

    Ok(rows)
}

/// Map header names to column indices
fn map_columns(headers: &[String]) -> HashMap<&'static str, usize> {
    let mut indices = HashMap::new();

    for (i, header) in headers.iter().enumerate() {
        let lower = header.to_lowercase();

        // Date column
        if lower.contains("data") || lower.contains("inizio") || lower.contains("date") {
            indices.entry("date").or_insert(i);
        }

        // Subject column
        if lower.contains("materia") || lower.contains("subject") || lower.contains("corso") {
            indices.entry("subject").or_insert(i);
        }

        // Task/description column
        if lower.contains("nota")
            || lower.contains("descrizione")
            || lower.contains("task")
            || lower.contains("compito")
        {
            indices.entry("task").or_insert(i);
        }

        // Type column (but not "tipo evento")
        if lower == "tipo" || (lower.contains("tipo") && !lower.contains("evento")) {
            indices.entry("type").or_insert(i);
        }
    }

    indices
}

/// Parse a single row into a HomeworkEntry
fn parse_row(row: &[String], col_indices: &HashMap<&'static str, usize>) -> Option<HomeworkEntry> {
    let get_col = |key: &str| -> String {
        col_indices
            .get(key)
            .and_then(|&i| row.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };

    let entry_type = get_col("type");
    let date = normalize_date(&get_col("date"));
    let subject = get_col("subject");
    let task = get_col("task");

    // Only include entries with meaningful data
    if task.is_empty() && subject.is_empty() {
        return None;
    }

    Some(HomeworkEntry::new(entry_type, date, subject, task))
}

/// Normalize date to YYYY-MM-DD format
fn normalize_date(date: &str) -> String {
    // If it contains a space (datetime), take just the date part
    let date_part = date.split_whitespace().next().unwrap_or(date);

    // Already in correct format
    if date_part.len() == 10 && date_part.chars().nth(4) == Some('-') {
        return date_part.to_string();
    }

    date_part.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a temporary Excel XML file for testing
    fn create_test_xml_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    /// Minimal valid Excel XML with headers and one data row
    fn minimal_excel_xml() -> String {
        r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Table1">
<Table>
<Row>
<Cell><Data ss:Type="String">tipo</Data></Cell>
<Cell><Data ss:Type="String">data_inizio</Data></Cell>
<Cell><Data ss:Type="String">materia</Data></Cell>
<Cell><Data ss:Type="String">nota</Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">compiti</Data></Cell>
<Cell><Data ss:Type="String">2025-01-15</Data></Cell>
<Cell><Data ss:Type="String">MATEMATICA</Data></Cell>
<Cell><Data ss:Type="String">Pag. 100 es. 1-5</Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#
            .to_string()
    }

    /// Excel XML with multiple rows
    fn multi_row_excel_xml() -> String {
        r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Table1">
<Table>
<Row>
<Cell><Data ss:Type="String">tipo</Data></Cell>
<Cell><Data ss:Type="String">data_inizio</Data></Cell>
<Cell><Data ss:Type="String">materia</Data></Cell>
<Cell><Data ss:Type="String">nota</Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">compiti</Data></Cell>
<Cell><Data ss:Type="String">2025-01-15</Data></Cell>
<Cell><Data ss:Type="String">MATEMATICA</Data></Cell>
<Cell><Data ss:Type="String">Pag. 100 es. 1-5</Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">nota</Data></Cell>
<Cell><Data ss:Type="String">2025-01-16</Data></Cell>
<Cell><Data ss:Type="String">ITALIANO</Data></Cell>
<Cell><Data ss:Type="String">Verifica capitolo 3</Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">compiti</Data></Cell>
<Cell><Data ss:Type="String">2025-01-17</Data></Cell>
<Cell><Data ss:Type="String">INGLESE</Data></Cell>
<Cell><Data ss:Type="String">Exercise page 50</Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#
            .to_string()
    }

    // ========== parse_excel_xml tests ==========

    #[test]
    fn test_parse_excel_xml_single_row() {
        let file = create_test_xml_file(&minimal_excel_xml());
        let entries = parse_excel_xml(file.path()).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "compiti");
        assert_eq!(entries[0].date, "2025-01-15");
        assert_eq!(entries[0].subject, "MATEMATICA");
        assert_eq!(entries[0].task, "Pag. 100 es. 1-5");
    }

    #[test]
    fn test_parse_excel_xml_multiple_rows() {
        let file = create_test_xml_file(&multi_row_excel_xml());
        let entries = parse_excel_xml(file.path()).unwrap();

        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].subject, "MATEMATICA");
        assert_eq!(entries[1].subject, "ITALIANO");
        assert_eq!(entries[2].subject, "INGLESE");
    }

    #[test]
    fn test_parse_excel_xml_not_xml_format() {
        let file = create_test_xml_file("This is not XML content");
        let result = parse_excel_xml(file.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Excel XML format"));
    }

    #[test]
    fn test_parse_excel_xml_empty_table() {
        let xml = r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Table1">
<Table>
</Table>
</Worksheet>
</Workbook>"#;

        let file = create_test_xml_file(xml);
        let result = parse_excel_xml(file.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No data rows"));
    }

    #[test]
    fn test_parse_excel_xml_headers_only() {
        let xml = r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Table1">
<Table>
<Row>
<Cell><Data ss:Type="String">tipo</Data></Cell>
<Cell><Data ss:Type="String">data_inizio</Data></Cell>
<Cell><Data ss:Type="String">materia</Data></Cell>
<Cell><Data ss:Type="String">nota</Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#;

        let file = create_test_xml_file(xml);
        let entries = parse_excel_xml(file.path()).unwrap();

        // Only headers, no data rows = empty result
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_excel_xml_file_not_found() {
        let result = parse_excel_xml(Path::new("/nonexistent/file.xls"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_excel_xml_with_empty_cells() {
        let xml = r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Table1">
<Table>
<Row>
<Cell><Data ss:Type="String">tipo</Data></Cell>
<Cell><Data ss:Type="String">data_inizio</Data></Cell>
<Cell><Data ss:Type="String">materia</Data></Cell>
<Cell><Data ss:Type="String">nota</Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">compiti</Data></Cell>
<Cell><Data ss:Type="String">2025-01-15</Data></Cell>
<Cell><Data ss:Type="String"></Data></Cell>
<Cell><Data ss:Type="String">Task without subject</Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#;

        let file = create_test_xml_file(xml);
        let entries = parse_excel_xml(file.path()).unwrap();

        assert_eq!(entries.len(), 1);
        assert!(entries[0].subject.is_empty());
        assert_eq!(entries[0].task, "Task without subject");
    }

    #[test]
    fn test_parse_excel_xml_skips_empty_task_and_subject() {
        let xml = r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Table1">
<Table>
<Row>
<Cell><Data ss:Type="String">tipo</Data></Cell>
<Cell><Data ss:Type="String">data_inizio</Data></Cell>
<Cell><Data ss:Type="String">materia</Data></Cell>
<Cell><Data ss:Type="String">nota</Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">compiti</Data></Cell>
<Cell><Data ss:Type="String">2025-01-15</Data></Cell>
<Cell><Data ss:Type="String"></Data></Cell>
<Cell><Data ss:Type="String"></Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">compiti</Data></Cell>
<Cell><Data ss:Type="String">2025-01-16</Data></Cell>
<Cell><Data ss:Type="String">MATEMATICA</Data></Cell>
<Cell><Data ss:Type="String">Valid task</Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#;

        let file = create_test_xml_file(xml);
        let entries = parse_excel_xml(file.path()).unwrap();

        // First row should be skipped (empty task and subject)
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].task, "Valid task");
    }

    #[test]
    fn test_parse_excel_xml_with_special_characters() {
        let xml = r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Table1">
<Table>
<Row>
<Cell><Data ss:Type="String">tipo</Data></Cell>
<Cell><Data ss:Type="String">data_inizio</Data></Cell>
<Cell><Data ss:Type="String">materia</Data></Cell>
<Cell><Data ss:Type="String">nota</Data></Cell>
</Row>
<Row>
<Cell><Data ss:Type="String">compiti</Data></Cell>
<Cell><Data ss:Type="String">2025-01-15</Data></Cell>
<Cell><Data ss:Type="String">MATEMATICA</Data></Cell>
<Cell><Data ss:Type="String">Esercizi con àèìòù &amp; simboli</Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#;

        let file = create_test_xml_file(xml);
        let entries = parse_excel_xml(file.path()).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].task, "Esercizi con àèìòù & simboli");
    }

    // ========== parse_spreadsheet_rows tests ==========

    #[test]
    fn test_parse_spreadsheet_rows_basic() {
        let xml = r#"<?xml version="1.0"?>
<Workbook>
<Worksheet>
<Table>
<Row>
<Cell><Data>A1</Data></Cell>
<Cell><Data>B1</Data></Cell>
</Row>
<Row>
<Cell><Data>A2</Data></Cell>
<Cell><Data>B2</Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#;

        let rows = parse_spreadsheet_rows(xml).unwrap();

        assert_eq!(rows.len(), 2);
        // Parser adds empty string for Cell end when no Data was inside
        // The actual values are at positions 0 and 2 (with empty strings between)
        assert!(rows[0].contains(&"A1".to_string()));
        assert!(rows[0].contains(&"B1".to_string()));
        assert!(rows[1].contains(&"A2".to_string()));
        assert!(rows[1].contains(&"B2".to_string()));
    }

    #[test]
    fn test_parse_spreadsheet_rows_with_whitespace() {
        let xml = r#"<?xml version="1.0"?>
<Workbook>
<Worksheet>
<Table>
<Row>
<Cell><Data>  trimmed  </Data></Cell>
</Row>
</Table>
</Worksheet>
</Workbook>"#;

        let rows = parse_spreadsheet_rows(xml).unwrap();

        assert_eq!(rows[0][0], "trimmed");
    }

    #[test]
    fn test_parse_spreadsheet_rows_empty_xml() {
        let xml = r#"<?xml version="1.0"?>
<Workbook>
<Worksheet>
<Table>
</Table>
</Worksheet>
</Workbook>"#;

        let rows = parse_spreadsheet_rows(xml).unwrap();
        assert!(rows.is_empty());
    }

    // ========== map_columns tests ==========

    #[test]
    fn test_map_columns_standard_headers() {
        let headers = vec![
            "tipo".to_string(),
            "data_inizio".to_string(),
            "materia".to_string(),
            "nota".to_string(),
        ];

        let indices = map_columns(&headers);

        assert_eq!(indices.get("type"), Some(&0));
        assert_eq!(indices.get("date"), Some(&1));
        assert_eq!(indices.get("subject"), Some(&2));
        assert_eq!(indices.get("task"), Some(&3));
    }

    #[test]
    fn test_map_columns_case_insensitive() {
        let headers = vec![
            "TIPO".to_string(),
            "DATA_INIZIO".to_string(),
            "MATERIA".to_string(),
            "NOTA".to_string(),
        ];

        let indices = map_columns(&headers);

        assert_eq!(indices.get("type"), Some(&0));
        assert_eq!(indices.get("date"), Some(&1));
        assert_eq!(indices.get("subject"), Some(&2));
        assert_eq!(indices.get("task"), Some(&3));
    }

    #[test]
    fn test_map_columns_alternative_names() {
        let headers = vec![
            "date".to_string(),
            "subject".to_string(),
            "task".to_string(),
            "corso".to_string(),
        ];

        let indices = map_columns(&headers);

        assert_eq!(indices.get("date"), Some(&0));
        assert_eq!(indices.get("subject"), Some(&1)); // First match wins
        assert_eq!(indices.get("task"), Some(&2));
    }

    #[test]
    fn test_map_columns_tipo_evento_excluded() {
        let headers = vec![
            "tipo_evento".to_string(),
            "tipo".to_string(),
            "data_inizio".to_string(),
        ];

        let indices = map_columns(&headers);

        // "tipo_evento" contains "evento" so it should NOT match
        // "tipo" should match
        assert_eq!(indices.get("type"), Some(&1));
    }

    #[test]
    fn test_map_columns_first_match_wins() {
        let headers = vec![
            "data_inizio".to_string(),
            "date".to_string(),
            "another_data".to_string(),
        ];

        let indices = map_columns(&headers);

        // First matching column should be used
        assert_eq!(indices.get("date"), Some(&0));
    }

    #[test]
    fn test_map_columns_missing_columns() {
        let headers = vec!["unknown1".to_string(), "unknown2".to_string()];

        let indices = map_columns(&headers);

        assert!(!indices.contains_key("type"));
        assert!(!indices.contains_key("date"));
        assert!(!indices.contains_key("subject"));
        assert!(!indices.contains_key("task"));
    }

    #[test]
    fn test_map_columns_descrizione_matches_task() {
        let headers = vec!["descrizione".to_string()];

        let indices = map_columns(&headers);
        assert_eq!(indices.get("task"), Some(&0));
    }

    #[test]
    fn test_map_columns_compito_matches_task() {
        let headers = vec!["compito".to_string()];

        let indices = map_columns(&headers);
        assert_eq!(indices.get("task"), Some(&0));
    }

    // ========== normalize_date tests ==========

    #[test]
    fn test_normalize_date_already_correct() {
        assert_eq!(normalize_date("2025-01-15"), "2025-01-15");
    }

    #[test]
    fn test_normalize_date_with_time() {
        assert_eq!(normalize_date("2025-01-15 12:30:00"), "2025-01-15");
    }

    #[test]
    fn test_normalize_date_with_datetime() {
        assert_eq!(normalize_date("2025-01-15 08:00:00"), "2025-01-15");
    }

    #[test]
    fn test_normalize_date_empty() {
        assert_eq!(normalize_date(""), "");
    }

    #[test]
    fn test_normalize_date_short_date() {
        assert_eq!(normalize_date("2025-1-5"), "2025-1-5");
    }

    #[test]
    fn test_normalize_date_different_format() {
        // Non-standard format passes through
        assert_eq!(normalize_date("15/01/2025"), "15/01/2025");
    }

    // ========== parse_row tests ==========

    #[test]
    fn test_parse_row_complete() {
        let row = vec![
            "compiti".to_string(),
            "2025-01-15".to_string(),
            "MATEMATICA".to_string(),
            "Pag. 100".to_string(),
        ];

        let mut indices = HashMap::new();
        indices.insert("type", 0);
        indices.insert("date", 1);
        indices.insert("subject", 2);
        indices.insert("task", 3);

        let entry = parse_row(&row, &indices).unwrap();

        assert_eq!(entry.entry_type, "compiti");
        assert_eq!(entry.date, "2025-01-15");
        assert_eq!(entry.subject, "MATEMATICA");
        assert_eq!(entry.task, "Pag. 100");
    }

    #[test]
    fn test_parse_row_missing_columns() {
        let row = vec!["compiti".to_string(), "2025-01-15".to_string()];

        let mut indices = HashMap::new();
        indices.insert("type", 0);
        indices.insert("date", 1);
        indices.insert("subject", 5); // Out of bounds
        indices.insert("task", 6); // Out of bounds

        let result = parse_row(&row, &indices);

        // Should return None because task and subject are empty
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_row_only_task() {
        let row = vec![
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "Task only".to_string(),
        ];

        let mut indices = HashMap::new();
        indices.insert("type", 0);
        indices.insert("date", 1);
        indices.insert("subject", 2);
        indices.insert("task", 3);

        let entry = parse_row(&row, &indices).unwrap();
        assert_eq!(entry.task, "Task only");
    }

    #[test]
    fn test_parse_row_only_subject() {
        let row = vec![
            "".to_string(),
            "".to_string(),
            "MATEMATICA".to_string(),
            "".to_string(),
        ];

        let mut indices = HashMap::new();
        indices.insert("type", 0);
        indices.insert("date", 1);
        indices.insert("subject", 2);
        indices.insert("task", 3);

        let entry = parse_row(&row, &indices).unwrap();
        assert_eq!(entry.subject, "MATEMATICA");
    }

    #[test]
    fn test_parse_row_trims_whitespace() {
        let row = vec![
            "  compiti  ".to_string(),
            " 2025-01-15 ".to_string(),
            " MATEMATICA ".to_string(),
            " Pag. 100 ".to_string(),
        ];

        let mut indices = HashMap::new();
        indices.insert("type", 0);
        indices.insert("date", 1);
        indices.insert("subject", 2);
        indices.insert("task", 3);

        let entry = parse_row(&row, &indices).unwrap();

        assert_eq!(entry.entry_type, "compiti");
        assert_eq!(entry.date, "2025-01-15");
        assert_eq!(entry.subject, "MATEMATICA");
        assert_eq!(entry.task, "Pag. 100");
    }

    #[test]
    fn test_parse_row_normalizes_date_with_time() {
        let row = vec![
            "compiti".to_string(),
            "2025-01-15 12:00:00".to_string(),
            "MATEMATICA".to_string(),
            "Task".to_string(),
        ];

        let mut indices = HashMap::new();
        indices.insert("type", 0);
        indices.insert("date", 1);
        indices.insert("subject", 2);
        indices.insert("task", 3);

        let entry = parse_row(&row, &indices).unwrap();
        assert_eq!(entry.date, "2025-01-15");
    }

    // ========== Integration test with real-world format ==========

    #[test]
    fn test_parse_real_world_format() {
        // This matches the actual format from the sample data
        let xml = r#"<?xml version="1.0"?>
<?mso-application progid="Excel.Sheet"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet" 
xmlns:o="urn:schemas-microsoft-com:office:office" 
xmlns:x="urn:schemas-microsoft-com:office:excel" 
xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet" 
xmlns:html="http://www.w3.org/TR/REC-html40"> 
<DocumentProperties xmlns="urn:schemas-microsoft-com:office:office"> 
</DocumentProperties> 
<ExcelWorkbook xmlns="urn:schemas-microsoft-com:office:excel"> 
</ExcelWorkbook> 
<Styles> 
</Styles> 
<Worksheet ss:Name="Table1"> 
<Table > 
<Row> 
<Cell><Data ss:Type="String">tipo_evento</Data></Cell> 
<Cell><Data ss:Type="String">data_inizio</Data></Cell> 
<Cell><Data ss:Type="String">data_fine</Data></Cell> 
<Cell><Data ss:Type="String">ora_inizio</Data></Cell> 
<Cell><Data ss:Type="String">ora_fine</Data></Cell> 
<Cell><Data ss:Type="String">tutto_il_giorno</Data></Cell> 
<Cell><Data ss:Type="String">data_inserimento</Data></Cell> 
<Cell><Data ss:Type="String">autore</Data></Cell> 
<Cell><Data ss:Type="String">classe_desc</Data></Cell> 
<Cell><Data ss:Type="String">gruppo_desc</Data></Cell> 
<Cell><Data ss:Type="String">nota</Data></Cell> 
<Cell><Data ss:Type="String">aula</Data></Cell> 
<Cell><Data ss:Type="String">tipo</Data></Cell> 
<Cell><Data ss:Type="String">materia</Data></Cell> 
</Row> 
<Row> 
<Cell><Data ss:Type="String">Nota Agenda</Data></Cell> 
<Cell><Data ss:Type="String">2025-12-01</Data></Cell> 
<Cell><Data ss:Type="String">2025-12-01</Data></Cell> 
<Cell><Data ss:Type="String">12:00:00</Data></Cell> 
<Cell><Data ss:Type="String">13:00:00</Data></Cell> 
<Cell><Data ss:Type="String">NO</Data></Cell> 
<Cell><Data ss:Type="String">2025-11-30 17:59:10</Data></Cell> 
<Cell><Data ss:Type="String">DE STEFANI DEBORA</Data></Cell> 
<Cell><Data ss:Type="String">2C SEC. I GRADO CHIAVENNA</Data></Cell> 
<Cell><Data ss:Type="String"></Data></Cell> 
<Cell><Data ss:Type="String">Ü 15 auf Seite 118</Data></Cell> 
<Cell><Data ss:Type="String">-</Data></Cell> 
<Cell><Data ss:Type="String">compiti</Data></Cell> 
<Cell><Data ss:Type="String">SECONDA LINGUA COMUNITARIA</Data></Cell> 
</Row>
</Table> 
</Worksheet> 
</Workbook>"#;

        let file = create_test_xml_file(xml);
        let entries = parse_excel_xml(file.path()).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "compiti");
        assert_eq!(entries[0].date, "2025-12-01");
        assert_eq!(entries[0].subject, "SECONDA LINGUA COMUNITARIA");
        assert_eq!(entries[0].task, "Ü 15 auf Seite 118");
    }
}
