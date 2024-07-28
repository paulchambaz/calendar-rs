use crate::date::{CalendarDate, CalendarDateTime};
use anyhow::{anyhow, Result};
use chrono::{Duration, NaiveDate, NaiveDateTime};
use clap::{Parser, Subcommand};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List events from all or specific calendars")]
    List(ListArgs),
    #[command(about = "Add a new event to a calendar")]
    Add(AddArgs),
    #[command(about = "Edit an existing event")]
    Edit(EditArgs),
    #[command(about = "Delete an event")]
    Delete(DeleteArgs),
    #[command(about = "Show details of a specific event")]
    Show(ShowArgs),
    #[command(about = "Display calendar in various formats (daily, weekly, monthly)")]
    View(ViewArgs),
    #[command(about = "Synchronize calendars using vdirsyncer")]
    Sync(SyncArgs),
}

// Validated structs for each command

#[derive(Debug)]
pub struct CalendarListArgs {
    pub query: Option<String>,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub limit: Option<usize>,
    pub id: bool,
    pub calendar: Option<String>,
}

#[derive(Debug)]
pub struct CalendarAddArgs {
    pub calendar: String,
    pub name: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub loc: Option<String>,
    pub desc: Option<String>,
    pub repeat: Option<RepeatFrequency>,
    pub every: Option<u32>,
    pub until: Option<NaiveDate>,
}

#[derive(Debug)]
pub struct CalendarEditArgs {
    pub event_id: Uuid,
    pub calendar: String,
    pub name: Option<String>,
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub loc: Option<String>,
    pub desc: Option<String>,
}

#[derive(Debug)]
pub struct CalendarDeleteArgs {
    pub event_id: Uuid,
    pub calendar: String,
    pub force: bool,
}

#[derive(Debug)]
pub struct CalendarShowArgs {
    pub event_id: Uuid,
    pub calendar: String,
}

#[derive(Debug)]
pub struct CalendarViewArgs {
    pub date: NaiveDate,
    pub mode: ViewMode,
    pub calendar: Option<String>,
    pub number: u32,
}

#[derive(Debug)]
pub struct CalendarSyncArgs {
    pub calendar: Option<String>,
}

// Enums for specific types

#[derive(Debug, Clone, Copy)]
pub enum RepeatFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug)]
pub enum SearchField {
    Name,
    Description,
    Location,
}

#[derive(Debug)]
pub enum ViewMode {
    Day,
    Week,
    Month,
}

// Implementation of FromStr for custom enums

impl FromStr for RepeatFrequency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(RepeatFrequency::Daily),
            "weekly" => Ok(RepeatFrequency::Weekly),
            "monthly" => Ok(RepeatFrequency::Monthly),
            "yearly" => Ok(RepeatFrequency::Yearly),
            _ => Err(anyhow!("Invalid repeat frequency")),
        }
    }
}

impl FromStr for SearchField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "name" => Ok(SearchField::Name),
            "description" => Ok(SearchField::Description),
            "location" => Ok(SearchField::Location),
            _ => Err(anyhow!("Invalid search field")),
        }
    }
}

impl FromStr for ViewMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "day" => Ok(ViewMode::Day),
            "week" => Ok(ViewMode::Week),
            "month" => Ok(ViewMode::Month),
            _ => Err(anyhow!("Invalid view mode")),
        }
    }
}

// Argument structs for each command

#[derive(Parser)]
pub struct ListArgs {
    #[arg(help = "Query terms in the fzf search")]
    pub query: Vec<String>,
    #[arg(short, long, help = "Specify the calendar to list (default: all)")]
    calendar: Option<String>,
    #[arg(short, long, help = "Start date for listing (default: today)")]
    from: Option<String>,
    #[arg(
        short,
        long,
        help = "End date for listing (default: 1 month from today)"
    )]
    to: Option<String>,
    #[arg(short, long, help = "Limit the number of events shown")]
    limit: Option<usize>,
    #[arg(
        short,
        long,
        help = "Show the uuid of the tasks for future modification"
    )]
    id: bool,
}

