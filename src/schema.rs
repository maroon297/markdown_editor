table! {
    articles (id) {
        id -> Bigint,
        author_id -> Bigint,
        title -> Varchar,
        content -> Nullable<Text>,
    }
}

table! {
    editors (editor_id) {
        editor_id -> Bigint,
        editor_name -> Varchar,
        editor_call_name -> Varchar,
        password -> Varchar,
    }
}

joinable!(articles -> editors (author_id));

allow_tables_to_appear_in_same_query!(
    articles,
    editors,
);
