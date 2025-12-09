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
        let mut day_events = events.on_date(target_date);
        
        // Sort events by start time
        day_events.sort_by(|a, b| a.start.cmp(&b.start));
        
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
            let mut day_events = events.on_date(current_date);
            
            // Sort events by start time
            day_events.sort_by(|a, b| a.start.cmp(&b.start));
            
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
    
    // Calculate the total number of rows for all months
    let mut total_rows = 0;
    let mut all_month_data = Vec::new();
    
    for month_num in 0..options.number {
        let target_date = add_months_to_date(options.date, month_num);
        let (first_of_month, last_of_month) = get_month_bounds(target_date);
        
        let mut current_date = first_of_month
            - chrono::Duration::days(first_of_month.weekday().num_days_from_monday() as i64);
        
        let mut month_rows = 0;
        while current_date <= last_of_month {
            month_rows += 1;
            current_date = current_date + chrono::Duration::days(7);
        }
        
        total_rows += month_rows + 2; // +2 for month header and weekday header
        all_month_data.push((first_of_month, last_of_month, month_rows));
    }
    
    // Get upcoming events from the start datetime (not just date)
    let start_datetime = if options.date == chrono::Local::now().date_naive() {
        chrono::Local::now().naive_local() // Start from now if viewing today
    } else {
        options.date.and_hms_opt(0, 0, 0).unwrap() // Start from beginning of specified date
    };
    
    let mut upcoming_events: Vec<_> = events.all().iter()
        .filter(|e| e.start >= start_datetime)
        .collect();
    upcoming_events.sort_by(|a, b| a.start.cmp(&b.start));
    let upcoming_events: Vec<_> = upcoming_events.into_iter().take(total_rows).collect();
    let mut upcoming_iter = upcoming_events.iter();
    
    // Display each month
    for (month_index, (first_of_month, _last_of_month, row_count)) in all_month_data.into_iter().enumerate() {
        // Add spacing between months - fill calendar space and show next event
        if month_index > 0 {
            print!("                     "); // 21 spaces to match calendar width
            if let Some(event) = upcoming_iter.next() {
                print_upcoming_event(event, term_width);
            } else {
                println!();
            }
        }
        
        // Print month header
        let month_year = first_of_month.format("%B %Y").to_string().bold();
        print!("{:^20} ", month_year);
        if month_index == 0 {
            println!(); // First month header gets its own line
        } else if let Some(event) = upcoming_iter.next() {
            print_upcoming_event(event, term_width);
        } else {
            println!();
        }
        
        // Print weekday header with "Coming up:" for first month only
        if month_index == 0 {
            println!("Mo Tu We Th Fr Sa Su    Coming up:"); // Coming up gets its own line
        } else {
            print!("Mo Tu We Th Fr Sa Su ");
            if let Some(event) = upcoming_iter.next() {
                print_upcoming_event(event, term_width);
            } else {
                println!();
            }
        }
        
        // Print calendar grid
        let mut current_date = first_of_month
            - chrono::Duration::days(first_of_month.weekday().num_days_from_monday() as i64);
        
        for _week in 0..row_count {
            for _weekday in 0..7 {
                print_day_cell(current_date, first_of_month.month(), chrono::Local::now().date_naive(), events);
                current_date = current_date + chrono::Duration::days(1);
            }
            
            // Print upcoming event for this row
            if let Some(event) = upcoming_iter.next() {
                print_upcoming_event(event, term_width);
            } else {
                println!();
            }
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
