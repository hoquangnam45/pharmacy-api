use crate::handler::root::hello_world;
use crate::model::config::DBType;
use crate::DBConnection::{POSTGRES, SQLITE};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use config::{Case, Config, Environment, File, FileFormat};
use derive_getters::Getters;
use derive_new::new;
use diesel::migration::{Migration, MigrationVersion};
use diesel::r2d2::{ConnectionManager, ManageConnection, Pool};
use diesel::{Connection, PgConnection, SqliteConnection};
use diesel_migrations::{
    FileBasedMigrations, MigrationHarness,
};
use model::config::AppConfig;
use serde::Deserialize;
use std::error::Error;

#[derive(Parser, Deserialize)]
pub struct Args {
    #[arg(short, long)]
    cfg_path: String,
}

#[derive(new, Getters, Clone)]
pub struct App {
    con: DBConnection,
}

#[derive(Clone)]
pub enum DBConnection {
    POSTGRES(Pool<ConnectionManager<PgConnection>>),
    SQLITE(Pool<ConnectionManager<SqliteConnection>>),
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

    let app = App::new(db_connection);

    if let Some(migration_path) = app_config.migration_path() {
        let migration =
            FileBasedMigrations::from_path(migration_path).expect("cannot load migration path");
        run_migration(&app, &migration).expect("cannot run migration on db");
    }

    tracing_subscriber::fmt::init();
    let router = Router::new().with_state(app).route("/", get(hello_world));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

pub fn establish_db_connection(app: &AppConfig) -> Result<DBConnection, String> {
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

fn establish_db_pool<T: ManageConnection>(
    manager: ConnectionManager<T>,
) -> Result<Pool<ConnectionManager<T>>, String> {
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .map_err(|e| e.to_string())?;
    Ok(pool)
}

pub fn run_migration<DB>(
    app: &App,
    migration: &dyn Migration<DB>,
) -> Result<MigrationVersion<'static>, Box<dyn Error + Send + Sync>> {
    match app.con() {
        POSTGRES(ref mut con) => con.run_migration(),
        SQLITE(ref mut con) => con.run_migration(&migration),
    }
}
