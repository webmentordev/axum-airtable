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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct AppResponse {
    pub unique_id: Option<String>,
    pub title: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    #[sqlx(default)]
    pub members_count: i64,
    pub is_owner: bool,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Workspace {
    pub unique_id: String,
    pub title: String,
    pub position: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
pub struct SingleAppResponse {
    pub app: AppResponse,
    pub workspaces: Vec<Workspace>,
}

#[derive(Serialize, Deserialize)]
pub struct AppRecord {
    pub title: String,
}

pub async fn get_apps(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AppResponse>(
        "SELECT a.unique_id, a.title, a.created_at, a.updated_at, COUNT(m.id) as members_count,
                (a.owner_id = $1) as is_owner
         FROM apps a
         JOIN members mem ON a.unique_id = mem.app_id
         LEFT JOIN members m ON a.unique_id = m.app_id
         WHERE mem.member_id = $1
         GROUP BY a.id, a.unique_id, a.title, a.created_at, a.updated_at, a.owner_id
         ORDER BY a.created_at DESC",
    )
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
        .bind(&app_id)
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
            "message": "App has been created!",
            "data": {
                "unique_id": &app_id,
                "title": &body.title,
                "members_count": 1,
                "is_owner": true
            }
        })),
    )
}

pub async fn get_app(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AppResponse>(
        "SELECT a.unique_id, a.title, a.created_at, a.updated_at, COUNT(m.id) as members_count,
                (a.owner_id = $2) as is_owner
         FROM apps a
         JOIN members mem ON a.unique_id = mem.app_id
         LEFT JOIN members m ON a.unique_id = m.app_id
         WHERE a.unique_id = $1 AND mem.member_id = $2
         GROUP BY a.id, a.unique_id, a.title, a.created_at, a.updated_at, a.owner_id",
    )
    .bind(&uid)
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(app) => {
            let workspaces = sqlx::query_as::<_, Workspace>(
                "SELECT unique_id, title, position, created_at, updated_at
                 FROM workspaces
                 WHERE app_id = $1
                 ORDER BY position ASC",
            )
            .bind(&uid)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();
            (
                StatusCode::OK,
                Json(json!({
                    "message": "App has been fetched!",
                    "data": SingleAppResponse{
                        app,
                        workspaces
                    }
                })),
            )
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
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
            StatusCode::NOT_FOUND,
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
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Failed to delete the app." })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Something went wrong!" })),
        ),
    }
}
