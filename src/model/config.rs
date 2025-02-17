use clap_derive::Parser;
use derive_getters::Getters;
use serde::Deserialize;
use std::fmt::format;
use diesel::{PgConnection, SqliteConnection};

#[derive(Deserialize, Getters)]
pub struct AppConfig {
    db: DBConfig,
}

#[derive(Deserialize, Getters)]
pub struct DBConfig {
    user: Option<String>,
    password: Option<String>,
    address: Option<String>,
    port: Option<u32>,
    schema: Option<String>,
    database: Option<String>,
    ssl: Option<bool>,
    file: Option<String>,
    kind: DBType
}

#[derive(Deserialize)]
pub enum DBType {
    POSTGRESQL,
    SQLITE,
}

impl DBConfig {
    pub fn get_db_url(&self) -> String {
        match self.kind() {
            DBType::POSTGRESQL => {
                let ssl = self.ssl().unwrap_or(false);
                format!(
                    "postgres://{}:{}@{}:{}/{}?sslmode={}&options=-c search_path={}",
                    self.user().as_ref().expect("missing db user"),
                    self.password().as_ref().expect("missing db password"),
                    self.address().as_ref().expect("missing db address"),
                    self.port().as_ref().expect("missing db port"),
                    self.database().as_ref().expect("missing db database"),
                    if ssl { "require" } else { "disable" },
                    self.schema().as_ref().expect("missing db schema"),
                )
            }
            DBType::SQLITE => {
                let file_name = self.file.as_ref().expect("missing db file");
                format!("sqlite://{file_name}")
            },
        }
    }
}

pub enum DBConnection {
    POSTGRES(PgConnection),
    SQLITE(SqliteConnection),
}
