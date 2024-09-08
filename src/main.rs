use dotenv::dotenv;
use services::bsky::Bsky;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

mod database;
mod services;
mod tasks;
mod utils;

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    println!("App is running");
    
    dotenv().ok();

    database::embeded_migrations::migrate();

    let bsky = Bsky::new();
    tasks::sync_users(&bsky).await;
    drop(bsky);

    let mut sched = JobScheduler::new().await?;

    sched
        .add(Job::new_async("0 0 12 */1 * *", |uuid, mut l| {
            Box::pin(async move {
                let mut bsky = Bsky::new();

                if !bsky.is_authenticated() {
                    let auth_res = bsky.authenticate().await;
                    if auth_res.is_err() {
                        println!(
                            "Error during authentication to bsky: {}",
                            auth_res.unwrap_err()
                        );
                    }
                }

                tasks::sync_users(&bsky).await;
                tasks::post_top_ten(&bsky).await;
                l.next_tick_for_job(uuid).await.unwrap();
            })
        })?)
        .await?;

    sched
        .add(Job::new_async("0 0 */1 * * *", move |uuid, mut l| {
            Box::pin(async move {
                let bsky = Bsky::new();
                tasks::sync_users_posts(&bsky).await;
                l.next_tick_for_job(uuid).await.unwrap();
            })
        })?)
        .await?;

    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            println!("App was closed");
        })
    }));

    sched.start().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(31536000)).await;
    }
}
