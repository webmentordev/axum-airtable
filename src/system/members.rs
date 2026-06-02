use crate::database::AppState;
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
pub struct FormData {
    pub app: String,
    pub email: String,
}

pub async fn add_member(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<FormData>,
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

    if sqlx::query("SELECT 1 FROM apps WHERE unique_id = $1 AND owner_id = $2")
        .bind(&payload.app)
        .bind(&user_id)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "App not found." })),
        );
    }

    let member_id = match sqlx::query_scalar::<_, i32>("SELECT id from users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&mut *tx)
        .await
    {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "User not found." })),
            );
        }
    };

    if sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(member_id)
        .bind(&payload.app)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None)
        .is_some()
    {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "message": "User is already a member." })),
        );
    }

    if sqlx::query("INSERT INTO members (member_id, app_id) VALUES ($1, $2)")
        .bind(member_id)
        .bind(&payload.app)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to add member." })),
        );
    }

    let _ = tx.commit().await;

    (
        StatusCode::OK,
        Json(json!({ "message": "Member has been added." })),
    )
}
