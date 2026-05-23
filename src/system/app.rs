use crate::database::AppState;
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
pub struct AppRecord {
    pub unique_id: Option<String>,
    pub title: String,
}

pub async fn get_apps(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AppRecord>("SELECT unique_id, title FROM apps WHERE owner_id = $1")
        .bind(user_id)
        .fetch_all(&state.pool)
        .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(json!({
                "message": "Apps have been fetched!",
                "data": records
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to fetch apps"
            })),
        ),
    }
}

pub async fn create_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Json(body): Json<AppRecord>,
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

    let app_id = match sqlx::query_scalar::<_, String>(
        "INSERT INTO apps (owner_id, unique_id, title) VALUES ($1, $2, $3) RETURNING unique_id",
    )
    .bind(user_id)
    .bind(generate_id("app_"))
    .bind(&body.title)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Something went wrong!" })),
            );
        }
    };

    if let Err(_) = sqlx::query("INSERT INTO members (member_id, app_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(app_id)
        .execute(&mut *tx)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to create member"
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
        StatusCode::CREATED,
        Json(json!({
            "message": "App has been created!"
        })),
    )
}

pub async fn get_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AppRecord>(
        "SELECT unique_id, title FROM apps WHERE unique_id = $1 AND owner_id = $2",
    )
    .bind(&uid)
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => (
            StatusCode::OK,
            Json(json!({
                "message": "App has been fetched!",
                "data": row
            })),
        ),
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "App not found!"
            })),
        ),
    }
}

pub async fn update_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(uid): Path<String>,
    Json(payload): Json<AppRecord>,
) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE apps SET title = $1, updated_at = $2 WHERE unique_id = $3 AND owner_id = $4",
    )
    .bind(&payload.title)
    .bind(chrono::Utc::now().naive_utc())
    .bind(&uid)
    .bind(&user_id)
    .execute(&state.pool)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({
                "message": format!("App {} title has been updated!", uid)
            })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "App not found!" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Something went wrong!" })),
        ),
    }
}

pub async fn delete_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("DELETE from apps WHERE unique_id = $1 AND owner_id = $2")
        .bind(&uid)
        .bind(&user_id)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": format!("App {} has been deleted!", uid) })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Only owner can delete the app" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Something went wrong!" })),
        ),
    }
}
