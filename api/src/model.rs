use crate::schema::tokens;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, QueryableByName, Selectable};
use rocket::serde::{Deserialize, Serialize};

#[cfg(feature = "sqlite")]
type DbInt = i32;

#[cfg(feature = "postgres")]
type DbInt = i32;

#[derive(Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = tokens)]
pub struct TokenInsert {
    pub code: String,
    pub description: String,
    pub namespace: String,
    pub creation_timestamp: NaiveDateTime,
    pub permission_read: bool,
    pub permission_write: bool,
    pub permission_share_share: bool,
    pub permission_share_read: bool,
    pub permission_share_write: bool,
    pub parent: Option<DbInt>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Clone, QueryableByName)]
#[diesel(table_name = tokens)]
#[cfg_attr(feature = "sqlite", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
#[cfg_attr(feature = "postgres", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct Token {
    pub id: DbInt,
    pub code: String,
    pub description: String,
    pub namespace: String,
    pub creation_timestamp: NaiveDateTime,
    pub permission_read: bool,
    pub permission_write: bool,
    pub permission_share_share: bool,
    pub permission_share_read: bool,
    pub permission_share_write: bool,
    pub parent: Option<DbInt>,
}
