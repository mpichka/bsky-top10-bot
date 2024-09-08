// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Integer,
        created_at -> Timestamp,
        user_id -> Integer,
        uri -> Text,
        cid -> Text,
        reply_count -> Integer,
        repost_count -> Integer,
        like_count -> Integer,
        quote_count -> Integer,
        total_points -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        did -> Text,
        handle -> Text,
        display_name -> Nullable<Text>,
    }
}

diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
