use chrono::{Datelike, NaiveDate};
use colored::Colorize;
use terminal_size::{terminal_size, Width};
use crate::calendar::events::{Event, EventList};
use crate::args::ViewMode;

/// Configuration for calendar display
pub struct CalendarOptions {
    pub date: NaiveDate,
    pub mode: ViewMode,
    pub number: u32,
}

/// Display calendar in the specified mode
pub fn show_calendar(events: &EventList, options: &CalendarOptions) {
    match options.mode {
        ViewMode::Day => show_day_view(events, options),
        ViewMode::Week => show_week_view(events, options),
        ViewMode::Month => show_month_view(events, options),
    }
}

/// Show day view for N days
fn show_day_view(events: &EventList, options: &CalendarOptions) {
    for i in 0..options.number {
        let target_date = options.date + chrono::Duration::days(i.into());
        let day_events = events.on_date(target_date);

        println!("{}", target_date.format("%A, %d %B %Y").to_string().bold());

        if day_events.is_empty() {
            println!("No events");
        } else {
            for event in day_events {
                print_event_time_line(event);
            }
        }

        // Add spacing between days
        if i < options.number - 1 {
            println!();
        }
    }
}

/// Show week view for N weeks
fn show_week_view(events: &EventList, options: &CalendarOptions) {
    for week in 0..options.number {
        let start_of_week = get_start_of_week(options.date + chrono::Duration::weeks(week.into()));

        for day in 0..7 {
            let current_date = start_of_week + chrono::Duration::days(day);
            let day_events = events.on_date(current_date);

            println!("{}", current_date.format("%A, %d %B").to_string().bold());

            if day_events.is_empty() {
                println!("No events");
            } else {
                for event in day_events {
                    print_event_time_line(event);
                }
            }
            println!();
        }

        // Add spacing between weeks
        if week < options.number - 1 {
            println!();
        }
    }
}

/// Show month view for N months with upcoming events sidebar
fn show_month_view(events: &EventList, options: &CalendarOptions) {
    let term_width = get_terminal_width();

    for month_num in 0..options.number {
        let target_date = add_months_to_date(options.date, month_num);
        let (first_of_month, last_of_month) = get_month_bounds(target_date);

        // Get upcoming events for this month
        let upcoming_events = get_upcoming_events_for_month(events, first_of_month, last_of_month);
        let mut upcoming_iter = upcoming_events.iter();

        // Print month header
        print_month_header(&first_of_month, &mut upcoming_iter, term_width);

        // Print weekday header
        print_weekday_header(&mut upcoming_iter, term_width, month_num == 0);

        // Print calendar grid with events sidebar
        print_month_grid(events, first_of_month, last_of_month, &mut upcoming_iter, term_width);

        // Add spacing between months
        if month_num < options.number - 1 {
            println!();
        }
    }
}

/// Print event in time format: "14:00-15:00 - Event Name in Location"
fn print_event_time_line(event: &Event) {
    let start_time = event.start.format("%H:%M");
    let end_time = event.end.format("%H:%M");
    let location_part = event
        .location
        .as_ref()
        .map_or(String::new(), |loc| format!(" in {}", loc));

    println!("{}-{} - {}{}", start_time, end_time, event.name, location_part);
}

/// Get start of week (Monday)
fn get_start_of_week(date: NaiveDate) -> NaiveDate {
    let days_from_monday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(days_from_monday.into())
}

/// Get first and last day of the month
fn get_month_bounds(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let first = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
    let last = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap() - chrono::Duration::days(1)
    };
    (first, last)
}

/// Add months to a date
fn add_months_to_date(date: NaiveDate, months: u32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() + months;

    while month > 12 {
        month -= 12;
        year += 1;
    }

    NaiveDate::from_ymd_opt(year, month, 1).unwrap()
}

/// Get upcoming events for the month, limited and sorted
fn get_upcoming_events_for_month(events: &EventList, first: NaiveDate, last: NaiveDate) -> Vec<&Event> {
    let mut upcoming: Vec<&Event> = events.all().iter()
        .filter(|e| e.start.date() >= first && e.start.date() <= last)
        .collect();
    
    upcoming.sort_by(|a, b| a.start.cmp(&b.start));
    upcoming.truncate(20); // Limit to avoid too many
    upcoming
}

/// Print month header with optional upcoming event
fn print_month_header(first_of_month: &NaiveDate, upcoming_iter: &mut std::slice::Iter<&Event>, term_width: u16) {
    let month_year = first_of_month.format("%B %Y").to_string().bold();
    print!("{:^20} ", month_year);
    
    if let Some(event) = upcoming_iter.next() {
        print_upcoming_event(event, term_width);
    } else {
        println!();
    }
}

/// Print weekday header with optional upcoming event
fn print_weekday_header(upcoming_iter: &mut std::slice::Iter<&Event>, term_width: u16, show_coming_up: bool) {
    if show_coming_up {
        print!("Mo Tu We Th Fr Sa Su    Coming up:");
    } else {
        print!("Mo Tu We Th Fr Sa Su ");
    }
    
    if let Some(event) = upcoming_iter.next() {
        print_upcoming_event(event, term_width);
    } else {
        println!();
    }
}

/// Print the month grid with events sidebar
fn print_month_grid(events: &EventList, first: NaiveDate, last: NaiveDate, upcoming_iter: &mut std::slice::Iter<&Event>, term_width: u16) {
    let mut current_date = get_start_of_week(first);
    let today = chrono::Local::now().date_naive();

    while current_date <= last || current_date.month() == first.month() {
        // Print one week
        let week_start = current_date;
        
        for _day in 0..7 {
            print_day_cell(current_date, first.month(), today, events);
            current_date = current_date + chrono::Duration::days(1);
        }

        // Print upcoming event for this week row
        if let Some(event) = upcoming_iter.next() {
            print_upcoming_event(event, term_width);
        } else {
            println!();
        }

        // Stop if we've gone past the month
        if current_date > last && week_start.month() != first.month() {
            break;
        }
    }
}

/// Print a single day cell in the calendar grid
fn print_day_cell(date: NaiveDate, target_month: u32, today: NaiveDate, events: &EventList) {
    let day_str = format!("{:2}", date.day());
    
    if date.month() != target_month {
        // Day from different month - show as blank
        print!("   ");
    } else if date == today {
        // Today - highlight with background
        print!("{} ", day_str.on_white().black());
    } else if !events.on_date(date).is_empty() {
        // Has events - show in bold
        print!("{} ", day_str.bold());
    } else {
        // Regular day
        print!("{} ", day_str);
    }
}

/// Print an upcoming event in the sidebar
fn print_upcoming_event(event: &Event, term_width: u16) {
    print!("   "); // Spacing from calendar
    
    let date = event.start.format("%d %b").to_string();
    let start_time = event.start.format("%H:%M").to_string();
    let end_time = event.end.format("%H:%M").to_string();
    
    let location_part = event
        .location
        .as_ref()
        .map_or(String::new(), |loc| format!(" in {}", loc));
    
    let full_event = format!(
        "{} {}-{} - {}{}",
        date, start_time, end_time, event.name, location_part
    );
    
    // Calculate available width and truncate if needed
    let available_width = (term_width as usize).saturating_sub(22);
    
    if full_event.len() > available_width {
        let truncated = format!("{}...", &full_event[..available_width.saturating_sub(3)]);
        println!("{}", truncated);
    } else {
        println!("{}", full_event);
    }
}

/// Get terminal width, defaulting to 80
fn get_terminal_width() -> u16 {
    terminal_size()
        .map(|(Width(w), _)| w)
        .unwrap_or(80)
}
