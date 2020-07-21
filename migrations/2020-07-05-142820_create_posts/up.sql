-- Your SQL goes here
CREATE TABLE editors (
  editor_id bigint AUTO_INCREMENT primary key,
  editor_name varchar(128) unique not null,
  editor_call_name varchar(64) not null,
  password varchar(256) not null
)
