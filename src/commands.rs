use chrono::{Duration, Local};
use temporis;
use std::io::{self, Write};
use crate::storage::files;
use crate::storage::ics;
use crate::calendar::events::{Event, EventList};
use crate::display::{list, calendar, event as event_display};

/// Handle the list command
pub fn handle_list(
    query: Vec<String>, 
    calendar: Option<String>, 
    from: Option<String>,
    to: Option<String>,
    limit: Option<usize>,
    show_ids: bool
) -> Result<(), String> {
    let events = load_events_from_calendar(calendar)?;
    
    let from_date = if let Some(from_str) = from {
        parse_date(&from_str)?
    } else {
        chrono::Local::now().date_naive()
    };
    
    let to_date = if let Some(to_str) = to {
        parse_date(&to_str)?
    } else {
        from_date + chrono::Duration::days(3000)
    };
    
    let query_string = if query.is_empty() { 
        None 
    } else { 
        Some(query.join(" ")) 
    };
    
    let options = list::ListOptions {
        show_ids,
        from_date,
        to_date,
        query: query_string,
        limit,
    };
    
    list::show_events(&events, &options);
    Ok(())
}

/// Handle the add command
pub fn handle_add(
    name: Vec<String>,
    at: String,
    to: Option<String>,
    calendar: Option<String>,
    location: Option<String>,
    description: Option<String>,
    repeat: Option<String>,
    every: Option<u32>,
    until: Option<String>,
) -> Result<(), String> {
    let calendar_name = calendar.unwrap_or_else(|| "personal".to_string());
    let event_name = name.join(" ");
    
    if event_name.trim().is_empty() {
        return Err("Event name cannot be empty".to_string());
    }
    
    // For now, ignore repeat/every/until (recurring functionality)
    if repeat.is_some() || every.is_some() || until.is_some() {
        return Err("Recurring events not yet implemented in new version".to_string());
    }
    
    // Rest stays the same...
    ensure_calendar_exists(&calendar_name)?;
    
    let start = parse_datetime(&at)?;
    let end = if let Some(to_str) = to {
        parse_datetime(&to_str)?
    } else {
        start + Duration::hours(1)
    };
    
    if end <= start {
        return Err("End time must be after start time".to_string());
    }
    
    let event = Event::new_with_details(event_name, start, end, location, description);
    save_event_to_calendar(&event, &calendar_name)?;
    
    println!("Event added successfully");
    Ok(())
}

/// Handle the edit command
pub fn handle_edit(
    id: String,
    calendar_name: String,
    name: Option<String>,
    at: Option<String>,
    to: Option<String>,
    location: Option<String>,
    description: Option<String>,
) -> Result<(), String> {
    let calendar_path = files::calendar_path(&calendar_name)?;
    let mut events = load_events_from_calendar(Some(calendar_name.clone()))?;
    
    // Find the event
    let event = events.find_by_id_mut(&id)
        .ok_or_else(|| format!("Event with ID '{}' not found", id))?;
    
    // Parse new times if provided
    let new_start = if let Some(at_str) = at {
        Some(parse_datetime(&at_str)?)
    } else {
        None
    };
    
    let new_end = if let Some(to_str) = to {
        Some(parse_datetime(&to_str)?)
    } else {
        None
    };
    
    // Validate times
    if let (Some(start), Some(end)) = (new_start, new_end) {
        if end <= start {
            return Err("End time must be after start time".to_string());
        }
    }
    
    // Update the event
    event.update(name, new_start, new_end, location, description);
    
    // Find the original file location and save back there
    let original_file_path = files::find_event_file(&calendar_path, &id)?;
    ics::save_event_to_file(event, &original_file_path)?;
    
    println!("Event updated successfully");
    Ok(())
}

/// Handle the delete command
pub fn handle_delete(id: String, calendar_name: String, force: bool) -> Result<(), String> {
    let calendar_path = files::calendar_path(&calendar_name)?;
    let events = load_events_from_calendar(Some(calendar_name))?;
    
    // Find the event to confirm what we're deleting
    let event = events.find_by_id(&id)
        .ok_or_else(|| format!("Event with ID '{}' not found", id))?;
    
    // Check if this is a recurring event instance (has a dash followed by numbers)
    let base_id = if id.contains('-') {
        let parts: Vec<&str> = id.split('-').collect();
        // If the last part is a number, this is likely a recurring instance
        if parts.len() > 4 && parts.last().unwrap().parse::<u32>().is_ok() {
            // Reconstruct the base ID (all parts except the last)
            parts[0..parts.len()-1].join("-")
        } else {
            id.clone()
        }
    } else {
        id.clone()
    };
    
    // Confirm deletion unless --force
    if !force {
        if base_id != id {
            println!("This is instance #{} of a recurring event.", id.split('-').last().unwrap());
            println!("Deleting will remove the ENTIRE recurring series:");
        }
        event_display::show_event_for_deletion(event);
        print!("Are you sure? (y/N) ");
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| format!("IO error: {}", e))?;
        
        if input.trim().to_lowercase() != "y" {
            println!("Deletion cancelled");
            return Ok(());
        }
    }
    
    // Delete using the base ID
    files::delete_ics_file(&calendar_path, &base_id)?;
    
    if base_id != id {
        println!("Entire recurring series deleted successfully");
    } else {
        println!("Event deleted successfully");
    }
    Ok(())
}

