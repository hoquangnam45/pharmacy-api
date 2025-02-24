use crate::App;
use axum::extract::State;

pub enum RegisterRequestType {
    BASIC,
}

pub struct RegisterRequest {
    username: String,
    password: Option<String>,
    kind: Option<RegisterRequestType>,
}

pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn register(State(app): State<App>) {}

pub async fn login(State(app): State<App>) {}

pub async fn logout(State(app): State<App>) {}

pub async fn refresh(State(app): State<App>) {}

pub async fn admin_login(State(app): State<App>) {}

pub async fn admin_logout(State(app): State<App>) {}

pub async fn admin_refresh(State(app): State<App>) {}
