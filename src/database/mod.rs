use std::error::Error;
use async_trait::async_trait;
use serde::Deserialize;
use teloxide::prelude::UserId;
use tracing::info;

mod sqlite;
mod entities;
pub use entities::{UserEntity, InsertUserEntity};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[async_trait]
pub trait Database: Send + Sync {
    async fn get_user_by_tg_id(&self, id: UserId) -> Result<Option<UserEntity>>;

    async fn get_user_by_topic(&self, topic: i64) -> Result<Option<UserEntity>>;

    async fn insert(&self, entity: InsertUserEntity) -> Result<UserEntity>;
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