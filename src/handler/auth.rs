use crate::repo::user::LoadUserParams;
use crate::App;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use derive_getters::Getters;
use derive_new::new;
use std::error::Error;
use std::fmt::Debug;
use base64::alphabet::STANDARD;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use jwt::{Claims, RegisteredClaims, SignWithKey};
use crate::util::jwt::generate_token_pair;

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
    phone_number: Option<String>,
    email: Option<String>,
    password: Option<String>,
    r#type: LoginType,
}

#[derive(new)]
pub struct AppError {
    status_code: StatusCode,
    err: Box<dyn Error>,
}

impl<E: Debug> From<E> for AppError {
    fn from(value: E) -> Self {
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, value)
    }
}

impl From<(StatusCode, Box<dyn Error>)> for AppError {
    fn from(value: (StatusCode, Box<dyn Error>)) -> Self {
        AppError::new(value.0, value.1)
    }
}

pub async fn register(State(app): State<App>) {}

pub async fn login(State(app): State<App>, Json(req): Json<LoginRequest>) -> Result<(), AppError> {
    match req.r#type() {
        LoginType::NATIVE => {
            let password = match req.password() {
                None => return Err((StatusCode::BAD_REQUEST, "Missing password".to_string()).into()),
                Some(v) => v
            };
            let load_user_params = if let Some(username) = req.username() {
                LoadUserParams::USERNAME(username.to_owned())
            } else if let Some(email) = req.email() {
                LoadUserParams::EMAIL(email.to_owned())
            } else if let Some(phone_number) = req.phone_number() {
                LoadUserParams::PHONE_NUMBER(phone_number.to_string())
            } else {
                return Err((StatusCode::BAD_REQUEST, "Invalid login request").into());
            };
            let mut con = app.pool().get_con().map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
            let user = app.repos().user().load_user(&mut con, &load_user_params).map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
            if user.is_none() {
                return Err(StatusCode::UNAUTHORIZED.into());
            }
            let eq = bcrypt::verify(password, user.unwrap().password()).map_err(|e| StatusCode::UNAUTHORIZED)?;
            if !eq {
                return Err(StatusCode::UNAUTHORIZED.into());
            }
            let claims = Claims::new(app.config().jwt().access_token().clone());
            let access_token = claims.sign_with_key(app.secrets().jwt_key()).map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
            let refresh_token = BASE64_STANDARD.encode(uuid::Uuid::new_v4().to_string());

        }
        LoginType::GOOGLE => {
            // Redirect to google login page
        }
        LoginType::FACEBOOK => {
            // Redirect to facebook login page
        }
    }
    Ok(())
}

pub async fn logout(State(app): State<App>) {}

pub async fn refresh(State(app): State<App>) {}

pub async fn admin_login(State(app): State<App>) {}

pub async fn admin_logout(State(app): State<App>) {}

pub async fn admin_refresh(State(app): State<App>) {}
