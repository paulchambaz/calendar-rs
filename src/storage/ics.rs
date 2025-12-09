use std::fs;
use std::path::Path;
use chrono::{NaiveDateTime, NaiveDate};
use crate::calendar::events::Event;
use crate::calendar::recurring::RecurrenceRule;

/// Load all events from an ICS file
pub fn load_events_from_file(file_path: &Path) -> Result<Vec<Event>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Cannot read file {}: {}", file_path.display(), e))?;
    
    parse_ics_content(&content)
}

/// Parse ICS content and return expanded events
pub fn parse_ics_content(content: &str) -> Result<Vec<Event>, String> {
    let mut events = Vec::new();
    let mut current_event = None;
    let mut in_event = false;

    for line in content.lines() {
        let line = line.trim();
        
        match line {
            "BEGIN:VEVENT" => {
                in_event = true;
                current_event = Some(IcsEvent::new());
            }
            "END:VEVENT" => {
                if let Some(ics_event) = current_event.take() {
                    let mut event_instances = ics_event.to_events()?;
                    events.append(&mut event_instances);
                }
                in_event = false;
            }
            _ if in_event => {
                if let Some(ref mut event) = current_event {
                    event.parse_line(line)?;
                }
            }
            _ => {} // Ignore non-event lines
        }
    }
    
    Ok(events)
}

/// Save an event to ICS format
pub fn save_event_to_file(event: &Event, file_path: &Path) -> Result<(), String> {
    let ics_content = format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//calendar-rs//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:{}\r\n\
         DTSTART:{}\r\n\
         DTEND:{}\r\n\
         SUMMARY:{}\r\n\
         {}\
         {}\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
        event.id,
        format_ics_datetime(event.start),
        format_ics_datetime(event.end),
        escape_ics_text(&event.name),
        event.location.as_ref().map_or(String::new(), |loc| format!("LOCATION:{}\r\n", escape_ics_text(loc))),
        event.description.as_ref().map_or(String::new(), |desc| format!("DESCRIPTION:{}\r\n", escape_ics_text(desc))),
    );

    fs::write(file_path, ics_content)
        .map_err(|e| format!("Cannot write to file {}: {}", file_path.display(), e))
}

/// Temporary struct for parsing ICS events before conversion
struct IcsEvent {
    uid: Option<String>,
    summary: Option<String>,
    dtstart: Option<NaiveDateTime>,
    dtend: Option<NaiveDateTime>,
    location: Option<String>,
    description: Option<String>,
    rrule: Option<String>,
}

impl IcsEvent {
    fn new() -> Self {
        IcsEvent {
            uid: None,
            summary: None,
            dtstart: None,
            dtend: None,
            location: None,
            description: None,
            rrule: None,
        }
    }

    fn parse_line(&mut self, line: &str) -> Result<(), String> {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Ok(()); // Skip malformed lines
        }

        let key_full = parts[0];
        let value = parts[1];
        
        // Extract the main key (before any parameters like ;TZID=)
        let key = key_full.split(';').next().unwrap_or(key_full);

        match key {
            "UID" => self.uid = Some(value.to_string()),
            "SUMMARY" => self.summary = Some(unescape_ics_text(value)),
            "LOCATION" => self.location = Some(unescape_ics_text(value)),
            "DESCRIPTION" => self.description = Some(unescape_ics_text(value)),
            "RRULE" => self.rrule = Some(value.to_string()),
            "DTSTART" => self.dtstart = Some(parse_ics_datetime(value)?),
            "DTEND" => self.dtend = Some(parse_ics_datetime(value)?),
            _ => {} // Ignore unknown fields
        }

        Ok(())
    }

    fn to_events(self) -> Result<Vec<Event>, String> {
        let uid = self.uid.ok_or("Event missing UID")?;
        let summary = self.summary.ok_or("Event missing SUMMARY")?;
        let dtstart = self.dtstart.ok_or("Event missing DTSTART")?;
        
        // Default end time to 1 hour after start if not specified
        let dtend = self.dtend.unwrap_or_else(|| dtstart + chrono::Duration::hours(1));

        let base_event = Event::with_id(
            uid,
            summary,
            dtstart,
            dtend,
            self.location,
            self.description,
        );

        // If there's a recurrence rule, expand the event
        if let Some(rrule_str) = self.rrule {
            let rrule = RecurrenceRule::from_ics_string(&rrule_str)?;
            Ok(rrule.expand_event(&base_event))
        } else {
            Ok(vec![base_event])
        }
    }
}

/// Parse various ICS datetime formats
fn parse_ics_datetime(value: &str) -> Result<NaiveDateTime, String> {
    // YYYYMMDDTHHMMSSZ (UTC)
    if value.ends_with('Z') && value.len() == 16 {
        let date_part = &value[0..8];
        let time_part = &value[9..15];
        parse_date_time_parts(date_part, time_part)
    }
    // YYYYMMDDTHHMMSS (local time)
    else if value.len() == 15 && value.contains('T') {
        let date_part = &value[0..8];
        let time_part = &value[9..15];
        parse_date_time_parts(date_part, time_part)
    }
    // YYYYMMDD (date only, assume start of day)
    else if value.len() == 8 {
        parse_date_time_parts(value, "000000")
    }
    else {
        Err(format!("Unsupported datetime format: {}", value))
    }
}

fn parse_date_time_parts(date_str: &str, time_str: &str) -> Result<NaiveDateTime, String> {
    let year: i32 = date_str[0..4].parse().map_err(|_| "Invalid year")?;
    let month: u32 = date_str[4..6].parse().map_err(|_| "Invalid month")?;
    let day: u32 = date_str[6..8].parse().map_err(|_| "Invalid day")?;
    
    let hour: u32 = time_str[0..2].parse().map_err(|_| "Invalid hour")?;
    let minute: u32 = time_str[2..4].parse().map_err(|_| "Invalid minute")?;
    let second: u32 = time_str[4..6].parse().map_err(|_| "Invalid second")?;
    
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or("Invalid date")?;
    
    date.and_hms_opt(hour, minute, second)
        .ok_or("Invalid time".to_string())
}

/// Format datetime for ICS output
fn format_ics_datetime(datetime: NaiveDateTime) -> String {
    datetime.format("%Y%m%dT%H%M%S").to_string()
}

/// Escape special characters for ICS text fields
fn escape_ics_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

/// Unescape ICS text fields
fn unescape_ics_text(text: &str) -> String {
    text.replace("\\\\", "\\")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\n", "\n")
}
