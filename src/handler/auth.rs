use axum::body::Body;
use axum::extract::State;
use axum::Json;
use crate::App;

pub struct RegisterRequest {
    username: String,
    password: String,
    email: String,
}

pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn register(State(app): State<App>, Json(register_request): Json<RegisterRequest>) {

}

pub async fn login(State(app): State<App>) {

}