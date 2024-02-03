create table users(
    id integer primary key autoincrement not null,
    telegram_id bigint not null,
    topic bigint not null
);

create index users_topic_idx on users(topic);
create index users_telegram_id_idx on users(telegram_id);