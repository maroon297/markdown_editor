use serde::{Serialize};
use crate::schema::{editors,articles};

#[derive(Debug, Clone, Serialize, Queryable, Insertable)]
pub struct Editor {
    pub editor_id: i64,
    pub editor_name: String,
    pub editor_call_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Queryable, Insertable)]
pub struct Article {
    pub id: i64,
    pub author_id: i64,
    pub title: String,
    pub content: Option<String>,
}
