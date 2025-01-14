use crate::calendar;
use crate::cli;
use crate::storage;
use anyhow::{anyhow, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, Timelike};
use colored::Colorize;
use std::io::Write;
use std::process::Command;
use terminal_size::{terminal_size, Width};

pub fn list(cmd: cli::CalendarListArgs) -> Result<()> {
    let mut events = if let Some(calendar_name) = cmd.calendar {
        if calendar_name == "personal" {
            create_personal()?;
        }

        // Load events from the specified calendar
        let calendar = calendar::load(&calendar_name)?;
        calendar.events
    } else {
        // Load events from all calendars
        let calendars = calendar::load_all()?;
        calendars
            .into_iter()
            .flat_map(|calendar| calendar.events)
            .collect()
    };

    // Sort events by start date
    events.sort_by(|a, b| a.start.cmp(&b.start));

    let current_time = Local::now().time();

    // Filter events by date range
    events.retain(|event| {
        let from_datetime = cmd.from.and_time(current_time);
        let to_datetime = cmd.to.and_time(current_time);
        event.start >= from_datetime && event.start <= to_datetime
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

        let location_part = event
            .location
            .as_ref()
            .map_or(String::new(), |loc| format!(" in {}", loc));

        if cmd.id {
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

    Ok(())
}

pub fn add(cmd: cli::CalendarAddArgs) -> Result<()> {
    create_personal()?;

    let mut calendar = calendar::load(&cmd.calendar)?;

    if let Some(repeat) = cmd.repeat {
        let every = cmd
            .every
            .ok_or_else(|| anyhow!("Every option must be set for recurring events"))?;

        let until = cmd.until.unwrap_or(match repeat {
            cli::RepeatFrequency::Daily => cmd.start.date() + Duration::days(7),
            cli::RepeatFrequency::Weekly => cmd.start.date() + Duration::days(30),
            cli::RepeatFrequency::Monthly => cmd.start.date() + Duration::days(365),
            cli::RepeatFrequency::Yearly => cmd.start.date() + Duration::days(3652),
        });

        let mut i = 0;
        loop {
            if i % every == 0 {
                match repeat {
                    cli::RepeatFrequency::Daily => {
                        let start = cmd.start + Duration::days(i.into());

                        if start >= until.into() {
                            break;
                        }

                        let duration = cmd.end - cmd.start;
                        let end = start + duration;

                        calendar.add_event(
                            cmd.name.clone(),
                            start,
                            end,
                            cmd.loc.clone(),
                            cmd.desc.clone(),
                        )?;
                    }
                    cli::RepeatFrequency::Weekly => {
                        let start = cmd.start + Duration::days((i * 7).into());

                        if start >= until.into() {
                            break;
                        }

                        let duration = cmd.end - cmd.start;
                        let end = start + duration;

                        calendar.add_event(
                            cmd.name.clone(),
                            start.into(),
                            end.into(),
                            cmd.loc.clone(),
                            cmd.desc.clone(),
                        )?;
                    }
                    cli::RepeatFrequency::Monthly => {
                        let start_date = cmd.start;
                        let mut year = start_date.year();
                        let mut month = start_date.month();
                        let day = start_date.day();
                        let hour = start_date.hour();
                        let min = start_date.minute();
                        let sec = start_date.second();

                        month += i;
                        while month > 12 {
                            month -= 12;
                            year += 1;
                        }

                        let start = get_real_date(year, month, day)?
                            .and_hms_opt(hour, min, sec)
                            .ok_or_else(|| anyhow!("Failed to create NaiveDateTime"))?;

                        if start >= until.into() {
                            break;
                        }

                        let duration = cmd.end - cmd.start;
                        let end = start + duration;

                        calendar.add_event(
                            cmd.name.clone(),
                            start,
                            end,
                            cmd.loc.clone(),
                            cmd.desc.clone(),
                        )?;
                    }
                    cli::RepeatFrequency::Yearly => {
                        let start_date = cmd.start;
                        let mut year = start_date.year();
                        let month = start_date.month();
                        let day = start_date.day();
                        let hour = start_date.hour();
                        let min = start_date.minute();
                        let sec = start_date.second();

                        year += i as i32;

                        let start = get_real_date(year, month, day)?
                            .and_hms_opt(hour, min, sec)
                            .ok_or_else(|| anyhow!("Failed to create NaiveDateTime"))?;

                        if start >= until.into() {
                            break;
                        }

                        let duration = cmd.end - cmd.start;
                        let end = start + duration;

                        calendar.add_event(
                            cmd.name.clone(),
                            start,
                            end,
                            cmd.loc.clone(),
                            cmd.desc.clone(),
                        )?;
                    }
                };
            }
            i += 1;
        }
    } else {
        calendar.add_event(cmd.name, cmd.start, cmd.end, cmd.loc, cmd.desc)?;
    }

    Ok(())
}

pub fn edit(cmd: cli::CalendarEditArgs) -> Result<()> {
    create_personal()?;

    let mut calendar = calendar::load(&cmd.calendar)?;
    calendar.edit_event(
        cmd.event_id,
        cmd.name,
        cmd.start,
        cmd.end,
        cmd.loc,
        cmd.desc,
    )?;

    Ok(())
}

pub fn delete(cmd: cli::CalendarDeleteArgs) -> Result<()> {
    create_personal()?;

    let mut calendar = calendar::load(&cmd.calendar)?;

    if !cmd.force {
        let event = calendar
            .get_event(cmd.event_id.clone())
            .ok_or_else(|| anyhow!("Could not find event with this uuid"))?;
        print!(
            "You are about to delete '{}', are you sure? (y/N) ",
            event.name
        );
        let mut input = String::new();
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            return Ok(());
        }
    }

    calendar.remove_event(cmd.event_id)?;

    Ok(())
}

pub fn show(cmd: cli::CalendarShowArgs) -> Result<()> {
    if cmd.calendar == "personal" {
        create_personal()?;
    }

    let calendar = calendar::load(&cmd.calendar)?;

    let event = calendar
        .get_event(cmd.event_id)
        .ok_or_else(|| anyhow!("Could not find event with this uuid"))?;

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

    Ok(())
}

pub fn view(cmd: cli::CalendarViewArgs) -> Result<()> {
    let mut events = if let Some(calendar_name) = cmd.calendar {
        if calendar_name == "personal" {
            create_personal()?;
        }

        let calendar = calendar::load(&calendar_name)?;
        calendar.events
    } else {
        let calendars = calendar::load_all()?;
        calendars
            .into_iter()
            .flat_map(|calendar| calendar.events)
            .collect()
    };

    events.sort_by(|a, b| a.start.cmp(&b.start));

    match cmd.mode {
        cli::ViewMode::Day => {
            for i in 0..cmd.number {
                let target_date = cmd.date + chrono::Duration::days(i.into());
                let events_for_day: Vec<_> = events
                    .iter()
                    .filter(|event| event.start.date() == target_date)
                    .collect();

                println!("{}", target_date.format("%A, %d %B %Y").to_string().bold());

                for event in events_for_day {
                    let start_time = event.start.format("%H:%M");
                    let end_time = event.end.format("%H:%M");
                    let location_part = event
                        .location
                        .as_ref()
                        .map_or(String::new(), |loc| format!(" in {}", loc));

                    println!(
                        "{}-{} - {}{}",
                        start_time, end_time, event.name, location_part
                    );
                }
            }
        }
        cli::ViewMode::Week => {
            for week in 0..cmd.number {
                let start_of_week = (cmd.date + chrono::Duration::weeks(week.into()))
                    .week(chrono::Weekday::Mon)
                    .first_day();

                for day in 0..7 {
                    let current_date = start_of_week + chrono::Duration::days(day);

                    println!("{}", current_date.format("%A, %d %B").to_string().bold());

                    let events_for_day: Vec<_> = events
                        .iter()
                        .filter(|event| event.start.date() == current_date)
                        .collect();

                    for event in events_for_day {
                        let start_time = event.start.format("%H:%M");
                        let end_time = event.end.format("%H:%M");
                        let location_part = event
                            .location
                            .as_ref()
                            .map_or(String::new(), |loc| format!(" in {}", loc));

                        println!(
                            "{}-{} - {}{}",
                            start_time, end_time, event.name, location_part
                        );
                    }
                }
            }
        }
        cli::ViewMode::Month => {
            // Get terminal width
            let term_width = terminal_size().map(|(Width(w), _)| w).unwrap_or(80);

            // Calculate the total number of rows for all months
            let mut total_rows = 0;
            let mut all_month_dates = Vec::new();

            for month in 0..cmd.number {
                let target_date = cmd.date + chrono::Months::new(month);
                let (year, month, _) = (target_date.year(), target_date.month(), target_date.day());
                let first_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let last_of_month = first_of_month + chrono::Months::new(1) - chrono::Days::new(1);

                let mut current_date = first_of_month
                    - chrono::Days::new(first_of_month.weekday().num_days_from_monday() as u64);

                let mut month_rows = 0;
                while current_date <= last_of_month {
                    month_rows += 1;
                    current_date = current_date + chrono::Days::new(7);
                }

                total_rows += month_rows + 2; // +2 for month header and weekday header
                all_month_dates.push((first_of_month, last_of_month, month_rows));
            }

            // Filter upcoming events for all displayed months
            let last_displayed_date = all_month_dates.last().unwrap().1;

            let mut line_count = 0;

            let upcoming_events: Vec<_> = events
                .iter()
                .filter(|e| e.start.date() >= cmd.date && e.start.date() <= last_displayed_date)
                .take(total_rows)
                .collect();
            let mut upcoming_iter = upcoming_events.iter().peekable();

            // Display each month
            for (month_index, (first_of_month, _, row_count)) in
                all_month_dates.into_iter().enumerate()
            {
                // Center the month and year
                let month_year = first_of_month.format("%B %Y").to_string().bold();
                print!("{:^20} ", month_year);
                if line_count == 0 {
                    println!();
                    line_count = 1;
                }

                // Print upcoming event for the month header line
                if line_count >= 2 {
                    if let Some(event) = upcoming_iter.next() {
                        print_event(event, term_width);
                    } else {
                        println!();
                    }
                }

                // Print weekday header and "Coming up:" for the first month only
                if month_index == 0 {
                    print!("Mo Tu We Th Fr Sa Su    Coming up:");
                } else {
                    print!("Mo Tu We Th Fr Sa Su ");
                }

                // Print upcoming event for the weekday header line
                if line_count >= 2 {
                    if let Some(event) = upcoming_iter.next() {
                        print_event(event, term_width);
                    } else {
                        println!();
                    }
                }

                if line_count == 1 {
                    println!();
                    line_count = 2;
                }

                let mut current_date = first_of_month
                    - chrono::Days::new(first_of_month.weekday().num_days_from_monday() as u64);

                for _week in 0..row_count {
                    for _weekday in 0..7 {
                        let day_str = format!("{:2}", current_date.day());
                        if current_date.month() != first_of_month.month() {
                            print!("   ");
                        } else if current_date == chrono::Local::now().date_naive() {
                            print!("{} ", day_str.on_white().black());
                        } else if events.iter().any(|e| e.start.date() == current_date) {
                            print!("{} ", day_str.bold());
                        } else {
                            print!("{} ", day_str);
                        }
                        current_date = current_date + chrono::Days::new(1);
                    }

                    // Print upcoming event for this line
                    if let Some(event) = upcoming_iter.next() {
                        print_event(event, term_width);
                    } else {
                        println!();
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn sync(cmd: cli::CalendarSyncArgs) -> Result<()> {
    let mut vdirsyncer_command = Command::new("vdirsyncer");
    vdirsyncer_command.arg("sync");
    vdirsyncer_command.arg("--force-delete");

    if let Some(calendar) = cmd.calendar {
        vdirsyncer_command.arg(&calendar);
        println!("Syncing calendar '{}' with vdirsyncer", calendar);
    } else {
        println!("Syncing calendars with vdirsyncer");
    }
    let output = vdirsyncer_command.output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("vdirsyncer sync failed"));
    }

    Ok(())
}

fn print_event(event: &calendar::Event, term_width: u16) {
    print!("   ");
    let date = event.start.format("%d %b").to_string();
    let start_time = event.start.format("%H:%M").to_string();
    let end_time = event.end.format("%H:%M").to_string();

    let location_part = event
        .location
        .as_ref()
        .map_or(String::new(), |loc| format!(" in {}", loc));

    // Format the entire string
    let formatted_string = format!(
        "{} {}-{} - {}{}",
        date, start_time, end_time, event.name, location_part
    );

    // Calculate available width
    let available_width = term_width as usize - 22;

    // Truncate the formatted string if necessary
    let truncated_string = if formatted_string.len() > available_width {
        format!("{}...", &formatted_string[..available_width - 3])
    } else {
        formatted_string
    };

    println!("{}", truncated_string);
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

fn get_real_date(target_year: i32, target_month: u32, target_day: u32) -> Result<NaiveDate> {
    let mut day = target_day;
    while day > 0 {
        if let Some(date) = NaiveDate::from_ymd_opt(target_year, target_month, day) {
            return Ok(date);
        }
        day -= 1;
    }

    // If we've exhausted all possible days, return an error
    Err(anyhow!(
        "Unable to create a valid date for year {}, month {}",
        target_year,
        target_month
    ))
}
fn create_personal() -> Result<()> {
    let calendars = storage::list_calendars()?;

    if !calendars.contains(&"personal".to_string()) {
        println!("No personal calendar found. Creating it...");
        storage::create_personal()?;
    }

    Ok(())
}
