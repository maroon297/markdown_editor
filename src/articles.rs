use diesel::prelude::*;

use crate::models;
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

pub fn get_titles(
    req_editor_id : i64,
    conn: &MysqlConnection
) -> Result<Vec<models::Article>,diesel::result::Error> {
    use crate::schema::articles::dsl::*;
    let articles_vec = articles
        .filter(author_id.eq(req_editor_id))
        .load::<models::Article>(conn)
        .expect("Error loading posts");
    Ok(articles_vec)
}

pub fn find_article(
    article_id : i64,
    conn: &MysqlConnection
) -> Result<Option<models::Article>,diesel::result::Error> {
    use crate::schema::articles::dsl::*;
    let article = articles
        .filter(id.eq(article_id))
        .first::<models::Article>(conn)
        .optional()?;
    Ok(article)
}

pub fn update_article(
    article_id: i64,
    article_title : String,
    article_content : String,
    conn: &MysqlConnection
) -> Result<bool,diesel::result::Error> {
    use crate::schema::articles::dsl::*;

    let target = articles.filter(id.eq(article_id));
    diesel::update(target)
        .set((
            title.eq(article_title),
            content.eq(article_content),))
        .execute(conn)?;
    Ok(true)
}

pub fn delete_article(
    article_id: i64,
    conn: &MysqlConnection
) -> Result<bool,diesel::result::Error> {
    use crate::schema::articles::dsl::*;

    let target = articles.filter(id.eq(article_id));
    diesel::delete(target).execute(conn)?;
    Ok(true)
}
