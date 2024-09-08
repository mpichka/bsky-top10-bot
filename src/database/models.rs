use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = super::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::users)]
pub struct NewUser {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
}

#[derive(Queryable, Selectable, Associations, Debug)]
#[diesel(table_name = super::schema::posts)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub user_id: i32,
    pub uri: String,
    pub cid: String,
    pub reply_count: i32,
    pub repost_count: i32,
    pub like_count: i32,
    pub quote_count: i32,
    pub total_points: i32,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::posts)]
pub struct NewPost {
    pub created_at: NaiveDateTime,
    pub user_id: i32,
    pub uri: String,
    pub cid: String,
    pub reply_count: i32,
    pub repost_count: i32,
    pub like_count: i32,
    pub quote_count: i32,
    pub total_points: i32,
}
