-- Your SQL goes here
CREATE TABLE instance_configs (
    id             serial        primary key,
    title          varchar(250)  not null,
    description    text,
    created_at     timestamp     not null default now(),
    creator        varchar(30)   references users(id) on delete set null,
    player         varchar(30)   references users(id),    
    yaml           text          not null,
    begin_at       timestamp     not null default now(),
    token          uuid          default gen_random_uuid()
)