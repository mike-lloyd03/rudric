create table user (
    id integer primary key,
    master_password_hash text not null,
    salt blob not null
);

create table secrets (
    id integer primary key,
    name text not null unique,
    value blob not null,
    description text
);

create table session_keys (
    id blob primary key not null,
    key blob not null,
    expire_time datetime not null
)
