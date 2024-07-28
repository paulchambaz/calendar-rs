mod calendar;
mod cli;
mod date;
mod event;
mod storage;
use std::fs;

use anyhow::{anyhow, Result};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = cli::parse_cli()?;

    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    let calendar_dir = home_dir.join(".calendars");
    fs::create_dir_all(&calendar_dir)?;

    match command {
        cli::CalendarCommand::List(args) => {
            event::list(args)?;
        }
        cli::CalendarCommand::Add(args) => {
            event::add(args)?;
        }
        cli::CalendarCommand::Edit(args) => {
            event::edit(args)?;
        }
        cli::CalendarCommand::Delete(args) => {
            event::delete(args)?;
        }
        cli::CalendarCommand::Show(args) => {
            event::show(args)?;
        }
        cli::CalendarCommand::View(args) => {
            event::view(args)?;
        }
        cli::CalendarCommand::Sync(args) => {
            event::sync(args)?;
        }
    }

    Ok(())
}
