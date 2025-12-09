use std::fs;
use std::path::{Path, PathBuf};

/// Get the main calendars directory in the user's home folder
pub fn calendars_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir()
        .ok_or("Cannot find home directory")?;
    
    let calendar_dir = home.join(".calendars");
    
    // Create the directory if it doesn't exist
    if !calendar_dir.exists() {
        fs::create_dir_all(&calendar_dir)
            .map_err(|e| format!("Cannot create calendar directory: {}", e))?;
    }
    
    Ok(calendar_dir)
}

/// List all available calendar names
pub fn list_calendar_names() -> Result<Vec<String>, String> {
    let calendar_dir = calendars_dir()?;
    let mut names = Vec::new();
    
    let entries = fs::read_dir(calendar_dir)
        .map_err(|e| format!("Cannot read calendar directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot read directory entry: {}", e))?;
        
        if entry.file_type().map_err(|e| format!("Cannot check file type: {}", e))?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                // Only include directories that have .ics files or subdirectories
                if calendar_has_events(&entry.path())? {
                    names.push(name.to_string());
                }
            }
        }
    }
    
    names.sort();
    Ok(names)
}

/// Check if a calendar directory contains any events
fn calendar_has_events(dir: &Path) -> Result<bool, String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "ics") {
            return Ok(true);
        }
        
        if path.is_dir() && calendar_has_events(&path)? {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Get the path to a specific calendar directory
pub fn calendar_path(name: &str) -> Result<PathBuf, String> {
    let path = calendars_dir()?.join(name);
    
    if !path.exists() {
        return Err(format!("Calendar '{}' does not exist", name));
    }
    
    if !path.is_dir() {
        return Err(format!("'{}' is not a valid calendar directory", name));
    }
    
    Ok(path)
}

/// Create a new calendar directory
pub fn create_calendar(name: &str) -> Result<PathBuf, String> {
    let calendar_dir = calendars_dir()?.join(name);
    
    if calendar_dir.exists() {
        return Err(format!("Calendar '{}' already exists", name));
    }
    
    fs::create_dir_all(&calendar_dir)
        .map_err(|e| format!("Cannot create calendar directory: {}", e))?;
    Ok(calendar_dir)
}

/// Create the default "personal" calendar if it doesn't exist
pub fn ensure_personal_calendar() -> Result<PathBuf, String> {
    let personal_path = calendars_dir()?.join("personal");
    
    if !personal_path.exists() {
        fs::create_dir_all(&personal_path)
            .map_err(|e| format!("Cannot create personal calendar: {}", e))?;
    }
    
    Ok(personal_path)
}

/// List all .ics files in a calendar directory (recursively)
pub fn list_ics_files(calendar_path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_ics_files(calendar_path, &mut files)?;
    Ok(files)
}

/// Recursively collect .ics files from a directory
fn collect_ics_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "ics") {
            files.push(path);
        } else if path.is_dir() {
            collect_ics_files(&path, files)?;
        }
    }
    
    Ok(())
}

/// Delete an .ics file by filename (searches recursively)
pub fn delete_ics_file(calendar_path: &Path, filename: &str) -> Result<(), String> {
    let target_filename = format!("{}.ics", filename);
    let file_path = find_ics_file(calendar_path, &target_filename)?;
    
    fs::remove_file(file_path)
        .map_err(|e| format!("Cannot delete file: {}", e))?;
    Ok(())
}

/// Find an .ics file by name in the calendar directory (recursively)
fn find_ics_file(dir: &Path, target_filename: &str) -> Result<PathBuf, String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(target_filename) {
            return Ok(path);
        } else if path.is_dir() {
            if let Ok(found) = find_ics_file(&path, target_filename) {
                return Ok(found);
            }
        }
    }
    
    Err(format!("Event file '{}' not found", target_filename.trim_end_matches(".ics")))
}

/// Find the file path for an event by ID (searches recursively)
pub fn find_event_file(calendar_path: &Path, event_id: &str) -> Result<PathBuf, String> {
    let target_filename = format!("{}.ics", event_id);
    find_ics_file(calendar_path, &target_filename)
}
