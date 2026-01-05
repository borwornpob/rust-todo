# Todo CLI

A fast, minimal todo app for the command line, written in Rust.

## Features

- Simple numeric selection (`todo done 1` instead of copying long IDs)
- Color-coded output (pending/done status)
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
| `add <title>` | `a` | Add a new todo |
| `list` | `l`, `ls` | List all todos |
| `done <#>` | `d` | Mark a todo as done |
| `undone <#>` | `u` | Mark a todo as pending |
| `edit <#> <title>` | `e` | Edit a todo's title |
| `rm <#>` | `r` | Remove a todo |
| `clear` | | Remove all completed todos |
| `help` | | Show help |

### Examples

```bash
# Add todos
todo add "Buy groceries"
todo a "Learn Rust"

# List todos
todo list

# Mark as done
todo done 1

# Edit a todo
todo edit 2 "Master Rust"

# Remove a todo
todo rm 1

# Clear all completed
todo clear
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

## License

MIT
