use crate::handler::root::hello_world;
use crate::model::config::DBType;
use crate::DBPool::{POSTGRES, SQLITE};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use config::{Case, Config, Environment, File, FileFormat};
use derive_getters::Getters;
use derive_new::new;
use diesel::migration::{Migration, MigrationSource, MigrationVersion};
use diesel::r2d2::{ConnectionManager, ManageConnection, Pool, R2D2Connection};
use diesel::{Connection, PgConnection, SqliteConnection};
use diesel_migrations::{
    FileBasedMigrations, MigrationHarness,
};
use model::config::AppConfig;
use serde::Deserialize;
use std::error::Error;
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;

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
        return &mut self.pool;
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
        run_migration(&mut app, &migration);
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
    file_migrations: &FileBasedMigrations,
) -> Vec<MigrationVersion<'static>> {
    match app.pool_mut() {
        POSTGRES(ref mut conPool) => {
            let mut con = conPool.try_get().expect("failed to get connection to perform migration");
            file_migrations.migrations().expect("failed to load migration file").iter().map(|m| con.run_migration(m).expect("failed to execute migration file")).collect()
        },
        SQLITE(ref mut conPool) => {
            let mut con = conPool.try_get().expect("failed to get connection to perform migration");
            file_migrations.migrations().expect("failed to load migration file").iter().map(|m| con.run_migration(m).expect("failed to execute migration file")).collect()
        }
    }
}
