#!/usr/bin/env python3
"""
Parse homework calendar export and generate a simple web view.
"""

import sys
import json
import xml.etree.ElementTree as ET
from datetime import datetime
from pathlib import Path

try:
    import pandas as pd
except ImportError:
    pd = None

def parse_xml_excel(file_path):
    """Parse Excel XML (SpreadsheetML) format."""
    try:
        tree = ET.parse(file_path)
        root = tree.getroot()
        
        # Define namespaces
        ns = {
            'ss': 'urn:schemas-microsoft-com:office:spreadsheet',
            'html': 'http://www.w3.org/TR/REC-html40'
        }
        
        # Find the worksheet and table
        worksheet = root.find('.//ss:Worksheet', ns)
        if worksheet is None:
            worksheet = root.find('.//Worksheet')
            ns = {}
        
        table = worksheet.find('.//ss:Table', ns) if ns else worksheet.find('.//Table')
        if table is None:
            table = worksheet.find('Table')
        
        rows = table.findall('.//ss:Row', ns) if ns else table.findall('.//Row')
        
        if not rows:
            print("No rows found in the file")
            return None
        
        # Get header row
        header_row = rows[0]
        headers = []
        for cell in header_row.findall('.//ss:Cell', ns) if ns else header_row.findall('.//Cell'):
            data = cell.find('.//ss:Data', ns) if ns else cell.find('.//Data')
            if data is not None and data.text:
                headers.append(data.text.strip())
            else:
                headers.append('')
        
        print(f"Found columns: {headers}")
        
        # Parse data rows
        data_rows = []
        for row in rows[1:]:
            cells = row.findall('.//ss:Cell', ns) if ns else row.findall('.//Cell')
            row_data = []
            for cell in cells:
                data = cell.find('.//ss:Data', ns) if ns else cell.find('.//Data')
                if data is not None and data.text:
                    row_data.append(data.text.strip())
                else:
                    row_data.append('')
            data_rows.append(row_data)
        
        # Create DataFrame if pandas is available, otherwise return as dict
        if pd is not None:
            df = pd.DataFrame(data_rows, columns=headers[:len(data_rows[0])] if data_rows else headers)
            print(f"Successfully loaded Excel file. Shape: {df.shape}")
            print(f"Columns: {list(df.columns)}")
            print("\nFirst few rows:")
            print(df.head())
            return df
        else:
            # Return as list of dicts
            result = []
            for row_data in data_rows:
                row_dict = {}
                for i, header in enumerate(headers):
                    if i < len(row_data):
                        row_dict[header] = row_data[i]
                result.append(row_dict)
            return result
            
    except Exception as e:
        print(f"Error parsing XML Excel file: {e}")
        import traceback
        traceback.print_exc()
        return None

def parse_excel_file(file_path):
    """Parse the Excel file and extract homework data."""
    # Check if it's XML format first
    try:
        with open(file_path, 'rb') as f:
            first_bytes = f.read(100)
            if first_bytes.startswith(b'<?xml') or first_bytes.startswith(b'<Workbook'):
                print("Detected Excel XML format")
                return parse_xml_excel(file_path)
    except:
        pass
    
    # Try pandas with different engines
    if pd is None:
        print("Error: pandas is required for binary Excel files. Install it with: pip install pandas openpyxl xlrd")
        return None
    
    engines = ['openpyxl', 'xlrd']
    
    for engine in engines:
        try:
            print(f"Trying to read with engine: {engine}")
            df = pd.read_excel(file_path, engine=engine)
            print(f"Successfully loaded Excel file. Shape: {df.shape}")
            print(f"Columns: {list(df.columns)}")
            print("\nFirst few rows:")
            print(df.head())
            return df
        except Exception as e:
            print(f"  Failed with {engine}: {e}")
            continue
    
    print(f"Error: Could not read Excel file with any engine")
    return None

