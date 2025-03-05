use std::time::Duration;
use derive_getters::Getters;
use jwt::RegisteredClaims;
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters)]
pub struct AppConfig {
    db: DBType,
    pool: Option<PoolConfig>,
    migration_path: Option<String>,
    port: Option<u32>,
    address: Option<String>,
    jwt: JwtConfig
}

#[derive(Clone, Deserialize, Getters)]
pub struct DBConfig {
    user: String,
    password: String,
    address: String,
    port: u32,
    schema: String,
    database: String,
    ssl: Option<bool>,
}

#[derive(Clone, Deserialize, Getters)]
pub struct PoolConfig {
    max_size: Option<u32>,
    min_idle: Option<u32>,
    test_on_check_out: Option<bool>,
    max_lifetime_in_sec: Option<u32>,
    idle_timeout_in_sec: Option<u32>,
    connection_timeout_in_sec: Option<u32>,
}

#[derive(Clone, Deserialize, Getters)]
pub struct SqliteConfig {
    file_name: String,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DBType {
    POSTGRESQL(DBConfig),
    SQLITE(SqliteConfig),
}

#[derive(Clone, Deserialize, Getters)]
pub struct JwtConfig {
    access_token: AccessTokenConfig,
    refresh_token: RefreshTokenConfig
}

#[derive(Clone, Deserialize, Getters)]
pub struct RefreshTokenConfig {
    expire_in_min: Option<Duration>
}

#[derive(Clone, Deserialize, Getters)]
pub struct AccessTokenConfig {

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
