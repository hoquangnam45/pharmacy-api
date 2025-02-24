use crate::DBPool;
use derive_new::new;

#[derive(new, Clone)]
pub struct AuthRepo {
    pool: DBPool,
}

pub struct PhoneNumber {
    country_code: u16,
}

pub struct ContactedEmail {}

pub struct User {
    id: String,
    email: String,
    password: Option<String>,
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

pub struct LoadUserParams {
    username: Option<String>,
    email: Option<String>,
    id: Option<String>,
}

impl AuthRepo {
    pub fn load_user(&mut self) {}
}
