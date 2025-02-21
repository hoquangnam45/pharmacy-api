use crate::handler::root::hello_world;
use crate::model::config::DBType;
use crate::DBPool::{POSTGRES, SQLITE};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use config::{Case, Config, Environment, File, FileFormat};
use derive_getters::Getters;
use derive_new::new;
use diesel::backend::Backend;
use diesel::migration::{Migration, MigrationConnection, MigrationSource, MigrationVersion};
use diesel::r2d2::{ConnectionManager, ManageConnection, Pool, PooledConnection, R2D2Connection};
use diesel::{Connection, PgConnection, QueryDsl, SqliteConnection};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use model::config::AppConfig;
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;

#[derive(Parser, Deserialize)]
pub struct Args {
    #[arg(short, long)]
    cfg_path: String,
}

#[derive(new, Getters, Clone)]
pub struct App {
    pool: DBPool,
}

#[derive(Clone)]
pub enum DBPool {
    POSTGRES(Pool<ConnectionManager<PgConnection>>),
    SQLITE(Pool<ConnectionManager<SqliteConnection>>),
}

impl App {
    fn pool_mut(&mut self) -> &mut DBPool {
        &mut self.pool
    }
}

pub mod handler;
pub mod model;
pub mod repo;

#[tokio::main]
async fn main() -> () {
    let args = match Args::try_parse() {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            print!("{msg}");
            return;
        }
    };
    let cfg_path = args.cfg_path;
    let raw_cfg = Config::builder()
        .add_source(
            Environment::default()
                .convert_case(Case::Lower)
                .separator("_"),
        )
        .add_source(File::new(cfg_path.as_str(), FileFormat::Yaml))
        .build()
        .expect("cannot parse config");
    let app_config: AppConfig = raw_cfg.try_deserialize().unwrap();
    let db_connection =
        establish_db_connection(&app_config).expect("cannot establish connection to db");

    let mut app = App::new(db_connection);

    if let Some(migration_path) = app_config.migration_path() {
        let migration =
            FileBasedMigrations::from_path(migration_path).expect("cannot load migration path");
        run_migration(&mut app, migration).expect("failed to migrate database");
    }

    tracing_subscriber::fmt::init();
    let router = Router::new().route("/", get(hello_world)).with_state(app);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

pub fn establish_db_connection(app: &AppConfig) -> Result<DBPool, String> {
    match app.db() {
        DBType::POSTGRESQL(_) => {
            let manager = ConnectionManager::<PgConnection>::new(app.db().get_db_url());
            let pool = establish_db_pool(manager)?;
            Ok(POSTGRES(pool))
        }
        DBType::SQLITE(_) => {
            let manager = ConnectionManager::<SqliteConnection>::new(app.db().get_db_url());
            let pool = establish_db_pool(manager)?;
            Ok(SQLITE(pool))
        }
    }
}

fn establish_db_pool<T: R2D2Connection + 'static>(
    manager: ConnectionManager<T>,
) -> Result<Pool<ConnectionManager<T>>, String> {
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .map_err(|e| e.to_string())?;
    Ok(pool)
}

pub fn run_migration(
    app: &mut App,
    file_migrations: FileBasedMigrations,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match app.pool_mut() {
        POSTGRES(ref mut con_pool) => {
            let mut con = con_pool
                .try_get()
                .ok_or("failed to get connection to perform migration")?;
            apply_migration(file_migrations, &mut con)
        }
        SQLITE(ref mut con_pool) => {
            let mut con = con_pool
                .try_get()
                .ok_or("failed to get connection to perform migration")?;
            apply_migration(file_migrations, &mut con)
        }
    }
}

fn apply_migration<DB, Con>(
    file_migrations: FileBasedMigrations,
    con: &mut PooledConnection<ConnectionManager<Con>>,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    DB: Backend,
    Con: R2D2Connection + MigrationHarness<DB> + MigrationConnection + 'static,
{
    con.setup()?;
    con.run_pending_migrations(file_migrations)?;
    Ok(())
}
