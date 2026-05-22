use crate::AuthUser;
use crate::Response;
use crate::database::AppState;
use crate::utils::generate_id;

use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkspaceRecord {
    pub unique_id: Option<String>,
    pub title: String,
    pub position: i32,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkspaceResponse {
    pub unique_id: String,
    pub title: String,
    pub position: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn get_workspaces(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(app_uid): Path<String>,
) -> impl IntoResponse {
    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Failed to fetch workspaces"
            })),
        );
    }

    match sqlx::query_as::<_, WorkspaceResponse>(
        "SELECT unique_id, title, position, created_at, updated_at FROM workspaces WHERE app_id = $1",
    )
    .bind(app_uid)
    .fetch_all(&state.pool)
    .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(json!({
                "message": "Workspaces have been fetched!",
                "data": records
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to fetch workspace"
            })),
        ),
    }
}

pub async fn create_workspace(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path(app_uid): Path<String>,
    Json(payload): Json<WorkspaceRecord>,
) -> impl IntoResponse {
    if payload.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Title is required"
            })),
        );
    }
    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Failed to create the workspace"
            })),
        );
    }

    let unique_id = generate_id("wrk");
    if let Err(err) =
        sqlx::query("INSERT INTO workspaces (unique_id, app_id, title) VALUES ($1, $2, $3)")
            .bind(&unique_id)
            .bind(&app_uid)
            .bind(&payload.title)
            .execute(&state.pool)
            .await
    {
        println!("{}", err.to_string());
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Failed to create the workspace"
            })),
        );
    }
    (
        StatusCode::CREATED,
        Json(json!({
            "message": "Workspace has been created!",
            "data": WorkspaceRecord {
                unique_id: Some(unique_id),
                title: payload.title.to_string(),
                position: payload.position
            }
        })),
    )
}

pub async fn update_workspace(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((app_uid, workspace_uid)): Path<(String, String)>,
    Json(payload): Json<WorkspaceRecord>,
) -> impl IntoResponse {
    if payload.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Title or position is missing"
            })),
        );
    }
    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Failed to update the workspace"
            })),
        );
    }
    match sqlx::query(
        "UPDATE workspaces SET title = $1, position = $2, updated_at = $3 WHERE unique_id = $4",
    )
    .bind(&payload.title)
    .bind(&payload.position)
    .bind(chrono::Utc::now().naive_utc())
    .bind(&workspace_uid)
    .execute(&state.pool)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Workspace has been updated" })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Workspace not found!" })),
        ),
        Err(e) => {
            println!("{}", e.to_string());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Something went wrong!" })),
            )
        }
    }
}

pub async fn delete_workspace(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Path((app_uid, workspace_uid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(_) = sqlx::query("SELECT id FROM members WHERE member_id = $1 AND app_id = $2")
        .bind(&user_id)
        .bind(&app_uid)
        .fetch_one(&state.pool)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "Failed to delete the workspace"
            })),
        );
    }

    match sqlx::query("DELETE from workspaces WHERE app_id = $1 AND unique_id = $2")
        .bind(&app_uid)
        .bind(&workspace_uid)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": format!("Workspace {} has been deleted!", workspace_uid) })),
        ),
        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "Unable to delete the workspace" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Something went wrong!" })),
        ),
    }
}
