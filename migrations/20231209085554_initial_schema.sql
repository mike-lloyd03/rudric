create table user (
    id int primary key not null,
    master_password_hash text not null,
    salt blob not null
);

create table secrets (
    id int primary key,
    name text not null unique,
    value blob not null,
    description text
);
