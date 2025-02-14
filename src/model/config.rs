use std::fmt::format;
use clap_derive::Parser;
use derive_getters::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
pub struct AppConfig {
    db: DBConfig,
}

#[derive(Deserialize, Getters)]
pub struct DBConfig {
    user: String,
    password: String,
    address: String,
    port: u32,
    kind: DBType,
    schema: String,
    database: String,
    ssl: Option<bool>,
}

#[derive(Deserialize, Getters)]
pub enum DBType {
    POSTGRESQL, SQLITE
}

impl DBConfig {
    pub fn get_db_url(self) -> String {
        let ssl = self.ssl().unwrap_or(false);
        match self.kind() {
            DBType::POSTGRESQL => format!("postgres://{}:{}@{}:{}/{}?ssl={}", self.user(), self.password(), self.address(), self.port(), self.database(), self.ssl().unwrap_or(false), );
            DBType::SQLITE => {}
        }
    }
}