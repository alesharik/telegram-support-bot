create table notes(
    id integer primary key autoincrement not null,
    user_id integer not null references users(id),
    key text not null,
    value text not null
)