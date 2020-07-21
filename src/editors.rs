use diesel::prelude::*;

use crate::models;
use crate::payloads;

pub fn find_editor(
    req_editor_id : &str,
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
