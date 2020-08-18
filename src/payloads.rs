use serde::{Deserialize};
#[derive(Deserialize)]
pub struct CreateEditorReq {
    pub editor_name: String,
    pub editor_call_name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub editor_name: String,    
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdatePasswordReq {
    pub password: String,    
    pub password_again: String,
}

#[derive(Deserialize)]
pub struct CreateArticleReq {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct CreateArticleDbReq {
    pub author_id: i64,
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateArticleReq {
    pub id: i64,
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct DeleteArticleReq {
    pub id: i64,
}
