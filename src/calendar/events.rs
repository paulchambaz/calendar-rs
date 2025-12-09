use chrono::NaiveDateTime;
use uuid::Uuid;

/// A single calendar event
#[derive(Debug, Clone)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub location: Option<String>,
    pub description: Option<String>,
}

impl Event {
    /// Create a new event with all details
    pub fn new_with_details(
        name: String,
        start: NaiveDateTime,
        end: NaiveDateTime,
        location: Option<String>,
        description: Option<String>,
    ) -> Self {
        Event {
            id: Uuid::new_v4().to_string(),
            name,
            start,
            end,
            location,
            description,
        }
    }
    
    /// Create an event with a specific ID (for loading from files)
    pub fn with_id(
        id: String,
        name: String,
        start: NaiveDateTime,
        end: NaiveDateTime,
        location: Option<String>,
        description: Option<String>,
    ) -> Self {
        Event {
            id,
            name,
            start,
            end,
            location,
            description,
        }
    }
    
    /// Update this event's details
    pub fn update(
        &mut self,
        name: Option<String>,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        location: Option<String>,
        description: Option<String>,
    ) {
        if let Some(new_name) = name {
            self.name = new_name;
        }
        if let Some(new_start) = start {
            self.start = new_start;
        }
        if let Some(new_end) = end {
            self.end = new_end;
        }
        if let Some(new_location) = location {
            self.location = Some(new_location);
        }
        if let Some(new_description) = description {
            self.description = Some(new_description);
        }
    }
    
    /// Check if this event happens on a specific date
    pub fn is_on_date(&self, date: chrono::NaiveDate) -> bool {
        self.start.date() == date
    }
    
    /// Get the duration of this event
    pub fn duration(&self) -> chrono::Duration {
        self.end - self.start
    }
}

/// A collection of events for a calendar
#[derive(Debug, Clone)]
pub struct EventList {
    events: Vec<Event>,
}

impl EventList {
    /// Create a new empty event list
    pub fn new() -> Self {
        EventList {
            events: Vec::new(),
        }
    }
    
    /// Create an event list from a vector
    pub fn from_events(events: Vec<Event>) -> Self {
        EventList { events }
    }
    
    
    /// Find an event by ID
    pub fn find_by_id(&self, id: &str) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }
    
    /// Find a mutable event by ID
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Event> {
        self.events.iter_mut().find(|e| e.id == id)
    }
    
    /// Get all events
    pub fn all(&self) -> &[Event] {
        &self.events
    }
    
    
    /// Get events on a specific date
    pub fn on_date(&self, date: chrono::NaiveDate) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.is_on_date(date))
            .collect()
    }
}

impl Default for EventList {
    fn default() -> Self {
        Self::new()
    }
}
