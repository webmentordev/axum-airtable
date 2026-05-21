use crate::database::AppState;
use crate::utils::generate_token;
use crate::{AuthUser, utils::generate_id};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenRecord {
    pub unique_id: String,
    pub app_id: Option<String>,
    pub token: Option<String>,
}

pub async fn get_tokens(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(app_id): Path<String>,
) -> impl IntoResponse {
    if !sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_id)
        .fetch_one(&state.pool)
        .await
        .is_ok()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "App not found!"
            })),
        );
    }

    match sqlx::query_as::<_, TokenRecord>(
        "SELECT app_id, unique_id, token FROM tokens WHERE app_id = $1",
    )
    .bind(&app_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(json!({
                "message": "Tokens have been fetched!",
                "data": records
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to fetch tokens"
            })),
        ),
    }
}

pub async fn create_token(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(app_id): Path<String>,
) -> impl IntoResponse {
    if let Err(_) = sqlx::query("SELECT app_id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_id)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Faild to create token"
            })),
        );
    }

    let token = generate_token("tk");
    let unique_id = generate_id("tk");
    if let Err(_) = sqlx::query(
        "INSERT INTO tokens (owner_id, app_id, token, unique_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(&user_id)
    .bind(&app_id)
    .bind(&token)
    .bind(&unique_id)
    .execute(&state.pool)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Internal server error"
            })),
        );
    }
    (
        StatusCode::OK,
        Json(json!({
            "message": "Token has been created!",
            "data": TokenRecord{
                app_id: Some(app_id),
                unique_id: unique_id,
                token: Some(token)
            }
        })),
    )
}

pub async fn delete_token(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(app_id): Path<String>,
    Json(payload): Json<TokenRecord>,
) -> impl IntoResponse {
    if let Err(_) = sqlx::query("SELECT app_id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_id)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Faild to delete the token"
            })),
        );
    }
    match sqlx::query("DELETE FROM tokens WHERE unique_id = $1")
        .bind(&payload.unique_id)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": format!("Token has been deleted!") })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Token not found!" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Internal server error!" })),
        ),
    }
}
