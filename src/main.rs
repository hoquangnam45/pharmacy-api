use crate::model::config::{DBConnection, DBType};
use clap::Parser;
use config::{Case, Config, Environment, File, FileFormat};
use derive_new::new;
use diesel::{Connection, ConnectionError, PgConnection, SqliteConnection};
use model::config::AppConfig;
use serde::Deserialize;
use std::error::Error;
use axum::Router;
use axum::routing::{get, post};

#[derive(Parser, Deserialize)]
pub struct Args {
    #[arg(short, long)]
    cfg_path: String,
}

#[derive(new)]
pub struct App {
    con: DBConnection,
}

pub mod model;
pub mod handler;
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

    tracing_subscriber::fmt::init();
    let app = Router::new();
        // `GET /` goes to `root`
        // .route("/", get(root))
        // `POST /users` goes to `create_user`
        // .route("/users", post(create_user));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub fn establish_db_connection(app: &AppConfig) -> Result<DBConnection, ConnectionError> {
    match app.db().kind() {
        DBType::POSTGRESQL => {
            PgConnection::establish(app.db().get_db_url().as_str()).map(|v| DBConnection::POSTGRES(v))
        }
        DBType::SQLITE => {
            SqliteConnection::establish(app.db().get_db_url().as_str()).map(|v| DBConnection::SQLITE(v))
        }
    }
}
