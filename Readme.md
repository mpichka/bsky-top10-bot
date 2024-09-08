# Bsky-Top10-Bot

Make top 10 posts ranking

## Migrations

cargo install diesel_cli --no-default-features --features sqlite-bundled

diesel migration generate <name_of_migration>

diesel migration run
diesel migration redo
diesel migration revert

## Roadmap

- [x] Sync users from provider account
- [x] Update users data on sync
- [x] Collect posts and replies of users created in 24h interval
- [x] Evaluate posts:
  - like - 1 point
  - comment - 5 points
  - repost - 3 points
  - quote - 4 points
- [x] Cronjob to update post score on hourly basis
- [x] Cronjob to make a thread with top10 posts
- [ ] Don't include to ranking posts within same thread of the same user (Prevent one user to occupy all 10 positions in thread)
