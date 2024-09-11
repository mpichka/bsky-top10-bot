use chrono::{DateTime, Duration, Utc};
use futures::future::join_all;
use std::vec;
use tokio::task;

const START_TIME: i64 = 24;
const END_TIME: i64 = 25;

use crate::{
    database::{
        models::{NewPost, NewUser, User},
        queries,
    },
    services::bsky::{
        facets::parse_facets_with_users,
        structs::{
            Embed, EmbedType, FeedFilter, FeedListOptions, FollowersListOptions, PostRef,
            ReasonType,
        },
        Bsky,
    },
    utils::bench::Bench,
};

pub async fn sync_users(bsky: &Bsky) {
    let mut options = FollowersListOptions {
        actor: String::from("bsky.one"),
        limit: 100,
        cursor: None,
    };

    let mut new_users: Vec<NewUser> = Vec::new();
    let bench = Bench::start("Collecting users from Bsky");
    loop {
        let res = bsky.get_actor_followers(&options).await.unwrap();

        let mut follows: Vec<NewUser> = res
            .follows
            .iter()
            .map(|v| NewUser {
                did: v.did.clone(),
                handle: v.handle.clone().unwrap(),
                display_name: v.display_name.clone(),
            })
            .collect();

        new_users.append(&mut follows);

        options.cursor = res.cursor;
        if options.cursor.is_none() {
            break;
        }
    }
    bench.end();

    let bench = Bench::start(format!("Syncing {} users with database", new_users.len()).as_str());
    if let Err(error) = queries::sync_users(&new_users).await {
        println!("Error during sync with database: {}", error);
    }
    bench.end();
}

pub async fn sync_users_posts(bsky: &Bsky) {
    let limit: i64 = 100;
    let mut cursor: i32 = 0;

    let mut collected_posts: Vec<NewPost> = Vec::new();
    let total_users = queries::get_total_users_count().unwrap();
    let mut processed_users_count: f32 = 0.0;

    let start_time = Utc::now() - Duration::hours(START_TIME);

    let bench = Bench::start("Collecting posts from Bsky");
    loop {
        let users_bench = Bench::start_silent();

        let users = queries::get_users_list(limit, cursor).unwrap();

        if users.is_empty() {
            break;
        }

        processed_users_count += users.len() as f32;
        if let Some(last_user) = users.last() {
            cursor = last_user.id;
        }

        let futures = users.iter().map(|user| {
            let user = user.clone();
            let bsky = bsky.clone();
            task::spawn(async move { sync_latest_author_posts(&bsky, &user, &start_time).await })
        });

        let results = join_all(futures).await;

        let mut posts: Vec<NewPost> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .flat_map(|posts| posts)
            .collect();

        collected_posts.append(&mut posts);

        let percentage = processed_users_count / total_users as f32 * 100.0;
        let log_message = format!(
            "Synced {} users of {} ({:.2}%)",
            processed_users_count, total_users, percentage
        );

        users_bench.end_with(log_message.as_str());
    }
    bench.end();

    let bench =
        Bench::start(format!("Syncing {} posts with database", collected_posts.len()).as_str());
    if let Err(error) = queries::save_posts(&collected_posts) {
        println!("Error during sync with database: {}", error);
    }
    bench.end();
}

async fn sync_latest_author_posts(
    bsky: &Bsky,
    user: &User,
    start_time: &DateTime<Utc>,
) -> Vec<NewPost> {
    let mut options = FeedListOptions {
        actor: user.did.clone(),
        limit: Some(100),
        cursor: None,
        filter: Some(FeedFilter::PostsWithReplies),
    };

    let like_weight = 1;
    let reply_weight = 5;
    let repost_weight = 3;
    let quote_weight = 4;

    let end_time = Utc::now() - Duration::hours(END_TIME);

    let mut posts: Vec<NewPost> = Vec::new();
    let mut is_out_range = false;
    loop {
        let res = bsky.get_author_feed(&options).await;

        if res.is_err() {
            println!("Error during syncing users: {}", res.err().unwrap());
            break;
        }

        let res = res.unwrap();
        options.cursor = res.cursor;

        for feed in res.feed {
            if feed.post.cid.is_none() {
                continue;
            }

            if let Some(reason) = feed.reason {
                if reason.reason_type == ReasonType::Repost {
                    continue;
                }
            }

            let record_created_at = &feed.post.record.as_ref().unwrap().created_at;
            let created_at = DateTime::parse_from_rfc3339(&record_created_at)
                .unwrap()
                .with_timezone(&Utc);

            if !(*start_time >= created_at && created_at >= end_time) {
                is_out_range = true;
                break;
            }

            let like_count = feed.post.like_count.unwrap();
            let reply_count = feed.post.reply_count.unwrap();
            let repost_count = feed.post.repost_count.unwrap();
            let quote_count = feed.post.quote_count.unwrap();

            let like_points = like_count * like_weight;
            let reply_points = reply_count * reply_weight;
            let repost_points = repost_count * repost_weight;
            let quote_points = quote_count * quote_weight;

            let total_points = like_points + reply_points + repost_points + quote_points;

            let post = NewPost {
                created_at: created_at.naive_utc(),
                uri: feed.post.uri,
                cid: feed.post.cid.unwrap(),
                user_id: user.id,
                reply_count,
                repost_count,
                like_count,
                quote_count,
                total_points,
            };

            posts.push(post);
        }

        if is_out_range || options.cursor.is_none() {
            break;
        }
    }

    posts
}

pub async fn post_top_ten(bsky: &Bsky) {
    let bench = Bench::start("Posting thread");
    let posts_with_users = match queries::get_top_ten_posts_with_users() {
        Ok(res) => Some(res),
        Err(error) => {
            println!("Error during sync with database: {}", error);
            None
        }
    };

    let posts_with_users = if posts_with_users.is_some() {
        posts_with_users.unwrap()
    } else {
        return;
    };

    if posts_with_users.len() == 0 {
        let bench = Bench::start("Dropping all posts");
        if let Err(error) = queries::drop_all_posts() {
            println!("Error during sync with database: {}", error);
        }
        bench.end();
        return;
    }

    for (post, user) in posts_with_users.iter() {
        let display_name = user.display_name.clone().unwrap_or_default();
        let message = format!(
            "#Топ10 {}",
            if display_name.is_empty() {
                user.handle.clone()
            } else {
                display_name
            }
        );

        let facets = parse_facets_with_users(&message, &vec![user.to_owned()]);

        let embed = Embed {
            embed_type: EmbedType::Record,
            record: PostRef {
                uri: post.uri.clone(),
                cid: post.cid.clone(),
            },
        };

        let _ = bsky
            .create_post(message, Some(facets), None, Some(embed))
            .await
            .unwrap();

        // 5 minute delay
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    }
    bench.end();

    let bench = Bench::start("Dropping all posts");
    if let Err(error) = queries::drop_all_posts() {
        println!("Error during sync with database: {}", error);
    }
    bench.end();
}
