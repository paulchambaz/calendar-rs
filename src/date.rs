use anyhow::{anyhow, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

lazy_static! {
    static ref DAY_MONTH_REGEX: Regex = Regex::new(r"^(\d{1,2})[-/]([a-zA-Z]+)$").unwrap();
    static ref MONTH_DAY_REGEX: Regex = Regex::new(r"^([a-zA-Z]+)[-/](\d{1,2})$").unwrap();
    static ref MONTH_YEAR_REGEX: Regex = Regex::new(r"^([a-zA-Z]+)[-/](\d{4})$").unwrap();
    static ref YEAR_MONTH_REGEX: Regex = Regex::new(r"^(\d{4})[-/]([a-zA-Z]+)$").unwrap();
    static ref DATE_REGEX_YMD: Regex = Regex::new(r"^(\d{4})[-/](\d{1,2})[-/](\d{1,2})$").unwrap();
    static ref DATE_REGEX_DMY: Regex = Regex::new(r"^(\d{1,2})[-/](\d{1,2})[-/](\d{4})$").unwrap();
    static ref SHORT_DATE_REGEX: Regex = Regex::new(r"^(\d{1,2})[-/](\d{1,2})$").unwrap();
    static ref TIME_REGEX: Regex = Regex::new(r"^(\d{1,2}):(\d{2})(?::(\d{2}))?$").unwrap();
    static ref RELATIVE_DATE_REGEX: Regex =
        Regex::new(r"^(yesterday|yes|today|tomorrow|tom|(\d+)([dwmy]))$").unwrap();
    static ref WEEKDAY_REGEX: Regex = Regex::new(
        r"^(monday|mon|tuesday|tue|wednesday|wed|thursday|thu|friday|fri|saturday|sat|sunday|sun)$"
    )
    .unwrap();
    static ref MONTH_MAP: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("jan", 1);
        m.insert("january", 1);
        m.insert("feb", 2);
        m.insert("february", 2);
        m.insert("mar", 3);
        m.insert("march", 3);
        m.insert("apr", 4);
        m.insert("april", 4);
        m.insert("may", 5);
        m.insert("jun", 6);
        m.insert("june", 6);
        m.insert("jul", 7);
        m.insert("july", 7);
        m.insert("aug", 8);
        m.insert("august", 8);
        m.insert("sep", 9);
        m.insert("september", 9);
        m.insert("oct", 10);
        m.insert("october", 10);
        m.insert("nov", 11);
        m.insert("november", 11);
        m.insert("dec", 12);
        m.insert("december", 12);
        m
    };
}

#[derive(Debug, Clone, Copy)]
pub struct CalendarDate(NaiveDate);

#[derive(Debug, Clone, Copy)]
pub struct CalendarDateTime(NaiveDateTime);

#[derive(Debug, Clone, Copy)]
pub struct CalendarTime(NaiveTime);

impl FromStr for CalendarDate {
    type Err = anyhow::Error;

