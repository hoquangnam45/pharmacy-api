use crate::App;
use axum::extract::State;

pub async fn hello_world(app: State(App)) {}
