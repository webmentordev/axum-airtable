use crate::AppState;

use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use bcrypt::{hash, verify};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::prelude::FromRow;
use std::env;

#[derive(Deserialize, Serialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize, FromRow)]
pub struct Login {
    pub email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct Register {
    name: String,
    username: String,
    email: String,
    password: String,
    confirm_password: String,
}

pub struct AuthUser(pub String);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = String;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or("Missing token")?;
        let secret = env::var("JWT_SECRET").unwrap();
        let key = DecodingKey::from_secret(secret.as_bytes());
        let claims =
            decode::<Claims>(token, &key, &Validation::default()).map_err(|_| "Invalid token")?;
        Ok(AuthUser(claims.claims.sub))
    }
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(Login { email, password }): Json<Login>,
) -> impl IntoResponse {
    if email.trim().is_empty() || password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Email or password is missing!"
            })),
        );
    }
    if !email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Email format is incorrect"
            })),
        );
    }

    let secret = match env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Internal server error"
                })),
            );
        }
    };

    let row = match sqlx::query_as::<_, Login>("SELECT email, password FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&state.pool)
        .await
    {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "message": "Invalid login credientials email"
                })),
            );
        }
    };

    let is_valid = verify(&password, &row.password).unwrap_or(false);
    if !is_valid {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Invalid login credientials"
            })),
        );
    }

    let key = EncodingKey::from_secret(secret.as_bytes());
    let claims = Claims {
        sub: email,
        exp: (Utc::now().timestamp() * 3600) as usize,
    };
    match encode(&Header::default(), &claims, &key) {
        Ok(token) => {
            return (
                StatusCode::OK,
                Json(json!({
                    "message": "Login success!",
                    "token": token
                })),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Internal server error"
                })),
            );
        }
    };
}

pub async fn signup_handler(
    State(state): State<AppState>,
    Json(Register {
        name,
        username,
        email,
        password,
        confirm_password,
    }): Json<Register>,
) -> impl IntoResponse {
    let password = password.trim();
    if name.trim().is_empty()
        || email.trim().is_empty()
        || username.trim().is_empty()
        || password.is_empty()
        || confirm_password.trim().is_empty()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Form data is missing."
            })),
        );
    }
    if !email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Email format is incorrect"
            })),
        );
    }
    if password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Password must be 8 chars long!"
            })),
        );
    }
    if password != confirm_password.trim() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Password & confirm password does not match"
            })),
        );
    }

    if sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE email = $1 OR username = $2")
        .bind(&email)
        .bind(&username)
        .fetch_one(&state.pool)
        .await
        .is_ok()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Username or email is already taken!"
            })),
        );
    }
    let hashed_password = match hash(password, 4) {
        Ok(password) => password,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Internal server error!"
                })),
            );
        }
    };

    if sqlx::query("INSERT into users (name, username, email, password) VALUES ($1, $2, $3, $4)")
        .bind(&name)
        .bind(&username)
        .bind(&email)
        .bind(&hashed_password)
        .execute(&state.pool)
        .await
        .is_ok()
    {
        return (
            StatusCode::CREATED,
            Json(json!({
                "message": "Account has been created!"
            })),
        );
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "message": "Internal server error!"
        })),
    )
}

pub async fn logout_handler() -> impl IntoResponse {
    return (
        StatusCode::OK,
        Json(json!({
            "message": "Logged out successfully!"
        })),
    );
}
