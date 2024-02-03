use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use crate::schema::users;

#[derive(Queryable, Selectable, AsChangeset, Insertable, Clone)]
#[diesel(table_name = users)]
pub struct UserEntity {
    pub telegram_id: i64,
    pub topic: i64,
}