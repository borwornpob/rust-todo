mod commands;
mod db;
mod display;
mod models;
mod remind;

use std::env;

use anyhow::Result;
use colored::Colorize;

use commands::{
    cmd_add, cmd_clear, cmd_done, cmd_edit, cmd_list, cmd_notify, cmd_remind, cmd_remove,
    cmd_undone,
};
use db::TodoDb;
use display::{print_error, print_usage};

fn run() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().cloned().unwrap_or_else(|| "help".to_string());

    if matches!(cmd.as_str(), "help" | "--help" | "-h") {
        print_usage();
        return Ok(());
    }

    let cmd_args: Vec<String> = if args.len() > 1 {
        args.drain(1..).collect()
    } else {
        vec![]
    };

    let db = TodoDb::open()?;

    match cmd.as_str() {
        "add" | "a" => cmd_add(&db, cmd_args),
        "list" | "ls" | "l" => cmd_list(&db),
        "done" | "d" => cmd_done(&db, cmd_args),
        "undone" | "u" => cmd_undone(&db, cmd_args),
        "edit" | "e" => cmd_edit(&db, cmd_args),
        "remind" => cmd_remind(&db, cmd_args),
        "rm" | "remove" | "r" => cmd_remove(&db, cmd_args),
        "clear" => cmd_clear(&db),
        "notify" => cmd_notify(&db),
        unknown => {
            print_error(&format!("Unknown command: {}", unknown));
            println!("Run {} for usage information", "todo help".cyan());
            Ok(())
        }
    }
}

fn main() {
    if let Err(e) = run() {
        print_error(&format!("{:#}", e));
        std::process::exit(1);
    }
}
