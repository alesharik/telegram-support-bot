use async_trait::async_trait;
use diesel::{Connection, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use diesel::associations::HasTable;
use teloxide::prelude::UserId;
use tokio::sync::Mutex;
use diesel::ExpressionMethods;
use crate::database::entities::{InsertNoteEntity, NoteEntity};
use super::{InsertMessageEntity, InsertUserEntity, MessageEntity, MessageType, UserEntity};
use crate::schema::users::dsl::users;
use crate::schema::users::{telegram_id, topic};
use crate::schema::messages::dsl::messages;
use crate::schema::notes::dsl::notes;

pub struct SqliteDatabase {
    conn: Mutex<SqliteConnection>,
}

impl SqliteDatabase {
    pub fn connect(db: &str) -> anyhow::Result<SqliteDatabase> {
        Ok(SqliteDatabase {
            conn: Mutex::new(SqliteConnection::establish(db)?)
        })
    }
}

#[async_trait]
impl super::Database for SqliteDatabase {
    async fn get_user_by_tg_id(&self, id: UserId) -> super::Result<Option<UserEntity>> {
        let mut conn = self.conn.lock().await;
        Ok(users
            .select(UserEntity::as_select())
            .filter(telegram_id.eq(id.0 as i64))
            .first(&mut *conn).
            optional()?)
    }

    async fn get_user_by_topic(&self, t: i64) -> super::Result<Option<UserEntity>> {
        let mut conn = self.conn.lock().await;
        Ok(users
            .select(UserEntity::as_select())
            .filter(topic.eq(t))
            .first(&mut *conn)
            .optional()?)
    }

    async fn insert_user(&self, entity: InsertUserEntity) -> super::Result<UserEntity> {
        let mut conn = self.conn.lock().await;
        Ok(diesel::insert_into(users::table())
            .values(&entity)
            .get_result(&mut *conn)?)
    }

    async fn update_user(&self, user: UserEntity) -> crate::database::Result<()> {
        use crate::schema::users::{id};

        let mut conn = self.conn.lock().await;
        diesel::update(users::table())
            .filter(id.eq(user.id))
            .set(user)
            .execute(&mut *conn)?;
        Ok(())
    }

    async fn insert_message(&self, message: InsertMessageEntity) -> crate::database::Result<MessageEntity> {
        let mut conn = self.conn.lock().await;
        Ok(diesel::insert_into(messages::table())
            .values(&message)
            .get_result(&mut *conn)?)
    }

    async fn get_message(&self, user: &UserEntity, typ: MessageType, rx_id: i64) -> crate::database::Result<Option<MessageEntity>> {
        use crate::schema::messages::{user_id, type_, rx_msg_id};

        let mut conn = self.conn.lock().await;
        Ok(messages.select(MessageEntity::as_select())
            .filter(user_id.eq(user.id))
            .filter(type_.eq(typ as i16))
            .filter(rx_msg_id.eq(rx_id))
            .first(&mut *conn)
            .optional()?)
    }

    async fn save_note(&self, note: InsertNoteEntity) -> crate::database::Result<NoteEntity> {
        use crate::schema::notes::{user_id, id, key};

        let mut conn = self.conn.lock().await;
        let existing: Option<NoteEntity> = notes.select(NoteEntity::as_select())
            .filter(user_id.eq(note.user_id))
            .filter(key.eq(&note.key))
            .first(&mut *conn)
            .optional()?;
        Ok(match existing {
            None => {
                diesel::insert_into(notes::table())
                    .values(&note)
                    .get_result(&mut *conn)?
            }
            Some(mut entity) => {
                entity.value = note.value;
                diesel::update(notes::table())
                    .filter(id.eq(entity.id))
                    .set(entity.clone())
                    .execute(&mut *conn)?;
                entity
            }
        })
    }

    async fn get_notes(&self, user: &UserEntity) -> crate::database::Result<Vec<NoteEntity>> {
        use crate::schema::notes::user_id;

        let mut conn = self.conn.lock().await;
        Ok(notes.select(NoteEntity::as_select())
            .filter(user_id.eq(user.id))
            .get_results(&mut *conn)?)
    }

    async fn delete_note(&self, user: &UserEntity, note_key: &str) -> crate::database::Result<()> {
        use crate::schema::notes::{user_id, key};

        let mut conn = self.conn.lock().await;
        diesel::delete(notes::table())
            .filter(user_id.eq(user.id))
            .filter(key.eq(note_key))
            .execute(&mut *conn)?;
        Ok(())
    }
}