/// Handle the show command
pub fn handle_show(id: String, calendar_name: String) -> Result<(), String> {
    let events = load_events_from_calendar(Some(calendar_name))?;
    
    let event = events.find_by_id(&id)
        .ok_or_else(|| format!("Event with ID '{}' not found", id))?;
    
    event_display::show_event_details(event);
    Ok(())
}

/// Handle the view command
pub fn handle_view(
    date: Option<String>,
    mode: crate::args::ViewMode,
    calendar_name: Option<String>,
    number: Option<u32>,
) -> Result<(), String> {
    let events = load_events_from_calendar(calendar_name)?;
    
    // Parse the target date
    let target_date = if let Some(date_str) = date {
        parse_date(&date_str)?
    } else {
        Local::now().date_naive()
    };
    
    let options = calendar::CalendarOptions {
        date: target_date,
        mode, // Use the ViewMode directly, no conversion needed
        number: number.unwrap_or(1),
    };
    
    calendar::show_calendar(&events, &options);
    Ok(())
}

/// Load all events from a calendar (or all calendars if None)
fn load_events_from_calendar(calendar_name: Option<String>) -> Result<EventList, String> {
    if let Some(name) = calendar_name {
        // Load from specific calendar
        let calendar_path = files::calendar_path(&name)?;
        let ics_files = files::list_ics_files(&calendar_path)?;
        
        let mut all_events = Vec::new();
        for file_path in ics_files {
            let mut events = ics::load_events_from_file(&file_path)?;
            all_events.append(&mut events);
        }
        
        Ok(EventList::from_events(all_events))
    } else {
        // Load from all calendars
        let calendar_names = files::list_calendar_names()?;
        let mut all_events = Vec::new();
        
        for name in calendar_names {
            let calendar_path = files::calendar_path(&name)?;
            let ics_files = files::list_ics_files(&calendar_path)?;
            
            for file_path in ics_files {
                let mut events = ics::load_events_from_file(&file_path)?;
                all_events.append(&mut events);
            }
        }
        
        Ok(EventList::from_events(all_events))
    }
}

/// Save an event to a calendar
fn save_event_to_calendar(event: &Event, calendar_name: &str) -> Result<(), String> {
    let calendar_path = files::calendar_path(calendar_name)?;
    let file_path = calendar_path.join(format!("{}.ics", event.id));
    ics::save_event_to_file(event, &file_path)
}

/// Ensure a calendar exists, creating it if needed
fn ensure_calendar_exists(name: &str) -> Result<(), String> {
    if name == "personal" {
        files::ensure_personal_calendar()?;
    } else {
        // Try to get the path, create if it doesn't exist
        if files::calendar_path(name).is_err() {
            files::create_calendar(name)?;
        }
    }
    Ok(())
}

/// Parse a date string using temporis
fn parse_date(date_str: &str) -> Result<chrono::NaiveDate, String> {
    temporis::parse_date(date_str)
        .map_err(|e| format!("Invalid date '{}': {}", date_str, e))
}

/// Parse a datetime string (date@time format)
fn parse_datetime(datetime_str: &str) -> Result<chrono::NaiveDateTime, String> {
    if datetime_str.contains('@') {
        let parts: Vec<&str> = datetime_str.split('@').collect();
        if parts.len() != 2 {
            return Err("Datetime must be in 'date@time' format".to_string());
        }
        
        let date = parse_date(parts[0])?;
        let time = parse_time(parts[1])?;
        
        Ok(date.and_time(time))
    } else {
        Err("Datetime must contain '@' separator (e.g., 'tomorrow@2pm')".to_string())
    }
}

/// Parse a time string
fn parse_time(time_str: &str) -> Result<chrono::NaiveTime, String> {
    // Try HH:MM format first
    if let Ok(time) = chrono::NaiveTime::parse_from_str(time_str, "%H:%M") {
        return Ok(time);
    }
    
    // Try single digit hour
    if let Ok(hour) = time_str.parse::<u32>() {
        if hour < 24 {
            return chrono::NaiveTime::from_hms_opt(hour, 0, 0)
                .ok_or("Invalid hour".to_string());
        }
    }
    
    Err(format!("Invalid time format: '{}'. Use HH:MM or just hour", time_str))
}