def extract_homework(data):
    """Extract homework entries from the dataframe or list of dicts."""
    homework = []
    
    # Check if it's a DataFrame or list of dicts
    if pd is not None and isinstance(data, pd.DataFrame):
        df = data
        is_dataframe = True
    else:
        # Convert list of dicts to DataFrame-like structure
        is_dataframe = False
        if not data:
            return homework
    
    # Common column names that might contain homework info
    if is_dataframe:
        date_columns = [col for col in df.columns if any(keyword in col.lower() for keyword in ['date', 'data', 'giorno', 'day', 'inizio'])]
        subject_columns = [col for col in df.columns if any(keyword in col.lower() for keyword in ['subject', 'materia', 'course', 'corso'])]
        task_columns = [col for col in df.columns if any(keyword in col.lower() for keyword in ['homework', 'compito', 'task', 'description', 'descrizione', 'nota'])]
        tipo_columns = [col for col in df.columns if 'tipo' in col.lower()]
        
        print(f"\nDetected columns:")
        print(f"  Date columns: {date_columns}")
        print(f"  Subject columns: {subject_columns}")
        print(f"  Task columns: {task_columns}")
        print(f"  Type columns: {tipo_columns}")
        
        # Extract rows
        for idx, row in df.iterrows():
            entry = {}
            
            # Include all entry types (compiti, nota, etc.)
            # Get entry type for display
            tipo_col = None
            for col in tipo_columns:
                if col.lower() == 'tipo' and 'evento' not in col.lower():
                    tipo_col = col
                    break
            
            if tipo_col:
                tipo_val = row[tipo_col]
                if pd.notna(tipo_val):
                    entry['type'] = str(tipo_val).strip()
            elif 'tipo' in df.columns:
                tipo_val = row['tipo']
                if pd.notna(tipo_val):
                    entry['type'] = str(tipo_val).strip()
            
            # Get date
            if date_columns:
                date_val = row[date_columns[0]]
                if pd.notna(date_val):
                    if isinstance(date_val, datetime):
                        entry['date'] = date_val.strftime('%Y-%m-%d')
                    elif isinstance(date_val, str):
                        entry['date'] = date_val.split()[0] if ' ' in date_val else date_val
                    else:
                        entry['date'] = str(date_val)
            
            # Get subject
            if subject_columns:
                subject_val = row[subject_columns[0]]
                if pd.notna(subject_val):
                    entry['subject'] = str(subject_val).strip()
            
            # Get task/description
            if task_columns:
                task_parts = []
                for col in task_columns:
                    val = row[col]
                    if pd.notna(val) and str(val).strip():
                        task_parts.append(str(val).strip())
                if task_parts:
                    entry['task'] = ' | '.join(task_parts)
            
            # Only add if we have meaningful data
            if entry and ('task' in entry or 'subject' in entry):
                homework.append(entry)
    else:
        # Handle list of dicts
        for row in data:
            entry = {}
            
            # Include all entry types - get tipo for display
            tipo_val = row.get('tipo', '')
            if tipo_val:
                entry['type'] = str(tipo_val).strip()
            
            # Get date
            date_val = row.get('data_inizio', '')
            if date_val:
                entry['date'] = str(date_val).split()[0] if ' ' in str(date_val) else str(date_val)
            
            # Get subject
            subject_val = row.get('materia', '')
            if subject_val:
                entry['subject'] = str(subject_val).strip()
            
            # Get task
            nota_val = row.get('nota', '')
            if nota_val:
                entry['task'] = str(nota_val).strip()
            
            # Only add if we have meaningful data
            if entry and ('task' in entry or 'subject' in entry):
                homework.append(entry)
    
    return homework

def generate_html(homework_data, output_path):
    """Generate a simple HTML view of the homework."""
    html_content = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Compitutto</title>
    <style>
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
    </style>
</head>
<body>
    <div class="container">
        <h1>Compitutto</h1>
        <div class="stats">
            <span id="total-count">0</span> entries
        </div>
        <div class="homework-list" id="homework-list">
"""
    
    if homework_data:
        # Group by date for better organization
        homework_by_date = {}
        for item in homework_data:
            date = item.get('date', 'No Date')
            if date not in homework_by_date:
                homework_by_date[date] = []
            homework_by_date[date].append(item)
        
        # Sort dates
        sorted_dates = sorted(homework_by_date.keys())
        
        entry_id = 0
        for date in sorted_dates:
            items = homework_by_date[date]
            
            # Date group header
            html_content += f"""
            <div class="date-group">
                <div class="date-header">📅 {date}</div>
