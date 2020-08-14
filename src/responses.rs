use serde::{Serialize,Deserialize};
#[derive(Deserialize, Serialize)]
pub struct TitlesRes {
    pub article_id: i64,
    pub title: String,
}
