mod args;
mod storage {
    pub mod files;
    pub mod ics;
}
mod calendar {
    pub mod events;
    pub mod recurring;
}
mod display {
    pub mod list;
    pub mod calendar;
    pub mod event;
}
mod commands;
mod sync;

use args::{Command, ViewMode, parse, get_command};
use commands::*;

fn main() {
    let cli = parse();
    let command = get_command(cli);

    let result = match command {
        Command::List { query, calendar, from, to, limit, id } => {
            handle_list(query, calendar, from, to, limit, id)
        }
        
        Command::Add { name, at, to, calendar, loc, desc, repeat, every, until } => {
            handle_add(name, at, to, calendar, loc, desc, repeat, every, until)
        }
        
        Command::Edit { event_id, calendar, name, at, to, loc, desc } => {
            let calendar_name = calendar.unwrap_or_else(|| "personal".to_string());
            handle_edit(event_id, calendar_name, name, at, to, loc, desc)
        }
        
        Command::Delete { event_id, calendar, force } => {
            let calendar_name = calendar.unwrap_or_else(|| "personal".to_string());
            handle_delete(event_id, calendar_name, force)
        }
        
        Command::Show { event_id, calendar } => {
            let calendar_name = calendar.unwrap_or_else(|| "personal".to_string());
            handle_show(event_id, calendar_name)
        }
        
        Command::View { date, mode, calendar, number } => {
            mode.parse::<ViewMode>()
                .and_then(|view_mode| handle_view(date, view_mode, calendar, number))
        }
        
        Command::Sync { calendar } => {
            if sync::check_vdirsyncer_available() {
                sync::sync_calendar(calendar)
            } else {
                Err("vdirsyncer is not installed or not in PATH".to_string())
            }
        }
    };

    if let Err(error) = result {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}
