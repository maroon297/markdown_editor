-- Your SQL goes here
CREATE TABLE articles (
 id bigint AUTO_INCREMENT primary key,
 author_id bigint not null,
 title varchar(256) not null,
 content text,
 FOREIGN KEY(author_id)
 REFERENCES editors(editor_id)
)
