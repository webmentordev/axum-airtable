use crate::database::AppState;
use crate::utils::generate_token;
use crate::{AuthUser, utils::generate_id};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct TokenRecord {
    pub unique_id: String,
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenResponse {
    pub unique_id: String,
    pub token: String,
    pub created_at: NaiveDateTime,
}

impl TokenResponse {
    pub fn mask_token(token: &str) -> String {
        let rest = &token[0..];
        let start = &rest[..6];
        let end = &rest[rest.len() - 5..];
        format!("{}********{}", start, end)
    }
}

pub async fn get_tokens(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, (String, String, String, NaiveDateTime)>(
        "SELECT a.title, t.unique_id, t.token, t.created_at
         FROM tokens t
         JOIN apps a ON t.app_id = a.unique_id
         WHERE a.owner_id = $1 OR a.unique_id IN (
            SELECT app_id FROM members WHERE member_id = $1
         )
         ORDER BY t.created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(records) => {
            let data: Vec<_> = records
                .into_iter()
                .map(|(title, unique_id, token, created_at)| {
                    json!({
                        "title": title,
                        "unique_id": unique_id,
                        "token": TokenResponse::mask_token(&token),
                        "created_at": created_at
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "message": "Tokens have been fetched!",
                    "data": data
                })),
            )
        }
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
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Something went wrong!" })),
            );
        }
    };

    if let Err(_) = sqlx::query("SELECT app_id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_id)
        .fetch_one(&mut *tx)
        .await
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "Token not found."
            })),
        );
    }

    let token = generate_token("tk");
    let unique_id = generate_id("tk_");
    if let Err(_) = sqlx::query(
        "INSERT INTO tokens (owner_id, app_id, token, unique_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(&user_id)
    .bind(&app_id)
    .bind(&token)
    .bind(&unique_id)
    .execute(&mut *tx)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Internal server error"
            })),
        );
    }

    if let Err(_) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to commit transaction"
            })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Token has been created!",
            "data": TokenRecord{
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
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "Token not found."
            })),
        );
    }
    match sqlx::query("DELETE FROM tokens WHERE unique_id = $1 AND app_id = $2")
        .bind(&payload.unique_id)
        .bind(&app_id)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": format!("Token has been deleted!") })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Token not found." })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Internal server error!" })),
        ),
    }
}
