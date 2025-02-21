use crate::handler::root::hello_world;
use config::DBType;
use crate::DBPool::{POSTGRES, SQLITE};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use config::{Case, Config, Environment, File, FileFormat};
use derive_getters::Getters;
use derive_new::new;
use diesel::backend::Backend;
use diesel::migration::{Migration, MigrationConnection, MigrationSource};
use diesel::r2d2::{
    Builder, ConnectionManager, ManageConnection, Pool, PooledConnection, R2D2Connection,
};
use diesel::{Connection, PgConnection, QueryDsl, SqliteConnection};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use config::AppConfig;
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;

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
pub mod config;

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
    let pool = connect_db(&app_config).expect("cannot establish connection to db");

    let mut app = App::new(pool);

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

pub fn connect_db(app_config: &AppConfig) -> Result<DBPool, String> {
    match app_config.db() {
        DBType::POSTGRESQL(_) => {
            let manager = ConnectionManager::<PgConnection>::new(app_config.db().get_db_url());
            let pool = establish_db_pool(app_config, manager)?;
            Ok(POSTGRES(pool))
        }
        DBType::SQLITE(_) => {
            let manager = ConnectionManager::<SqliteConnection>::new(app_config.db().get_db_url());
            let pool = establish_db_pool(app_config, manager)?;
            Ok(SQLITE(pool))
        }
    }
}

fn establish_db_pool<T: R2D2Connection + 'static>(
    app_config: &AppConfig,
    manager: ConnectionManager<T>,
) -> Result<Pool<ConnectionManager<T>>, String> {
    let mut pool_builder: Builder<ConnectionManager<T>> = Pool::builder();
    if let Some(pool_config) = app_config.pool() {
        if let Some(test_on_checkout) = pool_config.test_on_check_out() {
            pool_builder = pool_builder.test_on_check_out(test_on_checkout.to_owned());
        }
        if let Some(connection_timeout_in_sec) = pool_config.connection_timeout_in_sec() {
            pool_builder = pool_builder.connection_timeout(Duration::from_secs(
                connection_timeout_in_sec.to_owned() as u64,
            ));
        }
        if let Some(max_size) = pool_config.max_size() {
            pool_builder = pool_builder.max_size(max_size.to_owned());
        }
        pool_builder = pool_builder.min_idle(pool_config.min_idle().to_owned());
        pool_builder = pool_builder.max_lifetime(
            pool_config
                .max_lifetime_in_sec()
                .map(|v| Duration::from_secs(v as u64)),
        );
        pool_builder = pool_builder.idle_timeout(
            pool_config
                .idle_timeout_in_sec()
                .map(|v| Duration::from_secs(v as u64)),
        );
    }
    let pool = pool_builder.build(manager).map_err(|e| e.to_string())?;
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
