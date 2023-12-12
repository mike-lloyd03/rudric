create table user (
    master_password_hash blob not null,
    salt blob not null
);

create table secrets (
    id int primary key,
    name text not null unique,
    value blob not null,
    description text
);
