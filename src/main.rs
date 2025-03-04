use crate::handler::auth::{
    admin_login, admin_logout, admin_refresh, login, logout, refresh, register,
};
use crate::handler::root::hello_world;
use crate::repo::user::UserRepo;
use crate::service::auth::AuthService;
use app_config::AppConfig;
use app_config::DBType;
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use config::{Case, Config, Environment, File, FileFormat};
use derive_getters::Getters;
use derive_new::new;
use diesel::migration::MigrationConnection;
use diesel::r2d2::{Builder, ConnectionManager, Pool, PooledConnection, R2D2Connection};
use diesel::{Connection, PgConnection, SqliteConnection};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use repo::DBPool;
use repo::DBPool::{POSTGRES, SQLITE};
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
    repos: Repos,
    services: Services,
    secrets: Secrets,
    config: AppConfig,
}

#[derive(new, Getters, Clone)]
pub struct Services {
    auth: AuthService,
}

#[derive(new, Getters, Clone)]
pub struct Repos {
    user: UserRepo,
}

#[derive(new, Getters, Clone)]
pub struct Secrets {
    jwt_key: String
}

pub mod app_config;
pub mod handler;
pub mod repo;
pub mod schema;
pub mod service;
pub mod util;

#[tokio::main]
async fn main() -> () {
    let args = Args::try_parse()
        .map_err(|e| {
            let msg = e.to_string();
            format!("{msg}")
        })
        .unwrap();
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

    let repos = init_repos();
    let services = init_services(repos.clone());
    let secrets = init_secrets();
    let mut app = App::new(pool, repos, services, secrets);

    // NOTE: Doing it like this instead of additional commands to set up the DB to reduce setup complexity
    if let Some(migration_path) = app_config.migration_path() {
        let migration =
            FileBasedMigrations::from_path(migration_path).expect("cannot load migration path");
        run_migration(&mut app, migration).expect("failed to migrate database");
    }

    tracing_subscriber::fmt::init();

    let v1_public_api = Router::new().nest(
        "/auth",
        Router::new()
            .route("/login", post(login))
            .route("/register", post(register))
            .route("/logout", post(logout))
            .route("/refresh", post(refresh)),
    );
    let v1_admin_api = Router::new().nest(
        "/admin",
        Router::new().nest(
            "/auth",
            Router::new()
                .route("/login", post(admin_login))
                .route("/logout", post(admin_logout))
                .route("/refresh", post(admin_refresh)),
        ),
    );
    let v1_api = v1_public_api.merge(v1_admin_api);

    let router = Router::new()
        .route("/", get(hello_world))
        .nest("/api", Router::new().nest("/v1", v1_api))
        .with_state(app);

    let bind_address = format!(
        "{}:{}",
        app_config
            .address()
            .as_ref()
            .map(String::as_str)
            .unwrap_or("0.0.0.0"),
        app_config.port().unwrap_or(8888)
    );
    let listener = tokio::net::TcpListener::bind(bind_address.as_str())
        .await
        .expect(format!("failed to bind to {bind_address}").as_str());
    axum::serve(listener, router)
        .await
        .expect("failed to serve API");
}

fn init_secrets() -> Secrets {
    Secrets::new(uuid::Uuid::new_v4().to_string())
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
    run_pool!(app.pool(), mut c, { apply_migration(file_migrations, &mut c) });
}

fn apply_migration<C: R2D2Connection + 'static>(
    file_migrations: FileBasedMigrations,
    c: &mut PooledConnection<ConnectionManager<C>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    c.setup()?;
    c.run_pending_migrations(file_migrations)?;
}

fn init_repos() -> Repos {
    Repos::new(UserRepo::new())
}

fn init_services(repos: Repos) -> Services {
    Services::new(AuthService::new(repos.user().clone()))
}
