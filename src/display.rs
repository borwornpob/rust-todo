use chrono::{Local, TimeZone};
use colored::Colorize;
use polodb_core::bson::DateTime as BsonDateTime;

use crate::models::Todo;

fn format_datetime(dt: &BsonDateTime) -> String {
    let millis = dt.timestamp_millis();
    let secs = millis / 1000;
    let nsecs = ((millis % 1000) * 1_000_000) as u32;

    if let Some(local_dt) = Local.timestamp_opt(secs, nsecs).single() {
        local_dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        "unknown".to_string()
    }
}

fn format_reminder(dt: &BsonDateTime) -> String {
    let millis = dt.timestamp_millis();
    let secs = millis / 1000;
    let nsecs = ((millis % 1000) * 1_000_000) as u32;

    if let Some(local_dt) = Local.timestamp_opt(secs, nsecs).single() {
        let now = Local::now();
        let diff = local_dt.signed_duration_since(now);

        if diff.num_seconds() < 0 {
            "overdue".to_string()
        } else if diff.num_minutes() < 1 {
            "now".to_string()
        } else if diff.num_minutes() < 60 {
            format!("{}m", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h", diff.num_hours())
        } else {
            format!("{}d", diff.num_days())
        }
    } else {
        "?".to_string()
    }
}

pub fn print_todo_table(todos: &[Todo]) {
    if todos.is_empty() {
        println!(
            "{}",
            "No todos yet. Add one with: todo add \"your task\"".yellow()
        );
        return;
    }

    // Calculate column widths
    let max_title_len = todos
        .iter()
        .map(|t| t.title.len())
        .max()
        .unwrap_or(5)
        .max(5);
    let title_width = max_title_len.min(40);

    let has_reminders = todos.iter().any(|t| t.remind_at.is_some());

    // Print header
    println!();
    if has_reminders {
        println!(
            "  {}  {}  {:title_width$}  {}  {}",
            "#".dimmed(),
            "Status".dimmed(),
            "Title".dimmed(),
            "Remind".dimmed(),
            "Created".dimmed(),
            title_width = title_width
        );
        println!("  {}", "─".repeat(4 + 8 + title_width + 10 + 18).dimmed());
    } else {
        println!(
            "  {}  {}  {:title_width$}  {}",
            "#".dimmed(),
            "Status".dimmed(),
            "Title".dimmed(),
            "Created".dimmed(),
            title_width = title_width
        );
        println!("  {}", "─".repeat(4 + 8 + title_width + 18).dimmed());
    }

    // Print rows
    for (i, todo) in todos.iter().enumerate() {
        let index = format!("{:>2}", i + 1);
        let status = if todo.done {
            " ✓ ".green()
        } else {
            " ○ ".yellow()
        };

        let title = if todo.done {
            let truncated = truncate_str(&todo.title, title_width);
            format!("{:title_width$}", truncated)
                .dimmed()
                .strikethrough()
        } else {
            let truncated = truncate_str(&todo.title, title_width);
            format!("{:title_width$}", truncated).normal()
        };

        let created = format_datetime(&todo.created_at).dimmed();

        if has_reminders {
            let remind = if let Some(ref r) = todo.remind_at {
                let r_str = format_reminder(r);
                if r_str == "overdue" {
                    format!("{:>7}", r_str).red()
                } else {
                    format!("{:>7}", r_str).magenta()
                }
            } else {
                format!("{:>7}", "-").dimmed()
            };

            println!(
                "  {}  {}   {}  {}  {}",
                index.cyan(),
                status,
                title,
                remind,
                created
            );
        } else {
            println!(
                "  {}  {}   {}  {}",
                index.cyan(),
                status,
                title,
                created
            );
        }
    }

    println!();

    // Summary
    let done_count = todos.iter().filter(|t| t.done).count();
    let pending_count = todos.len() - done_count;
    let reminder_count = todos
        .iter()
        .filter(|t| !t.done && t.remind_at.is_some())
        .count();

    print!("  ");
    if pending_count > 0 {
        print!("{} pending", pending_count.to_string().yellow());
    }
    if done_count > 0 {
        if pending_count > 0 {
            print!(" · ");
        }
        print!("{} done", done_count.to_string().green());
    }
    if reminder_count > 0 {
        print!(" · {} with reminders", reminder_count.to_string().magenta());
    }
    println!();
    println!();
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message.green());
}

pub fn print_error(message: &str) {
    println!("{} {}", "✗".red().bold(), message.red());
}

pub fn print_info(message: &str) {
    println!("{} {}", "→".cyan().bold(), message);
}

pub fn print_warning(message: &str) {
    println!("{} {}", "!".yellow().bold(), message.yellow());
}

pub fn print_added_todo(index: usize, title: &str) {
    println!(
        "{} Added todo #{}: {}",
        "✓".green().bold(),
        index.to_string().cyan(),
        title
    );
}

pub fn print_usage() {
    let title = "Todo CLI".cyan().bold();
    let version = "v0.3.0".dimmed();

    println!("\n{} {}\n", title, version);
    println!("{}", "USAGE:".yellow().bold());
    println!("    {} <command> [arguments]\n", "todo".green());

    println!("{}", "COMMANDS:".yellow().bold());
    println!("    {}   Add a new todo", "add <title> [-r <time>]".green());
    println!("    {}                  List all todos", "list".green());
    println!("    {}                Mark a todo as done", "done <#>".green());
    println!("    {}              Mark a todo as pending", "undone <#>".green());
    println!("    {}      Edit a todo's title", "edit <#> <title>".green());
    println!("    {}      Set/clear a reminder", "remind <#> <time>".green());
    println!("    {}                  Remove a todo", "rm <#>".green());
    println!("    {}                 Clear completed todos", "clear".green());
    println!(
        "    {}                Send due notifications",
        "notify".green()
    );
    println!("    {}                  Show this help", "help".green());

    println!("\n{}", "REMINDER FORMATS:".yellow().bold());
    println!("    {}            15 minutes from now", "15m".dimmed());
    println!("    {}             2 hours from now", "2h".dimmed());
    println!("    {}             1 day from now", "1d".dimmed());
    println!("    {}             1 week from now", "1w".dimmed());
    println!("    {}           At 2:30 PM today/tomorrow", "14:30".dimmed());
    println!("    {}        Tomorrow same time", "tomorrow".dimmed());

    println!("\n{}", "EXAMPLES:".yellow().bold());
    println!("    {} \"Buy groceries\"", "todo add".dimmed());
    println!("    {} \"Meeting\" -r 2h", "todo add".dimmed());
    println!("    {} 1 15m", "todo remind".dimmed());
    println!("    {} 1 clear", "todo remind".dimmed());

    println!("\n{}", "NOTIFICATIONS:".yellow().bold());
    println!("    Run {} periodically via cron or launchd", "todo notify".dimmed());
    println!("    Example crontab: {} todo notify", "* * * * *".dimmed());
    println!();
}
