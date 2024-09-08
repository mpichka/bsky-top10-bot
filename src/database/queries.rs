use super::{models::*, schema};
use crate::database::lib::{create_pool, establish_connection};
use diesel::{prelude::*, result::Error};
use futures::future::join_all;
use std::collections::HashMap;
use tokio::task;

pub async fn sync_users(users_list: &Vec<NewUser>) -> Result<(), Error> {
    use super::schema::users::dsl::*;
    let mut conn = establish_connection();

    let dids: Vec<&str> = users_list.iter().map(|v| v.did.as_str()).collect();

    let existed_users: Vec<User> = users
        .filter(did.eq_any(dids))
        .select(User::as_select())
        .load::<User>(&mut conn)?;

    let existed_users_map: HashMap<&String, &User> =
        existed_users.iter().map(|user| (&user.did, user)).collect();

    let mut to_update: Vec<User> = Vec::new();
    let mut to_insert: Vec<&NewUser> = Vec::new();

    for user in users_list {
        if let Some(existed_user) = existed_users_map.get(&user.did) {
            if existed_user.handle != user.handle || existed_user.display_name != user.display_name
            {
                to_update.push(User {
                    id: existed_user.id,
                    created_at: existed_user.created_at,
                    updated_at: existed_user.updated_at,
                    did: user.did.clone(),
                    handle: user.handle.clone(),
                    display_name: user.display_name.clone(),
                });
            }
        } else {
            to_insert.push(user);
        }
    }

    if !to_update.is_empty() {
        let connection_pool = create_pool(to_update.len() as u32);
        let futures = to_update.into_iter().map(|user| {
            let mut conn = connection_pool.get().unwrap();
            task::spawn_blocking(move || {
                diesel::update(users.find(user.id))
                    .set((handle.eq(user.handle), display_name.eq(user.display_name)))
                    .execute(&mut conn)
            })
        });
        join_all(futures).await;
    }

    if !to_insert.is_empty() {
        diesel::insert_into(users)
            .values(to_insert)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn get_total_users_count() -> Result<i64, Error> {
    use super::schema::users::dsl::*;
    let mut conn = establish_connection();

    let count = users.count().get_result(&mut conn)?;

    Ok(count)
}

pub fn get_users_list(limit: i64, cursor: i32) -> Result<Vec<User>, Error> {
    use super::schema::users::dsl::*;
    let mut conn = establish_connection();

    let rows = users
        .filter(id.gt(cursor))
        .limit(limit)
        .select(User::as_select())
        .get_results(&mut conn)?;

    Ok(rows)
}

pub fn save_posts(new_posts: &Vec<NewPost>) -> Result<(), Error> {
    use super::schema::posts::dsl::*;
    let mut conn = establish_connection();

    if !new_posts.is_empty() {
        diesel::insert_into(posts)
            .values(new_posts)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn get_top_ten_posts_with_users() -> Result<Vec<(Post, User)>, Error> {
    let mut conn = establish_connection();

    let rows: Vec<(Post, User)> = schema::posts::table
        .inner_join(schema::users::table)
        .filter(schema::posts::total_points.gt(0))
        .order(schema::posts::total_points.desc())
        .limit(10)
        .select((Post::as_select(), User::as_select()))
        .load::<(Post, User)>(&mut conn)?;

    Ok(rows)
}

pub fn drop_all_posts() -> Result<(), Error> {
    use super::schema::posts::dsl::*;

    let mut conn = establish_connection();

    diesel::delete(posts).execute(&mut conn)?;

    Ok(())
}
