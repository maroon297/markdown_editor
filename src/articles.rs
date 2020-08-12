use diesel::prelude::*;

use crate::payloads;

pub fn add_article(
    req_article : payloads::CreateArticleDbReq,
    conn: &MysqlConnection
) -> Result<bool,diesel::result::Error> {
    use crate::schema::articles::dsl::*;

    diesel::insert_into(articles).values(
        (author_id.eq(&req_article.author_id),
        title.eq(&req_article.title),
        content.eq(&req_article.content))
    ).execute(conn)?;    
    Ok(true)
}
