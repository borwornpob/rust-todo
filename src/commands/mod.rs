use anyhow::{anyhow, Result};

use crate::db::TodoDb;
use crate::display::{
    print_added_todo, print_info, print_success, print_todo_table, print_warning,
};
use crate::models::Todo;
use crate::remind::{format_remind_at, parse_reminder, send_notification};

fn get_todo_by_index(db: &TodoDb, index_str: &str) -> Result<(usize, Todo)> {
    let index: usize = index_str
        .parse()
        .map_err(|_| anyhow!("Invalid number: {}. Use a number like 1, 2, 3...", index_str))?;

    if index == 0 {
        return Err(anyhow!("Todo numbers start at 1"));
    }

    let todos = db.list_all()?;
    let actual_index = index - 1;

    if actual_index >= todos.len() {
        return Err(anyhow!(
            "Todo #{} not found. You have {} todos.",
            index,
            todos.len()
        ));
    }

    Ok((index, todos[actual_index].clone()))
}

/// Parse args to extract --remind or -r flag and its value
fn extract_reminder(args: &[String]) -> (Vec<String>, Option<String>) {
    let mut remaining = Vec::new();
    let mut reminder = None;
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--remind" || args[i] == "-r" {
            if i + 1 < args.len() {
                reminder = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if args[i].starts_with("--remind=") {
            reminder = Some(args[i].trim_start_matches("--remind=").to_string());
            i += 1;
            continue;
        } else if args[i].starts_with("-r=") {
            reminder = Some(args[i].trim_start_matches("-r=").to_string());
            i += 1;
            continue;
        }
        remaining.push(args[i].clone());
        i += 1;
    }

    (remaining, reminder)
}

pub fn cmd_add(db: &TodoDb, args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!(
            "Missing title. Usage: todo add \"your task\" [--remind 15m]"
        ));
    }

    let (title_args, reminder_str) = extract_reminder(&args);

    let title = title_args.join(" ").trim().to_string();
    if title.is_empty() {
        return Err(anyhow!("Title cannot be empty"));
    }

    let todo = if let Some(ref remind_str) = reminder_str {
        let remind_at = parse_reminder(remind_str)?;
        Todo::with_reminder(title.clone(), remind_at)
    } else {
        Todo::new(title.clone())
    };

    db.insert(&todo)?;

    let todos = db.list_all()?;
    let index = todos.len();

    print_added_todo(index, &title);

    if let Some(remind_at) = &todo.remind_at {
        print_info(&format!("  Reminder: {}", format_remind_at(remind_at)));
    }

    Ok(())
}

pub fn cmd_list(db: &TodoDb) -> Result<()> {
    let todos = db.list_all()?;
    print_todo_table(&todos);
    Ok(())
}

pub fn cmd_done(db: &TodoDb, args: Vec<String>) -> Result<()> {
    let index_str = args
        .first()
        .ok_or_else(|| anyhow!("Missing todo number. Usage: todo done <#>"))?;
    let (index, todo) = get_todo_by_index(db, index_str)?;

    if todo.done {
        print_warning(&format!("Todo #{} is already done", index));
        return Ok(());
    }

    db.mark_done(&todo.id)?;
    print_success(&format!("Marked #{} as done: {}", index, todo.title));
    Ok(())
}

pub fn cmd_undone(db: &TodoDb, args: Vec<String>) -> Result<()> {
    let index_str = args
        .first()
        .ok_or_else(|| anyhow!("Missing todo number. Usage: todo undone <#>"))?;
    let (index, todo) = get_todo_by_index(db, index_str)?;

    if !todo.done {
        print_warning(&format!("Todo #{} is not marked as done", index));
        return Ok(());
    }

    db.mark_undone(&todo.id)?;
    print_success(&format!("Marked #{} as pending: {}", index, todo.title));
    Ok(())
}

pub fn cmd_edit(db: &TodoDb, args: Vec<String>) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!(
            "Missing arguments. Usage: todo edit <#> \"new title\""
        ));
    }

    let index_str = &args[0];
    let (index, todo) = get_todo_by_index(db, index_str)?;

    let new_title = args[1..].join(" ").trim().to_string();
    if new_title.is_empty() {
        return Err(anyhow!("New title cannot be empty"));
    }

    let old_title = todo.title.clone();
    db.update_title(&todo.id, &new_title)?;

    print_info(&format!("Updated #{}", index));
    print_info(&format!("  Old: {}", old_title));
    print_success(&format!("  New: {}", new_title));
    Ok(())
}

pub fn cmd_remind(db: &TodoDb, args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!(
            "Usage: todo remind <#> <time>  or  todo remind <#> clear"
        ));
    }

    let index_str = &args[0];
    let (index, todo) = get_todo_by_index(db, index_str)?;

    if args.len() < 2 {
        // Show current reminder
        if let Some(remind_at) = &todo.remind_at {
            print_info(&format!(
                "Todo #{} reminder: {}",
                index,
                format_remind_at(remind_at)
            ));
        } else {
            print_info(&format!("Todo #{} has no reminder", index));
        }
        return Ok(());
    }

    let time_str = &args[1];

    if time_str == "clear" || time_str == "off" || time_str == "none" {
        db.clear_reminder(&todo.id)?;
        print_success(&format!("Cleared reminder for #{}: {}", index, todo.title));
        return Ok(());
    }

    let remind_at = parse_reminder(time_str)?;
    db.set_reminder(&todo.id, Some(remind_at))?;

    print_success(&format!(
        "Set reminder for #{}: {} ({})",
        index,
        todo.title,
        format_remind_at(&remind_at)
    ));

    Ok(())
}

pub fn cmd_remove(db: &TodoDb, args: Vec<String>) -> Result<()> {
    let index_str = args
        .first()
        .ok_or_else(|| anyhow!("Missing todo number. Usage: todo rm <#>"))?;
    let (index, todo) = get_todo_by_index(db, index_str)?;

    db.delete(&todo.id)?;
    print_success(&format!("Removed #{}: {}", index, todo.title));
    Ok(())
}

pub fn cmd_clear(db: &TodoDb) -> Result<()> {
    let todos = db.list_all()?;
    let completed: Vec<_> = todos.iter().filter(|t| t.done).collect();

    if completed.is_empty() {
        print_warning("No completed todos to clear");
        return Ok(());
    }

    let count = completed.len();
    for todo in completed {
        db.delete(&todo.id)?;
    }

    print_success(&format!("Cleared {} completed todo(s)", count));
    Ok(())
}

/// Check for due reminders and send notifications (one-shot, for cron/launchd)
pub fn cmd_notify(db: &TodoDb) -> Result<()> {
    let due = db.get_due_reminders()?;

    if due.is_empty() {
        return Ok(());
    }

    for todo in due {
        if send_notification(&todo.title, "Time for your todo!").is_ok() {
            db.mark_notified(&todo.id)?;
            print_info(&format!("Notified: {}", todo.title));
        }
    }

    Ok(())
}
