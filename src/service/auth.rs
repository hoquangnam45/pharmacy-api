use crate::repo::auth::AuthRepo;
use derive_getters::Getters;
use derive_new::new;

#[derive(new, Getters, Clone)]
pub struct AuthService {
    repo: AuthRepo,
}