"""
            
            for item in items:
                entry_id += 1
                subject = item.get('subject', 'No Subject')
                task = item.get('task', 'No description')
                entry_type = item.get('type', '')
                entry_key = f"entry-{entry_id}"
                
                type_badge = f'<span class="homework-type">{entry_type}</span>' if entry_type else ''
                
                html_content += f"""
                <div class="homework-item" data-entry-id="{entry_id}">
                    <input type="checkbox" class="homework-checkbox" id="{entry_key}" data-entry-id="{entry_id}">
                    <div class="homework-content">
                        <div class="homework-subject">
                            {subject}{type_badge}
                        </div>
                        <div class="homework-task">{task}</div>
                    </div>
                </div>
"""
            
            html_content += """
            </div>
"""
    else:
        html_content += """
            <div class="empty-state">
                <p>No homework entries found.</p>
            </div>
"""
    
    html_content += """
        </div>
    </div>
    <script>
        // Update total count
        document.getElementById('total-count').textContent = document.querySelectorAll('.homework-item').length;
        
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
    </script>
</body>
</html>"""
    
    output_path.write_text(html_content, encoding='utf-8')
    print(f"\n✅ HTML file generated: {output_path}")

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='Parse homework calendar export')
    parser.add_argument('--download', action='store_true', 
                       help='Download export from ClasseViva before parsing')
    parser.add_argument('--file', type=str, 
                       help='Path to Excel file (default: auto-detect latest in data/)')
    args = parser.parse_args()
    
    # Download if requested
    if args.download:
        print("Downloading export from ClasseViva...")
        print("=" * 60)
        try:
            import subprocess
            result = subprocess.run([sys.executable, str(Path(__file__).parent / 'download_export.py')], 
                                 capture_output=True, text=True)
            
            # Always show output for debugging
            if result.stdout:
                print("Download script output:")
                print(result.stdout)
            if result.stderr:
                print("Download script errors:")
                print(result.stderr)
            
            if result.returncode == 0:
                # Find the newly downloaded file
                export_files = list(data_dir.glob("export_*.xls*"))
                if export_files:
                    excel_file = max(export_files, key=lambda f: f.stat().st_mtime)
                    print(f"✅ Using newly downloaded file: {excel_file.name}")
                else:
                    print("⚠️  Warning: Download completed but file not found, trying to use existing file...")
                    print(f"   Checked directory: {data_dir}")
                    print(f"   Files found: {list(data_dir.glob('*'))}")
            else:
                print("=" * 60)
                print(f"❌ Download failed with exit code: {result.returncode}")
                print("   Trying to use existing file...")
                print("=" * 60)
        except Exception as e:
            print("=" * 60)
            print(f"❌ Error during download: {e}")
            import traceback
            traceback.print_exc()
            print("   Trying to use existing file...")
            print("=" * 60)
    
    data_dir = Path(__file__).parent / 'data'
    
    # Find Excel file
    if args.file:
        excel_file = Path(args.file)
    else:
        # Auto-detect latest export file
        export_files = list(data_dir.glob("export_*.xls*"))
        if export_files:
            excel_file = max(export_files, key=lambda f: f.stat().st_mtime)
            print(f"Using file: {excel_file.name}")
        else:
            excel_file = data_dir / 'export_26122025-2018.xls'  # fallback
    
    if not excel_file.exists():
        print(f"Error: File not found: {excel_file}")
        print("Run with --download to fetch from ClasseViva, or place export file in data/ directory")
        sys.exit(1)
    
    print(f"Reading Excel file: {excel_file}")
    df = parse_excel_file(excel_file)
    
    if df is None:
        sys.exit(1)
    
    print("\nExtracting homework data...")
    homework = extract_homework(df)
    
    print(f"\nFound {len(homework)} homework entries")
    
    # Save as JSON for reference
    json_path = Path(__file__).parent / 'homework.json'
    with open(json_path, 'w', encoding='utf-8') as f:
        json.dump(homework, f, indent=2, ensure_ascii=False)
    print(f"✅ JSON data saved: {json_path}")
    
    # Generate HTML
    html_path = Path(__file__).parent / 'index.html'
    generate_html(homework, html_path)
    
    print(f"\n🎉 Done! Open {html_path} in your browser to view the homework calendar.")

if __name__ == '__main__':
    main()

