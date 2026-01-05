use chrono::{Local, TimeZone};
use colored::Colorize;

use crate::models::Todo;

fn format_datetime(dt: &polodb_core::bson::DateTime) -> String {
    let millis = dt.timestamp_millis();
    let secs = millis / 1000;
    let nsecs = ((millis % 1000) * 1_000_000) as u32;

    if let Some(local_dt) = Local.timestamp_opt(secs, nsecs).single() {
        local_dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn print_todo_table(todos: &[Todo]) {
    if todos.is_empty() {
        println!("{}", "No todos yet. Add one with: todo add \"your task\"".yellow());
        return;
    }

    // Calculate column widths
    let max_title_len = todos.iter().map(|t| t.title.len()).max().unwrap_or(5).max(5);
    let title_width = max_title_len.min(50); // Cap at 50 chars

    // Print header
    println!();
    println!(
        "  {}  {}  {:title_width$}  {}",
        "#".dimmed(),
        "Status ".dimmed(),
        "Title".dimmed(),
        "Created".dimmed(),
        title_width = title_width
    );
    println!(
        "  {}",
        "─".repeat(4 + 8 + title_width + 18).dimmed()
    );

    // Print rows
    for (i, todo) in todos.iter().enumerate() {
        let index = format!("{:>2}", i + 1);
        let status = if todo.done {
            "  ✓   ".green()
        } else {
            "  ○   ".yellow()
        };

        let title = if todo.done {
            let truncated = truncate_str(&todo.title, title_width);
            format!("{:title_width$}", truncated).dimmed().strikethrough()
        } else {
            let truncated = truncate_str(&todo.title, title_width);
            format!("{:title_width$}", truncated).normal()
        };

        let created = format_datetime(&todo.created_at).dimmed();

        println!(
            "  {}  {}  {}  {}",
            index.cyan(),
            status,
            title,
            created
        );
    }

    println!();

    // Summary
    let done_count = todos.iter().filter(|t| t.done).count();
    let pending_count = todos.len() - done_count;

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
    let version = "v0.2.0".dimmed();

    println!("\n{} {}\n", title, version);
    println!("{}", "USAGE:".yellow().bold());
    println!("    {} <command> [arguments]\n", "todo".green());

    println!("{}", "COMMANDS:".yellow().bold());
    println!(
        "    {}          Add a new todo",
        "add <title>".green()
    );
    println!("    {}               List all todos", "list".green());
    println!("    {}             Mark a todo as done", "done <#>".green());
    println!(
        "    {}           Mark a todo as not done",
        "undone <#>".green()
    );
    println!(
        "    {} {}   Edit a todo's title",
        "edit <#>".green(),
        "<title>".green()
    );
    println!("    {}               Remove a todo", "rm <#>".green());
    println!(
        "    {}              Clear all completed todos",
        "clear".green()
    );
    println!("    {}               Show this help", "help".green());

    println!("\n{}", "EXAMPLES:".yellow().bold());
    println!("    {} \"Buy groceries\"", "todo add".dimmed());
    println!("    {}", "todo list".dimmed());
    println!("    {} 1", "todo done".dimmed());
    println!("    {} 2 \"Updated task\"", "todo edit".dimmed());
    println!("    {} 3", "todo rm".dimmed());
    println!();
}
