use diesel::backend::Backend;

pub struct AuthRepo<DB: Backend> {}

pub struct LoadUserParams {
    username: String,
    email: String,
    id: String
}
impl <DB: Backend> AuthRepo<DB> {
    pub fn load_user(username: String, ) ->
}