#[derive(Parser)]
pub struct AddArgs {
    #[arg(required = true, help = "Name of the event")]
    pub name: Vec<String>,
    #[arg(
        short,
        long,
        help = "Event start time (eg. tom@21 14-jul@12:30 2024/08/06@08:00)"
    )]
    pub at: String,
    #[arg(short, long, help = "Event end time (default: 1 hour after start)")]
    pub to: Option<String>,
    #[arg(
        short,
        long,
        help = "The calendar to add the event to (default: personal)"
    )]
    pub calendar: Option<String>,
    #[arg(short, long, help = "Event location")]
    pub loc: Option<String>,
    #[arg(short, long, help = "Event description")]
    pub desc: Option<String>,
    #[arg(
        short,
        long,
        help = "Repeat frequency (daily, weekly, monthly, yearly)"
    )]
    pub repeat: Option<String>,
    #[arg(short, long, help = "Repeat every N days/weeks/months/years")]
    pub every: Option<u32>,
    #[arg(short, long, help = "Repeat until this date")]
    pub until: Option<String>,
}

#[derive(Parser)]
pub struct EditArgs {
    pub event_id: String,
    #[arg(
        short,
        long,
        help = "The calendar to edit the event from (default: personal)"
    )]
    calendar: Option<String>,
    #[arg(short, long, help = "Name of the event")]
    name: Option<String>,
    #[arg(short, long, help = "New event start time")]
    at: Option<String>,
    #[arg(short, long, help = "New event end time")]
    to: Option<String>,
    #[arg(short, long, help = "New event location")]
    loc: Option<String>,
    #[arg(short, long, help = "New event description")]
    desc: Option<String>,
}

#[derive(Parser)]
pub struct DeleteArgs {
    pub event_id: String,
    #[arg(short, long, help = "Specify the calendar")]
    calendar: Option<String>,
    #[arg(short, long, help = "Delete without confirmation")]
    force: bool,
}

#[derive(Parser)]
pub struct ShowArgs {
    pub event_id: String,
    #[arg(short, long, help = "Specify the calendar to show from")]
    calendar: Option<String>,
}

#[derive(Parser)]
pub struct ViewArgs {
    #[arg(help = "Specify the date for which the calendar will be run")]
    pub date: Option<String>,
    #[arg(
        short,
        long,
        default_value = "month",
        help = "View mode: day, week, month"
    )]
    mode: String,
    #[arg(short, long, help = "Specify the calendar to view")]
    calendar: Option<String>,
    #[arg(short, long, help = "Show n times")]
    number: Option<u32>,
}

#[derive(Parser)]
pub struct SyncArgs {
    #[arg(long, help = "Specify the calendar to sync")]
    calendar: Option<String>,
}

// Helper functions

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    Ok(CalendarDate::parse(date_str)?.inner())
}

fn parse_datetime(datetime_str: &str) -> Result<NaiveDateTime> {
    Ok(CalendarDateTime::parse(datetime_str)?.inner())
}

fn parse_uuid(uuid_str: &str) -> Result<Uuid> {
    Uuid::parse_str(uuid_str).map_err(|e| anyhow!("Invalid UUID: {}", e))
}

impl ListArgs {
    pub fn validate(self) -> Result<CalendarListArgs> {
        let query: Option<String> = Some(self.query.join(" "))
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_owned());

        let now = chrono::Local::now().naive_local().date();
        let from = self
            .from
            .map(|d| parse_date(&d))
            .transpose()?
            .unwrap_or(now);
        let to = self
            .to
            .map(|d| parse_date(&d))
            .transpose()?
            .unwrap_or(now + Duration::days(30));

        if to < from {
            return Err(anyhow!("'to' date must be after 'from' date"));
        }

        Ok(CalendarListArgs {
            query,
            from,
            to,
            limit: self.limit,
            id: self.id,
            calendar: self.calendar,
        })
    }
}

