mod calendar;
mod cli;
mod date;
mod event;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = cli::parse_cli()?;

    match command {
        cli::CalendarCommand::List(args) => {
            event::list(args);
        }
        cli::CalendarCommand::Add(args) => {
            println!("Calendar: {:?}", args.calendar);
            println!("Name: {:?}", args.name);
            println!("Start: {:?}", args.start);
            println!("End: {:?}", args.end);
            println!("Loc: {:?}", args.loc);
            println!("Desc: {:?}", args.desc);
            println!("Repeat: {:?}", args.repeat);
            println!("Every: {:?}", args.every);
            println!("Until: {:?}", args.until);
        }
        cli::CalendarCommand::Edit(args) => {
            println!("EventId: {:?}", args.event_id);
            println!("Calendar: {:?}", args.calendar);
            println!("Name: {:?}", args.name);
            println!("Start: {:?}", args.start);
            println!("End: {:?}", args.end);
            println!("Loc: {:?}", args.loc);
            println!("Desc: {:?}", args.desc);
            println!("Repeat: {:?}", args.repeat);
            println!("Every: {:?}", args.every);
            println!("Until: {:?}", args.until);
        }
        cli::CalendarCommand::Delete(args) => {
            println!("EventId: {:?}", args.event_id);
            println!("Calendar: {:?}", args.calendar);
            println!("Force: {:?}", args.force);
        }
        cli::CalendarCommand::Show(args) => {
            println!("EventId: {:?}", args.event_id);
            println!("Calendar: {:?}", args.calendar);
        }
        cli::CalendarCommand::View(args) => {
            println!("Date: {:?}", args.date);
            println!("Mode: {:?}", args.mode);
            println!("Calendar: {:?}", args.calendar);
        }
        cli::CalendarCommand::Sync(args) => {
            println!("Calendar: {:?}", args.calendar);
        }
    }

    Ok(())
}
