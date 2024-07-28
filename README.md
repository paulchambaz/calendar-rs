# calendar-rs

A simple CLI tool for managing calendars from the terminal.

## Description

calendar-rs is a lightweight, terminal-based calendar management tool designed to work with vdirsyncer for synchronization. It allows users to view, add, edit, and delete calendar events directly from the command line.

## Installation

### Using Nix Flakes

If you have Nix with flakes enabled, you can run calendar-rs directly from the repository:

```
nix run github:paulchambaz/calendar-rs
```

You can also add it to your nixos configuration.

### Using Cargo

Alternatively, if you have Rust and Cargo installed, you can install calendar-rs using:

```
cargo install --git https://github.com/paulchambaz/calendar-rs.git
```

Make sure you have vdirsyncer installed and configured for calendar synchronization.

## Usage

For detailed usage instructions, please refer to the [man page](calendar.1.scd).

Basic usage:

```
calendar [OPTIONS] [COMMAND]
```

Common commands:

- `calendar list`: List events
- `calendar add`: Add a new event
- `calendar edit`: Edit an existing event
- `calendar delete`: Delete an event
- `calendar view`: Display calendar in various formats
- `calendar sync`: Synchronize calendars using vdirsyncer

## Quick Demo

1. View this month's calendar:

   ```
   calendar
   ```

2. Add a simple event for tomorrow at 2 PM:

   ```
   calendar add "Team Meeting" --at tom@14
   ```

3. List events for the next week:

   ```
   calendar list --from today --to 7d
   ```

4. Sync all calendars:
   ```
   calendar sync
   ```

For more examples and detailed usage, please consult the [man page](calendar.1.scd).

## License

This project is licensed under the GPLv3. See the [LICENSE](LICENSE) file for details.

## Authors

Written by Paul Chambaz in 2024.

## See Also

- [vdirsyncer](https://github.com/pimutils/vdirsyncer)
