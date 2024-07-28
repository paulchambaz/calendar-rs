use anyhow::{anyhow, Result};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use ical::parser::ical::component::{IcalCalendar, IcalEvent};
use ical::property::Property;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;

use crate::storage;

#[derive(Debug, Clone)]
pub struct Calendar {
    pub name: String,
    pub path: PathBuf,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub location: Option<String>,
    pub description: Option<String>,
}

pub fn load_all() -> Result<Vec<Calendar>> {
    storage::load_calendars()
}

pub fn load(name: &str) -> Result<Calendar> {
    storage::load_calendar(name)
}

impl Calendar {
    pub fn add_event(
        &mut self,
        name: String,
        start: NaiveDateTime,
        end: NaiveDateTime,
        location: Option<String>,
        description: Option<String>,
    ) -> Result<()> {
        let event = Event::new(name, start, end, location, description);
        self.events.push(event);

        todo!("write event to disk");
        Ok(())
    }

    pub fn remove_event(&mut self, event_id: Uuid) -> Result<()> {
        let event = self
            .get_event(event_id)
            .ok_or_else(|| anyhow!("Could not find event with this uuid"))?;

        todo!("delete event from disk");
        Ok(())
    }

    pub fn edit_event(
        &mut self,
        id: Uuid,
        name: Option<String>,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        location: Option<String>,
        description: Option<String>,
    ) -> Result<()> {
        let event = self
            .get_event_mut(id)
            .ok_or_else(|| anyhow!("Could not find event with this uuid"))?;
        if let Some(new_name) = name {
            event.name = new_name;
        }
        if let Some(new_start) = start {
            event.start = new_start;
        }
        if let Some(new_end) = end {
            event.end = new_end;
        }
        if let Some(new_location) = location {
            event.location = Some(new_location);
        }
        if let Some(new_description) = description {
            event.description = Some(new_description);
        }

        todo!("write event to disk");
        Ok(())
    }

    pub fn get_event(&self, id: Uuid) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }

    fn get_event_mut(&mut self, id: Uuid) -> Option<&mut Event> {
        self.events.iter_mut().find(|e| e.id == id)
    }
}

impl Event {
    pub fn new(
        name: String,
        start: NaiveDateTime,
        end: NaiveDateTime,
        location: Option<String>,
        description: Option<String>,
    ) -> Self {
        Event {
            id: Uuid::new_v4(),
            name,
            start,
            end,
            location,
            description,
        }
    }

    pub fn from_ical_event(event: IcalEvent) -> Result<Self> {
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

    pub fn to_ical_event(&self) -> IcalEvent {
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
