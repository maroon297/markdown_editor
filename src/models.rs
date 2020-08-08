use serde::{Serialize};
use crate::schema::editors;

#[derive(Debug, Clone, Serialize, Queryable, Insertable)]
pub struct Editor {
    pub editor_id: i64,
    pub editor_name: String,
    pub editor_call_name: String,
    pub password: String,
}
