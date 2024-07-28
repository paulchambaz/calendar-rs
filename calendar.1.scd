CALENDAR(1) "https://github.com/username/calendar-rs" "General Commands Manual"

# NAME

*calendar-rs* - A simple CLI tool for managing calendars from the terminal

# SYNOPSIS

*calendar* [OPTIONS] [COMMAND]

# DESCRIPTION

*calendar-rs* is a lightweight, terminal-based calendar management tool designed to work with vdirsyncer for synchronization. It allows users to view, add, edit, and delete calendar events directly from the command line.

# CONFIGURATION

*calendar-rs* relies on vdirsyncer for calendar synchronization. Before using *calendar-rs*, ensure you have vdirsyncer installed and configured. A typical vdirsyncer configuration file (*~/.config/vdirsyncer/config*) might look like this:

```
[general]
status_path = "~/.vdirsyncer/status/"

[pair personal]
a = "personal_local"
b = "personal_remote"
collections = ["from a", "from b"]

[storage personal_local]
type = "filesystem"
path = "~/.calendars/personal/"
fileext = ".ics"

[storage personal_remote]
type = "caldav"
url = "https://your-caldav-server.com/your-calendar-path/"
username = "your-username"
password = "your-password"

[pair university]
a = "university_local"
b = "university_remote"
collections = ["from b"]

[storage university_local]
type = "filesystem"
path = "~/.calendars/university/"
fileext = ".ics"

[storage university_remote]
type = "caldav"
url = "https://your-university-caldav-server.com/your-degree-calendar-path/"
username = "university-username"
password = "university-password"

# Additional calendar pairs can be defined similarly
```

