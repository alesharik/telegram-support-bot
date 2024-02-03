use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use crate::schema::users;

#[derive(Queryable, Selectable, AsChangeset, Identifiable, Clone)]
#[diesel(table_name = users)]
pub struct UserEntity {
    pub id: i32,
    pub telegram_id: i64,
    pub topic: i64,
    pub info_message: Option<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct InsertUserEntity {
    pub telegram_id: i64,
    pub topic: i64,
    pub info_message: Option<i64>,
}