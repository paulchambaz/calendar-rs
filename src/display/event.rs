use crate::calendar::events::Event;

/// Display detailed information about a single event
pub fn show_event_details(event: &Event) {
    let date = event.start.format("%A, %d %B");
    let start_time = event.start.format("%H:%M");
    let end_time = event.end.format("%H:%M");

    println!("Name: {}", event.name);
    println!("Date: {}", date);
    println!("Time: {}-{}", start_time, end_time);

    if let Some(location) = &event.location {
        println!("Location: {}", location);
    }

    if let Some(description) = &event.description {
        println!("Description: {}", description);
    }

    println!("Id: {}", event.id);
}

/// Display event for deletion confirmation
pub fn show_event_for_deletion(event: &Event) {
    println!("You are about to delete '{}'", event.name);
    
    let date = event.start.format("%A, %d %B");
    let start_time = event.start.format("%H:%M");
    let end_time = event.end.format("%H:%M");
    
    println!("Scheduled for {} from {} to {}", date, start_time, end_time);
    
    if let Some(location) = &event.location {
        println!("Location: {}", location);
    }
}
