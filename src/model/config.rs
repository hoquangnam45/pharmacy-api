use derive_getters::Getters;
use diesel::{PgConnection, SqliteConnection};
use serde::Deserialize;

#[derive(Deserialize, Getters)]
pub struct AppConfig {
    db: DBType,
    migration_path: Option<String>,
}

#[derive(Deserialize, Getters)]
pub struct DBConfig {
    user: String,
    password: String,
    address: String,
    port: u32,
    schema: String,
    database: String,
    ssl: Option<bool>,
}

#[derive(Deserialize, Getters)]
pub struct SqliteConfig {
    file_name: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum DBType {
    POSTGRESQL(DBConfig),
    SQLITE(SqliteConfig),
}

impl DBType {
    pub fn get_db_url(&self) -> String {
        match self {
            DBType::POSTGRESQL(db_config) => {
                let ssl = db_config.ssl().unwrap_or(false);
                format!(
                    "postgres://{}:{}@{}:{}/{}?sslmode={}&options=-c search_path={}",
                    db_config.user(),
                    db_config.password(),
                    db_config.address(),
                    db_config.port(),
                    db_config.database(),
                    if ssl { "require" } else { "disable" },
                    db_config.schema(),
                )
            }
            DBType::SQLITE(db_config) => {
                let file_name = db_config.file_name();
                format!("sqlite://{file_name}")
            }
        }
    }
}

