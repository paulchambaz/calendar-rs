use crate::calendar;
use crate::cli;

pub fn list(cmd: cli::CalendarListArgs) {
    println!("Calendar: {:?}", cmd.calendar);
    let mut events = calendar::generate_random_events(100);

    // Sort events by start date
    events.sort_by(|a, b| a.start.cmp(&b.start));

    // Filter events by date range
    events.retain(|event| {
        let event_date = event.start.date();
        event_date >= cmd.from && event_date <= cmd.to
    });

    // Implement basic fuzzy search if query is provided
    if let Some(query) = cmd.query {
        events.retain(|event| fuzzy_match(&event.name, &query));
    }

    // Limit the number of events if specified
    if let Some(limit) = cmd.limit {
        events.truncate(limit);
    }

    // Print events
    for event in &events {
        let day_of_week = event.start.format("%a");
        let date = event.start.format("%d %b");
        let start_time = event.start.format("%H:%M");
        let end_time = event.end.format("%H:%M");

        let location = event.location.as_deref().unwrap_or("");

        if cmd.id {
            println!(
                "{} {}, {} {}-{} - {} in {}",
                event.id, day_of_week, date, start_time, end_time, event.name, location
            );
        } else {
            println!(
                "{}, {} {}-{} - {} in {}",
                day_of_week, date, start_time, end_time, event.name, location
            );
        }
    }
}

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
