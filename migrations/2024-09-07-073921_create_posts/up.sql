CREATE TABLE IF NOT EXISTS "posts" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    "created_at" DATETIME NOT NULL,
    "user_id" INTEGER NOT NULL,
    "uri" TEXT NOT NULL,
    "cid" TEXT NOT NULL,
    "reply_count" INTEGER NOT NULL,
    "repost_count" INTEGER NOT NULL,
    "like_count" INTEGER NOT NULL,
    "quote_count" INTEGER NOT NULL,
    "total_points" INTEGER NOT NULL,
    CONSTRAINT "posts_to_users" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
