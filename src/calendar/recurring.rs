use chrono::{Duration, NaiveDateTime, NaiveDate, Datelike};
use crate::calendar::events::Event;

/// A parsed recurrence rule from an ICS file
#[derive(Debug, Clone)]
pub struct RecurrenceRule {
    pub frequency: Frequency,
    pub interval: u32,
    pub until: Option<NaiveDateTime>,
    pub count: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceRule {
    /// Parse an RRULE string from ICS format
    pub fn from_ics_string(rrule: &str) -> Result<Self, String> {
        let mut frequency = None;
        let mut interval = 1;
        let mut until = None;
        let mut count = None;

        for part in rrule.split(';') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "FREQ" => {
                    frequency = Some(match kv[1] {
                        "DAILY" => Frequency::Daily,
                        "WEEKLY" => Frequency::Weekly,
                        "MONTHLY" => Frequency::Monthly,
                        "YEARLY" => Frequency::Yearly,
                        _ => return Err(format!("Unknown frequency: {}", kv[1])),
                    });
                }
                "INTERVAL" => {
                    interval = kv[1].parse()
                        .map_err(|_| format!("Invalid interval: {}", kv[1]))?;
                }
                "UNTIL" => {
                    until = Some(parse_ics_datetime(kv[1])?);
                }
                "COUNT" => {
                    count = Some(kv[1].parse()
                        .map_err(|_| format!("Invalid count: {}", kv[1]))?);
                }
                _ => {} // Ignore other rules for now
            }
        }

        let frequency = frequency
            .ok_or("FREQ is required in RRULE")?;

        Ok(RecurrenceRule {
            frequency,
            interval,
            until,
            count,
        })
    }

    /// Expand a base event into individual occurrences
    pub fn expand_event(&self, base_event: &Event) -> Vec<Event> {
        let mut events = vec![base_event.clone()];
        let duration = base_event.duration();
        
        // Limit expansion to avoid infinite loops and huge lists
        let max_occurrences = self.count.unwrap_or(365); // Max 1 year of daily events
        let max_date = chrono::Local::now().naive_local().date() + Duration::days(730); // Max 2 years ahead
        
        for i in 1..max_occurrences {
            let next_start = match self.frequency {
                Frequency::Daily => base_event.start + Duration::days(i as i64 * self.interval as i64),
                Frequency::Weekly => base_event.start + Duration::weeks(i as i64 * self.interval as i64),
                Frequency::Monthly => add_months(base_event.start, i * self.interval),
                Frequency::Yearly => add_years(base_event.start, i * self.interval),
            };

            // Check if we've hit the until date
            if let Some(until) = self.until {
                if next_start > until {
                    break;
                }
            }

            // Don't generate events too far in the future
            if next_start.date() > max_date {
                break;
            }

            let mut next_event = base_event.clone();
            next_event.id = format!("{}-{}", base_event.id, i);
            next_event.start = next_start;
            next_event.end = next_start + duration;
            
            events.push(next_event);
        }

        events
    }
}

/// Parse ICS datetime format (basic support)
fn parse_ics_datetime(datetime_str: &str) -> Result<NaiveDateTime, String> {
    // Handle YYYYMMDDTHHMMSSZ format
    if datetime_str.ends_with('Z') && datetime_str.len() == 16 {
        let date_part = &datetime_str[0..8];
        let time_part = &datetime_str[9..15];
        
        let year: i32 = date_part[0..4].parse()
            .map_err(|_| "Invalid year")?;
        let month: u32 = date_part[4..6].parse()
            .map_err(|_| "Invalid month")?;
        let day: u32 = date_part[6..8].parse()
            .map_err(|_| "Invalid day")?;
        
        let hour: u32 = time_part[0..2].parse()
            .map_err(|_| "Invalid hour")?;
        let minute: u32 = time_part[2..4].parse()
            .map_err(|_| "Invalid minute")?;
        let second: u32 = time_part[4..6].parse()
            .map_err(|_| "Invalid second")?;
        
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or("Invalid date")?;
        
        date.and_hms_opt(hour, minute, second)
            .ok_or("Invalid time")
            .map_err(|e| e.to_string())
    }
    // Handle YYYYMMDD format (date only)
    else if datetime_str.len() == 8 {
        let year: i32 = datetime_str[0..4].parse()
            .map_err(|_| "Invalid year")?;
        let month: u32 = datetime_str[4..6].parse()
            .map_err(|_| "Invalid month")?;
        let day: u32 = datetime_str[6..8].parse()
            .map_err(|_| "Invalid day")?;
        
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or("Invalid date")?;
        
        Ok(date.and_hms_opt(0, 0, 0).unwrap())
    }
    else {
        Err(format!("Unsupported datetime format: {}", datetime_str))
    }
}

/// Add months to a datetime, handling month boundaries properly
fn add_months(datetime: NaiveDateTime, months: u32) -> NaiveDateTime {
    let mut year = datetime.date().year();
    let mut month = datetime.date().month();
    let day = datetime.date().day();
    
    month += months;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    
    // Handle end-of-month dates (e.g., Jan 31 -> Feb 28)
    let target_date = loop {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            break date;
        }
        // Day doesn't exist in this month, try previous day
        if day > 1 {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day - 1) {
                break date;
            }
        }
        // Fallback to last day of month
        let last_day = match month {
            2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        break NaiveDate::from_ymd_opt(year, month, last_day).unwrap();
    };
    
    target_date.and_time(datetime.time())
}

/// Add years to a datetime, handling leap years
fn add_years(datetime: NaiveDateTime, years: u32) -> NaiveDateTime {
    let new_year = datetime.date().year() + years as i32;
    let month = datetime.date().month();
    let day = datetime.date().day();
    
    // Handle Feb 29 on non-leap years
    let target_date = if month == 2 && day == 29 {
        if new_year % 4 == 0 && (new_year % 100 != 0 || new_year % 400 == 0) {
            NaiveDate::from_ymd_opt(new_year, month, day).unwrap()
        } else {
            NaiveDate::from_ymd_opt(new_year, month, 28).unwrap()
        }
    } else {
        NaiveDate::from_ymd_opt(new_year, month, day).unwrap()
    };
    
    target_date.and_time(datetime.time())
}
