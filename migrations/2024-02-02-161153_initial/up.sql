create table users(
    telegram_id bigint primary key not null,
    topic bigint not null
);

create index users_topic_idx on users(topic);