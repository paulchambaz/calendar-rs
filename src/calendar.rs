use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use std::path::PathBuf;

use crate::storage;

#[derive(Debug, Clone)]
pub struct Calendar {
    pub path: PathBuf,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: String,
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

        storage::write_event(&self.path, &event)?;

        Ok(())
    }

    pub fn remove_event(&mut self, event_id: String) -> Result<()> {
        let _ = self
            .get_event(event_id.clone())
            .ok_or_else(|| anyhow!("Could not find event with this uuid"))?;

        storage::delete_event(&self.path, event_id)?;

        Ok(())
    }

    pub fn edit_event(
        &mut self,
        id: String,
        name: Option<String>,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        location: Option<String>,
        description: Option<String>,
    ) -> Result<()> {
        let path = self.path.clone();

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

        storage::write_event(&path, &event)?;

        Ok(())
    }

    pub fn get_event(&self, id: String) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }

    fn get_event_mut(&mut self, id: String) -> Option<&mut Event> {
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
            id: String::new(),
            name,
            start,
            end,
            location,
            description,
        }
    }
}
