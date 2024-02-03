use std::error::Error;
use async_trait::async_trait;
use serde::Deserialize;
use teloxide::prelude::UserId;
use tracing::info;

mod sqlite;
mod entities;
pub use entities::{UserEntity, InsertUserEntity, InsertMessageEntity, MessageType, MessageEntity};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[async_trait]
pub trait Database: Send + Sync {
    async fn get_user_by_tg_id(&self, id: UserId) -> Result<Option<UserEntity>>;

    async fn get_user_by_topic(&self, topic: i64) -> Result<Option<UserEntity>>;

    async fn insert_user(&self, entity: InsertUserEntity) -> Result<UserEntity>;

    async fn update_user(&self, user: UserEntity) -> Result<()>;

    async fn insert_message(&self, message: InsertMessageEntity) -> Result<MessageEntity>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum  DatabaseConfig {
    Sqlite { path: String }
}

pub async fn connect(config: DatabaseConfig) -> anyhow::Result<Box<dyn Database>> {
    match config {
        DatabaseConfig::Sqlite { path } => {
            info!("Connecting to sqlite database at path {path}");
            Ok(Box::new(sqlite::SqliteDatabase::connect(&path)?))
        }
    }
}