Ensure that your main personal calendar is stored under *~/.calendars/personal/*, as this is the assumed default calendar for *calendar-rs*.

# OPTIONS

*-m*, *--mode* <MODE>
	View mode: day, week, month

*-c*, *--calendar* <CALENDAR>
	Specify the calendar to view

*-n*, *--number* <NUMBER>
	Show n times

*-h*, *--help*
	Print help information

*-V*, *--version*
	Print version information

# COMMANDS

## list [OPTIONS] [QUERY]...

List events from all or specific calendars.

*-c*, *--calendar* <CALENDAR>
	Specify the calendar to list (default: all)

*-f*, *--from* <FROM>
	Start date for listing (default: today)

*-t*, *--to* <TO>
	End date for listing (default: 1 month from today)

*-l*, *--limit* <LIMIT>
	Limit the number of events shown

*-i*, *--id*
	Show the UUID of the tasks for future modification

## add [OPTIONS] --at <AT> <NAME>...

Add a new event to a calendar.

*-a*, *--at* <AT>
	Event start time (e.g., tom@21, 14-jul@12:30, 2024/08/06@08:00)

*-t*, *--to* <TO>
	Event end time (default: 1 hour after start)

*-c*, *--calendar* <CALENDAR>
	The calendar to add the event to (default: personal)

*-l*, *--loc* <LOC>
	Event location

*-d*, *--desc* <DESC>
	Event description

*-r*, *--repeat* <REPEAT>
	Repeat frequency (daily, weekly, monthly, yearly)

*-e*, *--every* <EVERY>
	Repeat every N days/weeks/months/years

*-u*, *--until* <UNTIL>
	Repeat until this date

## edit [OPTIONS] <EVENT_ID>

Edit an existing event.

*-c*, *--calendar* <CALENDAR>
	The calendar to edit the event from (default: personal)

*-n*, *--name* <NAME>
	Name of the event

*-a*, *--at* <AT>
	New event start time

*-t*, *--to* <TO>
	New event end time

*-l*, *--loc* <LOC>
	New event location

*-d*, *--desc* <DESC>
	New event description

## delete [OPTIONS] <EVENT_ID>

Delete an event.

*-c*, *--calendar* <CALENDAR>
	Specify the calendar

*-f*, *--force*
	Delete without confirmation

## show [OPTIONS] <EVENT_ID>

Show details of a specific event.

*-c*, *--calendar* <CALENDAR>
	Specify the calendar to show from

## view [OPTIONS] [DATE]

Display calendar in various formats (daily, weekly, monthly).

*-m*, *--mode* <MODE>
	View mode: day, week, month (default: month)

*-c*, *--calendar* <CALENDAR>
	Specify the calendar to view

*-n*, *--number* <NUMBER>
	Show n times

## sync [OPTIONS]

Synchronize calendars using vdirsyncer.

*--calendar* <CALENDAR>
	Specify the calendar to sync

# DATE AND TIME FORMATS

*calendar-rs* supports various date and time formats for flexible input:

1. Full date: YYYY-MM-DD, DD-MM-YYYY, YYYY/MM/DD, DD/MM/YYYY (e.g., 2023-07-28)
2. Short date: MM-DD (e.g., 07-28)
3. Relative date: yesterday, yes, today, tomorrow, tom, Nd, Nw, Nm, Ny (e.g., 3d for 3 days from today)
4. Weekday: monday, mon, tuesday, tue, etc.
5. Month name: january, jan, february, feb, etc.
6. Day-Month: DD-MMM (e.g., 28-Jul)
7. Month-Day: MMM-DD (e.g., Jul-28)
8. Time: HH:MM[:SS] or single-digit hour (e.g., 14:30, 14:30:00, 2)
9. DateTime: any supported date format followed by '@' and a time format (e.g., tom@14:30, 2023-07-28@9)

# EXAMPLES

Here's a typical workflow demonstrating the features of calendar-rs:

1. View this month's calendar:
```
$ calendar
```

2. Add a simple event for tomorrow at 2 PM:
```
$ calendar add "Team Meeting" --at tom@14
```

3. Add a more detailed event with location and description:
```
$ calendar add "Project Kickoff" --at "jan-08@10:00" --to "jan-08@11:30" --loc "Conference Room A" --desc "Initial meeting for the new project"
```

4. Add a recurring event:
```
$ calendar add "Weekly Stand-up" --at "mon@09:30" --repeat weekly --until "2024-12-31"
```

5. List events for the next week:
```
$ calendar list --from today --to 7d
```

6. List events with a specific keyword:
```
$ calendar list meeting
```

7. View events for a specific date range, and show their UUID:
```
$ calendar list --from "2023-08-01" --to "2023-08-15" --id
```

8. Edit an event (replace <EVENT_ID> with the actual UUID):
```
$ calendar edit <EVENT_ID> --name "Updated Meeting" --at "2023-08-01@15:30" --loc "Virtual Conference"
```

9. Show details of a specific event:
```
$ calendar show <EVENT_ID>
```

10. Delete an event:
```
$ calendar delete <EVENT_ID>
```

11. View a specific day's schedule:
```
$ calendar view 2023-08-01 --mode day
```

12. View next week's schedule:
```
$ calendar view --mode week -n 2
```

13. View calendar for a specific month:
```
$ calendar view december --mode month
```

14. Sync all calendars:
```
$ calendar sync
```

15. Sync a specific calendar:
```
$ calendar sync --calendar work
```

16. Add an event using various date formats:
```
$ calendar add "Doctor Appointment" --at tuesday@14:30
$ calendar add "Sister's Birthday" --at 5-aug@12
$ calendar add "Project Deadline" --at 2023-09-30@18:00
```

17. List events with limit:
```
$ calendar list --limit 5
```

18. View multiple months:
```
$ calendar view --mode month --number 3
```

# NOTES

- *calendar-rs* does not handle the creation or deletion of calendars themselves, only the management of events within existing calendars.
- The sync command requires vdirsyncer to be installed and properly configured.
- By default, *calendar-rs* uses the calendars under ~/.calendars/ with ~/.calendars/personal/ assumed to be the default calendar.

# AUTHORS

This program was written by Paul Chambaz in 2024. It is licensed under the GPLv3, for more information, please read the *LICENSE* file.

# SEE ALSO

vdirsyncer(1)
