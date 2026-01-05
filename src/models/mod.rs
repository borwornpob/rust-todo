use polodb_core::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime,
}

impl Todo {
    pub fn new(title: String) -> Self {
        Self {
            id: ObjectId::new(),
            title,
            done: false,
            created_at: DateTime::now(),
        }
    }
}
