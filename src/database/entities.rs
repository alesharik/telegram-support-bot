use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use crate::schema::users;

#[derive(Queryable, Selectable, AsChangeset, Clone)]
#[diesel(table_name = users)]
pub struct UserEntity {
    pub id: i32,
    pub telegram_id: i64,
    pub topic: i64,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct InsertUserEntity {
    pub telegram_id: i64,
    pub topic: i64,
}