impl AddArgs {
    pub fn validate(self) -> Result<CalendarAddArgs> {
        let calendar = self.calendar.unwrap_or_else(|| "personal".to_string());
        let name = self.name.join(" ");

        if name.trim().is_empty() {
            return Err(anyhow!("Name cannot be empty"));
        }

        let start = parse_datetime(&self.at)?;
        let end = self
            .to
            .map(|t| parse_datetime(&t))
            .transpose()?
            .unwrap_or(start + Duration::hours(1));

        if end < start {
            return Err(anyhow!("End time must be after start time"));
        }

        let repeat = self
            .repeat
            .map(|r| RepeatFrequency::from_str(&r))
            .transpose()?;
        let until = self.until.map(|u| parse_date(&u)).transpose()?;

        let every = if repeat.is_some() {
            Some(self.every.unwrap_or(1))
        } else {
            None
        };

        if repeat.is_none() {
            if self.every.is_some() {
                return Err(anyhow!("'repeat' must be specified when using 'every'"));
            }
            if until.is_some() {
                return Err(anyhow!("'repeat' must be specified when using 'until'"));
            }
        }

        Ok(CalendarAddArgs {
            calendar,
            name,
            start,
            end,
            loc: self.loc,
            desc: self.desc,
            repeat,
            every,
            until,
        })
    }
}

impl EditArgs {
    pub fn validate(self) -> Result<CalendarEditArgs> {
        let calendar = self.calendar.unwrap_or_else(|| "personal".to_string());

        let event_id = parse_uuid(&self.event_id)?;
        let start = self.at.map(|w| parse_datetime(&w)).transpose()?;
        let end = self.to.map(|t| parse_datetime(&t)).transpose()?;

        if let (Some(start), Some(end)) = (start, end) {
            if end < start {
                return Err(anyhow!("End time must be after start time"));
            }
        }

        Ok(CalendarEditArgs {
            event_id,
            calendar,
            name: self.name,
            start,
            end,
            loc: self.loc,
            desc: self.desc,
        })
    }
}

impl DeleteArgs {
    pub fn validate(self) -> Result<CalendarDeleteArgs> {
        let calendar = self.calendar.unwrap_or_else(|| "personal".to_string());
        let event_id = parse_uuid(&self.event_id)?;
        Ok(CalendarDeleteArgs {
            event_id,
            calendar,
            force: self.force,
        })
    }
}

impl ViewArgs {
    pub fn validate(self) -> Result<CalendarViewArgs> {
        let date = self
            .date
            .map(|d| parse_date(&d))
            .transpose()?
            .unwrap_or_else(|| chrono::Local::now().naive_local().date());
        let mode = ViewMode::from_str(&self.mode)?;

        let number = self.number.unwrap_or(1);

        Ok(CalendarViewArgs {
            date,
            mode,
            calendar: self.calendar,
            number,
        })
    }
}

impl ShowArgs {
    pub fn validate(self) -> Result<CalendarShowArgs> {
        let calendar = self.calendar.unwrap_or_else(|| "personal".to_string());
        let event_id = parse_uuid(&self.event_id)?;
        Ok(CalendarShowArgs { event_id, calendar })
    }
}

impl SyncArgs {
    pub fn validate(self) -> Result<CalendarSyncArgs> {
        Ok(CalendarSyncArgs {
            calendar: self.calendar,
        })
    }
}

pub fn parse_cli() -> Result<CalendarCommand> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::View(ViewArgs {
        date: None,
        mode: "month".to_string(),
        calendar: None,
        number: None,
    })) {
        Commands::List(args) => args.validate().map(CalendarCommand::List),
        Commands::Add(args) => args.validate().map(CalendarCommand::Add),
        Commands::Edit(args) => args.validate().map(CalendarCommand::Edit),
        Commands::Delete(args) => args.validate().map(CalendarCommand::Delete),
        Commands::View(args) => args.validate().map(CalendarCommand::View),
        Commands::Show(args) => args.validate().map(CalendarCommand::Show),
        Commands::Sync(args) => args.validate().map(CalendarCommand::Sync),
    }
}

pub enum CalendarCommand {
    List(CalendarListArgs),
    Add(CalendarAddArgs),
    Edit(CalendarEditArgs),
    Delete(CalendarDeleteArgs),
    Show(CalendarShowArgs),
    View(CalendarViewArgs),
    Sync(CalendarSyncArgs),
}
