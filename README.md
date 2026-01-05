# Todo CLI

A fast, minimal todo app for the command line with native macOS notifications.

## Features

- Simple numeric selection (`todo done 1` instead of copying long IDs)
- Color-coded output (pending/done status)
- **Reminders with native macOS notifications**
- Persistent storage using embedded database
- Single global database across your machine
- Short command aliases (`a`, `l`, `d`, `e`, `r`)

## Installation

```bash
# Clone and install
git clone <repo-url>
cd rust-todo
cargo install --path .

# Create shortcut (optional)
ln -sf ~/.cargo/bin/rust-todo ~/.cargo/bin/todo
```

## Usage

```
todo <command> [arguments]
```

### Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `add <title> [-r <time>]` | `a` | Add a new todo (with optional reminder) |
| `list` | `l`, `ls` | List all todos |
| `done <#>` | `d` | Mark a todo as done |
| `undone <#>` | `u` | Mark a todo as pending |
| `edit <#> <title>` | `e` | Edit a todo's title |
| `remind <#> <time>` | | Set or clear a reminder |
| `rm <#>` | `r` | Remove a todo |
| `clear` | | Remove all completed todos |
| `notify` | | Check and send due notifications |
| `help` | | Show help |

### Examples

```bash
# Add todos
todo add "Buy groceries"
todo add "Meeting with team" -r 2h    # remind in 2 hours
todo add "Call mom" --remind 14:30    # remind at 2:30 PM

# List todos
todo list

# Mark as done
todo done 1

# Set/change reminder on existing todo
todo remind 2 30m       # remind in 30 minutes
todo remind 2 tomorrow  # remind tomorrow
todo remind 2 clear     # remove reminder

# Check for due reminders (sends macOS notifications)
todo notify
```

## Reminders

### Time Formats

| Format | Description |
|--------|-------------|
| `15m` | 15 minutes from now |
| `2h` | 2 hours from now |
| `1d` | 1 day from now |
| `1w` | 1 week from now |
| `14:30` | At 2:30 PM today (or tomorrow if time passed) |
| `tomorrow` | Tomorrow at current time |

### Setting Up Notifications

The `todo notify` command checks for due reminders and sends macOS notifications. Run it periodically using cron or launchd.

#### Option 1: Crontab (runs every minute)

```bash
crontab -e
```

Add this line:
```
* * * * * /Users/YOUR_USERNAME/.cargo/bin/todo notify
```

#### Option 2: launchd (recommended for macOS)

Create `~/Library/LaunchAgents/com.todo.notify.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.todo.notify</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOUR_USERNAME/.cargo/bin/todo</string>
        <string>notify</string>
    </array>
    <key>StartInterval</key>
    <integer>60</integer>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
```

Load it:
```bash
launchctl load ~/Library/LaunchAgents/com.todo.notify.plist
```

To unload:
```bash
launchctl unload ~/Library/LaunchAgents/com.todo.notify.plist
```

## Data Storage

Database location: `~/.local/share/todo/todo.db`

The database is global - your todos are accessible from any directory.

## Upgrading

After making changes to the source code:

```bash
cd /path/to/rust-todo
cargo install --path .
```

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- [PoloDB](https://github.com/PoloDB/polodb) - Embedded document database
- [colored](https://github.com/colored-rs/colored) - Terminal colors
- [chrono](https://github.com/chronotope/chrono) - Date/time handling
- macOS native notifications via `osascript`

## License

MIT
