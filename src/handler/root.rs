use crate::App;
use axum::extract::State;

pub async fn hello_world(State(app): State<App>) {}
