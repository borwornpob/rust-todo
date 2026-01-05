use anyhow::{anyhow, Result};

use crate::db::TodoDb;
use crate::display::{
    print_added_todo, print_info, print_success, print_todo_table, print_warning,
};
use crate::models::Todo;

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

pub fn cmd_add(db: &TodoDb, args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("Missing title. Usage: todo add \"your task\""));
    }

    let title = args.join(" ").trim().to_string();
    if title.is_empty() {
        return Err(anyhow!("Title cannot be empty"));
    }

    let todo = Todo::new(title.clone());
    db.insert(&todo)?;

    let todos = db.list_all()?;
    let index = todos.len();

    print_added_todo(index, &title);
    Ok(())
}

pub fn cmd_list(db: &TodoDb) -> Result<()> {
    let todos = db.list_all()?;
    print_todo_table(&todos);
    Ok(())
}

pub fn cmd_done(db: &TodoDb, args: Vec<String>) -> Result<()> {
    let index_str = args.first().ok_or_else(|| anyhow!("Missing todo number. Usage: todo done <#>"))?;
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

pub fn cmd_remove(db: &TodoDb, args: Vec<String>) -> Result<()> {
    let index_str = args.first().ok_or_else(|| anyhow!("Missing todo number. Usage: todo rm <#>"))?;
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
