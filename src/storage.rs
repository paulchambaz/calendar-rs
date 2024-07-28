use crate::calendar::{Calendar, Event};
use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use uuid::Uuid;

pub fn load_calendars() -> Result<Vec<Calendar>> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    let calendar_dir = home_dir.join(".calendars");

    let mut calendars = Vec::new();

    for entry in fs::read_dir(calendar_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let calendar_name = entry.file_name();
            let calendar_path = entry.path();

            calendars.push(read_calendar(
                &calendar_name.to_string_lossy(),
                &calendar_path,
            )?);
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

    read_calendar(calendar_name, &calendar_path)
}

pub fn read_calendar(name: &str, path: &Path) -> Result<Calendar> {
    let mut calendar = Calendar {
        name: name.to_string(),
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
        NaiveDate::parse_from_str(value, "%Y%m%d")?.and_hms(0, 0, 0)
    };

    Ok((datetime, tz.or_else(|| timezone.clone())))
}

// pub fn write_event(calendar_path: &Path, event: &Event) -> Result<()> {
//     let subcalendar_path = calendar_path.join(event.id.to_string());
//     fs::create_dir_all(&subcalendar_path)?;
//
//     let event_path = subcalendar_path.join(format!("{}.ics", event.id));
//     let ical_event = event.to_ical_event();
//     let mut ical_calendar = ical::parser::ical::component::IcalCalendar::new();
//     ical_calendar.events.push(ical_event);
//
//     let ical_string = ical::generator::generate_calendar(&ical_calendar)?;
//     fs::write(event_path, ical_string)?;
//
//     Ok(())
// }

// pub fn delete_event(calendar_path: &Path, event_id: Uuid) -> Result<()> {
//     let subcalendar_path = calendar_path.join(event_id.to_string());
//     let event_path = subcalendar_path.join(format!("{}.ics", event_id));
//
//     if event_path.exists() {
//         fs::remove_file(event_path)?;
//         // If the subcalendar directory is empty after deleting the event, remove it
//         if fs::read_dir(&subcalendar_path)?.next().is_none() {
//             fs::remove_dir(subcalendar_path)?;
//         }
//         Ok(())
//     } else {
//         Err(anyhow!("Event file not found"))
//     }
// }
