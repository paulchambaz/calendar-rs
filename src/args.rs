use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "calendar")]
#[command(author, version, about = "Calendar-rs is a small cli to handle your calendars from the terminal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    
    // Global options for when no subcommand is given (default view mode)
    #[arg(short, long, help = "View mode: day, week, month")]
    mode: Option<String>,
    #[arg(short, long, help = "Specify the calendar to view")]
    calendar: Option<String>,
    #[arg(short, long, help = "Show n times")]
    number: Option<u32>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List events from all or specific calendars
    List {
        /// Query terms in the fzf search
        query: Vec<String>,
        /// Specify the calendar to list (default: all)
        #[arg(short, long)]
        calendar: Option<String>,
        /// Start date for listing (default: today)
        #[arg(short, long)]
        from: Option<String>,
        /// End date for listing (default: 1 month from today)
        #[arg(short, long)]
        to: Option<String>,
        /// Limit the number of events shown
        #[arg(short, long)]
        limit: Option<usize>,
        /// Show the uuid of the tasks for future modification
        #[arg(short, long)]
        id: bool,
    },
    
    /// Add a new event to a calendar
    Add {
        /// Name of the event
        name: Vec<String>,
        /// Event start time (eg. tom@21 14-jul@12:30 2024/08/06@08:00)
        #[arg(short, long)]
        at: String,
        /// Event end time (default: 1 hour after start)
        #[arg(short, long)]
        to: Option<String>,
        /// The calendar to add the event to (default: personal)
        #[arg(short, long)]
        calendar: Option<String>,
        /// Event location
        #[arg(short, long)]
        loc: Option<String>,
        /// Event description
        #[arg(short, long)]
        desc: Option<String>,
        /// Repeat frequency (daily, weekly, monthly, yearly)
        #[arg(short, long)]
        repeat: Option<String>,
        /// Repeat every N days/weeks/months/years
        #[arg(short, long)]
        every: Option<u32>,
        /// Repeat until this date
        #[arg(short, long)]
        until: Option<String>,
    },
    
    /// Display calendar in various formats (daily, weekly, monthly)
    View {
        /// Specify the date for which the calendar will be run
        date: Option<String>,
        /// View mode: day, week, month
        #[arg(short, long, default_value = "month")]
        mode: String,
        /// Specify the calendar to view
        #[arg(short, long)]
        calendar: Option<String>,
        /// Show n times
        #[arg(short, long)]
        number: Option<u32>,
    },
    
    /// Edit an existing event
    Edit {
        /// Event ID to edit
        event_id: String,
        /// The calendar to edit the event from (default: personal)
        #[arg(short, long)]
        calendar: Option<String>,
        /// Name of the event
        #[arg(short, long)]
        name: Option<String>,
        /// New event start time
        #[arg(short, long)]
        at: Option<String>,
        /// New event end time
        #[arg(short, long)]
        to: Option<String>,
        /// New event location
        #[arg(short, long)]
        loc: Option<String>,
        /// New event description
        #[arg(short, long)]
        desc: Option<String>,
    },
    
    /// Delete an event
    Delete {
        /// Event ID to delete
        event_id: String,
        /// Specify the calendar
        #[arg(short, long)]
        calendar: Option<String>,
        /// Delete without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show details of a specific event
    Show {
        /// Event ID to show
        event_id: String,
        /// Specify the calendar to show from
        #[arg(short, long)]
        calendar: Option<String>,
    },
    
    /// Synchronize calendars using vdirsyncer
    Sync {
        /// Specify the calendar to sync
        #[arg(long)]
        calendar: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub enum ViewMode {
    Day,
    Week,
    Month,
}

impl std::str::FromStr for ViewMode {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "day" | "d" => Ok(ViewMode::Day),
            "week" | "w" => Ok(ViewMode::Week),
            "month" | "m" => Ok(ViewMode::Month),
            _ => Err(format!("Invalid view mode: {}. Use day, week, or month", s)),
        }
    }
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Month
    }
}

/// Parse command line arguments
pub fn parse() -> Cli {
    Cli::parse()
}

/// Get the command to run, defaulting to View if none specified
pub fn get_command(cli: Cli) -> Command {
    cli.command.unwrap_or(Command::View {
        date: None,
        mode: cli.mode.unwrap_or("month".to_string()),
        calendar: cli.calendar,
        number: cli.number,
    })
}
