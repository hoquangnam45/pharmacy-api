pub struct AppConfig {
    db: DBConfig,
    admins: Vec<AdminUser>,
}

struct DBConfig {
    db_user: String,
    db_password: String,
    db_host: String,
    db_type: DBType,
    db_schema: String,
    db_database: String,
}

struct AdminUser {
    username: String,
    password: String,
}

enum DBType {
    POSTGRESQL, SQLITE
}