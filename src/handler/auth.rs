use crate::repo::user::LoadUserParams;
use crate::App;
use axum::extract::State;
use axum::Json;
use derive_getters::Getters;
use derive_new::new;

pub enum RegisterRequestType {
    BASIC,
}

pub struct RegisterRequest {
    username: String,
    password: Option<String>,
    r#type: Option<RegisterRequestType>,
}

pub enum LoginType {
    NATIVE,
    GOOGLE,
    FACEBOOK,
}

#[derive(new, Getters)]
pub struct LoginRequest {
    username: Option<String>,
    password: Option<String>,
    r#type: LoginType,
}

pub async fn register(State(app): State<App>) {}

pub async fn login(State(app): State<App>, Json(req): Json<LoginRequest>) {
    let user_repo = app.repos().user();
    match req.r#type() {
        LoginType::NATIVE => {
            let load_user_params = LoadUserParams::new(req.username().expect("Invalid username"),
        }
        LoginType::GOOGLE => {}
        LoginType::FACEBOOK => {}
    }
    let user = user_repo.load_user()
}

pub async fn logout(State(app): State<App>) {}

pub async fn refresh(State(app): State<App>) {}

pub async fn admin_login(State(app): State<App>) {}

pub async fn admin_logout(State(app): State<App>) {}

pub async fn admin_refresh(State(app): State<App>) {}
