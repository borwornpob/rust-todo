use std::path::PathBuf;

use anyhow::{Context, Result};
use polodb_core::bson::{doc, oid::ObjectId, DateTime as BsonDateTime};
use polodb_core::{Collection, CollectionT, Database};

use crate::models::Todo;

const COLLECTION_NAME: &str = "todos";

fn db_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let data_dir = PathBuf::from(home).join(".local/share/todo");
    std::fs::create_dir_all(&data_dir).context("failed to create data directory")?;
    Ok(data_dir.join("todo.db"))
}

pub struct TodoDb {
    db: Database,
}

impl TodoDb {
    pub fn open() -> Result<Self> {
        let path = db_path()?;
        let db = Database::open_path(&path).context("failed to open database")?;
        Ok(Self { db })
    }

    fn collection(&self) -> Collection<Todo> {
        self.db.collection::<Todo>(COLLECTION_NAME)
    }

    pub fn insert(&self, todo: &Todo) -> Result<()> {
        self.collection()
            .insert_one(todo)
            .context("failed to insert todo")?;
        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<Todo>> {
        let cursor = self
            .collection()
            .find(doc! {})
            .run()
            .context("failed to query todos")?;

        let mut todos: Vec<Todo> = cursor
            .map(|item| item.context("failed to decode todo"))
            .collect::<Result<Vec<_>>>()?;

        // Sort: pending first, then by created_at ascending
        todos.sort_by(|a, b| {
            match (a.done, b.done) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.created_at.timestamp_millis().cmp(&b.created_at.timestamp_millis()),
            }
        });

        Ok(todos)
    }

    #[allow(dead_code)]
    pub fn find_by_id(&self, id: &ObjectId) -> Result<Option<Todo>> {
        let cursor = self
            .collection()
            .find(doc! { "_id": id })
            .run()
            .context("failed to query todo")?;

        for item in cursor {
            return Ok(Some(item.context("failed to decode todo")?));
        }
        Ok(None)
    }

    pub fn mark_done(&self, id: &ObjectId) -> Result<bool> {
        let res = self
            .collection()
            .update_one(doc! { "_id": id }, doc! { "$set": { "done": true } })
            .context("failed to update todo")?;
        Ok(res.matched_count > 0)
    }

    pub fn mark_undone(&self, id: &ObjectId) -> Result<bool> {
        let res = self
            .collection()
            .update_one(doc! { "_id": id }, doc! { "$set": { "done": false } })
            .context("failed to update todo")?;
        Ok(res.matched_count > 0)
    }

    pub fn update_title(&self, id: &ObjectId, new_title: &str) -> Result<bool> {
        let res = self
            .collection()
            .update_one(doc! { "_id": id }, doc! { "$set": { "title": new_title } })
            .context("failed to update todo")?;
        Ok(res.matched_count > 0)
    }

    pub fn delete(&self, id: &ObjectId) -> Result<bool> {
        let res = self
            .collection()
            .delete_one(doc! { "_id": id })
            .context("failed to delete todo")?;
        Ok(res.deleted_count > 0)
    }

    pub fn set_reminder(&self, id: &ObjectId, remind_at: Option<BsonDateTime>) -> Result<bool> {
        let res = self
            .collection()
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "remind_at": remind_at, "notified": false } },
            )
            .context("failed to set reminder")?;
        Ok(res.matched_count > 0)
    }

    pub fn get_due_reminders(&self) -> Result<Vec<Todo>> {
        let now = BsonDateTime::now();
        let todos = self.list_all()?;

        Ok(todos
            .into_iter()
            .filter(|t| {
                !t.done
                    && !t.notified
                    && t.remind_at
                        .map(|r| r.timestamp_millis() <= now.timestamp_millis())
                        .unwrap_or(false)
            })
            .collect())
    }

    pub fn mark_notified(&self, id: &ObjectId) -> Result<bool> {
        let res = self
            .collection()
            .update_one(doc! { "_id": id }, doc! { "$set": { "notified": true } })
            .context("failed to mark notified")?;
        Ok(res.matched_count > 0)
    }

    pub fn clear_reminder(&self, id: &ObjectId) -> Result<bool> {
        let res = self
            .collection()
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "remind_at": null, "notified": false } },
            )
            .context("failed to clear reminder")?;
        Ok(res.matched_count > 0)
    }
}
