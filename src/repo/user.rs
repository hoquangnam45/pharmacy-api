use crate::schema::users::users;
use crate::repo::DBConnection;
use derive_getters::Getters;
use derive_new::new;
use diesel::pg::Pg;
use diesel::sqlite::Sqlite;
use diesel::{AsChangeset, ExpressionMethods, Identifiable, Insertable, Queryable, Selectable, SelectableHelper};
use std::error::Error;
use diesel::query_dsl::select_dsl::SelectDsl;
use crate::run_con;

#[derive(new, Clone, Getters)]
pub struct UserRepo {}

pub struct PhoneNumber {
    country_code: u16,
}

pub struct ContactedEmail {}

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Insertable, Getters)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Pg))]
#[diesel(check_for_backend(Sqlite))]
pub struct User {
    id: String,
    username: String,
    email: Option<String>,
    phone_number: Option<String>,
    password: String, // NOTE: User can log in using other methods so password is not required
    activated: bool,
}

pub enum AdminPermission {}

pub struct AdminUser {
    id: String,
    username: String,
    password: String,
    permissions: Option<Vec<AdminPermission>>,
    notify_email: Option<String>,
    notify_phone: Option<String>,
}

#[derive(new)]
pub enum LoadUserParams {
    USERNAME(String),
    EMAIL(String),
    ID(String),
    PHONE_NUMBER(String),
}

impl UserRepo {
    pub fn load_user(
        &mut self,
        con: &mut DBConnection,
        load_user_params: &LoadUserParams,
    ) -> Result<Option<User>, Box<dyn Error>> {
        run_con!(con, c, {
            match load_user_params {
                LoadUserParams::USERNAME(v) => users::dsl::users.filter(users::username.eq(v)).select(User::as_select()).first(c).optional()?,
                LoadUserParams::EMAIL(v) => users::dsl::users.filter(users::email.eq(v)).select(User::as_select()).first(c).optional()?,
                LoadUserParams::ID(v) => users::dsl::users.filter(users::id.eq(v)).select(User::as_select()).first(c).optional()?,
                LoadUserParams::PHONE_NUMBER(v) => users::dsl::users.filter(users::phone_number.eq(v)).select(User::as_select()).first(c).optional()?,
            }
        });
    }
}
