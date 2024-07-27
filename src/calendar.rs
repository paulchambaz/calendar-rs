use anyhow::{anyhow, Result};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use ical::parser::ical::component::{IcalCalendar, IcalEvent};
use ical::property::Property;
use rand::Rng;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;

pub struct Calendar {
    pub name: String,
    pub path: PathBuf,
    pub events: Vec<Event>,
}

pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub location: Option<String>,
    pub description: Option<String>,
}

pub fn generate_random_events(count: usize) -> Vec<Event> {
    let mut rng = rand::thread_rng();
    let now = Local::now().naive_local();

    let event_names = vec![
        "Team Meeting",
        "Project Deadline",
        "Client Call",
        "Lunch Break",
        "Code Review",
        "Gym Session",
        "Doctor's Appointment",
        "Birthday Party",
        "Conference Call",
        "Presentation Prep",
    ];

    let locations = vec![
        "Office",
        "Home",
        "Coffee Shop",
        "Conference Room",
        "Gym",
        "Doctor's Office",
        "Restaurant",
        "Park",
        "Client's Office",
    ];

    (0..count)
        .map(|_| {
            let days_offset = rng.gen_range(-30..30);
            let hours_offset = rng.gen_range(0..24);
            let start = now + Duration::days(days_offset) + Duration::hours(hours_offset);
            let duration = Duration::hours(rng.gen_range(1..4));
            let end = start + duration;

            Event {
                id: Uuid::new_v4(),
                name: event_names[rng.gen_range(0..event_names.len())].to_string(),
                start,
                end,
                location: if rng.gen_bool(0.7) {
                    Some(locations[rng.gen_range(0..locations.len())].to_string())
                } else {
                    None
                },
                description: if rng.gen_bool(0.5) {
                    Some(format!(
                        "Random description for event {}",
                        rng.gen_range(1..100)
                    ))
                } else {
                    None
                },
            }
        })
        .collect()
}

impl Calendar {
    pub fn new(name: String, path: PathBuf) -> Result<Self> {
        let mut calendar = Calendar {
            name,
            path,
            events: Vec::new(),
        };
        calendar.load()?;
        Ok(calendar)
    }

    fn load(&mut self) -> Result<()> {
        let mut file = File::open(&self.path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let ical = ical::IcalParser::new(contents.as_bytes())
            .next()
            .ok_or_else(|| anyhow!("Failed to parse calendar file"))??;

        self.events = ical
            .events
            .into_iter()
            .filter_map(|e| Event::from_ical_event(e).ok())
            .collect();

        Ok(())
    }

    pub fn add_event(&mut self, event: Event) -> Result<()> {
        self.events.push(event);
        // self.save();
        Ok(())
    }

    pub fn remove_event(&mut self, id: Uuid) -> Result<()> {
        self.events.retain(|e| e.id != id);
        // self.save();
        Ok(())
    }

    pub fn edit_event(&mut self, id: Uuid, new_event: Event) -> Result<()> {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            *event = new_event;
            // self.save()?;
        }
        Ok(())
    }

    pub fn get_event(&self, id: Uuid) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }

    pub fn list_events(&self, from: NaiveDate, to: NaiveDate) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.start.date() >= from && e.end.date() <= to)
            .collect()
    }

    // fn save(&self) -> Result<()> {
    //     let mut ical = IcalCalendar::new();
    //     ical.properties.push(Property {
    //         name: "VERSION".to_string(),
    //         params: None,
    //         value: Some("2.0".to_string()),
    //     });
    //
    //     for event in &self.events {
    //         ical.events.push(event.to_ical_event());
    //     }
    //
    //     let ical_string = ical::write_calendar(&ical)
    //         .map_err(|e| anyhow!("Failed to serialize calendar: {}", e))?;
    //
    //     let mut file = OpenOptions::new()
    //         .write(true)
    //         .truncate(true)
    //         .open(&self.path)?;
    //
    //     file.write_all(ical_string.as_bytes())?;
    //     Ok(())
    // }
}

impl Event {
    pub fn new(
        summary: String,
        start: NaiveDateTime,
        end: NaiveDateTime,
        location: Option<String>,
        description: Option<String>,
    ) -> Self {
        Event {
            id: Uuid::new_v4(),
            name: summary,
            start,
            end,
            location,
            description,
        }
    }

    fn from_ical_event(event: IcalEvent) -> Result<Self> {
        let id = event
            .properties
            .iter()
            .find(|p| p.name == "UID")
            .and_then(|p| p.value.as_ref())
            .and_then(|v| Uuid::parse_str(v).ok())
            .ok_or_else(|| anyhow!("Event is missing UID"))?;

        let summary = event
            .properties
            .iter()
            .find(|p| p.name == "SUMMARY")
            .and_then(|p| p.value.as_ref())
            .ok_or_else(|| anyhow!("Event is missing SUMMARY"))?
            .to_string();

        let start = event
            .properties
            .iter()
            .find(|p| p.name == "DTSTART")
            .and_then(|p| p.value.as_ref())
            .and_then(|v| NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok())
            .ok_or_else(|| anyhow!("Event is missing or has invalid DTSTART"))?;

        let end = event
            .properties
            .iter()
            .find(|p| p.name == "DTEND")
            .and_then(|p| p.value.as_ref())
            .and_then(|v| NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok())
            .ok_or_else(|| anyhow!("Event is missing or has invalid DTEND"))?;

        let location = event
            .properties
            .iter()
            .find(|p| p.name == "LOCATION")
            .and_then(|p| p.value.as_ref())
            .map(|v| v.to_string());

        let description = event
            .properties
            .iter()
            .find(|p| p.name == "DESCRIPTION")
            .and_then(|p| p.value.as_ref())
            .map(|v| v.to_string());

        Ok(Event {
            id,
            name: summary,
            start,
            end,
            location,
            description,
        })
    }

    fn to_ical_event(&self) -> IcalEvent {
        let mut event = IcalEvent::new();
        event.properties.push(Property {
            name: "UID".to_string(),
            params: None,
            value: Some(self.id.to_string()),
        });
        event.properties.push(Property {
            name: "SUMMARY".to_string(),
            params: None,
            value: Some(self.name.clone()),
        });
        event.properties.push(Property {
            name: "DTSTART".to_string(),
            params: None,
            value: Some(self.start.format("%Y%m%dT%H%M%S").to_string()),
        });
        event.properties.push(Property {
            name: "DTEND".to_string(),
            params: None,
            value: Some(self.end.format("%Y%m%dT%H%M%S").to_string()),
        });
        if let Some(location) = &self.location {
            event.properties.push(Property {
                name: "LOCATION".to_string(),
                params: None,
                value: Some(location.clone()),
            });
        }
        if let Some(description) = &self.description {
            event.properties.push(Property {
                name: "DESCRIPTION".to_string(),
                params: None,
                value: Some(description.clone()),
            });
        }
        event
    }
}
