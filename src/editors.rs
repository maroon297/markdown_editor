use diesel::prelude::*;

use crate::models;
use crate::payloads;

pub fn find_editor(
    req_editor_id : String,
    conn: &MysqlConnection
) -> Result<Option<models::Editor>,diesel::result::Error> {
    use crate::schema::editors::dsl::*;

    let editor = editors
        .filter(editor_name.eq(req_editor_id))
        .first::<models::Editor>(conn)
        .optional()?;
    Ok(editor)
}

pub fn add_editor(
    req_editor_id : payloads::CreateEditorReq,
    conn: &MysqlConnection
) -> Result<Option<models::Editor>,diesel::result::Error> {
    use crate::schema::editors::dsl::*;

    diesel::insert_into(editors).values(
        (editor_name.eq(&req_editor_id.editor_name),
        editor_call_name.eq(&req_editor_id.editor_call_name),
        password.eq(&req_editor_id.password))
    ).execute(conn)?;
    
    let editor = editors
        .filter(editor_name.eq(&req_editor_id.editor_name))
        .first::<models::Editor>(conn)
        .optional()?;
    Ok(editor)
}

pub fn update_password(
    id : String,
    new_password : String,
    conn: &MysqlConnection
) -> Result<bool,diesel::result::Error> {
    use crate::schema::editors::dsl::*;

    let target = editors.filter(editor_name.eq(id));
    diesel::update(target).set(password.eq(new_password)).execute(conn)?;
    Ok(true)
}
