use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use teloxide::prelude::Message;
use teloxide::types::MessageId;
use crate::schema::{users, messages, notes};

#[derive(Queryable, Selectable, AsChangeset, Identifiable, Clone)]
#[diesel(table_name = users)]
pub struct UserEntity {
    pub id: i32,
    pub telegram_id: i64,
    pub topic: i64,
    pub info_message: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub lang_code: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct InsertUserEntity {
    pub telegram_id: i64,
    pub topic: i64,
    pub info_message: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub lang_code: Option<String>,
}

#[repr(i16)]
pub enum MessageType {
    Incoming,
    Outgoing
}

#[derive(Insertable)]
#[diesel(table_name = messages)]
pub struct InsertMessageEntity {
    pub user_id: i32,
    pub type_: i16,
    pub rx_msg_id: i64,
    pub rx_msg: String,
    pub tx_msg_id: i64,
}

#[derive(Queryable, Selectable, AsChangeset, Identifiable, Clone)]
#[diesel(table_name = messages)]
pub struct MessageEntity {
    pub id: i32,
    pub user_id: i32,
    pub type_: i16,
    pub rx_msg_id: i64,
    pub rx_msg: String,
    pub tx_msg_id: i64,
}

impl InsertMessageEntity {
    pub fn incoming(user: &UserEntity, rx: &Message, tx_id: MessageId) -> InsertMessageEntity {
        InsertMessageEntity {
            user_id: user.id,
            type_: MessageType::Incoming as i16,
            rx_msg_id: rx.id.0 as i64,
            rx_msg: serde_json::to_string(rx).unwrap(),
            tx_msg_id: tx_id.0 as i64,
        }
    }

    pub fn outgoing(user: &UserEntity, rx: &Message, tx_id: MessageId) -> InsertMessageEntity {
        InsertMessageEntity {
            user_id: user.id,
            type_: MessageType::Outgoing as i16,
            rx_msg_id: rx.id.0 as i64,
            rx_msg: serde_json::to_string(rx).unwrap(),
            tx_msg_id: tx_id.0 as i64,
        }
    }
}


#[derive(Insertable)]
#[diesel(table_name = notes)]
pub struct InsertNoteEntity {
    pub user_id: i32,
    pub key: String,
    pub value: String,
}

#[derive(Queryable, Selectable, AsChangeset, Identifiable, Clone)]
#[diesel(table_name = notes)]
pub struct NoteEntity {
    pub id: i32,
    pub user_id: i32,
    pub key: String,
    pub value: String,
}
