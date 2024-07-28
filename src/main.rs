mod calendar;
mod cli;
mod date;
mod event;
mod storage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = cli::parse_cli()?;

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
