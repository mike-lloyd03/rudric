create table app (
    master_password_hash text not null
);

create table secret (
    id int primary key,
    name text not null,
    value text not null
);