    fn from_str(date_str: &str) -> Result<Self, Self::Err> {
        let today = Local::now().naive_local().date();

        if let Some(caps) = DATE_REGEX_YMD.captures(date_str) {
            return NaiveDate::from_ymd_opt(caps[1].parse()?, caps[2].parse()?, caps[3].parse()?)
                .map(CalendarDate)
                .ok_or_else(|| anyhow!("Invalid date"));
        }

        if let Some(caps) = DATE_REGEX_DMY.captures(date_str) {
            return NaiveDate::from_ymd_opt(caps[3].parse()?, caps[2].parse()?, caps[1].parse()?)
                .map(CalendarDate)
                .ok_or_else(|| anyhow!("Invalid date"));
        }

        if let Some(caps) = SHORT_DATE_REGEX.captures(date_str) {
            let day: u32 = caps[1].parse()?;
            let month: u32 = caps[2].parse()?;
            return NaiveDate::from_ymd_opt(today.year(), month, day)
                .map(CalendarDate)
                .ok_or_else(|| anyhow!("Invalid date"));
        }

        if let Some(caps) = RELATIVE_DATE_REGEX.captures(date_str) {
            return match &caps[1] {
                "yesterday" => Ok(CalendarDate(today - Duration::days(1))),
                "yes" => Ok(CalendarDate(today - Duration::days(1))),
                "today" => Ok(CalendarDate(today)),
                "tomorrow" => Ok(CalendarDate(today + Duration::days(1))),
                "tom" => Ok(CalendarDate(today + Duration::days(1))),
                _ => {
                    let amount: i64 = caps[2].parse()?;
                    match &caps[3] {
                        "d" => Ok(CalendarDate(today + Duration::days(amount))),
                        "w" => Ok(CalendarDate(today + Duration::weeks(amount))),
                        "m" => Ok(CalendarDate(today + Duration::days(amount * 30))), // Approximate
                        "y" => Ok(CalendarDate(today + Duration::days(amount * 365))), // Approximate
                        _ => Err(anyhow!("Invalid date format")),
                    }
                }
            };
        }

        if let Some(caps) = WEEKDAY_REGEX.captures(date_str) {
            let target_weekday = match &caps[1] {
                "mon" => Weekday::Mon,
                "monday" => Weekday::Mon,
                "tue" => Weekday::Tue,
                "tuesday" => Weekday::Tue,
                "wed" => Weekday::Wed,
                "wednesday" => Weekday::Wed,
                "thu" => Weekday::Thu,
                "thurday" => Weekday::Thu,
                "fri" => Weekday::Fri,
                "friday" => Weekday::Fri,
                "sat" => Weekday::Sat,
                "saturday" => Weekday::Sat,
                "sun" => Weekday::Sun,
                "sunday" => Weekday::Sun,
                _ => return Err(anyhow!("Invalid weekday")),
            };
            let days_ahead = (7 + target_weekday.num_days_from_monday()
                - today.weekday().num_days_from_monday())
                % 7;
            return Ok(CalendarDate(today + Duration::days(days_ahead as i64)));
        }

        if let Some(month) = MONTH_MAP.get(date_str.to_lowercase().as_str()) {
            let mut year = today.year();
            if today.month() > *month {
                year += 1;
            }
            return NaiveDate::from_ymd_opt(year, *month, 1)
                .map(CalendarDate)
                .ok_or_else(|| anyhow!("Invalid month"));
        }

        if let Some(caps) = MONTH_YEAR_REGEX.captures(date_str) {
            let month = parse_month(&caps[1])?;
            let year: i32 = caps[2].parse()?;
            return NaiveDate::from_ymd_opt(year, month, 1)
                .map(CalendarDate)
                .ok_or_else(|| anyhow!("Invalid month-year combination"));
        }

        if let Some(caps) = YEAR_MONTH_REGEX.captures(date_str) {
            let year: i32 = caps[1].parse()?;
            let month = parse_month(&caps[2])?;
            return NaiveDate::from_ymd_opt(year, month, 1)
                .map(CalendarDate)
                .ok_or_else(|| anyhow!("Invalid year-month combination"));
        }

        if let Some(caps) = DAY_MONTH_REGEX.captures(date_str) {
            let day: u32 = caps[1].parse()?;
            let month = parse_month(&caps[2])?;
            return get_next_occurrence(today, month, day);
        }

        if let Some(caps) = MONTH_DAY_REGEX.captures(date_str) {
            let month = parse_month(&caps[1])?;
            let day: u32 = caps[2].parse()?;
            return get_next_occurrence(today, month, day);
        }

        Err(anyhow!("Unrecognized date format"))
    }
}

fn get_next_occurrence(
    today: NaiveDate,
    month: u32,
    day: u32,
) -> Result<CalendarDate, anyhow::Error> {
    let this_year = today.year();
    let next_year = this_year + 1;

    // Try this year first
    if let Some(date) = NaiveDate::from_ymd_opt(this_year, month, day) {
        if date >= today {
            return Ok(CalendarDate(date));
        }
    }

    // If this year's date has passed or doesn't exist, try next year
    if let Some(date) = NaiveDate::from_ymd_opt(next_year, month, day) {
        Ok(CalendarDate(date))
    } else {
        // If the date doesn't exist in either year (e.g., February 30th)
        Err(anyhow!(
            "Invalid date: the specified day does not exist for this month"
        ))
    }
}

fn parse_month(month_str: &str) -> Result<u32, anyhow::Error> {
    MONTH_MAP
        .get(month_str.to_lowercase().as_str())
        .copied()
        .ok_or_else(|| anyhow!("Invalid month name"))
}

impl FromStr for CalendarDateTime {
    type Err = anyhow::Error;

    fn from_str(datetime_str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = datetime_str.split('@').collect();
        let date_str = parts.get(0).ok_or_else(|| anyhow!("Missing date"))?;
        let time_str = parts.get(1).ok_or_else(|| anyhow!("Missing time"))?;

        let CalendarDate(date) = date_str.parse()?;
        let CalendarTime(time) = time_str.parse()?;

        Ok(CalendarDateTime(date.and_time(time)))
    }
}

impl FromStr for CalendarTime {
    type Err = anyhow::Error;
    fn from_str(time_str: &str) -> Result<Self, Self::Err> {
        if let Some(caps) = TIME_REGEX.captures(time_str) {
            let hour: u32 = caps[1].parse()?;
            let minute: u32 = caps[2].parse()?;
            let second: u32 = caps.get(3).map_or(Ok(0), |m| m.as_str().parse())?;
            return NaiveTime::from_hms_opt(hour, minute, second)
                .map(CalendarTime)
                .ok_or_else(|| anyhow!("Invalid time"));
        }
        // Allow single-digit hour input
        if let Ok(hour) = time_str.parse::<u32>() {
            if hour < 24 {
                return NaiveTime::from_hms_opt(hour, 0, 0)
                    .map(CalendarTime)
                    .ok_or_else(|| anyhow!("Invalid time"));
            }
        }
        Err(anyhow!("Unrecognized time format"))
    }
}

impl CalendarDate {
    pub fn parse(s: &str) -> Result<Self> {
        s.parse()
    }

    pub fn inner(&self) -> NaiveDate {
        self.0
    }
}

impl CalendarDateTime {
    pub fn parse(s: &str) -> Result<Self> {
        s.parse()
    }

    pub fn inner(&self) -> NaiveDateTime {
        self.0
    }
}
