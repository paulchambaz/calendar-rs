use crate::calendar;
use crate::cli;
use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, Timelike};
use std::io::Write;

pub fn list(cmd: cli::CalendarListArgs) -> Result<()> {
    let mut events = if let Some(calendar_name) = cmd.calendar {
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

        println!("{:?}", until);

        let mut i = 0;
        loop {
            if i % every == 0 {
                match repeat {
                    cli::RepeatFrequency::Daily => {
                        let start = cmd.start.date() + Duration::days(i.into());

                        if start >= until {
                            break;
                        }

                        let duration = cmd.end.date() - cmd.start.date();
                        let end = start + duration;

                        calendar.add_event(
                            cmd.name.clone(),
                            start.into(),
                            end.into(),
                            cmd.loc.clone(),
                            cmd.desc.clone(),
                        )?;
                    }
                    cli::RepeatFrequency::Weekly => {
                        let start = cmd.start.date() + Duration::days((i * 7).into());

                        if start >= until {
                            break;
                        }

                        let duration = cmd.end.date() - cmd.start.date();
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
                        let start_date = cmd.start.date();
                        let mut year = start_date.year();
                        let mut month = start_date.month();
                        let day = start_date.day();

                        month += i;
                        while month > 12 {
                            month -= 12;
                            year += 1;
                        }

                        let start = get_real_date(year, month, day)?;

                        if start >= until {
                            break;
                        }

                        let duration = cmd.end.date() - cmd.start.date();
                        let end = start + duration;

                        calendar.add_event(
                            cmd.name.clone(),
                            start.into(),
                            end.into(),
                            cmd.loc.clone(),
                            cmd.desc.clone(),
                        )?;
                    }
                    cli::RepeatFrequency::Yearly => {
                        let start_date = cmd.start.date();
                        let mut year = start_date.year();
                        let month = start_date.month();
                        let day = start_date.day();

                        year += i as i32;

                        let start = get_real_date(year, month, day)?;

                        if start >= until {
                            break;
                        }

                        let duration = cmd.end.date() - cmd.start.date();
                        let end = start + duration;

                        calendar.add_event(
                            cmd.name.clone(),
                            start.into(),
                            end.into(),
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
    let mut calendar = calendar::load(&cmd.calendar)?;
    calendar.edit_event(
        cmd.event_id,
        cmd.name,
        cmd.start,
        cmd.end,
        cmd.loc,
        cmd.desc,
    )?;

    for event in calendar.events {
        println!("{:?}", event);
    }

    Ok(())
}

pub fn delete(cmd: cli::CalendarDeleteArgs) -> Result<()> {
    let mut calendar = calendar::load(&cmd.calendar)?;

    if !cmd.force {
        let event = calendar
            .get_event(cmd.event_id)
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
    println!("EventId: {:?}", cmd.event_id);
    println!("Calendar: {:?}", cmd.calendar);

    todo!("show event");

    Ok(())
}

pub fn view(cmd: cli::CalendarViewArgs) -> Result<()> {
    println!("Date: {:?}", cmd.date);
    println!("Mode: {:?}", cmd.mode);
    println!("Calendar: {:?}", cmd.calendar);

    todo!("view event");

    Ok(())
}

pub fn sync(cmd: cli::CalendarSyncArgs) -> Result<()> {
    println!("Calendar: {:?}", cmd.calendar);

    todo!("sync event");

    Ok(())
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
