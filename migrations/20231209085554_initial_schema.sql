create table app (
    master_password_hash text not null
);

create table secrets (
    id int primary key,
    name text not null unique,
    value text not null
);
