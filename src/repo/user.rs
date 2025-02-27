use crate::DBConnection;
use derive_getters::Getters;
use derive_new::new;
use diesel::pg::Pg;
use diesel::sqlite::Sqlite;
use diesel::{AsChangeset, ExpressionMethods, Identifiable, Insertable, QueryDsl, Queryable, RunQueryDsl, Selectable, SelectableHelper};
use crate::schema::users::users;

#[derive(new, Clone, Getters)]
pub struct UserRepo {}

pub struct PhoneNumber {
    country_code: u16,
}

pub struct ContactedEmail {}

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Insertable)]
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
pub struct LoadUserParams {
    username: Option<String>,
    email: Option<String>,
    id: Option<String>,
    phone_number: Option<String>,
}

impl UserRepo {
    pub fn load_user(
        &mut self,
        con: &mut DBConnection,
        load_user_params: &LoadUserParams,
    ) -> Option<User> {
        let test = users::dsl::users.filter(users::username.eq()).first(con).select(User::as_select());
    }
}
