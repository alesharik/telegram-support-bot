use async_trait::async_trait;
use diesel::{Connection, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use diesel::associations::HasTable;
use teloxide::prelude::UserId;
use tokio::sync::Mutex;
use diesel::ExpressionMethods;
use super::{InsertMessageEntity, InsertUserEntity, MessageEntity, UserEntity};
use crate::schema::users::dsl::users;
use crate::schema::users::{telegram_id, topic};
use crate::schema::messages::dsl::messages;

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
        let mut conn = self.conn.lock().await;
        diesel::update(users::table())
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
}