use chrono::NaiveDate;
use crate::calendar::events::{Event, EventList};

/// Configuration for listing events
pub struct ListOptions {
    pub show_ids: bool,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

impl Default for ListOptions {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive();
        ListOptions {
            show_ids: false,
            from_date: today,
            to_date: today + chrono::Duration::days(30),
            query: None,
            limit: None,
        }
    }
}

/// Display events as a simple list
pub fn show_events(events: &EventList, options: &ListOptions) {
    let mut filtered_events = filter_events(events, options);
    
    // Sort by start time
    filtered_events.sort_by(|a, b| a.start.cmp(&b.start));
    
    // Apply limit if specified
    if let Some(limit) = options.limit {
        filtered_events.truncate(limit);
    }
    
    for event in &filtered_events {
        print_event(event, options.show_ids);
    }
}

/// Filter events based on options
fn filter_events<'a>(events: &'a EventList, options: &ListOptions) -> Vec<&'a Event> {
    let current_time = chrono::Local::now().naive_local().time();
    let from_datetime = options.from_date.and_time(current_time);
    let to_datetime = options.to_date.and_time(current_time);
    
    events.all().iter()
        .filter(|event| {
            // Filter by date range
            event.start >= from_datetime && event.start <= to_datetime
        })
        .filter(|event| {
            // Filter by search query if provided
            if let Some(query) = &options.query {
                fuzzy_match(&event.name, query) ||
                event.location.as_ref().map_or(false, |loc| fuzzy_match(loc, query)) ||
                event.description.as_ref().map_or(false, |desc| fuzzy_match(desc, query))
            } else {
                true
            }
        })
        .collect()
}

/// Print a single event in list format
fn print_event(event: &Event, show_id: bool) {
    let day_of_week = event.start.format("%a");
    let date = event.start.format("%d %b");
    let start_time = event.start.format("%H:%M");
    let end_time = event.end.format("%H:%M");

    let location_part = event
        .location
        .as_ref()
        .map_or(String::new(), |loc| format!(" in {}", loc));

    if show_id {
        println!(
            "{}: {} {} {}-{} - {}{}",
            event.id, day_of_week, date, start_time, end_time, event.name, location_part
        );
    } else {
        println!(
            "{} {} {}-{} - {}{}",
            day_of_week, date, start_time, end_time, event.name, location_part
        );
    }
}

/// Simple fuzzy matching like the original
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let text = text.to_lowercase();
    let pattern = pattern.to_lowercase();
    let mut text_chars = text.chars();
    
    for p in pattern.chars() {
        if !text_chars.any(|c| c == p) {
            return false;
        }
    }
    true
}
