use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{PgConnection, SqliteConnection};
use diesel::row::NamedRow;

pub mod user;

#[macro_export]
macro_rules! run_pool {
    ($db:expr, mut $conn:expr, $body:block) => {
        match $db {
            DBPool::POSTGRES(pool) => {
                let mut $conn = pool.get().expect("Postgres connection error");
                $body
            }
            DBPool::SQLITE(pool) => {
                let mut $conn = pool.get().expect("SQLite connection error");
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! run_con {
    ($db:expr, mut $conn:ident, $body:block) => {
        match $db {
            DBConnection::POSTGRES(c) => {
                let mut $conn = c;
                $body
            }
            DBConnection::SQLITE(c) => {
                let $conn = c;
                $body
            }
        }
    };
}

#[derive(Clone)]
pub enum DBPool {
    POSTGRES(Pool<ConnectionManager<PgConnection>>),
    SQLITE(Pool<ConnectionManager<SqliteConnection>>),
}

pub enum DBConnection {
    POSTGRES(PooledConnection<ConnectionManager<PgConnection>>),
    SQLITE(PooledConnection<ConnectionManager<SqliteConnection>>),
}

impl DBPool {
    pub fn get_con(&self) -> Result<DBConnection, String> {
        match self {
            DBConnection::POSTGRES(pool) => pool
                .get()
                .map_err(|e| e.to_string())
                .map(DBConnection::POSTGRES),
            DBConnection::SQLITE(pool) => pool
                .get()
                .map_err(|e| e.to_string())
                .map(DBConnection::SQLITE),
        }
    }
}
