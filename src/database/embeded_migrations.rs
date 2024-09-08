use super::lib::establish_connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn migrate() {
    let mut conn = establish_connection();

    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
