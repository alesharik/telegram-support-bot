create table messages(
    id integer primary key autoincrement not null,
    user_id integer references users(id) not null,
    type smallint not null,
    rx_msg_id bigint not null,
    rx_msg text not null,
    tx_msg_id bigint not null
);

create index messages_user_id_rx_msg_id_idx on messages(user_id, rx_msg_id);