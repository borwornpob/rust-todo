use anyhow::{anyhow, Context, Result};
use polodb_core::bson::{doc, oid::ObjectId, DateTime};
use polodb_core::{CollectionT, Database};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    // Concrete model: we store the primary key in Mongo-style "_id"
    #[serde(rename = "_id")]
    id: ObjectId,

    title: String,
    done: bool,
    created_at: DateTime,
}

fn usage() -> &'static str {
    r#"Usage:
  todo add "buy milk"
  todo list
  todo done <id>
  todo rm <id>

Notes:
  - <id> is a 24-char hex ObjectId (printed by 'add' / 'list')
"#
}

fn db_path() -> &'static str {
    // simplest: store DB file in the current directory
    "todo.polo.db"
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1); // skip binary name
    let cmd = args.next().unwrap_or_else(|| "help".to_string());

    if cmd == "help" || cmd == "--help" || cmd == "-h" {
        print!("{}", usage());
        return Ok(());
    }

    // Open / create the embedded database file
    let db = Database::open_path(db_path()).context("failed to open PoloDB file")?;
    let todos = db.collection::<Todo>("todos");

    match cmd.as_str() {
        "add" => {
            // Accept: todo add "..." OR todo add buy milk
            let title_parts: Vec<String> = args.collect();
            if title_parts.is_empty() {
                return Err(anyhow!("missing title\n\n{}", usage()));
            }
            let title = title_parts.join(" ").trim().to_string();
            if title.is_empty() {
                return Err(anyhow!("title is empty\n\n{}", usage()));
            }

            let todo = Todo {
                id: ObjectId::new(),
                title,
                done: false,
                created_at: DateTime::now(),
            };

            // insert_one exists for typed collections (Serialize) :contentReference[oaicite:2]{index=2}
            todos.insert_one(&todo).context("insert failed")?;

            println!("Added:");
            println!("  id:   {}", todo.id.to_hex());
            println!("  title {}", todo.title);
        }

        "list" => {
            // find().sort(...).run() is the documented query flow :contentReference[oaicite:3]{index=3}
            let cursor = todos
                .find(doc! {})
                // show unfinished first, then oldest-first
                .sort(doc! { "done": 1, "created_at": 1 })
                .run()
                .context("query failed")?;

            let mut count = 0usize;
            for item in cursor {
                let t = item.context("cursor item decode failed")?;
                count += 1;

                let checkbox = if t.done { "[x]" } else { "[ ]" };
                println!("{} {} {}", checkbox, t.id.to_hex(), t.title);
            }

            if count == 0 {
                println!("(no todos yet)");
            }
        }

        "done" => {
            let id_str = args.next().ok_or_else(|| anyhow!("missing <id>\n\n{}", usage()))?;
            let oid = ObjectId::parse_str(id_str.trim()).context("invalid <id> (expected 24 hex chars)")?;

            // update_one + $set is part of PoloDB CRUD :contentReference[oaicite:4]{index=4}
            let res = todos
                .update_one(
                    doc! { "_id": oid },
                    doc! { "$set": { "done": true } },
                )
                .context("update failed")?;

            if res.matched_count == 0 {
                println!("No todo found with that id.");
            } else {
                println!("Marked done. (matched={}, modified={})", res.matched_count, res.modified_count);
            }
        }

        "rm" => {
            let id_str = args.next().ok_or_else(|| anyhow!("missing <id>\n\n{}", usage()))?;
            let oid = ObjectId::parse_str(id_str.trim()).context("invalid <id> (expected 24 hex chars)")?;

            // delete_one / delete_many exist :contentReference[oaicite:5]{index=5}
            let res = todos
                .delete_one(doc! { "_id": oid })
                .context("delete failed")?;

            if res.deleted_count == 0 {
                println!("No todo found with that id.");
            } else {
                println!("Removed. (deleted={})", res.deleted_count);
            }
        }

        _ => {
            print!("{}", usage());
        }
    }

    Ok(())
}
