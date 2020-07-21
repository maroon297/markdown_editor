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
