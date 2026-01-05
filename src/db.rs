use anyhow::{Context, Result};
use polodb_core::bson::{doc, oid::ObjectId};
use polodb_core::{Collection, CollectionT, Database};

use crate::models::Todo;

const DB_PATH: &str = "todo.polo.db";
const COLLECTION_NAME: &str = "todos";

pub struct TodoDb {
    db: Database,
}

impl TodoDb {
    pub fn open() -> Result<Self> {
        let db = Database::open_path(DB_PATH).context("failed to open database")?;
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
}
