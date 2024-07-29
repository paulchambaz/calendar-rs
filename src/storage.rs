use crate::calendar::{Calendar, Event};
use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use std::fs::{self, File};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use uuid::Uuid;

pub fn list_calendars() -> Result<Vec<String>> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    let calendar_dir = home_dir.join(".calendars");
    let mut valid_calendars = Vec::new();

    for entry in fs::read_dir(calendar_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let calendar_path = entry.path();
            // Check if this directory contains at least one subdirectory
            if fs::read_dir(&calendar_path)?
                .filter_map(Result::ok)
                .any(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            {
                if let Some(name) = calendar_path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        valid_calendars.push(name_str.to_string());
                    }
                }
            }
        }
    }

    Ok(valid_calendars)
}

pub fn create_personal() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    let personal_dir = home_dir.join(".calendars").join("personal");
    fs::create_dir_all(&personal_dir)?;

    let uuid = Uuid::new_v4();
    let uuid_dir = personal_dir.join(uuid.to_string());
    fs::create_dir_all(&uuid_dir)?;

    Ok(())
}

pub fn load_calendars() -> Result<Vec<Calendar>> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    let calendar_dir = home_dir.join(".calendars");

    let mut calendars = Vec::new();

    for entry in fs::read_dir(calendar_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let calendar_path = entry.path();

            calendars.push(read_calendar(&calendar_path)?);
        }
    }

    Ok(calendars)
}

pub fn load_calendar(calendar_name: &str) -> Result<Calendar> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    let calendar_path = home_dir.join(".calendars").join(calendar_name);

    if !calendar_path.is_dir() {
        return Err(anyhow!("Calendar '{}' not found", calendar_name));
    }

    read_calendar(&calendar_path)
}

pub fn read_calendar(path: &Path) -> Result<Calendar> {
    let mut calendar = Calendar {
        path: path.to_path_buf(),
        events: Vec::new(),
    };
    for subcalendar in fs::read_dir(path).context("Failed to read directory")? {
        let subcalendar = subcalendar.context("Failed to read subdirectory entry")?;
        if subcalendar
            .file_type()
            .context("Failed to get file type")?
            .is_dir()
        {
            calendar.path = subcalendar.path();
            for entry in fs::read_dir(subcalendar.path()).context("Failed to read subdirectory")? {
                let entry = entry.context("Failed to read directory entry")?;
                if entry
                    .file_type()
                    .context("Failed to get file type")?
                    .is_file()
                    && entry.path().extension().map_or(false, |ext| ext == "ics")
                {
                    let event = read_event(&entry.path()).context("Failed to read event")?;
                    calendar.events.push(event);
                }
            }
        }
    }
    Ok(calendar)
}

fn read_event(path: &Path) -> Result<Event> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut event = Event {
        id: Uuid::new_v4(),
        name: String::new(),
        start: Utc::now().naive_utc(),
        end: Utc::now().naive_utc(),
        location: None,
        description: None,
    };
    let mut in_event = false;
    let mut timezone: Option<String> = None;

    for line in reader.lines() {
        let line = line?;
        match line.as_str() {
            "BEGIN:VEVENT" => in_event = true,
            "END:VEVENT" => break,
            _ if !in_event => continue,
            _ => {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let (key, value) = (parts[0], parts[1]);
                    let key_parts: Vec<&str> = key.split(';').collect();
                    let main_key = key_parts[0];

                    match main_key {
                        "UID" => event.id = Uuid::parse_str(value)?,
                        "SUMMARY" => event.name = value.to_string(),
                        "LOCATION" => event.location = Some(value.to_string()),
                        "DESCRIPTION" => event.description = Some(value.to_string()),
                        "DTSTART" | "DTEND" => {
                            let (datetime, tz) = parse_datetime(key, value, &timezone)?;
                            timezone = tz;
                            if main_key == "DTSTART" {
                                event.start = datetime;
                            } else {
                                event.end = datetime;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(event)
}

fn parse_datetime(
    key: &str,
    value: &str,
    timezone: &Option<String>,
) -> Result<(NaiveDateTime, Option<String>)> {
    let key_parts: Vec<&str> = key.split(';').collect();
    let tz = if key_parts.len() > 1 {
        key_parts[1].split('=').nth(1).map(String::from)
    } else {
        None
    };

    let datetime = if value.contains('T') {
        NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S")?
    } else {
        NaiveDate::parse_from_str(value, "%Y%m%d")?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow::anyhow!("Failed to create NaiveDateTime"))?
    };

    Ok((datetime, tz.or_else(|| timezone.clone())))
}

pub fn write_event(calendar_path: &Path, event: &Event) -> Result<()> {
    let filename = format!("{}.ics", event.id);
    let file_path = calendar_path.join(filename);
    let mut file = File::create(file_path)?;

    let ics_content = format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//paulchambaz//calendar-rs 1.0.2//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:{}\r\n\
         DTSTART:{}\r\n\
         DTEND:{}\r\n\
         SUMMARY:{}\r\n\
         {}\
         {}\
         BEGIN:VALARM\r\n\
         ACTION:DISPLAY\r\n\
         TRIGGER:-PT10M\r\n\
         END:VALARM\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
        event.id,
        event.start.format("%Y%m%dT%H%M%S").to_string(),
        event.end.format("%Y%m%dT%H%M%S").to_string(),
        event.name,
        event
            .location
            .as_ref()
            .map_or(String::new(), |loc| format!("LOCATION:{}\r\n", loc)),
        event
            .description
            .as_ref()
            .map_or(String::new(), |desc| format!("DESCRIPTION:{}\r\n", desc)),
    );

    file.write_all(ics_content.as_bytes())?;
    Ok(())
}

pub fn delete_event(calendar_path: &Path, event_id: Uuid) -> Result<()> {
    let filename = format!("{}.ics", event_id);
    let file_path = calendar_path.join(filename);

    if file_path.exists() {
        fs::remove_file(&file_path)?;
        Ok(())
    } else {
        Err(anyhow!("Event file not found in the calendar directory"))
    }
}
